#!/usr/bin/env python3
"""B3 : distance de la batterie rendue de `gmd_latin_brazilian_baiao` au Groove MIDI Dataset, contre une copie non calibrée.

Reprend `b8/distance_gmd.py` (modèle non calibré par B4 + B7 et B8) : fréquence d'attaque par pas (16 pas) de la
grosse caisse, de la caisse claire (et cross-stick), du charleston et du ride, plus « un coup quelconque », distance
de Bray-Curtis à la référence : TOUTES les 200 mesures `beat` 4/4 droites du style `latin/brazilian-baiao`
(pas seulement les deux groupes retenus). 16 graines x 48 mesures, densité 1 par défaut.

**Les voix latines sont exclues** : triangle (80, 81) et agogô (67, 68) n'existent pas dans le GMD (kit Roland TD-11),
elles ne comptent dans aucun rôle ni dans « un coup quelconque ». Tout le reste (zabumba = grosse caisse mesurée,
charleston, ride, caisse claire) est comparé.

Colonnes : `profil` (le profil livré, avec son habillage et son arc), `sans_arc` (même profil sans arc ni habillage),
`non_calibre` (grille, basse et comping identiques, `percussion.toml` de `wjazzd_swing`, sans arc ni habillage) ;
la plus petite distance est la plus proche du GMD. Lancé avec le venv du labo, qui n'est ni modifié ni écrit.

    ~/Project/bascanada/harmonium_lab/.venv/bin/python \\
        specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_distance.py \\
        [--profile profiles/gmd_latin_brazilian_baiao] [--out f.json]

Rendus et copies dans `/tmp/harmonium-019/b3/baiao/dist/` (à supprimer).
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

ICI = Path(__file__).resolve().parent
sys.path.insert(0, str(ICI.parent / "b8"))
import distance_gmd as D  # noqa: E402
import gmd_batterie as B  # noqa: E402

REPO = ICI.parents[2]
WORK = Path("/tmp/harmonium-019/b3/baiao/dist")
SOURCE = "latin/brazilian-baiao"
LATINES = {80, 81, 67, 68}
ROLES = {"kick": {36, 35}, "snare": {37, 38, 39, 40}, "hat": {42, 44, 46}, "ride": {51, 53, 59}}


def rendu(profile: Path, seeds: int, bars: int, tag: str) -> dict[str, list[float]]:
    """Fréquence d'attaque par pas des mesures où le kit (hors voix latines) joue."""
    measures: list[list[tuple[int, int]]] = []
    for seed in range(1, seeds + 1):
        truth = WORK / f"{tag}_{seed}.json"
        if not truth.is_file():
            subprocess.run([str(D.CLI), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profile),
                            "--record-truth", str(truth)], cwd=D.ENGINE_CWD, capture_output=True, text=True, check=True)
        last: dict[int, dict[str, Any]] = {}
        for rec in json.loads(truth.read_text())["measures"]:
            last[rec["measure"]["index"]] = rec["measure"]
        for index in sorted(last):
            if index > bars - 1:  # la mesure de clôture de l'export joue la tonique et la grosse caisse
                continue
            notes = [(n["start_step"], n["pitch"]) for n in last[index]["notes"]
                     if n["track"] in (3, 4) and n["pitch"] not in LATINES]
            if notes:
                measures.append(notes)
    out = {name: [sum(1 for m in measures if any(s == step and p in pitches for s, p in m)) / len(measures)
                  for step in range(16)] for name, pitches in ROLES.items()}
    out["any"] = [sum(1 for m in measures if any(s == step for s, _ in m)) / len(measures) for step in range(16)]
    return out


def copie(profile: Path, nom: str, percussion: Path | None = None) -> Path:
    """Copie sans arc ni habillage (la batterie seule est comparée), `percussion.toml` éventuellement remplacé."""
    dst = WORK / f"{nom}_{profile.name}"
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(profile, dst)
    if percussion:
        shutil.copy(percussion, dst / "percussion.toml")
    for f in ("arc.toml", "session.toml"):
        (dst / f).unlink()
    texte = (dst / "profile.toml").read_text()
    (dst / "profile.toml").write_text(re.sub(r'(?m)^(arc|session) = "(arc|session).toml"\n', "", texte))
    return dst


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--profile", type=Path, default=REPO / "profiles/gmd_latin_brazilian_baiao")
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    args.profile = args.profile.resolve()
    WORK.mkdir(parents=True, exist_ok=True)
    D.CLI = REPO / "target/release/harmonium-cli"
    ref = D.reference(B.load_results(args.lab), [SOURCE])
    mine = rendu(args.profile, args.seeds, args.bars, "profil")
    plain = rendu(copie(args.profile, "sans_arc"), args.seeds, args.bars, "sans_arc")
    base = rendu(copie(args.profile, "non_calibre", REPO / "profiles/wjazzd_swing/percussion.toml"), args.seeds, args.bars, "non_calibre")
    row: dict[str, dict[str, float]] = {}
    for name in ("any", "kick", "snare", "hat", "ride"):
        row[name] = {"profil": round(D.bc(mine[name], ref[name]), 3), "sans_arc": round(D.bc(plain[name], ref[name]), 3),
                     "non_calibre": round(D.bc(base[name], ref[name]), 3)}
    row["moyenne"] = {k: round(sum(row[m][k] for m in ("any", "kick", "snare", "hat")) / 4, 3)
                      for k in ("profil", "sans_arc", "non_calibre")}
    print("rôle        profil  sans_arc  non_calibre   (distance de Bray-Curtis au GMD : plus petit = plus proche)")
    for name, v in row.items():
        print(f"{name:10} {v['profil']:7.3f} {v['sans_arc']:9.3f} {v['non_calibre']:11.3f}")
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps({"profil": args.profile.name, "source": SOURCE, "seeds": args.seeds, "bars": args.bars,
                                        "distances": row, "reference_par_pas": ref, "profil_par_pas": mine}, indent=2) + "\n")
    shutil.rmtree(WORK, ignore_errors=True)


if __name__ == "__main__":
    main()
