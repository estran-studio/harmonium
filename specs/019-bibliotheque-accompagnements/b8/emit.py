"""B8 : écriture des fichiers d'un profil d'accompagnement sous grille (TOML lisible, commentaires gardés)."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


def scalar(v: Any) -> str:
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, str):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    if isinstance(v, float):
        return repr(round(v, 4))
    if isinstance(v, (list, tuple)):
        return "[" + ", ".join(scalar(x) for x in v) + "]"
    if isinstance(v, dict):
        return it(**v)
    return str(v)


def it(**kw: Any) -> str:
    """Table inline ; l'ordre des clés est celui des arguments."""
    return "{ " + ", ".join(f"{k} = {scalar(v)}" for k, v in kw.items()) + " }"


def bars_wrap(items: list[str], indent: str = "  ") -> str:
    return "[\n" + "".join(f"{indent}{x},\n" for x in items) + "]"


# ---------------------------------------------------------------------------------------------
# Description d'un style


@dataclass
class Chord:
    semitones: int
    quality: str
    beats: int  # durée en temps (4 pas chacun)
    bass: int | None = None

    def toml(self) -> str:
        kw: dict[str, Any] = {"semitones": self.semitones, "quality": self.quality}
        if self.bass is not None:
            kw["bass"] = self.bass
        kw["steps"] = self.beats * 4
        return it(**kw)


@dataclass
class Loop:
    weight: int
    chords: list[Chord]
    comment: str = ""

    @property
    def beats(self) -> int:
        return sum(c.beats for c in self.chords)


@dataclass
class Part:
    name: str
    bars: int
    comment: str
    loops: list[Loop]


@dataclass
class Comp:
    """Un rythme de comping d'une mesure : (pas, durée, vélocité, options)."""

    weight: int
    tag: str
    hits: list[tuple]
    comment: str = ""


@dataclass
class CompPhrase:
    weight: int
    tag: str
    motifs: list[int]
    at: str = "any"


@dataclass
class BassNote:
    step: int
    duration: int
    semitones: int
    velocity: int


@dataclass
class BassMotif:
    bars: int
    weight: int
    notes: list[BassNote]
    comment: str = ""


@dataclass
class Style:
    id: str
    name: str
    tradition: str
    tags: list[str]
    aliases: list[str]
    title: str  # titre du README
    # tonalité, tempo
    key: int
    mode: str
    bpm: tuple[float, float, float]  # min, défaut, max
    bpm_note: str
    # forme
    parts: list[Part]
    sequence: list[int]
    tours: int
    # accompagnement
    bass_low: int
    bass_high: int
    bass_motifs: list[BassMotif]
    bass_templates: list[str] = field(default_factory=list)  # tables TOML toutes faites
    bass_passing: str = ""  # bloc TOML `[accompaniment.bass.passing]`
    chords_center: int = 60
    chords_drift: int = 6
    chords_low: int = 52
    chords_high: int = 76
    voicing: str = "close_triad"
    note_count: int = 3
    spread: str = ""  # bloc TOML `[accompaniment.chords.spread]`
    default_comp: list[tuple] = field(default_factory=list)
    comping: list[Comp] = field(default_factory=list)
    comp_phrases: list[CompPhrase] = field(default_factory=list)
    # instruments (programmes GM 0-based de GeneralUser GS)
    chords_program: int = 0
    bass_program: int = 33
    kit_program: int = 0
    # timing, mixage
    chords_offset: float = 0.0
    chords_jitter: float = 0.1
    bass_jitter: float = 0.1
    mix: dict[str, float] = field(default_factory=lambda: {
        "master": 20.0, "Chords": -12.0, "Bass": -14.0, "Percussion": -8.0})
    # arc
    arc: str = ""  # contenu d'arc.toml
    # documentation
    comments: dict[str, str] = field(default_factory=dict)
    readme: dict[str, str] = field(default_factory=dict)
    corpus: str = ""
    velocity_curves: dict[str, list[list[float]]] = field(default_factory=dict)
    # R2-styles : habillage par graine (`session.toml`), micro-timing par voix, couleur et dérive des voicings
    hab: Any = None  # `r2-styles/habillage.py::Habillage`
    voices_toml: str = ""  # tables `[performance.tracks.Percussion*]`
    color: str = ""  # `chords.color` (ligne TOML)
    register_drift: str = ""  # `chords.register_drift` (ligne TOML)


