#!/usr/bin/env python3
"""B3 samba : résumé des sorties `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus).

Trois mesures, même binaire, même graines (voir le compte rendu pour les commandes) :

    livre  : `profiles/gmd_latin_brazilian_samba` (avec `session.toml`)
    avant  : la même copie sans `session.toml` (« l'avant » : ni tempo, ni tonalité, ni instruments, ni arc nommé tirés)
    v0     : `profiles/wjazzd_swing`, remesuré avec le binaire de la branche (ligne de base de V0)

    python3 gmd_latin_brazilian_samba_variete.py <dossier des JSON> [sortie.json]

Définitions : `harmonium_lab/references/v0-variance/README.md` ; mêmes colonnes que `b8/resumer_variete.py`.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "b8"))
from resumer_variete import LAYERS, summarize  # noqa: E402

NAMES = {"livre": "gmd_latin_brazilian_samba (livré)", "avant": "même profil sans session.toml (avant)",
         "v0": "wjazzd_swing (V0, même binaire)"}


def main() -> int:
    src = Path(sys.argv[1])
    table = {}
    for key, label in NAMES.items():
        data = json.loads((src / f"{key}.json").read_text())
        table[label] = summarize(data["profiles"][0])
    if len(sys.argv) > 2:
        Path(sys.argv[2]).write_text(json.dumps(table, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    print("| Couche | " + " | ".join(NAMES.values()) + " |")
    print("|---|" + "---|" * len(NAMES))
    for label, _ in LAYERS:
        cells = []
        for name in NAMES.values():
            m = table[name][label]
            cells.append(f"{m['intra_mesures_changees']:.0%} · {m['intra_paires_identiques']:.0%} · "
                         f"{m['chorus_distincts_par_graine']} · {m['inter_suites_distinctes']}/16 · {m['inter_distance_graduee']:.0%}")
        print(f"| {label} | " + " | ".join(cells) + " |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
