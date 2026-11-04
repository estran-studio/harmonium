#!/usr/bin/env python3
"""B3 afro-cubain : génère les fichiers du profil `gmd_afrocuban` (sauf `README.md`).

Reproduire (labo en lecture seule : seuls deux JSON d'agrégats du labo sont lus) :

    python3 specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_generer.py --out profiles/gmd_afrocuban \
        --mix specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.json

Ce qui est MESURÉ (agrégats du Groove MIDI Dataset v1.0.0, CC BY 4.0, `references/gmd-afrocuban/measures-drums.json`) :
  - kit : probabilité par pas, vélocité relative et micro-timing médian de chaque rôle du kit (grosse caisse,
    charleston pédale, ride, caisse claire, cross-stick, toms) dans `afrocuban` 4-4 droit (1 fichier, 98 mesures,
    1 batteur, « trop peu ») et `afrocuban/rhumba` 4-4 droit (1 fichier, 103 mesures, 1 batteur, « trop peu ») ;
  - relances : 18 fichiers `fill` de `afrocuban/rhumba` (1 batteur, « juste »), leur queue après le pas 12 ;
  - tempos : 105 (afrocuban droit), 110 (rhumba) ; 128 et 105 pour les deux fichiers ternaires du style.
Ce qui est de MÉTIER (idiomes traditionnels, aucune source ouverte : `l2-ingestion.md` § 5) : la clave son 3-2 / 2-3, le
montuno, le tumbao, les congas, la cloche, les bongos, les timbales, le shaker, les maracas, le güiro, les phrases
alignées sur la clave, l'arc, l'habillage, les grilles, le mixage. Le statut juridique d'un idiome rythmique relève
du propriétaire (README). Les mesures ternaires du style (178 mesures) ne sont pas rabattues sur 16 pas droits ;
`afrocuban/bembe` (ternaire) et `afrocuban/calypso` ne sont pas lus. Aucun MIDI ni aucune mesure individuelle copiés.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
sys.path.insert(0, str(ICI.parent / "b8"))
from emit import scalar  # noqa: E402

ID = "gmd_afrocuban"
COMMANDE = ("python3 specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_generer.py --out profiles/gmd_afrocuban "
            "--mix specs/019-bibliotheque-accompagnements/b3/gmd_afrocuban_mix.json")

# --- clave son, deux mesures (16 pas) -----------------------------------------------------------------------
CLAVE = {"three": [0, 6, 12], "two": [4, 8]}
AUTRE = {"three": "two", "two": "three"}

# --- tempos ----------------------------------------------------------------------------------------------
BPM_MIN, BPM_DEFAUT, BPM_MAX = 105.0, 105.0, 128.0

# --- niveaux par défaut du mixage (réglés par LUFS, voir README) ---------------------------------------------------
MIX = {"master_db": 25.0, "Chords": -17.0, "Bass": -18.5, "PercussionLow": -19.0, "PercussionHigh": -19.0}

TAGS = ("sparse", "medium", "dense")


def inline(**champs) -> str:
    return "{ " + ", ".join(f"{k} = {scalar(v)}" for k, v in champs.items()) + " }"


# --- voix : (note GM, vélocité de service, piste) ---------------------------------------------------------
KIT = {
    "kick": (36, 56, "PercussionLow"), "hat_pedal": (44, 40, "PercussionHigh"), "hat_closed": (42, 40, "PercussionHigh"),
    "ride": (51, 44, "PercussionHigh"), "ride_bell": (53, 50, "PercussionHigh"), "snare": (38, 40, "PercussionHigh"),
    "sidestick": (37, 54, "PercussionHigh"), "tom_high": (50, 46, "PercussionHigh"), "tom_low": (43, 50, "PercussionHigh"),
}
LATIN = {
    "conga_low": (64, 50, "PercussionLow"), "conga_open": (63, 50, "PercussionLow"), "conga_mute": (62, 44, "PercussionLow"),
    "timbale_low": (66, 48, "PercussionLow"), "claves": (75, 72, "PercussionHigh"), "cowbell": (56, 58, "PercussionHigh"),
    "bongo_hi": (60, 54, "PercussionHigh"), "bongo_lo": (61, 56, "PercussionHigh"), "timbale_hi": (65, 56, "PercussionHigh"),
    "shaker": (82, 38, "PercussionHigh"), "maracas": (70, 36, "PercussionHigh"), "guiro_long": (74, 42, "PercussionHigh"),
    "guiro_short": (73, 40, "PercussionHigh"),
}
# Seuils de probabilité par pas du kit, par densité et par source (réglages de service ; la probabilité mesurée
# est conservée comme probabilité du coup quand elle est < 0,9).
SEUILS = {
    "afrocuban": {
        0: {"hat_pedal": 0.6, "kick": 0.6},
        1: {"hat_pedal": 0.4, "kick": 0.4, "ride": 0.4, "snare": 0.4},
        2: {"hat_pedal": 0.4, "kick": 0.25, "ride": 0.3, "snare": 0.25, "sidestick": 0.25, "tom_high": 0.3, "hat_closed": 0.25},
    },
    "rhumba": {
        0: {"hat_pedal": 0.6, "kick": 0.55},
        1: {"hat_pedal": 0.55, "kick": 0.4, "sidestick": 0.35, "snare": 0.3},
        2: {"hat_pedal": 0.5, "kick": 0.25, "sidestick": 0.2, "snare": 0.2, "tom_high": 0.3, "ride_bell": 0.2},
    },
}


def delta(base: int, rel: float) -> int:
    return max(-20, min(20, round(base * (rel - 1.0))))


def kit_hits(groupe: dict, seuils: dict[str, float]) -> list[dict]:
    coups = []
    for role, seuil in seuils.items():
        base = KIT[role][1]
        for pas, _n, proba, _vel, rel, _off in groupe["per_step"].get(role, []):
            if proba >= seuil:
                coup = dict(step=pas, voice=role)
                if proba < 0.9:
                    coup["probability"] = round(proba, 2)
                d = delta(base, rel)
                if d:
                    coup["delta_velocity"] = d
                coups.append(coup)
    return coups


def latin_hits(tag: str, cote: str) -> list[dict]:
    """Section rythmique latine d'une mesure : règles de métier, pas des mesures."""
    coups = [dict(step=s, voice="claves") for s in CLAVE[cote]]

    def ajouter(nom, pas, **options):
        coups.extend(dict(step=s, voice=nom, **options) for s in pas)

    # Tumbao de conga : sourde sur 2 et 4, ouverte sur les « et », grave sur 1 et 3.
    ajouter("conga_low", [0, 8])
    ajouter("conga_open", [6, 14])
    if tag != "sparse":
        ajouter("conga_mute", [4, 12])
        ajouter("shaker", list(range(0, 16, 2)), delta_velocity=-6)
        # Cloche : cloche de ride dans le corps, cowbell au sommet (même cellule, autre voix).
        cloche = [0, 4, 6, 8, 12] if cote == "three" else [0, 4, 8, 12, 14]
        if tag == "medium":
            ajouter("ride_bell", cloche)
        else:
            ajouter("cowbell", [0, 4, 6, 8, 10, 12] if cote == "three" else [0, 4, 8, 10, 12, 14])
    if tag == "dense":
        ajouter("bongo_lo", [0, 8])
        ajouter("bongo_hi", [2, 6, 10, 14], delta_velocity=-8)
        ajouter("maracas", [4, 12], delta_velocity=-6)
        ajouter("guiro_long", [0, 8])
        ajouter("guiro_short", [6, 14])
        ajouter("timbale_hi", [10] if cote == "three" else [2], probability=0.35)
        ajouter("timbale_low", [15], probability=0.3)
    return coups


