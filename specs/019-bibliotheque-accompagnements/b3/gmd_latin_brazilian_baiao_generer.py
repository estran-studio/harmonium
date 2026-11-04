#!/usr/bin/env python3
"""B3 : écrit le profil `gmd_latin_brazilian_baiao` (batterie du GMD, tout le reste de métier), sans lecture audio.

Commande (venv du labo, qui lit le GMD en cache vérifié par SHA256 ; le labo n'est jamais modifié) :

    ~/Project/bascanada/harmonium_lab/.venv/bin/python \\
        specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_generer.py --out profiles/gmd_latin_brazilian_baiao

`--out` est obligatoire : le script ne choisit jamais le dossier d'un profil distribué. Les fichiers écrits sont
la source du profil ; seul `percussion.toml` est GÉNÉRÉ (relancer le script le réécrit). Réutilise les modules de B8
(`b8/gmd_batterie.py`, `b8/emit.py`) : même construction des patterns, phrases et relances.

Ce qui est mesuré : la batterie du kit (probabilités par pas du GMD `latin/brazilian-baiao`, 3 fichiers `beat`,
2 batteurs, 29 `fill`, 1 batteur), le tempo (95, 95, 110 BPM), le feel droit ; la basse : registre et approches
de WJazzD `LATIN`. Tout le reste est une règle de métier dite comme telle dans les fichiers et le README : zabumba
écrite par-dessus les coups mesurés, triangle, agogô, motifs de basse, comping, grilles, arc, habillage.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ICI = Path(__file__).resolve().parent
sys.path.insert(0, str(ICI.parent / "b8"))
import emit as E  # noqa: E402
import gmd_batterie as B  # noqa: E402

ID = "gmd_latin_brazilian_baiao"
LAB = Path.home() / "Project/bascanada/harmonium_lab"
SOURCE = "latin/brazilian-baiao"
MESURES = LAB / "references/gmd-latin-brazilian-baiao/measures-drums.json"

# Fenêtre de la fondamentale de basse (`low..high`) ; WJazzD LATIN : hauteur p1 31, p50 40, p99 50, étendue 28..53 ;
# 33..44 est le meilleur des trois essais (31..42, 33..44, 35..46) pour la médiane des temps (README, « Basse »).
BASSE_BAS, BASSE_HAUT = 33, 44
# Niveaux de piste (dB), réglés sur le LUFS intégré en solo par `gmd_latin_brazilian_baiao_mix.py`.
MIX = {"master": 20.0, "Chords": -12.4, "Bass": -12.4, "Percussion": -9.5}
# Gain par rôle appliqué au niveau de rendu de B8 (réglage de service, voir README « Mixage »).
# Valeurs de B47 (niveau visé en solo sur le kit 128/0, `b47_calibrage_kit.py`), reprises pour les rôles communs, sauf la
# caisse claire : sa vélocité médiane GMD (42) est celle de coups fantômes joués à presque chaque pas (groupe `caixa`),
# le niveau visé par B47 (un coup net par temps) la rendrait trop forte ; 0,5 donne 48, les accents (x1,9) restent < 100.
GAIN = {"kick": 0.86, "snare": 0.5, "sidestick": 0.77, "ride": 0.89, "hat_closed": 0.93, "hat_pedal": 1.11,
        "tom_high": 0.52, "tom_low": 0.65}
# Voix latines (note GM, vélocité de rendu) calées par `gmd_latin_brazilian_baiao_calibrage_kit.py` : le triangle muet
# (80) reste sous -45 LUFS en solo même à 127 dans GeneralUser GS, l'agogô est plus faible que la cloche de ride.
LATINES = [("triangle_muet", 80, 127), ("triangle_ouvert", 81, 72), ("agogo_aigu", 67, 112), ("agogo_grave", 68, 105)]

MENTIONS = """Gillick, Roberts, Engel, Eck, Bamman, Learning to Groove with Inverse Sequence Transformations, ICML 2019 ; Groove MIDI Dataset v1.0.0, Google LLC, CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/).
Contains information from the Weimar Jazz Database (Jazzomat Research Project, HfM Weimar), which is made available here under the Open Database License (ODbL); individual contents under the Database Contents License (DbCL 1.0).
GMD : batterie du kit et tempo, style latin/brazilian-baiao (3 fichiers beat, 200 mesures, 2 batteurs ; 29 fill, un seul batteur) ; probabilités par pas agrégées, jamais un enregistrement. WJazzD LATIN : registre et approches de basse seulement (23 solos valides, 1639 mesures, aucune croche observable). Zabumba écrite, triangle, agogô, basse syncopée, comping, vamps, arc, habillage et mixage : règles de métier, non mesurés dans ces corpus. Aucun enregistrement ni suite de notes de solo reproduit."""

ARC = """# Arc de métier : cinq chorus de seize mesures, sans changement de grille (aucune mesure de corpus).
# Exposition à la charleston et au comping clairsemé, deux chorus de corps au ride (le second aussi à la caixa, la
# caisse claire en doubles croches), sommet dense au ride ou à la caixa avec stop-time une fois sur deux sur les deux
# dernières mesures, retombée à la charleston avec stop-time (accords et basse sur le un seulement).
repeat_from = 1
[[stages]]
choruses = 1
variants = [{ intensity = 0.65, density = 0.8, comping = [{ tag = "sparse" }], percussion = [{ tag = "hat" }] }]
[[stages]]
choruses = 2
variants = [
  { intensity = 1.0, density = 1.0, comping = [{ tag = "medium" }], percussion = [{ tag = "ride" }, { tag = "hat" }] },
  { intensity = 1.1, density = 1.1, comping = [{ tag = "medium" }, { tag = "dense" }], percussion = [{ tag = "ride" }, { tag = "caixa" }], breaks = [{ start = 9, bars = 1, chance = 0.5, tracks = ["Chords"] }] },
]
[[stages]]
choruses = 1
variants = [{ intensity = 1.5, density = 1.4, comping = [{ tag = "dense" }], percussion = [{ tag = "ride" }, { tag = "caixa" }], breaks = [{ start = -2, bars = 2, chance = 0.5, tracks = ["Chords", "Bass"], keep_steps = [0] }] }]
[[fall]]
intensity = 0.5
density = 0.5
comping = [{ tag = "sparse" }]
percussion = [{ tag = "hat" }]
breaks = [{ start = -2, bars = 2, tracks = ["Chords", "Bass"], keep_steps = [0] }]
"""

# Habillage : groupes indépendants, chacun règle des champs distincts (A2). Les plages de tempo sont tirées de la
# SOURCE : le GMD n'a que trois fichiers, à 95, 95 et 110 BPM.
TEMPOS = [("pose", [95.0, 98.0]), ("intermediaire", [101.0, 104.0]), ("allant", [107.0, 110.0])]
TONALITES = [("sol", 7), ("re", 2), ("do", 0), ("fa", 5), ("la", 9), ("si_bemol", 10)]
COMPING = [("accordeon", 21), ("guitare_nylon", 24), ("piano", 0)]
BASSES = [("contrebasse", 32), ("basse_aux_doigts", 33)]
KITS = [("standard", 0), ("salle", 8), ("jazz", 32), ("balais", 40)]
VOICINGS = [("serre", "close_triad"), ("drop_2", "drop2"), ("coquille", "shell"), ("ouvert", "open")]


def session() -> str:
    lignes = [
        "# Habillage par graine (A2) du profil `gmd_latin_brazilian_baiao` : choix de métier, aucune mesure de corpus ne",
        "# donne une variante. Programmes General MIDI de GeneralUser GS (sans Odin) : accordéon 21, guitare nylon 24,",
        "# piano 0, contrebasse 32, basse aux doigts 33 ; kits de la banque 128 : standard 0, salle 8, jazz 32, balais 40.",
        "#",
        "# Tempo : la source n'a que 95, 95 et 110 BPM ; les trois plages (95..98, 101..104, 107..110) restent dans cette",
        "# étendue de 15 BPM, elles NE SONT PAS trois régimes mesurés, et seules la première et la dernière sont séparées",
        "# de plus de 8 BPM. Aucun BPM hors de 95..110 (min_bpm et max_bpm du profil) : la plage de la source est gardée.",
        "# Tonalité : six toniques en mixolydien (G, D, C, F, A, Si bémol), sans `[fixed_tonality]` pour que la variante",
        "# prime. Limite connue : une boucle de grille ne dépend pas de la tonalité tirée (le mode n'est qu'une étiquette).",
        "#",
        "# 3 tempos x 6 tonalités x 3 comping x 2 basses x 4 kits x 4 voicings x 2 registres x 2 énergies x 2 arcs",
        "# = 13 824 habillages possibles en 28 variantes.",
    ]

    def variante(groupe: str, nom: str, **champs: object) -> None:
        lignes.extend(["", "[[variants]]", f'group = "{groupe}"', f'name = "{nom}"'])
        lignes.extend(f"{k} = {E.scalar(v)}" for k, v in champs.items())

    for nom, plage in TEMPOS:
        variante("tempo", nom, bpm=plage)
    for nom, cle in TONALITES:
        variante("tonalite", nom, tonality={"key": cle, "mode": "mixolydian"})
    for nom, prog in COMPING:
        variante("comping", nom, instruments={"chords_program": prog})
    for nom, prog in BASSES:
        variante("basse", nom, instruments={"bass_program": prog})
    for nom, prog in KITS:
        variante("kit", nom, instruments={"percussion_bank": 128, "percussion_program": prog})
    for nom, famille in VOICINGS:
        variante("voicing", nom, voicing=famille)
    variante("registre", "normal")
    variante("registre", "grave", register_octaves={"Chords": -1})
    variante("energie", "souple", intensity=0.9, density=0.9)
    variante("energie", "franc", intensity=1.1, density=1.1)
    variante("arc", "continu")
    variante("arc", "orchestre", arc="orchestre")
    # Même arc que `arc.toml`, avec orchestration par chorus : balais à l'exposition, accordéon au sommet.
    orchestre = ARC.replace("[[stages]]", "[[arcs.orchestre.stages]]").replace("[[fall]]", "[[arcs.orchestre.fall]]")
    orchestre = orchestre.replace("repeat_from = 1", "[arcs.orchestre]\nrepeat_from = 1")
    orchestre = orchestre.replace("variants = [{ intensity = 0.65,", "variants = [{ instruments = { percussion_program = 40 }, intensity = 0.65,")
    orchestre = orchestre.replace("variants = [{ intensity = 1.5,", "variants = [{ instruments = { chords_program = 21 }, intensity = 1.5,")
    return "\n".join(lignes) + "\n\n" + orchestre.replace("# Arc de métier", "# Arc nommé « orchestre » : arc de métier", 1)


def style() -> E.Style:
    C, L = E.Chord, E.Loop
    boucles = [
        L(2, [C(0, "7", 64)], "I7, seize mesures : vamp modal."),
        L(3, [C(0, "7", 16), C(10, "", 16)] * 2, "I7 quatre mesures, bVII quatre mesures, deux fois."),
        L(3, [C(0, "7", 16), C(5, "", 16)] * 2, "I7 quatre mesures, IV quatre mesures, deux fois."),
        L(2, [C(0, "7", 8), C(10, "", 8), C(0, "7", 8), C(5, "", 8)] * 2,
          "Vamp de huit mesures, deux fois : I7, bVII, I7, IV (deux mesures chacun)."),
    ]
    # Basse (pas, durée, demi-tons, vélocité) : fondamentale sur le un et le trois, quinte en contretemps (pas 6, 14).
    motifs = [
        [(0, 4, 0, 94), (6, 2, 7, 82), (8, 4, 0, 90), (14, 2, 7, 80)],
        [(0, 3, 0, 96), (3, 1, 0, 64), (6, 2, 7, 84), (8, 3, 0, 92), (11, 1, 0, 64), (14, 2, 7, 82)],
        [(0, 6, 0, 94), (6, 2, 7, 82), (8, 6, 0, 90), (14, 2, 7, 80)],
        [(0, 3, 0, 96), (3, 3, 0, 82), (6, 2, 7, 82), (8, 3, 0, 90), (11, 3, 0, 78), (14, 2, 7, 80)],
        [(0, 4, 0, 94), (6, 2, 7, 82), (8, 4, 0, 90), (14, 2, 0, 80)],
        [(0, 6, 0, 94), (6, 2, 7, 80), (8, 4, 0, 88)],
    ]
    rythmes = [
        ("sparse", [(0, 6, 68), (8, 6, 64)]),
        ("sparse", [(0, 12, 66)]),
        ("sparse", [(0, 3, 70), (6, 2, 60), (8, 4, 66)]),
        ("medium", [(0, 3, 74), (6, 2, 66), (8, 3, 70), (14, 2, 68, "anticipate")]),
        ("medium", [(0, 3, 74), (3, 1, 60), (6, 2, 66), (8, 3, 70), (11, 1, 58), (14, 2, 66, "anticipate")]),
        ("medium", [(0, 4, 72), (6, 2, 64), (10, 2, 62), (14, 2, 68, "anticipate")]),
        ("dense", [(0, 3, 78), (3, 1, 62), (6, 2, 70), (8, 3, 74), (11, 1, 62), (14, 2, 70, "anticipate")]),
        ("dense", [(0, 2, 78), (2, 1, 48, ("probability", 0.35)), (3, 1, 64), (6, 2, 70), (8, 2, 74),
                   (10, 1, 48, ("probability", 0.35)), (11, 1, 64), (14, 2, 70, "anticipate")]),
        ("dense", [(0, 3, 78), (3, 1, 64), (4, 1, 50, ("probability", 0.4)), (6, 2, 70), (8, 3, 74), (11, 1, 64),
                   (12, 1, 50, ("probability", 0.4)), (14, 2, 72, "anticipate")]),
    ]
    s = E.Style(
        id=ID, name="Baião brésilien (GMD latin/brazilian-baiao)",
        tradition="GMD : deux batteurs au kit ; voix traditionnelles du Nordeste écrites par métier",
        tags=["baiao", "bresil", "latin", "accompagnement", "grille"], aliases=[], title="Baião brésilien",
        key=7, mode="mixolydian", bpm=(95.0, 95.0, 110.0),
        bpm_note="GMD : fichiers 95, 95, 110 BPM ; p10 95, médiane 95, p90 107. Étendue observée 95..110, sans extension.",
        parts=[E.Part("Vamp", 16, "Formes communes écrites, relatives à la tonique ; aucune œuvre nommée.", boucles)],
        sequence=[0], tours=5, bass_low=BASSE_BAS, bass_high=BASSE_HAUT,
        bass_motifs=[E.BassMotif(1, 4 if i < 2 else 2, [E.BassNote(*n) for n in ns],
                                 "Métier : fondamentale sur le un et le trois, quinte en contretemps.")
                     for i, ns in enumerate(motifs)],
        chords_center=64, chords_low=50, chords_high=80, voicing="drop2", chords_program=21, bass_program=33,
        kit_program=0, chords_jitter=0.025, bass_jitter=0.02, mix=dict(MIX), corpus=MENTIONS)
    s.bass_passing = """# WJazzD LATIN : approche du temps 4 avant un changement d'accord à la barre, 1205 événements, 22 solos :