# ---------------------------------------------------------------------------------------------


def comp_hit(h: tuple) -> str:
    step, dur, vel, *extra = h
    kw: dict[str, Any] = {"step": step, "duration": dur, "velocity": vel}
    for e in extra:
        if e == "anticipate":
            kw["anticipate"] = True
        elif e in ("down", "up", "block"):
            kw["direction"] = e
        elif isinstance(e, tuple) and e[0] == "probability":
            kw["probability"] = e[1]
    return it(**kw)


def comping_line(c: Comp, index: int) -> str:
    hits = ", ".join(comp_hit(h) for h in c.hits)
    head = f"  # {index} {c.tag}" + (f" : {c.comment}" if c.comment else "")
    return f"{head}\n  {{ weight = {c.weight}, tag = \"{c.tag}\", hits = [{hits}] }},"


def profile_toml(s: Style, header: str) -> str:
    lo, default, hi = s.bpm
    out: list[str] = [header.rstrip("\n"), ""]
    out += ["[files]", 'cells = "cells.toml"', 'percussion = "percussion.toml"', 'arrangement = "arrangement.toml"',
            'dynamics = "dynamics.toml"', 'set = "set.toml"', 'arc = "arc.toml"', 'session = "session.toml"', ""]
    out += ["[meta]", f'id = "{s.id}"', f"name = {scalar(s.name)}", f"tradition = {scalar(s.tradition)}",
            f"tags = {scalar(s.tags)}"]
    if s.aliases:
        out.append(f"aliases = {scalar(s.aliases)}")
    out += ['corpus = """\\', s.corpus.rstrip("\n") + '"""', ""]
    out += ["[meter]", 'time_signature = { numerator = 4, denominator = 4 }', "steps_per_bar = 16", "steps_per_unit = 2",
            "steps_per_pulse = 4", "pulses_per_bar = 4", "# GMD : feel droit (rapport médian 0,50) dans tous les fichiers retenus.",
            "default_swing_ratio = 0.5", f"# {s.bpm_note}", f"default_bpm = {default}", f"min_bpm = {lo}", f"max_bpm = {hi}", ""]
    # Pas de `[fixed_tonality]` : la tonalité vient de la variante de session (R2) ; training installe le sien (le
    # verrou de l'hôte gagne sur la variante).
    out += [s.comments.get("form", "").rstrip("\n"), "[form]", f"sequence = {s.sequence}"]
    for p in s.parts:
        out += ["[[form.parts]]", f'name = "{p.name}"', "melody_center_offset = 0", f"phrase_ends = [{p.bars}]",
                "roles = " + scalar(["Open"] + ["Free"] * (p.bars - 1)), ""]
    out += ["# Pas de mélodie jouée : mêmes réglages neutres que `wjazzd_swing`.", "[melody]", "cell_units = 8",
            "strong_units = [0, 2, 4, 6]", "weak_units = [1, 3, 5, 7]",
            "key_modes = " + s.hab.key_modes(), "[melody.params]", "weight_role_run = 3",
            "weight_role_arp = 0", "weight_role_turn = 0", "max_continuity_step_distance = 2", "continuity_max_retries = 8",
            "melody_center_midi = 72", "tonic_midi_min = 60", "tonic_midi_max = 72", "tessitura_min = 55",
            "tessitura_max = 88", "base_velocity = 80", "strong_eighth_velocity_offset = 8",
            "even_eighth_velocity_offset = -6", ""]
    out += [s.comments.get("harmony", "").rstrip("\n"), "[harmony]", 'model = "loop"', "steps_per_chord = 4", ""]
    for p in s.parts:
        out += [f"# {p.comment}", f"[harmony.parts.{p.name}]", 'kind = "chord_loop"', "loops = ["]
        for loop in p.loops:
            assert loop.beats == p.bars * 4, f"{s.id} {p.name} : {loop.beats} temps, attendu {p.bars * 4}"
            if loop.comment:
                out.append(f"  # {loop.comment}")
            out.append(f"  {{ weight = {loop.weight}, chords = [")
            out += [f"    {c.toml()}," for c in loop.chords]
            out.append("  ] },")
        out += ["]", ""]
    out += ["[accompaniment]", "strong_steps = [0, 8]", "strong_weight = 2", "weak_weight = 1",
            'tie_priority = ["I", "V", "IV"]', ""]
    out += [s.comments.get("bass", "").rstrip("\n"), "[accompaniment.bass]", f"low = {s.bass_low}", f"high = {s.bass_high}"]
    out.append("motifs = [")
    for i, m in enumerate(s.bass_motifs):
        notes = ", ".join(it(step=n.step, duration=n.duration, semitones=n.semitones, velocity=n.velocity)
                          for n in m.notes)
        out.append(f"  # {i}" + (f" : {m.comment}" if m.comment else ""))
        out.append(f'  {{ bars = {m.bars}, weight = {m.weight}, reference = "chord_root", notes = [{notes}] }},')
    out += ["]"]
    for t in s.bass_templates:
        out += ["", t.rstrip("\n")]
    if s.bass_passing:
        out += ["", s.bass_passing.rstrip("\n")]
    out += ["", s.comments.get("chords", "").rstrip("\n"), "[accompaniment.chords]", f"center = {s.chords_center}",
            f"max_drift = {s.chords_drift}", "default_pattern = 0", f'voicing = "{s.voicing}"']
    if s.note_count != 3:
        out.append(f"note_count = {s.note_count}")
    if s.color:
        out.append("color = " + s.color)
    if s.register_drift:
        out.append("register_drift = " + s.register_drift)
    out += [f"low = {s.chords_low}", f"high = {s.chords_high}",
            "patterns = [{ hits = [" + ", ".join(comp_hit(h) for h in s.default_comp) + "] }]", "comping = ["]
    for i, c in enumerate(s.comping):
        out.append(comping_line(c, i))
    out.append("]")
    if s.spread:
        out += ["", s.spread.rstrip("\n")]
    for ph in s.comp_phrases:
        out += ["", "[[accompaniment.chords.phrases]]", f"weight = {ph.weight}", f'tag = "{ph.tag}"']
        if ph.at != "any":
            out.append(f'at = "{ph.at}"')
        out.append(f"motifs = {ph.motifs}")
    out += ["", "[instruments]", "melody_program = 0", f"chords_program = {s.chords_program}",
            f"bass_program = {s.bass_program}", "percussion_bank = 128", f"percussion_program = {s.kit_program}", "",
            "[ornaments]", "rate_scale = 0.0", ""]
    out += [s.comments.get("timing", "").rstrip("\n"), s.voices_toml.rstrip("\n"), "[performance.tracks.Bass]", f"jitter = {s.bass_jitter}",
            "[performance.tracks.Chords]"]
    if s.chords_offset:
        out.append("offsets = " + scalar([s.chords_offset] * 16))
    out += [f"jitter = {s.chords_jitter}", ""]
    m = s.mix
    out += [s.comments.get("mix", "").rstrip("\n"), "[mix]", f"master_db = {m['master']}", "[mix.room]", "room_size = 0.3",
            "damping = 0.6", "width = 0.6", "level = 0.15", "[mix.tracks.Melody]", "level_db = 0.0", "pan = 0.0",
            "reverb_send = 0.0", "[mix.tracks.Chords]", f"level_db = {m['Chords']}", f"pan = {m.get('ChordsPan', -0.25)}",
            "reverb_send = 0.15", "[mix.tracks.Bass]", f"level_db = {m['Bass']}", "pan = 0.0", "reverb_send = 0.05",
            "[mix.tracks.PercussionLow]", f"level_db = {m['Percussion']}", "pan = 0.0", "reverb_send = 0.1",
            "[mix.tracks.PercussionHigh]", f"level_db = {m['Percussion']}", "pan = 0.1", "reverb_send = 0.1", "[mix.eq]",
            "highpass_hz = 35.0", "presence_hz = 3000.0", "presence_db = 1.0", "presence_q = 0.8", ""]
    return "\n".join(x for x in _squash(out)).rstrip("\n") + "\n"


