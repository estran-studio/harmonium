#!/usr/bin/env python3
"""J4 : ce que chacune des 4 émotions fait entendre, par style, sur le flux publié.

Python du labo (numpy) ; binaire `--release` :

    cargo build --release -p harmonium-cli
    harmonium_lab/.venv/bin/python specs/013-moteur-dans-le-jeu/measure_emotions.py \\
        --binary target/release/harmonium-cli

Plan : 7 profils × 20 graines × (neutre + 4 émotions × 5 niveaux + 6 combinaisons), `--automation` constante
(une seule clé `bar = 1` : arousal, valence, density, tension ; jamais `intensity`, que le moteur
déduit alors de l'arousal et de la tension). Une émotion bouge à la fois, les trois autres au
**neutre du profil** : arousal 0,5, densité 0,5, valence = `neutral_valence` du profil, tension = le
genou des courbes de tension du profil (premier abscisse où `tension_duration` vaut 1).

Niveaux 0 / 0,25 / 0,5 / 0,75 / 1 : la valence vit dans −1..1 (`valence_range`), le niveau est donc
normalisé (valeur = −1 + 2 × niveau : −1, −0,5, 0, 0,5, 1) ; les trois autres axes sont lus tels quels.

Mesures (2ᵉ tour de l'air : toutes les pistes de l'arrangement jouent ; la part de mode sombre
porte sur les mesures de tour des deux tours) :
  tempo, vélocité moyenne (mélodie, toutes pistes), notes par mesure par piste, hauteur moyenne de
  la mélodie, part de mode sombre, articulations (Detached / Linked / Accent, en % des notes de la
  mélodie) et gate moyen, durée moyenne des notes, ornements (notes ajoutées par mesure : différence
  avec le même air rendu **sans** `[ornaments]`, dont le flux est identique), notes de passage de
  basse (notes hors des pas écrits de la basse), variations de la 2ᵉ voix (% des notes de la mélodie
  sans note identique — pas, hauteur — dans la 2ᵉ voix, sur les mesures où elle joue), densité de pieds
  (Heel + Toe par mesure), crête (dBFS) et échantillons écrêtés (>= 1) du rendu WAV complet.

Banques : GeneralUser GS pour tout, `harmonium_trad.sf2` pour les pieds, sauf la marche (pieds en GM :
sa caisse claire est muette dans `harmonium_trad.sf2`, voir `GM_FEET`).

Sorties : `emotions-mesures.json` (moyennes et écarts types sur les graines) et `emotions-mesures.md`
(tableaux), dans `--output` (défaut : ce dossier).
"""
from __future__ import annotations

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import json
import math
from pathlib import Path
import re
import shutil
import statistics
import struct
import subprocess
import sys
import tempfile
import tomllib

import numpy as np

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ASSETS = Path('/Users/william.quintal/Project/bascanada/harmonium/assets')
PROFILES = ('reel_quebecois', 'reel_acadien', 'gigue_quebecoise', 'valse_quebecoise',
            'marche_britannique_coloniale', 'gigue_britannique_coloniale',
            'hornpipe_britannique_coloniale')
EMOTIONS = ('arousal', 'valence', 'density', 'tension')
LEVELS = (0.0, 0.25, 0.5, 0.75, 1.0)
TRACKS = {0: 'melodie', 1: 'accords', 2: 'basse', 3: 'talon', 4: 'orteil', 5: 'voix2'}
DARK_MODES = {'dorian', 'aeolian'}  # Mode::is_minor
BARS = 66
# La caisse claire (Toe, notes GM 38 et 40) de la marche est muette dans `harmonium_trad.sf2`
# (mesuré : solo Toe −130 dBFS) : ses pieds sont rendus avec la banque GM (percussion_bank = 128).
GM_FEET = {'marche_britannique_coloniale'}

