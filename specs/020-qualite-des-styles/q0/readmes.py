#!/usr/bin/env python3
"""Q0 : section « Corrections Q0 » des README des 13 profils d'E1 et lignes « Timing » corrigées.

Idempotent : la section est délimitée par `<!-- q0:début -->` / `<!-- q0:fin -->`. Lit les tables de `appliquer.py` et de `batterie.py`.

    python3 specs/020-qualite-des-styles/q0/readmes.py
"""
from __future__ import annotations

import re
import sys

from appliquer import AMPLITUDE, BASS_JITTER, CHORDS_JITTER, DRUM_JITTER, E1, PHRASE, REPO, SHAPES, VEL_MAX, VEL_MIN, WALKING
from batterie import PARAMS

BEGIN, END = "<!-- q0:début -->", "<!-- q0:fin -->"
CLASSES = {"b1": "temps 1", "b3": "temps 3", "bk": "contretemps 2 et 4", "and_": "croche de contretemps", "e": "1re double croche",
           "a": "3e double croche", "ant": "attaque anticipée"}
WALTZ = {"bk": "temps 2", "b3": "temps 3", "and_": "croche de contretemps", "b1": "temps 1"}

#: Règle de métier de chaque style (comping, basse) : aucune mesure de corpus ne les porte (décision F3 de 018).
METIER = {
    "wjazzd_swing": ("charleston : accent sur le « et » du 2 (pas 6) et sur l'anticipation du prochain accord, temps 1 et 3 légers",
                     "marche : arrivée d'accord +20, courbe de phrase ±18 (A6)"),
    "wjazzd_ballad": ("accords tenus : accent sur le « et » du 2 et l'anticipation, temps légers",
                      "deux-temps et marche : courbe de phrase ±18 (A6), arrivées +20"),
    "gmd_jazz_waltz": ("temps 2 appuyé, temps 1 et 3 plus légers (« pah-pah » qui penche vers le 2), anticipations discrètes",
                       "marche : courbe de phrase ±18 (A6), arrivées +20"),
    "gmd_latin_brazilian_bossa": ("syncopes des pas 6 et 14 accentuées (le motif de la guitare), temps 1 appuyé, doubles croches fantômes",
                                  "temps 1 appuyé, reprises de croche plus légères"),
    "gmd_latin_brazilian_samba": ("accent sur les contretemps 2 et 4 (surdo) et la 3e double croche (tamborim), temps 1 plus léger",
                                  "surdo : accent sur les 2/4, notes fantômes aux 3e doubles croches"),
    "gmd_latin_brazilian_baiao": ("temps 1 (zabumba) et pas 6 appuyés, doubles croches fantômes", "temps 1 appuyé, pas 6 un peu appuyé"),
    "gmd_afrocuban": ("clave : pas 6, 14 et anticipation accentués, temps plats, doubles croches fantômes",
                      "tumbao : anticipation et pas 6 et 12 accentués"),
    "gmd_funk": ("le « un » (temps 1) très appuyé, 3e double croche accentuée (syncope), scratchs fantômes aux 1res doubles croches",
                 "règle de métier déjà portée par les motifs de 019 (écart-type ≥ 9,8) : inchangée"),
    "gmd_soul": ("temps 1 et backbeat appuyés, 3e double croche légèrement appuyée, scratchs fantômes", "temps 1 appuyé, notes mortes aux 1res doubles croches"),
    "gmd_reggae": ("skank : « et » des pas 6 et 14 très appuyés, temps 1 et 3 plus légers (one drop)", "temps 3 appuyé (one drop), temps 1 moyen"),
    "gmd_rock": ("temps 1, backbeat et temps 3 appuyés, croches de contretemps et doubles croches fantômes, anticipation poussée",
                 "temps 1 appuyé, croches de contretemps et doubles croches plus légères"),
    "gmd_pop": ("temps 1 et backbeat appuyés, croches de contretemps et doubles croches fantômes", "temps 1 appuyé, reprises plus légères"),
    "gmd_blues_shuffle": ("temps 1 et backbeat (2 et 4) très appuyés, triolet du milieu fantôme",
                          "motifs de 019 (longue-brève déjà accentuée, écart-type ≥ 9,2) : inchangés"),
}


def describe(shape: dict, waltz: bool) -> str:
    names = WALTZ | {k: v for k, v in CLASSES.items() if k not in WALTZ} if waltz else CLASSES
    plus = [f"{names[c]} ({shape[c]:+g})" for c in names if shape[c] >= 0.3]
    minus = [f"{names[c]} ({shape[c]:+g})" for c in names if shape[c] <= -0.3]
    steps = [f"pas {s} ({v:+g})" for s, v in shape["s"].items()]
    parts = []
    if plus or steps:
        parts.append("appuyés : " + ", ".join(plus + steps))
    if minus:
        parts.append("légers : " + ", ".join(minus))
    parts.append(f"coups probabilistes : {shape['ghost']:+g} de plus")
    return " ; ".join(parts)


