#!/usr/bin/env python3
"""B8 : distance de la batterie rendue d'un profil `gmd_*` au Groove MIDI Dataset, contre une copie non calibrée.

`make style-distance` mesure un profil contre des PISTES AUDIO de référence et leurs transcriptions : le GMD est du
MIDI de batterie sans audio ni transcription de basse, l'outil ne s'y applique pas. Cette distance reprend les
métriques de batterie de `harmonium_lab/styledist/symbolic.py` (`drums_any`, `drums_kick`, `drums_snare`, `drums_hat`,
Bray-Curtis des fréquences d'attaque par pas) et ajoute le ride, avec pour référence TOUTES les mesures `beat` 4/4
droites du style (pas seulement les groupes retenus par le profil).

Copie non calibrée : le même profil (grille, basse, comping) dont `percussion.toml` est remplacé par celui de
`wjazzd_swing` (batterie jazz calibrée pour un autre style) et sans arc. Lancé avec le venv du labo ; n'écrit rien
dans le labo ni dans le dépôt (JSON dans `--out`, rendus et copies dans `/tmp/harmonium-019/b8/`).

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b8/distance_gmd.py [--out f.json]
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
sys.path.insert(0, str(HERE))

import gmd_batterie as B  # noqa: E402
from styles import PROFILES  # noqa: E402

REPO = HERE.parents[2]
CLI = Path(os.environ.get("HARMONIUM_CLI", REPO / "target/release/harmonium-cli"))
ENGINE_CWD = Path(os.environ.get("HARMONIUM_CWD", Path.home() / "Project/bascanada/harmonium"))
WORK = Path("/tmp/harmonium-019/b8/distance")
ROLES = {"kick": {36, 35}, "snare": {37, 38, 39, 40}, "hat": {42, 44, 46}, "ride": {51, 53, 59}}
GMD_ROLES = {"kick": {"kick"}, "snare": {"snare", "sidestick"}, "hat": {"hat_closed", "hat_pedal", "hat_open"},
             "ride": {"ride", "ride_bell"}}


def bc(a: list[float], b: list[float]) -> float:
    num = sum(abs(x - y) for x, y in zip(a, b))
    den = sum(x + y for x, y in zip(a, b))
    return num / den if den else 0.0


def reference(results: list[Any], styles: list[str]) -> dict[str, list[float]]:
    measures = [m for r in results if r.row.style in styles and r.row.beat_type == "beat" and not r.reject
                and r.row.time_signature == "4-4" and r.feel == "straight" for m in r.measures]
    active = [m for m in measures if m.steps]
    out: dict[str, list[float]] = {}
    for name, roles in GMD_ROLES.items():
        out[name] = [sum(1 for m in active if any(s in m.steps.get(r, {}) for r in roles)) / len(active) for s in range(16)]
    out["any"] = [sum(1 for m in active if any(s in slots for slots in m.steps.values())) / len(active) for s in range(16)]
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
    out = {name: [sum(1 for m in measures if any(s == step and p in pitches for s, p in m)) / len(measures) for step in range(16)]
           for name, pitches in ROLES.items()}
    out["any"] = [sum(1 for m in measures if any(s == step for s, _ in m)) / len(measures) for step in range(16)]
    return out


def copy_without_arc(profile: Path, name: str, reference_percussion: Path | None = None) -> Path:
    dst = WORK / f"{name}_{profile.name}"
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(profile, dst)
    if reference_percussion:
        shutil.copy(reference_percussion, dst / "percussion.toml")
    (dst / "arc.toml").unlink()
    text = (dst / "profile.toml").read_text()
    (dst / "profile.toml").write_text(re.sub(r'(?m)^arc = "arc.toml"\n', "", text))
    return dst


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--out", type=Path, default=Path("/tmp/harmonium-019/b8/distance-gmd.json"))
    args = ap.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    results = B.load_results(args.lab)
    wjazzd = REPO / "profiles/wjazzd_swing/percussion.toml"
    table: dict[str, Any] = {}
    for p in PROFILES:
        style = p["style"]
        profile = REPO / "profiles" / style.id
        ref = reference(results, p["drum"].styles)
        mine = rendered(profile, args.seeds, args.bars, style.id)
        plain = rendered(copy_without_arc(profile, "sans_arc"), args.seeds, args.bars, "sa_" + style.id)
        base = rendered(copy_without_arc(profile, "non_calibre", wjazzd), args.seeds, args.bars, "nc_" + style.id)
        row = {}
        for name in ("any", "kick", "snare", "hat", "ride"):
            row[name] = {"profil": round(bc(mine[name], ref[name]), 3), "sans_arc": round(bc(plain[name], ref[name]), 3),
                         "non_calibre": round(bc(base[name], ref[name]), 3)}
        for key in ("profil", "sans_arc", "non_calibre"):
            # Moyenne sur les quatre métriques de L2 (le ride n'y figure pas : il manque à la plupart des styles).
            row.setdefault("moyenne", {})[key] = round(sum(row[m][key] for m in ("any", "kick", "snare", "hat")) / 4, 3)
        table[style.id] = row
        print(f"{style.id:18} " + "  ".join(f"{m} {row[m]['profil']:.3f}/{row[m]['sans_arc']:.3f}/{row[m]['non_calibre']:.3f}" for m in row))
    args.out.write_text(json.dumps(table, indent=2, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
