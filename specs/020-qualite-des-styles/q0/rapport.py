#!/usr/bin/env python3
"""Q0 : tableaux avant / après du compte rendu, à partir des sorties de `mesure.py` (`avant.json`, `apres.json`), des rendus de
vérité terrain (`runs/<étiquette>/<profil>_<graine>.json`) et de `distance.py` (`dist13-<étiquette>.json`).

    python3 specs/020-qualite-des-styles/q0/rapport.py /tmp/h020-q0 > tableaux.md
"""
from __future__ import annotations

import json
import statistics as st
import sys
from pathlib import Path

E1 = ["wjazzd_swing", "wjazzd_ballad", "gmd_jazz_waltz", "gmd_latin_brazilian_bossa", "gmd_latin_brazilian_samba",
      "gmd_latin_brazilian_baiao", "gmd_afrocuban", "gmd_funk", "gmd_soul", "gmd_reggae", "gmd_rock", "gmd_pop",
      "gmd_blues_shuffle"]
W = Path(sys.argv[1])
A, B = json.loads((W / "avant.json").read_text()), json.loads((W / "apres.json").read_text())


def truth(tag: str, pid: str, seed: int) -> list[dict]:
    last = {}
    for rec in json.loads((W / "runs" / tag / f"{pid}_{seed}.json").read_text())["measures"]:
        last[rec["measure"]["index"]] = rec["measure"]
    return [last[i] for i in sorted(last) if i <= 47]


def dressings(tag: str, pid: str) -> int:
    seen = set()
    for seed in range(1, 17):
        m = truth(tag, pid, seed)[0]
        seen.add(tuple((v["group"], v["name"]) for v in m.get("session", [])))
    return len(seen)


def mean_vel(tag: str, pid: str, track: int) -> float:
    return st.mean(n["velocity"] for seed in range(1, 17) for m in truth(tag, pid, seed) for n in m["notes"] if n["track"] == track)


def fmt(x: float, d: int = 2) -> str:
    return f"{x:.{d}f}".replace(".", ",")


def main() -> None:
    print("### Point 2 — vélocités (16 graines × 47 mesures)\n")
    print("| Profil | Accords : écart-type min (avant → après) | Accords : médiane des écarts-types | Accords : niveau moyen | Basse : écart-type min | Basse : niveau moyen | Plage accords + basse après |")
    print("|---|---|---|---|---|---|---|")
    for pid in E1:
        a, b = A[pid]["vel"], B[pid]["vel"]
        lo = min(b["chords"]["vmin"], b["bass"]["vmin"]); hi = max(b["chords"]["vmax"], b["bass"]["vmax"])
        print(f"| `{pid}` | {fmt(a['chords']['std_min'], 1)} → **{fmt(b['chords']['std_min'], 1)}** | {fmt(a['chords']['std_med'], 1)} → {fmt(b['chords']['std_med'], 1)} | "
              f"{fmt(mean_vel('avant', pid, 1), 1)} → {fmt(mean_vel('apres', pid, 1), 1)} | {fmt(a['bass']['std_min'], 1)} → {fmt(b['bass']['std_min'], 1)} | "
              f"{fmt(mean_vel('avant', pid, 2), 1)} → {fmt(mean_vel('apres', pid, 2), 1)} | {lo} à {hi} |")
    print("\n### Point 3 — groove de batterie (médiane sur 16 graines des fenêtres de la longueur médiane des fichiers GMD)\n")
    print("| Profil | GMD (fichiers ; mesures) | Part de la mesure la plus fréquente | Mesures distinctes / mesures | Identique à la précédente | Écart max au GMD après |")
    print("|---|---|---|---|---|---|")
    for pid in E1:
        r, a, b = B[pid]["ref"], A[pid]["groove"], B[pid]["groove"]
        gap = max(abs(b["top"] - r["top"]), abs(b["distinct"] - r["distinct"]), abs(b["same"] - r["same"]))
        mark = "✓" if gap <= 0.05 + 1e-9 else "✗"
        print(f"| `{pid}` | {r['files']} ; {r['bars']} | {fmt(r['top'])} : {fmt(a['top'])} → **{fmt(b['top'])}** | {fmt(r['distinct'])} : {fmt(a['distinct'])} → **{fmt(b['distinct'])}** | "
              f"{fmt(r['same'])} : {fmt(a['same'])} → **{fmt(b['same'])}** | {fmt(gap)} {mark} |")
    print("\n### Point 4 — mise en place (ms ; attaques simultanées sur 16 graines)\n")
    print("| Profil | Basse − grosse caisse : médiane ; écart-type (avant → après) | 1re corde − caisse claire : médiane ; écart-type | Barycentre des cordes − caisse claire : médiane | Dérive de la grille p95 : accords, basse, batterie max (après) |")
    print("|---|---|---|---|---|")
    for pid in E1:
        a, b = A[pid]["timing"], B[pid]["timing"]
        def cell(key: str) -> str:
            if key not in b:
                return "—"
            x = b[key]; y = a.get(key)
            old = f"{y['med']:+.1f} ; {y['sd']:.1f} → " if y else ""
            return f"{old}**{x['med']:+.1f} ; {x['sd']:.1f}**"
        drift = B[pid]["drift"]
        drums = max(drift.get(k, {"p95": 0})["p95"] for k in ("kick", "snare", "tk"))
        print(f"| `{pid}` | {cell('bk')} | {cell('cs')} | {b['ccs']['med']:+.1f} | {drift['chord']['p95']:.1f}, {drift['bass']['p95']:.1f}, {drums:.1f} |".replace(".", ","))
    print("\n### Habillages distincts sur 16 graines (premier tirage de session)\n")
    print("| Profil | avant | après |")
    print("|---|---:|---:|")
    for pid in E1:
        print(f"| `{pid}` | {dressings('avant', pid)} | {dressings('apres', pid)} |")
    for tag in ("avant", "apres"):
        f = W / f"dist13-{tag}.json"
        if not f.exists():
            return
    DA, DB = json.loads((W / "dist13-avant.json").read_text()), json.loads((W / "dist13-apres.json").read_text())
    print("\n### Distance de la batterie rendue au GMD (méthode de `b8/distance_gmd.py`, Bray-Curtis, plus bas = plus proche)\n")
    print("| Profil | toutes voix | grosse caisse | caisse claire | charleston | moyenne (avant → après) |")
    print("|---|---|---|---|---|---|")
    for pid in E1:
        a, b = DA[pid], DB[pid]
        print(f"| `{pid}` | {fmt(a['any'], 3)} → {fmt(b['any'], 3)} | {fmt(a['kick'], 3)} → {fmt(b['kick'], 3)} | {fmt(a['snare'], 3)} → {fmt(b['snare'], 3)} | "
              f"{fmt(a['hat'], 3)} → {fmt(b['hat'], 3)} | {fmt(a['moyenne'], 3)} → **{fmt(b['moyenne'], 3)}** |")


main()
