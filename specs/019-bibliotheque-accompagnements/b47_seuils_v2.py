#!/usr/bin/env python
"""B4 + B7 : les 12 seuils V2 proposés par V0 (`harmonium_lab/references/v0-variance/README.md`,
« Seuils V2 proposés ») lus dans les JSON de `harmonium-lab seed-variety --chorus 16`.

    python b47_seuils_v2.py wjazzd_swing.json gmd_funk.json gmd_neworleans_funk_secondline.json

Sortie : tableau Markdown (valeur mesurée par profil, ✓ si le seuil est tenu, ✗ sinon).
`stable` = chorus voisins de rang ≥ 3 (`adjacent_steady`).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def load(path: str) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))["profiles"][0]


def pct(x: float) -> str:
    return f"{100 * x:.0f} %"


def mark(ok: bool) -> str:
    return "✓" if ok else "✗"


def rows(p: dict) -> list[tuple[str, str, bool | None]]:
    """(libellé de la valeur, valeur, seuil tenu) pour les 12 seuils, dans l'ordre."""
    intra, inter, num = p["intra"], p["inter"], p["numeric"]
    out: list[tuple[str, str, bool | None]] = []
    ident = intra["all"]["identical_pairs"]
    out.append(("1", f"{pct(ident)} ; {intra['all']['distinct_choruses_mean']:.1f} / 16", ident <= 0.0 and intra["all"]["distinct_choruses_mean"] >= 15.99))
    layers = ["comping_rhythm", "comping_voicing", "bass", "drums"]
    vals = [intra[k]["identical_pairs"] for k in layers]
    out.append(("2", " / ".join(pct(v) for v in vals), all(v <= 0.05 for v in vals)))
    chords = intra["chords"]["adjacent_steady"]["exact"]
    out.append(("3", pct(chords), chords <= 0.02))
    cr = intra["comping_rhythm"]["adjacent_steady"]["exact"]
    out.append(("4", pct(cr), cr >= 0.75))
    cv = intra["comping_voicing"]["adjacent_steady"]["exact"]
    out.append(("5", pct(cv), cv >= 0.65))
    bs = intra["bass"]["adjacent_steady"]["exact"]
    out.append(("6", pct(bs), bs >= 0.80))
    dr, drg = intra["drums"]["adjacent_steady"]["exact"], intra["drums"]["adjacent_steady"]["graded"]
    out.append(("7", f"{pct(dr)} ; {pct(drg)}", dr >= 0.30 and drg >= 0.10))
    runs = [inter[k]["distinct_runs"] for k in ("comping_rhythm", "bass", "drums")]
    out.append(("8", " / ".join(f"{r}" for r in runs), all(r >= 12 for r in runs)))
    out.append(("9", "1 programme par piste (A2 : vague 4)", None))
    d = num["all"]["density"]
    out.append(("10", f"cv {pct(d['intra']['cv'])} ; étendue {pct(d['trajectory_range'])}",
                d["intra"]["cv"] >= 0.10 and d["trajectory_range"] >= 0.25))
    v = num["all"]["velocity"]
    out.append(("11", f"cv {pct(v['intra']['cv'])} ; étendue {pct(v['trajectory_range'])}",
                v["intra"]["cv"] >= 0.03 and v["trajectory_range"] >= 0.08))
    e = num["engine"]["intensity"]
    out.append(("12", f"étendue {pct(e['trajectory_range'])}", e["trajectory_range"] >= 0.20))
    return out


THRESHOLDS = {
    "1": "paires de chorus identiques, accompagnement : 0 % ; 16 / 16 chorus distincts",
    "2": "paires identiques par couche (comping rythme / voicings / basse / batterie) ≤ 5 %",
    "3": "accords changés d'un chorus au suivant ≤ 2 %",
    "4": "comping rythme changé (stable) ≥ 75 %",
    "5": "comping voicings changés (stable) ≥ 65 %",
    "6": "basse changée (stable) ≥ 80 %",
    "7": "batterie changée (stable) ≥ 30 % ; distance graduée ≥ 10 %",
    "8": "séquences distinctes sur 16 graines (comping rythme / basse / batterie) ≥ 12",
    "9": "orchestration : comping ≥ 3 programmes, basse ≥ 2, kit ≥ 2 (A2)",
    "10": "arc, densité (notes / mesure) : cv intra ≥ 10 % ; étendue ≥ 25 %",
    "11": "arc, vélocité moyenne : cv intra ≥ 3 % ; étendue ≥ 8 %",
    "12": "arc, `intensity` du moteur : étendue ≥ 20 %",
}


def main(argv: list[str]) -> int:
    profiles = [load(a) for a in argv]
    names = [p["profile"] for p in profiles]
    table = [rows(p) for p in profiles]
    print("| # | Seuil V2 | " + " | ".join(f"`{n}`" for n in names) + " |")
    print("|---|---|" + "---|" * len(names))
    for i, key in enumerate(THRESHOLDS):
        cells = []
        for t in table:
            _, value, ok = t[i]
            cells.append(f"{value} {'' if ok is None else mark(ok)}".strip())
        print(f"| {key} | {THRESHOLDS[key]} | " + " | ".join(cells) + " |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
