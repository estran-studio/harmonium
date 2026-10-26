#!/usr/bin/env python3
"""V3b : taux d'ornements générés par type, par mesure de phrase et par arousal, comparés au corpus.

Python du labo (numpy, scipy, mido) avec `PYTHONPATH=<labo>/src` ; binaire `--release`.

    harmonium_lab/.venv/bin/python specs/012-vie-realisme/measure_ornaments.py \\
        --binary target/release/harmonium-cli rates
    ... wav     # rendus avant/après de trois profils (hors dépôt)
    ... peaks   # crête à émotions maximales, 5 graines x 7 profils
    ... midi    # le MIDI enregistré rejoue chaque note de la vérité

`rates` : 20 graines par profil, arousal 0 / 0,5 / 1, un export de référence sans `[ornaments]` et un
export par type activé (seul type de la section). Le flux d'un air ne dépend pas des ornements (graine
dérivée du matériel écrit), donc la différence des deux vérités isole le type : appogiature/double corde/
accord = notes ajoutées (1, 2, 1, 2 par occurrence) ; rouleau, turn, triolet, trille = porteuses dont
(hauteur, pas, durée) a disparu. Aucune étiquette n'est lue dans le flux : seul le contrat publié.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import math
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import tomllib

import numpy as np

LAB = Path('/Users/william.quintal/Project/bascanada/harmonium_lab')
ASSETS = Path('/Users/william.quintal/Project/bascanada/harmonium/assets')
AROUSALS = {'a0': 0.0, 'a5': 0.5, 'a10': 1.0}
REFERENCES = {'reel_quebecois': 'quebecoise-reel', 'gigue_quebecoise': 'quebecoise-jig',
              'valse_quebecoise': 'quebecoise-waltz', 'reel_acadien': 'acadienne-reel',
              'marche_britannique_coloniale': 'britannique_coloniale-march',
              'gigue_britannique_coloniale': 'britannique_coloniale-jig',
              'hornpipe_britannique_coloniale': 'britannique_coloniale-hornpipe',
              # R1 (phase 013) : nouveaux styles
              'gigue_acadienne': 'acadienne-jig', 'reel_britannique_coloniale': 'britannique_coloniale-reel',
              'valse_britannique_coloniale': 'britannique_coloniale-waltz'}
LISTEN = ['reel_quebecois', 'gigue_quebecoise', 'valse_quebecoise']
# Notes ajoutées par occurrence pour les types additifs ; None : porteuse réécrite.
ADDED = {'grace_single': 1, 'grace_multiple': 2, 'double_stop': 1, 'chord': 2,
         'roll': None, 'turn': None, 'triplet_repeated': None, 'trill': None}
KINDS = list(ADDED)
MAX_EMOTIONS = ('[[keys]]\nbar = 1\nintensity = 1.0\narousal = 1.0\nvalence = 0.0\n'
                'density = 1.0\ntension = 1.0\nfeet = 1.0\n')


# ---------------------------------------------------------------- profils

def sections(text):
    """Blocs (en-tête, texte) ; le premier bloc a l'en-tête None."""
    blocks, header, lines = [], None, []
    for line in text.splitlines(keepends=True):
        match = re.match(r'^\[\[?([^\]\n]+)\]\]?\s*(?:#.*)?$', line)
        if match:
            blocks.append((header, ''.join(lines)))
            header, lines = match.group(1).strip(), []
        lines.append(line)
    blocks.append((header, ''.join(lines)))
    return blocks


def is_ornament(header):
    return header is not None and (header == 'ornaments' or header.startswith('ornaments.'))


def strip_ornaments(text):
    return ''.join(body for header, body in sections(text) if not is_ornament(header))


def only_type(text, kind):
    keep = {'ornaments', f'ornaments.types.{kind}'}
    return ''.join(body for header, body in sections(text) if not is_ornament(header) or header in keep)


def write_profile(source: Path, destination: Path, transform):
    shutil.copytree(source, destination)
    profile = destination / 'profile.toml'
    profile.write_text(transform(profile.read_text()))


