#!/usr/bin/env python3
"""V2 : exports réels MIDI/vérité, 20 seeds, axes constants et écoutes déterministes.

Python du labo (mido). Aucun parseur ABC : --corpus relit exclusivement les
références DS-T10/DS-T10b produites par le parseur commun du laboratoire.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import tempfile
import tomllib

LAB = Path('/Users/william.quintal/Project/bascanada/harmonium_lab')
ASSETS = Path('/Users/william.quintal/Project/bascanada/harmonium/assets')
SCENARIOS = {'neutral': (0.5, 0.4), 'tension0': (0.5, 0.0), 'tension1': (0.5, 1.0),
             'arousal0': (0.0, 0.4), 'arousal1': (1.0, 0.4)}
LISTEN = {'reel_quebecois', 'gigue_quebecoise', 'valse_quebecoise'}
REFERENCES = {'reel_quebecois': 'quebecoise-reel', 'gigue_quebecoise': 'quebecoise-jig',
              'valse_quebecoise': 'quebecoise-waltz', 'reel_acadien': 'acadienne-reel',
              'marche_britannique_coloniale': 'britannique_coloniale-march',
              'gigue_britannique_coloniale': 'britannique_coloniale-jig',
              'hornpipe_britannique_coloniale': 'britannique_coloniale-hornpipe'}


def distribution(values):
    values = sorted(values)
    if not values:
        return {'n': 0}
    return {'n': len(values), 'mean': statistics.fmean(values),
            'min': values[0], 'p10': values[int((len(values)-1)*.1)],
            'median': statistics.median(values), 'p90': values[int((len(values)-1)*.9)],
            'max': values[-1]}


def midi_notes(path):
    import mido
    midi = mido.MidiFile(path)
    time = 0.0
    active, notes = {}, []
    for event in midi:
        time += event.time
        if event.type == 'note_on' and event.velocity and event.channel == 0:
            key = (event.channel, event.note)
            if key in active:
                raise ValueError(f'{path}: retrigger sans note_off {key}')
            active[key] = (time, event.velocity)
        elif event.type in ('note_on', 'note_off') and event.channel == 0:
            key = (event.channel, event.note)
            if key in active:
                start, velocity = active.pop(key)
                notes.append({'pitch': event.note, 'velocity': velocity,
                              'on_ms': start*1000, 'duration_ms': (time-start)*1000})
    notes.sort(key=lambda n: n['on_ms'])
    return notes


def measure_case(root, bars):
    truth = json.loads(root.with_suffix('.json').read_text())
    rows = []
    for entry in truth['measures']:
        measure = entry['measure']
        if measure['index'] > bars:
            continue
        for note in measure['notes']:
            if note['track'] != 0:
                continue
            p = note.get('performance', {})
            rows.append({'bar': measure['index'], 'step': note['start_step'],
                         'offset': p.get('offset', 0), 'gate': p.get('gate', 1000),
                         'breath': p.get('breath', 0), 'velocity': note['velocity'],
                         'duration': note['duration_steps'], 'pitch': note['pitch'],
                         'articulation': note.get('articulation', 'Normal'),
                         'position': measure.get('position'), 'tempo': measure['tempo'],
                         'steps': measure['steps'], 'numerator': measure['time_sig_numerator'],
                         'denominator': measure['time_sig_denominator'],
                         'steps_per_pulse': measure['pulse']['steps_per_pulse'],
                         'swing': measure['swing_ratio']})
    rows.sort(key=lambda n: (n['bar'], n['step']*1000+n['offset'], n['pitch']))
    observed = midi_notes(root.with_suffix('.mid'))
    # Le rendu coupe à la barre demandée ; ne comparer que les notes réellement
    # fermées avant cette coupe, sans fabriquer de note_off pour la mesure future.
    available = defaultdict(list)
    for row in reversed(rows):
        available[(row['pitch'], row['velocity'])].append(row)
    for midi in observed:
        candidates = available[(midi['pitch'], midi['velocity'])]
        if not candidates:
            raise ValueError(f'{root}: MIDI sans note correspondante dans la vérité: {midi}')
        row = candidates.pop()
        row['midi_duration_ms'] = midi['duration_ms']
        # Grille exacte en frames à tempo constant, même arrondi de pulse que
        # PlaybackEngine. Pas de quantification cachée du décalage musical.
        bar_frame = truth['bar_start_frames'].get(str(row['bar']))
        if bar_frame is not None:
            quarter = int(44100*60/row['tempo']+.5)
            steps_per_quarter = row['steps']*row['denominator']/(row['numerator']*4)
            if row['swing'] > 0.5 and steps_per_quarter == 4 and row['steps_per_pulse'] % 4 == 0:
                r = row['swing']
                first, second = int(r*quarter/2+.5), int((1-r)*quarter/2+.5)
                unit = [first, first, second, second]
                unit[-1] += quarter-sum(unit)
                durations = [unit[i % 4] for i in range(row['steps'])]
            else:
                durations = [int(44100*60/row['tempo']/steps_per_quarter)]*row['steps']
            grid = bar_frame + sum(durations[:row['step']])
            row['midi_offset_ms'] = midi['on_ms']-grid/44.1
    return rows


def aggregate(rows, profile):
    """Mesures des seuls tours : les intros gardent leurs notes d'arrangement."""
    rows = [n for n in rows if n['position'] and n['position']['section'] == 'Tour']
    offsets, gates, velocities, breaths, midi_offsets = (defaultdict(list) for _ in range(5))
    midi_durations, midi_residuals = defaultdict(list), []
    ends = [part.get('phrase_ends', []) for part in profile['form']['parts']]
    for n in rows:
        offsets[str(n['step'])].append(n['offset']/1000)
        gates[n['articulation']].append(n['gate']/1000)
        if 'midi_offset_ms' in n:
            midi_offsets[str(n['step'])].append(n['midi_offset_ms'])
            nominal_ms = 60000/n['tempo']/(n['steps']*n['denominator']/(n['numerator']*4))
            midi_residuals.append(n['midi_offset_ms'] - n['offset']/1000*nominal_ms)
        if 'midi_duration_ms' in n:
            midi_durations[n['articulation']].append(n['midi_duration_ms'])
        position = n['position']
        if position and position['section'] == 'Tour':
            # TunePosition bar_in_strain est 1-based ; les frontières viennent
            # du profil, jamais d'une hypothèse fixe 4+4.
            bar = position['bar_in_strain']
            part = position['part']
            if isinstance(part, dict):
                part = next(iter(part.values()))
            previous = max([0]+[end for end in ends[part] if end < bar])
            velocities[str(bar-previous)].append(n['velocity'])
        if n['breath']:
            breaths['milli_steps'].append(n['breath'])
            step_ms = 60000/n['tempo']/(n['steps']*n['denominator']/(n['numerator']*4))
            breaths['milliseconds_nominal'].append(n['breath']/1000*step_ms)
    return {'notes': len(rows), 'attack_step_fractions_by_position': {k: distribution(v) for k,v in offsets.items()},
            'midi_minus_truth_ms': distribution(midi_residuals),
            'midi_attack_ms_by_position': {k: distribution(v) for k,v in midi_offsets.items()},
            'gates_by_articulation': {k: distribution(v) for k,v in gates.items()},
            'midi_duration_ms_by_articulation': {k: distribution(v) for k,v in midi_durations.items()},
            'velocity_by_phrase_bar': {k: distribution(v) for k,v in velocities.items()},
            'breaths': {k: distribution(v) for k,v in breaths.items()},
            'attack_overall': distribution([n['offset']/1000 for n in rows]),
            'gate_overall': distribution([n['gate']/1000 for n in rows])}