# nom → (libellé, décimales) ; l'ordre est celui des tableaux.
METRICS = {
    'tempo': ('Tempo (BPM)', 1),
    'vel_melodie': ('Vélocité moyenne, mélodie', 1),
    'vel_toutes': ('Vélocité moyenne, toutes pistes', 1),
    'vel_accords': ('Vélocité moyenne, accords', 1),
    'vel_pieds': ('Vélocité moyenne, pieds', 1),
    'notes_melodie': ('Notes par mesure, mélodie', 2),
    'notes_accords': ('Notes par mesure, accords', 2),
    'notes_basse': ('Notes par mesure, basse', 2),
    'notes_talon': ('Notes par mesure, talon', 2),
    'notes_orteil': ('Notes par mesure, orteil', 2),
    'notes_voix2': ('Notes par mesure, 2ᵉ voix', 2),
    'hauteur_melodie': ('Hauteur moyenne de la mélodie (MIDI)', 1),
    'sombre': ('Part de mesures en mode sombre (%)', 1),
    'detache': ('Notes Detached (% de la mélodie)', 1),
    'lie': ('Notes Linked (% de la mélodie)', 1),
    'accent': ('Notes Accent (% de la mélodie)', 1),
    'gate': ('Gate moyen de la mélodie (‰)', 0),
    'duree': ('Durée moyenne d\'une note de mélodie (pas)', 2),
    'ornements': ('Ornements : notes ajoutées par mesure', 3),
    'passage': ('Notes de passage de basse par mesure', 3),
    'variation_voix2': ('Variations de la 2ᵉ voix (% des notes de la mélodie)', 1),
    'pieds': ('Densité de pieds (notes talon + orteil par mesure)', 2),
    'crete': ('Crête du rendu (dBFS)', 2),
    'ecretes': ('Échantillons écrêtés (>= 1)', 1),
}
# Pour chaque émotion, les mesures montrées dans le résumé de `emotions.md`.
SUMMARY = ('tempo', 'vel_melodie', 'notes_melodie', 'notes_accords', 'notes_basse', 'pieds',
           'sombre', 'hauteur_melodie', 'detache', 'ornements', 'passage', 'variation_voix2', 'crete')
# Combinaisons types (valeurs absolues, valence dans −1..1) : l'effet de plusieurs axes à la fois,
# dont le cas « tout au maximum » pour la crête. Mêmes valeurs pour tous les profils.
SCENARIOS = {
    'calme': {'arousal': 0.2, 'valence': 0.5, 'density': 0.3, 'tension': 0.1},
    'combat': {'arousal': 0.8, 'valence': 0.0, 'density': 0.8, 'tension': 0.6},
    'danger': {'arousal': 0.3, 'valence': -0.6, 'density': 0.3, 'tension': 0.9},
    'boss': {'arousal': 1.0, 'valence': -0.5, 'density': 1.0, 'tension': 0.8},
    'tout_au_maximum': {'arousal': 1.0, 'valence': 1.0, 'density': 1.0, 'tension': 1.0},
    'tout_au_maximum_sombre': {'arousal': 1.0, 'valence': -1.0, 'density': 1.0, 'tension': 1.0},
}


# ---------------------------------------------------------------- profils

def strip_ornaments(text: str) -> str:
    kept, skip = [], False
    for line in text.splitlines(keepends=True):
        heading = re.match(r'^\[\[?([^\]\n]+)\]\]?\s*(?:#.*)?$', line)
        if heading:
            name = heading.group(1).strip()
            skip = name == 'ornaments' or name.startswith('ornaments.')
        if not skip:
            kept.append(line)
    return ''.join(kept)


def neutral(name: str, profiles_root: Path) -> dict[str, float]:
    """Point neutre du profil : les courbes y valent leur valeur « sans effet »."""
    dynamics = tomllib.loads((profiles_root / name / 'dynamics.toml').read_text())
    knee = next(x for x, y in dynamics['tension_duration'] if abs(y - 1.0) < 1e-9)
    return {'arousal': 0.5, 'valence': dynamics['neutral_valence'], 'density': 0.5, 'tension': knee}


