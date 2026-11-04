#!/usr/bin/env python3
"""B6 : batterie du profil `gmd_blues_shuffle` (GMD shuffle ternaire, 12 pas) et micro-timing par voix.

Écrit `profiles/gmd_blues_shuffle/percussion.toml`, `stats.json` (preuves, dans `b6/preuves/`) et le bloc
`[performance.tracks.Percussion*]` (`b6/preuves/performance.toml`, recopié dans `profile.toml`). Lancer avec le venv du
labo, GMD en cache vérifié (lecture seule) :

    cd ~/workspace/harmonium_lab && .venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b6/batterie.py

Sources, toutes du Groove MIDI Dataset v1.0.0 (CC BY 4.0), fichiers `beat` 4/4 de feel ternaire (triolets, 12 pas) :
`blues/shuffle` (2), `rock/shuffle` (2), `funk/purdieshuffle` (1) ; relances : fichiers `fill` ternaires de
`funk/purdieshuffle` (26) et de `rock` (3). Les fichiers `blues/shuffle` droit et swing léger sont écartés (autre grille).
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "b8"))
LAB = Path(os.environ.get("HARMONIUM_LAB", Path.home() / "workspace/harmonium_lab"))
os.environ.setdefault("H019_GMD_CACHE", "/tmp/h019-b6/results.pkl")

import gmd_batterie as B  # noqa: E402

PROFILE = HERE.parents[2] / "profiles" / "gmd_blues_shuffle"
STYLES = ["blues/shuffle", "rock/shuffle", "funk/purdieshuffle"]
REFS = {"blues/shuffle": "gmd-blues-shuffle", "rock/shuffle": "gmd-rock-shuffle", "funk/purdieshuffle": "gmd-funk-purdieshuffle"}
ROLE_OF_NOTE = {36: "kick", 38: "snare", 37: "sidestick", 42: "hat_closed", 44: "hat_pedal", 46: "hat_open", 51: "ride",
                53: "ride_bell", 49: "crash", 50: "tom_high", 47: "tom_mid", 43: "tom_low"}

SPEC = B.DrumSpec(
    STYLES, feel="ternary", steps=12, start_cuts=(2, 5, 8),
    min_files=1, min_measures=8, max_share=1.0, max_groups=6,
    drop_groups=[("hat_closed", "pas:0,2,4,5,7"), ("hat_closed", "pas:2,4,5,7")],
    fill_styles=["funk/purdieshuffle", "rock"], min_fill_files=3,
    min_phrase_count=2, min_phrase_files=1, min_change_count=2,
    arc_tags=["hat", "ride"],
)
#: Poids de « pas de relance » : 0,8 × le poids total des relances d'une densité (15 + 5 + 6 = 26), soit 45 % de fins de phrase
#: sans relance ; valeur de service (le GMD isole les relances : la part sans relance n'y est pas mesurable), comme `gmd_funk`.
FILLS_NONE_WEIGHT = 21
DEFAULT_BPM = 125.0


def voice_timing(bpm: float) -> dict[int, dict[str, float]]:
    """Médiane de l'écart à la grille par voix (ms, groupe `4-4:ternary`, `beat`), pondérée par l'effectif, en fraction de
    pas au tempo `bpm` : un pas de la grille ternaire est un tiers de temps, soit `20 000 / bpm` ms, donc
    `ms × bpm / 20 000`."""
    pooled: dict[str, dict[str, list[tuple[float, int]]]] = {}
    for style, ref in REFS.items():
        data = json.loads((LAB / "references" / ref / "measures-drums.json").read_text())
        group = data["styles"][style]["groups"]["4-4:ternary"]["beat"]
        for role, by_place in group["micro_timing"].items():
            for place, m in by_place.items():
                pooled.setdefault(role, {}).setdefault(place, []).append((m["ms"]["p50"], m["n"]))
    out: dict[int, dict[str, float]] = {}
    for note, role in ROLE_OF_NOTE.items():
        places = pooled.get(role, {})
        entry: dict[str, float] = {}
        for place in ("on_beat", "off_beat"):
            rows = places.get(place)
            if not rows or sum(n for _, n in rows) < 40:
                continue
            n = sum(r[1] for r in rows)
            ms = sum(r[0] * r[1] for r in rows) / n
            entry[place] = round(max(-0.2, min(0.2, ms * bpm / 20000.0)), 3)
        if "on_beat" not in entry and "off_beat" in entry:
            entry["on_beat"] = entry["off_beat"]
        if "on_beat" in entry:
            out[note] = entry
    return out


def performance_toml(timing: dict[int, dict[str, float]], notes: set[int]) -> str:
    out = []
    for track, candidates in (("PercussionLow", [36]), ("PercussionHigh", [37, 38, 42, 44, 46, 49, 51, 53, 43, 47, 50])):
        rows = []
        for note in candidates:
            t = timing.get(note)
            if t is None or note not in notes:
                continue
            fields = [f"note = {note}", f"on_beat = {t['on_beat']}"]
            if "off_beat" in t and t["off_beat"] != t["on_beat"]:
                fields.append(f"off_beat = {t['off_beat']}")
            rows.append("  { " + ", ".join(fields) + " },")
        if rows:
            out.append(f"[performance.tracks.{track}]\nvoices = [\n" + "\n".join(rows) + "\n]")
    return "\n".join(out) + "\n"


HEADER = """\
# Batterie du profil `gmd_blues_shuffle` : GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b6/batterie.py` depuis le
# Groove MIDI Dataset v1.0.0 (CC BY 4.0, Gillick, Roberts, Engel, Eck, Bamman, ICML 2019 ; Google LLC) : les fichiers `beat`
# 4/4 de feel ternaire (triolets, grille de 12 pas) de `blues/shuffle`, `rock/shuffle` et `funk/purdieshuffle` (5 fichiers,
# 4 batteurs, 340 mesures) ; relances : fichiers `fill` ternaires de `funk/purdieshuffle` (26, un batteur) et de `rock` (3).
# Chaque pattern est construit sur les probabilités PAR PAS d'un groupe de mesures (timekeeper et figure), pas sur une
# mesure d'un enregistrement. Groupes d'au moins 8 mesures (un seul fichier suffit : voir README, limites).
# Seuils : densité 0 timekeeper ≥ 0,5 ; densité 1 + pédale, charleston ouverte, grosse caisse, caisse claire ≥ 0,5 (en
# dessous, `probability` = fréquence mesurée dès 0,25) ; densité 2 + crash, grosse caisse et caisse claire dès 0,08.
# Pas = un tiers de temps (0, 3, 6, 9 = les temps ; 2, 5, 8, 11 = les contretemps du shuffle).
# Poids = effectif de mesures du groupe ; étiquette = timekeeper du groupe (`hat`, `ride`).
# Kit : kit standard de GeneralUser GS (banque 128, programme 0) ; vélocités de service (celles du GMD sont celles d'un
# Roland TD-11). Poids de « pas de relance » : 0,8 × le poids total des relances d'une densité, soit 45 % de fins de phrase sans
# relance (valeur de service : le GMD isole les relances, la part sans relance n'y est pas mesurable).
"""


def main() -> int:
    results = B.load_results(LAB)
    data = B.build(results, SPEC, LAB)
    text = B.percussion_toml(data, HEADER, FILLS_NONE_WEIGHT, humanize=3)
    PROFILE.mkdir(parents=True, exist_ok=True)
    (PROFILE / "percussion.toml").write_text(text)
    notes = {v["note"] for v in data["voices"].values()} | {h["note"] for p in data["patterns"] for h in p["hits"]}
    timing = voice_timing(DEFAULT_BPM)
    proofs = HERE / "preuves"
    proofs.mkdir(exist_ok=True)
    (proofs / "performance.toml").write_text(performance_toml(timing, notes))
    (proofs / "batterie.json").write_text(json.dumps(
        {**data["stats"], "voices": data["voices"], "n_patterns": len(data["patterns"]), "n_phrases": len(data["phrases"]),
         "n_fills": len(data["fills"]), "fill_tags": data["fill_tags"], "timing": {str(k): v for k, v in timing.items()}},
        indent=1, ensure_ascii=False))
    print(json.dumps(data["stats"], indent=1, ensure_ascii=False))
    print(len(data["patterns"]), "patterns", len(data["phrases"]), "phrases", len(data["fills"]), "relances")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