def section(pid: str) -> str:
    (cname, camp, cbias), bass = AMPLITUDE[pid]
    waltz = pid == "gmd_jazz_waltz"
    m = METIER[pid]
    drums = PARAMS[pid]
    lines = [
        BEGIN,
        "## Corrections Q0 (phase 020, `TUNING_VERSION` 34)",
        "",
        "Compte rendu, mesures avant / après et limites : `specs/020-qualite-des-styles/q0-corrections-rapides.md`. Scripts : "
        "`specs/020-qualite-des-styles/q0/` (`appliquer.py` : accents et mise en place, `batterie.py` : groove de batterie).",
        "",
        f"- **Accents du comping** (règle de métier : {m[0]}). Chaque coup garde la vélocité de 019 et reçoit un écart selon sa place, "
        f"forme `{cname}`, amplitude {camp}, décalage de niveau {cbias:+d} (le niveau moyen est conservé : la calibration du mixage reste valable) ; "
        f"{describe(SHAPES[cname], waltz)}. Plage de vélocité des accords et de la basse : {VEL_MIN} à {VEL_MAX} (`velocity_min`, `velocity_max` d'A6).",
    ]
    if bass:
        bname, bamp, bbias = bass
        lines.append(f"- **Accents de la basse** ({m[1]}) : forme `{bname}`, amplitude {bamp}, décalage {bbias:+d} ; {describe(SHAPES[bname], waltz)}.")
    elif pid in WALKING:
        lines.append(f"- **Accents de la basse** ({m[1]}) : vélocité de la marche {WALKING[pid]['velocity']}, arrivées +{WALKING[pid]['accent']}, "
                     f"courbe de phrase ±{PHRASE[pid]}.")
    else:
        lines.append(f"- **Accents de la basse** : {m[1]}.")
    lines.append(
        f"- **Groove de batterie** : phrases et patterns de 019 inchangés ; probabilité des coups secondaires {drums['fixed']:g}, "
        f"{drums['extra']} note(s) fantôme(s) à {drums['ghost_chance']:g}, exposant des probabilités {drums['prob_exp']:g}, "
        f"phrases tenues ×{drums['hold']:g}, part de fins de phrase sans relance ×{drums['fill_scale']:g}.")
    if pid in DRUM_JITTER:
        lines.append(f"- **Mise en place** : basse sur les décalages de la grosse caisse (A6, par pas), accords sur la moyenne caisse claire / timekeeper ; "
                     f"`jitter` basse {BASS_JITTER:g} et accords {CHORDS_JITTER:g} pas ; `jitter` de la batterie {DRUM_JITTER[pid]:g} pas "
                     "(écart-type intra-fichier du GMD) ; égrenage ancré sur la première corde (`anchor = 0`).")
    elif pid in {"wjazzd_swing", "wjazzd_ballad", "gmd_jazz_waltz"}:
        lines.append(f"- **Mise en place** : basse et accords sans décalage systématique (la batterie n'a aucun décalage par voix ; le retard de +0,2 pas du piano "
                     f"est retiré : aucun instrument ne dérive de la grille plus que la batterie) ; `jitter` {BASS_JITTER:g} pas.")
    else:
        lines.append(f"- **Mise en place** : basse sur les décalages de la grosse caisse (A6, par pas), accords sur la moyenne caisse claire / timekeeper ; "
                     f"`jitter` basse {BASS_JITTER:g} et accords {CHORDS_JITTER:g} pas (celui de la batterie est déjà mesuré) ; égrenage ancré sur la première corde "
                     "quand le profil en a un (`anchor = 0`).")
    lines += ["", END, ""]
    return re.sub(r"(?<=\d)\.(?=\d)", ",", "\n".join(lines))


TIMING_ROW = {
    "wjazzd_swing": "basse et accords sur la batterie (0 pas, Q0), jitter 0,02 pas",
    "wjazzd_ballad": "basse et accords sur la batterie (0 pas, Q0), jitter 0,02 pas",
    "gmd_jazz_waltz": "basse et accords sur la batterie (0 pas, Q0), jitter 0,02 pas",
    "gmd_soul": "basse sur la grosse caisse, accords sur la caisse claire et le charleston (décalages par voix, Q0), jitter 0,02 pas ; batterie 0,045 pas",
    "gmd_reggae": "basse sur la grosse caisse, accords sur la caisse claire et le charleston (décalages par voix, Q0), jitter 0,02 pas ; batterie 0,06 pas",
    "gmd_rock": "basse sur la grosse caisse, accords sur la caisse claire et le charleston (décalages par voix, Q0), jitter 0,02 pas ; batterie 0,07 pas",
    "gmd_pop": "basse sur la grosse caisse, accords sur la caisse claire et le charleston (décalages par voix, Q0), jitter 0,02 pas ; batterie 0,06 pas",
}


def main() -> int:
    for pid in E1:
        path = REPO / "profiles" / pid / "README.md"
        text = path.read_text()
        block = section(pid)
        if BEGIN in text:
            text = re.sub(re.escape(BEGIN) + r".*?" + re.escape(END) + r"\n?", lambda m: block, text, flags=re.S)
        else:
            text = text.rstrip("\n") + "\n\n" + block
        if pid in TIMING_ROW:
            text = re.sub(r"(?m)^\| Timing \|[^\n]*\|$", lambda m: f"| Timing | {TIMING_ROW[pid]} |", text, count=1)
        path.write_text(text)
        print("ok", pid)
    return 0


if __name__ == "__main__":
    sys.exit(main())
