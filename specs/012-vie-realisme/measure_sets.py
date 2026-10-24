#!/usr/bin/env python3
"""V7c : sets complets générés (tonalités, raccords, signatures, tempo, énergie),
comparés aux tableaux du §3 de l'étude (`forme-longue-etude.md`).

Python standard (tomllib) ; binaire `--release` de l'exemple `set_metrics` du moteur :

    cargo run --release -p harmonium_core --example set_metrics -- /tmp/sets.json 500 16
    python3 specs/012-vie-realisme/measure_sets.py --metrics /tmp/sets.json
    python3 specs/012-vie-realisme/measure_sets.py            # lance l'exemple (500 graines, 16 airs)

Par profil distribué : 500 sessions de 16 airs à valence neutre, 0,6 et −0,6, un air par `position` publiée
à sa première mesure (l'intro, avant tout changement de mode). Seuls les **trois premiers sets** de chaque
session servent aux longueurs et aux transitions : leurs longueurs sont indépendantes et identiquement
distribuées, donc sans biais de troncature (trois sets de 5 airs tiennent dans 16 airs).

Le script recalcule en Python indépendant du moteur les candidats, les classes et les poids,
avec les retours de matériel dans leur fenêtre inter-sets. Les probabilités sont conditionnées
par l'état réellement entendu avant chaque tirage ; la chaîne sans signature reste la borne
à tirages indépendants. Tempo et énergie (vélocités et attaques par mesure, comme l'étude §7)
sont mesurés sur les trois premiers sets complets, jamais sur un air tronqué.
`--render --binary target/release/harmonium-cli` rend deux sets complets par profil avec
GeneralUser-GS + pieds, sans bpm explicite (numpy requis seulement pour les WAV).
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import json
import math
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]
CLASSES = ['same', 'parallel', 'relative', 'fifth_up', 'fourth_up', 'step_up', 'step_down', 'other']
LABELS = ['même tonalité', 'même tonique, autre mode', 'relatif', 'tonique +5ᵗᵉ', 'tonique +4ᵗᵉ',
          'tonique +1 ton', 'tonique −1 ton', 'autre']
COLLECTION = {'ionian': 0, 'dorian': 2, 'mixolydian': 7, 'aeolian': 9}
MINOR = {'dorian', 'aeolian'}
SETS_USED = 3

# Étude §3.2 : parts observées dans The Session (reel → reel, jig → jig, tous types) ; §3.3 : pistes
# enregistrées des artistes acadiens (n = 205). Pourcentages.
REAL = {
    'reel → reel (The Session, 80 553)': [20.9, 7.8, 22.2, 15.5, 16.0, 5.6, 4.0, 7.8],
    'jig → jig (The Session, 51 672)': [21.2, 8.0, 23.2, 15.8, 17.0, 4.6, 4.0, 6.2],
    'tous types (The Session, 198 985)': [23.9, 7.8, 20.7, 15.2, 16.2, 5.5, 4.3, 6.6],
    'artistes acadiens enregistrés (205)': [41.5, 24.9, 3.9, 4.9, 5.9, 5.9, 4.9, 8.3],
}
REAL_COLUMN = {
    'reel_quebecois': 'reel → reel (The Session, 80 553)',
    'gigue_quebecoise': 'jig → jig (The Session, 51 672)',
    'gigue_britannique_coloniale': 'jig → jig (The Session, 51 672)',
    'reel_acadien': 'artistes acadiens enregistrés (205)',
    'hornpipe_britannique_coloniale': 'tous types (The Session, 198 985)',
    'valse_quebecoise': 'tous types (The Session, 198 985)',
}
# Étude §3.2 : longueurs des sets de 2 airs ou plus (n = 105 959), repliées sur 5.
REAL_LENGTH = {2: 45.2, 3: 44.3, 4: 6.8, 5: 3.8}


# ---------------------------------------------------------------- profils

def curve(points, x):
    if x <= points[0][0]:
        return points[0][1]
    for (x0, y0), (x1, y1) in zip(points, points[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0)
    return points[-1][1]


def load_profile(directory):
    profile = tomllib.loads((directory / 'profile.toml').read_text())
    files = profile.get('files', {})
    dynamics = tomllib.loads((directory / files['dynamics']).read_text())
    set_toml = tomllib.loads((directory / files['set']).read_text())
    return {
        'id': profile['meta']['id'],
        'key_modes': [(c['key'], c['mode'], float(c['weight'])) for c in profile['melody']['key_modes']],
        'color': [(c['key'], c['mode'], c['weight']) for c in dynamics['color']],
        'neutral': dynamics['neutral_valence'],
        'range': dynamics['valence_range'],
        'length': [(e['tunes'], e['weight']) for e in set_toml.get('length', [])],
        'factors': {name: set_toml.get('key', {}).get(name, 1.0) for name in CLASSES + ['last_dark']},
        'signature': set_toml.get('signature', {'return_rate': 0.0, 'memory': 0}),
        'transitions': set_toml.get('transitions', [{'kind': 'direct', 'weight': 1}]),
        'ramp': set_toml.get('ramp'),
        'meter': profile['meter'],
        'dynamics': dynamics,
        'bars_per_tour': sum(len(profile['form']['parts'][part]['roles']) for part in profile['form']['sequence']),
    }


def candidates(profile, valence):
    if valence == profile['neutral']:
        return [(k, m, w) for k, m, w in profile['key_modes']]
    low, high = profile['range']
    x = min(1.0, max(0.0, (valence - low) / (high - low)))
    return [(k, m, max(0.0, curve(points, x))) for k, m, points in profile['color']]


def classify(prev, nxt):
    (pk, pm), (nk, nm) = prev, nxt
    if pk == nk:
        return 'same' if pm == nm else 'parallel'
    if (pk - COLLECTION[pm]) % 12 == (nk - COLLECTION[nm]) % 12:
        return 'relative'
    return {7: 'fifth_up', 5: 'fourth_up', 2: 'step_up', 10: 'step_down'}.get((nk - pk) % 12, 'other')


# ---------------------------------------------------------------- lois attendues

def probabilities(factors, prev, cands, last):
    """Loi du candidat suivant : poids × classe × last_dark, en retirant la classe puis last_dark si tout tombe à 0."""
    def dark(c):
        return factors['last_dark'] if last and c[1] in MINOR else 1.0
    stages = [
        [w * (factors[classify(prev, (k, m))] if prev else 1.0) * dark((k, m, w)) for k, m, w in cands],
        [w * dark((k, m, w)) for k, m, w in cands],
        [w for _, _, w in cands],
    ]
    for weights in stages:
        total = sum(weights)
        if total > 0:
            return [w / total for w in weights]
    raise ValueError('aucun candidat')


def expected(profile, valence, use_factors=True):
    """Parts de classes et part « même tonalité » par transition d'un set, selon la loi de longueur du profil."""
    factors = profile['factors'] if use_factors else {name: 1.0 for name in CLASSES + ['last_dark']}
    cands = candidates(profile, valence)
    lengths = profile['length']
    weight_total = sum(w for _, w in lengths)
    shares = defaultdict(float)
    transitions = 0.0
    for tunes, weight in lengths:
        p_length = weight / weight_total
        distribution = probabilities(factors, None, cands, False)
        for step in range(2, tunes + 1):
            last = step == tunes
            following = [0.0] * len(cands)
            for i, p_prev in enumerate(distribution):
                if p_prev == 0:
                    continue
                prev = (cands[i][0], cands[i][1])
                row = probabilities(factors, prev, cands, last)
                for j, p in enumerate(row):
                    following[j] += p_prev * p
                    shares[classify(prev, (cands[j][0], cands[j][1]))] += p_length * p_prev * p
            transitions += p_length
            distribution = following
    return {name: shares[name] / transitions for name in CLASSES} if transitions else None


