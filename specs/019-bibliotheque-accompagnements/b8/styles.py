"""B8 : données des profils `gmd_*` (grille, basse, comping, instruments, mixage) ; batterie : `gmd_batterie`.

Aucune mesure de basse ni de comping pour ces styles (WJazzD n'a que le jazz ; aucun corpus distribuable
de voicings, décision F3) : basse, comping et grilles sont des RÈGLES DE MÉTIER écrites ici, dites comme
telles dans chaque README. Seules la batterie (GMD), le feel et la plage de tempo viennent de la source.
"""

from __future__ import annotations

from emit import BassMotif, BassNote, Chord, Comp, CompPhrase, Loop, Part, Style
from gmd_batterie import DrumSpec

PROFILES: list[dict] = []

# --- aides d'écriture ------------------------------------------------------------------------


def C(semitones: int, quality: str, beats: int = 4, bass: int | None = None) -> Chord:
    return Chord(semitones, quality, beats, bass)


def bars(spec: str, q: str = "") -> list[Chord]:
    """`"0 7 9:m 5"` : une mesure par accord, `demi-tons[:qualité]` ; `q` : qualité par défaut."""
    out = []
    for tok in spec.split():
        semis, _, quality = tok.partition(":")
        out.append(C(int(semis), quality or q))
    return out


def N(step: int, duration: int, semitones: int, velocity: int) -> BassNote:
    return BassNote(step, duration, semitones, velocity)


def row(steps: list[int], duration: int, semitones: int | list[int], vel: int | list[int]) -> list[BassNote]:
    """Une note par pas de `steps`."""
    sem = semitones if isinstance(semitones, list) else [semitones] * len(steps)
    v = vel if isinstance(vel, list) else [vel] * len(steps)
    return [N(s, duration, a, b) for s, a, b in zip(steps, sem, v)]


EIGHTHS = list(range(0, 16, 2))
QUARTERS = [0, 4, 8, 12]

COMMON_CORPUS = (
    "Groove MIDI Dataset v1.0.0 (Google LLC), CC BY 4.0, https://creativecommons.org/licenses/by/4.0/ : Jon Gillick, "
    "Adam Roberts, Jesse Engel, Douglas Eck, David Bamman, « Learning to Groove with Inverse Sequence "
    "Transformations », ICML 2019 ; dix batteurs, kit électronique Roland TD-11. Seuls des agrégats entrent dans le "
    "profil (probabilités par pas d'un groupe de mesures, suites de groupes, moyennes de relances, plage de "
    "tempo) : aucune mesure d'un enregistrement n'est copiée. Basse, comping et grilles : règles de métier et "
    "formes communes écrites pour ce profil (aucune source mesurée de basse ni de voicings pour ce style : voir README). "
    "Manifeste et mesures : harmonium_lab corpus/gmd, references/gmd-*."
)

STRUM = """\
[accompaniment.chords.spread]
down_gap_ms = {down}
up_gap_ms = {up}
up_note_count = 3
gap_jitter = 0.3
anchor = 0.5
tempo_exponent = 0.0
ref_bpm = 120.0
max_spread_ratio = 0.4
velocity_slope = 4
velocity_jitter = 4
"""


def phrases(spec: list[tuple]) -> list[CompPhrase]:
    return [CompPhrase(w, t, m, at) for w, t, m, at in spec]


# --- rock ---------------------------------------------------------------------------------------

ROCK = Style(
    id="gmd_rock",
    name="Accompagnement rock (Groove MIDI Dataset, style rock)",
    tradition="Groove MIDI Dataset v1.0.0, style rock, 4/4 droit : 154 fichiers de batterie, 9 batteurs ; basse et guitare : règles de métier",
    tags=["rock", "accompagnement", "grille"],
    aliases=["rock"],
    title="accompagnement rock sous une grille",
    key=4,
    mode="mixolydian",
    bpm=(0, 0, 0),  # rempli depuis la source
    bpm_note="",
    parts=[
        Part("A", 8, "Couplet (huit mesures) : quatre grilles de forme commune, une tirée par air selon la graine.", [
            Loop(3, bars("0 0 5 5 0 0 7 5"), "I I IV IV I I V IV"),
            Loop(3, bars("0 7 9:m 5 0 7 9:m 5"), "I V vi IV (deux fois)"),
            Loop(2, bars("0 10 5 0 0 10 5 7"), "I bVII IV I | I bVII IV V (mode mixolydien)"),
            Loop(2, bars("9:m 5 0 7 9:m 5 0 7"), "vi IV I V (deux fois)"),
        ]),
        Part("B", 8, "Refrain (huit mesures) : quatre grilles.", [
            Loop(3, bars("5 7 0 0 5 7 9:m 7"), "IV V I I IV V vi V"),
            Loop(2, bars("5 0 7 9:m 5 0 7 7"), "IV I V vi IV I V V"),
            Loop(2, bars("0 5 7 5 0 5 7 5"), "I IV V IV (deux fois)"),
            Loop(1, bars("8 10 0 0 8 10 0 0"), "bVI bVII I I (deux fois)"),
        ]),
    ],
    sequence=[0, 1],
    tours=4,
    bass_low=28,
    bass_high=39,
    bass_motifs=[
        BassMotif(1, 4, row(EIGHTHS, 2, 0, [100, 88, 92, 88, 96, 88, 92, 88]), "croches à la fondamentale"),
        BassMotif(1, 3, row(QUARTERS, 4, 0, [100, 92, 96, 92]), "noires à la fondamentale"),
        BassMotif(1, 2, row(EIGHTHS, 2, [0, 0, 0, 12, 0, 0, 0, 12], [100, 88, 92, 94, 96, 88, 92, 94]), "croches, octave en contretemps des temps 2 et 4"),
        BassMotif(1, 2, [N(0, 3, 0, 100), N(3, 3, 0, 92), N(6, 2, 0, 90), N(8, 4, 0, 98), N(12, 2, 7, 92), N(14, 2, 0, 90)], "syncopé : fondamentale, contretemps, quinte"),
        BassMotif(2, 2, row(QUARTERS, 4, 0, 96) + row(list(range(16, 32, 2)), 2, [0, 0, 0, 0, 0, 0, 7, 0], [100, 88, 92, 88, 96, 88, 94, 90]), "deux mesures : noires, puis croches avec la quinte au temps 4"),
        BassMotif(1, 1, [N(0, 16, 0, 92)], "note tenue (couplet calme)"),
    ],
    bass_templates=[
        """\
# Gabarit : croches à la fondamentale, relance vers la fondamentale de l'accord suivant (pickup).
[[accompaniment.bass.templates]]
weight = 3
bars = 1
rhythm = [""" + ", ".join(f"{{ step = {s}, duration = 2, velocity = {v} }}" for s, v in zip(EIGHTHS, [98, 86, 90, 86, 94, 86, 90, 86])) + """]
cycle = [{ degree = "root" }]
pickup = { lead = 2, duration = 2, velocity = 90, degree = "root" }
"""
    ],
    chords_center=55,
    chords_drift=6,
    voicing="close_triad",
    note_count=4,
    spread=STRUM.format(down=7.0, up=5.0),
    default_comp=[(0, 8, 84), (8, 8, 80)],
    comping=[
        Comp(3, "sparse", [(0, 16, 82, "down")], "accord tenu"),
        Comp(3, "sparse", [(0, 8, 84, "down"), (8, 8, 78, "down")], "deux blanches"),
        Comp(1, "sparse", [], "silence (guitare tue)"),
        Comp(4, "medium", [(0, 4, 88, "down"), (4, 4, 80, "down"), (8, 4, 86, "down"), (12, 4, 80, "down")], "noires"),
        Comp(3, "medium", [(0, 6, 90, "down"), (6, 6, 82, "down"), (12, 4, 84, "down")], "x..x..x."),
        Comp(3, "medium", [(0, 6, 90, "down"), (6, 2, 78, "down"), (8, 4, 86, "down"), (14, 2, 84, "down", "anticipate")], "anticipation de l'accord suivant"),
        Comp(4, "dense", [(s, 2, v, "down") for s, v in zip(EIGHTHS, [90, 76, 82, 76, 86, 76, 82, 76])], "croches en descendant"),
        Comp(3, "dense", [(s, 2, v, "down" if i % 2 == 0 else "up") for i, (s, v) in enumerate(zip(EIGHTHS, [90, 70, 82, 70, 86, 70, 82, 70]))], "croches bas-haut"),
    ],
    comp_phrases=phrases([
        (3, "sparse", [0, 0, 1, 0], "any"), (2, "sparse", [1, 1], "any"), (2, "sparse", [0, 0, 0, 2], "any"), (2, "sparse", [1, 0], "part_end"),
        (3, "medium", [3, 3, 3, 5], "any"), (3, "medium", [4, 3, 4, 5], "any"), (2, "medium", [3, 4], "any"), (3, "medium", [3, 5], "part_end"),
        (3, "dense", [6, 6, 6, 7], "any"), (2, "dense", [7, 6, 7, 6], "any"), (2, "dense", [6, 7], "any"), (3, "dense", [6, 5], "part_end"),
    ]),
    chords_program=29,
    bass_program=34,
    kit_program=8,
    chords_offset=0.0,
    corpus="",
)