def _squash(lines: list[str]) -> list[str]:
    """Deux lignes vides de suite deviennent une (les commentaires absents laissent des blancs)."""
    out: list[str] = []
    for line in lines:
        if line == "" and out and out[-1] == "":
            continue
        out.append(line)
    return out


def arrangement_toml(s: Style) -> str:
    return (
        f"# {s.tours} chorus de la forme ({' + '.join(str(p.bars) for p in s.parts)} mesures par chorus) : accords, basse et\n"
        "# batterie jouent dès la première mesure (accompagnement sous une grille, pas de montée en couches). Pas de\n"
        "# piste Melody. Le nombre de chorus est connu (un air par set, voir set.toml) : la retombée de l'arc joue au\n"
        "# dernier.\n"
        f"tours_per_tune = {s.tours}\n\n[entries]\n"
        "Chords = { tour = 1, part = 0, pass = 1 }\nBass = { tour = 1, part = 0, pass = 1 }\n"
        "PercussionLow = { tour = 1, part = 0, pass = 1 }\nPercussionHigh = { tour = 1, part = 0, pass = 1 }\n\n"
        "[intro]\nforms = []\npickup_eighths = 0\n"
    )


def dynamics_toml(s: Style) -> str:
    lo, default, hi = s.bpm
    down = round(min(30.0, default - lo), 1)  # le chargeur borne le décalage de tempo à ±30
    up = round(min(30.0, hi - default), 1)
    vc = {
        "chords_velocity": [[0.0, 0.7], [0.5, 1.0], [1.0, 1.25]],
        "bass_velocity": [[0.0, 0.8], [0.5, 1.0], [1.0, 1.2]],
        "percussion_velocity": [[0.0, 0.7], [0.5, 1.0], [1.0, 1.3]],
    }
    vc.update(s.velocity_curves)

    def curve(name: str) -> str:
        return f"{name} = {scalar(vc[name])}"

    return f"""# Pilotage par les quatre émotions : réglages de service, aucune mesure de corpus ne relie une émotion à un
# paramètre (README du profil). Arousal : tempo (−{down} à +{up} BPM autour de {default}, la plage de la source),
# vélocités des trois pistes, niveau de densité de la batterie (0 : timekeeper seul ; 1 : ossature ; 2 : notes
# fantômes). `density_percussion` ({'{'}-1..+1{'}'} niveau) rend l'arc (intensité et densité par chorus) audible.
# Tonalité : tirée par la variante de session (`session.toml`) ; les seuils de bascule parallèle sont écartés de la
# valence neutre (0,3) pour que le mode tiré ne bascule pas de lui-même.
neutral_valence = 0.3
valence_range = [-1.0, 1.0]
mode_shift_boundary = "part"
dark_below = 0.1
bright_above = 0.9
parallel_modes = {s.hab.parallel_modes()}
arousal_intensity = [[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]]
tension_intensity = [[0.0, 0.0], [1.0, 0.0]]
tempo_offset = [[0.0, -{down}], [0.5, 0.0], [1.0, {up}]]
melody_velocity = [[0.0, 1.0], [1.0, 1.0]]
{curve('chords_velocity')}
{curve('bass_velocity')}
{curve('percussion_velocity')}
percussion_intensity = [[0.0, -1.0], [0.5, 0.0], [1.0, 1.0]]
melody_duration = [[0.0, 1.0], [1.0, 1.0]]
strong_accent = [[0.0, 0.0], [1.0, 0.0]]
chords_intensity = [[0.0, 0.0], [1.0, 0.0]]
run_weight = [[0.0, 1.0], [1.0, 1.0]]
arp_weight = [[0.0, 1.0], [1.0, 1.0]]
turn_weight = [[0.0, 1.0], [1.0, 1.0]]
color_register = [[0.0, 0.0], [1.0, 0.0]]
brightness_accent = [[0.0, 0.0], [1.0, 0.0]]
density_percussion = [[0.0, -1.0], [0.5, 0.0], [1.0, 1.0]]
density_chords = [[0.0, 0.0], [1.0, 0.0]]
density_tracks = []
tension_offbeat = [[0.0, 0.0], [1.0, 0.0]]
tension_duration = [[0.0, 1.0], [1.0, 1.0]]
tension_dominant = [[0.0, 0.0], [1.0, 0.0]]
tension_chord_duration = [[0.0, 1.0], [1.0, 1.0]]
color = {s.hab.color()}
"""


