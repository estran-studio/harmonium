"""R2-styles : habillage par graine (`[session]`, A2) des neuf profils GMD, micro-timing par voix (A6).

Données des neuf profils `gmd_*` (rock, pop, hip-hop, soul, reggae, disco, afrobeat : B8 ; funk et New Orleans :
B4 + B7) et écriture de `session.toml`, des tables de tonalité de `dynamics.toml` et des voix de batterie de
`[performance.tracks.*]`. Aucun mécanisme du moteur n'est ajouté : tout passe par des champs existants.

D'où vient chaque valeur :
- plages de tempo : centiles des BPM des fichiers `beat` du GMD (`references/gmd-*/measures-drums.json`, L2) ;
- swing léger : rapport médian des fichiers `swing` du GMD (hip-hop, 10 fichiers `fill` ; rock, 7 fichiers `beat`) ;
- micro-timing par voix : médianes en millisecondes du GMD (groupe 4/4 droit, tous styles du profil, pondérées par
  l'effectif), converties en fraction de pas au tempo par défaut du profil ;
- tonalités, instruments, kits, familles de voicings, registres, énergie et arcs : règles de métier de chaque style
  (aucun corpus distribuable : décision F3), dites comme telles dans chaque README.

Programmes General MIDI 0-based de GeneralUser GS (banque 0, vérifiés dans le `phdr` de la banque) et kits de la
banque 128 (0 standard, 8 room, 16 power, 24 electronic, 25 808/909).
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

LAB = Path.home() / "Project/bascanada/harmonium_lab"

# --- GeneralUser GS (banque 0 : programmes ; banque 128 : kits) ---------------------------------------------------
PROGRAMS = {
    0: "Grand Piano", 1: "Bright Grand Piano", 2: "Electric Grand Piano", 3: "Honky-Tonk Piano", 4: "Tine Electric Piano",
    5: "FM Electric Piano", 7: "Clavinet", 11: "Vibraphone", 16: "Tonewheel Organ", 17: "Percussive Organ",
    18: "Rock Organ", 21: "Accordion", 24: "Nylon Guitar", 25: "Steel Guitar", 26: "Jazz Guitar", 27: "Clean Guitar",
    28: "Muted Guitar", 29: "Overdrive Guitar", 30: "Distortion Guitar", 32: "Acoustic Bass", 33: "Finger Bass", 34: "Pick Bass",
    35: "Fretless Bass", 36: "Slap Bass 1", 38: "Synth Bass 1", 39: "Synth Bass 2", 48: "Fast Strings",
    49: "Slow Strings", 58: "Tuba", 61: "Brass Section", 89: "Warm Pad",
}
KITS = {0: "Standard 1", 8: "Room", 16: "Power", 24: "Electronic", 25: "808/909", 32: "Jazz", 40: "Brush"}


def check_programs(fields: dict[str, Any], where: str) -> None:
    for key in ("melody_program", "countermelody_program", "chords_program", "bass_program"):
        if key in fields and fields[key] not in PROGRAMS:
            raise SystemExit(f"{where} : programme {fields[key]} absent de la table vérifiée")
    if "percussion_program" in fields and fields["percussion_program"] not in KITS:
        raise SystemExit(f"{where} : kit {fields['percussion_program']} absent de la table vérifiée")


# --- description d'un habillage ------------------------------------------------------------------------------------
@dataclass
class Variant:
    group: str
    name: str
    weight: int = 1
    bpm: tuple[float, float] | float | None = None
    swing: float | None = None
    tonality: tuple[int, str] | None = None
    instruments: dict[str, int] = field(default_factory=dict)
    voicing: str | None = None
    register: dict[str, int] = field(default_factory=dict)
    comping: list[tuple[str, int]] = field(default_factory=list)
    percussion: list[tuple[str, int]] = field(default_factory=list)
    arc: str | None = None
    intensity: float | None = None
    density: float | None = None

    def toml(self) -> str:
        check_programs(self.instruments, f"{self.group}/{self.name}")
        out = ["[[variants]]", f'group = "{self.group}"', f'name = "{self.name}"']
        if self.weight != 1:
            out.append(f"weight = {self.weight}")
        if self.bpm is not None:
            out.append("bpm = " + (f"[{self.bpm[0]:.1f}, {self.bpm[1]:.1f}]" if isinstance(self.bpm, tuple) else f"{self.bpm:.1f}"))
        if self.swing is not None:
            out.append(f"swing = {self.swing}")
        if self.tonality is not None:
            out.append(f'tonality = {{ key = {self.tonality[0]}, mode = "{self.tonality[1]}" }}')
        if self.instruments:
            out.append("instruments = { " + ", ".join(f"{k} = {v}" for k, v in
                                                     ((k, self.instruments[k]) for k in sorted(self.instruments))) + " }")
        if self.voicing:
            out.append(f'voicing = "{self.voicing}"')
        if self.register:
            out.append("register_octaves = { " + ", ".join(f"{k} = {v}" for k, v in self.register.items()) + " }")
        if self.comping:
            out.append("comping = [" + ", ".join(f'{{ tag = "{t}", weight = {w} }}' for t, w in self.comping) + "]")
        if self.percussion:
            out.append("percussion = [" + ", ".join(f'{{ tag = "{t}", weight = {w} }}' for t, w in self.percussion) + "]")
        if self.arc:
            out.append(f'arc = "{self.arc}"')
        if self.intensity is not None:
            out.append(f"intensity = {self.intensity}")
        if self.density is not None:
            out.append(f"density = {self.density}")
        return "\n".join(out)


NOTE_NAMES = ["do", "do#", "ré", "mi♭", "mi", "fa", "fa#", "sol", "la♭", "la", "si♭", "si"]


def tonalities(keys: list[tuple[int, int]], mode: str, group: str = "tonalite") -> list[Variant]:
    names = ["do", "do_diese", "re", "mi_bemol", "mi", "fa", "fa_diese", "sol", "la_bemol", "la", "si_bemol", "si"]
    return [Variant(group, f"{names[k]}_{mode}", w, tonality=(k, mode)) for k, w in keys]


def tempos(bands: list[tuple[str, int, float, float]], swing: float | None = None) -> list[Variant]:
    return [Variant("tempo", n, w, bpm=(lo, hi), swing=swing) for n, w, lo, hi in bands]


def instruments(group: str, field_name: str, choices: list[tuple[str, int, int]]) -> list[Variant]:
    return [Variant(group, n, w, instruments={field_name: p}) for n, w, p in choices]


def kits(choices: list[tuple[str, int, int]]) -> list[Variant]:
    return [Variant("kit", n, w, instruments={"percussion_bank": 128, "percussion_program": p}) for n, w, p in choices]


def voicings(choices: list[tuple[str, int, str]]) -> list[Variant]:
    return [Variant("voicing", n, w, voicing=v) for n, w, v in choices]


def energy(soft: tuple[float, float] = (0.85, 0.85), strong: tuple[float, float] = (1.15, 1.1)) -> list[Variant]:
    return [Variant("energie", "doux", 1, intensity=soft[0], density=soft[1]),
            Variant("energie", "normal", 2),
            Variant("energie", "fort", 1, intensity=strong[0], density=strong[1])]


def registers(up: int = 1, down: int = 0, normal: int = 4, up_weight: int = 1, down_weight: int = 1) -> list[Variant]:
    out = [Variant("registre", "normal", normal)]
    if up:
        out.append(Variant("registre", "haut", up_weight, register={"Chords": 1}))
    if down:
        out.append(Variant("registre", "grave", down_weight, register={"Chords": -1}))
    return out


# --- arcs nommés (valeurs de service : aucun corpus ne donne l'arc d'un morceau) -----------------------------------
def arc_text(name: str, kind: str, perc_quiet: str, perc_loud: str | None, orchestre: dict[str, dict[str, int]] | None,
             break_start: int | None = None) -> str:
    """Arc nommé. `perc_quiet` / `perc_loud` : étiquettes de batterie (« hat », « sparse »…). `orchestre` : programmes
    par étape ('expo', 'corps', 'sommet', 'retombee') pour l'arc d'orchestration."""
    def inst(stage: str) -> str:
        if not orchestre or stage not in orchestre:
            return ""
        check_programs(orchestre[stage], f"arc {name}/{stage}")
        return ", instruments = { " + ", ".join(f"{k} = {v}" for k, v in sorted(orchestre[stage].items())) + " }"

    quiet = f'[{{ tag = "{perc_quiet}" }}]'
    loud = f'[{{ tag = "{perc_loud}" }}]' if perc_loud else quiet
    p = lambda x: f", percussion = {x}"  # noqa: E731
    break_mid = (f', breaks = [{{ start = {break_start}, bars = 1, chance = 0.4, tracks = ["Chords"] }}]' if break_start else "")
    out = [f"[arcs.{name}]", "repeat_from = 1", ""]
    if kind == "orchestre":
        # Quatre chorus qui ne jouent pas avec les mêmes instruments : le corps rend ceux de la session.
        out += ["[[arcs.%s.stages]]" % name, "choruses = 1", "variants = [",
                f'  {{ weight = 1, intensity = 0.65, density = 0.9, comping = [{{ tag = "sparse", weight = 3 }}, {{ tag = "medium", weight = 1 }}]{p(quiet)}{inst("expo")} }},',
                "]", "",
                "[[arcs.%s.stages]]" % name, "choruses = 2", "variants = [",
                f'  {{ weight = 2, comping = [{{ tag = "medium", weight = 2 }}, {{ tag = "dense", weight = 1 }}]{p(loud)}{break_mid} }},',
                f'  {{ weight = 1, intensity = 1.1, density = 1.1, comping = [{{ tag = "dense" }}]{p(loud)}{inst("corps")} }},',
                "]", "",
                "[[arcs.%s.stages]]" % name, "choruses = 1", "variants = [",
                f'  {{ intensity = 1.4, density = 1.4, comping = [{{ tag = "dense" }}]{p(loud)}{inst("sommet")}, breaks = [{{ start = -2, bars = 2, chance = 0.5, tracks = ["Chords", "Bass"], keep_steps = [0] }}] }},',
                "]", "",
                "[[arcs.%s.fall]]" % name, "intensity = 0.5", "density = 0.5", 'comping = [{ tag = "sparse" }]',
                f"percussion = {quiet}", inst('retombee').removeprefix(", "),
                'breaks = [{ start = -2, bars = 2, tracks = ["Chords", "Bass"], keep_steps = [0] }]']
    elif kind == "direct":
        # Pas d'exposition : le morceau part dans le corps, monte d'un cran, sans retombée creuse (fin tenue).
        out += ["[[arcs.%s.stages]]" % name, "choruses = 2", "variants = [",
                f'  {{ weight = 2, intensity = 0.9, comping = [{{ tag = "medium", weight = 2 }}, {{ tag = "dense", weight = 1 }}]{p(loud)}{break_mid} }},',
                f'  {{ weight = 1, intensity = 1.0, comping = [{{ tag = "medium" }}]{p(loud)} }},',
                "]", "",
                "[[arcs.%s.stages]]" % name, "choruses = 1", "variants = [",
                f'  {{ intensity = 1.25, density = 1.25, comping = [{{ tag = "dense" }}]{p(loud)}, breaks = [{{ start = -1, bars = 1, chance = 0.5, tracks = ["Chords", "Bass"] }}] }},',
                "]", "",
                "[[arcs.%s.fall]]" % name, "intensity = 0.8", "density = 0.7", 'comping = [{ tag = "medium" }]',
                f"percussion = {loud}",
                'breaks = [{ start = -1, bars = 1, tracks = ["Chords", "Bass"], keep_steps = [0] }]']
    elif kind == "montee":
        # Montée lente : un cran par chorus, sans break avant le dernier, retombée nette.
        out += ["[[arcs.%s.stages]]" % name, "choruses = 1", "variants = [",
                f'  {{ intensity = 0.55, density = 0.8, comping = [{{ tag = "sparse" }}]{p(quiet)} }},', "]", "",
                "[[arcs.%s.stages]]" % name, "choruses = 1", "variants = [",
                f'  {{ intensity = 0.8, density = 1.0, comping = [{{ tag = "medium" }}]{p(quiet)} }},', "]", "",
                "[[arcs.%s.stages]]" % name, "choruses = 1", "variants = [",
                f'  {{ intensity = 1.0, density = 1.1, comping = [{{ tag = "medium", weight = 2 }}, {{ tag = "dense", weight = 1 }}]{p(loud)}{inst("corps")} }},',
                "]", "",
                "[[arcs.%s.stages]]" % name, "choruses = 1", "variants = [",
                f'  {{ intensity = 1.3, density = 1.3, comping = [{{ tag = "dense" }}]{p(loud)}{inst("sommet")} }},', "]", "",
                "[[arcs.%s.fall]]" % name, "intensity = 0.45", "density = 0.5", 'comping = [{ tag = "sparse" }]',
                f"percussion = {quiet}", inst('retombee').removeprefix(", "),
                'breaks = [{ start = -2, bars = 2, tracks = ["Chords", "Bass"], keep_steps = [0] }]']
    else:
        raise SystemExit(kind)
    return "\n".join(out)


# --- micro-timing par voix (GMD, L2) -------------------------------------------------------------------------------
ROLE_OF_NOTE = {36: "kick", 38: "snare", 37: "sidestick", 42: "hat_closed", 44: "hat_pedal", 46: "hat_open", 51: "ride",
                53: "ride_bell", 49: "crash", 50: "tom_high", 47: "tom_mid", 43: "tom_low"}


def voice_timing(families: list[str], styles: list[str], bpm: float,
                 lab: Path = LAB) -> dict[int, dict[str, float]]:
    """Médianes de l'écart à la grille par voix (ms, groupe 4/4 droit, `beat`), pondérées par l'effectif, en fraction de
    pas au tempo `bpm` : `ms × bpm / 15000` (un pas = une double croche = 15000 / bpm ms). `jitter` : écart-type mesuré en
    pas, converti en amplitude d'un tirage uniforme (écart-type = amplitude / √3), plafonné à 0,1 (borne du schéma)."""
    pooled: dict[str, dict[str, list[tuple[float, float, int]]]] = {}
    for family in families:
        data = json.loads((lab / "references" / f"gmd-{family}" / "measures-drums.json").read_text())
        for style in styles:
            group = data["styles"].get(style, {}).get("groups", {}).get("4-4:straight", {}).get("beat")
            if not group:
                continue
            for role, by_place in group["micro_timing"].items():
                for place, m in by_place.items():
                    pooled.setdefault(role, {}).setdefault(place, []).append((m["ms"]["p50"], m["ms"]["std"], m["n"]))
    out: dict[int, dict[str, float]] = {}
    for note, role in ROLE_OF_NOTE.items():
        places = pooled.get(role)
        if not places:
            continue
        entry: dict[str, float] = {}
        for place in ("on_beat", "off_beat"):
            rows = places.get(place)
            if not rows or sum(n for *_, n in rows) < 40:
                continue
            n = sum(r[2] for r in rows)
            ms = sum(r[0] * r[2] for r in rows) / n
            std = sum(r[1] * r[2] for r in rows) / n
            entry[place] = round(max(-0.2, min(0.2, ms * bpm / 15000.0)), 3)
            entry["jitter"] = max(entry.get("jitter", 0.0), round(min(0.1, std * bpm / 15000.0 * 3 ** 0.5), 2))
        if "on_beat" not in entry and "off_beat" in entry:
            entry["on_beat"] = entry["off_beat"]
        if "on_beat" in entry:
            out[note] = entry
    return out