# --- pop ------------------------------------------------------------------------------------------

POP = Style(
    id="gmd_pop", name="Accompagnement pop (Groove MIDI Dataset, famille pop)",
    tradition="Groove MIDI Dataset v1.0.0, famille pop (pop, pop/soft, pop/groove7), 4/4 droit : 14 fichiers de batterie, 4 batteurs ; basse et piano : règles de métier",
    tags=["pop", "accompagnement", "grille"], aliases=["pop"], title="accompagnement pop sous une grille",
    key=7, mode="ionian", bpm=(0, 0, 0), bpm_note="",
    parts=[
        Part("A", 8, "Couplet (huit mesures) : quatre grilles de forme commune.", [
            Loop(3, bars("0 7 9:m 5 0 7 9:m 5"), "I V vi IV (deux fois)"),
            Loop(2, bars("9:m 5 0 7 9:m 5 0 7"), "vi IV I V (deux fois)"),
            Loop(2, bars("0 9:m 5 7 0 9:m 5 7"), "I vi IV V (deux fois)"),
            Loop(1, bars("0 5 9:m 7 0 5 9:m 7"), "I IV vi V (deux fois)"),
        ]),
        Part("B", 8, "Refrain (huit mesures) : quatre grilles.", [
            Loop(3, bars("5 0 7 9:m 5 0 7 9:m"), "IV I V vi (deux fois)"),
            Loop(2, bars("5 7 4:m 9:m 5 7 0 0"), "IV V iii vi IV V I I"),
            Loop(2, bars("9:m 7 5 0 9:m 7 5 0"), "vi V IV I (deux fois)"),
            Loop(1, bars("0 7 5 5 0 7 9:m 7"), "I V IV IV I V vi V"),
        ]),
    ],
    sequence=[0, 1], tours=4, bass_low=28, bass_high=39,
    bass_motifs=[
        BassMotif(1, 4, row(QUARTERS, 4, 0, [96, 88, 92, 88]), "noires à la fondamentale"),
        BassMotif(1, 4, [N(0, 6, 0, 98), N(6, 2, 0, 86), N(8, 4, 0, 94), N(12, 2, 7, 86), N(14, 2, 0, 84)], "syncopé : anticipation du temps 3 et du temps 1 suivant"),
        BassMotif(1, 3, row(EIGHTHS, 2, 0, [98, 82, 90, 82, 94, 82, 90, 82]), "croches à la fondamentale"),
        BassMotif(1, 2, [N(0, 8, 0, 94), N(8, 8, 0, 90)], "blanches (couplet calme)"),
        BassMotif(1, 2, row(EIGHTHS, 2, [0, 12, 0, 12, 0, 12, 0, 12], [98, 80, 90, 80, 94, 80, 90, 80]), "octaves en croches (refrain porté)"),
        BassMotif(2, 2, [N(0, 6, 0, 98), N(6, 2, 0, 86), N(8, 6, 0, 94), N(14, 2, 7, 84), N(16, 4, 0, 96), N(20, 4, 0, 88), N(24, 4, 0, 94), N(28, 2, 7, 86), N(30, 2, 12, 84)], "deux mesures : syncopé, puis noires, quinte et octave pour relancer"),
    ],
    bass_templates=["""\
# Gabarit : noires, relance vers la fondamentale de l'accord suivant (pickup en croche avant le changement).
[[accompaniment.bass.templates]]
weight = 3
bars = 1
rhythm = [{ step = 0, duration = 4, velocity = 96 }, { step = 4, duration = 4, velocity = 88 }, { step = 8, duration = 4, velocity = 92 }, { step = 12, duration = 2, velocity = 88 }]
cycle = [{ degree = "root" }]
pickup = { lead = 2, duration = 2, velocity = 86, degree = "root" }
"""],
    chords_center=62, chords_drift=6, voicing="close_triad", note_count=3,
    default_comp=[(0, 8, 74), (8, 8, 70)],
    comping=[
        Comp(3, "sparse", [(0, 16, 72)], "accord tenu"),
        Comp(3, "sparse", [(0, 8, 74), (8, 8, 68)], "deux blanches"),
        Comp(1, "sparse", [], "silence"),
        Comp(4, "medium", [(0, 4, 78), (4, 4, 70), (8, 4, 76), (12, 4, 70)], "noires"),
        Comp(3, "medium", [(0, 6, 78), (6, 2, 68), (8, 4, 74), (14, 2, 72, "anticipate")], "syncopé, anticipation de l'accord suivant"),
        Comp(3, "medium", [(0, 4, 78), (6, 2, 70), (10, 2, 72), (14, 2, 70)], "x..x.x.x"),
        Comp(4, "dense", [(s, 2, v) for s, v in zip(EIGHTHS, [80, 66, 74, 66, 78, 66, 74, 66])], "croches"),
        Comp(3, "dense", [(0, 2, 80)] + [(s, 2, v) for s, v in zip([2, 6, 10, 14], [72, 74, 72, 74])], "temps 1 puis croches en contretemps"),
    ],
    comp_phrases=phrases([
        (3, "sparse", [0, 1, 0, 1], "any"), (2, "sparse", [1, 1], "any"), (2, "sparse", [0, 0, 0, 2], "any"), (2, "sparse", [1, 0], "part_end"),
        (3, "medium", [3, 3, 4, 3], "any"), (3, "medium", [3, 5, 3, 4], "any"), (2, "medium", [3, 4], "any"), (3, "medium", [3, 4], "part_end"),
        (3, "dense", [6, 6, 6, 7], "any"), (2, "dense", [6, 7, 6, 7], "any"), (2, "dense", [6, 7], "any"), (3, "dense", [6, 4], "part_end"),
    ]),
    chords_program=0, bass_program=33, kit_program=0,
)