def setting_values(base: dict[str, float], emotion: str | None, level: float) -> dict[str, float]:
    values = dict(base)
    if emotion is not None:
        values[emotion] = -1.0 + 2.0 * level if emotion == 'valence' else level
    return values


def settings(base: dict[str, float]) -> dict[str, dict[str, float]]:
    result = {'neutre': setting_values(base, None, 0.0)}
    for emotion in EMOTIONS:
        for level in LEVELS:
            result[f'{emotion}@{level:g}'] = setting_values(base, emotion, level)
    for name, values in SCENARIOS.items():
        result[f'scenario:{name}'] = dict(values)
    return result


def slug(label: str) -> str:
    """Nom de fichier sûr (le suffixe d'un `Path` ne doit pas couper un niveau comme 0.25)."""
    return label.replace('.', 'p').replace(':', '-')


def automation_text(values: dict[str, float]) -> str:
    return '[[keys]]\nbar = 1\n' + ''.join(f'{k} = {values[k]}\n' for k in EMOTIONS)


# ---------------------------------------------------------------- rendu

def run_export(binary: Path, profile: Path, seed: int, automation: Path, root: Path,
               assets: Path, wav: bool, gm_feet: bool) -> None:
    command = [str(binary), '--export', '--profile', str(profile), '--seed', str(seed),
               '--bars', str(BARS), '--tours', '2',
               '--soundfont', str(assets / 'GeneralUser-GS.sf2'),
               '--feet-soundfont', str(assets / ('GeneralUser-GS.sf2' if gm_feet else 'harmonium_trad.sf2')),
               '--automation', str(automation), '--record-truth', str(root.with_suffix('.json'))]
    if wav:
        command += ['--record-wav', str(root.with_suffix('.wav'))]
    subprocess.run(command, check=True, capture_output=True, timeout=300)


def wav_peak(path: Path) -> tuple[float, int]:
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
    if fmt is None or data is None or fmt[0] != 3 or fmt[5] != 32:
        raise ValueError(f'{path}: float32 requis')
    x = np.abs(np.frombuffer(data, dtype='<f4'))
    return 20.0 * math.log10(float(x.max()) + 1e-12), int(np.count_nonzero(x >= 1.0))


def read_measures(path: Path) -> dict[int, dict]:
    truth = json.loads(path.read_text())
    return {e['measure']['index']: e['measure'] for e in truth['measures']
            if e['measure'].get('position') and e['measure']['position']['section'] == 'Tour'}


def bass_hits(profile: dict) -> set[int]:
    acc = profile.get('accompaniment') or {}
    if 'bass' in acc:
        return {h['step'] for h in acc['bass']['hits']}
    return set(acc.get('bass_steps', []))