def voices_toml(timing: dict[int, dict[str, float]], notes_in_profile: set[int], with_jitter: bool) -> str:
    """Tables `[performance.tracks.PercussionLow|High]` : une entrée par note GM jouée par le profil."""
    out = []
    for track, notes in (("PercussionLow", [36]), ("PercussionHigh", [37, 38, 42, 44, 46, 49, 51, 53, 43, 47, 50])):
        rows = []
        for note in notes:
            t = timing.get(note)
            if t is None or note not in notes_in_profile:
                continue
            fields = [f"note = {note}", f"on_beat = {t['on_beat']}"]
            if "off_beat" in t and t["off_beat"] != t["on_beat"]:
                fields.append(f"off_beat = {t['off_beat']}")
            if with_jitter and t.get("jitter"):
                fields.append(f"jitter = {t['jitter']}")
            rows.append("  { " + ", ".join(fields) + " },")
        if rows:
            out.append(f"[performance.tracks.{track}]\nvoices = [\n" + "\n".join(rows) + "\n]")
    return "\n".join(out)


# --- données des neuf profils --------------------------------------------------------------------------------------
@dataclass
class Habillage:
    id: str
    mode: str  # mode du profil (celui des tonalités tirées)
    dark: str  # mode sombre (ou clair) parallèle, pour la bascule d'émotion
    keys: list[tuple[int, int]]  # (classe de hauteur, poids)
    groups: list[Variant]
    orchestration: dict[str, dict[str, dict[str, int]]]  # arc nommé → étape → programmes
    arc_weights: dict[str, int]  # défaut, orchestre, direct, montee
    families: list[str]  # dossiers `gmd-*` du labo
    styles: list[str]  # styles GMD lus pour le micro-timing
    jitter: bool = True
    break_start: int | None = None  # break à mi-chorus de l'arc « orchestre » (mesure 1-based)

    def arcs(self, quiet: str, loud: str | None) -> dict[str, str]:
        out = {}
        if self.arc_weights.get("orchestre"):
            out["orchestre"] = arc_text("orchestre", "orchestre", quiet, loud, self.orchestration["orchestre"], self.break_start)
        if self.arc_weights.get("direct"):
            out["direct"] = arc_text("direct", "direct", quiet, loud, None, self.break_start)
        if self.arc_weights.get("montee"):
            out["montee"] = arc_text("montee", "montee", quiet, loud, self.orchestration.get("montee"))
        return out

    def all_groups(self) -> list[Variant]:
        arc = [Variant("arc", "defaut", self.arc_weights.get("defaut", 3))]
        arc += [Variant("arc", name, w, arc=name) for name, w in self.arc_weights.items() if name != "defaut" and w]
        return self.groups + arc

    def session_toml(self, header: str, quiet: str, loud: str | None) -> str:
        out = [header.rstrip("\n"), ""]
        last = None
        for v in self.all_groups():
            if v.group != last:
                last = v.group
            out += [v.toml(), ""]
        for text in self.arcs(quiet, loud).values():
            out += [text, ""]
        return "\n".join(out).rstrip("\n") + "\n"

    def sizes(self) -> dict[str, int]:
        sizes: dict[str, int] = {}
        for v in self.all_groups():
            sizes[v.group] = sizes.get(v.group, 0) + 1
        return sizes

    def possible(self) -> int:
        total = 1
        for n in self.sizes().values():
            total *= n
        return total

    def key_modes(self) -> str:
        return "[" + ", ".join(f'{{ key = {k}, mode = "{self.mode}", weight = {w} }}' for k, w in self.keys) + "]"

    def color(self) -> str:
        return "[" + ", ".join(f'{{ key = {k}, mode = "{self.mode}", weight = [[0.0, 1.0], [1.0, 1.0]] }}' for k, _ in self.keys) + "]"

    def parallel_modes(self) -> str:
        bright, dark = (self.dark, self.mode) if self.mode in MINOR else (self.mode, self.dark)
        return f'[{{ bright = "{bright}", dark = "{dark}", weight = 1 }}]'


