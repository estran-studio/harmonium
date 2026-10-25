#!/usr/bin/env python3
"""Régénère les kits d'écoute de la phase 012 avec le moteur courant.

Chaque kit A/B isole **une** couche : A ou B = profil distribué complet, l'autre =
même profil avec la seule couche retirée (sections TOML supprimées dans une copie
temporaire du dossier de profil). Même seed, mêmes banques, même durée ; les deux
versions sont ramenées au même volume moyen (RMS −20 dBFS) et l'ordre est tiré au
hasard (graine fixe). La correspondance est écrite à part.

Exemple :
    cargo build --release -p harmonium-cli
    /Users/william.quintal/Project/bascanada/harmonium_lab/.venv/bin/python \
        specs/012-vie-realisme/kits_ecoute.py \
        --output /Users/william.quintal/Project/bascanada/harmonium_lab/target/ecoute
"""

from __future__ import annotations

import argparse
import json
import random
import shutil
import struct
import subprocess
import sys
import tempfile
import wave
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]

# Couche → préfixes de sections TOML (dans profile.toml) retirées pour la version « sans ».
LAYERS: dict[str, tuple[str, ...]] = {
    "mixage": ("mix",),
    "interpretation": ("performance",),
    "ornements": ("ornaments",),
    "accompagnement": ("accompaniment.bass.passing", "accompaniment.chords.strum"),
    "deuxieme_voix": ("heterophony",),
}
KIT_PROFILES: dict[str, tuple[str, ...]] = {
    "mixage": ("reel_quebecois", "valse_quebecoise", "marche_britannique_coloniale"),
    "interpretation": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise"),
    "ornements": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise"),
    "accompagnement": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise"),
    "deuxieme_voix": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise",
                      "marche_britannique_coloniale"),
}
QUESTIONS = {
    "mixage": "Salle crédible ? Violon devant ? Pieds assez présents ?",
    "interpretation": "Plus vivant ? Effets trop discrets, justes ou trop marqués ?",
    "ornements": "Assez d'ornements ? Trop ? Naturels ?",
    "accompagnement": "Basse et guitare plus humaines ? Grattage trop lent ou trop rapide ?",
    "deuxieme_voix": "La 2ᵉ voix (entrée au 2ᵉ tour) aide-t-elle ? Trop forte ?",
}
SET_PROFILES = ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise")
NEUTRAL_AUTOMATION = """[[keys]]
bar = 1
intensity = 0.5
arousal = 0.5
valence = 0.3
density = 0.5
tension = 0.4
"""


def strip_sections(text: str, prefixes: tuple[str, ...]) -> str:
    """Retire les tables TOML dont le nom commence par un des préfixes."""
    kept, skip = [], False
    for line in text.splitlines(keepends=True):
        stripped = line.strip()
        if stripped.startswith("["):
            name = stripped.strip("[]").strip()
            skip = any(name == p or name.startswith(p + ".") for p in prefixes)
        if not skip:
            kept.append(line)
    return "".join(kept)


def read_wav(path: Path) -> tuple[np.ndarray, int]:
    data = path.read_bytes()
    i, channels, rate, bits = 12, 2, 44100, 32
    while i < len(data):
        cid, size = data[i:i + 4], struct.unpack("<I", data[i + 4:i + 8])[0]
        body = data[i + 8:i + 8 + size]
        if cid == b"fmt ":
            channels, rate = struct.unpack("<HI", body[2:8])
            bits = struct.unpack("<H", body[14:16])[0]
        if cid == b"data":
            if bits == 16:
                samples = np.frombuffer(body, dtype="<i2").astype(np.float64) / 32768.0
            elif bits == 24:
                raw = np.frombuffer(body, dtype=np.uint8).reshape(-1, 3).astype(np.int32)
                val = raw[:, 0] | (raw[:, 1] << 8) | (raw[:, 2] << 16)
                samples = np.where(val & 0x800000, val - 0x1000000, val) / 8388608.0
            else:
                samples = np.frombuffer(body, dtype="<f4").astype(np.float64)
            return samples.reshape(-1, channels), rate
        i += 8 + size + (size & 1)
    raise ValueError(f"{path} : pas de bloc data")