# ---------------------------------------------------------------- mesures

def eligible_sources(session, index, profile, valence):
    set_index, rank, _, _, _ = session['airs'][index]
    if rank == 1:
        return []
    memory = profile['signature']['memory']
    previous = [i for i in range(index) if session['airs'][i][0] < set_index][-memory:] if memory else []
    current = {session['materials'][i] for i in range(index) if session['airs'][i][0] == set_index}
    allowed = {(k, m) for k, m, w in candidates(profile, valence) if w > 0}
    return [i for i in reversed(previous)
            if tuple(session['airs'][i][3:]) in allowed and session['materials'][i] not in current]


def conditional_expected(sessions, profile, valence):
    """Probabilité de chaque classe avant le tirage, retours éligibles compris."""
    cands = candidates(profile, valence)
    shares, count = Counter(), 0
    for session in sessions:
        for i, (set_index, rank, length, _, _) in enumerate(session['airs']):
            if set_index >= SETS_USED or rank == 1:
                continue
            prev = tuple(session['airs'][i - 1][3:])
            heard = (prev[0], session['details'][i - 1]['last_mode'])
            normal = probabilities(profile['factors'], heard, cands, rank == length)
            sources = eligible_sources(session, i, profile, valence)
            rate = profile['signature']['return_rate'] if sources else 0.0
            for (key, mode, _), probability in zip(cands, normal):
                shares[classify(prev, (key, mode))] += (1 - rate) * probability
            for source in sources:
                shares[classify(prev, tuple(session['airs'][source][3:]))] += rate / len(sources)
            count += 1
    return {name: shares[name] / count for name in CLASSES} if count else None