PROFILES.append({
    "style": POP,
    "drum": DrumSpec(["pop", "pop/soft", "pop/groove7"], min_files=2, min_measures=6, max_share=0.9, max_groups=7, arc_tags=["hat", "ride"]),
    "drum_why": "14 fichiers seulement, quatre batteurs : les seuils de L2 (3 fichiers, 20 mesures, 60 %) ne laissent qu'un groupe ; relâchés, ils en gardent plusieurs, qui décrivent surtout un ou deux batteurs",
    "source": "la famille `pop` du Groove MIDI Dataset v1.0.0 (`pop` : 8 fichiers, 174 mesures, « juste » ; `pop/soft` et `pop/groove7` : « juste » chacun ; L2 § 4.2). Les trois sous-styles du GMD sont réunis parce que L2 leur donne chacun « juste » et que le GMD les range sous la même famille.",
    "key_name": "sol ionien",
    "fills_none_weight": 3,
    "corpus": COMMON_CORPUS,
    "readme": {
        "genre": "pop",
        "nom_limite": "il décrit la batterie de 14 fichiers joués par quatre batteurs ; le piano et la basse sont des règles de métier. Les fichiers `pop` sont à 132-142 BPM, `pop/soft` à 83.",
        "intro": "Profil d'accompagnement **sous une grille d'accords** : basse au doigt (motifs, gabarit à relance), piano en triades (rythmes de comping, phrases) et batterie pop du GMD. Sans grille fournie : deux parties de huit mesures de forme commune. Aucune mélodie.",
        "swing": "0,49 (médiane de 14 fichiers)",
        "kit": "kit standard",
        "basse": "basse au doigt (programme 33), six motifs d'une ou deux mesures tirés par poids (noires, syncopé, croches, blanches, octaves, deux mesures) et un gabarit en noires avec relance vers la fondamentale ; fenêtre 28..39",
        "comping": "piano à queue (programme 0), triades conduites par les voix communes (centre 62) ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures, quatre de mise en place `part_end`",
        "timing": "basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service)",
        "regles": "basse de pop (noires, syncope, octaves), piano en noires, croches et contretemps",
        "variete": "Entre graines : une des quatre boucles de chaque partie, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus : l'arc (comping clairsemé puis dense, charleston puis ride, breaks, stop-time).",
        "limites": "- **Peu de batteurs** : quatre ; les groupes de batterie retenus décrivent un ou deux batteurs chacun (voir « groupes retenus »).\n- Les fichiers `pop/soft` (83 BPM) et `pop` (132-142 BPM) forment deux régimes de tempo : la plage de tempo est la plus large du lot (centiles 10 et 90).",
    },
})

# --- hip-hop --------------------------------------------------------------------------------------

HIPHOP = Style(
    id="gmd_hiphop", name="Accompagnement hip-hop (Groove MIDI Dataset, famille hiphop)",
    tradition="Groove MIDI Dataset v1.0.0, famille hiphop (hiphop, hiphop/groove6), 4/4 droit : 34 fichiers de batterie, 5 batteurs ; basse et piano électrique : règles de métier",
    tags=["hiphop", "accompagnement", "grille"], aliases=["hiphop", "hip-hop"], title="accompagnement hip-hop sous une grille",
    key=0, mode="aeolian", bpm=(0, 0, 0), bpm_note="",
    parts=[
        Part("A", 8, "Couplet (huit mesures) : quatre boucles mineures de forme commune (septièmes).", [
            Loop(3, bars("0:m7 8:maj7 3:maj7 10:7 0:m7 8:maj7 3:maj7 10:7"), "i7 bVI bIII bVII7 (deux fois)"),
            Loop(3, bars("0:m7 0:m7 5:m7 5:m7 0:m7 0:m7 5:m7 5:m7"), "i7 i7 iv7 iv7 (deux fois)"),
            Loop(2, bars("0:m7 10:7 8:maj7 7:7 0:m7 10:7 8:maj7 7:7"), "i7 bVII7 bVI V7 (deux fois)"),
            Loop(1, bars("0:m7 3:maj7 10:7 5:m7 0:m7 3:maj7 10:7 7:7"), "i7 bIII bVII7 iv7 i7 bIII bVII7 V7"),
        ]),
        Part("B", 8, "Refrain (huit mesures) : quatre boucles.", [
            Loop(3, bars("8:maj7 10:7 0:m7 0:m7 8:maj7 10:7 0:m7 0:m7"), "bVI bVII7 i7 i7 (deux fois)"),
            Loop(2, bars("5:m7 0:m7 8:maj7 7:7 5:m7 0:m7 8:maj7 7:7"), "iv7 i7 bVI V7 (deux fois)"),
            Loop(2, bars("3:maj7 10:7 0:m7 0:m7 3:maj7 10:7 5:m7 7:7"), "bIII bVII7 i7 i7 bIII bVII7 iv7 V7"),
            Loop(1, bars("0:m7 0:m7 0:m7 0:m7 8:maj7 8:maj7 10:7 10:7"), "i7 (quatre mesures) bVI bVI bVII7 bVII7"),
        ]),
    ],
    sequence=[0, 1], tours=4, bass_low=24, bass_high=35,
    bass_motifs=[
        BassMotif(1, 4, [N(0, 6, 0, 104), N(8, 4, 0, 96), N(12, 4, 0, 90)], "sub : fondamentale longue, temps 3, temps 4"),
        BassMotif(1, 3, [N(0, 3, 0, 104), N(3, 3, 0, 90), N(8, 6, 0, 98), N(14, 2, 0, 88)], "sub syncopé"),
        BassMotif(1, 3, [N(0, 8, 0, 102), N(10, 6, 12, 90)], "fondamentale tenue, octave au temps 3,5"),
        BassMotif(1, 2, [N(0, 16, 0, 100)], "note tenue"),
        BassMotif(2, 2, [N(0, 4, 0, 104), N(6, 2, 0, 88), N(8, 6, 0, 98), N(16, 6, 0, 102), N(22, 2, 12, 86), N(24, 4, 0, 96), N(28, 4, 7, 88)], "deux mesures : sub, puis octave et quinte pour relancer"),
    ],
    chords_center=62, chords_drift=8, voicing="drop2", note_count=3, chords_low=52, chords_high=76,
    default_comp=[(0, 8, 70), (8, 8, 66)],
    comping=[
        Comp(3, "sparse", [(0, 16, 68)], "accord tenu"),
        Comp(3, "sparse", [(0, 6, 72), (10, 6, 64)], "deux accords, le second en contretemps"),
        Comp(1, "sparse", [], "silence"),
        Comp(4, "medium", [(0, 6, 74), (6, 4, 66), (12, 4, 68)], "x..x..x."),
        Comp(3, "medium", [(4, 4, 70), (10, 4, 68), (14, 2, 68, "anticipate")], "contretemps, anticipation"),
        Comp(3, "medium", [(0, 4, 74), (8, 4, 70), (14, 2, 66, "anticipate")], "temps 1, 3 et anticipation"),
        Comp(3, "dense", [(0, 2, 76), (3, 2, 68), (6, 2, 72), (10, 2, 70), (13, 3, 68)], "syncopé en doubles croches"),
        Comp(3, "dense", [(2, 2, 70), (6, 2, 74), (10, 2, 70), (14, 2, 74)], "croches en contretemps"),
    ],
    comp_phrases=phrases([
        (3, "sparse", [0, 1, 0, 1], "any"), (2, "sparse", [1, 1], "any"), (2, "sparse", [0, 0, 0, 2], "any"), (2, "sparse", [1, 0], "part_end"),
        (3, "medium", [3, 3, 4, 3], "any"), (3, "medium", [3, 5, 3, 4], "any"), (2, "medium", [3, 4], "any"), (3, "medium", [5, 3], "part_end"),
        (3, "dense", [6, 6, 7, 6], "any"), (2, "dense", [7, 6, 7, 6], "any"), (2, "dense", [6, 7], "any"), (3, "dense", [6, 4], "part_end"),
    ]),
    chords_program=4, bass_program=38, kit_program=25,
)