def analyze(full: dict[int, dict], plain: dict[int, dict], hits: set[int]) -> dict[str, float]:
    """Mesures d'un rendu ; `plain` = même air sans `[ornaments]`."""
    tour2 = {b: m for b, m in full.items() if m['position']['tour'] == 2}
    if not tour2:
        raise ValueError('aucune mesure du 2ᵉ tour')
    bars = len(tour2)
    vel: dict[int, list[int]] = {t: [] for t in TRACKS}
    counts: Counter = Counter()
    melody_notes, gates, durations, pitches = 0, [], [], []
    articulation: Counter = Counter()
    voice_bars = varied = lead_in_voice_bars = passing = 0
    added = 0
    for bar, m in tour2.items():
        by_track: dict[int, list[dict]] = {t: [] for t in TRACKS}
        for n in m['notes']:
            by_track[n['track']].append(n)
            vel[n['track']].append(n['velocity'])
        for t, notes in by_track.items():
            counts[t] += len(notes)
        for n in by_track[0]:
            melody_notes += 1
            gates.append(n.get('performance', {}).get('gate', 1000))
            durations.append(n['duration_steps'])
            pitches.append(n['pitch'])
            articulation[n.get('articulation', 'Normal')] += 1
        passing += sum(1 for n in by_track[2] if n['start_step'] not in hits)
        if by_track[5]:
            voice_bars += 1
            second = {(n['start_step'], n['pitch']) for n in by_track[5]}
            lead = [(n['start_step'], n['pitch']) for n in by_track[0]]
            lead_in_voice_bars += len(lead)
            varied += sum(1 for key in lead if key not in second)
        other = plain[bar]
        for t in (1, 3, 4):  # la basse évite les hauteurs en conflit avec la mélodie ornée : exclue
            if [n for n in m['notes'] if n['track'] == t] != [n for n in other['notes'] if n['track'] == t]:
                raise ValueError(f'mesure {bar} : la piste {t} dépend des ornements')
        key = lambda n: (n['start_step'], n['pitch'], n['duration_steps'])  # noqa: E731
        added += sum((Counter(key(n) for n in by_track[0])
                      - Counter(key(n) for n in other['notes'] if n['track'] == 0)).values())
    all_velocities = [v for values in vel.values() for v in values]
    mean = lambda values: statistics.fmean(values) if values else float('nan')  # noqa: E731
    dark = sum(1 for m in full.values() if m['position']['mode'] in DARK_MODES)
    return {
        'tempo': mean([m['tempo'] for m in tour2.values()]),
        'vel_melodie': mean(vel[0]), 'vel_toutes': mean(all_velocities),
        'vel_accords': mean(vel[1]), 'vel_pieds': mean(vel[3] + vel[4]),
        **{f'notes_{TRACKS[t]}': counts[t] / bars for t in TRACKS},
        'hauteur_melodie': mean(pitches),
        'sombre': 100.0 * dark / len(full),
        'detache': 100.0 * articulation['Detached'] / melody_notes,
        'lie': 100.0 * articulation['Linked'] / melody_notes,
        'accent': 100.0 * articulation['Accent'] / melody_notes,
        'gate': mean(gates), 'duree': mean(durations),
        'ornements': added / bars, 'passage': passing / bars,
        'variation_voix2': 100.0 * varied / lead_in_voice_bars if lead_in_voice_bars else float('nan'),
        'pieds': (counts[3] + counts[4]) / bars,
    }


# ---------------------------------------------------------------- agrégation et tableaux

def aggregate(per_seed: list[dict[str, float]]) -> dict[str, dict[str, float]]:
    out = {}
    for metric in METRICS:
        values = [row[metric] for row in per_seed if not math.isnan(row[metric])]
        out[metric] = {'mean': statistics.fmean(values) if values else float('nan'),
                       'sd': statistics.stdev(values) if len(values) > 1 else 0.0,
                       'max': max(values) if values else float('nan'), 'n': len(values)}
    return out


def cell(stats: dict[str, float], digits: int, key: str = 'mean') -> str:
    value = stats[key]
    return '—' if math.isnan(value) else f'{value:.{digits}f}'


def level_label(emotion: str, level: float) -> str:
    return f'{-1.0 + 2.0 * level:+g}' if emotion == 'valence' else f'{level:g}'


def scenario_lines(result: dict, bases: dict) -> list[str]:
    """Les combinaisons de `SCENARIOS` face au neutre, une table par mesure."""
    names = list(bases)
    labels = ['neutre'] + [f'scenario:{s}' for s in SCENARIOS]
    lines = ['## Combinaisons', '',
             '| Scénario | arousal | valence | densité | tension |', '|---|---:|---:|---:|---:|']
    lines += [f"| {s} | {v['arousal']:g} | {v['valence']:g} | {v['density']:g} | {v['tension']:g} |"
              for s, v in SCENARIOS.items()]
    lines += ['', 'Crête et écrêtés : maximum sur les graines ; les autres mesures : moyennes.', '']
    for metric in ('tempo', 'vel_melodie', 'notes_accords', 'pieds', 'sombre', 'duree', 'ornements',
                   'crete', 'ecretes'):
        label, digits = METRICS[metric]
        key = 'max' if metric in ('crete', 'ecretes') else 'mean'
        lines += [f'### {label}', '', '| Scénario | ' + ' | '.join(names) + ' |', '|---|' + '---:|' * len(names)]
        for scenario in labels:
            lines.append(f"| {scenario.replace('scenario:', '')} | "
                         + ' | '.join(cell(result[n][scenario][metric], digits, key) for n in names) + ' |')
        lines.append('')
    return lines


