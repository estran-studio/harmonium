#!/usr/bin/env python3
"""Q0 : distance de la batterie rendue au GMD, méthode de `b8/distance_gmd.py` (019), sur les 13 profils d'E1.

Bray-Curtis des fréquences d'attaque par pas (toutes voix, grosse caisse, caisse claire, charleston, ride) entre la batterie rendue
(16 graines × 48 mesures, densité 1) et les mesures `beat` 4/4 du groupe GMD du profil (mêmes groupes que `mesure.py`). Les profils
`gmd_rock`, `gmd_pop`, `gmd_soul`, `gmd_reggae` sont aussi mesurés par `b8/distance_gmd.py` lui-même (voir le compte rendu).

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/020-qualite-des-styles/q0/distance.py <racine des profils> <étiquette>
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(REPO / "specs/019-bibliotheque-accompagnements/b8"))
os.environ.setdefault("H019_GMD_CACHE", "/tmp/h020-q0/gmd_results.pkl")
import distance_gmd as D  # noqa: E402
import gmd_batterie as B  # noqa: E402

#: groupe GMD de chaque profil : styles, signature, feels, tempo maximal (même table que `mesure.py`)
REF = {
    "wjazzd_swing": (["jazz", "jazz/swing"], "4-4", {"swing", "ternary"}, 999),
    "wjazzd_ballad": (["jazz"], "4-4", {"swing", "ternary"}, 112),
    "gmd_jazz_waltz": (["jazz"], "3-4", {"swing", "ternary", "straight"}, 999),
    "gmd_latin_brazilian_bossa": (["latin/brazilian-bossa"], "4-4", {"straight"}, 999),
    "gmd_latin_brazilian_samba": (["latin/brazilian-samba"], "4-4", {"straight"}, 999),
    "gmd_latin_brazilian_baiao": (["latin/brazilian-baiao"], "4-4", {"straight"}, 999),
    "gmd_afrocuban": (["afrocuban", "afrocuban/rhumba"], "4-4", {"straight"}, 999),
    "gmd_funk": (["funk"], "4-4", {"straight"}, 999),
    "gmd_soul": (["soul", "soul/motown", "soul/groove3", "soul/groove4", "soul/groove9", "soul/groove10"], "4-4", {"straight"}, 999),
    "gmd_reggae": (["reggae"], "4-4", {"straight"}, 999),
    "gmd_rock": (["rock"], "4-4", {"straight"}, 999),
    "gmd_pop": (["pop", "pop/soft", "pop/groove7"], "4-4", {"straight"}, 999),
    "gmd_blues_shuffle": (["blues/shuffle", "rock/shuffle", "funk/purdieshuffle"], "4-4", {"ternary"}, 999),
}


def reference(results, pid: str) -> dict[str, list[float]]:
    styles, sig, feels, bpm = REF[pid]
    measures = [m for r in results if r.row.style in styles and r.row.beat_type == "beat" and not r.reject
                and r.row.time_signature == sig and r.feel in feels and r.row.bpm <= bpm for m in r.measures]
    active = [m for m in measures if m.steps]
    out = {}
    for name, roles in D.GMD_ROLES.items():
        out[name] = [sum(1 for m in active if any(s in m.steps.get(r, {}) for r in roles)) / len(active) for s in range(16)]
    out["any"] = [sum(1 for m in active if any(s in slots for slots in m.steps.values())) / len(active) for s in range(16)]
    return out


def main() -> int:
    root, tag = Path(sys.argv[1]), sys.argv[2]
    D.WORK = Path(f"/tmp/h020-q0/dist13/{tag}")
    D.WORK.mkdir(parents=True, exist_ok=True)
    D.CLI = Path(os.environ.get("HARMONIUM_CLI", REPO / "target/release/harmonium-cli"))
    results = B.load_results(Path.home() / "Project/bascanada/harmonium_lab")
    table = {}
    for pid in REF:
        ref = reference(results, pid)
        mine = D.rendered(root / pid, 16, 48, pid)
        row = {m: round(D.bc(mine[m], ref[m]), 3) for m in ("any", "kick", "snare", "hat", "ride")}
        row["moyenne"] = round(sum(row[m] for m in ("any", "kick", "snare", "hat")) / 4, 3)
        table[pid] = row
        print(f"{pid:28} " + "  ".join(f"{k} {v:.3f}" for k, v in row.items()), flush=True)
    Path(f"/tmp/h020-q0/dist13-{tag}.json").write_text(json.dumps(table, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
