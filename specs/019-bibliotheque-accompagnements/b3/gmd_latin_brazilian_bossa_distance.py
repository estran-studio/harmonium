#!/usr/bin/env python3
"""B3 bossa : distance de la batterie rendue de `gmd_latin_brazilian_bossa` au Groove MIDI Dataset (hors ligne, en silence).

`make style-distance` mesure contre des pistes AUDIO et leurs transcriptions : le GMD est du MIDI de batterie sans audio, l'outil
ne s'y applique pas (même constat que B8 et B4 + B7). Cette distance reprend le modèle de `b8/distance_gmd.py` : Bray-Curtis des
fréquences d'attaque par pas (16 pas), pour « un coup quelconque », la grosse caisse, la caisse claire (cross-stick compris), le
charleston (fermé, pédale, ouvert) et le ride, contre TOUTES les mesures `beat` 4/4 droites du style (631 mesures, 1 batteur).

VOIX LATINES EXCLUES : le shaker GM (note 82), règle de métier sans équivalent dans le GMD (aucune percussion latine dans le GMD),
est retiré du rendu avant la mesure ; il n'entre dans aucune des cinq fréquences. Les toms (47) restent dans « un coup quelconque ».

Trois rendus (16 graines × 48 mesures, densité neutre, arc compris sauf mention) :
  profil       : le profil livré ;
  sans_arc     : le même sans `arc.toml` ni `session.toml` (une seule étiquette de batterie, pas d'exposition ni de sommet) ;
  non_calibre  : le même sans arc ni session, `percussion.toml` de `wjazzd_swing` (batterie jazz calibrée pour un autre style).

    cd ~/Project/bascanada/harmonium_lab
    .venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_distance.py --out /tmp/d.json
Lancé avec le venv du labo ; n'écrit rien dans le labo ni dans le dépôt (JSON dans `--out`, rendus dans `/tmp/harmonium-019/b3/bossa/`).
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
import gmd_batterie as B  # noqa: E402

REPO = HERE.parents[2]
ID = "gmd_latin_brazilian_bossa"
CLI = Path(os.environ.get("HARMONIUM_CLI", REPO / "target/release/harmonium-cli"))
ENGINE_CWD = Path(os.environ.get("HARMONIUM_CWD", Path.home() / "Project/bascanada/harmonium"))
WORK = Path("/tmp/harmonium-019/b3/bossa/distance")
ROLES = {"kick": {36, 35}, "snare": {37, 38, 39, 40}, "hat": {42, 44, 46}, "ride": {51, 53, 59}}
GMD_ROLES = {"kick": {"kick"}, "snare": {"snare", "sidestick"}, "hat": {"hat_closed", "hat_pedal", "hat_open"},
             "ride": {"ride", "ride_bell"}}
LATIN = {82}


def bc(a: list[float], b: list[float]) -> float:
    num = sum(abs(x - y) for x, y in zip(a, b))
    den = sum(x + y for x, y in zip(a, b))
    return num / den if den else 0.0


def reference(results: list[Any]) -> tuple[dict[str, list[float]], int]:
    measures = [m for r in results if r.row.style == "latin/brazilian-bossa" and r.row.beat_type == "beat" and not r.reject
                and r.row.time_signature == "4-4" and r.feel == "straight" for m in r.measures]
    active = [m for m in measures if m.steps]
    out: dict[str, list[float]] = {}
    for name, roles in GMD_ROLES.items():
        out[name] = [sum(1 for m in active if any(s in m.steps.get(r, {}) for r in roles)) / len(active) for s in range(16)]
    out["any"] = [sum(1 for m in active if any(s in slots for slots in m.steps.values())) / len(active) for s in range(16)]
    return out, len(active)


def rendered(profile: Path, seeds: int, bars: int, tag: str) -> tuple[dict[str, list[float]], int]:
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
            notes = [(n["start_step"], n["pitch"]) for n in last[index]["notes"] if n["track"] in (3, 4) and n["pitch"] not in LATIN]
            if notes:
                measures.append(notes)
    out = {name: [sum(1 for m in measures if any(s == step and p in pitches for s, p in m)) / len(measures) for step in range(16)]
           for name, pitches in ROLES.items()}
    out["any"] = [sum(1 for m in measures if any(s == step for s, _ in m)) / len(measures) for step in range(16)]
    return out, len(measures)


def copy_plain(profile: Path, name: str, percussion: Path | None = None) -> Path:
    dst = WORK / name
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(profile, dst)
    if percussion:
        shutil.copy(percussion, dst / "percussion.toml")
    for f in ("arc.toml", "session.toml"):
        (dst / f).unlink()
    text = (dst / "profile.toml").read_text()
    (dst / "profile.toml").write_text(re.sub(r'(?m)^(arc|session) = "[a-z]+\.toml"\n', "", text))
    return dst


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    results = B.load_results(args.lab)
    profile = REPO / "profiles" / ID
    ref, n_ref = reference(results)
    runs = {"profil": rendered(profile, args.seeds, args.bars, "profil"),
            "sans_arc": rendered(copy_plain(profile, "plain"), args.seeds, args.bars, "sans_arc"),
            "non_calibre": rendered(copy_plain(profile, "uncal", REPO / "profiles/wjazzd_swing/percussion.toml"), args.seeds, args.bars, "non_calibre")}
    row: dict[str, Any] = {"mesures_reference": n_ref, "mesures_rendues": {k: v[1] for k, v in runs.items()}}
    for name in ("any", "kick", "snare", "hat", "ride"):
        row[name] = {k: round(bc(v[0][name], ref[name]), 3) for k, v in runs.items()}
    row["moyenne"] = {k: round(sum(row[m][k] for m in ("any", "kick", "snare", "hat")) / 4, 3) for k in runs}
    row["frequences_par_pas"] = {"reference": {k: [round(x, 2) for x in v] for k, v in ref.items()},
                                 "profil": {k: [round(x, 2) for x in runs["profil"][0][k]] for k in ref}}
    args.out.write_text(json.dumps(row, indent=2, ensure_ascii=False) + "\n")
    for m in ("any", "kick", "snare", "hat", "ride", "moyenne"):
        print(f"{m:8} profil {row[m]['profil']:.3f}  sans_arc {row[m]['sans_arc']:.3f}  non_calibre {row[m]['non_calibre']:.3f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
