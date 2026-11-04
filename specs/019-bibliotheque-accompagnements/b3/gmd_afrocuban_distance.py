#!/usr/bin/env python3
"""B3 afro-cubain : distance de la batterie du kit de `gmd_afrocuban` au Groove MIDI Dataset, contre une copie non calibrée.

Reprend la méthode de `b8/distance_gmd.py` (Bray-Curtis des fréquences d'attaque par pas, métriques de batterie de
`harmonium_lab/styledist/symbolic.py` : tout, grosse caisse, caisse claire, charleston), avec trois différences, dites :

- **voix latines exclues** : le GMD n'a aucune percussion latine. Seules les notes du kit (grosse caisse, caisse claire,
  cross-stick, charleston, ride, toms) comptent dans le profil ; claves, congas, bongos, timbales, cloche (cowbell 56
  et cloche de ride 53, que le profil joue comme cloche de métier, pas comme rôle mesuré), shaker, maracas, güiro sont
  écartés du rendu ET de la référence (`ride_bell`) ;
- référence : toutes les mesures `beat` 4/4 droites de `afrocuban` (1 fichier, 98 mesures) ET celles de
  `afrocuban/rhumba` (1 fichier, 103 mesures), les deux groupes qui alimentent le kit ; l'une et l'autre sont aussi
  données seules ;
- copie non calibrée : le même profil (grille, basse, comping, voix latines) dont `percussion.toml` est celui de
  `wjazzd_swing` (batterie jazz calibrée pour un autre style), sans arc ni session ; plus la copie « sans arc » du profil.

Lancé avec le venv du labo (le cache `results.pkl` de B8 est lu s'il existe, sinon recalculé par le labo) ; n'écrit
rien dans le labo ni dans le dépôt (JSON dans `--out`, rendus et copies dans `/tmp/harmonium-019/b3/afrocubain/`).

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_distance.py \
        [--profile profiles/gmd_afrocuban] [--out f.json]
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
import gmd_batterie as B  # noqa: E402

DEPOT = ICI.parents[2]
CLI = DEPOT / "target/release/harmonium-cli"
MOTEUR = Path.home() / "Project/bascanada/harmonium"
TRAVAIL = Path("/tmp/harmonium-019/b3/afrocubain/distance")
#: Notes du kit retenues par rôle (rendu) ; 53 (cloche de ride) et 56 (cowbell) sont des voix de métier.
ROLES = {"kick": {35, 36}, "snare": {37, 38, 39, 40}, "hat": {42, 44, 46}, "ride": {51, 59}}
KIT = set().union(*ROLES.values()) | {41, 43, 45, 47, 48, 49, 50, 52, 55, 57}
GMD_ROLES = {"kick": {"kick"}, "snare": {"snare", "sidestick"}, "hat": {"hat_closed", "hat_pedal", "hat_open"}, "ride": {"ride"}}
KIT_ROLES = {role for roles in GMD_ROLES.values() for role in roles} | {"tom_high", "tom_mid", "tom_low", "crash"}


def bc(a: list[float], b: list[float]) -> float:
    num = sum(abs(x - y) for x, y in zip(a, b))
    den = sum(x + y for x, y in zip(a, b))
    return num / den if den else 0.0


def reference(results: list[Any], styles: list[str]) -> dict[str, list[float]]:
    measures = [m for r in results if r.row.style in styles and r.row.beat_type == "beat" and not r.reject
                and r.row.time_signature == "4-4" and r.feel == "straight" for m in r.measures]
    active = [m for m in measures if any(role in KIT_ROLES for role in m.steps)]
    out: dict[str, list[float]] = {}
    for name, roles in GMD_ROLES.items():
        out[name] = [sum(1 for m in active if any(s in m.steps.get(r, {}) for r in roles)) / len(active) for s in range(16)]
    out["any"] = [sum(1 for m in active if any(s in slots for role, slots in m.steps.items() if role in KIT_ROLES)) / len(active)
                  for s in range(16)]
    out["_n"] = [float(len(active))]
    return out


def rendered(profile: Path, seeds: int, bars: int, tag: str) -> dict[str, list[float]]:
    """Fréquence d'attaque par pas des mesures où le kit joue, sur `seeds` graines (notes du kit seulement)."""
    measures: list[list[tuple[int, int]]] = []
    for seed in range(1, seeds + 1):
        truth = TRAVAIL / f"{tag}_{seed}.json"
        if not truth.is_file():
            subprocess.run([str(CLI), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profile),
                            "--record-truth", str(truth)], cwd=MOTEUR, capture_output=True, text=True, check=True)
        last: dict[int, dict[str, Any]] = {}
        for rec in json.loads(truth.read_text())["measures"]:
            last[rec["measure"]["index"]] = rec["measure"]
        for index in sorted(last):
            if index > bars - 1:  # la mesure de clôture de l'export joue la tonique et la grosse caisse
                continue
            notes = [(n["start_step"], n["pitch"]) for n in last[index]["notes"] if n["track"] in (3, 4) and n["pitch"] in KIT]
            if notes:
                measures.append(notes)
    out = {name: [sum(1 for m in measures if any(s == step and p in pitches for s, p in m)) / len(measures) for step in range(16)]
           for name, pitches in ROLES.items()}
    out["any"] = [sum(1 for m in measures if any(s == step for s, _ in m)) / len(measures) for step in range(16)]
    out["_n"] = [float(len(measures))]
    return out


