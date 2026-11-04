#!/usr/bin/env python3
"""B3 : mixage de `gmd_latin_brazilian_baiao`, hors ligne et en silence (rendus WAV + LUFS intégré `ffmpeg ebur128`).

Commande (depuis n'importe où ; le CLI se lance depuis `~/Project/bascanada/harmonium` pour `assets/`) :

    python3 specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_mix.py \\
        --profile profiles/gmd_latin_brazilian_baiao [--cli <harmonium-cli>] [--out mix.json]

Mesure, par piste en solo (`--solo-track`), le LUFS intégré de 48 mesures, graines 1 et 2, puis le mix complet et sa
crête vraie (`ebur128=peak=true`). Cibles de service (aucun corpus ne donne un mixage ; mêmes que B8) :
accords -26 LUFS, basse -25, batterie (grosse caisse + reste du kit, somme en puissance) -25,5, mix vers -21 à -22,
crête vraie <= -1 dBFS. Affiche le niveau de piste (`level_db`) qui ramènerait chaque piste à sa cible :
`niveau actuel + cible - mesure`. Chaque WAV (20 Mo) est supprimé après la mesure.
"""

from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
from pathlib import Path

ENGINE = Path.home() / "Project/bascanada/harmonium"
CLI = Path(__file__).resolve().parents[3] / "target/release/harmonium-cli"
CIBLES = {"Chords": -26.0, "Bass": -25.0, "kit": -25.5}
WORK = Path("/tmp/harmonium-019/b3/baiao/wav")


def lufs(wav: Path) -> tuple[float, float]:
    r = subprocess.run(["ffmpeg", "-nostats", "-i", str(wav), "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True, check=False)
    lu = re.findall(r"\n\s+I:\s+(-?[\d.]+) LUFS", r.stderr)
    peak = re.findall(r"Peak:\s+(-?[\d.]+) dBFS", r.stderr)
    return (float(lu[-1]) if lu else -120.0, float(peak[-1]) if peak else 0.0)


def rendre(cli: Path, profil: Path, seed: int, solo: str | None, bars: int) -> tuple[float, float]:
    WORK.mkdir(parents=True, exist_ok=True)
    wav = WORK / f"{profil.name}_{seed}_{solo or 'mix'}.wav"
    cmd = [str(cli), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profil), "--record-wav", str(wav)]
    if solo:
        cmd += ["--solo-track", solo]
    subprocess.run(cmd, cwd=ENGINE, capture_output=True, text=True, check=True)
    mesure = lufs(wav)
    wav.unlink()
    return mesure


def somme_puissance(*v: float) -> float:
    return 10 * math.log10(sum(10 ** (x / 10) for x in v))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--profile", type=Path, required=True)
    ap.add_argument("--cli", type=Path, default=CLI)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--seeds", type=int, nargs="+", default=[1, 2])
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    args.profile, args.cli = args.profile.resolve(), args.cli.resolve()
    texte = (args.profile / "profile.toml").read_text()
    niveau = {t: float(re.search(rf"\[mix\.tracks\.{t}\]\nlevel_db = (-?[\d.]+)", texte).group(1))
              for t in ("Chords", "Bass", "PercussionHigh")}
    par_graine = {}
    for seed in args.seeds:
        solo = {t: rendre(args.cli, args.profile, seed, t, args.bars)[0]
                for t in ("Chords", "Bass", "PercussionLow", "PercussionHigh")}
        complet, crete = rendre(args.cli, args.profile, seed, None, args.bars)
        solo["kit"] = somme_puissance(solo["PercussionLow"], solo["PercussionHigh"])
        solo["mix"], solo["crete_vraie_dBFS"] = complet, crete
        par_graine[seed] = solo
        print(f"graine {seed} : " + "  ".join(f"{k} {v:6.1f}" for k, v in solo.items()))
    moyenne = {k: sum(g[k] for g in par_graine.values()) / len(par_graine) for k in next(iter(par_graine.values()))}
    propose = {
        "Chords": round(niveau["Chords"] + CIBLES["Chords"] - moyenne["Chords"], 1),
        "Bass": round(niveau["Bass"] + CIBLES["Bass"] - moyenne["Bass"], 1),
        "Percussion": round(niveau["PercussionHigh"] + CIBLES["kit"] - moyenne["kit"], 1),
    }
    print("moyenne :", {k: round(v, 1) for k, v in moyenne.items()})
    print("niveaux actuels :", niveau, "; niveaux proposés (level_db) :", propose)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps({"par_graine": par_graine, "moyenne": moyenne, "niveaux_actuels": niveau,
                                        "niveaux_proposes": propose}, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