def strip_performance(text):
    kept, skip = [], False
    for line in text.splitlines(keepends=True):
        heading = re.match(r'^\[\[?([^\]\n]+)\]\]?\s*(?:#.*)?$', line)
        if heading:
            section = heading.group(1)
            skip = section == 'performance' or section.startswith('performance.')
        if not skip:
            kept.append(line)
    return ''.join(kept)


def measure_corpus(lab, profiles):
    """Conditionnelles conjointes, sans second parseur ni arrondi musical."""
    from fractions import Fraction
    import sys
    sys.path.insert(0, str(lab/'src'))
    from harmonium_lab.trad.ornaments import extract_ornaments, PROFILES, CORPORA
    result = {}
    for name, (tradition, tune_type) in PROFILES.items():
        profile = profiles[name]
        meter = profile['meter']
        factor = Fraction(meter['steps_per_bar']*meter['time_signature']['denominator'],
                          8*meter['time_signature']['numerator'])
        groups = defaultdict(lambda: {'notes': 0, 'Linked': 0, 'Detached': 0, 'Accent': 0})
        excluded = 0
        inline_tunes, inline_notes = 0, 0
        root = lab/'corpus'/CORPORA[tradition]
        entries = [json.loads(line) for line in (root/'manifest.jsonl').read_text().splitlines() if line.strip()]
        def group(row):
            step = Fraction(row['onset_eighths'])*factor
            duration = Fraction(row['duration_eighths'])*factor
            if step.denominator != 1 or duration.denominator != 1 or not (0 <= step < meter['steps_per_bar']) or not (1 <= duration <= meter['steps_per_bar']):
                return None
            previous = row['previous_interval']
            context = 'any' if step == 0 else ('repeated' if previous == 0 else 'ascending' if previous is not None and previous > 0 else 'descending' if previous is not None else None)
            if context is None:
                return None
            length = 'short' if duration <= 2 else 'long'
            return f'{step}|{length}|{context}'
        for entry in entries:
            if entry['type'] != tune_type:
                continue
            data = extract_ornaments((root/'tunes'/tune_type/(entry['id']+'.abc')).read_text())
            if data['inline_changes']:
                inline_tunes += 1
                inline_notes += len(data['notes'])
                continue
            for row in data['notes']:
                key = group(row)
                if key is None:
                    excluded += 1
                else:
                    groups[key]['notes'] += 1
            for event in data['articulations']:
                key = group(event)
                kind = {'slur':'Linked','tie':'Linked','tenuto':'Linked',
                        'staccato':'Detached','accent':'Accent','marcato':'Accent'}.get(event['type'])
                if key is not None and kind is not None:
                    groups[key][kind] += 1
        result[name] = {'excluded_unrepresentable_or_unknown_context': excluded,
                        'excluded_inline_tunes': inline_tunes,
                        'excluded_inline_notes': inline_notes,
                        'groups': dict(sorted(groups.items())),
                        'method': 'Positions et durées exactes sur la grille, croches ABC ; short=1..2 pas, long=3..grille ; pas 0 : contexte regroupé, autres : intervalle précédent. Liaisons/ties/tenuto → Linked, points → Detached, accent/marcato → Accent. Une marque de liaison = porteuse initiale, pas toutes les notes du groupe.'}
    return result


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--stage', choices=['before','after'])
    parser.add_argument('--corpus-only', action='store_true')
    parser.add_argument('--output', type=Path, default=Path('/tmp/harmonium-perf'))
    parser.add_argument('--profiles-root', type=Path, default=Path('profiles'))
    parser.add_argument('--lab', type=Path, default=LAB)
    parser.add_argument('--seeds', type=int, default=20)
    parser.add_argument('--bars', type=int, default=34)
    parser.add_argument('--jobs', type=int, default=6)
    parser.add_argument('--reuse', action='store_true')
    args = parser.parse_args()
    profiles = {name: tomllib.loads((args.profiles_root/name/'profile.toml').read_text()) for name in REFERENCES}
    if args.corpus_only:
        args.output.mkdir(parents=True, exist_ok=True)
        destination = args.output/'corpus-performance.json'
        destination.write_text(json.dumps(measure_corpus(args.lab,profiles),ensure_ascii=False,indent=2)+'\n')
        print(destination)
        return
    if args.binary is None or args.stage is None:
        parser.error('--binary et --stage requis hors --corpus-only')
    binary = args.binary.resolve()
    output = (args.output/args.stage).resolve()
    output.mkdir(parents=True, exist_ok=True)
    corpus = {name: json.loads((args.lab/'references/ornaments'/f'{key}.json').read_text()) for name,key in REFERENCES.items()}
    (output/'corpus.json').write_text(json.dumps(corpus, ensure_ascii=False, indent=2)+'\n')
    with tempfile.TemporaryDirectory(prefix='harmonium-perf-') as temporary:
        temporary = Path(temporary)
        for name in REFERENCES:
            shutil.copytree(args.profiles_root/name, temporary/name)
            if args.stage == 'before':
                profile = temporary/name/'profile.toml'
                profile.write_text(strip_performance(profile.read_text()))
        for scenario, (arousal, tension) in SCENARIOS.items():
            (temporary/f'{scenario}.toml').write_text(f'[[keys]]\nbar = 1\narousal = {arousal}\nvalence = 0.3\ndensity = 0.2\ntension = {tension}\n')
        cases = [(name, scenario, seed) for name in REFERENCES for scenario in SCENARIOS for seed in range(1,args.seeds+1)]
        def run(case):
            name, scenario, seed = case
            root = output/f'{name}-{scenario}-{seed:02}'
            wav = name in LISTEN and scenario == 'neutral' and seed == 3
            command = [str(binary), '--export', '--profile', str(temporary/name), '--seed', str(seed), '--bars', str(args.bars), '--tours', '2',
                       '--soundfont', str(ASSETS/'GeneralUser-GS.sf2'), '--feet-soundfont', str(ASSETS/'harmonium_trad.sf2'),
                       '--automation', str(temporary/f'{scenario}.toml'), '--record-midi', str(root.with_suffix('.mid')), '--record-truth', str(root.with_suffix('.json'))]
            if wav:
                command += ['--record-wav', str(root.with_suffix('.wav'))]
            if not args.reuse or not root.with_suffix('.json').exists():
                completed = subprocess.run(command, capture_output=True, text=True)
                root.with_suffix('.log').write_text(completed.stdout+completed.stderr)
                completed.check_returncode()
            if wav:
                repeat = root.with_name(root.name+'-repeat').with_suffix('.wav')
                if not args.reuse or not repeat.exists():
                    index = command.index('--record-wav')
                    repeated = command[:index]+['--record-wav', str(repeat)]
                    completed = subprocess.run(repeated, capture_output=True, text=True)
                    completed.check_returncode()
                first = hashlib.sha256(root.with_suffix('.wav').read_bytes()).hexdigest()
                second = hashlib.sha256(repeat.read_bytes()).hexdigest()
                if first != second:
                    raise ValueError(f'{name}: WAV non déterministe')
            return name, scenario, measure_case(root,args.bars)
        combined = defaultdict(list)
        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            for i, (name, scenario, rows) in enumerate(pool.map(run,cases),1):
                combined[(name,scenario)].extend(rows)
                print(f'{args.stage} {i}/{len(cases)} {name} {scenario}', flush=True)
    result = {name: {scenario: aggregate(combined[(name,scenario)], profiles[name]) for scenario in SCENARIOS} for name in REFERENCES}
    result['_method'] = {'seeds': args.seeds, 'bars': args.bars, 'melody_only': True,
                         'attack_unit': 'fraction du pas publié ; MIDI en millisecondes mesurées, précision du recorder',
                         'breath_ms': 'conversion nominale sans swing ; milli_steps est la vérité exacte',
                         'wav_determinism': 'SHA-256 identiques pour les 3 profils écoutables, seed 3'}
    (output/'measurements.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(output/'measurements.json')

if __name__ == '__main__':
    main()
