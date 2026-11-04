#!/usr/bin/env python3
"""B3 samba : distance de la batterie rendue de `gmd_latin_brazilian_samba` au Groove MIDI Dataset.

Même méthode que `b8/distance_gmd.py` et `b47_distance_gmd.py` (Bray-Curtis des fréquences d'attaque par pas, métriques de
`harmonium_lab/styledist/symbolic.py` : `drums_any`, `drums_kick`, `drums_snare`, `drums_hat`) ; référence = TOUTES les mesures
`beat` 4/4 droites de `latin/brazilian-samba` (322 mesures, un batteur), pas seulement les groupes que le profil retient.

**Voix latines exclues** : le GMD n'a aucune percussion latine (kit Roland TD-11), le rendu est donc réduit aux notes du kit
GM que le GMD mesure (grosse caisse 36, cross-stick 37, caisse claire 38, charleston 42/44/46, toms 43/47/50) ; tamborim
(76), agogô (67/68), chocalho (82) et pandeiro (54) ne comptent ni dans `any` ni ailleurs. Le tom grave 43 est le surdo
ouvert (GMD `tom_low`, pas 4 et 12).

Copie non calibrée : le même profil (grille, basse, comping) dont `percussion.toml` est celui de `wjazzd_swing` (batterie
jazz calibrée pour un autre style), sans arc ni session. « Sans arc » : le profil livré sans `arc.toml` ni `session.toml` (les
étiquettes de batterie ne sont plus nommées : la mesure tire parmi tous les patterns du niveau de densité). Lancé avec le venv du
labo ; écrit seulement dans `--out` et `/tmp/harmonium-019/b3/samba/distance/`.

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_distance.py \
      --profile profiles/gmd_latin_brazilian_samba [--seeds 16 --bars 48] --out preuves.json
"""
from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "b8"))
import gmd_batterie as B  # noqa: E402

REPO = HERE.parents[2]
CLI = REPO / "target/release/harmonium-cli"
ENGINE = Path.home() / "Project/bascanada/harmonium"
WORK = Path("/tmp/harmonium-019/b3/samba/distance")
ROLES = {"kick": {36}, "snare": {37, 38}, "hat": {42, 44, 46}, "surdo": {43}}
GMD_ROLES = {"kick": {"kick"}, "snare": {"snare", "sidestick"}, "hat": {"hat_closed", "hat_pedal", "hat_open"},
             "surdo": {"tom_low"}}
#: Notes du kit que le GMD mesure : tout le reste (voix latines) est exclu du rendu.
KIT_GMD = {36, 37, 38, 42, 43, 44, 46, 47, 48, 50, 51, 53}
METRICS = ("any", "kick", "snare", "hat", "surdo")
MEAN = ("any", "kick", "snare", "hat")


def bc(a: list[float], b: list[float]) -> float:
    den = sum(x + y for x, y in zip(a, b))
    return sum(abs(x - y) for x, y in zip(a, b)) / den if den else 0.0


def reference(results) -> dict[str, list[float]]:
    measures = [m for r in results if r.row.style == "latin/brazilian-samba" and r.row.beat_type == "beat"
                and not r.reject and r.row.time_signature == "4-4" and r.feel == "straight" for m in r.measures]
    active = [m for m in measures if m.steps]
    out = {name: [sum(1 for m in active if any(s in m.steps.get(r, {}) for r in roles)) / len(active)
                  for s in range(16)] for name, roles in GMD_ROLES.items()}
    out["any"] = [sum(1 for m in active if any(s in slots for slots in m.steps.values())) / len(active)
                  for s in range(16)]
    return out


def rendered(profile: Path, seeds: int, bars: int, tag: str, tour: int | None = None) -> dict[str, list[float]]:
    """Fréquence d'attaque par pas des mesures où la batterie joue (voix latines exclues), `seeds` graines ;
    `tour` : seulement les mesures de ce chorus (le sommet est le chorus 4 d'un air de 80 mesures)."""
    measures: list[list[tuple[int, int]]] = []
    WORK.mkdir(parents=True, exist_ok=True)
    for seed in range(1, seeds + 1):
        truth = WORK / f"{tag}_{seed}_{bars}.json"
        if not truth.is_file():
            subprocess.run([str(CLI), "--export", "--seed", str(seed), "--bars", str(bars), "--profile", str(profile),
                            "--record-truth", str(truth)], cwd=ENGINE, capture_output=True, text=True, check=True)
        last = {}
        for rec in json.loads(truth.read_text())["measures"]:
            last[rec["measure"]["index"]] = rec["measure"]
        for index in sorted(last):
            if index > bars - 1:  # la mesure de clôture de l'export
                continue
            if tour is not None and last[index]["position"].get("tour") != tour:
                continue
            notes = [(n["start_step"], n["pitch"]) for n in last[index]["notes"]
                     if n["track"] in (3, 4) and n["pitch"] in KIT_GMD]
            if notes:
                measures.append(notes)
    out = {name: [sum(1 for m in measures if any(s == step and p in pitches for s, p in m)) / len(measures)
                  for step in range(16)] for name, pitches in ROLES.items()}
    out["any"] = [sum(1 for m in measures if any(s == step for s, _ in m)) / len(measures) for step in range(16)]
    out["mesures"] = len(measures)
    return out


def variant(profile: Path, name: str, percussion: Path | None = None) -> Path:
    """Copie sans arc ni session (et, si demandé, avec la batterie d'un autre profil)."""
    dst = WORK / f"{name}_{profile.name}"
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(profile, dst)
    if percussion:
        shutil.copy(percussion, dst / "percussion.toml")
    for f in ("arc.toml", "session.toml"):
        (dst / f).unlink()
    text = re.sub(r'(?m)^(arc|session) = "[a-z]+\.toml"\n', "", (dst / "profile.toml").read_text())
    (dst / "profile.toml").write_text(text)
    return dst


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--profile", type=Path, default=REPO / "profiles/gmd_latin_brazilian_samba")
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    args.profile = args.profile.resolve()  # le CLI se lance depuis le dépôt du moteur
    ref = reference(B.load_results(args.lab))
    versions = {
        "profil": args.profile,
        "sans_arc_ni_session": variant(args.profile, "sans_arc"),
        "non_calibre": variant(args.profile, "non_calibre", REPO / "profiles/wjazzd_swing/percussion.toml"),
    }
    table: dict[str, dict] = {}
    for key, path in versions.items():
        mine = rendered(path, args.seeds, args.bars, key)
        table[key] = {m: round(bc(mine[m], ref[m]), 3) for m in METRICS}
        table[key]["moyenne"] = round(sum(table[key][m] for m in MEAN) / len(MEAN), 3)
        table[key]["mesures_rendues"] = mine["mesures"]
        print(key, table[key])
    # Le sommet seul (chorus 4 : batucada pleine, densité 2) : toutes les voix du kit mesurées jouent.
    peak = rendered(args.profile, args.seeds, 80, "sommet", tour=4)
    table["sommet_chorus_4"] = {m: round(bc(peak[m], ref[m]), 3) for m in METRICS}
    table["sommet_chorus_4"]["moyenne"] = round(sum(table["sommet_chorus_4"][m] for m in MEAN) / len(MEAN), 3)
    table["sommet_chorus_4"]["mesures_rendues"] = peak["mesures"]
    print("sommet_chorus_4", table["sommet_chorus_4"])
    table["reference"] = {"mesures_gmd": 322, "par_pas": {k: [round(x, 2) for x in v] for k, v in ref.items()}}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(table, indent=2, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
