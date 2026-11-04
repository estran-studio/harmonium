#!/usr/bin/env python3
"""B6 : écrit `profiles/gmd_blues_shuffle/session.toml` (habillage par graine, A2) et `arc.toml`. Valeurs de métier et de service ;
seules les plages de tempo viennent du GMD. Idempotent, n'importe quel Python 3 :

    python3 specs/019-bibliotheque-accompagnements/b6/emit_session.py
"""
from __future__ import annotations

from pathlib import Path

PROFILE = Path(__file__).resolve().parents[3] / "profiles" / "gmd_blues_shuffle"

#: Programmes General MIDI 0-based de GeneralUser GS (banque 0) et kits de la banque 128, vérifiés dans le `phdr`
#: (`r2-styles/habillage.py`, tables PROGRAMS et KITS).
TEMPOS = [("lent", 2, 72, 92), ("medium", 3, 96, 112), ("vif", 3, 114, 126), ("rapide", 2, 128, 134)]
TONALITES = [("mi", 4, 3), ("la", 9, 3), ("sol", 7, 2), ("do", 0, 2), ("si_bemol", 10, 2), ("re", 2, 1)]
COMPING = [("piano", 3, 0), ("guitare_claire", 2, 27), ("guitare_saturee", 2, 29), ("orgue", 2, 16), ("orgue_rock", 1, 18),
           ("piano_honky_tonk", 1, 3)]
BASSE = [("electrique_doigts", 3, 33), ("contrebasse", 3, 32), ("mediator", 1, 34), ("fretless", 1, 35)]
KITS = [("standard", 3, 0), ("room", 2, 8), ("jazz", 2, 32), ("power", 1, 16), ("balais", 1, 40)]
VOICINGS = [("notes_guides", 3, "guide_tones"), ("shell", 2, "shell"), ("drop_2", 1, "drop2"), ("rootless", 1, "rootless")]


def variant(group: str, name: str, weight: int = 1, **fields: str) -> str:
    lines = ["[[variants]]", f'group = "{group}"', f'name = "{name}"']
    if weight != 1:
        lines.append(f"weight = {weight}")
    lines += [f"{k} = {v}" for k, v in fields.items()]
    return "\n".join(lines)


def build_variants() -> str:
    out = [variant("tempo", n, w, bpm=f"[{lo:.1f}, {hi:.1f}]") for n, w, lo, hi in TEMPOS]
    out += [variant("tonalite", f"{n}_mixolydien", w, tonality=f'{{ key = {k}, mode = "mixolydian" }}') for n, k, w in TONALITES]
    out += [variant("comping", n, w, instruments=f"{{ chords_program = {p} }}") for n, w, p in COMPING]
    out += [variant("basse", n, w, instruments=f"{{ bass_program = {p} }}") for n, w, p in BASSE]
    out += [variant("kit", n, w, instruments=f"{{ percussion_bank = 128, percussion_program = {p} }}") for n, w, p in KITS]
    out += [variant("voicing", n, w, voicing=f'"{v}"') for n, w, v in VOICINGS]
    out += [variant("registre", "normal", 4), variant("registre", "haut", 1, register_octaves="{ Chords = 1 }"),
            variant("registre", "grave", 1, register_octaves="{ Chords = -1 }")]
    out += [variant("energie", "doux", 1, intensity="0.85", density="0.85"), variant("energie", "normal", 2),
            variant("energie", "fort", 1, intensity="1.15", density="1.1")]
    out += [variant("arc", "defaut", 3), variant("arc", "orchestre", 2, arc='"orchestre"'), variant("arc", "direct", 1, arc='"direct"'),
            variant("arc", "montee", 1, arc='"montee"')]
    return "\n\n".join(out)


def n_combinations() -> int:
    sizes = [len(TEMPOS), len(TONALITES), len(COMPING), len(BASSE), len(KITS), len(VOICINGS), 3, 3, 4]
    total = 1
    for s in sizes:
        total *= s
    return total


def break_(start: int, bars: int, chance: float | None, tracks: list[str], keep: bool = False) -> str:
    parts = [f"start = {start}", f"bars = {bars}"]
    if chance is not None:
        parts.append(f"chance = {chance}")
    parts.append("tracks = [" + ", ".join(f'"{t}"' for t in tracks) + "]")
    if keep:
        parts.append("keep_steps = [0]")
    return "{ " + ", ".join(parts) + " }"