def summary_lines(result: dict, bases: dict) -> list[str]:
    """Niveau 0 → niveau 1 par émotion et style, puis la variation moyenne de chaque mesure par émotion."""
    names = list(bases)
    pick = lambda metric: 'max' if metric == 'crete' else 'mean'  # noqa: E731
    lines = ['## Résumé : niveau 0 → niveau 1', '',
             'Cellule = valeur au niveau 0 → valeur au niveau 1 (valence : −1 → +1) ; les trois autres axes '
             'au neutre du profil. Crête : maximum sur les graines.', '']
    for emotion in EMOTIONS:
        lines += [f'### {emotion}', '', '| Mesure | ' + ' | '.join(names) + ' |', '|---|' + '---:|' * len(names)]
        for metric in SUMMARY:
            label, digits = METRICS[metric]
            cells = [f"{cell(result[n][f'{emotion}@0'][metric], digits, pick(metric))} → "
                     f"{cell(result[n][f'{emotion}@1'][metric], digits, pick(metric))}" for n in names]
            lines.append(f'| {label} | ' + ' | '.join(cells) + ' |')
        lines.append('')
    lines += ['## Qui fait bouger quoi', '',
              'Variation moyenne (niveau 1 − niveau 0) sur les profils où la mesure existe.', '',
              '| Mesure | ' + ' | '.join(EMOTIONS) + ' |', '|---|' + '---:|' * len(EMOTIONS)]
    for metric, (label, digits) in METRICS.items():
        cells = []
        for emotion in EMOTIONS:
            deltas = [result[n][f'{emotion}@1'][metric][pick(metric)] - result[n][f'{emotion}@0'][metric][pick(metric)]
                      for n in names]
            deltas = [d for d in deltas if not math.isnan(d)]
            cells.append(f'{statistics.fmean(deltas):+.{digits}f}' if deltas else '—')
        lines.append(f'| {label} | ' + ' | '.join(cells) + ' |')
    return lines + ['']


