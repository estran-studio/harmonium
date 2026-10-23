#!/usr/bin/env python3
"""Mesure reproductible V1 : WAV float32 du CLI, seed 3, 34 mesures, deux tours.

Exécuter avec harmonium_lab/.venv/bin/python (numpy + bibliothèque standard).
Le binaire « before » doit être main avec seulement --solo-track ajouté.
Les silences d'arrangement sont inclus dans le RMS ; aucune normalisation.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import math
from pathlib import Path
import re
import shutil
import struct
import subprocess
import tempfile
import tomllib

import numpy as np

TRACKS = ("Melody", "Chords", "Bass", "Heel", "Toe")
GM = "/Users/william.quintal/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2"
FEET = "/Users/william.quintal/Project/bascanada/harmonium/assets/harmonium_trad.sf2"


def db(value):
    return 20 * math.log10(float(value)) if value > 0 else None


def samples(path):
    raw = path.read_bytes()
    if raw[:4] != b"RIFF" or raw[8:12] != b"WAVE":
        raise ValueError(f"{path}: RIFF/WAVE requis")
    offset, fmt, data = 12, None, None
    while offset + 8 <= len(raw):
        name, size = struct.unpack_from("<4sI", raw, offset)
        chunk = raw[offset + 8:offset + 8 + size]
        if name == b"fmt ":
            fmt = struct.unpack_from("<HHIIHH", chunk)
            if fmt[0] == 65534 and struct.unpack_from("<I", chunk, 24)[0] == 3:
                fmt = (3, *fmt[1:])
        elif name == b"data":
            data = chunk
        offset += 8 + size + size % 2
    if fmt is None or data is None or fmt[0] != 3 or fmt[1] != 2 or fmt[5] != 32:
        raise ValueError(f"{path}: float32 stéréo requis ({fmt})")
    return np.frombuffer(data, dtype="<f4").reshape(-1, 2).astype(np.float64)


def metrics(path):
    x = samples(path)
    energy = np.mean(x * x, axis=0)
    rms = np.sqrt(np.mean(energy))
    correlation = float(np.corrcoef(x.T)[0, 1]) if np.all(energy > 0) else None
    return {"rms_dbfs": db(rms), "peak_dbfs": db(np.max(np.abs(x))),
            "left_minus_right_db": db(np.sqrt(energy[0] / energy[1])) if np.all(energy > 0) else None,
            "correlation_lr": correlation, "frames": len(x),
            "clipped_samples": int(np.count_nonzero(np.abs(x) >= 1)),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "wav": str(path)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--stage", choices=("before", "after"), required=True)
    parser.add_argument("--profiles-root", type=Path, default=Path("profiles"))
    parser.add_argument("--output", type=Path, default=Path("/tmp/harmonium-mix"))
    parser.add_argument("--soundfont", default=GM)
    parser.add_argument("--feet-soundfont", default=FEET)
    parser.add_argument("--jobs", type=int, default=2)
    parser.add_argument("--bars", type=int, default=34, help="34 : contrat ; 66 : complément avec pieds au tour 2")
    parser.add_argument("--check-acceptance", action="store_true")
    parser.add_argument("--reuse", action="store_true", help="Réanalyser les WAV existants sans les rendre")
    args = parser.parse_args()
    binary = args.binary.resolve()
    root = args.output.resolve() / args.stage
    root.mkdir(parents=True, exist_ok=True)
    profiles = sorted(p.parent for p in args.profiles_root.glob("*/profile.toml"))
    if len(profiles) != 7:
        raise ValueError(f"Sept profils attendus, trouvés : {len(profiles)}")
    temporary = tempfile.TemporaryDirectory(prefix="harmonium-mix-dry-")
    dry_profiles = {}
    baseline_profiles = {}
    if args.stage == "before":
        # Le binaire main ne connaît pas [mix] : retirer uniquement ses tables,
        # dans une copie temporaire ; toutes les données de génération restent identiques.
        for profile in profiles:
            baseline = Path(temporary.name) / profile.name
            shutil.copytree(profile, baseline)
            config = baseline / "profile.toml"
            lines, in_mix = [], False
            for line in config.read_text().splitlines(keepends=True):
                if line.lstrip().startswith("["):
                    in_mix = re.match(r"\s*\[mix(?:\.|\])", line) is not None
                if not in_mix:
                    lines.append(line)
            config.write_text("".join(lines))
            baseline_profiles[profile.name] = baseline
    if args.stage == "after":
        for profile in profiles:
            dry = Path(temporary.name) / profile.name
            shutil.copytree(profile, dry)
            config = dry / "profile.toml"
            text, count = re.subn(r"(\[mix\.room\][^\[]*?\blevel\s*=\s*)[-+\d.e]+",
                                  r"\g<1>0.0", config.read_text())
            if count != 1:
                raise ValueError(f"{config}: section mix.room explicite requise")
            config.write_text(text)
            dry_profiles[profile.name] = dry

    def run_render(profile, track, path):
        command = [str(binary), "--export", "--profile", str(profile.resolve()),
                   "--seed", "3", "--bars", str(args.bars), "--tours", "2", "--soundfont", args.soundfont,
                   "--feet-soundfont", args.feet_soundfont, "--record-wav", str(path)]
        if track != "Mix":
            command += ["--solo-track", track]
        if not args.reuse:
            result = subprocess.run(command, text=True, capture_output=True)
            path.with_suffix(".log").write_text(result.stdout + result.stderr)
            result.check_returncode()

    def render(job):
        profile, track = job
        path = root / f"{profile.name}-{track}.wav"
        run_render(baseline_profiles.get(profile.name, profile), track, path)
        measured = metrics(path)
        measured["audible"] = measured["rms_dbfs"] is not None and measured["rms_dbfs"] > -90
        if args.stage == "after":
            dry_path = root / f"{profile.name}-{track}-dry.wav"
            run_render(dry_profiles[profile.name], track, dry_path)
            wet, dry = samples(path), samples(dry_path)
            dry_energy = np.sum(dry ** 2)
            measured["reverb_over_dry_db"] = (
                db(np.sqrt(np.sum((wet - dry) ** 2) / dry_energy))
                if measured["audible"] and dry_energy > 0 else None)
            measured["dry_wav"] = str(dry_path)
            if track == "Mix":
                repeat = root / f"{profile.name}-Mix-repeat.wav"
                run_render(profile, track, repeat)
                measured["deterministic"] = repeat.read_bytes() == path.read_bytes()
        return profile.name, track, measured

    report = {"stage": args.stage, "seed": 3, "bars": args.bars, "tours": 2,
              "soundfont": args.soundfont, "feet_soundfont": args.feet_soundfont, "profiles": {}}
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        for profile, track, measured in pool.map(render, [(p, t) for p in profiles for t in ("Mix", *TRACKS)]):
            report["profiles"].setdefault(profile, {})[track] = measured
            print(profile, track, json.dumps(measured, ensure_ascii=False), flush=True)
    failures = []
    for name, tracks in report["profiles"].items():
        lead = tracks["Melody"]["rms_dbfs"]
        if args.stage == "after":
            settings = tomllib.loads((args.profiles_root / name / "profile.toml").read_text())["mix"]
            if not tracks["Mix"]["deterministic"]:
                failures.append(f"{name}: deux rendus différents")
            for track in TRACKS:
                measured = tracks[track]
                if not measured["audible"]:
                    continue
                pan = settings["tracks"][track]["pan"]
                balance = measured["left_minus_right_db"]
                if (pan < -0.01 and balance < 0) or (pan > 0.01 and balance > 0) or (pan == 0 and abs(balance) > 0.5):
                    failures.append(f"{name}/{track}: pan {pan}, balance G−D {balance:.2f} dB")
                if settings["tracks"][track]["reverb_send"] > 0 and measured["reverb_over_dry_db"] is None:
                    failures.append(f"{name}/{track}: aucune contribution réverbérée mesurée")
        for track in TRACKS[1:]:
            level = tracks[track]["rms_dbfs"]
            gap = lead - level if level is not None else None
            tracks[track]["melody_minus_track_db"] = gap
            if gap is not None and gap < (8 if track in ("Heel", "Toe") else 4):
                failures.append(f"{name}/{track}: écart {gap:.2f} dB insuffisant")
        peak = tracks["Mix"]["peak_dbfs"]
        if peak is None or not -6 <= peak <= -1 or tracks["Mix"]["clipped_samples"]:
            failures.append(f"{name}: crête {peak}, saturation {tracks['Mix']['clipped_samples']}")
    report["acceptance_failures"] = failures
    output = root / "measurements.json"
    output.write_text(json.dumps(report, indent=2, ensure_ascii=False, allow_nan=False) + "\n")
    print(f"Rapport : {output}")
    temporary.cleanup()
    if args.check_acceptance and failures:
        raise SystemExit("\n".join(failures))


if __name__ == "__main__":
    main()