# aucune 60 %, chromatique 22 % (IC 17 à 26 %), diatonique 19 % (IC 11 à 29 %) ; poids 60 / 22 / 19.
# Le moteur applique la règle à toute note de motif qui change de hauteur au suivant (`next_target_differs`), donc
# plus souvent que la part mesurée (README, limites).
[accompaniment.bass.passing]
none_weight = 60
density = [[0.0, 0.2], [0.5, 1.0], [1.0, 1.8]]
phrase = [[0.0, 1.0], [1.0, 1.0]]
avoid_melody_intervals = []
rules = [
  { when = "next_target_differs", before = [4], motion = "chromatic", weight = 22, duration = 2, velocity_delta = -16, min_leap = 0 },
  { when = "next_target_differs", before = [4], motion = "scale_step", weight = 19, duration = 2, velocity_delta = -16, min_leap = 0 },
]"""
    s.comments = {
        "bass": "# Registre WJazzD LATIN (hauteur p1 31, p50 40, p99 50 ; étendue 28..53). Rythmes et intervalles : métier.",
        "chords": "# Baião à l'accordéon, à la guitare nylon ou au piano : rythmes, phrases et voicings de métier, aucune mesure de comping.",
        "form": "# Seize mesures de vamp modal : deux modules de huit, règle de métier.",
        "timing": "# Jitter tonal de métier ; médianes par voix du GMD ajoutées plus bas.",
    }
    s.default_comp = [(0, 3, 70), (6, 2, 64), (8, 3, 68), (14, 2, 64)]
    s.comping = [E.Comp(3, t, h) for t, h in rythmes]
    for debut, tag in [(0, "sparse"), (3, "medium"), (6, "dense")]:
        s.comp_phrases.extend([
            E.CompPhrase(3, tag, [debut, debut, debut + 1, debut]),
            E.CompPhrase(2, tag, [debut + 1, debut + 2]),
            E.CompPhrase(2, tag, [debut, debut + 1, debut, debut + 2], "part_end"),
        ])
    return s


def hit_toml(data: dict, h: dict) -> str:
    v = data["voices"][h["role"]]
    xs = [f"step = {h['step']}", f'voice = "{v["name"]}"']
    if h["note"] != v["note"]:
        xs.append(f"note = {h['note']}")
    d = B.delta(v, h["rel"])
    if d:
        xs.append(f"delta_velocity = {d}")
    return "{ " + ", ".join(xs) + " }"


def mesures_groupes(lab: Path) -> tuple[dict, dict]:
    """Mesures GMD des trois groupes de la batterie (étiquettes d'arc) et mesures de tout le style, pour les probabilités.

    `ride` et `hat` : les groupes (timekeeper, figure `quarters_plus`) de B8. `caixa` : les mesures SANS timekeeper (la
    caisse claire en doubles croches tient le temps), que B8 écarte ; 67 mesures, dont 55 d'un seul fichier (82 %,
    au-delà du seuil de 60 % de B8) : groupe gardé hors seuil, dit dans le README.
    """
    sys.path.insert(0, str(lab / "src"))
    from harmonium_lab.backing import gmd_styles as G  # noqa: PLC0415

    beats = [r for r in B.load_results(lab) if r.row.style == SOURCE and r.row.beat_type == "beat" and not r.reject
             and r.row.time_signature == "4-4" and r.feel == "straight" and r.measures]
    tous = [m for r in beats for m in r.measures]
    groupes = {"ride": [], "hat": [], "caixa": []}
    for m in tous:
        tk = G.timekeeper_of(m)
        if tk == "none":
            groupes["caixa"].append(m)
        elif G.figure_of(m) == "quarters_plus" and tk in ("ride", "hat_closed"):
            groupes["ride" if tk == "ride" else "hat"].append(m)
    fenetres: dict = {}
    for r in beats:
        etiquettes = [G.timekeeper_of(m) == "none" for m in r.measures]
        for taille in (2, 4):
            for debut in range(0, len(etiquettes) - taille + 1, taille):
                if all(etiquettes[debut:debut + taille]):
                    f = fenetres.setdefault(taille, {"n": 0, "fichiers": set()})
                    f["n"] += 1
                    f["fichiers"].add(r.row.path)
    return {"groupes": groupes, "medianes": G.role_velocity_medians(tous), "fenetres": fenetres}, groupes


def proba(ms: list, role: str, step: int) -> float:
    """Probabilité mesurée du coup (rôle GMD, pas) dans un groupe de mesures."""
    return sum(1 for m in ms if step in m.steps.get(role, {})) / len(ms)


def ajouter_caixa(data: dict, mesures: dict) -> None:
    """Patterns, phrases et relances de l'étiquette `caixa` : mêmes seuils par densité que B8, la caisse claire fait le timekeeper."""
    ms, medianes = mesures["groupes"]["caixa"], mesures["medianes"]
    from collections import Counter  # noqa: PLC0415

    for densite, seuils in B.LEVELS.items():
        seuils = {("snare" if k == "timekeeper" else k): v for k, v in seuils.items() if k in ("timekeeper", "hat_pedal", "kick", "snare", "sidestick")}
        coups = []
        for role, seuil in seuils.items():
            for pas in range(16):
                presents = [m.steps[role][pas] for m in ms if pas in m.steps.get(role, {})]
                if presents and len(presents) / len(ms) >= seuil:
                    rel = sum(v / medianes[role] for v, _, _ in presents) / len(presents)
                    note = sorted(Counter(n for _, n, _ in presents).items(), key=lambda kv: (-kv[1], kv[0]))[0][0]
                    coup = {"step": pas, "role": role, "note": note, "rel": rel}
                    if role == "snare" and len(presents) / len(ms) < 0.95:
                        coup["prob"] = round(len(presents) / len(ms), 3)
                    coups.append(coup)
        coups.sort(key=lambda h: (h["step"], h["role"] != "kick", h["role"]))
        data["patterns"].append({"density": densite, "weight": len(ms), "tag": "caixa", "hits": coups, "sources": ["caixa:aucun timekeeper"]})
    base = len(data["patterns"]) - 3
    for taille, f in sorted(mesures["fenetres"].items(), reverse=True):
        if f["n"] >= 2:  # seuil relâché de B8 : >= 2 occurrences, >= 1 fichier
            for densite in range(3):
                data["phrases"].append({"density": densite, "weight": f["n"], "tag": "caixa", "bars": taille,
                                        "patterns": [base + densite] * taille, "n_files": len(f["fichiers"]), "n_drummers": 2})
    data["fill_tags"].append("caixa")


