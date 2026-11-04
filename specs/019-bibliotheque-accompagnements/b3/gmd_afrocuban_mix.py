#!/usr/bin/env python3
"""B3 afro-cubain : calage du mixage de `gmd_afrocuban` (hors ligne, en silence : rendus WAV + LUFS intégré ffmpeg).

Mesure, par piste en solo (`--solo-track`), le LUFS intégré de 48 mesures (graines 1 et 2, densité 1, arc du profil),
puis règle `level_db` de chaque piste vers sa cible :

    accords -26 LUFS, basse -25 LUFS, batterie -25,5 LUFS (somme en puissance de PercussionLow et PercussionHigh)

et écrit les niveaux dans le JSON passé à `gmd_afrocuban_generer.py --mix`. Les deux pistes de percussion partagent
le canal 9 du kit GM : le niveau de `PercussionHigh` s'applique aux deux (celui de `PercussionLow` est sans effet,
il est écrit égal) ; l'équilibre entre grosse caisse et congas graves (`PercussionLow`) et reste du kit
(`PercussionHigh`) se règle par la vélocité de chaque voix dans `percussion.toml`. Les cibles sont des réglages de service
(mêmes que B8) : aucune mesure de corpus ne donne un mixage. Mesure aussi le LUFS du mélange et la crête vraie.

    python3 specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.py --out /tmp/harmonium-019/b3/afrocubain/mix.json \
        [--iterations 3] [--cli <harmonium-cli>] [--lab ~/Project/bascanada/harmonium_lab]
"""
from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
sys.path.insert(0, str(ICI))
import gmd_afrocuban_generer as G  # noqa: E402

MOTEUR = Path.home() / "Project/bascanada/harmonium"
CIBLES = {"Chords": -26.0, "Bass": -25.0, "kit": -25.5}
TRAVAIL = Path("/tmp/harmonium-019/b3/afrocubain/mix")


def lufs(wav: Path) -> tuple[float, float]:
    r = subprocess.run(["ffmpeg", "-nostats", "-i", str(wav), "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True, check=False)
    integre = re.findall(r"I:\s+(-?[\d.]+) LUFS", r.stderr)
    crete = re.findall(r"Peak:\s+(-?[\d.]+) dBFS", r.stderr)
    return (float(integre[-1]) if integre else -120.0, float(crete[-1]) if crete else 0.0)


def rendre(cli: Path, profil: Path, graine: int, solo: str | None) -> tuple[float, float]:
    wav = TRAVAIL / f"{graine}_{solo or 'mix'}.wav"
    cmd = [str(cli), "--export", "--seed", str(graine), "--bars", "48", "--profile", str(profil), "--record-wav", str(wav)]
    if solo:
        cmd += ["--solo-track", solo]
    subprocess.run(cmd, cwd=MOTEUR, capture_output=True, text=True, check=True)
    mesure = lufs(wav)
    wav.unlink()
    return mesure


def somme_puissance(*v: float) -> float:
    return 10 * math.log10(sum(10 ** (x / 10) for x in v))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--iterations", type=int, default=3)
    ap.add_argument("--cli", type=Path, default=ICI.parents[2] / "target/release/harmonium-cli")
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--profile-dir", type=Path, default=TRAVAIL / "profil")
    ap.add_argument("--measure-only", action="store_true")
    args = ap.parse_args()
    TRAVAIL.mkdir(parents=True, exist_ok=True)
    mix = dict(G.MIX)
    if args.out.is_file():
        mix.update(json.loads(args.out.read_text()))
    for tour in range(args.iterations):
        G.construire(args.lab, args.profile_dir, mix)
        mesures = {piste: [rendre(args.cli, args.profile_dir, g, piste) for g in (1, 2)]
                   for piste in ("Chords", "Bass", "PercussionLow", "PercussionHigh")}
        solo = {piste: sum(v[0] for v in vals) / 2 for piste, vals in mesures.items()}
        solo["kit"] = somme_puissance(solo["PercussionLow"], solo["PercussionHigh"])
        melange = [rendre(args.cli, args.profile_dir, g, None) for g in (1, 2)]
        lufs_melange = sum(v[0] for v in melange) / 2
        crete = max(v[1] for v in melange)
        print(f"tour {tour} : accords {solo['Chords']:.1f}, basse {solo['Bass']:.1f}, grosse caisse et congas graves "
              f"{solo['PercussionLow']:.1f}, reste du kit {solo['PercussionHigh']:.1f}, batterie {solo['kit']:.1f}, "
              f"mix {lufs_melange:.1f} LUFS, crête vraie {crete:.1f} dBFS ; niveaux {({k: v for k, v in mix.items() if k != 'master_db'})}")
        if args.measure_only:
            break
        stable = all(abs(solo[p] - CIBLES[p]) < 0.3 for p in CIBLES)
        for piste in ("Chords", "Bass"):
            mix[piste] = round(mix[piste] + CIBLES[piste] - solo[piste], 1)
        mix["PercussionHigh"] = round(mix["PercussionHigh"] + CIBLES["kit"] - solo["kit"], 1)
        mix["PercussionLow"] = mix["PercussionHigh"]
        args.out.write_text(json.dumps(mix, indent=2, sort_keys=True) + "\n")
        if stable:
            break
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
