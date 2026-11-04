#!/usr/bin/env python3
"""R2-trad : tableau avant / après de `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus).

    python3 comparer_variete.py <dossier avant> <dossier après> [--out f.json]

Par profil et par couche : distance graduée entre graines (`inter.graded`) et suites distinctes sur 16 graines
(`inter.distinct_runs`), puis, intra-graine, la part de mesures changées d'un chorus au suivant (`intra.adjacent.exact`).
La couche « orchestration » lit les `program_change` du MIDI (A2) : séquences de programmes distinctes sur 16 graines.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

LAYERS = [("accompagnement", "all"), ("comping : rythme", "comping_rhythm"), ("comping : voicings", "comping_voicing"),
          ("basse", "bass"), ("batterie", "drums"), ("mélodie", "melody"), ("orchestration", "orchestration")]
IDS = ["reel_quebecois", "reel_acadien", "reel_britannique_coloniale", "gigue_quebecoise", "gigue_acadienne",
       "gigue_britannique_coloniale", "hornpipe_britannique_coloniale", "marche_britannique_coloniale", "valse_quebecoise",
       "valse_britannique_coloniale"]


def load(folder: Path, pid: str) -> dict:
    return json.loads((folder / f"sv-{pid}.json").read_text())["profiles"][0]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("avant", type=Path)
    ap.add_argument("apres", type=Path)
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    table = {}
    print("| Profil | Couche | Suites distinctes avant → après | Distance graduée entre graines avant → après | Mesures changées d'un chorus au suivant avant → après |")
    print("|---|---|---|---|---|")
    for pid in IDS:
        a, b = load(args.avant, pid), load(args.apres, pid)
        table[pid] = {}
        for label, key in LAYERS:
            ia, ib = a["inter"][key], b["inter"][key]
            ra, rb = a["intra"][key], b["intra"][key]
            row = {"suites_avant": ia["distinct_runs"], "suites_apres": ib["distinct_runs"],
                   "graduee_avant": round(ia["graded"] or 0, 3), "graduee_apres": round(ib["graded"] or 0, 3),
                   "chorus_avant": (round(ra["adjacent"]["exact"], 3) if ra["adjacent"]["exact"] is not None else 0.0), "chorus_apres": (round(rb["adjacent"]["exact"], 3) if rb["adjacent"]["exact"] is not None else 0.0)}
            table[pid][label] = row
            print(f"| `{pid}` | {label} | {row['suites_avant']} → {row['suites_apres']} | {row['graduee_avant']:.0%} → {row['graduee_apres']:.0%} | "
                  f"{row['chorus_avant']:.0%} → {row['chorus_apres']:.0%} |")
    if args.out:
        args.out.write_text(json.dumps(table, indent=1, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
