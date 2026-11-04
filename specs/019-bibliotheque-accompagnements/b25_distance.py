#!/usr/bin/env python3
"""B2 + B5 : distance de la batterie rendue de `wjazzd_ballad` et `gmd_jazz_waltz` au Groove MIDI Dataset.

Même métrique que `b8/distance_gmd.py` (Bray-Curtis des fréquences d'attaque par pas : `any`, `kick`, `snare`,
`hat`, `ride`), sur les mesures `beat` du sous-ensemble du profil (`b25_batterie.SUBSETS`, tous les groupes, pas
seulement ceux retenus). Copie témoin : le même profil sans arc, sans session, dont `percussion.toml` est remplacé par celui de
`wjazzd_swing` (ballade, 16 pas) ou de la fixture `grille_valse_jazz` (valse, 12 pas : valeurs d'exercice).

    cd ~/Project/bascanada/harmonium_lab
    .venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b25_distance.py [--out f.json]
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import b25_batterie as B  # noqa: E402

REPO = HERE.parents[1]
CLI = REPO / "target/release/harmonium-cli"
ENGINE_CWD = Path.home() / "Project/bascanada/harmonium"
WORK = Path("/tmp/harmonium-019/b25/distance")
ROLES = {"kick": {36, 35}, "snare": {37, 38, 39, 40}, "hat": {42, 44, 46}, "ride": {51, 53, 59}}
GMD_ROLES = {"kick": {"kick"}, "snare": {"snare"}, "hat": {"hat_pedal", "hat_open"}, "ride": {"ride"}}
WITNESS = {"wjazzd_ballad": REPO / "profiles/wjazzd_swing/percussion.toml",
           "gmd_jazz_waltz": REPO / "tests/fixtures/profiles/grille_valse_jazz/percussion.toml"}
BARS = {"wjazzd_ballad": 96, "gmd_jazz_waltz": 128}


def bc(a, b):
    den = sum(x + y for x, y in zip(a, b))
    return sum(abs(x - y) for x, y in zip(a, b)) / den if den else 0.0


def reference(profile: str) -> dict[str, list[float]]:
    cfg = B.SUBSETS[profile]
    beats, _ = B.load(cfg)
    n = B.steps_per_bar(cfg)
    bars = [b for _, bs, _ in beats for b in bs if b.steps]
    out = {name: [sum(1 for b in bars if any(s in b.steps.get(r, {}) for r in roles)) / len(bars) for s in range(n)]
           for name, roles in GMD_ROLES.items()}
    out["any"] = [sum(1 for b in bars if any(s in slots for slots in b.steps.values())) / len(bars) for s in range(n)]
    return out


def rendered(profile_dir: Path, tag: str, n: int, bars: int, seeds: int) -> dict[str, list[float]]:
    measures = []
    for seed in range(1, seeds + 1):
        truth = WORK / f"{tag}_{seed}.json"
        if not truth.is_file():
            subprocess.run([str(CLI), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profile_dir),
                            "--record-truth", str(truth)], cwd=ENGINE_CWD, capture_output=True, text=True, check=True)
        last = {}
        for rec in json.loads(truth.read_text())["measures"]:
            last[rec["measure"]["index"]] = rec["measure"]
        for index in sorted(last):
            if index > bars - 1:
                continue
            notes = [(x["start_step"], x["pitch"]) for x in last[index]["notes"] if x["track"] in (3, 4)]
            if notes:
                measures.append(notes)
    out = {name: [sum(1 for m in measures if any(s == step and p in pitches for s, p in m)) / len(measures)
                  for step in range(n)] for name, pitches in ROLES.items()}
    out["any"] = [sum(1 for m in measures if any(s == step for s, _ in m)) / len(measures) for step in range(n)]
    return out


def witness(profile: str) -> Path:
    src = REPO / "profiles" / profile
    dst = WORK / f"temoin_{profile}"
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    shutil.copy(WITNESS[profile], dst / "percussion.toml")
    # Sans arc ni session : leurs étiquettes de batterie n'existent pas dans les patterns du témoin.
    (dst / "arc.toml").unlink()
    (dst / "session.toml").unlink()
    text = (dst / "profile.toml").read_text()
    text = re.sub(r'(?m)^(arc|session) = "[a-z]+.toml"\n', "", text)
    (dst / "profile.toml").write_text(text)
    return dst


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--out", type=Path, default=Path("/tmp/harmonium-019/b25/distance-gmd.json"))
    args = ap.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    table = {}
    for profile in B.SUBSETS:
        n = B.steps_per_bar(B.SUBSETS[profile])
        ref = reference(profile)
        mine = rendered(REPO / "profiles" / profile, profile, n, BARS[profile], args.seeds)
        base = rendered(witness(profile), "temoin_" + profile, n, BARS[profile], args.seeds)
        row = {m: {"profil": round(bc(mine[m], ref[m]), 3), "temoin": round(bc(base[m], ref[m]), 3)}
               for m in ("any", "kick", "snare", "hat", "ride")}
        row["moyenne"] = {k: round(sum(row[m][k] for m in ("any", "kick", "snare", "hat")) / 4, 3) for k in ("profil", "temoin")}
        table[profile] = row
        print(profile, json.dumps(row, ensure_ascii=False))
    args.out.write_text(json.dumps(table, indent=2, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
