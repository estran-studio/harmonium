#!/usr/bin/env python3
"""H1 : marge de crête sur toute la grille d'émotions (données seulement).

Grille complète par profil : arousal × densité × tension à 0 / 0,5 / 1,
valence à -1 / neutre du profil / 1, intensité directe à 0,5 / 1
(3^4 × 2 = 162 réglages), 66 mesures, 2 tours, 5 graines, rendu WAV
flottant. Pour chaque profil : pire crête, réglage en cause, crête et RMS
au neutre ; tableau markdown ; échec si une pire crête dépasse -1,5 dBFS.

Banques : GeneralUser GS pour tout, `harmonium_trad.sf2` pour les pieds,
sauf la marche (pieds en GM : sa caisse claire est muette dans
`harmonium_trad.sf2`, voir `GM_FEET`).

Usage :
    labo/.venv/bin/python specs/013-moteur-dans-le-jeu/measure_marge.py \
        --binary target/release/harmonium-cli
"""
from __future__ import annotations

import argparse
import itertools
import json
import math
import struct
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    import tomli as tomllib  # type: ignore[no-redef]

import numpy as np

REPO = Path(__file__).resolve().parents[2]
ASSETS = Path('/Users/william.quintal/Project/bascanada/harmonium/assets')
PROFILES = ('reel_quebecois', 'gigue_quebecoise', 'valse_quebecoise', 'reel_acadien',
            'marche_britannique_coloniale', 'gigue_britannique_coloniale',
            'hornpipe_britannique_coloniale', 'gigue_acadienne',
            'reel_britannique_coloniale', 'valse_britannique_coloniale')
# La caisse claire (Toe, notes GM 38 et 40) de la marche est muette dans
# `harmonium_trad.sf2` : ses pieds sont rendus avec la banque GM.
GM_FEET = {'marche_britannique_coloniale'}
LIMIT_DBFS = -1.5
BARS = 66
SEEDS = (1, 2, 3, 4, 5)


def neutral_valence(profiles_root: Path, name: str) -> float:
    dynamics = tomllib.loads((profiles_root / name / 'dynamics.toml').read_text())
    return float(dynamics['neutral_valence'])
def neutral_tension(profiles_root: Path, name: str) -> float:
    """Genou des courbes de tension : premier abscisse où `tension_duration` vaut 1."""
    dynamics = tomllib.loads((profiles_root / name / 'dynamics.toml').read_text())
    return float(next(x for x, y in dynamics['tension_duration'] if abs(y - 1.0) < 1e-9))

def grid(neutral: float) -> list[dict[str, float]]:
    points = (0.0, 0.5, 1.0)
    combos = []
    for arousal, density, tension, intensity in itertools.product(points, points, points, (0.5, 1.0)):
        for valence in (-1.0, neutral, 1.0):
            combos.append({'arousal': arousal, 'valence': valence, 'density': density,
                           'tension': tension, 'intensity': intensity})
    return combos


def automation_text(values: dict[str, float]) -> str:
    return '[[keys]]\nbar = 1\n' + ''.join(f'{k} = {v}\n' for k, v in values.items())


def slug(values: dict[str, float]) -> str:
    return '-'.join(f'{k}{v}'.replace('.', 'p').replace('-', 'm') for k, v in values.items())


def peak_dbfs(path: Path) -> tuple[float, int]:
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
        raise ValueError(f'{path}: WAV flottant stéréo requis')
    x = np.frombuffer(data, dtype='<f4').reshape(-1, 2).astype(np.float64)
    peak = float(np.max(np.abs(x)))
    return 20 * math.log10(peak + 1e-12), int(np.count_nonzero(np.abs(x) >= 1.0))


def rms_dbfs(path: Path) -> float:
    raw = path.read_bytes()
    offset, data = 12, None
    while offset + 8 <= len(raw):
        name, size = struct.unpack_from('<4sI', raw, offset)
        chunk = raw[offset + 8:offset + 8 + size]
        if name == b'data':
            data = chunk
        offset += 8 + size + size % 2
    x = np.frombuffer(data, dtype='<f4').astype(np.float64)
    return 20 * math.log10(float(np.sqrt(np.mean(x * x))) + 1e-12)


def export(binary: Path, profile: Path, seed: int, automation: Path, root: Path,
           feet_bank: Path, solo: str | None = None) -> None:
    command = [str(binary), '--export', '--profile', str(profile), '--seed', str(seed),
               '--bars', str(BARS), '--tours', '2',
               '--soundfont', str(ASSETS / 'GeneralUser-GS.sf2'),
               '--feet-soundfont', str(feet_bank),
               '--automation', str(automation), '--record-truth', str(root.with_suffix('.json'))]
    if solo is not None:
        command += ['--solo-track', solo]
    command += ['--record-wav', str(root.with_suffix('.wav'))]
    subprocess.run(command, check=True, capture_output=True, timeout=300)