def batterie(ref: dict) -> tuple[dict, str]:
    spec = B.DrumSpec(styles=[SOURCE], min_files=2, min_measures=20, max_share=0.6, min_phrase_count=2,
                      min_phrase_files=1, min_change_count=2, arc_tags=["ride", "hat"], gain=GAIN)
    data = B.build(B.load_results(LAB), spec, LAB)
    mesures, groupes = mesures_groupes(LAB)
    ajouter_caixa(data, mesures)
    # Ajouts de métier absents du GMD (kit Roland sans percussion latine) : triangle et agogô.
    for role, note, vel in LATINES:
        B.VOICES[role] = (role, "PercussionHigh")
        data["voices"][role] = {"name": role, "note": note, "track": "PercussionHigh", "velocity": vel}
    for p in data["patterns"]:
        # Zabumba dès la densité 0 : les quatre pas de grosse caisse (0, 3, 8, 11) ont une probabilité >= 0,75 sur tout
        # le corpus (mesure) ; les écrire à toutes les densités est un choix d'ossature.
        if p["density"] == 0:
            vus = {h["step"] for h in p["hits"] if h["role"] == "kick"}
            p["hits"] += [{"step": x[0], "role": "kick", "note": 36, "rel": x[4]} for x in ref["per_step"]["kick"]
                          if x[2] >= 0.75 and x[0] not in vus]
        for step in range(0, 16, 2):
            role = "triangle_muet" if step % 4 == 0 else "triangle_ouvert"
            p["hits"].append({"step": step, "role": role, "note": data["voices"][role]["note"], "rel": 0.8 if p["density"] == 0 else 1.0})
        if p["density"] == 2:
            for step, role in [(0, "agogo_grave"), (3, "agogo_aigu"), (8, "agogo_grave"), (11, "agogo_aigu")]:
                p["hits"].append({"step": step, "role": role, "note": data["voices"][role]["note"], "rel": 1.0})
    entete = f"""# GÉNÉRÉ : ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b3/{ID}_generer.py --out profiles/{ID}
# Batterie du profil `{ID}` : Groove MIDI Dataset v1.0.0 (Google LLC, CC BY 4.0 ; Gillick, Roberts, Engel, Eck, Bamman, ICML 2019).
# {SOURCE} : 3 fichiers beat / 200 mesures / 2 batteurs (limite écrite : « juste », deux batteurs) ; 29 fill / 1 batteur.
# Trois groupes de mesures, une étiquette d'arc chacun : `ride` (timekeeper ride, figure `quarters_plus`, 57 mesures, 2 fichiers,
# 2 batteurs), `hat` (charleston fermée, même figure, 21 mesures, 2 fichiers, 1 batteur), `caixa` (aucun timekeeper : la caisse
# claire en doubles croches tient le temps, 67 mesures, 2 fichiers, 2 batteurs, mais 82 % d'un seul fichier : au-delà du seuil
# de 60 % de B8, gardé hors seuil et dit tel dans le README). Les 76 autres mesures (figures `sparse` et `other`) n'ont pas de pattern.
# Seuils de B8 relâchés (3 fichiers seulement) : phrases >= 2 occurrences sur >= 1 fichier (aucune prétention inter-batteurs) ;
# `probability` = fréquence mesurée du coup dans le groupe pour la caisse claire non systématique.
# Zabumba (grosse caisse, dès la densité 0), triangle (80 muet sur les temps, 81 ouvert sur les contretemps) et agogô
# (67 aigu, 68 grave, densité 2) : règles de métier, le GMD (kit Roland TD-11) n'a aucune percussion latine.
# Relances : 29 fichiers, un groupe (1 mesure, départ pas 0), seuil 0,25 ; poids sans relance 46 : valeur de service.
"""
    perc = B.percussion_toml(data, entete, 46)
    # `probability` (schéma) : la caisse claire n'est pas systématique ; fréquence mesurée dans le groupe du pattern.
    sections = perc.split("[[patterns]]")
    for i, p in enumerate(data["patterns"], start=1):
        for h in p["hits"]:
            if h["role"] != "snare":
                continue
            if p["tag"] == "caixa":
                q = h.get("prob")
                if q is None:
                    continue
            else:
                if p["density"] != 2 or h["rel"] >= 1.4:
                    continue
                q = round(proba(groupes[p["tag"]], "snare", h["step"]), 3)
            old = "  " + hit_toml(data, h) + ","
            sections[i] = sections[i].replace(old, old[:-3] + f", probability = {q} }},", 1)
    return data, "[[patterns]]".join(sections)