MINOR = {"aeolian", "dorian", "phrygian", "locrian"}
STD_ARCS = {"defaut": 3, "orchestre": 2, "direct": 1, "montee": 1}
KIT = {"percussion_bank": 128}


def orch(expo: dict, sommet: dict, retombee: dict | None = None, corps: dict | None = None) -> dict[str, dict]:
    """Programmes par étape des arcs « orchestre » et « montee »."""
    stages = {"expo": expo, "sommet": sommet, "retombee": retombee if retombee is not None else expo}
    if corps:
        stages["corps"] = corps
    return {"orchestre": stages, "montee": {k: v for k, v in stages.items() if k != "expo"}}


def kit_of(program: int) -> dict[str, int]:
    return {"percussion_bank": 128, "percussion_program": program}


HABILLAGES: dict[str, Habillage] = {}


def add(h: Habillage) -> None:
    HABILLAGES[h.id] = h


add(Habillage(
    id="gmd_rock", mode="mixolydian", dark="dorian", keys=[(4, 3), (9, 3), (2, 2), (7, 2)],
    groups=[
        *tempos([("lent", 2, 90.0, 99.0), ("medium", 3, 102.0, 116.0), ("vif", 2, 120.0, 130.0)]),
        Variant("feel", "droit", 5), Variant("feel", "swing_leger", 1, swing=0.60),
        *tonalities([(4, 3), (9, 3), (2, 2), (7, 2)], "mixolydian"),
        *instruments("comping", "chords_program", [("overdrive", 4, 29), ("saturee", 2, 30), ("claire", 2, 27), ("orgue_rock", 1, 18)]),
        *instruments("basse", "bass_program", [("mediator", 3, 34), ("doigts", 2, 33)]),
        *kits([("room", 3, 8), ("standard", 2, 0), ("power", 2, 16)]),
        *voicings([("triades_serrees", 3, "close_triad"), ("ouvert", 1, "open")]),
        *registers(up=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 27, **kit_of(0)}, {"chords_program": 30, **kit_of(16)}),
    arc_weights=STD_ARCS, families=["rock"], styles=["rock"]))