def tags(*items: tuple[str, int]) -> str:
    return "[" + ", ".join(f'{{ tag = "{t}", weight = {w} }}' if w != 1 else f'{{ tag = "{t}" }}' for t, w in items) + "]"


def variants_block(rows: list[str]) -> str:
    return "variants = [\n" + "\n".join(f"  {r}," for r in rows) + "\n]"


def v(weight: int | None = None, intensity: float | None = None, density: float | None = None, comping: str | None = None,
      percussion: str | None = None, instruments: str | None = None, breaks: list[str] | None = None) -> str:
    parts = []
    if weight is not None:
        parts.append(f"weight = {weight}")
    if intensity is not None:
        parts.append(f"intensity = {intensity}")
    if density is not None:
        parts.append(f"density = {density}")
    if comping:
        parts.append(f"comping = {comping}")
    if percussion:
        parts.append(f"percussion = {percussion}")
    if instruments:
        parts.append(f"instruments = {{ {instruments} }}")
    if breaks:
        parts.append("breaks = [" + ", ".join(breaks) + "]")
    return "{ " + ", ".join(parts) + " }"


def stage(name: str, choruses: int, rows: list[str]) -> str:
    return f"[[{name}]]\nchoruses = {choruses}\n{variants_block(rows)}"


def fall(name: str, intensity: float, density: float, comping: str, percussion: str, breaks: list[str], instruments: str | None = None) -> str:
    lines = [f"[[{name}]]", f"intensity = {intensity}", f"density = {density}", f"comping = {comping}", f"percussion = {percussion}"]
    if instruments:
        lines.append(f"instruments = {{ {instruments} }}")
    lines.append("breaks = [" + ", ".join(breaks) + "]")
    return "\n".join(lines)


# Un chorus fait douze mesures. Cassures : `start` compte à partir de zéro depuis le début du chorus (négatif : depuis la fin).
# Mesure 8 (le VI du retournement) : un break de comping d'une mesure ; les deux dernières mesures : stop-time (retournement).
STOP = break_(-2, 2, 0.5, ["Chords", "Bass"], keep=True)


def default_arc() -> str:
    sparse_med = tags(("sparse", 3), ("medium", 1))
    out = [
        stage("stages", 1, [
            v(3, 0.7, 0.6, sparse_med, tags(("hat", 1))),
            v(1, 0.8, 0.7, tags(("medium", 1)), tags(("hat", 1))),
        ]),
        stage("stages", 2, [
            v(2, comping=tags(("medium", 2), ("dense", 1)), percussion=tags(("ride", 1)),
              breaks=[break_(7, 1, 0.4, ["Chords"])]),
            v(1, 1.15, 1.1, tags(("dense", 1)), tags(("ride", 1)), breaks=[break_(-1, 1, 0.5, ["Chords", "Bass"])]),
        ]),
        stage("stages", 1, [
            v(None, 1.3, 1.3, tags(("dense", 1)), tags(("ride", 1)), breaks=[STOP]),
        ]),
        fall("fall", 0.45, 0.5, tags(("sparse", 1)), tags(("hat", 1)), [break_(-2, 2, None, ["Chords", "Bass"], keep=True)]),
    ]
    return "\n\n".join(out)


ARC_HEADER = """\
# Arc de `gmd_blues_shuffle` sur plusieurs chorus (A3) : valeurs de métier, aucune mesure de corpus (le GMD isole beat et fill,
# WJazzD ne donne pas la dynamique d'un chorus). GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b6/emit_session.py`. Un chorus
# = une passe de la grille de douze mesures ; cinq chorus par air, nombre connu, donc la retombée joue au dernier :
#   1      exposition : comping clairsemé ou médian, batterie à la charleston (`hat`), doux ;
#   2-3    corps : comping médian ou dense, ride ; un break du comping à la mesure 8 (basse et batterie continuent) ou un break du
#          comping et de la basse à la dernière mesure (la batterie seule fait le retournement) ;
#   4      sommet : dense, plus fort, stop-time sur les deux dernières mesures (le retournement) une fois sur deux ;
#   5      retombée : clairsemé, doux, charleston, stop-time sur le un des deux dernières mesures ;
# au-delà (plus de cinq chorus ou set sans fin) : le sommet puis le corps et le sommet en boucle (`repeat_from`), sans retombée.
repeat_from = 1

"""