def markdown(result: dict, bases: dict, seeds: int) -> str:
    lines = ['# Mesures des 4 émotions (généré par `measure_emotions.py`)', '',
             f'{len(bases)} profils × {seeds} graines × (neutre + 4 émotions × 5 niveaux + combinaisons), '
             f'{BARS} mesures, 2 tours, `--automation` constante. Moyennes sur les graines ; '
             'l\'écart type entre graines est dans `emotions-mesures.json`. '
             'Une émotion bouge, les trois autres restent au neutre du profil :', '',
             '| Profil | arousal | valence | densité | tension |', '|---|---:|---:|---:|---:|']
    for name in bases:
        b = bases[name]
        lines.append(f"| {name} | {b['arousal']:g} | {b['valence']:g} | {b['density']:g} | {b['tension']:g} |")
    lines += ['', 'Colonnes = niveau de l\'émotion (valence : −1, −0,5, 0, 0,5, 1) ; **neutre** = les quatre au neutre.', '']
    lines += summary_lines(result, bases)
    lines += scenario_lines(result, bases)
    for emotion in EMOTIONS:
        lines += [f'## {emotion}', '']
        for metric, (label, digits) in METRICS.items():
            lines += [f'### {emotion} — {label}', '',
                      '| Profil | neutre | ' + ' | '.join(level_label(emotion, v) for v in LEVELS) + ' |',
                      '|---|---:|' + '---:|' * len(LEVELS)]
            for name in bases:
                row = [cell(result[name]['neutre'][metric], digits)]
                row += [cell(result[name][f'{emotion}@{v:g}'][metric], digits,
                             'max' if metric == 'crete' else 'mean') for v in LEVELS]
                lines.append(f'| {name} | ' + ' | '.join(row) + ' |')
            lines.append('')
        # note sur la crête : maximum sur les graines, pas la moyenne
    lines.insert(lines.index('## arousal'),
                 'La **crête** est le maximum sur les graines ; toutes les autres colonnes sont des moyennes.\n')
    return '\n'.join(lines) + '\n'


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/harmonium-cli')
    parser.add_argument('--profiles-root', type=Path, default=ROOT / 'profiles')
    parser.add_argument('--assets', type=Path, default=ASSETS)
    parser.add_argument('--output', type=Path, default=HERE)
    parser.add_argument('--seeds', type=int, default=20)
    parser.add_argument('--jobs', type=int, default=8)
    parser.add_argument('--profiles', default=','.join(PROFILES))
    args = parser.parse_args()
    names = args.profiles.split(',')
    binary = args.binary.resolve()
    bases = {name: neutral(name, args.profiles_root) for name in names}
    hits = {name: bass_hits(tomllib.loads((args.profiles_root / name / 'profile.toml').read_text()))
            for name in names}
    result: dict[str, dict[str, dict]] = {name: {} for name in names}
    with tempfile.TemporaryDirectory(prefix='harmonium-emotions-') as temp:
        temp = Path(temp)
        for name in names:
            shutil.copytree(args.profiles_root / name, temp / name)
            shutil.copytree(args.profiles_root / name, temp / f'{name}-plain')
            plain = temp / f'{name}-plain' / 'profile.toml'
            plain.write_text(strip_ornaments(plain.read_text()))
            for label, values in settings(bases[name]).items():
                (temp / f'{name}--{slug(label)}.toml').write_text(automation_text(values))

        def run(case: tuple[str, str, int]) -> tuple[str, str, int, dict[str, float]]:
            name, label, seed = case
            automation = temp / f'{name}--{slug(label)}.toml'
            stem = f'{name}--{slug(label)}--{seed}'
            full, plain = temp / stem, temp / f'{stem}-plain'
            gm = name in GM_FEET
            run_export(binary, temp / name, seed, automation, full, args.assets, wav=True, gm_feet=gm)
            run_export(binary, temp / f'{name}-plain', seed, automation, plain, args.assets, wav=False, gm_feet=gm)
            row = analyze(read_measures(full.with_suffix('.json')), read_measures(plain.with_suffix('.json')),
                          hits[name])
            row['crete'], row['ecretes'] = wav_peak(full.with_suffix('.wav'))
            for path in (full.with_suffix('.json'), full.with_suffix('.wav'), plain.with_suffix('.json')):
                path.unlink()
            return name, label, seed, row

        cases = [(name, label, seed) for name in names for label in settings(bases[name])
                 for seed in range(1, args.seeds + 1)]
        collected: dict[tuple[str, str], list[dict[str, float]]] = {}
        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            for i, (name, label, _, row) in enumerate(pool.map(run, cases), 1):
                collected.setdefault((name, label), []).append(row)
                if i % 100 == 0 or i == len(cases):
                    print(f'{i}/{len(cases)}', flush=True)
    for (name, label), rows in collected.items():
        result[name][label] = aggregate(rows)
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'emotions-mesures.json').write_text(json.dumps(
        {'seeds': args.seeds, 'bars': BARS, 'neutral': bases, 'levels': LEVELS, 'metrics': {
            k: v[0] for k, v in METRICS.items()}, 'results': result},
        ensure_ascii=False, indent=1, allow_nan=True) + '\n')
    (args.output / 'emotions-mesures.md').write_text(markdown(result, bases, args.seeds))
    worst = max(result[n][label]['crete']['max'] for n in names for label in result[n])
    print(f'crête maximale {worst:.2f} dBFS')
    for n in names:
        for label, stats in result[n].items():
            if stats['ecretes']['max'] > 0:
                print(f"écrêtage : {n} {label} : crête max {stats['crete']['max']:.2f} dBFS, "
                      f"jusqu'à {stats['ecretes']['max']:.0f} échantillons >= 1 sur {args.seeds} graines")


if __name__ == '__main__':
    main()