# ---------------------------------------------------------------- exports

def export(binary, profile_dir, seed, bars, automation, root, wav=False, midi=False):
    command = [str(binary), '--export', '--profile', str(profile_dir), '--seed', str(seed),
               '--bars', str(bars), '--tours', '2',
               '--soundfont', str(ASSETS / 'GeneralUser-GS.sf2'),
               '--feet-soundfont', str(ASSETS / 'harmonium_trad.sf2'),
               '--automation', str(automation), '--record-truth', str(root.with_suffix('.json'))]
    if midi:
        command += ['--record-midi', str(root.with_suffix('.mid'))]
    if wav:
        command += ['--record-wav', str(root.with_suffix('.wav'))]
    completed = subprocess.run(command, capture_output=True, text=True)
    root.with_suffix('.log').write_text(completed.stdout + completed.stderr)
    completed.check_returncode()
    return command


def automation_file(directory, name, arousal, text=None):
    path = directory / f'{name}.toml'
    path.write_text(text or f'[[keys]]\nbar = 1\narousal = {arousal}\nvalence = 0.3\n'
                            'density = 0.2\ntension = 0.4\n')
    return path


def melody_by_bar(truth_path, bars):
    """Mesures de tour : {index: (bar_in_strain, part, [(pitch, step, duration, offset)])}."""
    truth = json.loads(Path(truth_path).read_text())
    result = {}
    for entry in truth['measures']:
        m = entry['measure']
        position = m.get('position')
        if m['index'] > bars or not position or position['section'] != 'Tour':
            continue
        part = position['part']
        if isinstance(part, dict):
            part = next(iter(part.values()))
        notes = [(n['pitch'], n['start_step'], n['duration_steps'], n.get('performance', {}).get('offset', 0))
                 for n in m['notes'] if n['track'] == 0]
        result[m['index']] = (position['bar_in_strain'], part, position['pass'], position['tour'], notes)
    return result


# ---------------------------------------------------------------- statistiques

def poisson_interval(observed, confidence=0.95):
    from scipy.stats import chi2
    alpha = 1 - confidence
    low = 0.0 if observed == 0 else chi2.ppf(alpha / 2, 2 * observed) / 2
    high = chi2.ppf(1 - alpha / 2, 2 * (observed + 1)) / 2
    return low, high


def verdict(observed, expected):
    if expected <= 0:
        return 'ok' if observed == 0 else 'hors cible'
    if abs(observed / expected - 1) <= 0.10:
        return 'ok'
    low, high = poisson_interval(observed)
    return 'effectif faible' if low <= expected <= high else 'hors cible'


def rate_at(rule, bar):
    by_bar = rule.get('per_100_notes_by_bar', [])
    return by_bar[bar - 1] if 1 <= bar <= len(by_bar) else rule.get('written_per_100_notes', 0.0)


def unique_roots(bars):
    """Mesures dont le contenu écrit (hauteurs, positions) n'a pas encore été vu dans l'air : une
    copie exacte (CopyOf, reprise, second tour) porte les mêmes ornements et ne compte qu'une fois."""
    seen, roots = set(), []
    for index in sorted(bars):
        content = tuple((p, s) for p, s, _, _ in bars[index][4])
        if content not in seen:
            seen.add(content)
            roots.append(index)
    return roots


def occurrences(kind, base, ornamented):
    """Occurrences du type, par mesure."""
    result = {}
    for index, (bar, part, pas, tour, notes) in base.items():
        other = ornamented[index][4]
        if ADDED[kind]:
            result[index] = (len(other) - len(notes)) / ADDED[kind]
        else:
            result[index] = sum((Counter((p, s, d) for p, s, d, _ in notes)
                                 - Counter((p, s, d) for p, s, d, _ in other)).values())
    return result


def steps_per_eighth(profile):
    meter = profile['meter']
    return meter['steps_per_bar'] * meter['time_signature']['denominator'] / (8 * meter['time_signature']['numerator'])