PROFILES.append({
    "style": HIPHOP,
    "drum": DrumSpec(["hiphop", "hiphop/groove6"], min_files=2, min_measures=15, max_share=0.8, max_groups=7, arc_tags=["hat"]),
    "drum_why": "34 fichiers mais cinq batteurs, groupes de gestes de charleston très partagés : seuils de L2 relâchés à deux fichiers",
    "source": "la famille `hiphop` du Groove MIDI Dataset v1.0.0 (`hiphop` : 30 fichiers, 796 mesures, « assez » ; `hiphop/groove6` : « juste » ; fills « juste », L2 § 4.2). 90 % des fichiers sont droits (80 sur 91 ; 10 swing, 1 ternaire), seuls les droits sont retenus.",
    "key_name": "do éolien",
    "fills_none_weight": 2,
    "corpus": COMMON_CORPUS,
    "readme": {
        "genre": "hip-hop",
        "nom_limite": "il décrit la batterie de 34 fichiers (cinq batteurs) ; la basse sub, le piano électrique et les boucles sont des règles de métier. Aucun échantillonnage, scratch ni voix.",
        "intro": "Profil d'accompagnement **sous une grille d'accords** : basse synthétique tenue (programme 38), piano électrique en voicings à quatre sons, batterie hip-hop du GMD (kit 808/909). Sans grille fournie : deux parties de huit mesures de boucles mineures à septièmes. Aucune mélodie.",
        "swing": "0,50 (médiane de 34 fichiers droits ; 10 fichiers à swing léger écartés)",
        "kit": "kit 808/909",
        "basse": "basse synthétique (programme 38), cinq motifs d'une ou deux mesures tirés par poids (sub tenu, sub syncopé, octave en contretemps, note tenue, deux mesures avec quinte) ; fenêtre 24..35 (do grave)",
        "comping": "piano électrique « Tine » (programme 4), voicings `drop2` à quatre sons dans la fenêtre 52..76 ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures, quatre de mise en place `part_end`",
        "timing": "basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service)",
        "regles": "sub tenu et syncopé, accords de touches clairsemés",
        "variete": "Entre graines : une des quatre boucles de chaque partie, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie (charleston en croches, doubles croches, noires plus) et leurs phrases, les relances. Entre chorus : l'arc.",
        "limites": "- **Un seul timekeeper** : le hip-hop du GMD est joué à la charleston fermée ; les étiquettes d'arc n'ont donc que `hat` (pas de ride) et le corps de l'arc ne change pas de timekeeper.\n- **Swing** : dix fichiers à swing léger écartés (feel droit seulement).",
    },
})

# --- soul -----------------------------------------------------------------------------------------

SOUL = Style(
    id="gmd_soul", name="Accompagnement soul (Groove MIDI Dataset, famille soul)",
    tradition="Groove MIDI Dataset v1.0.0, famille soul (soul, soul/motown, soul/groove3, 4, 9, 10), 4/4 droit : 28 fichiers de batterie, 4 batteurs ; basse et piano électrique : règles de métier",
    tags=["soul", "rnb", "accompagnement", "grille"], aliases=["soul"], title="accompagnement soul sous une grille",
    key=3, mode="ionian", bpm=(0, 0, 0), bpm_note="",
    parts=[
        Part("A", 8, "Couplet (huit mesures) : quatre grilles de forme commune à septièmes.", [
            Loop(3, bars("0:maj7 9:m7 2:m7 7:7 0:maj7 9:m7 2:m7 7:7"), "I vi ii V (deux fois)"),
            Loop(2, bars("0:maj7 0:maj7 5:maj7 5:maj7 0:maj7 9:m7 2:m7 7:7"), "I I IV IV I vi ii V"),
            Loop(2, bars("0:7 5:7 0:7 0:7 5:7 5:7 0:7 7:7"), "I7 IV7 I7 I7 IV7 IV7 I7 V7 (soul blues)"),
            Loop(1, bars("2:m7 7:7 0:maj7 9:m7 2:m7 7:7 0:maj7 0:maj7"), "ii V I vi ii V I I"),
        ]),
        Part("B", 8, "Refrain (huit mesures) : trois grilles.", [
            Loop(3, bars("5:maj7 4:m7 2:m7 7:7 5:maj7 4:m7 2:m7 7:7sus4"), "IV iii ii V (deux fois, la seconde sur V7sus4)"),
            Loop(2, bars("5:maj7 7:7 4:m7 9:m7 5:maj7 7:7 0:maj7 0:maj7"), "IV V iii vi IV V I I"),
            Loop(2, bars("9:m7 2:m7 7:7 0:maj7 9:m7 2:m7 7:7 0:maj7"), "vi ii V I (deux fois)"),
        ]),
    ],
    sequence=[0, 1], tours=4, bass_low=28, bass_high=39,
    bass_motifs=[
        BassMotif(1, 4, [N(0, 4, 0, 100), N(4, 2, 12, 90), N(6, 2, 0, 88), N(8, 4, 0, 96), N(12, 2, 7, 88), N(14, 2, 0, 86)], "racine, octave, racine, quinte"),
        BassMotif(1, 3, [N(0, 3, 0, 100), N(3, 3, 0, 90), N(6, 2, 0, 86), N(8, 2, 12, 92), N(10, 2, 0, 86), N(12, 4, 7, 92)], "syncopé avec octave"),
        BassMotif(1, 3, row(QUARTERS, 4, 0, [98, 88, 94, 88]), "noires"),
        BassMotif(1, 2, [N(0, 8, 0, 96), N(8, 6, 0, 92), N(14, 2, 7, 84)], "blanches, anticipation de la quinte"),
        BassMotif(2, 2, [N(0, 4, 0, 100), N(4, 2, 12, 90), N(6, 2, 0, 88), N(8, 4, 0, 96), N(12, 4, 7, 90), N(16, 6, 0, 100), N(22, 2, 0, 86), N(24, 2, 12, 92), N(26, 2, 0, 86), N(28, 4, 7, 90)], "deux mesures : figure d'octave puis remontée par la quinte"),
    ],
    bass_templates=["""\
# Gabarit : croches roots/octaves en alternance, relance vers la fondamentale de l'accord suivant.
[[accompaniment.bass.templates]]
weight = 2
bars = 1
rhythm = [{ step = 0, duration = 4, velocity = 98 }, { step = 4, duration = 2, velocity = 88 }, { step = 6, duration = 2, velocity = 84 }, { step = 8, duration = 4, velocity = 94 }, { step = 12, duration = 2, velocity = 86 }]
cycle = [{ degree = "root" }, { degree = "root", octave = 1 }, { degree = "root" }, { degree = "fifth" }, { degree = "root" }]
pickup = { lead = 2, duration = 2, velocity = 86, degree = "root" }
"""],
    chords_center=62, chords_drift=8, voicing="drop2", note_count=3, chords_low=52, chords_high=76,
    default_comp=[(0, 8, 70), (8, 8, 66)],
    comping=[
        Comp(3, "sparse", [(0, 16, 70)], "accord tenu"),
        Comp(3, "sparse", [(0, 8, 72), (8, 8, 66)], "deux blanches"),
        Comp(1, "sparse", [], "silence"),
        Comp(4, "medium", [(0, 4, 76), (6, 2, 66), (10, 2, 70), (14, 2, 68, "anticipate")], "temps 1 puis contretemps"),
        Comp(3, "medium", [(0, 6, 76), (6, 2, 66), (8, 4, 72), (14, 2, 68, "anticipate")], "syncopé"),
        Comp(3, "medium", [(2, 2, 70), (6, 2, 74), (10, 2, 70), (14, 2, 74)], "croches en contretemps"),
        Comp(4, "dense", [(0, 2, 78), (3, 1, 62), (4, 2, 70), (6, 2, 72), (8, 2, 76), (11, 1, 62), (12, 2, 70), (14, 2, 72)], "chank en doubles croches"),
        Comp(3, "dense", [(s, 2, v) for s, v in zip(EIGHTHS, [80, 64, 72, 64, 78, 64, 72, 64])], "croches"),
    ],
    comp_phrases=phrases([
        (3, "sparse", [0, 1, 0, 1], "any"), (2, "sparse", [1, 1], "any"), (2, "sparse", [0, 0, 0, 2], "any"), (2, "sparse", [1, 0], "part_end"),
        (3, "medium", [3, 3, 4, 3], "any"), (3, "medium", [4, 3, 5, 3], "any"), (2, "medium", [3, 4], "any"), (3, "medium", [3, 5], "part_end"),
        (3, "dense", [6, 6, 7, 6], "any"), (2, "dense", [7, 6, 7, 6], "any"), (2, "dense", [6, 7], "any"), (3, "dense", [6, 4], "part_end"),
    ]),
    chords_program=4, bass_program=33, kit_program=0,
)