add(Habillage(
    id="gmd_pop", mode="ionian", dark="aeolian", keys=[(7, 3), (0, 3), (2, 2), (9, 1), (5, 2)],
    groups=[
        *tempos([("ballade", 1, 83.0, 90.0), ("rapide", 3, 132.0, 142.0)]),
        *tonalities([(7, 3), (0, 3), (2, 2), (9, 1), (5, 2)], "ionian"),
        *instruments("comping", "chords_program", [("piano", 4, 0), ("piano_electrique", 2, 4), ("guitare_claire", 2, 27), ("guitare_acier", 1, 25)]),
        *instruments("basse", "bass_program", [("doigts", 4, 33), ("mediator", 2, 34), ("fretless", 1, 35), ("synthetique", 1, 38)]),
        *kits([("standard", 4, 0), ("room", 2, 8), ("electronique", 1, 24), ("808", 1, 25)]),
        *voicings([("triades_serrees", 3, "close_triad"), ("ouvert", 2, "open"), ("drop_2", 1, "drop2")]),
        *registers(up=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 0}, {"chords_program": 27, **kit_of(8)}),
    arc_weights=STD_ARCS, families=["pop"], styles=["pop", "pop/soft", "pop/groove7"]))

add(Habillage(
    id="gmd_hiphop", mode="aeolian", dark="ionian", keys=[(0, 3), (9, 2), (7, 2), (5, 2), (2, 1), (4, 1)],
    groups=[
        *tempos([("lent", 2, 70.0, 79.0), ("medium", 3, 84.0, 95.0), ("vif", 1, 97.0, 100.0)]),
        Variant("feel", "droit", 4), Variant("feel", "swing_leger", 1, swing=0.59),
        *tonalities([(0, 3), (9, 2), (7, 2), (5, 2), (2, 1), (4, 1)], "aeolian"),
        *instruments("comping", "chords_program", [("piano_electrique", 4, 4), ("piano", 2, 0), ("fm", 1, 5), ("vibraphone", 1, 11), ("nylon", 1, 24)]),
        *instruments("basse", "bass_program", [("synth_1", 3, 38), ("synth_2", 2, 39), ("doigts", 1, 33)]),
        *kits([("808", 3, 25), ("electronique", 2, 24), ("standard", 1, 0), ("room", 1, 8)]),
        *voicings([("drop_2", 3, "drop2"), ("shell", 2, "shell"), ("structure_superieure", 1, "upper_structure"), ("ouvert", 1, "open")]),
        *registers(up=1, down=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 11}, {"chords_program": 5, "bass_program": 39}),
    arc_weights=STD_ARCS, families=["hiphop"], styles=["hiphop", "hiphop/groove6"]))