def write_wav16(path: Path, samples: np.ndarray, rate: int) -> None:
    pcm = (np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as out:
        out.setnchannels(samples.shape[1])
        out.setsampwidth(2)
        out.setframerate(rate)
        out.writeframes(pcm.tobytes())


def db(value: float) -> float:
    return 20.0 * float(np.log10(value + 1e-12))


def export(binary: Path, profile_dir: Path, wav: Path, args: argparse.Namespace,
           bars: int, automation: Path | None = None, seed: int | None = None) -> None:
    seed = args.seed if seed is None else seed
    cmd = [str(binary), "--export", "--profile", str(profile_dir), "--seed", str(seed),
           "--bars", str(bars), "--tours", "2", "--soundfont", str(args.soundfont),
           "--feet-soundfont", str(args.feet_soundfont), "--record-wav", str(wav)]
    if automation is not None:
        cmd += ["--automation", str(automation)]
    subprocess.run(cmd, check=True, capture_output=True, timeout=900)


def build_ab_kits(args: argparse.Namespace, tmp: Path) -> list[dict]:
    rng = random.Random(args.seed * 1000 + 12)
    report = []
    for kit, prefixes in LAYERS.items():
        out = args.output / kit
        out.mkdir(parents=True, exist_ok=True)
        mapping = []
        for profile in KIT_PROFILES[kit]:
            source = ROOT / "profiles" / profile
            stripped = tmp / kit / profile
            shutil.copytree(source, stripped)
            toml = stripped / "profile.toml"
            before_text = toml.read_text(encoding="utf-8")
            after_text = strip_sections(before_text, prefixes)
            if after_text == before_text:
                raise SystemExit(f"{kit}/{profile} : aucune section {prefixes} à retirer")
            toml.write_text(after_text, encoding="utf-8")
            # Seed où la couche s'entend le plus (écart d'énergie avec/sans), parmi `--seeds` seeds.
            best = None
            for seed in range(args.seed, args.seed + args.seeds):
                full_wav = tmp / f"{kit}-{profile}-{seed}-avec.wav"
                without_wav = tmp / f"{kit}-{profile}-{seed}-sans.wav"
                export(args.binary, source, full_wav, args, args.bars, seed=seed)
                export(args.binary, stripped, without_wav, args, args.bars, seed=seed)
                with_samples, rate = read_wav(full_wav)
                without_samples, _ = read_wav(without_wav)
                frames = min(len(with_samples), len(without_samples))
                a, b = with_samples[:frames], without_samples[:frames]
                change = db(float(np.sqrt(np.mean((a - b) ** 2)))) - db(float(np.sqrt(np.mean(a ** 2))))
                if best is None or change > best[0]:
                    best = (change, seed, rate, a, b, db(float(np.abs(with_samples).max())))
            change, seed, rate, with_part, without_part = best[:5]
            if change < -60.0:
                raise SystemExit(f"{kit}/{profile} : la couche ne change rien au rendu")
            versions = {"avec": with_part, "sans": without_part}
            order = ["avec", "sans"]
            rng.shuffle(order)
            for letter, version in zip("AB", order):
                samples = versions[version]
                gain = 10.0 ** ((-20.0 - db(float(np.sqrt(np.mean(samples ** 2))))) / 20.0)
                write_wav16(out / f"{profile}-{letter}.wav", samples * gain, rate)
                mapping.append(f"{profile} {letter} = {version} {kit} (seed {seed})")
            report.append({"kit": kit, "profile": profile, "seed": seed,
                           "seconds": round(len(with_part) / rate, 1),
                           "difference_db": round(change, 1), "peak_with_dbfs": round(best[5], 2)})
        (out / "correspondance.txt").write_text("\n".join(mapping) + "\n", encoding="utf-8")
        (out / "question.txt").write_text(
            f"{QUESTIONS[kit]}\nÉcouter A puis B ; noter la préférence avant d'ouvrir correspondance.txt.\n",
            encoding="utf-8")
    return report


def build_sets(args: argparse.Namespace, tmp: Path) -> list[dict]:
    out = args.output / "sets"
    out.mkdir(parents=True, exist_ok=True)
    automation = tmp / "neutre.toml"
    automation.write_text(NEUTRAL_AUTOMATION, encoding="utf-8")
    report = []
    for profile in SET_PROFILES:
        wav = tmp / f"set-{profile}.wav"
        export(args.binary, ROOT / "profiles" / profile, wav, args, args.set_bars, automation)
        samples, rate = read_wav(wav)
        write_wav16(out / f"{profile}.wav", samples, rate)
        report.append({"kit": "sets", "profile": profile, "seconds": round(len(samples) / rate, 1),
                       "peak_dbfs": round(db(float(np.abs(samples).max())), 2)})
    return report


# Ordre d'ajout des couches dans la progression (du moteur nu au moteur complet).
PROGRESSION = ("mixage", "interpretation", "ornements", "accompagnement", "deuxieme_voix")
PROGRESSION_PROFILES = ("reel_quebecois", "valse_quebecoise")


def build_progression(args: argparse.Namespace, tmp: Path) -> list[dict]:
    """Même air, couches ajoutées une à une : 0 = nu, 5 = complet (fichiers nommés, pas à l'aveugle)."""
    out = args.output / "progression"
    out.mkdir(parents=True, exist_ok=True)
    report = []
    for profile in PROGRESSION_PROFILES:
        source = ROOT / "profiles" / profile
        text = (source / "profile.toml").read_text(encoding="utf-8")
        for step in range(len(PROGRESSION) + 1):
            removed = tuple(p for layer in PROGRESSION[step:] for p in LAYERS[layer])
            variant = tmp / "progression" / profile / str(step)
            shutil.copytree(source, variant)
            (variant / "profile.toml").write_text(strip_sections(text, removed), encoding="utf-8")
            wav = tmp / f"prog-{profile}-{step}.wav"
            export(args.binary, variant, wav, args, args.bars)
            samples, rate = read_wav(wav)
            gain = 10.0 ** ((-20.0 - db(float(np.sqrt(np.mean(samples ** 2))))) / 20.0)
            label = "nu" if step == 0 else "+" + PROGRESSION[step - 1]
            write_wav16(out / f"{profile}-{step}-{label.lstrip('+')}.wav", samples * gain, rate)
            report.append({"kit": "progression", "profile": profile, "step": step, "label": label,
                           "seconds": round(len(samples) / rate, 1)})
    (out / "question.txt").write_text(
        "Écouter 0 → 5 dans l'ordre : 0 = moteur nu, chaque fichier ajoute une couche "
        "(mixage, interprétation, ornements, accompagnement, 2ᵉ voix). "
        "Laquelle s'entend ? Laquelle ne change rien pour toi ?\n", encoding="utf-8")
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/harmonium-cli")
    parser.add_argument("--soundfont", type=Path, default=ROOT / "assets/GeneralUser-GS.sf2")
    parser.add_argument("--feet-soundfont", type=Path, default=ROOT / "assets/harmonium_trad.sf2")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=3, help="première seed essayée")
    parser.add_argument("--seeds", type=int, default=10,
                        help="nombre de seeds essayées par kit (on garde la plus audible)")
    parser.add_argument("--bars", type=int, default=66)
    parser.add_argument("--set-bars", type=int, default=300)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="harmonium-kits-") as name:
        tmp = Path(name)
        report = build_ab_kits(args, tmp) + build_progression(args, tmp) + build_sets(args, tmp)
    (args.output / "kits.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n",
                                           encoding="utf-8")
    for row in report:
        print(row)
    return 0


if __name__ == "__main__":
    sys.exit(main())
