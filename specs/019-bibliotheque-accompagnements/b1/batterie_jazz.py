#!/usr/bin/env python3
"""B1 : `profiles/wjazzd_swing/percussion.toml` (patterns, phrases et relances du Groove MIDI Dataset, jazz).

Lancé avec le venv du labo (qui lit le GMD en cache vérifié, `corpus/gmd/raw/`) ; n'écrit rien dans le labo :

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b1/batterie_jazz.py \\
        [--out profiles/wjazzd_swing/percussion.toml] [--json /tmp/harmonium-019/b1/batterie.json]

Ce qui est MESURÉ (agrégats seulement, jamais une mesure d'enregistrement, principe VII) :
- les patterns ordinaires : ceux d'E7 (`drums.proposed_patterns`, un groupe par figure de ride, densités 0 à 2),
  étiquetés par figure : `spang` (spang-a-lang, partiel, orné) et `quarters` (ride sur les noires, plus) ;
- les phrases : suites de figures de ride sur 2 et 4 mesures alignées sur la première mesure d'un fichier
  (≥ 3 occurrences dans ≥ 2 fichiers ; ≥ 2 si la suite change de figure), mesure demandée par A1 ;
- les relances : fichiers `fill` du GMD (une mesure, un batteur), groupés par pas de départ (`start_step`),
  probabilités par pas et par voix, seuil 0,25 comme E7.

Ce qui ne l'est PAS, dit comme tel dans l'en-tête du fichier écrit : les patterns `brush` (balais : règle de
métier, le GMD ne contient aucun jeu aux balais), les phrases `quarters` (ride aux noires, réponse en spang) ajoutées aux figures sans suite
mesurée, la part de fins de phrase sans relance (`fills_none_weight`).
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

LAB = Path.home() / "Project/bascanada/harmonium_lab"
sys.path.insert(0, str(LAB / "src"))

from harmonium_lab.backing import drums, engine_values as ev  # noqa: E402

#: Figure de ride → étiquette d'arc.
TAG_OF_CLASS = {
    "spang_a_lang": "spang",
    "spang_a_lang_ornamented": "spang",
    "spang_partial": "spang",
    "quarters_plus": "quarters",
    "quarters": "quarters",
}
#: Une relance par étiquette que les arcs nomment (les relances sont filtrées par étiquette).
FILL_TAGS = ("spang", "quarters", "brush")
#: Poids de « pas de relance » : règle de métier (le GMD isole les fills, il ne dit pas leur fréquence).
FILLS_NONE_WEIGHT = 30
#: Seuils de phrases (A1, `a1-phrases.md`, « Phrases du GMD »).
MIN_COUNT, MIN_CHANGE_COUNT, MIN_FILES = 3, 2, 2
MAX_PHRASES = {4: 6, 2: 4}
#: Gain de rendu des balais (GeneralUser GS, kit 40) : réglage de service.
BRUSH_VOICES = {
    "swirl": (40, 64, "PercussionHigh"),  # « Jazz Brushed Snare » de la banque 128 (présent aux kits 32 et 40)
    "tap": (38, 52, "PercussionHigh"),  # caisse claire : « Brushed Snares » du kit 40
}

# Balais : règle de métier. Balayage (`swirl`) aux temps, tape (`tap`) sur le « a » des temps 2 et 4 (le
# spang-a-lang des balais) ; densité 1 ajoute la pédale aux temps 2 et 4, densité 2 une grosse caisse
# en plume et une tape fantôme. Deux figures : balayages à chaque temps, ou aux temps 1 et 3 seulement.
BRUSH = {
    "a": {  # balayages à chaque temps
        0: [(0, "swirl", 0), (4, "swirl", 6), (6, "tap", -4), (8, "swirl", 0), (12, "swirl", 6), (14, "tap", -4)],
        1: [(0, "swirl", 0), (4, "swirl", 6), (4, "pedal", 0), (6, "tap", -4), (8, "swirl", 0), (12, "swirl", 6), (12, "pedal", 0), (14, "tap", -4)],
        2: [(0, "kick", -14), (0, "swirl", 0), (4, "swirl", 6), (4, "pedal", 0), (6, "tap", -4), (8, "swirl", 0), (10, "tap", -12), (12, "swirl", 6), (12, "pedal", 0), (14, "tap", -4)],
    },
    "b": {  # balayages longs aux temps 1 et 3, tapes aux temps 2 et 4
        0: [(0, "swirl", 4), (4, "tap", 0), (6, "tap", -6), (8, "swirl", 4), (12, "tap", 0), (14, "tap", -6)],
        1: [(0, "swirl", 4), (4, "tap", 0), (4, "pedal", 0), (6, "tap", -6), (8, "swirl", 4), (12, "tap", 0), (12, "pedal", 0), (14, "tap", -6)],
        2: [(0, "kick", -14), (0, "swirl", 4), (4, "tap", 0), (4, "pedal", 0), (6, "tap", -6), (8, "swirl", 4), (12, "tap", 0), (12, "pedal", 0), (14, "tap", -6), (15, "tap", -16)],
    },
}
BRUSH_WEIGHT = {"a": 3, "b": 2}
BRUSH_PHRASES = [  # (poids, figures par mesure) : suites de 4 et 2 mesures, règle de métier
    (3, "aaaa"), (2, "aaab"), (2, "abab"), (1, "bbbb"), (2, "ab"), (2, "aa"),
]


def analyse() -> tuple[list[Any], list[Any], dict[str, Any]]:
    archive = drums.open_archive()
    rows = drums.select_rows(drums.read_info_rows(archive))
    results = drums.analyse_archive(archive, rows)
    core = [r for r in results if r.row.style in drums.CORE_STYLES]
    beats = [r for r in core if r.row.beat_type == "beat"]
    fills = [r for r in core if r.row.beat_type == "fill"]
    return beats, fills, {"rows": len(rows)}


def patterns_of(beats: list[Any], fills: list[Any], fill_groups: list[dict[str, Any]]) -> dict[str, Any]:
    beat_measures = [m for r in beats for m in r.measures]
    fill_measures = [m for r in fills for m in r.measures]
    proposed = drums.proposed_patterns(beat_measures, fill_measures)
    ordinary = [p for p in proposed["patterns"] if not ev._is_fill(p)]
    voices = ev.voices_from(ordinary + [{"weight": f["weight"], "hits": f["hits"]} for f in fill_groups])
    return {"proposed": proposed, "ordinary": ordinary, "voices": voices}


def class_index(ordinary: list[dict[str, Any]]) -> dict[tuple[str, int], int]:
    """(figure de ride, densité) → indice du pattern fusionné qui la porte."""
    index: dict[tuple[str, int], int] = {}
    for i, p in enumerate(ordinary):
        for source in p["sources"]:
            index[(source.split(":", 1)[1], p["density"])] = i
    return index


def phrases_of(beats: list[Any], index: dict[tuple[str, int], int]) -> list[dict[str, Any]]:
    classes = {cls for cls, _ in index}
    windows: dict[tuple[int, tuple[str, ...]], dict[str, Any]] = {}
    for r in beats:
        labels = [drums.ride_class(m) for m in r.measures]
        for size in (2, 4):
            for start in range(0, len(labels) - size + 1, size):
                window = tuple(labels[start:start + size])
                if all(label in classes for label in window):
                    e = windows.setdefault((size, window), {"n": 0, "files": set(), "drummers": set()})
                    e["n"] += 1
                    e["files"].add(r.row.path)
                    e["drummers"].add(r.row.drummer)
    out: list[dict[str, Any]] = []
    kept: Counter = Counter()
    for (size, window), e in sorted(windows.items(), key=lambda kv: (-kv[1]["n"], kv[0])):
        needed = MIN_CHANGE_COUNT if len(set(window)) > 1 else MIN_COUNT
        if e["n"] < needed or len(e["files"]) < MIN_FILES or kept[size] >= MAX_PHRASES[size]:
            continue
        kept[size] += 1
        tags = Counter(TAG_OF_CLASS[c] for c in window)
        tag = max(tags, key=lambda t: (tags[t], t == TAG_OF_CLASS[window[0]]))
        for density in (0, 1, 2):
            idx = [index.get((c, density)) for c in window]
            if None in idx:
                continue
            out.append({"density": density, "weight": e["n"], "tag": tag, "bars": size, "patterns": idx,
                        "n_files": len(e["files"]), "n_drummers": len(e["drummers"]), "window": list(window)})
    return out


def held_phrases(ordinary: list[dict[str, Any]], index: dict[tuple[str, int], int], phrases: list[dict[str, Any]],
                 ) -> list[dict[str, Any]]:
    """Un groupe sans suite mesurée joue tout de même : un batteur tient son groove quatre mesures (règle de
    métier), sinon le groupe ne serait jamais tiré dès qu'une phrase existe à sa densité."""
    covered = {(p["density"], i) for p in phrases for i in p["patterns"]}
    out = []
    for (cls, density), i in sorted(index.items()):
        if (density, i) in covered:
            continue
        covered.add((density, i))
        out.append({"density": density, "weight": ordinary[i]["weight"], "tag": TAG_OF_CLASS[cls], "bars": 4,
                    "patterns": [i] * 4, "held": True, "window": [cls] * 4})
    return out