PROFILES.append({
    "style": SOUL,
    "drum": DrumSpec(["soul", "soul/motown", "soul/groove3", "soul/groove4", "soul/groove9", "soul/groove10"], min_files=2, min_measures=10, max_share=0.6, max_groups=7, arc_tags=["hat", "ride"]),
    "drum_why": "quatre batteurs, groupes de charleston en croches et doubles croches ; seuils de L2 relâchés à deux fichiers et dix mesures",
    "source": "la famille `soul` du Groove MIDI Dataset v1.0.0 (`soul` : 8 fichiers, 221 mesures ; `soul/motown` et les quatre `soul/groove*` : « juste » chacun ; fills « juste », L2 § 4.2). Les sous-styles sont réunis parce que L2 leur donne chacun « juste » et qu'ils sont sous la même famille ; le soul tient lieu de R&B (iReal « RnB ») faute de style `rnb` dans le GMD.",
    "key_name": "mi bémol ionien",
    "fills_none_weight": 3,
    "corpus": COMMON_CORPUS,
    "readme": {
        "genre": "soul",
        "nom_limite": "il décrit la batterie de 28 fichiers joués par quatre batteurs (dont les quatre séries « groove » d'un même protocole) ; piano électrique et basse sont des règles de métier.",
        "intro": "Profil d'accompagnement **sous une grille d'accords** : basse au doigt, piano électrique « Tine » en voicings à quatre sons et batterie soul du GMD. Sans grille fournie : deux parties de huit mesures à septièmes. Aucune mélodie.",
        "swing": "0,48 (médiane de 28 fichiers)",
        "kit": "kit standard",
        "basse": "basse au doigt (programme 33), cinq motifs d'une ou deux mesures (racine-octave-racine-quinte, syncopé avec octave, noires, blanches, deux mesures) et un gabarit en cycle racine-octave-quinte avec relance ; fenêtre 28..39",
        "comping": "piano électrique « Tine » (programme 4), voicings `drop2` à quatre sons dans la fenêtre 52..76 ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense` (dont un « chank » en doubles croches), douze phrases de 2 à 4 mesures dont quatre `part_end`",
        "timing": "basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service)",
        "regles": "figures de basse soul (octave, quinte), « chank » du clavier électrique",
        "variete": "Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse et le gabarit, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus : l'arc.",
        "limites": "- **Peu de batteurs** : quatre, et quatre séries « groove » de 64 mesures chacune d'un même protocole.\n- **Pas de cuivres ni de cordes** : un seul instrument d'accords par profil.",
    },
})


# --- reggae ---------------------------------------------------------------------------------------

REGGAE = Style(
    id="gmd_reggae", name="Accompagnement reggae (Groove MIDI Dataset, style reggae)",
    tradition="Groove MIDI Dataset v1.0.0, style reggae, 4/4 droit : 2 fichiers de batterie, 2 batteurs (3 fichiers `beat` en tout) ; basse et guitare : règles de métier",
    tags=["reggae", "accompagnement", "grille"], aliases=["reggae"], title="accompagnement reggae sous une grille",
    key=9, mode="ionian", bpm=(0, 0, 0), bpm_note="",
    parts=[
        Part("A", 8, "Couplet (huit mesures) : quatre grilles de forme commune, deux accords ou boucle de quatre.", [
            Loop(3, bars("0 0 5 5 0 0 5 5"), "I I IV IV (deux fois)"),
            Loop(3, bars("0 5 0 7 0 5 0 7"), "I IV I V (deux fois)"),
            Loop(2, bars("0 9:m 5 7 0 9:m 5 7"), "I vi IV V (deux fois)"),
            Loop(1, bars("9:m 9:m 5 5 9:m 9:m 4:m 4:m"), "vi vi IV IV vi vi iii iii"),
        ]),
        Part("B", 8, "Refrain (huit mesures) : trois grilles.", [
            Loop(3, bars("5 5 0 0 5 5 7 7"), "IV IV I I IV IV V V"),
            Loop(2, bars("0 7 9:m 5 0 7 9:m 5"), "I V vi IV (deux fois)"),
            Loop(2, bars("5 0 5 0 5 7 0 0"), "IV I IV I IV V I I"),
        ]),
    ],
    sequence=[0, 1], tours=4, bass_low=28, bass_high=39,
    bass_motifs=[
        BassMotif(1, 4, [N(6, 2, 0, 92), N(8, 4, 0, 100), N(12, 2, 7, 90), N(14, 2, 0, 86)], "temps 1 laissé vide (one drop), basse sur le temps 3"),
        BassMotif(1, 3, [N(0, 6, 0, 100), N(8, 4, 0, 96), N(12, 4, 7, 90)], "fondamentale longue, temps 3, quinte au temps 4"),
        BassMotif(1, 3, row(QUARTERS, 4, 0, [96, 88, 94, 88]), "noires à la fondamentale"),
        BassMotif(1, 2, [N(2, 2, 0, 90), N(6, 2, 0, 88), N(8, 4, 7, 94), N(12, 4, 0, 92)], "contretemps, quinte au temps 3"),
        BassMotif(2, 2, [N(6, 2, 0, 92), N(8, 4, 0, 100), N(14, 2, 7, 88), N(16, 4, 0, 100), N(20, 2, 0, 88), N(22, 2, 12, 90), N(24, 4, 7, 94), N(28, 4, 0, 92)], "deux mesures : one drop, puis octave et quinte"),
    ],
    chords_center=64, chords_drift=6, voicing="close_triad", note_count=3, chords_low=52, chords_high=76,
    spread=STRUM.format(down=5.0, up=4.0),
    default_comp=[(6, 2, 80, "up"), (14, 2, 76, "up")],
    comping=[
        Comp(4, "sparse", [(6, 2, 80, "up"), (14, 2, 76, "up")], "skank sur les contretemps des temps 2 et 4"),
        Comp(2, "sparse", [(14, 2, 74, "up")], "un seul skank (temps 4)"),
        Comp(1, "sparse", [], "silence"),
        Comp(4, "medium", [(2, 2, 74, "up"), (6, 2, 82, "up"), (10, 2, 74, "up"), (14, 2, 82, "up")], "skank sur tous les contretemps"),
        Comp(3, "medium", [(6, 2, 82, "up"), (10, 2, 72, "up"), (14, 2, 80, "up")], "skank, premier contretemps vide"),
        Comp(3, "medium", [(2, 2, 76, "up"), (6, 2, 82, "up"), (14, 2, 82, "up")], "skank, troisième contretemps vide"),
        Comp(4, "dense", [(2, 2, 74, "up"), (3, 1, 60, "up"), (6, 2, 82, "up"), (7, 1, 60, "up"), (10, 2, 74, "up"), (11, 1, 60, "up"), (14, 2, 82, "up"), (15, 1, 60, "up")], "skank doublé en doubles croches"),
        Comp(2, "dense", [(2, 2, 76, "up"), (6, 2, 84, "up"), (10, 2, 76, "up"), (12, 2, 66, "down"), (14, 2, 84, "up")], "skank et gratté sur le temps 4"),
    ],
    comp_phrases=phrases([
        (3, "sparse", [0, 0, 1, 0], "any"), (2, "sparse", [0, 0], "any"), (2, "sparse", [0, 0, 0, 2], "any"), (2, "sparse", [1, 0], "part_end"),
        (3, "medium", [3, 3, 4, 3], "any"), (3, "medium", [3, 5, 3, 4], "any"), (2, "medium", [3, 4], "any"), (3, "medium", [3, 5], "part_end"),
        (3, "dense", [6, 6, 6, 7], "any"), (2, "dense", [6, 7, 6, 6], "any"), (2, "dense", [6, 7], "any"), (3, "dense", [6, 3], "part_end"),
    ]),
    chords_program=27, bass_program=33, kit_program=0,
)