def run_rates(args):
    binary = args.binary.resolve()
    output = (args.output / 'rates').resolve()
    output.mkdir(parents=True, exist_ok=True)
    profiles = {name: tomllib.loads((args.profiles_root / name / 'profile.toml').read_text()) for name in REFERENCES}
    corpus = {name: json.loads((args.lab / 'references/ornaments' / f'{key}.json').read_text())
              for name, key in REFERENCES.items()}
    jobs = []
    with tempfile.TemporaryDirectory(prefix='harmonium-orn-') as temporary:
        temporary = Path(temporary)
        automations = {label: automation_file(temporary, label, value) for label, value in AROUSALS.items()}
        # Durées écrites : intensité 0,5 et tension 0 sont neutres pour les courbes de durée.
        automations['w'] = automation_file(temporary, 'w', 0.5, '[[keys]]\nbar = 1\nintensity = 0.5\narousal = 0.5\n'
                                           'valence = 0.3\ndensity = 0.2\ntension = 0.0\n')
        directories = {}
        for name in REFERENCES:
            source = args.profiles_root / name
            ornaments = profiles[name].get('ornaments', {}).get('types', {})
            write_profile(source, temporary / f'{name}-base', strip_ornaments)
            directories[(name, 'base')] = temporary / f'{name}-base'
            for kind in KINDS:
                if kind in ornaments and ornaments[kind].get('enabled', True):
                    write_profile(source, temporary / f'{name}-{kind}', lambda text, kind=kind: only_type(text, kind))
                    directories[(name, kind)] = temporary / f'{name}-{kind}'
        for (name, kind), directory in directories.items():
            for label in [*AROUSALS, *(['w'] if kind == 'base' else [])]:
                for seed in range(1, args.seeds + 1):
                    jobs.append((name, kind, label, seed, directory))

        def run(job):
            name, kind, label, seed, directory = job
            root = output / f'{name}-{kind}-{label}-{seed:02}'
            if not (args.reuse and root.with_suffix('.json').exists()):
                export(binary, directory, seed, args.bars, automations[label], root)
            return job

        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            for i, _ in enumerate(pool.map(run, jobs), 1):
                if i % 200 == 0:
                    print(f'rates {i}/{len(jobs)}', flush=True)
    result = {}
    for name, profile in profiles.items():
        rules = profile.get('ornaments', {}).get('types', {})
        scale = profile.get('ornaments', {}).get('rate_scale', 1.0)
        spe = steps_per_eighth(profile)
        part_lengths = [len(part['roles']) for part in profile['form']['parts']]
        per_profile = {'rate_scale': scale, 'types': {}}
        written = {}
        for seed in range(1, args.seeds + 1):
            run_bars = melody_by_bar(output / f'{name}-base-w-{seed:02}.json', args.bars)
            written[seed] = {(tour, part, pas, bar): notes for bar, part, pas, tour, notes in run_bars.values()}
        for kind in KINDS:
            if (name, kind) not in directories:
                continue
            rule = rules[kind]
            min_steps = max(1, math.ceil(rule.get('min_carrier_eighths', 0) * spe - 1e-3))
            stats = {'corpus_floor_per_100_notes': corpus[name]['engine_proposal']['types'][kind]['written_per_100_notes'],
                     'configured_per_100_notes': rule.get('written_per_100_notes', 0.0) * scale,
                     'min_carrier_eighths': rule.get('min_carrier_eighths', 0.0), 'by_arousal': {}}
            for label in AROUSALS:
                obs = exp = notes_roots = notes_open = eligible_notes = protected_hits = 0.0
                by_bar = [[0.0, 0.0, 0] for _ in range(8)]
                for seed in range(1, args.seeds + 1):
                    base = melody_by_bar(output / f'{name}-base-{label}-{seed:02}.json', args.bars)
                    other = melody_by_bar(output / f'{name}-{kind}-{label}-{seed:02}.json', args.bars)
                    if set(base) != set(other):
                        raise ValueError(f'{name} {kind} {label} {seed}: mesures différentes')
                    counts = occurrences(kind, base, other)
                    for index in unique_roots(base):
                        bar, part, pas, tour, notes = base[index]
                        notes_roots += len(notes)
                        if bar == part_lengths[part]:
                            protected_hits += counts[index]
                            continue
                        row = by_bar[min(bar, 8) - 1]
                        row[2] += len(notes)
                        notes_open += len(notes)
                        unscaled = written[seed].get((tour, part, pas, bar), notes)
                        eligible = sum(1 for _, _, d, _ in unscaled if d >= min_steps)
                        eligible_notes += eligible
                        mult = rule.get('arousal_low_multiplier', 1.0) + (
                            rule.get('arousal_high_multiplier', 1.0) - rule.get('arousal_low_multiplier', 1.0)) * AROUSALS[label]
                        wanted = rate_at(rule, bar) * scale * mult / 100 * len(notes)
                        expected_bar = min(wanted, eligible)
                        obs += counts[index]
                        exp += expected_bar
                        row[0] += counts[index]
                        row[1] += expected_bar
                stats['by_arousal'][label] = {
                    'observed': obs, 'expected': exp, 'ratio': obs / exp if exp else None,
                    'verdict': verdict(round(obs), exp), 'notes_distinct_bars': notes_roots,
                    'notes_unprotected_distinct_bars': notes_open,
                    'eligible_share': eligible_notes / notes_open if notes_open else None,
                    'per_100_unprotected_notes': 100 * obs / notes_open if notes_open else None,
                    'occurrences_in_protected_bars': protected_hits,
                    'by_bar_observed_expected_notes': by_bar}
            per_profile['types'][kind] = stats
        result[name] = per_profile
    (output / 'measurements.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print_rates(result)
    print(output / 'measurements.json')


def print_rates(result):
    print('\n| Profil | Type | Corpus /100 | Configuré /100 | Min (croches) | Arousal | Observé | Attendu | Rapport | Verdict |')
    print('|---|---|---:|---:|---:|---|---:|---:|---:|---|')
    for name, profile in result.items():
        for kind, stats in profile['types'].items():
            for label, row in stats['by_arousal'].items():
                ratio = '—' if row['ratio'] is None else f"{row['ratio']:.2f}"
                print(f"| `{name}` | {kind} | {stats['corpus_floor_per_100_notes']:.3f} | {stats['configured_per_100_notes']:.3f} | "
                      f"{stats['min_carrier_eighths']:g} | {label} | {row['observed']:.0f} | {row['expected']:.1f} | {ratio} | {row['verdict']} |")
    print('\nPar mesure de la partie (arousal 0,5) : observé / attendu, notes de la mesure.')
    for name, profile in result.items():
        for kind, stats in profile['types'].items():
            by_bar = stats['by_arousal']['a5']['by_bar_observed_expected_notes']
            cells = ' '.join(f'm{i + 1}:{o:.0f}/{e:.1f}' for i, (o, e, _) in enumerate(by_bar))
            print(f'  {name} {kind}: {cells}')


# ---------------------------------------------------------------- WAV, crêtes, MIDI

def samples(path):
    raw = path.read_bytes()
    offset, fmt, data = 12, None, None
    while offset + 8 <= len(raw):
        name, size = struct.unpack_from('<4sI', raw, offset)
        chunk = raw[offset + 8:offset + 8 + size]
        if name == b'fmt ':
            fmt = struct.unpack_from('<HHIIHH', chunk)
            if fmt[0] == 65534 and struct.unpack_from('<I', chunk, 24)[0] == 3:
                fmt = (3, *fmt[1:])
        elif name == b'data':
            data = chunk
        offset += 8 + size + size % 2
    if fmt is None or data is None or fmt[0] != 3 or fmt[1] != 2 or fmt[5] != 32:
        raise ValueError(f'{path}: float32 stéréo requis')
    return np.frombuffer(data, dtype='<f4').reshape(-1, 2).astype(np.float64)


def run_wav(args):
    binary = args.binary.resolve()
    output = args.output.resolve()
    rows = {}
    with tempfile.TemporaryDirectory(prefix='harmonium-orn-wav-') as temporary:
        temporary = Path(temporary)
        automation = automation_file(temporary, 'neutral', 0.5)
        for stage, transform in (('before', strip_ornaments), ('after', lambda text: text)):
            (output / stage).mkdir(parents=True, exist_ok=True)
            for name in LISTEN:
                write_profile(args.profiles_root / name, temporary / f'{stage}-{name}', transform)
                root = output / stage / f'{name}-neutral-03'
                export(binary, temporary / f'{stage}-{name}', 3, args.bars, automation, root, wav=True)
                repeat = root.with_name(root.name + '-repeat')
                export(binary, temporary / f'{stage}-{name}', 3, args.bars, automation, repeat, wav=True)
                first, second = (hashlib.sha256(p.with_suffix('.wav').read_bytes()).hexdigest() for p in (root, repeat))
                if first != second:
                    raise ValueError(f'{stage} {name}: WAV non déterministe')
                x = samples(root.with_suffix('.wav'))
                rows[f'{stage}/{name}'] = {'sha256': first, 'peak_dbfs': 20 * math.log10(np.max(np.abs(x))),
                                           'rms_dbfs': 20 * math.log10(np.sqrt(np.mean(x * x)))}
                print(stage, name, rows[f'{stage}/{name}'], flush=True)
    (output / 'wav.json').write_text(json.dumps(rows, indent=2) + '\n')


def run_peaks(args):
    binary = args.binary.resolve()
    output = (args.output / 'peaks').resolve()
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='harmonium-orn-peaks-') as temporary:
        temporary = Path(temporary)
        automation = automation_file(temporary, 'max', 1.0, MAX_EMOTIONS)
        neutral = automation_file(temporary, 'neutral', 0.5)

        def run(case):
            name, seed, label = case
            root = output / f'{name}-{label}-{seed}'
            export(binary, args.profiles_root / name, seed, 66, automation if label == 'max' else neutral, root, wav=True)
            x = samples(root.with_suffix('.wav'))
            peak = float(np.max(np.abs(x)))
            root.with_suffix('.wav').unlink()
            return name, label, seed, 20 * math.log10(peak), int(np.count_nonzero(np.abs(x) >= 1))

        cases = [(name, seed, label) for name in REFERENCES for seed in range(1, 6) for label in ('max', 'neutral')]
        worst = defaultdict(lambda: {'max': -1e9, 'neutral': -1e9, 'clipped': 0})
        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            for name, label, seed, peak, clipped in pool.map(run, cases):
                worst[name][label] = max(worst[name][label], peak)
                worst[name]['clipped'] += clipped
    for name, row in worst.items():
        print(f"{name}: crête max-émotions {row['max']:.2f} dBFS, neutre {row['neutral']:.2f} dBFS, échantillons >= 1 : {row['clipped']}")
    (output / 'peaks.json').write_text(json.dumps(worst, indent=2) + '\n')
    if any(row['max'] > -1.5 for row in worst.values()):
        sys.exit('crête > -1,5 dBFS à émotions maximales')