def sets_of(session):
    """Les `SETS_USED` premiers sets d'une session : listes d'airs `(rang, longueur, tonalité)`."""
    sets = []
    for set_index, rank, length, key, mode in session['airs']:
        if set_index >= SETS_USED:
            break
        if rank == 1:
            sets.append([])
        sets[-1].append((rank, length, (key, mode)))
    return sets


def interval(p, n):
    return 1.96 * math.sqrt(p * (1 - p) / n) if n else float('nan')


def report(rows, profiles):
    failures = []
    for profile in profiles:
        name = profile['id']
        mine = [r for r in rows if r['style'] == name]
        print(f'\n### {name}\n')
        neutral = [r for r in mine if r['valence'] == profile['neutral']]

        lengths = Counter(s[0][1] for r in neutral for s in sets_of(r))
        total = sum(lengths.values())
        configured = {t: w for t, w in profile['length']}
        weight_total = sum(configured.values())
        print(f'Longueur des sets (airs), {SETS_USED} premiers sets de {len(neutral)} sessions, n = {total} :\n')
        print('| Airs | Configuré % | Généré % (IC 95 %) | The Session % |')
        print('|---:|---:|---:|---:|')
        for tunes in sorted(set(configured) | set(lengths)):
            p = configured.get(tunes, 0) / weight_total
            q = lengths[tunes] / total
            print(f'| {tunes} | {100 * p:.1f} | {100 * q:.1f} ± {100 * interval(p, total):.1f} | '
                  f'{REAL_LENGTH.get(tunes, "—") if len(configured) > 1 else "—"} |')
            if abs(q - p) > 4 * math.sqrt(p * (1 - p) / total) + 1e-12:
                failures.append(f'{name}: longueur {tunes} : {100 * q:.1f} % contre {100 * p:.1f} %')

        print('\nClasses de transition de tonalité dans un set (valence neutre) :\n')
        observed, n = Counter(), 0
        for r in neutral:
            for s in sets_of(r):
                for a, b in zip(s, s[1:]):
                    observed[classify(a[2], b[2])] += 1
                    n += 1
        if n == 0:
            print('Aucune transition : chaque air est son propre set.')
        else:
            rule = conditional_expected(neutral, profile, profile['neutral'])
            independent = expected(profile, profile['neutral'], use_factors=False)
            real = REAL.get(REAL_COLUMN.get(name, ''), [None] * 8)
            print(f'n = {n} transitions.\n')
            print('| Classe | Réel % | Indépendance % | Règle avec retours % | Généré % (IC 95 %) | Généré / indép. | Facteur |')
            print('|---|---:|---:|---:|---:|---:|---:|')
            for i, cls in enumerate(CLASSES):
                q = observed[cls] / n
                ratio = f'{q / independent[cls]:.2f}' if independent[cls] > 0 else '—'
                real_text = f'{real[i]:.1f}' if real[i] is not None else '—'
                print(f'| {LABELS[i]} | {real_text} | {100 * independent[cls]:.1f} | {100 * rule[cls]:.1f} | '
                      f'{100 * q:.1f} ± {100 * interval(rule[cls], n):.1f} | {ratio} | {profile["factors"][cls]:.2f} |')
                if abs(q - rule[cls]) > 4 * math.sqrt(max(rule[cls] * (1 - rule[cls]), 1e-12) / n):
                    failures.append(f'{name}: classe {cls} : {100 * q:.1f} % contre {100 * rule[cls]:.1f} % attendu')

        print('\nTonalités publiées et chaînage selon la valence (tous les airs de 16 airs, 500 sessions) :\n')
        print('| Valence | Airs | Hors candidats | Même tonalité d\'un air au suivant (set) généré % | attendu % | indépendance % |')
        print('|---:|---:|---:|---:|---:|---:|')
        for valence in sorted({r['valence'] for r in mine}, key=lambda v: (v != profile['neutral'], -v)):
            runs = [r for r in mine if r['valence'] == valence]
            allowed = {(k, m) for k, m, w in candidates(profile, valence) if w > 0}
            airs = [(k, m) for r in runs for _, _, _, k, m in r['airs']]
            outside = sum(1 for a in airs if a not in allowed)
            same = pairs = 0
            for r in runs:
                for s in sets_of(r):
                    for a, b in zip(s, s[1:]):
                        pairs += 1
                        same += a[2] == b[2]
            rule = conditional_expected(runs, profile, valence)
            independent = expected(profile, valence, use_factors=False)
            if pairs:
                print(f'| {valence:+.1f} | {len(airs)} | {outside} | {100 * same / pairs:.1f} ± '
                      f'{100 * interval(rule["same"], pairs):.1f} | {100 * rule["same"]:.1f} | {100 * independent["same"]:.1f} |')
                if abs(same / pairs - rule['same']) > 4 * math.sqrt(rule['same'] * (1 - rule['same']) / pairs):
                    failures.append(f'{name}: même tonalité à {valence}')
            else:
                print(f'| {valence:+.1f} | {len(airs)} | {outside} | — | — | — |')
            if outside:
                failures.append(f'{name}: {outside} airs hors candidats à la valence {valence}')
    return failures