add(Habillage(
    id="gmd_soul", mode="ionian", dark="aeolian", keys=[(3, 3), (10, 2), (8, 2), (5, 1), (0, 2)],
    groups=[
        *tempos([("lent", 1, 80.0, 88.0), ("medium", 3, 96.0, 105.0)]),
        *tonalities([(3, 3), (10, 2), (8, 2), (5, 1), (0, 2)], "ionian"),
        *instruments("comping", "chords_program", [("piano_electrique", 4, 4), ("orgue", 2, 16), ("piano", 2, 0), ("clavinet", 1, 7), ("guitare_claire", 1, 27)]),
        *instruments("basse", "bass_program", [("doigts", 4, 33), ("fretless", 2, 35), ("mediator", 1, 34)]),
        *kits([("standard", 4, 0), ("room", 2, 8), ("balais", 1, 40)]),
        *voicings([("drop_2", 3, "drop2"), ("shell", 1, "shell"), ("notes_guides", 1, "guide_tones"), ("ouvert", 1, "open")]),
        *registers(up=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 4}, {"chords_program": 16}),
    arc_weights=STD_ARCS, families=["soul"], styles=["soul", "soul/groove3", "soul/groove4", "soul/groove9", "soul/groove10", "soul/motown"]))

add(Habillage(
    id="gmd_reggae", mode="ionian", dark="aeolian", keys=[(9, 3), (7, 2), (2, 2), (0, 1), (4, 1)],
    groups=[
        *tempos([("lent", 1, 83.0, 94.0), ("medium", 1, 97.0, 108.0), ("vif", 1, 110.0, 121.0)]),
        *tonalities([(9, 3), (7, 2), (2, 2), (0, 1), (4, 1)], "ionian"),
        *instruments("comping", "chords_program", [("guitare_claire", 3, 27), ("guitare_sourdine", 2, 28), ("orgue", 2, 16), ("piano", 1, 0)]),
        *instruments("basse", "bass_program", [("doigts", 4, 33), ("fretless", 2, 35), ("mediator", 1, 34)]),
        *kits([("standard", 4, 0), ("room", 2, 8), ("electronique", 1, 24)]),
        *voicings([("triades_serrees", 4, "close_triad"), ("ouvert", 1, "open")]),
        *registers(up=1),
        *energy(soft=(0.9, 0.9), strong=(1.1, 1.1)),
    ],
    orchestration=orch({"chords_program": 28}, {"chords_program": 16}),
    arc_weights=STD_ARCS, families=["reggae"], styles=["reggae"]))