# --- comping : montuno, deux motifs par côté et par étiquette ---------------------------------------------------
def hit(step, duration, velocity, **o):
    return inline(step=step, duration=duration, velocity=velocity, **o)


MONTUNO = {
    # (étiquette, côté, variante) -> attaques
    ("sparse", "three", 0): [hit(0, 3, 68), hit(6, 2, 62), hit(12, 2, 66)],
    ("sparse", "three", 1): [hit(0, 2, 66), hit(8, 3, 62), hit(14, 2, 66, anticipate=True)],
    ("sparse", "two", 0): [hit(4, 2, 66), hit(8, 2, 64), hit(14, 2, 66, anticipate=True)],
    ("sparse", "two", 1): [hit(2, 2, 60), hit(6, 2, 64), hit(12, 3, 66)],
    ("medium", "three", 0): [hit(0, 2, 74), hit(6, 2, 68), hit(10, 2, 58), hit(12, 2, 72)],
    ("medium", "three", 1): [hit(2, 2, 66), hit(6, 2, 70), hit(8, 2, 60), hit(12, 2, 72)],
    ("medium", "two", 0): [hit(2, 2, 58), hit(4, 2, 72), hit(8, 2, 70), hit(14, 2, 72, anticipate=True)],
    ("medium", "two", 1): [hit(0, 2, 66), hit(4, 2, 72), hit(7, 1, 58), hit(10, 2, 66), hit(14, 2, 72, anticipate=True)],
    ("dense", "three", 0): [hit(0, 2, 80), hit(2, 1, 58, probability=0.6), hit(6, 2, 74), hit(8, 1, 58, probability=0.5),
                            hit(10, 2, 68), hit(12, 2, 78)],
    ("dense", "three", 1): [hit(0, 2, 78), hit(3, 1, 58, probability=0.6), hit(6, 2, 74), hit(8, 2, 66), hit(11, 1, 56, probability=0.5),
                            hit(12, 2, 78), hit(14, 2, 70, anticipate=True)],
    ("dense", "two", 0): [hit(2, 2, 66), hit(4, 2, 78), hit(6, 1, 58, probability=0.6), hit(8, 2, 76), hit(10, 1, 58, probability=0.5),
                          hit(14, 2, 78, anticipate=True)],
    ("dense", "two", 1): [hit(0, 1, 60, probability=0.6), hit(2, 2, 68), hit(4, 2, 78), hit(7, 1, 58, probability=0.5), hit(8, 2, 74),
                          hit(12, 1, 62, probability=0.5), hit(14, 2, 78, anticipate=True)],
}


def comping_banque():
    """Indices des motifs : 4 par étiquette, dans l'ordre (three 0, three 1, two 0, two 1)."""
    lignes, index = [], {}
    for tag in TAGS:
        for cote in ("three", "two"):
            for variante in (0, 1):
                index[tag, cote, variante] = len(lignes)
                lignes.append(f'  {{ tag = "{tag}", hits = [' + ", ".join(MONTUNO[tag, cote, variante]) + "] },")
    return lignes, index