def report_complete_sets(rows, profiles):
    failures, summary = [], {}
    for profile in profiles:
        name = profile['id']
        runs = [r for r in rows if r['style'] == name and r['valence'] == profile['neutral']]
        transitions, opportunities, returned, total_airs = Counter(), 0, 0, 0
        first, last = [], []
        for session in runs:
            for i, (set_index, rank, length, _, _) in enumerate(session['airs']):
                if set_index >= SETS_USED:
                    continue
                total_airs += 1
                detail = session['details'][i]
                if detail['tempo_max'] - detail['tempo_min'] > 1e-4:
                    failures.append(f'{name}: tempo variable dans un air, seed {session["seed"]}')
                intensity, lift, tempo_lift = 0.5, 0.0, 0.0
                ramp = profile['ramp']
                if ramp and length > 1:
                    p = (rank - 1) / (length - 1)
                    gate = curve(ramp['gate'], intensity)
                    lift = curve(ramp['intensity_lift'], p) * gate
                    tempo_lift = curve(ramp['tempo_bpm'], p) * gate
                dynamics, meter = profile['dynamics'], profile['meter']
                game_tempo = min(300, max(40, meter['default_bpm'] + curve(dynamics['tempo_offset'], intensity)))
                effective = min(1, intensity + lift)
                tempo = min(300, max(40, meter['default_bpm'] + curve(dynamics['tempo_offset'], effective)))
                ceiling = ramp.get('ceiling_bpm', meter['max_bpm']) if ramp else meter['max_bpm']
                if ramp and length > 1:
                    tempo = min(max(tempo, game_tempo), max(ceiling, game_tempo))
                    tempo += min(tempo_lift, max(0, ceiling - tempo))
                if abs(detail['tempo_min'] - tempo) > 1e-3:
                    failures.append(f'{name}: tempo {detail["tempo_min"]} contre {tempo}, seed {session["seed"]}')
                if rank == 1:
                    first.append(detail)
                if rank == length:
                    last.append(detail)
                if rank > 1:
                    masks = detail['intro_tracks']
                    if any(mask & 24 for mask in masks):
                        kind = 'call'
                    elif any(mask == 0 for mask in masks):
                        kind = 'silence'
                    elif session['details'][i - 1]['tail_is_outro']:
                        kind = 'held'
                    else:
                        kind = 'direct'
                    transitions[kind] += 1
                    sources = eligible_sources(session, i, profile, profile['neutral'])
                    if sources:
                        opportunities += 1
                        returned += any(session['materials'][i] == session['materials'][j] for j in sources)
            for group in sets_of(session):
                if len(group) != group[0][1]:
                    failures.append(f'{name}: set incomplet')
        n = sum(transitions.values())
        weights = Counter()
        for entry in profile['transitions']:
            weights[entry['kind']] += entry['weight']
        for kind in set(transitions) | set(weights):
            probability = weights[kind] / sum(weights.values())
            if n and abs(transitions[kind] / n - probability) > 4 * math.sqrt(probability * (1 - probability) / n) + 1e-9:
                failures.append(f'{name}: raccord {kind} hors distribution')
        rate = profile['signature']['return_rate']
        if opportunities and abs(returned / opportunities - rate) > 4 * math.sqrt(rate * (1 - rate) / opportunities):
            failures.append(f'{name}: taux de signature hors distribution')

        def energy(details, tracks):
            notes = sum(d['energy'][track][0] for d in details for track in tracks)
            velocity = sum(d['energy'][track][1] for d in details for track in tracks)
            bars = sum(d['body_bars'] for d in details)
            return {'velocity': velocity / notes if notes else 0, 'hits_per_bar': notes / bars if bars else 0}

        def endpoint(details):
            return {'tempo': sum(d['tempo_min'] for d in details) / len(details),
                    'melody': energy(details, [0]), 'chords': energy(details, [1]),
                    'feet': energy(details, [3, 4])}

        summary[name] = {'first': endpoint(first), 'last': endpoint(last), 'transitions': transitions,
                         'signature': {'eligible': opportunities, 'returned': returned, 'airs': total_airs}}
        a, b = summary[name]['first'], summary[name]['last']
        print(f'\n### Sets complets — {name}\n')
        print('| Mesure | Premier air | Dernier air |')
        print('|---|---:|---:|')
        print(f'| Tempo moyen (bpm) | {a["tempo"]:.2f} | {b["tempo"]:.2f} |')
        for track in ['melody', 'chords', 'feet']:
            print(f'| Vélocité moyenne {track} | {a[track]["velocity"]:.2f} | {b[track]["velocity"]:.2f} |')
        print(f'| Attaques de pieds / mesure | {a["feet"]["hits_per_bar"]:.2f} | {b["feet"]["hits_per_bar"]:.2f} |')
        print(f'Raccords observés : {dict(transitions)} ; pas de fréquence réelle dans le corpus symbolique.')
        if opportunities:
            print(f'Signatures : {returned}/{opportunities} occasions = {100*returned/opportunities:.2f} % '
                  f'(configuré {100*rate:.1f}, The Session 16,7 % de tous les airs) ; '
                  f'{100*returned/total_airs:.2f} % des {total_airs} airs. Premier air toujours neuf.')
        delta = 100 * (b['tempo'] / a['tempo'] - 1)
        print(f'Tempo : +{delta:.2f} % ; borne proxy Garner reels +2,6 à +5,5 %, danse constante aussi documentée.')
    return failures, summary


