#!/usr/bin/env python3
"""J4 : la proposition de données qui distingue les émotions (non appliquée aux profils du dépôt).

Écrit une copie patchée de `profiles/` : les profils du dépôt ne sont jamais modifiés. Pour mesurer :

    python3 specs/013-moteur-dans-le-jeu/proposition_emotions.py --output /tmp/profils-proposes
    harmonium_lab/.venv/bin/python specs/013-moteur-dans-le-jeu/measure_emotions.py \\
        --profiles-root /tmp/profils-proposes --output /tmp/mesures-proposees

Trois changements, détaillés et justifiés dans `emotions-inventaire.md` (§ Propositions) :

1. **densité = qui joue, arousal = comment ça joue** : `density_feet` et `density_chop` mis à zéro.
   Les pieds et le grattage suivent l'arousal seul ; la densité garde les seuils de pistes, les notes
   de passage (basse et ornement), et la 2ᵉ voix. Les pieds de la marche, sans seuil, reçoivent ceux du
   reel (Heel 0,10, Toe 0,35) : sans eux la densité n'y ferait plus rien.
2. **tension moins forte** : `tension_offbeat` × 0,6 et `tension_intensity` × 0,5 (la tension
   cesse de doubler l'arousal sur la force et le tempo ; elle garde notes courtes, contretemps,
   dominantes tenues).
3. **plus d'écrêtage** : `master_db` abaissé là où la crête mesurée à arousal 0,75–1 atteint 0 dBFS.

Seules les lignes visées sont réécrites ; les commentaires et le reste du fichier restent intacts.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import shutil
import tomllib

ROOT = Path(__file__).resolve().parents[2]
ZERO = [[0.0, 0.0], [1.0, 0.0]]
FEET_THRESHOLDS = {'Heel': 0.10, 'Toe': 0.35}
TENSION_OFFBEAT_SCALE = 0.6
TENSION_INTENSITY_SCALE = 0.5
# Seuls les styles dont la crête mesurée (maximum sur 20 graines) dépasse 0 dBFS ; ≈ 1 dBFS de marge ensuite.
MASTER_DB = {'gigue_britannique_coloniale': 14.0, 'valse_quebecoise': 11.0, 'reel_acadien': 14.5}


def curve_text(points: list[list[float]]) -> str:
    return '[' + ', '.join(f'[{x:g}, {y:g}]' for x, y in points) + ']'


def scaled_top(points: list[list[float]], factor: float) -> list[list[float]]:
    """Met à l'échelle les valeurs au-dessus de la valeur du point neutre (premier point où y = 0)."""
    return [[x, y * factor if y > 0 else y] for x, y in points]


def patch_dynamics(text: str) -> str:
    out = []
    for line in text.splitlines(keepends=True):
        key = line.split('=', 1)[0].strip()
        if key in ('density_feet', 'density_chop'):
            line = f'{key} = {curve_text(ZERO)}\n'
        elif key in ('tension_offbeat', 'tension_intensity'):
            points = tomllib.loads(line)[key]
            factor = TENSION_OFFBEAT_SCALE if key == 'tension_offbeat' else TENSION_INTENSITY_SCALE
            line = f'{key} = {curve_text(scaled_top(points, factor))}\n'
        for track, minimum in FEET_THRESHOLDS.items():
            line = line.replace(f'{{ track = "{track}", minimum = 0.0 }}', f'{{ track = "{track}", minimum = {minimum} }}')
        out.append(line)
    return ''.join(out)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--profiles-root', type=Path, default=ROOT / 'profiles')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    for source in sorted(p for p in args.profiles_root.iterdir() if (p / 'dynamics.toml').exists()):
        target = args.output / source.name
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(source, target)
        dynamics = target / 'dynamics.toml'
        dynamics.write_text(patch_dynamics(dynamics.read_text()))
        if source.name in MASTER_DB:
            profile = target / 'profile.toml'
            patched, count = re.subn(r'^master_db = .*$', f'master_db = {MASTER_DB[source.name]}',
                                     profile.read_text(), flags=re.M)
            assert count == 1, f'{source.name} : master_db introuvable ou multiple'
            profile.write_text(patched)
        print(target)


if __name__ == '__main__':
    main()