def micro_timing(data: dict, ref: dict) -> str:
    """Médianes du GMD par voix (pas) ; bornées à ±0,2 pas, limite du schéma. Pas de dispersion reproduite."""
    lignes = ["# Médianes GMD par voix, bornées à ±0,2 pas (limite du schéma) ; aucune dispersion reproduite."]
    for piste, roles in [("PercussionLow", ["kick"]),
                         ("PercussionHigh", ["snare", "sidestick", "hat_closed", "hat_pedal", "ride", "tom_high", "tom_low"])]:
        lignes += [f"[performance.tracks.{piste}]", "voices = ["]
        for role in roles:
            if role not in data["voices"]:
                continue
            v = ref["micro_timing"][role]
            on = v.get("on_beat", v.get("off_beat"))["steps"]["p50"]
            off = v.get("off_beat", v.get("on_beat"))["steps"]["p50"]
            lignes.append(f'  {{ note = {data["voices"][role]["note"]}, on_beat = {max(-.2, min(.2, on))}, off_beat = {max(-.2, min(.2, off))} }},')
        lignes += ["]", ""]
    return "\n".join(lignes)


def profil_toml(s: E.Style, data: dict, ref: dict) -> str:
    entete = """# Profil `gmd_latin_brazilian_baiao` (phase 019, B3) : baião brésilien sous une grille d'accords, sans Odin.
#
# Le nom est celui de la source qui définit le style (principe VII) : le style `latin/brazilian-baiao` du Groove MIDI
# Dataset v1.0.0 (3 fichiers `beat`, 200 mesures, 2 batteurs : « juste », pas « assez » ; 29 fichiers `fill`, un seul
# batteur). Ce n'est pas « le baião » : deux batteurs jouant un kit électronique Roland TD-11. La batterie du kit
# (grosse caisse aux pas 0, 3, 8, 11, charleston à la pédale, ride) et le tempo (95..110 BPM) viennent de là ; la basse
# est calibrée sur WJazzD `LATIN` pour ce qui s'y mesure (registre, approches avant un changement) ; la zabumba écrite,
# le triangle, l'agogô, les motifs de basse en fondamentale-quinte, le comping, les vamps modaux, l'arc et l'habillage
# sont des RÈGLES DE MÉTIER (idiome traditionnel du Nordeste), dites telles dans le README : aucun corpus ouvert ne les mesure.
#
# Fait pour jouer sous une grille fournie par l'utilisateur : basse, comping et batterie ne dépendent que de l'accord
# publié et de la graine. Les quatre grilles par défaut sont des formes communes écrites ici, relatives à la tonique
# (vamps de seize mesures à un ou deux accords : I7, bVII, IV mixolydiens) ; aucune réharmonisation (D4), aucune œuvre.
# Pas de `[fixed_tonality]` : la tonalité vient de l'habillage (`session.toml`). Aucune mélodie (piste `Melody` absente).
# Aucun swing : feel droit en 16 pas. Aucune clave : le baião n'en a pas. Tout sonne avec GeneralUser GS."""
    profil = E.profile_toml(s, entete)
    profil = profil.replace('[fixed_tonality]\nkey = 7\nmode = "mixolydian"\n\n', "")
    profil = profil.replace('arc = "arc.toml"', 'arc = "arc.toml"\nsession = "session.toml"', 1)
    profil = profil.replace(f"high = {BASSE_HAUT}\nmotifs", f"high = {BASSE_HAUT}\nwalk_low = 28\nwalk_high = 53\nmotifs", 1)
    profil = profil.replace('voicing = "drop2"', 'voicing = "drop2"\nregister_drift = { step = 1, max = 4 }', 1)
    cles = ", ".join(f'{{ key = {c}, mode = "mixolydian", weight = 1 }}' for _n, c in TONALITES)
    profil = profil.replace('key_modes = [{ key = 7, mode = "mixolydian", weight = 1 }]', f"key_modes = [{cles}]", 1)
    return profil + "\n" + micro_timing(data, ref)


