#!/usr/bin/env python3
"""B3 : résumé de `harmonium-lab seed-variety --chorus 4` (16 graines x 4 chorus) du profil `gmd_latin_brazilian_baiao`.

Commandes (depuis `~/Project/bascanada/harmonium`, binaire de la branche ; définitions : `references/v0-variance/README.md`) :

    L=~/Project/bascanada/harmonium_lab/.venv/bin/harmonium-lab
    for p in profiles/gmd_latin_brazilian_baiao <copie sans session.toml> profiles/wjazzd_swing; do
      $L seed-variety --profile $p --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium \\
        --engine-bin <worktree>/target/release/harmonium-cli --label B3 --work-dir /tmp/harmonium-019/b3/baiao/var/$(basename $p) \\
        -o /tmp/harmonium-019/b3/baiao/var/$(basename $p).json
    done
    python3 specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_variete.py \\
        /tmp/harmonium-019/b3/baiao/var/gmd_latin_brazilian_baiao.json \\
        /tmp/harmonium-019/b3/baiao/var/gmd_latin_brazilian_baiao_sans_session.json \\
        /tmp/harmonium-019/b3/baiao/var/wjazzd_swing.json

L'« avant » est le même profil sans `session.toml` (ni la ligne `session =` de `[files]`) ; `wjazzd_swing` (V0) est remesuré
avec le même binaire. Sortie : tableau par couche (mesures changées d'un chorus au suivant, paires de chorus identiques,
chorus distincts par graine sur 4, suites distinctes sur 16 graines, distance graduée entre graines), puis l'arc
(densité et vélocité par chorus) et l'orchestration (programmes vus). Reprend `summarize` de `b8/resumer_variete.py`.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
sys.path.insert(0, str(ICI.parent / "b8"))
import resumer_variete as R  # noqa: E402


def main(argv: list[str]) -> None:
    profils = {Path(a).stem: json.loads(Path(a).read_text())["profiles"][0] for a in argv}
    print("| Profil | Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine (sur 4) | Suites distinctes (16 graines) | Distance graduée entre graines |")
    print("|---|---|---:|---:|---:|---:|---:|")
    for nom, p in profils.items():
        for couche, m in R.summarize(p).items():
            print(f"| `{nom}` | {couche} | {m['intra_mesures_changees']:.0%} | {m['intra_paires_identiques']:.0%} | "
                  f"{m['chorus_distincts_par_graine']} | {m['inter_suites_distinctes']} / 16 | {m['inter_distance_graduee']:.0%} |")
    print()
    print("| Profil | Densité (notes / mesure) par chorus 1..4 | cv intra | Vélocité moyenne par chorus 1..4 | cv intra | Programmes vus (comping / basse / kit) |")
    print("|---|---|---:|---|---:|---|")
    for nom, p in profils.items():
        d, v = p["numeric"]["all"]["density"], p["numeric"]["all"]["velocity"]
        prog = p["orchestration_programs"]
        print(f"| `{nom}` | {' / '.join(f'{x:.1f}' for x in d['trajectory'])} | {d['intra']['cv']:.0%} | "
              f"{' / '.join(f'{x:.1f}' for x in v['trajectory'])} | {v['intra']['cv']:.0%} | "
              f"{prog.get('1')} / {prog.get('2')} / {prog.get('9')} |")


if __name__ == "__main__":
    main(sys.argv[1:])