def named_arcs() -> str:
    parts = []
    # orchestre : instruments du comping et de la basse changent de chorus en chorus.
    parts.append(f"""[arcs.orchestre]
repeat_from = 1

{stage("arcs.orchestre.stages", 1, [v(1, 0.65, 0.9, tags(("sparse", 3), ("medium", 1)), tags(("hat", 1)), "chords_program = 0")])}

{stage("arcs.orchestre.stages", 2, [
        v(2, comping=tags(("medium", 2), ("dense", 1)), percussion=tags(("ride", 1)), instruments="chords_program = 27",
          breaks=[break_(7, 1, 0.4, ["Chords"])]),
        v(1, 1.1, 1.1, tags(("dense", 1)), tags(("ride", 1)), "chords_program = 16")])}

{stage("arcs.orchestre.stages", 1, [v(None, 1.4, 1.4, tags(("dense", 1)), tags(("ride", 1)), "bass_program = 34, chords_program = 29", [STOP])])}

{fall("arcs.orchestre.fall", 0.45, 0.5, tags(("sparse", 1)), tags(("hat", 1)), [break_(-2, 2, None, ["Chords", "Bass"], keep=True)], "chords_program = 0")}""")
    parts.append(f"""[arcs.direct]
repeat_from = 1

{stage("arcs.direct.stages", 2, [
        v(2, 0.9, comping=tags(("medium", 2), ("dense", 1)), percussion=tags(("ride", 1)), breaks=[break_(7, 1, 0.4, ["Chords"])]),
        v(1, 1.0, comping=tags(("medium", 1)), percussion=tags(("ride", 1)))])}

{stage("arcs.direct.stages", 1, [v(None, 1.25, 1.25, tags(("dense", 1)), tags(("ride", 1)), breaks=[break_(-1, 1, 0.5, ["Chords", "Bass"])])])}

{fall("arcs.direct.fall", 0.8, 0.7, tags(("medium", 1)), tags(("ride", 1)), [break_(-1, 1, None, ["Chords", "Bass"], keep=True)])}""")
    parts.append(f"""[arcs.montee]
repeat_from = 1

{stage("arcs.montee.stages", 1, [v(None, 0.55, 0.8, tags(("sparse", 1)), tags(("hat", 1)))])}

{stage("arcs.montee.stages", 1, [v(None, 0.8, 1.0, tags(("medium", 1)), tags(("hat", 1)))])}

{stage("arcs.montee.stages", 1, [v(None, 1.0, 1.1, tags(("medium", 2), ("dense", 1)), tags(("ride", 1)))])}

{stage("arcs.montee.stages", 1, [v(None, 1.3, 1.3, tags(("dense", 1)), tags(("ride", 1)), "bass_program = 34, chords_program = 29")])}

{fall("arcs.montee.fall", 0.45, 0.5, tags(("sparse", 1)), tags(("hat", 1)), [break_(-2, 2, None, ["Chords", "Bass"], keep=True)], "chords_program = 0")}""")
    return "\n\n".join(parts)


SESSION_HEADER = f"""\
# Habillage par graine de `gmd_blues_shuffle` (A2, phase 019, B6) : un tirage par groupe, les groupes se combinent
# ({n_combinations()} habillages possibles). GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b6/emit_session.py`.
#
# Ce qui vient de la source (GMD, CC BY 4.0) : les plages de tempo (72 à 134 BPM, les cinq fichiers ternaires retenus ; quatre
# bandes qui ne dépassent pas ces bornes ; rien dans la source entre 72 et 112 que le fichier de blues lent, les bandes
# `lent` et `medium` interpolent). Le reste est de métier (tonalités courantes du blues, instruments, kits, familles de
# voicings, registre, énergie, arcs nommés) : aucun corpus distribuable ne le mesure (décision F3). Pas de variante de swing :
# les cinq fichiers ternaires donnent 0,63 à 0,70 sans pente avec le tempo (README). Programmes General MIDI (0-based) de
# GeneralUser GS, banque 0 ; kits de la banque 128 : 0 standard, 8 room, 16 power, 32 jazz, 40 balais. Aucune zone Odin (D1).
# Le tempo de l'hôte (`SetBpm`), son swing et son `[fixed_tonality]` gagnent sur la variante.

"""


def main() -> None:
    (PROFILE / "session.toml").write_text(SESSION_HEADER + build_variants() + "\n\n" + named_arcs() + "\n")
    (PROFILE / "arc.toml").write_text(ARC_HEADER + default_arc() + "\n")


if __name__ == "__main__":
    main()