PROFILES.append({
    "style": REGGAE,
    "drum": DrumSpec(["reggae"], min_files=2, min_measures=6, max_share=0.9, max_groups=6, arc_tags=["hat"]),
    "drum_why": "deux fichiers 4/4 droits et deux batteurs seulement : seuils réduits à deux fichiers et six mesures (les seuils de L2 ne retiennent aucun groupe)",
    "source": "le style `reggae` du Groove MIDI Dataset v1.0.0 (3 fichiers `beat`, 253 mesures, « juste » au sens de L2 ; le groupe 4/4 droit retenu n'en compte que 2, 132 mesures, deux batteurs, 16 fichiers `fill`). `reggae/slow` (1 fichier, 29 mesures, « trop peu ») et l'unique fichier ternaire ne sont pas lus.",
    "key_name": "la ionien",
    "fills_none_weight": 4,
    "corpus": COMMON_CORPUS,
    "readme": {
        "genre": "reggae",
        "nom_limite": "il décrit la batterie de DEUX fichiers joués par deux batteurs : les grooves y ont la grosse caisse sur les quatre temps (la moitié des mesures) plutôt que le « one drop » classique ; la basse et la guitare, elles, suivent le métier (temps 1 laissé vide, skank sur les contretemps).",
        "intro": "Profil d'accompagnement **sous une grille d'accords** : basse au doigt, guitare claire en skank (attaques vers le haut, trois cordes aiguës) et batterie reggae du GMD. Sans grille fournie : deux parties de huit mesures à deux ou quatre accords. Aucune mélodie.",
        "swing": "0,49 (médiane de 2 fichiers droits)",
        "kit": "kit standard",
        "basse": "basse au doigt (programme 33), cinq motifs d'une ou deux mesures (« one drop » qui laisse le temps 1 vide, fondamentale longue, noires, contretemps, deux mesures) ; fenêtre 28..39",
        "comping": "guitare claire (programme 27), triades jouées vers le haut sur les trois cordes aiguës (`up_note_count = 3`, écart 4 ms) : huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense` (skank sur les contretemps, doublé en doubles croches), douze phrases de 2 à 4 mesures dont quatre `part_end`",
        "timing": "basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service ; le reggae joue en retard sur le temps, rien n'est mesuré ici)",
        "regles": "one drop, skank sur les contretemps",
        "variete": "Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie (groupes de charleston fermée) et leurs phrases, les relances. Entre chorus : l'arc (comping clairsemé puis dense, breaks, stop-time) ; la batterie n'a qu'un timekeeper (`hat`).",
        "limites": "- **Fiabilité** : « juste » pour le style (3 fichiers), mais **le groupe retenu ne compte que deux fichiers et deux batteurs** : la variété de batterie d'une graine à l'autre est faible par construction (trois groupes, donc trois patterns par densité au plus). Le profil ne dit rien du reggae joué par d'autres batteurs.\n- **Pas d'orgue « bubble »** : un seul instrument d'accords par profil (la guitare tient le skank).\n- **Pas de retard sur le temps** (le micro-timing reggae n'est pas mesuré ici ; décalage par piste de basse et d'accords à 0).",
    },
})

# --- disco ----------------------------------------------------------------------------------------

DISCO = Style(
    id="gmd_dance_disco", name="Accompagnement disco (Groove MIDI Dataset, style dance/disco)",
    tradition="Groove MIDI Dataset v1.0.0, style dance/disco, 4/4 droit : 5 fichiers de batterie, 2 batteurs ; basse octaves et guitare : règles de métier",
    tags=["disco", "dance", "accompagnement", "grille"], aliases=["disco"], title="accompagnement disco sous une grille",
    key=9, mode="aeolian", bpm=(0, 0, 0), bpm_note="",
    parts=[
        Part("A", 8, "Couplet (huit mesures) : quatre boucles mineures de forme commune (septièmes).", [
            Loop(3, bars("0:m7 0:m7 5:m7 5:m7 0:m7 0:m7 5:m7 5:m7"), "i7 i7 iv7 iv7 (deux fois)"),
            Loop(3, bars("0:m7 5:m7 10:7 3:maj7 0:m7 5:m7 10:7 3:maj7"), "i7 iv7 bVII7 bIII (deux fois)"),
            Loop(2, bars("0:m7 10:7 8:maj7 7:7 0:m7 10:7 8:maj7 7:7"), "i7 bVII7 bVI V7 (deux fois)"),
            Loop(1, bars("0:m7 0:m7 0:m7 0:m7 5:m7 5:m7 7:7 7:7"), "i7 (quatre mesures) iv7 iv7 V7 V7"),
        ]),
        Part("B", 8, "Refrain (huit mesures) : trois boucles.", [
            Loop(3, bars("8:maj7 10:7 0:m7 0:m7 8:maj7 10:7 0:m7 7:7"), "bVI bVII7 i7 i7 bVI bVII7 i7 V7"),
            Loop(2, bars("3:maj7 8:maj7 10:7 0:m7 3:maj7 8:maj7 10:7 0:m7"), "bIII bVI bVII7 i7 (deux fois)"),
            Loop(2, bars("5:m7 5:m7 0:m7 0:m7 5:m7 5:m7 7:7 7:7"), "iv7 iv7 i7 i7 iv7 iv7 V7 V7"),
        ]),
    ],
    sequence=[0, 1], tours=4, bass_low=28, bass_high=39,
    bass_motifs=[
        BassMotif(1, 4, row(EIGHTHS, 2, [0, 12, 0, 12, 0, 12, 0, 12], [100, 88, 94, 88, 98, 88, 94, 88]), "octaves en croches"),
        BassMotif(1, 3, row([0, 2, 4, 6, 8, 10, 12, 14], 2, [0, 12, 0, 12, 0, 12, 7, 12], [100, 86, 92, 86, 98, 86, 92, 90]), "octaves en croches, quinte au temps 4"),
        BassMotif(1, 3, [N(0, 2, 0, 100), N(3, 1, 12, 88), N(4, 2, 0, 94), N(6, 2, 12, 88), N(8, 2, 0, 98), N(11, 1, 12, 88), N(12, 2, 0, 92), N(14, 2, 12, 90)], "octaves syncopées"),
        BassMotif(1, 2, [N(0, 6, 0, 100), N(6, 2, 12, 90), N(8, 6, 0, 96), N(14, 2, 12, 90)], "fondamentale longue, octave en levée"),
        BassMotif(2, 2, row(EIGHTHS, 2, [0, 12, 0, 12, 0, 12, 0, 12], 94) + row(list(range(16, 32, 2)), 2, [0, 12, 0, 12, 0, 7, 10, 12], [100, 86, 92, 86, 98, 90, 92, 94]), "deux mesures : octaves, puis remontée par la quinte et la septième"),
    ],
    chords_center=60, chords_drift=6, voicing="close_triad", note_count=3, chords_low=52, chords_high=76,
    spread=STRUM.format(down=3.0, up=3.0),
    default_comp=[(0, 2, 80), (6, 2, 76), (10, 2, 80)],
    comping=[
        Comp(3, "sparse", [(0, 16, 72)], "accord tenu"),
        Comp(3, "sparse", [(2, 2, 76, "up"), (10, 2, 72, "up")], "deux coups en contretemps"),
        Comp(1, "sparse", [], "silence"),
        Comp(4, "medium", [(2, 2, 78, "up"), (6, 2, 76, "up"), (10, 2, 78, "up"), (14, 2, 76, "up")], "contretemps en croches"),
        Comp(3, "medium", [(0, 2, 80), (3, 1, 66, "up"), (6, 2, 76, "up"), (10, 2, 78, "up"), (14, 2, 74, "up", "anticipate")], "syncopé"),
        Comp(3, "medium", [(0, 2, 80, "down"), (4, 2, 72, "up"), (8, 2, 78, "down"), (12, 2, 72, "up")], "bas-haut sur les temps"),
        Comp(4, "dense", [(0, 1, 82, "down")] + [(s, 1, 78 if s % 4 == 2 else 64, "up") for s in range(16) if s % 4 != 0], "chicken-scratch en doubles croches"),
        Comp(3, "dense", [(0, 2, 82), (2, 1, 66, "up"), (3, 1, 62, "up"), (6, 1, 70, "up"), (7, 1, 62, "up"), (8, 2, 80), (10, 1, 66, "up"), (11, 1, 62, "up"), (14, 1, 70, "up"), (15, 1, 62, "up")], "doubles croches avec trous"),
    ],
    comp_phrases=phrases([
        (3, "sparse", [0, 1, 0, 1], "any"), (2, "sparse", [1, 1], "any"), (2, "sparse", [0, 0, 0, 2], "any"), (2, "sparse", [1, 0], "part_end"),
        (3, "medium", [3, 3, 4, 3], "any"), (3, "medium", [3, 5, 3, 4], "any"), (2, "medium", [3, 4], "any"), (3, "medium", [3, 4], "part_end"),
        (3, "dense", [6, 6, 6, 7], "any"), (2, "dense", [6, 7, 6, 7], "any"), (2, "dense", [6, 7], "any"), (3, "dense", [6, 4], "part_end"),
    ]),
    chords_program=27, bass_program=33, kit_program=0,
)