def split_sets(root, truth, samples):
    """Découpe aux frames de barre réellement publiées, sans normaliser ni réinitialiser le synthé."""
    import struct
    bars = [e['measure'] for e in truth['measures']]
    boundary = next(m['index'] for m in bars if m['position']['set_index'] == 1)
    frame = truth['bar_start_frames'][str(boundary)]
    paths = []
    for index, (start, end) in enumerate([(0, frame), (frame, len(samples))], 1):
        data = samples[start:end].astype('<f4').tobytes()
        header = struct.pack('<4sI4s4sIHHIIHH4sI', b'RIFF', 36 + len(data), b'WAVE', b'fmt ',
                             16, 3, 2, 44100, 44100 * 8, 8, 32, b'data', len(data))
        path = root.with_name(root.name.removesuffix('-2sets') + f'-set{index}').with_suffix('.wav')
        with path.open('wb') as output:
            output.write(header)
            output.write(data)
        paths.append(str(path))
    return paths


def render_sets(args, profiles):
    """Deux sets complets par profil, vraie API CLI, banques GM + pieds, aucun bpm explicite."""
    from measure_ornaments import samples, ASSETS
    import numpy as np
    args.output.mkdir(parents=True, exist_ok=True)
    automation = args.output / 'constant-ramp.toml'
    automation.write_text('[[keys]]\nbar = 1\nintensity = 0.5\narousal = 0.5\nvalence = 0.3\n'
                          'density = 0.7\ntension = 0.3\nfeet = 1.0\n')
    results = {}
    for profile in profiles:
        name = profile['id']
        root = args.output / f'{name}-2sets'
        preflight = args.output / f'{name}-preflight.json'
        common = [str(args.binary.resolve()), '--export', '--profile', str((args.profiles_root / name).resolve()),
                  '--seed', '3', '--tours', '2', '--automation', str(automation.resolve())]
        completed = subprocess.run(common + ['--bars', '1000', '--soundfont', str(ROOT / 'assets/test.sf2'),
                                            '--record-truth', str(preflight.resolve())], capture_output=True, text=True, check=True)
        preflight.with_suffix('.log').write_text(completed.stdout + completed.stderr)
        truth = json.loads(preflight.read_text())
        selected = [e['measure'] for e in truth['measures'] if e['measure']['position']['set_index'] < 2]
        bars = selected[-1]['index']
        command = common + ['--bars', str(bars), '--soundfont', str(ASSETS / 'GeneralUser-GS.sf2'),
                            '--feet-soundfont', str(ASSETS / 'harmonium_trad.sf2'),
                            '--record-wav', str(root.with_suffix('.wav').resolve()),
                            '--record-midi', str(root.with_suffix('.mid').resolve()),
                            '--record-truth', str(root.with_suffix('.json').resolve())]
        completed = subprocess.run(command, capture_output=True, text=True, check=True)
        root.with_suffix('.log').write_text(completed.stdout + completed.stderr)
        rendered = json.loads(root.with_suffix('.json').read_text())
        measures = [e['measure'] for e in rendered['measures']]
        assert len(measures) == bars and measures[-1]['position']['set_index'] == 1
        assert measures[-1]['position']['tune_in_set'] == measures[-1]['position']['tunes_in_set']
        assert measures[-1]['position']['tour'] == 2
        assert measures[-1]['position']['bar_in_cycle'] + 1 == profile['bars_per_tour']
        x = samples(root.with_suffix('.wav'))
        peak = max(float(x.max()), -float(x.min()))
        results[name] = {'bars': bars, 'seconds': len(x) / 44100, 'peak_dbfs': 20 * math.log10(peak),
                         'rms_dbfs': 10 * math.log10(float(np.einsum('ij,ij->', x, x)) / x.size),
                         'clipped': int(np.count_nonzero(np.abs(x) >= 1)), 'command': command}
        results[name]['sets'] = split_sets(root, rendered, x)
        print(f'{name} : 2 sets, {bars} mesures, {len(x)/44100:.1f} s, crête {results[name]["peak_dbfs"]:.2f} dBFS')
        del x
    (args.output / 'renders.json').write_text(json.dumps(results, indent=2) + '\n')
    return [f'{name}: crête de set > −1,5 dBFS' for name, r in results.items() if r['peak_dbfs'] > -1.5]


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--metrics', type=Path, help='JSON de l\'exemple set_metrics (sinon : exécute l\'exemple)')
    parser.add_argument('--seeds', type=int, default=500)
    parser.add_argument('--airs', type=int, default=16)
    parser.add_argument('--profiles-root', type=Path, default=ROOT / 'profiles')
    parser.add_argument('--render', action='store_true', help='Rendre deux sets complets par profil (numpy)')
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/harmonium-cli')
    parser.add_argument('--output', type=Path, default=Path('/tmp/harmonium-sets'))
    args = parser.parse_args()
    metrics = args.metrics
    if metrics is None:
        metrics = Path('/tmp/harmonium-sets.json')
        subprocess.run(['cargo', 'run', '--release', '-q', '-p', 'harmonium_core', '--example', 'set_metrics', '--',
                        str(metrics), str(args.seeds), str(args.airs)], cwd=ROOT, check=True)
    rows = json.loads(metrics.read_text())
    for row in rows:
        row['valence'] = round(row['valence'], 4)  # f32 sérialisé en f64
    ids = sorted({r['style'] for r in rows})
    profiles = [load_profile(args.profiles_root / name) for name in ids]
    failures = report(rows, profiles)
    complete_failures, summary = report_complete_sets(rows, profiles)
    failures.extend(complete_failures)
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    if args.render:
        failures.extend(render_sets(args, profiles))
    print()
    if failures:
        print('Écarts :')
        for failure in failures:
            print(' -', failure)
        sys.exit(1)
    print('Aucun écart : candidats, distributions, retours, paliers et bornes de tempo conformes.')


if __name__ == '__main__':
    main()
