#!/usr/bin/env python3
"""B8 : calage du mixage des profils `gmd_*` (hors ligne, en silence : rendus WAV + LUFS intégré ffmpeg).

Mesure, par profil et par piste en solo (`--solo-track`), le LUFS intégré de 48 mesures (graines 1 et 2, densité 1,
arc du profil), puis écrit dans `mix.json` les niveaux de piste qui ramènent chaque piste à sa cible :

    accords  -26 LUFS, basse -25 LUFS, batterie (grosse caisse + reste du kit, somme en puissance) -25,5 LUFS

(`wjazzd_swing` : accords -25,8, basse -25,4 ; sa batterie à ride seul est volontairement 8 LU plus bas). Les cibles sont
des réglages de service : aucune mesure de corpus ne donne un mixage. Usage : `calibrer_mix.py [--apply] [id ...]`
(sans `--apply`, affiche seulement les LUFS mesurés ; `--write-lufs` les écrit dans `preuves/mix-lufs.json` ; `generer.py` relit `mix.json` ensuite).
"""
from __future__ import annotations

import json
import math
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ENGINE_CWD = Path(os.environ.get("HARMONIUM_CWD", Path.home() / "Project/bascanada/harmonium"))
CLI = Path(os.environ.get("HARMONIUM_CLI", HERE.parents[2] / "target/release/harmonium-cli"))
PROFILES = HERE.parents[2] / "profiles"
TARGET = {"Chords": -26.0, "Bass": -25.0, "kit": -25.5}
IDS = ["gmd_rock", "gmd_pop", "gmd_hiphop", "gmd_soul", "gmd_reggae", "gmd_dance_disco", "gmd_afrobeat"]


def lufs(wav: Path) -> float:
    r = subprocess.run(["ffmpeg", "-nostats", "-i", str(wav), "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True, check=False)
    found = re.findall(r"I:\s+(-?[\d.]+) LUFS", r.stderr)
    return float(found[-1]) if found else -120.0


def render(pid: str, seed: int, solo: str | None) -> float:
    wav = Path("/tmp/harmonium-019/b8/wav") / f"{pid}_{seed}_{solo or 'mix'}.wav"
    wav.parent.mkdir(parents=True, exist_ok=True)
    cmd = [str(CLI), "--export", "--seed", str(seed), "--bars", "48", "--profile", str(PROFILES / pid), "--record-wav", str(wav)]
    if solo:
        cmd += ["--solo-track", solo]
    subprocess.run(cmd, cwd=ENGINE_CWD, capture_output=True, text=True, check=True)
    value = lufs(wav)
    wav.unlink()  # 20 Mo par rendu : on ne garde que la mesure
    return value


def power_sum(*v: float) -> float:
    return 10 * math.log10(sum(10 ** (x / 10) for x in v))


def main() -> int:
    apply = "--apply" in sys.argv
    ids = [a for a in sys.argv[1:] if not a.startswith("--")] or IDS
    path = HERE / "mix.json"
    mix = json.loads(path.read_text()) if path.is_file() else {}
    measured: dict[str, dict[str, float]] = {}
    for pid in ids:
        got = {s: sum(render(pid, seed, s) for seed in (1, 2)) / 2 for s in ("Chords", "Bass", "PercussionLow", "PercussionHigh")}
        kit = power_sum(got["PercussionLow"], got["PercussionHigh"])
        full = sum(render(pid, seed, None) for seed in (1, 2)) / 2
        measured[pid] = {"Chords": got["Chords"], "Bass": got["Bass"], "PercussionLow": got["PercussionLow"],
                         "PercussionHigh": got["PercussionHigh"], "kit": kit, "mix": full}
        print(f"{pid:18} accords {got['Chords']:6.1f}  basse {got['Bass']:6.1f}  batterie {kit:6.1f} (grosse caisse {got['PercussionLow']:6.1f}, reste {got['PercussionHigh']:6.1f})  mix {full:6.1f}")
        if apply:
            text = (PROFILES / pid / "profile.toml").read_text()
            level = {t: float(re.search(rf"\[mix\.tracks\.{t}\]\nlevel_db = (-?[\d.]+)", text).group(1))
                     for t in ("Chords", "Bass", "PercussionHigh")}
            mix[pid] = {
                "Chords": round(level["Chords"] + TARGET["Chords"] - got["Chords"], 1),
                "Bass": round(level["Bass"] + TARGET["Bass"] - got["Bass"], 1),
                "Percussion": round(level["PercussionHigh"] + TARGET["kit"] - kit, 1),
            }
    if apply:
        path.write_text(json.dumps(mix, indent=2, sort_keys=True) + "\n")
    if "--write-lufs" in sys.argv:
        proofs = HERE / "preuves" / "mix-lufs.json"
        proofs.parent.mkdir(exist_ok=True)
        old = json.loads(proofs.read_text()) if proofs.is_file() else {}
        old.update({k: {n: round(x, 1) for n, x in v.items()} for k, v in measured.items()})
        proofs.write_text(json.dumps(old, indent=2, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
