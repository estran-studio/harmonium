#!/usr/bin/env python3
"""B3 afro-cubain : résumé des sorties `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus).

    for n in livre avant v0 ; do   # livre = profiles/gmd_afrocuban ; avant = copie sans session.toml ; v0 = profiles/wjazzd_swing
      harmonium-lab seed-variety --profile <dossier de $n> --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium \
          --engine-bin <worktree>/target/release/harmonium-cli --label B3 --work-dir /tmp/harmonium-019/b3/afrocubain/var/work_$n \
          -o /tmp/harmonium-019/b3/afrocubain/var/$n.json
    done
    python3 specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_variete.py /tmp/harmonium-019/b3/afrocubain/var [sortie.json]

Définitions : `harmonium_lab/references/v0-variance/README.md`. `wjazzd_swing` (V0) est remesuré avec le même binaire.
Les couches sont celles de `b8/resumer_variete.py` (`summarize`).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "b8"))
from resumer_variete import summarize  # noqa: E402

NOMS = {"livre": "gmd_afrocuban (livré)", "avant": "gmd_afrocuban sans session.toml (l'« avant »)", "v0": "wjazzd_swing (V0 remesuré)"}


def main() -> int:
    src = Path(sys.argv[1])
    table, extra = {}, {}
    for cle, nom in NOMS.items():
        data = json.loads((src / f"{cle}.json").read_text())["profiles"][0]
        table[nom] = summarize(data)
        num = data["numeric"]["all"]
        extra[nom] = {"densite_notes_par_mesure": num["density"]["trajectory"], "densite_etendue": round(num["density"]["trajectory_range"], 3),
                      "velocite": num["velocity"]["trajectory"], "velocite_etendue": round(num["velocity"]["trajectory_range"], 3),
                      "orchestration_programmes": data["orchestration_programs"]}
    print("| Couche | " + " | ".join(NOMS.values()) + " |")
    print("|---|" + "---|" * len(NOMS))
    for couche in next(iter(table.values())):
        cells = []
        for nom in NOMS.values():
            m = table[nom][couche]
            cells.append(f"{m['intra_mesures_changees']:.0%} / {m['intra_paires_identiques']:.0%} / {m['chorus_distincts_par_graine']} / {m['inter_suites_distinctes']} / {m['inter_distance_graduee']:.0%}")
        print(f"| {couche} | " + " | ".join(cells) + " |")
    print("\n(mesures changées d'un chorus au suivant / paires de chorus identiques / chorus distincts par graine / suites distinctes sur 16 graines / distance graduée entre graines)\n")
    for nom, e in extra.items():
        print(nom, json.dumps(e, ensure_ascii=False))
    if len(sys.argv) > 2:
        Path(sys.argv[2]).write_text(json.dumps({"couches": table, "arc_et_orchestration": extra}, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
