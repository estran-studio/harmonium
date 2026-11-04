#!/usr/bin/env python3
"""B3 bossa : LUFS intégrés en solo par piste (hors ligne, en silence : rendus WAV puis `ffmpeg ebur128`).

    python3 <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_mix.py \
        --profile <dossier> --cli <cli> [--seeds 1 2] [--bars 48] [--out mix.json]

Lancé depuis `~/Project/bascanada/harmonium` (pour `assets/`). Par piste (`--solo-track`) et pour le mix : LUFS intégré moyen sur
les graines, 48 mesures, crête vraie (`peak=true`). La batterie est la somme en puissance de `PercussionLow` et `PercussionHigh`.
Cibles de service (aucune mesure de corpus ne donne un mixage) : accords −26, basse −25, batterie −25,5, mix ≈ −21 à −22 LUFS ;
crête vraie ≤ −1 dBFS. Les WAV (20 Mo) sont supprimés après mesure.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
import tempfile
from pathlib import Path


def measure(wav: Path) -> tuple[float, float]:
    r = subprocess.run(["ffmpeg", "-nostats", "-i", str(wav), "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True, check=False)
    lufs = re.findall(r"I:\s+(-?[\d.]+) LUFS", r.stderr)
    peak = re.findall(r"Peak:\s+(-?[\d.]+) dBFS", r.stderr)
    return (float(lufs[-1]) if lufs else -120.0, float(peak[-1]) if peak else -120.0)


def render(cli: Path, profile: Path, seed: int, bars: int, solo: str | None, tmp: Path) -> tuple[float, float]:
    wav = tmp / f"{seed}_{solo or 'mix'}.wav"
    cmd = [str(cli), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profile), "--record-wav", str(wav)]
    if solo:
        cmd += ["--solo-track", solo]
    subprocess.run(cmd, capture_output=True, text=True, check=True)
    out = measure(wav)
    wav.unlink()
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--profile", type=Path, required=True)
    ap.add_argument("--cli", type=Path, required=True)
    ap.add_argument("--seeds", type=int, nargs="+", default=[1, 2])
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    result: dict = {"profil": args.profile.name, "graines": args.seeds, "mesures": args.bars, "pistes": {}}
    with tempfile.TemporaryDirectory(prefix="b3mix_") as t:
        tmp = Path(t)
        for solo in ("Chords", "Bass", "PercussionLow", "PercussionHigh", None):
            runs = [render(args.cli, args.profile, s, args.bars, solo, tmp) for s in args.seeds]
            result["pistes"][solo or "mix"] = {"lufs": round(sum(r[0] for r in runs) / len(runs), 1),
                                               "lufs_par_graine": [r[0] for r in runs],
                                               "crete_vraie_dbfs": max(r[1] for r in runs)}
    p = result["pistes"]
    result["batterie_lufs"] = round(10 * math.log10(10 ** (p["PercussionLow"]["lufs"] / 10) + 10 ** (p["PercussionHigh"]["lufs"] / 10)), 1)
    for k, v in p.items():
        print(f"{k:15} {v['lufs']:6.1f} LUFS  crête vraie {v['crete_vraie_dbfs']:6.1f} dBFS  ({v['lufs_par_graine']})")
    print(f"batterie (somme en puissance) {result['batterie_lufs']} LUFS")
    if args.out:
        args.out.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