SET_TOML = """# Un set d'un air (quatre chorus) : le nombre de chorus est connu, la retombée de l'arc joue au dernier. Sans
# `length`, le set serait sans fin (pas de retombée). La tonalité est celle de la variante de session
# (`session.toml`) : un air par set, les poids de changement de tonalité n'ont pas d'effet.
length = [{ tunes = 1, weight = 1 }]

[key]
same = 1.0
parallel = 1.0
relative = 1.0
fifth_up = 1.0
fourth_up = 1.0
step_up = 1.0
step_down = 1.0
other = 1.0
last_dark = 1.0

[[transitions]]
kind = "direct"
weight = 100

[end]
outro = false
gap_bars = [{ bars = 0, weight = 1 }]

[ramp]
tempo_bpm = [[0.0, 0.0], [1.0, 0.0]]
intensity_lift = [[0.0, 0.0], [1.0, 0.0]]
gate = [[0.0, 0.0], [0.2, 1.0], [0.8, 1.0], [1.0, 0.0]]

[signature]
return_rate = 0.0
memory = 1
"""

CELLS_TOML = """# Banque minimale exigée par le chargeur : le profil ne joue pas de mélodie (Melody absente de
# l'arrangement), ces cellules ne sont jamais entendues.
cells = [
 { role = "open", notes = "1 3 5 3 1 3 5 3", weight = 1 },
 { role = "open", notes = "5 3 1 3 5 3 1 3", weight = 1 },
 { role = "run", notes = "1 2 3 4 5 6 7 8", weight = 1 },
 { role = "run", notes = "8 7 6 5 4 3 2 1", weight = 1 },
]
"""


