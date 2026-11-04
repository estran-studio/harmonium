#!/usr/bin/env python3
"""B6 : distance de la batterie rendue de `gmd_blues_shuffle` au Groove MIDI Dataset, contre une copie non calibrée.

Mêmes métriques que `b8/distance_gmd.py` (Bray-Curtis des fréquences d'attaque par pas : 0 = identique ; grosse caisse,
caisse claire, charleston, ride, toutes voix) mais sur la grille ternaire de douze pas, et pour référence TOUTES les
mesures `beat` 4/4 de feel ternaire de `blues/shuffle`, `rock/shuffle` et `funk/purdieshuffle` (pas seulement les groupes
retenus). Copie non calibrée : le même profil dont `percussion.toml` est celui de la fixture `grille_shuffle_ternaire`
(batterie d'exercice écrite à la main, sans arc ni session). Lancé avec le venv du labo ; n'écrit rien dans le labo.

    cd ~/workspace/harmonium_lab && .venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b6/distance_gmd.py
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "b8"))
os.environ.setdefault("H019_GMD_CACHE", "/tmp/h019-b6/results.pkl")
import gmd_batterie as B  # noqa: E402

REPO = HERE.parents[2]
PROFILE = REPO / "profiles/gmd_blues_shuffle"
FIXTURE = REPO / "tests/fixtures/profiles/grille_shuffle_ternaire/percussion.toml"
CLI = Path(os.environ.get("HARMONIUM_CLI", REPO / "target/release/harmonium-cli"))
ENGINE_CWD = Path(os.environ.get("HARMONIUM_CWD", Path.home() / "workspace/harmonium"))
WORK = Path("/tmp/h019-b6/distance")
STYLES = ["blues/shuffle", "rock/shuffle", "funk/purdieshuffle"]
STEPS = 12
ROLES = {"kick": {36, 35}, "snare": {37, 38, 39, 40}, "hat": {42, 44, 46}, "ride": {51, 53, 59}}
GMD_ROLES = {"kick": {"kick"}, "snare": {"snare", "sidestick"}, "hat": {"hat_closed", "hat_pedal", "hat_open"},
             "ride": {"ride", "ride_bell"}}


def bc(a: list[float], b: list[float]) -> float:
    num = sum(abs(x - y) for x, y in zip(a, b))
    den = sum(x + y for x, y in zip(a, b))
    return num / den if den else 0.0


def reference(results: list[Any]) -> dict[str, list[float]]:
    measures = [m for r in results if r.row.style in STYLES and r.row.beat_type == "beat" and not r.reject
                and r.row.time_signature == "4-4" and r.feel == "ternary" for m in r.measures]
    active = [m for m in measures if m.steps]
    out = {name: [sum(1 for m in active if any(s in m.steps.get(r, {}) for r in roles)) / len(active) for s in range(STEPS)]
           for name, roles in GMD_ROLES.items()}
    out["any"] = [sum(1 for m in active if any(s in slots for slots in m.steps.values())) / len(active) for s in range(STEPS)]
    return out


def rendered(profile: Path, seeds: int, bars: int, tag: str) -> dict[str, list[float]]:
    """Fréquence d'attaque par pas des mesures où la batterie joue, sur `seeds` graines (densité 1 par défaut)."""
    measures: list[list[tuple[int, int]]] = []
    for seed in range(1, seeds + 1):
        truth = WORK / f"{tag}_{seed}.json"
        if not truth.is_file():
            subprocess.run([str(CLI), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profile),
                            "--record-truth", str(truth)], cwd=ENGINE_CWD, capture_output=True, text=True, check=True)
        last: dict[int, dict[str, Any]] = {}
        for rec in json.loads(truth.read_text())["measures"]:
            last[rec["measure"]["index"]] = rec["measure"]
        for index in sorted(last):
            if index > bars - 1:  # la mesure de clôture de l'export joue la tonique et la grosse caisse
                continue
            notes = [(n["start_step"], n["pitch"]) for n in last[index]["notes"] if n["track"] in (3, 4)]
            if notes:
                measures.append(notes)
    out = {name: [sum(1 for m in measures if any(s == step and p in pitches for s, p in m)) / len(measures) for step in range(STEPS)]
           for name, pitches in ROLES.items()}
    out["any"] = [sum(1 for m in measures if any(s == step for s, _ in m)) / len(measures) for step in range(STEPS)]
    return out


def copy_plain(name: str, percussion: Path | None = None) -> Path:
    dst = WORK / name
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(PROFILE, dst)
    if percussion:
        shutil.copy(percussion, dst / "percussion.toml")
    (dst / "arc.toml").unlink()
    (dst / "session.toml").unlink()
    text = (dst / "profile.toml").read_text()
    text = re.sub(r'(?m)^(arc|session) = "[a-z.]+"\n', "", text)
    (dst / "profile.toml").write_text(text)
    return dst


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lab", type=Path, default=Path.home() / "workspace/harmonium_lab")
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--out", type=Path, default=HERE / "preuves/distance-gmd.json")
    args = ap.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    ref = reference(B.load_results(args.lab))
    mine = rendered(PROFILE, args.seeds, args.bars, "profil")
    plain = rendered(copy_plain("sans_arc"), args.seeds, args.bars, "sans_arc")
    base = rendered(copy_plain("non_calibre", FIXTURE), args.seeds, args.bars, "non_calibre")
    row: dict[str, Any] = {}
    for name in ("any", "kick", "snare", "hat", "ride"):
        row[name] = {"profil": round(bc(mine[name], ref[name]), 3), "sans_arc": round(bc(plain[name], ref[name]), 3),
                     "non_calibre": round(bc(base[name], ref[name]), 3)}
    row["moyenne"] = {key: round(sum(row[m][key] for m in ("any", "kick", "snare", "hat")) / 4, 3)
                      for key in ("profil", "sans_arc", "non_calibre")}
    print("  ".join(f"{m} {row[m]['profil']:.3f}/{row[m]['sans_arc']:.3f}/{row[m]['non_calibre']:.3f}" for m in row))
    args.out.write_text(json.dumps(row, indent=2, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