def comping_phrases(index) -> list[str]:
    out = []
    for tag in TAGS:
        for cote in ("three", "two"):
            o = AUTRE[cote]
            i = lambda c, v: index[tag, c, v]  # noqa: E731
            suites = [
                ("any", [i(cote, 0), i(o, 0)]), ("any", [i(cote, 1), i(o, 1)]),
                ("any", [i(cote, 0), i(o, 1)]), ("any", [i(cote, 1), i(o, 0)]),
                ("any", [i(cote, 0), i(o, 0), i(cote, 1), i(o, 1)]),
                ("part_end", [i(cote, 0), i(o, 1)]), ("part_end", [i(cote, 1), i(o, 0)]),
            ]
            for place, motifs in suites:
                out.append('\n[[accompaniment.chords.phrases]]\n'
                           f'tag = "{tag}"\nweight = {2 if len(motifs) == 4 else 1}\nat = "{place}"\nclave = "{cote}"\nmotifs = {motifs}\n')
    return out


GRILLES = '''[harmony]
model = "loop"
steps_per_chord = 4

[harmony.parts.Grille]
kind = "chord_loop"
loops = [
  # Vamp à accord unique, seize mesures de I7 (poids 2).
  { weight = 2, chords = [{ semitones = 0, quality = "7", steps = 256 }] },
  # I, IV, V7, I : quatre mesures chacun (poids 3).
  { weight = 3, chords = [{ semitones = 0, quality = "", steps = 64 }, { semitones = 5, quality = "", steps = 64 }, { semitones = 7, quality = "7", steps = 64 }, { semitones = 0, quality = "", steps = 64 }] },
  # Im7 et IV7 : deux mesures chacun, quatre fois (poids 3).
  { weight = 3, chords = [{ semitones = 0, quality = "m7", steps = 32 }, { semitones = 5, quality = "7", steps = 32 }, { semitones = 0, quality = "m7", steps = 32 }, { semitones = 5, quality = "7", steps = 32 }, { semitones = 0, quality = "m7", steps = 32 }, { semitones = 5, quality = "7", steps = 32 }, { semitones = 0, quality = "m7", steps = 32 }, { semitones = 5, quality = "7", steps = 32 }] },
  # ii–V–I : iim7 deux mesures, V7 deux mesures, Imaj7 quatre mesures, deux fois (poids 2).
  { weight = 2, chords = [{ semitones = 2, quality = "m7", steps = 32 }, { semitones = 7, quality = "7", steps = 32 }, { semitones = 0, quality = "maj7", steps = 64 }, { semitones = 2, quality = "m7", steps = 32 }, { semitones = 7, quality = "7", steps = 32 }, { semitones = 0, quality = "maj7", steps = 64 }] },
  # Im7, bVII7, bVImaj7, V7 : deux mesures chacun, deux fois (poids 2).
  { weight = 2, chords = [{ semitones = 0, quality = "m7", steps = 32 }, { semitones = 10, quality = "7", steps = 32 }, { semitones = 8, quality = "maj7", steps = 32 }, { semitones = 7, quality = "7", steps = 32 }, { semitones = 0, quality = "m7", steps = 32 }, { semitones = 10, quality = "7", steps = 32 }, { semitones = 8, quality = "maj7", steps = 32 }, { semitones = 7, quality = "7", steps = 32 }] },
]
'''

# --- basse : tumbao (métier) ----------------------------------------------------------------------------------
BASSE = '''# Registre : WJazzD LATIN, hauteur de basse p1 31, p50 40, p99 50, étendue 28..53 (1 639 mesures, 23 solos).
# Fondamentale et quinte (repliée dans la fenêtre) entre 35 et 46 : médiane 40,5, symétrique autour de la médiane
# mesurée (p1 et p99 de WJazzD sont 9 et 10 demi-tons de part et d'autre). La fenêtre de douze demi-tons ne
# peut pas atteindre l'étendue mesurée de dix-neuf ; pas d'octave : 46 + 12 dépasserait le maximum mesuré (53).
[accompaniment.bass]
low = 35
high = 46
walk_low = 28
walk_high = 53

# Tumbao : idiome traditionnel, aucune source ouverte (WJazzD ne note qu'une hauteur par temps : les croches ne
# sont pas observables). Attaques aux pas 6 et 12 et anticipation du changement d'accord au pas 14 (`pickup` à
# chaque barre : la basse de l'accord de la mesure suivante, jouée une croche avant la barre). Le un est joué (le
# tumbao strict le tait) : la basse arrive ainsi sur la basse publiée à chaque changement, y compris sous un
# stop-time qui ne garde que le pas 0.
[[accompaniment.bass.templates]]
weight = 3
rhythm = [{ step = 0, duration = 3, velocity = 88 }, { step = 6, duration = 4, velocity = 92 }, { step = 12, duration = 2, velocity = 96 }]
cycle = [{ degree = "root" }, { degree = "fifth" }, { degree = "root" }]
pickup = { lead = 2, duration = 2, velocity = 94, when = "bar_end" }
[[accompaniment.bass.templates]]
weight = 2
rhythm = [{ step = 0, duration = 3, velocity = 86 }, { step = 6, duration = 4, velocity = 90 }, { step = 12, duration = 2, velocity = 96 }]
cycle = [{ degree = "root" }, { degree = "root" }, { degree = "fifth" }]
pickup = { lead = 2, duration = 2, velocity = 92, when = "bar_end" }
[[accompaniment.bass.templates]]
weight = 2
rhythm = [{ step = 0, duration = 2, velocity = 86 }, { step = 6, duration = 2, velocity = 94 }, { step = 8, duration = 2, velocity = 70 }, { step = 12, duration = 2, velocity = 96 }]
cycle = [{ degree = "root" }, { degree = "fifth" }, { degree = "root" }, { degree = "root" }]
pickup = { lead = 2, duration = 2, velocity = 94, when = "bar_end" }
# Marche posée : le temps 4 est libre, sans anticipation (« bolero » de la basse) ; c'est le seul gabarit où une
# note de passage peut approcher le changement.
[[accompaniment.bass.templates]]
weight = 2
rhythm = [{ step = 0, duration = 5, velocity = 90 }, { step = 6, duration = 2, velocity = 88 }, { step = 8, duration = 3, velocity = 92 }]
cycle = [{ degree = "root" }, { degree = "fifth" }, { degree = "root" }]

# Approche d'un changement d'accord à la barre : WJazzD LATIN, 1 205 événements, 22 solos : aucune 60 % [51–66],
# chromatique 22 % [17–26], diatonique 19 % [11–29]. Poids 60 / 22 / 19 (somme 101, normalisée par le moteur).
[accompaniment.bass.passing]
none_weight = 60
density = [[0.0, 0.5], [0.5, 1.0], [1.0, 1.3]]
phrase = [[0.0, 1.0], [1.0, 1.0]]
avoid_melody_intervals = []
rules = [
  { when = "next_target_differs", before = [4], motion = "chromatic", weight = 22, duration = 2, velocity_delta = -20, min_leap = 0 },
  { when = "next_target_differs", before = [4], motion = "scale_step", weight = 19, duration = 2, velocity_delta = -20, min_leap = 0 },
]
'''