def markdown(result: dict, limit: float) -> str:
    names = [n for n in PROFILES if n in result] or sorted(result)
    lines = ['| Profil | Pire crête grille (dBFS) | Réglage en cause | Crête neutre (dBFS) | '
             'RMS neutre (dBFS) | Échantillons ≥ 1 | Verdict |']
    for name in names:
        row = result[name]
        setting = ', '.join(f'{k}={v:g}' for k, v in row['pire_reglage'].items())
        verdict = 'ok' if row['pire_crete'] <= limit else 'ÉCRÊTE'
        lines.append(f"| {name} | {row['pire_crete']:.2f} | {setting} | "
                     f"{row['crete_neutre']:.2f} | {row['rms_neutre']:.2f} | "
                     f"{row['ecretes']} | {verdict} |")
    return '\n'.join(lines) + '\n'


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=REPO / 'specs/013-moteur-dans-le-jeu')
    parser.add_argument('--profiles-root', type=Path, default=REPO / 'profiles')
    parser.add_argument('--profiles', nargs='*', choices=list(PROFILES), default=list(PROFILES))
    parser.add_argument('--seeds', type=int, nargs='*', default=list(SEEDS))
    parser.add_argument('--jobs', type=int, default=10)
    parser.add_argument('--limit', type=float, default=LIMIT_DBFS)
    args = parser.parse_args()

    binary = args.binary.resolve()
    neutrals = {name: neutral_valence(args.profiles_root, name) for name in args.profiles}
    knees = {name: neutral_tension(args.profiles_root, name) for name in args.profiles}
    grids = {name: grid(neutrals[name]) for name in args.profiles}
    total = sum(len(grids[n]) * len(args.seeds) for n in args.profiles)
    print(f'{total} rendus ({len(args.profiles)} profils, '
          f'{sum(len(g) for g in grids.values()) // max(len(args.profiles), 1)} réglages, '
          f'{len(args.seeds)} graines)', flush=True)

    result: dict = {}
    with tempfile.TemporaryDirectory(prefix='harmonium-marge-') as temporary:
        temporary = Path(temporary)

        def run(case: tuple[str, dict[str, float] | None, int]) -> tuple:
            name, values, seed = case
            tag = 'neutre' if values is None else slug(values)
            stem = temporary / f'{name}-{tag}-{seed}'
            auto = temporary / f'{name}-{tag}.toml'
            if values is None:
                values = {'arousal': 0.5, 'valence': neutrals[name], 'density': 0.5,
                          'tension': knees[name], 'intensity': 0.5}
            if not auto.exists():
                auto.write_text(automation_text(values))
            feet = (ASSETS / 'GeneralUser-GS.sf2' if name in GM_FEET
                    else ASSETS / 'harmonium_trad.sf2')
            export(binary, args.profiles_root / name, seed, auto, stem, feet)
            peak, clipped = peak_dbfs(stem.with_suffix('.wav'))
            rms = rms_dbfs(stem.with_suffix('.wav')) if tag == 'neutre' else None
            stem.with_suffix('.wav').unlink()
            stem.with_suffix('.json').unlink()
            return name, tag, seed, values, peak, clipped, rms

        cases = [(n, None, s) for n in args.profiles for s in args.seeds]
        cases += [(n, v, s) for n in args.profiles for v in grids[n] for s in args.seeds]
        worst: dict = {}
        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            for i, (name, tag, seed, values, peak, clipped, rms) in enumerate(pool.map(run, cases), 1):
                row = worst.setdefault(name, {'pire_crete': -1e9, 'pire_reglage': {},
                                              'pire_graine': 0, 'crete_neutre': -1e9,
                                              'rms_neutre': [], 'ecretes': 0})
                if tag == 'neutre':
                    row['crete_neutre'] = max(row['crete_neutre'], peak)
                    row['rms_neutre'].append(rms)
                elif peak > row['pire_crete']:
                    row['pire_crete'] = peak
                    row['pire_reglage'] = values
                    row['pire_graine'] = seed
                row['ecretes'] += clipped
                if i % 500 == 0 or i == len(cases):
                    print(f'{i}/{len(cases)}', flush=True)
        for name, row in worst.items():
            row['rms_neutre'] = float(np.mean(row['rms_neutre']))

    result = worst
    table = markdown(result, args.limit)
    print()
    print(table)
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'marge-mesures.json').write_text(
        json.dumps(result, ensure_ascii=False, separators=(',', ':')) + '\n')
    (args.output / 'marge-mesures.md').write_text(table)
    print(args.output / 'marge-mesures.md')
    if any(row['pire_crete'] > args.limit for row in result.values()):
        raise SystemExit(f"pire crête > {args.limit} dBFS sur la grille")


if __name__ == '__main__':
    main()
