#!/usr/bin/env python3
"""B3 : niveau des voix latines (triangle, agogô) sur le kit GM de GeneralUser GS, hors ligne et en silence.

Commande :

    python3 specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_calibrage_kit.py \\
        [--profile profiles/gmd_latin_brazilian_baiao] [--program 0] [--cli <harmonium-cli>]

Même méthode que `b47_calibrage_kit.py` (B4 + B7) : un profil d'essai (copie du profil, sans arc ni session) ne joue
qu'une voix, quatre coups par mesure, 16 mesures, à vélocité 50 puis 100 ; `ffmpeg ebur128` donne le LUFS intégré en
solo de la piste `PercussionHigh`. La droite en log2 de la vélocité (L2 vélocité 50 -> 100) donne la vélocité qui
atteint le niveau visé. Les cibles sont celles de B47 pour les rôles voisins (charleston -38, ride -37, cloche -32) :
triangle sous le ride parce qu'il joue huit coups par mesure ; agogô avec la cloche, quatre coups par mesure.
Sortie : une ligne par voix (LUFS à 50 et 100, vélocité proposée), copiée dans le README (« Mixage »). Temporaires
dans `/tmp/harmonium-019/b3/baiao/cal/`, supprimés à la fin.
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
from pathlib import Path

ENGINE = Path.home() / "Project/bascanada/harmonium"
CLI = Path(__file__).resolve().parents[3] / "target/release/harmonium-cli"
WORK = Path("/tmp/harmonium-019/b3/baiao/cal")
#: voix -> (note GM, niveau visé en solo, mêmes conditions : 4 coups par mesure, 96 BPM, 16 mesures).
VOIX = {"triangle_muet": (80, -41.0), "triangle_ouvert": (81, -41.0), "agogo_aigu": (67, -34.0), "agogo_grave": (68, -34.0)}


def lufs(wav: Path) -> float:
    r = subprocess.run(["ffmpeg", "-nostats", "-i", str(wav), "-af", "ebur128", "-f", "null", "-"],
                       capture_output=True, text=True, check=False)
    found = re.findall(r"\n\s+I:\s+(-?[\d.]+) LUFS", r.stderr)
    return float(found[-1]) if found else -120.0


def mesurer(cli: Path, profil: Path, programme: int, nom: str, note: int, vel: int) -> float:
    d = WORK / f"{nom}_{vel}"
    if d.exists():
        shutil.rmtree(d)
    shutil.copytree(profil, d)
    texte = (d / "profile.toml").read_text()
    texte = re.sub(r"percussion_program = \d+", f"percussion_program = {programme}", texte)
    texte = re.sub(r'arc = "arc.toml"\nsession = "session.toml"\n', "", texte)
    (d / "profile.toml").write_text(texte)
    for f in ("arc.toml", "session.toml"):
        (d / f).unlink()
    perc = (f"low_note = 36\nhigh_note = 38\nlow_velocity = 64\nhigh_velocity = 64\npercussion_gain = 1.0\n"
            f"velocity_humanize = 0\n\n[[voices]]\nname = \"{nom}\"\nnote = {note}\nvelocity = {vel}\n"
            f"track = \"PercussionHigh\"\n\n")
    for dens in (0, 1, 2):
        perc += f"[[patterns]]\ndensity = {dens}\nhits = [\n" + "".join(
            f'  {{ step = {s}, voice = "{nom}" }},\n' for s in (0, 4, 8, 12)) + "]\n\n"
    (d / "percussion.toml").write_text(perc)
    wav = WORK / f"{nom}_{vel}.wav"
    subprocess.run([str(cli), "--export", "--seed", "1", "--bars", "16", "--profile", str(d), "--solo-track",
                    "PercussionHigh", "--record-wav", str(wav)], cwd=ENGINE, capture_output=True, check=True)
    valeur = lufs(wav)
    wav.unlink()
    shutil.rmtree(d)
    return valeur


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--profile", type=Path, default=Path(__file__).resolve().parents[3] / "profiles/gmd_latin_brazilian_baiao")
    ap.add_argument("--program", type=int, default=0)
    ap.add_argument("--cli", type=Path, default=CLI)
    args = ap.parse_args()
    args.profile, args.cli = args.profile.resolve(), args.cli.resolve()
    WORK.mkdir(parents=True, exist_ok=True)
    for nom, (note, cible) in VOIX.items():
        l50, l100 = (mesurer(args.cli, args.profile, args.program, nom, note, v) for v in (50, 100))
        par_octave = l100 - l50
        velocite = 50.0 * 2.0 ** ((cible - l50) / par_octave)
        print(f"{nom:16} note {note} : {l50:6.1f} LUFS à 50, {l100:6.1f} à 100 ({par_octave:.1f} LU par octave) ; "
              f"cible {cible} -> vélocité {velocite:.0f}")
    shutil.rmtree(WORK, ignore_errors=True)


if __name__ == "__main__":
    main()