add(Habillage(
    id="gmd_dance_disco", mode="aeolian", dark="ionian", keys=[(9, 3), (2, 2), (4, 2), (7, 1), (11, 1)],
    groups=[
        *tempos([("direct", 3, 120.0, 126.0), ("rapide", 2, 130.0, 137.0)]),
        *tonalities([(9, 3), (2, 2), (4, 2), (7, 1), (11, 1)], "aeolian"),
        *instruments("comping", "chords_program", [("guitare_claire", 3, 27), ("piano_electrique", 2, 4), ("clavinet", 1, 7), ("piano", 1, 0)]),
        *instruments("basse", "bass_program", [("doigts", 3, 33), ("slap", 2, 36), ("mediator", 1, 34), ("synthetique", 1, 38)]),
        *kits([("standard", 4, 0), ("electronique", 2, 24), ("room", 1, 8), ("808", 1, 25)]),
        *voicings([("triades_serrees", 3, "close_triad"), ("drop_2", 2, "drop2"), ("ouvert", 1, "open")]),
        *registers(up=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 27}, {"chords_program": 61}),
    arc_weights=STD_ARCS, families=["dance-disco"], styles=["dance/disco"]))

add(Habillage(
    id="gmd_afrobeat", mode="dorian", dark="mixolydian", keys=[(2, 3), (9, 2), (4, 2), (7, 1)],
    groups=[
        *tempos([("lent", 2, 86.0, 93.0), ("medium", 3, 95.0, 103.0), ("vif", 1, 106.0, 113.0)]),
        *tonalities([(2, 3), (9, 2), (4, 2), (7, 1)], "dorian"),
        *instruments("comping", "chords_program", [("guitare_claire", 3, 27), ("piano_electrique", 2, 4), ("orgue", 1, 16), ("guitare_sourdine", 1, 28)]),
        *instruments("basse", "bass_program", [("doigts", 3, 33), ("fretless", 2, 35), ("mediator", 1, 34)]),
        *kits([("standard", 4, 0), ("room", 2, 8), ("power", 1, 16)]),
        *voicings([("notes_guides", 3, "guide_tones"), ("drop_2", 2, "drop2"), ("shell", 1, "shell")]),
        *registers(up=1, down=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 27}, {"chords_program": 61}),
    arc_weights=STD_ARCS, families=["afrobeat"], styles=["afrobeat"]))