def copie(profile: Path, nom: str, percussion: Path | None = None) -> Path:
    dst = TRAVAIL / f"{nom}_{profile.name}"
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(profile, dst)
    if percussion:
        shutil.copy(percussion, dst / "percussion.toml")
    (dst / "arc.toml").unlink()
    (dst / "session.toml").unlink()
    texte = (dst / "profile.toml").read_text()
    texte = re.sub(r'(?m)^(arc|session) = "[a-z]+\.toml"\n', "", texte)
    (dst / "profile.toml").write_text(texte)
    return dst


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--profile", type=Path, default=DEPOT / "profiles/gmd_afrocuban")
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--out", type=Path, default=TRAVAIL / "distance-gmd.json")
    args = ap.parse_args()
    TRAVAIL.mkdir(parents=True, exist_ok=True)
    results = B.load_results(args.lab)
    refs = {"afrocuban + rhumba": reference(results, ["afrocuban", "afrocuban/rhumba"]),
            "afrocuban": reference(results, ["afrocuban"]), "rhumba": reference(results, ["afrocuban/rhumba"])}
    mine = rendered(args.profile, args.seeds, args.bars, "profil")
    plain = rendered(copie(args.profile, "sans_arc"), args.seeds, args.bars, "sans_arc")
    base = rendered(copie(args.profile, "non_calibre", DEPOT / "profiles/wjazzd_swing/percussion.toml"), args.seeds, args.bars, "non_calibre")
    table: dict[str, Any] = {"mesures_rendues": {"profil": int(mine["_n"][0]), "sans_arc": int(plain["_n"][0]), "non_calibre": int(base["_n"][0])}}
    for nom, ref in refs.items():
        row: dict[str, Any] = {"mesures_reference": int(ref["_n"][0])}
        for metrique in ("any", "kick", "snare", "hat", "ride"):
            row[metrique] = {"profil": round(bc(mine[metrique], ref[metrique]), 3), "sans_arc": round(bc(plain[metrique], ref[metrique]), 3),
                             "non_calibre": round(bc(base[metrique], ref[metrique]), 3)}
        for cle in ("profil", "sans_arc", "non_calibre"):
            # Moyenne sur les quatre métriques de L2 (le ride n'y figure pas).
            row.setdefault("moyenne", {})[cle] = round(sum(row[m][cle] for m in ("any", "kick", "snare", "hat")) / 4, 3)
        table[nom] = row
        print(f"{nom:20} (n={row['mesures_reference']}) " + "  ".join(f"{m} {row[m]['profil']:.3f}/{row[m]['sans_arc']:.3f}/{row[m]['non_calibre']:.3f}" for m in row if m != 'mesures_reference'))
    args.out.write_text(json.dumps(table, indent=2, sort_keys=True, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
