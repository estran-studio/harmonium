#!/usr/bin/env python
"""B4 + B7 : résumé des sorties de `make style-distance` contre le corpus GMD (profil calibré et
copie non calibrée), moyenne sur les pistes de référence.

    python b47_distance_resume.py gmd_funk.json gmd_funk_uncal.json

Sortie : tableau Markdown, scores 0 à 100 (100 = identique), moyenne des pistes de référence (et de
10 graines). Seuls les groupes `batterie` et `boucles` (`loop_drums`) du symbolique ont un sens
(le GMD n'a ni basse ni accords) ; l'audio compare un mixage complet à une batterie rendue seule.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from statistics import mean

SYMBOLIC = ("drums_any", "drums_kick", "drums_snare", "drums_hat", "loop_drums")


def summarize(path: str) -> dict[str, float]:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    out: dict[str, float] = {}
    sym = [s for s in data["symbolic"].values() if "scores" in s]
    for key in SYMBOLIC:
        out[key] = mean(s["scores"][key] for s in sym if s["scores"].get(key) is not None)
    out["batterie"] = mean(s["groups"]["batterie"] for s in sym)
    for bank in data["audio"]["banks"].values():
        out["audio"] = mean(r["score"] for r in bank["references"].values())
    out["pistes"] = len(sym)
    return out


def main(argv: list[str]) -> int:
    cols = [(Path(a).stem, summarize(a)) for a in argv]
    print("| Mesure | " + " | ".join(f"`{n}`" for n, _ in cols) + " |")
    print("|---|" + "---:|" * len(cols))
    labels = {"drums_any": "batterie : un coup quelconque par pas", "drums_kick": "batterie : grosse caisse",
              "drums_snare": "batterie : caisse claire", "drums_hat": "batterie : charleston",
              "batterie": "**groupe batterie**", "loop_drums": "boucle de 4 mesures de la batterie",
              "audio": "audio (score global)"}
    for key in ("drums_any", "drums_kick", "drums_snare", "drums_hat", "batterie", "loop_drums", "audio"):
        print(f"| {labels[key]} | " + " | ".join(f"{s[key]:.1f}" for _, s in cols) + " |")
    print(f"\nPistes de référence : {int(cols[0][1]['pistes'])}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