add(Habillage(
    id="gmd_funk", mode="dorian", dark="mixolydian", keys=[(4, 3), (9, 2), (2, 2), (7, 1)],
    groups=[
        *tempos([("lent", 2, 80.0, 90.0), ("medium", 3, 92.0, 102.0), ("vif", 2, 104.0, 114.0), ("rapide", 1, 118.0, 130.0)]),
        *tonalities([(4, 3), (9, 2), (2, 2), (7, 1)], "dorian"),
        *instruments("comping", "chords_program", [("guitare_claire", 3, 27), ("guitare_sourdine", 2, 28), ("clavinet", 2, 7), ("piano_electrique", 1, 4)]),
        *instruments("basse", "bass_program", [("doigts", 3, 33), ("slap", 2, 36), ("synthetique", 1, 38), ("mediator", 1, 34)]),
        *kits([("standard", 3, 0), ("room", 2, 8), ("power", 1, 16), ("electronique", 1, 24)]),
        *voicings([("notes_guides", 3, "guide_tones"), ("shell", 2, "shell"), ("drop_2", 1, "drop2")]),
        *registers(up=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 27}, {"chords_program": 7, "bass_program": 36}),
    arc_weights=STD_ARCS, families=["funk"], styles=["funk"],
    jitter=False, break_start=9))

add(Habillage(
    id="gmd_neworleans_funk_secondline", mode="mixolydian", dark="dorian", keys=[(10, 3), (5, 2), (3, 2), (0, 1)],
    groups=[
        *tempos([("lent", 2, 82.0, 88.0), ("medium", 3, 89.0, 96.0), ("second_line", 2, 98.0, 112.0), ("vif", 1, 114.0, 124.0)]),
        *tonalities([(10, 3), (5, 2), (3, 2), (0, 1)], "mixolydian"),
        *instruments("comping", "chords_program", [("piano", 3, 0), ("honky_tonk", 2, 3), ("accordeon", 1, 21), ("orgue", 1, 16)]),
        *instruments("basse", "bass_program", [("tuba", 4, 58), ("contrebasse", 2, 32), ("doigts", 1, 33)]),
        *kits([("standard", 3, 0), ("room", 1, 8), ("jazz", 2, 32)]),
        *voicings([("drop_2", 3, "drop2"), ("ouvert", 1, "open"), ("shell", 1, "shell")]),
        *registers(up=1, down=1),
        *energy(),
    ],
    orchestration=orch({"chords_program": 0}, {"chords_program": 61}),
    arc_weights=STD_ARCS, families=["neworleans-funk", "neworleans-secondline"],
    styles=["neworleans/funk", "neworleans/secondline"], jitter=False, break_start=7))


# --- couleur et dérive de registre des voicings (A5) : poids de métier --------------------------------------------
# Tension 9, 11 ou 13 que la qualité permet (jamais publiée, jamais en conflit avec la grille : D4). Agit sur les familles
# `drop2`, `open`, `guide_tones`, `rootless*`, `upper_structure`, pas sur `close_triad`.
LIGHT = "{ none_weight = 4, published_weight = 1, ninth_weight = 1 }"  # accords de rock, de pop, de reggae, de disco
JAZZY = "{ none_weight = 2, published_weight = 1, ninth_weight = 3, eleventh_weight = 1, thirteenth_weight = 2 }"
COLOR = {"gmd_rock": LIGHT, "gmd_pop": LIGHT, "gmd_reggae": LIGHT, "gmd_dance_disco": LIGHT,
         "gmd_hiphop": JAZZY, "gmd_soul": JAZZY, "gmd_afrobeat": JAZZY, "gmd_funk": JAZZY,
         "gmd_neworleans_funk_secondline": "{ none_weight = 3, published_weight = 1, ninth_weight = 2, thirteenth_weight = 1 }"}
DRIFT = {"gmd_rock": "{ step = 2, max = 4 }", "gmd_pop": "{ step = 2, max = 4 }", "gmd_reggae": "{ step = 2, max = 4 }",
         "gmd_dance_disco": "{ step = 2, max = 4 }", "gmd_hiphop": "{ step = 2, max = 6 }", "gmd_soul": "{ step = 2, max = 6 }",
         "gmd_afrobeat": "{ step = 2, max = 6 }", "gmd_funk": "{ step = 2, max = 6 }",
         "gmd_neworleans_funk_secondline": "{ step = 2, max = 6 }"}