def dynamique(s: E.Style) -> str:
    lignes = []
    for l in E.dynamics_toml(s).splitlines():
        if l.startswith("#"):
            continue
        if l.startswith("tempo_offset ="):
            l = "tempo_offset = [[0.0, 0.0], [1.0, 0.0]]"
        if l.startswith("parallel_modes ="):
            l = 'parallel_modes = [{ bright = "mixolydian", dark = "dorian", weight = 1 }]'
        if l.startswith("dark_below ="):
            l = "dark_below = 0.1"
        if l.startswith("bright_above ="):
            l = "bright_above = 0.9"
        lignes.append(l)
    return ("# Courbes de service : aucune mesure de corpus ne relie une émotion à un paramètre (README du profil).\n"
            "# Tempo plat : la source n'a que 95..110 BPM, la plage de chaque variante de session est donc respectée.\n"
            "# Pas de `fixed_tonality` : la variante de session fixe la tonalité ; seuils de bascule écartés de la valence neutre\n"
            "# (0,3) pour que le mode tiré (mixolydien, clair ; dorien, sombre) ne bascule pas de lui-même.\n"
            + "\n".join(lignes) + "\n")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    ref = json.loads(MESURES.read_text())["styles"][SOURCE]["groups"]["4-4:straight"]["beat"]
    data, perc = batterie(ref)
    s = style()
    fichiers = {
        "profile.toml": profil_toml(s, data, ref), "percussion.toml": perc, "arc.toml": ARC,
        "dynamics.toml": dynamique(s), "arrangement.toml": E.arrangement_toml(s), "set.toml": E.SET_TOML,
        "cells.toml": E.CELLS_TOML, "session.toml": session(),
    }
    for nom, texte in fichiers.items():
        (args.out / nom).write_text(texte, encoding="utf-8")
    print(json.dumps({"stats": data["stats"], "patterns": len(data["patterns"]), "phrases": len(data["phrases"]),
                      "fills": len(data["fills"])}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