def run_midi(args):
    import mido
    binary = args.binary.resolve()
    output = (args.output / 'midi').resolve()
    output.mkdir(parents=True, exist_ok=True)
    report = {}
    with tempfile.TemporaryDirectory(prefix='harmonium-orn-midi-') as temporary:
        temporary = Path(temporary)
        automation = automation_file(temporary, 'neutral', 0.5)
        for name in REFERENCES:
            matched = missing = 0
            for seed in range(1, 6):
                root = output / f'{name}-{seed}'
                export(binary, args.profiles_root / name, seed, args.bars, automation, root, midi=True)
                truth = json.loads(root.with_suffix('.json').read_text())
                expected = Counter()
                for entry in truth['measures']:
                    if entry['measure']['index'] <= args.bars - 2:
                        for n in entry['measure']['notes']:
                            if n['track'] == 0:
                                expected[(n['pitch'], n['velocity'])] += 1
                observed = Counter()
                for event in mido.MidiFile(root.with_suffix('.mid')):
                    if event.type == 'note_on' and event.velocity and event.channel == 0:
                        observed[(event.note, event.velocity)] += 1
                missing += sum((expected - observed).values())
                matched += sum(expected.values())
            report[name] = {'truth_notes': matched, 'missing_in_midi': missing}
            print(name, report[name], flush=True)
    (output / 'midi.json').write_text(json.dumps(report, indent=2) + '\n')