GROUP_ROLE = {
    "tempo": ("tempo (plage de BPM, un entier tiré)", "GMD : centiles des BPM des fichiers `beat`"),
    "feel": ("rapport de swing", "GMD : rapport médian des fichiers `swing` du style (patterns mesurés droits : voir limites)"),
    "tonalite": ("tonique et mode du premier air (grille relative transposée)", "métier : tonalités courantes du style"),
    "comping": ("programme des accords", "métier"),
    "basse": ("programme de la basse", "métier"),
    "kit": ("kit de la banque 128", "métier ; le GMD est joué sur un kit électronique Roland TD-11"),
    "voicing": ("famille de voicings du comping", "métier"),
    "registre": ("octave du comping", "métier"),
    "energie": ("multiplicateurs d'intensité et de densité de base", "service"),
    "arc": ("arc : `[arc]` du profil ou arc nommé de `session.toml`", "service (aucun corpus ne donne l'arc d'un morceau)"),
}


def readme_section(h: Habillage, default: float, timing: dict, prob: int, key_name: str) -> str:
    rows = []
    for group in h.sizes():
        names = ", ".join(f"{v.name} ({v.weight})" for v in h.all_groups() if v.group == group)
        what, source = GROUP_ROLE[group]
        rows.append(f"| `{group}` | {names} | {what} | {source} |")
    voice_rows = []
    for note, t in sorted(timing.items()):
        role = ROLE_OF_NOTE[note]
        voice_rows.append(f"| {role} | {note} | {t['on_beat']:+.3f} | {t.get('off_beat', t['on_beat']):+.3f} | "
                          f"{t.get('jitter', 0):.2f} |" if h.jitter else f"| {role} | {note} | {t['on_beat']:+.3f} | {t.get('off_beat', t['on_beat']):+.3f} | — |")
    return f"""## Habillage par graine (`session.toml`, A2 ; R2-styles)

Une graine tire **un habillage** : un tirage indépendant par groupe, tous les groupes se combinent ({len(h.sizes())} groupes,
{len(h.all_groups())} variantes, **{h.possible()} habillages possibles**). Le tirage est stable par graine, rejoué après un seek, et **l'hôte gagne** : `SetBpm` impose le tempo, `SetSwing`
le swing, `[fixed_tonality]` du profil de l'hôte (la grille de training, transposée par lui) la tonalité (la variante est
alors ignorée) ; programmes, voicings, registre, énergie et arc n'ont pas de commande (un hôte qui n'en veut pas installe
un profil sans `[session]`). Ce profil n'a donc **plus de `[fixed_tonality]`** : la tonalité par défaut ({key_name}) n'est
que la plus pondérée.

| Groupe | Variantes (poids) | Ce que règle le groupe | D'où vient la valeur |
|---|---|---|---|
{chr(10).join(rows)}

Les arcs nommés (`orchestre` : le comping et le kit changent de chorus en chorus et reviennent aux programmes de la session ;
`direct` : pas d'exposition ; `montee` : un cran par chorus) sont écrits dans `session.toml`, mêmes valeurs de service que
`arc.toml`. Tous les instruments sont des programmes General MIDI de GeneralUser GS (banque 0) et des kits de la banque 128
(0, 8, 16, 24, 25), vérifiés dans la banque ; aucune zone Odin (D1).

**Micro-timing par voix** (`performance.tracks.Percussion*.voices`, A6) : médianes de l'écart à la grille mesurées dans le
GMD (groupe 4/4 droit de {', '.join('`' + x + '`' for x in h.styles)}, pondérées par l'effectif), en **fraction de pas au tempo par
défaut ({default:g} BPM)** : un pas = 15 000 / BPM ms ; les décalages mesurés en ms ne suivent donc pas exactement le tempo tiré
(à 130 BPM le même retard en ms vaut 24 % de pas de plus). `jitter` : amplitude d'un tirage uniforme dont l'écart-type est celui
mesuré, plafonnée à 0,1 pas (borne du schéma ; l'écart-type mesuré est de 0,11 à 0,20 pas).{" Aucun `jitter` : la basse à notes de passage partage son tirage avec la percussion (contrat `passing_keeps_neighbour_tracks…`)." if not h.jitter else ""}

| Voix | Note GM | Sur le temps (pas) | Hors temps (pas) | Jitter (pas) |
|---|---:|---:|---:|---:|
{chr(10).join(voice_rows)}

**Coups probabilistes** (`probability`, A6) : {prob} coups de `percussion.toml` jouent avec la probabilité par pas mesurée dans le
groupe (grosse caisse, caisse claire, cross-stick entre 0,08 et le seuil de la densité ; notes fantômes à la densité 2),
au lieu d'un coup certain dès 0,25 (0,3 pour funk et New Orleans) ou absent : le tirage est sans état (graine, mesure, pas, note) donc stable par graine.

**Voicings** : `chords.color` (tension 9, 11 ou 13 que la qualité permet, jamais en conflit avec la grille, D4) et
`chords.register_drift` (dérive bornée du centre) sont déclarés ; ils n'agissent que sur les familles de voicings qui les
lisent (pas `close_triad`).
"""