def micro_timing(groupe: dict, role: str):
    """Médiane du décalage (pas) d'un rôle : `on_beat` et `off_beat` ; hors ±0,2 (borne de l'A6), neutre."""
    champs = {}
    for cle, mesures in groupe["micro_timing"].get(role, {}).items():
        m = mesures["steps"]["p50"]
        if abs(m) <= 0.2:
            champs[cle] = round(m, 3)
    return champs


def construire(labo: Path, sortie: Path, mix: dict):
    source = json.loads((labo / "references/gmd-afrocuban/measures-drums.json").read_text())
    droit = source["styles"]["afrocuban"]["groups"]["4-4:straight"]["beat"]
    rhumba = source["styles"]["afrocuban/rhumba"]["groups"]["4-4:straight"]
    rhumba_beat, rhumba_fill = rhumba["beat"], rhumba["fill"]
    sortie.mkdir(parents=True, exist_ok=True)

    def sauver(nom, texte):
        (sortie / nom).write_text(texte.rstrip() + "\n")

    lignes, index = comping_banque()
    lignes_comping = "\n".join(lignes)

    profil = f'''# Profil `{ID}` (phase 019, B3) : afro-cubain en 16 pas droits, clave son 3-2 / 2-3, montuno, tumbao et section
# rythmique latine sous une grille d'accords.
#
# Le nom est celui de la source qui définit le style (principe VII) : le style `afrocuban` du Groove MIDI Dataset
# v1.0.0 (3 fichiers `beat`, 2 batteurs, 276 mesures, « juste » au niveau du style ; groupe droit : 1 fichier,
# 98 mesures, 1 batteur, « trop peu »). Le GMD ne contient AUCUNE percussion latine (kit électronique Roland
# TD-11) : seul le kit (grosse caisse, charleston pédale, ride, caisse claire, cross-stick, toms) est mesuré, par
# probabilités par pas. La clave, le montuno, le tumbao, les congas, la cloche, les bongos, les timbales, le shaker,
# les maracas et le güiro sont des IDIOMES TRADITIONNELS écrits en règles de métier (aucune source ouverte :
# harmonium_lab `references/l2-ingestion.md` § 5) ; leur statut juridique relève du propriétaire (README).
# La basse est calibrée sur WJazzD `rhythmfeel = LATIN` pour ce qui y est mesurable (registre, approche avant un
# changement d'accord) : c'est du hard bop sur rythme latin, PAS du salsa, et les croches du tumbao ne sont pas
# observables.
#
# Fait pour training : l'utilisateur fournit sa grille, le profil joue dessous ; sans grille, cinq boucles de seize
# mesures tirées par la graine, relatives à la tonique (D4 : aucune réharmonisation). Aucune mélodie. Tout sonne avec
# GeneralUser GS (piano, guitare nylon, piano électrique ; basse acoustique, aux doigts, fretless ; kits 0, 8, 16, 32) :
# aucun preset Odin. `session.toml` : habillage par graine.
[files]
cells = "cells.toml"
percussion = "percussion.toml"
arrangement = "arrangement.toml"
dynamics = "dynamics.toml"
set = "set.toml"
arc = "arc.toml"
session = "session.toml"

[meta]
id = "{ID}"
name = "Afro-cubain droit : clave son et montuno (GMD afrocuban)"
tradition = "Groove MIDI Dataset v1.0.0, style afrocuban droit (1 fichier, 98 mesures, 1 batteur) et afrocuban/rhumba (1 fichier droit, 103 mesures ; 18 fills) : kit électronique Roland TD-11 ; basse : Weimar Jazz Database v2.1, rhythmfeel = LATIN (23 solos, jazz sur rythme latin). Clave, montuno, tumbao et percussions latines : idiomes traditionnels écrits en règles de métier."
tags = ["afrocubain", "clave", "montuno", "accompagnement", "grille"]
corpus = """\\
Gillick, Roberts, Engel, Eck, Bamman, Learning to Groove with Inverse Sequence Transformations, ICML 2019 ; Groove MIDI Dataset v1.0.0, Google LLC, CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/). Probabilités par pas, vélocités relatives, micro-timing et queues de relance du kit, agrégés, jamais un enregistrement ; le GMD ne contient aucune percussion latine. \\
Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is made available here under the Open Database License (ODbL); individual contents under the Database Contents License (DbCL 1.0). Registre et approches avant un changement d'accord seulement (rhythmfeel = LATIN : 23 solos, 1639 mesures, hard bop sur rythme latin, pas du salsa) ; aucune note de solo copiée, aucun tumbao mesuré. \\
Clave son, montuno, tumbao, congas, cloche, bongos, timbales, shaker, maracas, güiro, grilles par défaut et mixage : règles de métier (idiomes traditionnels, aucune source ouverte) ; leur statut juridique relève du propriétaire. Mesures et manifestes : harmonium_lab references/gmd-afrocuban/, references/wjazzd-latin/, references/l2-ingestion.md."""

[meter]
time_signature = {{ numerator = 4, denominator = 4 }}
steps_per_bar = 16
steps_per_unit = 2
steps_per_pulse = 4
pulses_per_bar = 4
# Feel droit en 16 pas : le fichier droit du GMD afrocuban est en doubles croches égales (rapport de swing 0,5). Les
# deux fichiers ternaires du style (178 mesures) ne sont pas rabattus sur cette grille.
default_swing_ratio = 0.5
# Tempos mesurés dans la source (GMD, médiane de chaque fichier) : afrocuban droit 105, rhumba 110 ; les deux fichiers
# ternaires du style 105 et 128. Bornes = ces extrêmes, pas élargies (WJazzD LATIN a un autre ordre de grandeur :
# médiane 152,7 ; sa classe 120-180 est « juste », mais c'est du jazz).
default_bpm = {BPM_DEFAUT}
min_bpm = {BPM_MIN}
max_bpm = {BPM_MAX}

# Clave son de deux mesures, sens tiré une fois par air : 3-2 (mesure « trois » puis « deux ») ou 2-3.
[clave]
direction = "either"

# Une partie de seize mesures : un chorus. Pas de `[fixed_tonality]` : la tonalité se tire (session et key_modes).
[form]
sequence = [0]
[[form.parts]]
name = "Grille"
melody_center_offset = 0
phrase_ends = [4, 8, 12, 16]
roles = ["Open", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free", "Free"]

# Pas de mélodie jouée : mêmes réglages neutres que `gmd_funk`. Les tonalités tirables sont celles de `session.toml`.
[melody]
cell_units = 8
strong_units = [0, 2, 4, 6]
weak_units = [1, 3, 5, 7]
key_modes = [{{ key = 0, mode = "ionian", weight = 1 }}, {{ key = 2, mode = "ionian", weight = 1 }}, {{ key = 5, mode = "ionian", weight = 1 }}, {{ key = 7, mode = "ionian", weight = 1 }}, {{ key = 9, mode = "ionian", weight = 1 }}, {{ key = 10, mode = "ionian", weight = 1 }}]
[melody.params]
weight_role_run = 3
weight_role_arp = 0
weight_role_turn = 0
max_continuity_step_distance = 2
continuity_max_retries = 8
melody_center_midi = 72
tonic_midi_min = 60
tonic_midi_max = 72
tessitura_min = 55
tessitura_max = 88
base_velocity = 80
strong_eighth_velocity_offset = 8
even_eighth_velocity_offset = -6

# Grilles relatives à la tonique : formes communes écrites pour ce profil, aucune œuvre nommée. Une boucle fait
# toujours seize mesures. Le mode tiré par la session n'est qu'une étiquette : les accords ne dépendent pas de lui.
{GRILLES}
[accompaniment]
strong_steps = [0, 8]
strong_weight = 2
weak_weight = 1
tie_priority = ["I", "V", "IV"]

{BASSE}
# Montuno (comping) : guajeo syncopé d'une mesure, un motif par côté de clave (« trois » : attaques autour des pas 0,
# 6, 12 ; « deux » : autour des pas 4, 8), deux variantes par côté et par étiquette `sparse` / `medium` / `dense`
# (l'arc nomme celles d'un chorus). Les attaques du pas 14 sont anticipées (`anticipate`) : elles jouent l'accord de la
# mesure suivante. Règles de métier, aucune source ouverte ; les notes fantômes sont des coups probabilistes.
[accompaniment.chords]
center = 65
max_drift = 6
default_pattern = 0
voicing = "drop2"
low = 48
high = 84
color = {{ none_weight = 1, published_weight = 2 }}
register_drift = {{ step = 1, max = 3 }}
patterns = [{{ hits = [{{ step = 0, duration = 2, velocity = 68 }}, {{ step = 6, duration = 2, velocity = 64 }}, {{ step = 12, duration = 2, velocity = 68 }}] }}]
comping = [
{lignes_comping}
]
'''
    for ligne in comping_phrases(index):
        profil += ligne
    profil += '''
[instruments]
melody_program = 0
chords_program = 0
bass_program = 32
percussion_bank = 128
percussion_program = 0

[ornaments]
rate_scale = 0.0

# Micro-timing du kit : médiane du décalage à la grille de chaque rôle dans le GMD afrocuban droit (pas ; sur le temps
# et hors du temps), neutre au-delà de ±0,2 (borne de l'A6). Les voix latines ne sont pas mesurées : aucun décalage.
'''
    for piste, roles in (("PercussionLow", ["kick"]), ("PercussionHigh", ["hat_pedal", "ride", "snare", "tom_high", "sidestick"])):
        voix = []
        for role in roles:
            champs = micro_timing(droit, role) or micro_timing(rhumba_beat, role)
            if champs:
                voix.append(inline(note=KIT[role][0], **champs))
        profil += f'[performance.tracks.{piste}]\nvoices = [' + ', '.join(voix) + ']\n'
    profil += '''[performance.tracks.Bass]
jitter = 0.04
[performance.tracks.Chords]
jitter = 0.05

# Mixage de service réglé sur le LUFS en solo de chaque piste (README), aucune mesure de corpus.
[mix]
master_db = %(master_db)s
[mix.room]
room_size = 0.25
damping = 0.6
width = 0.5
level = 0.1
[mix.tracks.Melody]
level_db = 0.0
pan = 0.0
reverb_send = 0.0
[mix.tracks.Chords]
level_db = %(Chords)s
pan = -0.2
reverb_send = 0.1
[mix.tracks.Bass]
level_db = %(Bass)s
pan = 0.0
reverb_send = 0.03
[mix.tracks.PercussionLow]
level_db = %(PercussionLow)s
pan = 0.0
reverb_send = 0.06
[mix.tracks.PercussionHigh]
level_db = %(PercussionHigh)s
pan = 0.1
reverb_send = 0.06
[mix.eq]
highpass_hz = 35.0
presence_hz = 3000.0
presence_db = 1.0
presence_q = 0.8
''' % {k: scalar(float(v)) for k, v in mix.items()}
    sauver("profile.toml", profil)

    # --- percussion.toml -------------------------------------------------------------------------------
    percu = f'''# GÉNÉRÉ : {COMMANDE}
# GMD CC BY 4.0 : Gillick, Roberts, Engel, Eck, Bamman, ICML 2019 ; Groove MIDI Dataset v1.0.0, Google LLC.
# Kit (MESURÉ, agrégats) : probabilité par pas, vélocité relative et micro-timing des rôles du kit dans `afrocuban`
# 4-4 droit (variante « afrocuban » : 1 fichier, 98 mesures, 1 batteur) et `afrocuban/rhumba` 4-4 droit (variante
# « rhumba » : 1 fichier, 103 mesures, 1 batteur), « trop peu » chacune : un idiome de batteur, pas du style.
# Section latine (MÉTIER) : clave son (voix `claves`, pas 0, 6, 12 | 4, 8), congas, cloche (cloche de ride dans le
# corps, cowbell au sommet), bongos, timbales, shaker, maracas, güiro : idiomes traditionnels, aucune source ouverte.
# Les deux pistes de percussion : congas graves, timbale grave et grosse caisse sur `PercussionLow`, le reste sur
# `PercussionHigh`. Relances : queue (pas 13 à 15) des 18 fichiers `fill` de `afrocuban/rhumba` ; une relance ne touche
# jamais la clave (dernier coup de clave au pas 12) : les fills n'ont pas de côté de clave.
low_note = 36
high_note = 75
low_velocity = 64
high_velocity = 64
percussion_gain = 1.0
velocity_humanize = 2
fills_none_weight = 18

'''
    for nom, (note, vel, piste) in {**KIT, **LATIN}.items():
        percu += f'[[voices]]\nname = "{nom}"\nnote = {note}\nvelocity = {vel}\ntrack = "{piste}"\n\n'
    sources = {"afrocuban": droit, "rhumba": rhumba_beat}
    indices: dict = {}
    nb = 0
    for densite in range(3):
        for tag in TAGS:
            for variante, groupe in sources.items():
                kit = kit_hits(groupe, SEUILS[variante][densite])
                for cote in ("three", "two"):
                    coups = latin_hits(tag, cote) + kit
                    coups.sort(key=lambda c: (c["step"], c["voice"]))
                    indices[densite, tag, variante, cote] = nb
                    percu += (f'# {nb} : densité {densite}, {tag}, kit « {variante} », mesure « {cote} ».\n'
                              f'[[patterns]]\ndensity = {densite}\ntag = "{tag}"\nweight = 1\nhits = [\n')
                    percu += ''.join('  ' + inline(**c) + ',\n' for c in coups) + ']\n\n'
                    nb += 1
    phrases = 0
    for densite in range(3):
        for tag in TAGS:
            for variante in sources:
                for cote in ("three", "two"):
                    ids = [indices[densite, tag, variante, cote], indices[densite, tag, variante, AUTRE[cote]]]
                    for place in ("any", "part_end"):
                        percu += (f'[[phrases]]\ndensity = {densite}\ntag = "{tag}"\nweight = 1\nat = "{place}"\n'
                                  f'clave = "{cote}"\npatterns = {ids}\n\n')
                        phrases += 1
    queue = []
    for role in ("kick", "snare", "tom_high", "tom_low"):
        base = KIT[role][1]
        for pas, _n, proba, _v, rel, _o in rhumba_fill["per_step"].get(role, []):
            if pas >= 13 and proba >= 0.15:
                coup = dict(step=pas, voice=role, probability=round(proba, 2))
                d = delta(base, rel)
                if d:
                    coup["delta_velocity"] = d
                queue.append(coup)
    queue.sort(key=lambda c: (c["step"], c["voice"]))
    relances = 0
    for densite in (1, 2):
        for tag in ("medium", "dense"):
            percu += (f'[[fills]]\ndensity = {densite}\ntag = "{tag}"\nweight = 18\nat = "part_end"\nstart_step = 13\n'
                      'bars = [{ hits = [' + ', '.join(inline(**c) for c in queue) + '] }]\n\n')
            relances += 1
    sauver("percussion.toml", percu)

    # --- arc.toml et arcs nommés -----------------------------------------------------------------------
    def stages(orchestre: bool) -> str:
        expo = ', instruments = { percussion_program = 40 }' if orchestre else ''
        somm = ', instruments = { chords_program = 61 }' if orchestre else ''
        fall = 'instruments = { percussion_program = 40 }\n' if orchestre else ''
        return f'''repeat_from = 1

[[stages]]
choruses = 1
variants = [
  {{ weight = 3, intensity = 0.65, density = 0.65{expo}, comping = [{{ tag = "sparse" }}], percussion = [{{ tag = "sparse" }}] }},
  {{ weight = 1, intensity = 0.75, density = 0.75{expo}, comping = [{{ tag = "sparse", weight = 1 }}, {{ tag = "medium", weight = 1 }}], percussion = [{{ tag = "sparse" }}] }},
]

[[stages]]
choruses = 2
variants = [
  {{ weight = 2, comping = [{{ tag = "medium" }}], percussion = [{{ tag = "medium" }}], breaks = [{{ start = 9, bars = 1, chance = 0.4, tracks = ["Chords"] }}] }},
  {{ weight = 1, intensity = 1.1, density = 1.1, comping = [{{ tag = "medium", weight = 1 }}, {{ tag = "dense", weight = 1 }}], percussion = [{{ tag = "medium" }}], breaks = [{{ start = -1, bars = 1, chance = 0.5, tracks = ["Chords", "Bass"] }}] }},
]

[[stages]]
choruses = 1
variants = [
  {{ intensity = 1.3, density = 1.25{somm}, comping = [{{ tag = "dense" }}], percussion = [{{ tag = "dense" }}], breaks = [{{ start = -2, bars = 2, chance = 0.5, tracks = ["Chords", "Bass"], keep_steps = [0] }}] }},
]

[[fall]]
{fall}intensity = 0.6
density = 0.6
comping = [{{ tag = "sparse" }}]
percussion = [{{ tag = "sparse" }}]
breaks = [{{ start = -2, bars = 2, tracks = ["Chords", "Bass"], keep_steps = [0] }}]
'''
    sauver("arc.toml", '''# Arc de `gmd_afrocuban` sur plusieurs chorus (A3) : règles de métier, aucune mesure de corpus (le GMD isole beat et
# fill, WJazzD ne donne pas la dynamique d'un chorus). Un chorus = une passe de la grille de seize mesures ; cinq
# chorus par air, nombre connu :
#   1      exposition : montuno et section latine clairsemés (clave, congas), doux ;
#   2-3    corps : montuno médian (parfois dense), cloche de ride, shaker ; un break du comping à mi-chorus ou des
#          accords et de la basse avant le chorus suivant ;
#   4      sommet : montuno dense, cowbell, bongos, güiro, timbales, plus fort, stop-time sur les deux dernières mesures
#          une fois sur deux ;
#   5      retombée : clairsemé, doux, stop-time sur le un des deux dernières mesures ;
# au-delà : le sommet puis le corps et le sommet en boucle (`repeat_from`).
''' + stages(False))

    # --- dynamics, arrangement, set, cells ----------------------------------------------------------------
    sauver('dynamics.toml', '''# Courbes de service ; aucun lien émotion/style mesuré. Le tempo ne bouge pas avec l'arousal : il est celui de la session
# (105 à 128, plage mesurée) ; les vélocités par piste sont élargies (0,6 à 1,3) pour que l'arc s'entende.
# Aucune bascule de mode : le mode tiré n'est qu'une étiquette (les accords s'écrivent relatifs à la tonique).
neutral_valence = 0.3
valence_range = [-1.0, 1.0]
mode_shift_boundary = "part"
dark_below = 0.1
bright_above = 0.9
parallel_modes = [{ bright = "ionian", dark = "aeolian", weight = 1 }]
arousal_intensity = [[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]]
tension_intensity = [[0.0, 0.0], [1.0, 0.0]]
tempo_offset = [[0.0, 0.0], [1.0, 0.0]]
melody_velocity = [[0.0, 1.0], [1.0, 1.0]]
chords_velocity = [[0.0, 0.65], [0.5, 1.0], [1.0, 1.3]]
bass_velocity = [[0.0, 0.75], [0.5, 1.0], [1.0, 1.2]]
percussion_velocity = [[0.0, 0.6], [0.5, 1.0], [1.0, 1.3]]
percussion_intensity = [[0.0, -1.0], [0.5, 0.0], [1.0, 1.0]]
melody_duration = [[0.0, 1.0], [1.0, 1.0]]
strong_accent = [[0.0, 0.0], [1.0, 0.0]]
chords_intensity = [[0.0, 0.0], [1.0, 0.0]]
run_weight = [[0.0, 1.0], [1.0, 1.0]]
arp_weight = [[0.0, 1.0], [1.0, 1.0]]
turn_weight = [[0.0, 1.0], [1.0, 1.0]]
color_register = [[0.0, 0.0], [1.0, 0.0]]
brightness_accent = [[0.0, 0.0], [1.0, 0.0]]
density_percussion = [[0.0, 0.0], [1.0, 0.0]]
density_chords = [[0.0, 0.0], [1.0, 0.0]]
density_tracks = []
tension_offbeat = [[0.0, 0.0], [1.0, 0.0]]
tension_duration = [[0.0, 1.0], [1.0, 1.0]]
tension_dominant = [[0.0, 0.0], [1.0, 0.0]]
tension_chord_duration = [[0.0, 1.0], [1.0, 1.0]]
color = [{ key = 0, mode = "ionian", weight = [[0.0, 1.0], [1.0, 1.0]] }]''')
    sauver('arrangement.toml', '''# Cinq chorus par air (une passe de la grille de seize mesures chacun) : exposition, deux chorus de corps, sommet,
# retombée. Tout joue dès la première mesure ; l'arc (`arc.toml`) module intensité, densité, comping et batterie.
# Pas de piste Melody : absente des entrées.
tours_per_tune = 5

[entries]
Chords = { tour = 1, part = 0, pass = 1 }
Bass = { tour = 1, part = 0, pass = 1 }
PercussionLow = { tour = 1, part = 0, pass = 1 }
PercussionHigh = { tour = 1, part = 0, pass = 1 }

[intro]
forms = []
pickup_eighths = 0''')
    modele = (ICI.parents[2] / 'profiles/gmd_funk/set.toml').read_text()
    sauver('set.toml', '# Un air de cinq chorus par set, raccord direct, ni outro ni rampe (le nombre de chorus est connu : l\'arc joue la\n# retombée au dernier).\n' + modele[modele.index('length ='):])
    sauver('cells.toml', '''# Banque minimale exigée par le chargeur : le profil ne joue pas de mélodie (Melody absente de l'arrangement), ces
# cellules ne sont jamais entendues.
cells = [
 { role = "open", notes = "1 3 5 3 1 3 5 3", weight = 1 },
 { role = "open", notes = "5 3 1 3 5 3 1 3", weight = 1 },
 { role = "run", notes = "1 2 3 4 5 6 7 8", weight = 1 },
 { role = "run", notes = "8 7 6 5 4 3 2 1", weight = 1 },
]''')

    # --- session.toml ------------------------------------------------------------------------------------
    session = '''# Habillage par graine (A2) : un tirage par groupe, les groupes se combinent ; deux groupes ne règlent jamais le même
# champ. Programmes GeneralUser GS vérifiés (banque 0 : 0 piano, 4 piano électrique Tine, 24 guitare nylon, 32
# contrebasse, 33 basse aux doigts, 35 fretless, 61 cuivres ; kits de la banque 128 : 0 standard, 8 room, 16 power, 32
# jazz, 40 balais).
# Tempo : bornes MESURÉES du style (GMD afrocuban : 105 droit, 110 rhumba, 128 le plus rapide des deux fichiers
# ternaires), pas de plage élargie : trois valeurs, deux séparées de plus de 8 BPM. Faible étendue : README.
'''

    def variante(groupe, nom, poids=1, **champs):
        nonlocal session
        session += f'\n[[variants]]\ngroup = "{groupe}"\nname = "{nom}"\nweight = {poids}\n' + ''.join(f'{k} = {scalar(v)}\n' for k, v in champs.items())

    variante('tempo', 'droit_105', bpm=105.0)
    variante('tempo', 'rhumba_110', bpm=110.0)
    variante('tempo', 'vif_128', bpm=128.0)
    for cle, nom in ((0, 'do'), (2, 're'), (5, 'fa'), (7, 'sol'), (9, 'la'), (10, 'si_bemol')):
        variante('tonalite', nom, tonality={'key': cle, 'mode': 'ionian'})
    for programme, nom in ((0, 'piano'), (4, 'piano_electrique'), (24, 'guitare_nylon')):
        variante('comping', nom, instruments={'chords_program': programme})
    for programme, nom in ((32, 'contrebasse'), (33, 'basse_aux_doigts'), (35, 'basse_fretless')):
        variante('basse', nom, instruments={'bass_program': programme})
    for programme, nom in ((0, 'standard'), (8, 'room'), (16, 'power'), (32, 'jazz')):
        variante('kit', nom, instruments={'percussion_bank': 128, 'percussion_program': programme})
    for nom, famille in (('serre', 'close_triad'), ('drop_2', 'drop2'), ('sans_fondamentale', 'rootless'), ('ouvert', 'open')):
        variante('voicing', nom, voicing=famille)
    variante('registre', 'normal', 2)
    variante('registre', 'grave', register_octaves={'Chords': -1})
    variante('energie', 'doux', intensity=0.85, density=0.85)
    variante('energie', 'soutenu', intensity=1.15, density=1.1)
    variante('arc', 'classique', 1)
    variante('arc', 'orchestre', 1, arc='orchestre')
    session += ('\n# Arc « orchestre » : exposition aux balais (kit 40), sommet aux cuivres (comping 61) ; un chorus qui ne déclare pas de\n'
                '# programme rend ceux de la session. Mêmes étiquettes que l\'arc par défaut.\n[arcs.orchestre]\n')
    session += stages(True).replace('[[stages]]', '[[arcs.orchestre.stages]]').replace('[[fall]]', '[[arcs.orchestre.fall]]')
    sauver('session.toml', session)
    print(f'{sortie} : {nb} patterns, {phrases} phrases de batterie, {relances} relances, {len(comping_phrases(index))} phrases de comping.')


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--lab', type=Path, default=Path.home() / 'Project/bascanada/harmonium_lab')
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--mix', type=Path, help='JSON des niveaux de piste (master_db, Chords, Bass, PercussionLow, PercussionHigh)')
    args = ap.parse_args()
    mix = dict(MIX)
    if args.mix and args.mix.is_file():
        mix.update(json.loads(args.mix.read_text()))
    construire(args.lab, args.out, mix)


if __name__ == '__main__':
    main()
