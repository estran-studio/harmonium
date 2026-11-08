#!/usr/bin/env python3
"""Q0 : les notes les plus douces des accords et de la basse restent audibles sur leur programme (rendu hors ligne, en silence).

Rend chaque profil piste par piste (`--solo-track`, GeneralUser GS, `--record-wav`) et, pour chaque attaque isolée (au moins 250 ms
après la précédente de la piste), mesure le crête des 120 ms qui suivent. Regroupe par vélocité : le niveau en dBFS de chaque tranche,
et son écart au niveau médian de la piste. Aucun son n'est joué ; la lecture du WAV est un calcul.

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/020-qualite-des-styles/q0/audible.py <dossier des profils> [profil...]
"""
from __future__ import annotations

import json
import os
import statistics as st
import subprocess
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
CLI = os.environ.get("HARMONIUM_CLI", str(HERE.parents[2] / "target/release/harmonium-cli"))
SF2 = Path.home() / "Project/bascanada/harmonium/assets/GeneralUser-GS.sf2"
WORK = Path(os.environ.get("H020_WORK", "/tmp/h020-q0")) / "audible"
TRACKS = {"Chords": 1, "Bass": 2}
BUCKETS = [(0, 40), (41, 60), (61, 90), (91, 127)]


def render(profile: Path, seed: int, solo: str) -> tuple[np.ndarray, int, dict]:
    WORK.mkdir(parents=True, exist_ok=True)
    wav, tj = WORK / f"{profile.name}_{seed}_{solo}.wav", WORK / f"{profile.name}_{seed}_{solo}.json"
    subprocess.run([CLI, "--export", "--bars", "24", "--profile", str(profile), "--seed", str(seed), "--solo-track", solo, "-s", str(SF2),
                    "--record-wav", str(wav), "--record-truth", str(tj)], check=True, capture_output=True)
    audio, rate = sf.read(wav, dtype="float32")
    truth = json.loads(tj.read_text())
    wav.unlink()
    tj.unlink()
    return audio, rate, truth


def attacks(truth: dict, track: int, rate: int) -> list[tuple[float, int]]:
    out = []
    starts = {int(k): v for k, v in truth["bar_start_frames"].items()}
    last = {}
    for rec in truth["measures"]:
        last[rec["measure"]["index"]] = rec["measure"]
    for bar, m in sorted(last.items()):
        if bar not in starts or bar + 1 not in starts or bar > 23:
            continue
        frames_per_step = (starts[bar + 1] - starts[bar]) / m["steps"]
        seen = {}
        for n in m["notes"]:
            if n["track"] == track:
                t = (starts[bar] + (n["start_step"] + n.get("performance", {}).get("offset", 0) / 1000.0) * frames_per_step) / rate
                seen[n["start_step"]] = (max(seen.get(n["start_step"], (0, t))[0], n["velocity"]), t)
        out += [(t, v) for v, t in seen.values()]
    return sorted(out)


def main() -> int:
    root = Path(sys.argv[1])
    pids = sys.argv[2:] or sorted(p.name for p in root.iterdir() if (p / "profile.toml").exists())
    print("| Profil | Piste | Vélocité min / max | " + " | ".join(f"{a}–{b}" for a, b in BUCKETS) + " |")
    print("|---|---|---|" + "---|" * len(BUCKETS))
    for pid in pids:
        for solo, track in TRACKS.items():
            peaks: dict[tuple[int, int], list[float]] = {b: [] for b in BUCKETS}
            vmin, vmax = 127, 0
            for seed in (1, 2, 3):
                audio, rate, truth = render(root / pid, seed, solo)
                mono = np.abs(audio).max(axis=1) if audio.ndim == 2 else np.abs(audio)
                att = attacks(truth, track, rate)
                for i, (t, v) in enumerate(att):
                    vmin, vmax = min(vmin, v), max(vmax, v)
                    if i and t - att[i - 1][0] < 0.25:
                        continue
                    a, b = int(t * rate), int((t + 0.12) * rate)
                    if a < 0 or b > len(mono):
                        continue
                    peak = float(mono[a:b].max())
                    for lo, hi in BUCKETS:
                        if lo <= v <= hi:
                            peaks[(lo, hi)].append(20 * np.log10(max(peak, 1e-9)))
            cells = [f"{st.median(p):.1f} dBFS (n={len(p)})" if p else "—" for p in peaks.values()]
            print(f"| `{pid}` | {solo} | {vmin} / {vmax} | " + " | ".join(cells) + " |", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