def arc_toml(s: Style, tags: list[str]) -> str:
    """Arc de chorus : exposition à la charleston, corps au ride (ou à la charleston ouverte), sommet, retombée.

    Valeurs de service (aucune mesure de corpus ne donne l'arc d'un morceau, A3). `tags` : étiquettes de
    batterie existantes ; l'exposition et la retombée prennent `hat`, le corps et le sommet `ride`, à défaut `open`.
    """
    quiet = '[{ tag = "hat" }]'
    loud_tag = "ride" if "ride" in tags else "open" if "open" in tags else None
    loud = f'[{{ tag = "{loud_tag}" }}]' if loud_tag else None

    def perc(p: str | None) -> str:
        return f", percussion = {p}" if p else ""

    return f"""# Arc de {s.id} sur plusieurs chorus (A3) : valeurs de service, aucune mesure de corpus (le GMD donne des
# grooves de 30 à 40 mesures, pas la forme d'un morceau). Quatre chorus de la forme :
#   1      exposition : comping clairsemé (tenue, blanches), ossature de batterie à la charleston fermée, doux
#          (niveau de densité 1 : grosse caisse, caisse claire, charleston) ;
#   2-3    corps : comping médian ou dense, {'ride' if loud_tag == 'ride' else 'charleston ouverte' if loud_tag == 'open' else 'charleston'}, intensité du profil ; la variante forte plus fort, avec un break
#          d'une mesure avant le chorus suivant (accords et basse se taisent, la batterie continue) ;
#   4      retombée (`fall`, nombre de chorus connu) : clairsemé, charleston seule (niveau de densité 0), stop-time
#          sur les deux dernières mesures ;
# au-delà (plus de quatre chorus ou set sans fin) : le sommet (plus fort, stop-time une fois sur deux), puis le corps
# et le sommet en boucle.
repeat_from = 1

[[stages]]
choruses = 1
variants = [
  {{ weight = 3, intensity = 0.65, density = 0.9, comping = [{{ tag = "sparse", weight = 3 }}, {{ tag = "medium", weight = 1 }}]{perc(quiet)} }},
  {{ weight = 1, intensity = 0.75, density = 1.0, comping = [{{ tag = "medium" }}]{perc(quiet)} }},
]

[[stages]]
choruses = 2
variants = [
  {{ weight = 2, comping = [{{ tag = "medium", weight = 2 }}, {{ tag = "dense", weight = 1 }}]{perc(loud)} }},
  {{ weight = 1, intensity = 1.2, density = 1.2, comping = [{{ tag = "dense" }}]{perc(loud)}, breaks = [{{ start = -1, bars = 1, chance = 0.5, tracks = ["Chords", "Bass"] }}] }},
]

[[stages]]
choruses = 1
variants = [
  {{ intensity = 1.4, density = 1.4, comping = [{{ tag = "dense" }}]{perc(loud)}, breaks = [{{ start = -2, bars = 2, chance = 0.5, tracks = ["Chords", "Bass"], keep_steps = [0] }}] }},
]

[[fall]]
intensity = 0.5
density = 0.5
comping = [{{ tag = "sparse" }}]
{('percussion = ' + quiet) if quiet else ''}
breaks = [{{ start = -2, bars = 2, tracks = ["Chords", "Bass"], keep_steps = [0] }}]
"""