#: Phrases `quarters` (règle de métier) : (poids, figures) ; `q` ride aux noires (GMD, `quarters_plus`), `s` réponse
#: en spang-a-lang (GMD, `spang_a_lang`). Les suites mesurées mélangent les figures sous l'étiquette `spang` ; sans
#: phrase propre, l'étiquette `quarters` (exposition, retombée) jouerait un pattern par chorus.
QUARTERS_PHRASES = [(3, "qqqq"), (2, "qqqs"), (2, "qq"), (1, "qsqq")]


def quarters_phrases(index: dict[tuple[str, int], int]) -> list[dict[str, Any]]:
    out = []
    for weight, figures in QUARTERS_PHRASES:
        for density in (0, 1, 2):
            idx = [index[("quarters_plus" if f == "q" else "spang_a_lang", density)] for f in figures]
            out.append({"density": density, "weight": weight, "tag": "quarters", "bars": len(figures), "patterns": idx,
                        "rule": True, "window": list(figures)})
    return out


def fills_of(fills: list[Any]) -> list[dict[str, Any]]:
    """Relances d'une mesure groupées par pas de départ (0, 4, 8 ou 12), probabilités par pas et par voix."""
    groups: dict[int, list[Any]] = defaultdict(list)
    for r in fills:
        if not r.measures:
            continue
        m = r.measures[0]
        first = min((s for slots in m.steps.values() for s in slots), default=0)
        groups[0 if first < 3 else 4 if first < 6 else 8 if first < 10 else 12].append(r)
    medians = drums.role_velocity_medians([m for r in fills for m in r.measures])
    out = []
    for start, rs in sorted(groups.items()):
        if len(rs) < drums.MIN_FILES_FOR_PROPOSAL:
            continue
        ms = [r.measures[0] for r in rs]
        hits = drums._subset_hits(ms, {role: drums.FILL_THRESHOLD for role in drums.FILL_ROLES}, medians)
        hits = [h for h in hits if h["step"] >= start]
        if hits:
            out.append({"weight": len(rs), "start_step": start, "hits": hits, "n_files": len(rs),
                        "n_drummers": len({r.row.drummer for r in rs})})
    # Relances en levée : tous les fichiers `fill` du GMD couvrent la mesure depuis le pas 0 ; un batteur relance
    # aussi sur le dernier ou les deux derniers temps. Mêmes probabilités par pas, tronquées au pas de départ
    # (règle de métier, dite dans l'en-tête), à la moitié du poids de la relance entière.
    whole = next((f for f in out if f["start_step"] == 0), None)
    if whole:
        for start in (8, 12):
            hits = [h for h in whole["hits"] if h["step"] >= start]
            if hits:
                out.append({"weight": max(1, whole["weight"] // 2), "start_step": start, "hits": hits,
                            "n_files": whole["n_files"], "n_drummers": whole["n_drummers"], "derived": True})
    return out


def toml(data: dict[str, Any]) -> str:
    ordinary, voices = data["ordinary"], data["voices"]
    ev_voices = {role: v for role, v in voices.items()}

    def pat_lines(table: str, p: dict[str, Any], tag: str, weight: int) -> list[str]:
        lines = [f"[[{table}]]", f"density = {p['density']}", f'tag = "{tag}"', f"weight = {weight}", "hits = ["]
        for h in sorted(p["hits"], key=lambda h: (h["step"], ev.VOICES[h["role"]][1] != "PercussionLow", h["role"])):
            lines.append(ev._hit_line(h, ev_voices, ev._pattern_scale(p)))
        return lines + ["]", ""]

    out = [data["header"].rstrip("\n"), "", "low_note = 36", "high_note = 51", "low_velocity = 64", "high_velocity = 64",
           "percussion_gain = 1.0", "velocity_humanize = 3", f"fills_none_weight = {FILLS_NONE_WEIGHT}", ""]
    for role in sorted(ev_voices, key=lambda r: (ev.VOICES[r][1] != "PercussionLow", ev.VOICES[r][0])):
        name, track = ev.VOICES[role]
        v = ev_voices[role]
        out += ["[[voices]]", f'name = "{name}"', f"note = {v['note']}", f"velocity = {v['velocity']}", f'track = "{track}"', ""]
    for name, (note, vel, track) in BRUSH_VOICES.items():
        out += ["[[voices]]", f'name = "{name}"', f"note = {note}", f"velocity = {vel}", f'track = "{track}"', ""]

    # Patterns : indices 0..n-1 = ordinaires du GMD, puis balais.
    for i, p in enumerate(ordinary):
        tags = sorted({TAG_OF_CLASS[s.split(":", 1)[1]] for s in p["sources"]})
        tag = tags[0]
        out.append(f"# {i} : " + ", ".join(p["sources"]) + f" (GMD, {p['n_files']} fichiers)")
        out += pat_lines("patterns", p, tag, p["weight"])
    brush_index: dict[tuple[str, int], int] = {}
    for fig in ("a", "b"):
        for density in (0, 1, 2):
            brush_index[(fig, density)] = len(ordinary) + len(brush_index)
            out.append(f"# {brush_index[(fig, density)]} : balais, figure {fig}, densité {density} (règle de métier)")
            out += ["[[patterns]]", f"density = {density}", 'tag = "brush"', f"weight = {BRUSH_WEIGHT[fig]}", "hits = ["]
            for step, voice, delta in sorted(BRUSH[fig][density], key=lambda h: (h[0], h[1] != "kick")):
                out.append(f"  {{ step = {step}, voice = \"{voice}\"" + (f", delta_velocity = {delta}" if delta else "") + " },")
            out += ["]", ""]

    for ph in data["phrases"]:
        out += ["[[phrases]]", f"density = {ph['density']}", f'tag = "{ph["tag"]}"', f"weight = {ph['weight']}",
                f"patterns = {ph['patterns']}", ""]
    for weight, figures in BRUSH_PHRASES:
        for density in (0, 1, 2):
            out += ["[[phrases]]", f"density = {density}", 'tag = "brush"', f"weight = {weight}",
                    f"patterns = {[brush_index[(f, density)] for f in figures]}", ""]

    for f in data["fills"]:
        scale = 1.0
        for density in (1, 2):
            for tag in FILL_TAGS:
                out += ["[[fills]]", f"density = {density}", f'tag = "{tag}"', f"weight = {f['weight']}"]
                if f["start_step"]:
                    out.append(f"start_step = {f['start_step']}")
                out.append("bars = [{ hits = [")
                for h in sorted(f["hits"], key=lambda h: (h["step"], ev.VOICES[h["role"]][1] != "PercussionLow", h["role"])):
                    out.append(ev._hit_line(h, ev_voices, scale).replace("  {", "  {", 1))
                out += ["] }]", ""]
    return "\n".join(out).rstrip("\n") + "\n"


def header(data: dict[str, Any]) -> str:
    st = data["stats"]
    lines = [
        "# Batterie du profil `wjazzd_swing` (phase 019, B1) : GÉNÉRÉ par",
        "# `specs/019-bibliotheque-accompagnements/b1/batterie_jazz.py` (relancer le script le réécrit).",
        "#",
        "# Source : Groove MIDI Dataset v1.0.0 (Google LLC), CC BY 4.0, Gillick, Roberts, Engel, Eck, Bamman, ICML 2019 ;",
        f"# styles `jazz` et `jazz/swing`, 4/4 : {st['n_beat_files']} fichiers `beat` ({st['n_drummers']} batteurs, {st['n_beat_measures']} mesures entières),",
        f"# {st['n_fill_files']} fichiers `fill` (une mesure, {st['n_fill_drummers']} batteur). Aucune mesure d'un enregistrement n'est copiée : chaque coup est un",
        "# (rôle, pas) dont la probabilité par pas dans un groupe de mesures atteint le seuil de sa densité (0,5 ; 0,25 pour la",
        "# grosse caisse, la caisse claire et les relances).",
        "#",
        "# MESURÉ (GMD) :",
        "# - patterns : un par figure de ride (spang-a-lang `0,4,6,8,12,14`, partiel, orné, ride aux noires plus), densités 0 à 2 ;",
        "#   étiquette `spang` ou `quarters` (la figure) ; poids = effectif de mesures du groupe ;",
        f"# - phrases : suites de figures sur 2 et 4 mesures alignées sur le début du fichier ({st['n_phrases_measured']} mesurées : ≥ {MIN_COUNT} occurrences",
        f"#   dans ≥ {MIN_FILES} fichiers, ≥ {MIN_CHANGE_COUNT} si la suite change de figure) ;",
        "# - relances : une mesure entière (tous les fichiers `fill` du GMD couvrent la mesure depuis le pas 0), copiées par",
        "#   étiquette d'arc (les relances sont filtrées par étiquette) ;",
        "#",
        "# RÈGLES DE MÉTIER (aucune mesure, le GMD ne contient aucun jeu aux balais ni la fréquence des relances) :",
        "# - relances en levée (`start_step` 8 et 12) : la relance mesurée tronquée au pas de départ, à la moitié de son poids ;",
        "# - patterns et phrases `brush` (balais : balayage `swirl` = touche 40 « Jazz Brushed Snare », tape = touche 38 du",
        "#   kit 40) ;",
        f"# - {st['n_phrases_held']} phrases `quarters` : ride aux noires avec réponse en spang-a-lang (les suites mesurées mélangent les figures sous `spang`) ;",
        f"# - `fills_none_weight = {FILLS_NONE_WEIGHT}` : part de fins de phrase sans relance (≈ {FILLS_NONE_WEIGHT * 100 // (FILLS_NONE_WEIGHT + st['fill_weight'])} %), non mesurée.",
        "#",
        "# Voix : ride (51), pédale (44), caisse claire (38), tom (43) sur PercussionHigh, grosse caisse (36) sur PercussionLow,",
        "# balayage (40) et tape (38). Densité 0 : timekeeper ; 1 : + pédale aux temps 2 et 4 ; 2 : + caisse claire fantôme et grosse",
        "# caisse. Niveaux de voix : `engine_values.ROLE_GAIN` (kit jazz de GeneralUser GS, banque 128).",
    ]
    return "\n".join(line for line in lines if not line.startswith("# - 0 phrases"))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", type=Path, default=Path("profiles/wjazzd_swing/percussion.toml"))
    ap.add_argument("--json", type=Path)
    args = ap.parse_args()
    beats, fills, _ = analyse()
    fill_groups = fills_of(fills)
    pat = patterns_of(beats, fills, fill_groups)
    index = class_index(pat["ordinary"])
    measured = phrases_of(beats, index)
    held = held_phrases(pat["ordinary"], index, measured) + quarters_phrases(index)
    data = {
        **pat, "phrases": measured + held, "fills": fill_groups,
        "stats": {
            "n_beat_files": len(beats), "n_beat_measures": sum(len(r.measures) for r in beats),
            "n_drummers": len({r.row.drummer for r in beats}),
            "n_fill_files": len(fills), "n_fill_drummers": len({r.row.drummer for r in fills}),
            "n_phrases_measured": len({(p["bars"], tuple(p["window"])) for p in measured}), "n_phrases_held": len({(p["bars"], tuple(p["window"])) for p in held}),
            "fill_weight": sum(f["weight"] for f in fill_groups),
        },
    }
    data["header"] = header(data)
    args.out.write_text(toml(data), encoding="utf-8")
    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps({k: data[k] for k in ("stats", "phrases", "fills")}, ensure_ascii=False, indent=1,
                                        default=list), encoding="utf-8")
    print(json.dumps(data["stats"], ensure_ascii=False))
    print("groupes", {f"{c}:{d}": i for (c, d), i in sorted(index.items())})
    print("phrases mesurées", [(p["bars"], p["window"], p["weight"], p["n_files"]) for p in measured if p["density"] == 0])
    print("relances", [(f["start_step"], f["weight"], len(f["hits"])) for f in fill_groups])
    return 0


if __name__ == "__main__":
    sys.exit(main())