PROFILES.append({
    "style": DISCO,
    "drum": DrumSpec(["dance/disco"], min_files=2, min_measures=6, max_share=0.9, max_groups=6, arc_tags=["hat"], fills=False),
    "drum_why": "cinq fichiers et deux batteurs : seuils réduits à deux fichiers et six mesures (aucun groupe ne tient les seuils de L2)",
    "source": "le style `dance/disco` du Groove MIDI Dataset v1.0.0 (5 fichiers `beat`, 431 mesures, 4/4 droit, « juste », L2 § 4.2 ; aucun fichier `fill`).",
    "key_name": "la éolien",
    "fills_none_weight": 0,
    "corpus": COMMON_CORPUS,
    "readme": {
        "genre": "disco",
        "nom_limite": "il décrit la batterie de CINQ fichiers joués par deux batteurs sur un kit électronique ; la basse en octaves et la guitare sont des règles de métier.",
        "intro": "Profil d'accompagnement **sous une grille d'accords** : basse en octaves, guitare claire en contretemps et batterie disco du GMD. Sans grille fournie : deux parties de huit mesures de boucles mineures à septièmes. Aucune mélodie.",
        "swing": "0,49 (médiane de 5 fichiers)",
        "kit": "kit standard",
        "basse": "basse au doigt (programme 33), cinq motifs d'une ou deux mesures en octaves (croches, croches avec quinte, syncopés, fondamentale longue, deux mesures) ; fenêtre 28..39",
        "comping": "guitare claire (programme 27), triades, contretemps et « chicken-scratch » en doubles croches (attaques vers le haut sur trois cordes) : huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures dont quatre `part_end`",
        "timing": "basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service)",
        "regles": "octaves de basse, guitare en contretemps",
        "variete": "Entre graines : une des quatre boucles de la partie A et trois de la partie B, le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie et leurs phrases. Entre chorus : l'arc. Pas de relance de batterie (voir limites).",
        "limites": "- **Fiabilité** : « juste » (5 fichiers) mais deux batteurs seulement ; un groupe domine (114 mesures sur 3 fichiers).\n- **Aucun fichier `fill`** dans le style : pas de relance de batterie ; la variation de fin de phrase vient des phrases, de l'arc (breaks, stop-time) et de la basse.\n- **Pas de cordes ni de cuivres** : un seul instrument d'accords par profil.",
    },
})

# --- afrobeat -------------------------------------------------------------------------------------

AFRO = Style(
    id="gmd_afrobeat", name="Accompagnement afrobeat (Groove MIDI Dataset, style afrobeat)",
    tradition="Groove MIDI Dataset v1.0.0, style afrobeat, 4/4 droit : 13 fichiers de batterie, 2 batteurs ; basse et guitare : règles de métier",
    tags=["afrobeat", "accompagnement", "grille"], aliases=["afrobeat"], title="accompagnement afrobeat sous une grille",
    key=2, mode="dorian", bpm=(0, 0, 0), bpm_note="",
    parts=[
        Part("A", 8, "Couplet (huit mesures) : boucles à un ou deux accords, longues (forme commune de l'afrobeat : vamp).", [
            Loop(3, bars("0:m7 0:m7 0:m7 0:m7 5:9 5:9 5:9 5:9"), "i7 (quatre mesures) IV9 (quatre mesures)"),
            Loop(3, bars("0:m7 0:m7 5:9 5:9 0:m7 0:m7 5:9 5:9"), "i7 i7 IV9 IV9 (deux fois)"),
            Loop(2, bars("0:m9 0:m9 0:m9 0:m9 0:m9 0:m9 0:m9 0:m9"), "i9 (huit mesures)"),
            Loop(1, bars("0:m7 10:9 0:m7 10:9 0:m7 10:9 5:9 7:m7"), "i7 bVII9 (alternés) IV9 v7"),
        ]),
        Part("B", 8, "Refrain (huit mesures) : trois boucles.", [
            Loop(3, bars("5:9 5:9 0:m7 0:m7 5:9 5:9 0:m7 0:m7"), "IV9 IV9 i7 i7 (deux fois)"),
            Loop(2, bars("10:9 10:9 5:9 5:9 0:m7 0:m7 0:m7 0:m7"), "bVII9 bVII9 IV9 IV9 i7 (quatre mesures)"),
            Loop(2, bars("0:m7 5:9 0:m7 5:9 0:m7 5:9 0:m7 5:9"), "i7 IV9 (alternés)"),
        ]),
    ],
    sequence=[0, 1], tours=4, bass_low=28, bass_high=39,
    bass_motifs=[
        BassMotif(2, 4, [N(0, 3, 0, 100), N(3, 2, 0, 88), N(6, 2, 7, 92), N(8, 3, 0, 96), N(11, 2, 10, 88), N(14, 2, 7, 90),
                         N(16, 3, 0, 100), N(19, 2, 0, 88), N(22, 2, 7, 92), N(24, 4, 10, 94), N(28, 2, 7, 88), N(30, 2, 0, 90)], "deux mesures : ostinato syncopé en doubles croches"),
        BassMotif(1, 3, [N(0, 3, 0, 100), N(3, 3, 0, 90), N(6, 2, 7, 90), N(8, 4, 0, 96), N(12, 2, 10, 88), N(14, 2, 7, 88)], "ostinato d'une mesure"),
        BassMotif(2, 3, [N(0, 6, 0, 100), N(6, 2, 0, 88), N(8, 4, 7, 94), N(12, 4, 0, 92), N(16, 6, 0, 100), N(22, 2, 10, 90), N(24, 4, 7, 94), N(28, 4, 0, 92)], "deux mesures : plus posé"),
        BassMotif(1, 2, row(EIGHTHS, 2, [0, 0, 7, 0, 10, 0, 7, 0], [100, 86, 92, 86, 94, 86, 92, 86]), "croches avec quinte et septième"),
        BassMotif(1, 1, [N(0, 16, 0, 96)], "note tenue"),
    ],
    chords_center=62, chords_drift=6, voicing="guide_tones", note_count=3, chords_low=52, chords_high=76,
    default_comp=[(0, 2, 80), (6, 2, 74), (10, 2, 78)],
    comping=[
        Comp(3, "sparse", [(0, 16, 70)], "accord tenu"),
        Comp(3, "sparse", [(0, 6, 74), (10, 4, 70)], "deux accords"),
        Comp(1, "sparse", [], "silence"),
        Comp(4, "medium", [(0, 2, 80), (3, 2, 70), (6, 2, 76), (10, 2, 78), (13, 2, 70)], "chicka syncopé"),
        Comp(3, "medium", [(2, 2, 74), (6, 2, 78), (10, 2, 74), (14, 2, 78)], "contretemps en croches"),
        Comp(3, "medium", [(0, 2, 80), (3, 2, 72), (8, 2, 78), (11, 2, 70), (14, 2, 76, "anticipate")], "temps 1 et 3, anticipation"),
        Comp(4, "dense", [(0, 2, 82), (2, 1, 62), (3, 1, 72), (6, 2, 76), (7, 1, 62), (10, 2, 78), (11, 1, 64), (13, 1, 72), (14, 2, 76)], "chicken-scratch en doubles croches"),
        Comp(3, "dense", [(s, 1, 76 if s in (0, 3, 6, 10, 13) else 62) for s in range(16) if s not in (1, 5, 9)], "doubles croches, trois trous"),
    ],
    comp_phrases=phrases([
        (3, "sparse", [0, 1, 0, 1], "any"), (2, "sparse", [1, 1], "any"), (2, "sparse", [0, 0, 0, 2], "any"), (2, "sparse", [1, 0], "part_end"),
        (3, "medium", [3, 3, 4, 3], "any"), (3, "medium", [3, 5, 3, 4], "any"), (2, "medium", [3, 4], "any"), (3, "medium", [3, 5], "part_end"),
        (3, "dense", [6, 6, 6, 7], "any"), (2, "dense", [7, 6, 7, 6], "any"), (2, "dense", [6, 7], "any"), (3, "dense", [6, 4], "part_end"),
    ]),
    chords_program=27, bass_program=33, kit_program=0,
)

