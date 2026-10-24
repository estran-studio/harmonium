#!/usr/bin/env python3
"""V7b : sets générés (longueurs, classes de transition de tonalité, tonalités hors candidats), comparés
aux tableaux du §3 de l'étude (`forme-longue-etude.md`).

Python standard (tomllib) ; binaire `--release` de l'exemple `set_metrics` du moteur :

    cargo run --release -p harmonium_core --example set_metrics -- /tmp/sets.json 500 16
    python3 specs/012-vie-realisme/measure_sets.py --metrics /tmp/sets.json
    python3 specs/012-vie-realisme/measure_sets.py            # lance l'exemple (500 graines, 16 airs)

Par profil distribué : 500 sessions de 16 airs à valence neutre, 0,6 et −0,6, un air par `position` publiée
à sa première mesure (l'intro, avant tout changement de mode). Seuls les **trois premiers sets** de chaque
session servent aux longueurs et aux transitions : leurs longueurs sont indépendantes et identiquement
distribuées, donc sans biais de troncature (trois sets de 5 airs tiennent dans 16 airs).

Le script recalcule en Python indépendant du moteur : les candidats (`key_modes` ou `[[color]]`), la
classification des transitions, la loi de la règle (chaîne de Markov exacte, `last_dark` compris) et la loi à
tirages indépendants (facteurs à 1). Il échoue (code 1) si une tonalité sort des candidats, si une fréquence
générée s'écarte de la loi de la règle de plus de quatre écarts-types, ou si une longueur s'écarte de la
distribution configurée.
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
            rule = expected(profile, profile['neutral'])
            independent = expected(profile, profile['neutral'], use_factors=False)
            real = REAL.get(REAL_COLUMN.get(name, ''), [None] * 8)
            print(f'n = {n} transitions.\n')
            print('| Classe | Réel % | Indépendance % | Règle attendue % | Généré % (IC 95 %) | Généré / indép. | Facteur |')
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
            rule = expected(profile, valence)
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


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--metrics', type=Path, help='JSON de l\'exemple set_metrics (sinon : exécute l\'exemple)')
    parser.add_argument('--seeds', type=int, default=500)
    parser.add_argument('--airs', type=int, default=16)
    parser.add_argument('--profiles-root', type=Path, default=ROOT / 'profiles')
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
    print()
    if failures:
        print('Écarts :')
        for failure in failures:
            print(' -', failure)
        sys.exit(1)
    print('Aucun écart : 0 tonalité hors candidats ; longueurs et classes conformes à la règle.')


if __name__ == '__main__':
    main()