def run_corpus(args):
    """Recalcule les taux avec le code du labo (PYTHONPATH=<labo>/src) et les compare aux profils."""
    sys.path.insert(0, str(args.lab / 'src'))
    from harmonium_lab.trad.ornaments import run_ornaments
    output = (args.output / 'corpus').resolve()
    run_ornaments(args.lab / 'corpus', output, output / 'references')
    mismatches = 0
    for name, key in REFERENCES.items():
        fresh = json.loads((output / 'references' / f'{key}.json').read_text())['engine_proposal']['types']
        committed = json.loads((args.lab / 'references/ornaments' / f'{key}.json').read_text())['engine_proposal']['types']
        profile = tomllib.loads((args.profiles_root / name / 'profile.toml').read_text())['ornaments']['types']
        fresh_by_position = json.loads((output / 'references' / f'{key}.json').read_text())['by_position']['bar_position']
        for kind in KINDS:
            if fresh[kind] != committed[kind]:
                print(f'{name} {kind}: la référence versionnée du labo diffère du recalcul')
                mismatches += 1
            if kind not in profile:
                continue
            rate = profile[kind].get('written_per_100_notes', 0.0)
            if abs(rate - fresh[kind]['written_per_100_notes']) > 1e-5:
                print(f'{name} {kind}: profil {rate} / corpus {fresh[kind]["written_per_100_notes"]}')
                mismatches += 1
            bars = profile[kind].get('per_100_notes_by_bar')
            reference = fresh[kind]['per_100_notes_by_bar']
            positioned = sum(
                fresh_by_position.get(str(b), {}).get('by_type', {}).get(kind, {}).get('count', 0) for b in range(1, 9))
            shape = [reference[str(i + 1)] for i in range(8)]
            if (positioned >= 30 and sum(shape) > 0) != bool(bars):
                print(f'{name} {kind}: taux par mesure attendu si et seulement si >= 30 occurrences positionnées ({positioned})')
                mismatches += 1
            elif bars:
                level = rate / (sum(shape) / 8)
                if any(abs(bars[i] - shape[i] * level) > 1e-5 for i in range(len(bars))):
                    print(f'{name} {kind}: taux par mesure du profil différents du recalcul normalisé')
                    mismatches += 1
    print('taux des profils = recalcul du corpus' if not mismatches else f'{mismatches} écart(s)')
    if mismatches:
        sys.exit(1)


def main():
    global REFERENCES
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('mode', choices=['rates', 'wav', 'peaks', 'midi', 'corpus'])
    parser.add_argument('--profiles', nargs='*', choices=list(REFERENCES), help='sous-ensemble des profils (défaut : tous)')
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--output', type=Path, default=Path('/tmp/harmonium-ornaments'))
    parser.add_argument('--profiles-root', type=Path, default=Path('profiles'))
    parser.add_argument('--lab', type=Path, default=LAB)
    parser.add_argument('--seeds', type=int, default=20)
    parser.add_argument('--bars', type=int, default=34)
    parser.add_argument('--jobs', type=int, default=6)
    parser.add_argument('--reuse', action='store_true')
    args = parser.parse_args()
    if args.profiles:
        REFERENCES = {name: REFERENCES[name] for name in args.profiles}
    if args.mode != 'corpus' and args.binary is None:
        parser.error('--binary requis')
    {'rates': run_rates, 'wav': run_wav, 'peaks': run_peaks, 'midi': run_midi,
     'corpus': run_corpus}[args.mode](args)


if __name__ == '__main__':
    main()
