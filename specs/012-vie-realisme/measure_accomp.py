#!/usr/bin/env python3
"""V5b : passages, cordes et conflits sur le flux publié ; rendus GM + pieds hors dépôt.

  .venv/bin/python specs/012-vie-realisme/measure_accomp.py --binary target/release/harmonium-cli rates
  ... peaks                     # 5 seeds x 7 profils, 66 mesures, toutes émotions à 1
  ... wav --stage after         # trois profils, seed 3, 34 mesures, deux tours

`rates` compare au binaire et aux profils de main 649c684 conservés avant la migration
(--before-binary, --before-profiles). Pour reproduire : compiler ce commit dans un checkout
séparé et passer ces deux chemins. Aucun gain, arrangement ni normalisation de mesure caché.
La part de passage est mesurée parmi les transitions entre cibles de basse distinctes ;
les frontières de mesure sont aussi comptées séparément (cible à la mesure suivante).
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import math
from pathlib import Path
import tempfile
import tomllib

from measure_ornaments import (REFERENCES, LISTEN, export, automation_file, samples,
                               run_peaks, AROUSALS)


def read_measures(path, bars):
    return {e['measure']['index']: e['measure'] for e in json.loads(path.read_text())['measures']
            if e['measure']['index'] <= bars and e['measure'].get('position', {}).get('section') == 'Tour'}


def conflict(m, bass, step):
    at = step * 1000
    for n in m['notes']:
        if n['track'] != 0:
            continue
        p = n.get('performance', {})
        start = n['start_step'] * 1000 + p.get('offset', 0)
        duration = max(1, max(1, n['duration_steps']) * p.get('gate', 1000) - p.get('breath', 0))
        if start <= at < start + duration and (n['pitch'] - bass) % 12 in (1, 6, 11):
            return True
    return False


def analyze(measures, profile):
    acc = profile.get('accompaniment')
    if not acc:
        return {'transitions': 0, 'with_passing': 0, 'boundary_transitions': 0,
                'boundary_with_passing': 0, 'passing_notes': 0, 'strong_notes': 0,
                'strong_conflicts': 0, 'passing_conflicts': 0, 'intervals_to_target': {},
                'gaps_ms': [], 'spreads_ms': [], 'order_errors': 0,
                'target_intervals': {}, 'bass_intervals': {}, 'up_strokes': 0,
                'down_strokes': 0, 'offset_errors': 0}
    bass_data = acc['bass'] if 'bass' in acc else {'hits': [{'step': s} for s in acc['bass_steps']]}
    hits = {h['step'] for h in bass_data['hits']}
    result = Counter()
    gaps, spreads, intervals = [], [], Counter()
    target_intervals, bass_intervals = Counter(), Counter()
    for bar, m in measures.items():
        bass = [n for n in m['notes'] if n['track'] == 2]
        anchors = [n for n in bass if n['start_step'] in hits]
        passing = [n for n in bass if n['start_step'] not in hits]
        result['passing_notes'] += len(passing)
        for n in anchors:
            if n['start_step'] in acc['strong_steps']:
                result['strong_notes'] += 1
                result['strong_conflicts'] += conflict(m, n['pitch'], n['start_step'])
        for n in passing:
            result['passing_conflicts'] += conflict(m, n['pitch'], n['start_step'])
        nxt = measures.get(bar + 1)
        next_anchors = [] if not nxt else [n for n in nxt['notes'] if n['track'] == 2 and n['start_step'] in hits]
        next_bass = [] if not nxt else [n for n in nxt['notes'] if n['track'] == 2]
        line = bass + next_bass[:1]
        bass_intervals.update(b['pitch'] - a['pitch'] for a, b in zip(line, line[1:]))
        targets = [(n['start_step'], n['pitch'], False) for n in anchors[1:]]
        if next_anchors:
            targets.append((m['steps'] + next_anchors[0]['start_step'], next_anchors[0]['pitch'], True))
        for source, (at, pitch, boundary) in zip(anchors, targets):
            target_intervals[pitch - source['pitch']] += 1
            if source['pitch'] == pitch:
                continue
            run = [n for n in passing if source['start_step'] < n['start_step'] < at]
            result['transitions'] += 1
            result['with_passing'] += bool(run)
            result['boundary_transitions'] += boundary
            result['boundary_with_passing'] += boundary and bool(run)
            if run:
                intervals[pitch - run[-1]['pitch']] += 1
        groups = defaultdict(list)
        for n in m['notes']:
            if n['track'] == 1:
                groups[n['start_step']].append(n)
        for group in groups.values():
            if len(group) < 2:
                continue
            offsets = [n.get('performance', {}).get('offset', 0) for n in group]
            pitches = [n['pitch'] for n in group]
            result['offset_errors'] += any(abs(offset) > 490 for offset in offsets)
            if offsets[-1] != offsets[0]:
                result['down_strokes' if pitches[0] < pitches[1] else 'up_strokes'] += 1
                result['order_errors'] += not (all(a < b for a, b in zip(offsets, offsets[1:])) and
                    (all(a < b for a, b in zip(pitches, pitches[1:])) or all(a > b for a, b in zip(pitches, pitches[1:]))))
                step_ms = 60000 / m['tempo'] * profile['meter']['time_signature']['numerator'] * 4 / profile['meter']['time_signature']['denominator'] / m['steps']
                gaps.extend((b - a) * step_ms / 1000 for a, b in zip(offsets, offsets[1:]))
                spreads.append((max(offsets) - min(offsets)) * step_ms / 1000)
    return {**{key: result[key] for key in ('transitions', 'with_passing', 'boundary_transitions',
        'boundary_with_passing', 'passing_notes', 'strong_notes', 'strong_conflicts', 'passing_conflicts',
        'order_errors', 'up_strokes', 'down_strokes', 'offset_errors')},
        'intervals_to_target': dict(intervals), 'target_intervals': dict(target_intervals),
        'bass_intervals': dict(bass_intervals), 'gaps_ms': gaps, 'spreads_ms': spreads}


def run_rates(args):
    output = (args.output / 'rates').resolve()
    output.mkdir(parents=True, exist_ok=True)
    profiles = {name: tomllib.loads((args.profiles_root / name / 'profile.toml').read_text()) for name in REFERENCES}
    before_profiles = {name: tomllib.loads((args.before_profiles / name / 'profile.toml').read_text()) for name in REFERENCES}
    rows = defaultdict(lambda: defaultdict(dict))
    with tempfile.TemporaryDirectory(prefix='harmonium-accomp-rates-') as temp:
        temp = Path(temp)
        automations = {label: automation_file(temp, label, a, f'[[keys]]\nbar = 1\narousal = {a}\nvalence = 0.3\ndensity = 0.5\ntension = 0.4\n') for label, a in AROUSALS.items()}
        def run(case):
            name, label, seed = case
            pair = []
            for stage, binary, profiles_root in [('before', args.before_binary, args.before_profiles), ('after', args.binary, args.profiles_root)]:
                root = output / f'{name}-{label}-{seed}-{stage}'
                if not args.reuse or not root.with_suffix('.json').exists():
                    export(binary.resolve(), (profiles_root / name).resolve(), seed, args.bars, automations[label], root)
                pair.append(read_measures(root.with_suffix('.json'), args.bars))
            for bar in pair[0].keys() & pair[1].keys():
                for track in (0, 3, 4):
                    a = [n for n in pair[0][bar]['notes'] if n['track'] == track]
                    b = [n for n in pair[1][bar]['notes'] if n['track'] == track]
                    if a != b:
                        raise ValueError(f'{name}/{label}/{seed}/m{bar}: piste voisine {track} modifiée')
            return name, label, [analyze(pair[0], before_profiles[name]), analyze(pair[1], profiles[name])]
        cases = [(name, label, seed) for name in REFERENCES for label in AROUSALS for seed in range(1, args.seeds + 1)]
        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            for name, label, pair in pool.map(run, cases):
                for stage, result in zip(('before', 'after'), pair):
                    row = rows[name][label].setdefault(stage, {'counts': Counter(), 'intervals': Counter(), 'target_intervals': Counter(), 'bass_intervals': Counter(), 'gaps': [], 'spreads': []})
                    row['counts'].update({k: v for k, v in result.items() if isinstance(v, int)})
                    row['intervals'].update(result['intervals_to_target'])
                    row['target_intervals'].update(result['target_intervals'])
                    row['bass_intervals'].update(result['bass_intervals'])
                    row['gaps'].extend(result['gaps_ms']); row['spreads'].extend(result['spreads_ms'])
    report = {}
    print('| Profil | Arousal | Transitions avec passage | Frontières avec passage | Conflits forts avant → après | Écarts cordes ms min–max |')
    print('|---|---:|---:|---:|---:|---:|')
    for name in REFERENCES:
        report[name] = {}
        for label, arousal in AROUSALS.items():
            pair = rows[name][label]
            for row in pair.values():
                c = row['counts']
                row['passing_fraction'] = c['with_passing'] / max(1, c['transitions'])
                row['boundary_fraction'] = c['boundary_with_passing'] / max(1, c['boundary_transitions'])
                row['strong_conflict_fraction'] = c['strong_conflicts'] / max(1, c['strong_notes'])
                for key in ('gaps', 'spreads'):
                    values = row[key]
                    row[key + '_ms'] = {'min': min(values), 'max': max(values), 'mean': sum(values) / len(values)} if values else None
                    del row[key]
            before, after = pair['before'], pair['after']
            if after['strong_conflict_fraction'] > before['strong_conflict_fraction'] + 1e-12:
                raise ValueError(f'{name}/{label}: conflits forts supérieurs à main')
            if any(after['counts'][key] for key in ('passing_conflicts', 'order_errors', 'offset_errors')):
                raise ValueError(f'{name}/{label}: conflit de passage ou ordre de cordes invalide')
            gap = after['gaps_ms']
            gap_text = f"{gap['min']:.2f}–{gap['max']:.2f}" if gap else '—'
            print(f"| {name} | {arousal:g} | {after['passing_fraction']:.1%} | {after['boundary_fraction']:.1%} | {before['strong_conflict_fraction']:.1%} → {after['strong_conflict_fraction']:.1%} | {gap_text} |", flush=True)
            report[name][label] = pair
    (output / 'measurements.json').write_text(json.dumps(report, indent=2) + '\n')


def run_wav(args):
    directory = args.output / args.stage
    directory.mkdir(parents=True, exist_ok=True)
    rows = {}
    with tempfile.TemporaryDirectory(prefix='harmonium-accomp-wav-') as temp:
        auto = automation_file(Path(temp), 'neutral', 0.5,
            '[[keys]]\nbar = 1\narousal = 0.5\nvalence = 0.3\ndensity = 0.5\ntension = 0.4\n')
        for name in LISTEN:
            root = (directory / name).resolve()
            export(args.binary.resolve(), (args.profiles_root / name).resolve(), 3, 34, auto, root, wav=True, midi=True)
            x = samples(root.with_suffix('.wav'))
            rows[name] = {'sha256': hashlib.sha256(root.with_suffix('.wav').read_bytes()).hexdigest(),
                'peak_dbfs': 20 * math.log10(abs(x).max()), 'rms_dbfs': 20 * math.log10((x*x).mean()**0.5)}
            print(name, rows[name], flush=True)
    (directory / 'wav.json').write_text(json.dumps(rows, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('mode', choices=['rates', 'peaks', 'wav'])
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--profiles-root', type=Path, default=Path('profiles'))
    parser.add_argument('--before-binary', type=Path, default=Path('/tmp/harmonium-accomp/baseline/harmonium-cli'))
    parser.add_argument('--before-profiles', type=Path, default=Path('/tmp/harmonium-accomp/baseline/profiles'))
    parser.add_argument('--output', type=Path, default=Path('/tmp/harmonium-accomp'))
    parser.add_argument('--seeds', type=int, default=20)
    parser.add_argument('--bars', type=int, default=34)
    parser.add_argument('--jobs', type=int, default=6)
    parser.add_argument('--stage', choices=['before', 'after'], default='after')
    parser.add_argument('--reuse', action='store_true')
    args = parser.parse_args()
    {'rates': run_rates, 'peaks': run_peaks, 'wav': run_wav}[args.mode](args)


if __name__ == '__main__':
    main()