PROFILES.append({
    "style": AFRO,
    "drum": DrumSpec(["afrobeat"], min_files=2, min_measures=10, max_share=0.9, max_groups=7, arc_tags=["hat", "ride"], fills=False),
    "drum_why": "deux batteurs : seuils réduits à deux fichiers et dix mesures",
    "source": "le style `afrobeat` du Groove MIDI Dataset v1.0.0 (13 fichiers `beat`, 982 mesures, 4/4 droit, « juste », L2 § 4.2 ; aucun fichier `fill`).",
    "key_name": "ré dorien",
    "fills_none_weight": 0,
    "corpus": COMMON_CORPUS,
    "readme": {
        "genre": "afrobeat",
        "nom_limite": "il décrit la batterie de 13 fichiers joués par deux batteurs sur un kit électronique, sans les percussions ni les cuivres du genre ; basse et guitare sont des règles de métier.",
        "intro": "Profil d'accompagnement **sous une grille d'accords** : basse en ostinato syncopé, guitare claire en « chicken-scratch » et batterie afrobeat du GMD. Sans grille fournie : deux parties de huit mesures de vamps à un ou deux accords. Aucune mélodie.",
        "swing": "0,50 (médiane de 13 fichiers)",
        "kit": "kit standard",
        "basse": "basse au doigt (programme 33), cinq motifs d'une ou deux mesures (ostinatos syncopés, version posée, croches avec quinte et septième, note tenue) ; fenêtre 28..39",
        "comping": "guitare claire (programme 27), voicings `guide_tones` (tierce et septième, plus une tension), huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense` (chicka syncopé, doubles croches), douze phrases de 2 à 4 mesures dont quatre `part_end`",
        "timing": "basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service)",
        "regles": "ostinato de basse, guitare en doubles croches",
        "variete": "Entre graines : une des quatre boucles de la partie A et trois de la partie B (de huit mesures d'un accord aux alternances), le plan de motifs de basse, les rythmes et phrases de comping, les groupes de batterie (charleston et ride) et leurs phrases. Entre chorus : l'arc. Pas de relance de batterie.",
        "limites": "- **Fiabilité** : « juste » (13 fichiers) mais deux batteurs seulement.\n- **Aucun fichier `fill`** dans le style : pas de relance de batterie.\n- **Pas de percussions, de cuivres ni de seconde guitare** : un seul instrument d'accords par profil, et le GMD n'a que des pads de kit (L2 § 3).",
    },
})


# --- registre ------------------------------------------------------------------------------------


PROFILES.append({
    "style": ROCK,
    "drum": DrumSpec(["rock"], min_files=3, min_measures=20, max_share=0.5, max_groups=8, arc_tags=["hat", "ride"]),
    "drum_why": "style le plus fourni du GMD : seuils de L2, part maximale d'un fichier resserrée",
    "source": "le style `rock` du Groove MIDI Dataset v1.0.0 (fiabilité « assez » en `beat`, L2 § 4.2 : 193 fichiers, 9 batteurs, 5 367 mesures, dont 154 fichiers 4/4 droits retenus ici).",
    "key_name": "mi mixolydien",
    "fills_none_weight": 2,
    "corpus": COMMON_CORPUS,
    "readme": {
        "genre": "rock",
        "nom_limite": "il décrit la batterie de ce style telle que les dix batteurs du GMD la jouent sur un kit électronique ; la guitare et la basse sont des règles de métier.",
        "intro": "Profil d'accompagnement **sous une grille d'accords** pour le client externe `harmonium_training` : basse (motifs), guitare saturée (rythmes de comping, phrases) et batterie rock du GMD ne dépendent que de l'accord publié et de la graine. Sans grille fournie, il joue deux parties de huit mesures de forme commune. Aucune mélodie.",
        "swing": "0,50 (médiane des 161 fichiers droits)",
        "kit": "kit « Room »",
        "basse": "basse au médiator (programme 34), six motifs d'une ou deux mesures tirés par poids (croches à la fondamentale, noires, octave en contretemps, syncope, deux mesures avec la quinte au temps 4, note tenue) et un gabarit de croches avec relance vers la fondamentale de l'accord suivant ; fenêtre des fondamentales 28..39 (mi grave)",
        "comping": "guitare saturée (programme 29), triade doublée à 4 sons conduite par les voix communes, grattée vers le bas (écart 7 ms) ; huit rythmes d'une mesure étiquetés `sparse`, `medium`, `dense`, douze phrases de 2 à 4 mesures dont quatre de mise en place `part_end`",
        "timing": "basse et accords sur la batterie (0 pas), jitter 0,1 pas (valeur de service : aucun corpus de placement rock)",
        "regles": "motifs de basse de rock, rythmes de guitare en noires, croches et syncopes",
        "variete": "Entre graines : une des quatre boucles de chaque partie (A et B), un des motifs de basse d'un plan de motifs par air, les rythmes et phrases de comping, les groupes de batterie et leurs phrases, les relances. Entre chorus d'une graine : l'arc (exposition, corps, sommet, retombée) change le comping, la batterie (charleston puis ride) et l'intensité ; les breaks et stop-time sont tirés par chorus.",
        "limites": "- **Accords de puissance** : le schéma n'a pas de qualité « 5 » (fondamentale et quinte) ; la guitare joue des triades majeures et mineures doublées à l'octave.\n- Le rock du GMD est joué majoritairement au ride et à la charleston fermée en croches ; les groupes de toms seuls (sans timekeeper) sont écartés.",
    },
})
