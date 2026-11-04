#!/usr/bin/env python3
"""B3 samba : calage du mixage de `gmd_latin_brazilian_samba` (hors ligne, en silence : WAV exportés + LUFS ffmpeg).

Mesure, par piste en solo (`--solo-track`), le LUFS intégré de 48 mesures (graines 1 et 2, arc et habillage de la
graine), puis le mix complet (LUFS intégré et crête vraie, `ffmpeg ebur128=peak=true`) ; `--apply` réécrit
`[mix.tracks.*] level_db` pour ramener chaque piste à sa cible de service (aucune mesure de corpus) :

    accords -26 LUFS, basse -25 LUFS, batterie (grosse caisse + reste du kit, somme en puissance) -25,5 LUFS ;
    mix complet visé vers -21 / -22 LUFS, crête vraie <= -1 dBFS.

La piste grave (grosse caisse, surdo) et la piste aiguë partagent le canal 9 du kit GM : un seul niveau de
batterie, posé sur les deux pistes. Le CLI se lance depuis `~/Project/bascanada/harmonium` (pour `assets/`).

    python3 gmd_latin_brazilian_samba_mixage.py --profile <dossier> [--apply] [--peaks 16] [--out mix.json]
"""
from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
from pathlib import Path

CLI = Path(__file__).resolve().parents[3] / "target/release/harmonium-cli"
ENGINE = Path.home() / "Project/bascanada/harmonium"
TARGET = {"Chords": -26.0, "Bass": -25.0, "kit": -25.5}
WORK = Path("/tmp/harmonium-019/b3/samba/wav")


def ebur(wav: Path) -> tuple[float, float]:
    r = subprocess.run(["ffmpeg", "-nostats", "-i", str(wav), "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True, check=False)
    loudness = re.findall(r"I:\s+(-?[\d.]+) LUFS", r.stderr)
    peak = re.findall(r"Peak:\s+(-?[\d.]+) dBFS", r.stderr)
    return (float(loudness[-1]) if loudness else -120.0, float(peak[-1]) if peak else 0.0)


def render(profile: Path, seed: int, solo: str | None, bars: int = 48) -> tuple[float, float]:
    WORK.mkdir(parents=True, exist_ok=True)
    wav = WORK / f"{profile.name}_{seed}_{solo or 'mix'}.wav"
    cmd = [str(CLI), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profile),
           "--record-wav", str(wav)]
    if solo:
        cmd += ["--solo-track", solo]
    subprocess.run(cmd, cwd=ENGINE, capture_output=True, text=True, check=True)
    out = ebur(wav)
    wav.unlink()
    return out


def power_sum(*v: float) -> float:
    return 10 * math.log10(sum(10 ** (x / 10) for x in v))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--profile", type=Path, required=True)
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--peaks", type=int, default=16, help="graines du mix complet pour la crête vraie")
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    got = {s: sum(render(args.profile, seed, s)[0] for seed in (1, 2)) / 2
           for s in ("Chords", "Bass", "PercussionLow", "PercussionHigh")}
    kit = power_sum(got["PercussionLow"], got["PercussionHigh"])
    mixes = [render(args.profile, seed, None) for seed in range(1, args.peaks + 1)]
    result = {**{k: round(v, 1) for k, v in got.items()}, "kit": round(kit, 1),
              "mix_graines_1_2": round(sum(m[0] for m in mixes[:2]) / 2, 1),
              "mix_min_max": [round(min(m[0] for m in mixes), 1), round(max(m[0] for m in mixes), 1)],
              "crete_vraie_max_dBFS": round(max(m[1] for m in mixes), 1), "graines_crete": args.peaks}
    print(json.dumps(result, indent=2))
    text = (args.profile / "profile.toml").read_text()
    level = {t: float(re.search(rf"\[mix\.tracks\.{t}\]\nlevel_db = (-?[\d.]+)", text).group(1))
             for t in ("Chords", "Bass", "PercussionLow", "PercussionHigh")}
    new = {"Chords": level["Chords"] + TARGET["Chords"] - got["Chords"],
           "Bass": level["Bass"] + TARGET["Bass"] - got["Bass"],
           "PercussionLow": level["PercussionLow"] + TARGET["kit"] - kit,
           "PercussionHigh": level["PercussionHigh"] + TARGET["kit"] - kit}
    print("niveaux proposés :", {k: round(v, 1) for k, v in new.items()})
    if args.apply:
        for track, db in new.items():
            text = re.sub(rf"(\[mix\.tracks\.{track}\]\nlevel_db = )-?[\d.]+", rf"\g<1>{round(db, 1)}", text)
        (args.profile / "profile.toml").write_text(text)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(result, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
