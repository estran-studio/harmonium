#!/usr/bin/env python3
"""B6 : écrit `profiles/gmd_blues_shuffle/profile.toml` (grille, basse, comping : règles de métier ; mètre, tempos, micro-timing :
GMD). Les motifs de basse et de comping sont écrits ici en clair, le reste est du texte. Idempotent. Lancer après `batterie.py`
(dont `preuves/performance.toml` est recopié) avec n'importe quel Python 3 :

    python3 specs/019-bibliotheque-accompagnements/b6/emit_profile.py
"""
from __future__ import annotations

from pathlib import Path

HERE = Path(__file__).resolve().parent
PROFILE = HERE.parents[2] / "profiles" / "gmd_blues_shuffle"

# Positions d'une mesure de 12 pas (triolets) : huit croches de shuffle, longue (2 pas) puis brève (1 pas) par temps.
EIGHTHS = [0, 2, 3, 5, 6, 8, 9, 11]
DURS = [2, 1, 2, 1, 2, 1, 2, 1]
BEATS = [0, 3, 6, 9]
VEL_STD = [104, 82, 94, 82, 98, 82, 94, 82]


def note(step: int, dur: int, semi: int, vel: int, octave: int = 0, off: int = 0) -> str:
    extra = f", octave = {octave}" if octave else ""
    return f"{{ step = {step + off}, duration = {dur}, semitones = {semi}, velocity = {vel}{extra} }}"


def eighths(semis: list[int], vels: list[int] = VEL_STD, off: int = 0, octaves: list[int] | None = None) -> list[str]:
    octaves = octaves or [0] * 8
    return [note(s, d, a, v, o, off) for s, d, a, v, o in zip(EIGHTHS, DURS, semis, vels, octaves)]


def motif(comment: str, weight: int, bars: int, notes: list[str]) -> str:
    body = ",\n    ".join(notes)
    return f'  # {comment}\n  {{ bars = {bars}, weight = {weight}, reference = "chord_root", notes = [\n    {body},\n  ] }},'


BASS = [
    motif("1. Boogie : fondamentale, quinte, sixte, quinte (croches ternaires, longue-brève).", 5, 1,
          eighths([0, 7, 9, 7, 0, 7, 9, 7])),
    motif("2. Montée : fondamentale, quinte, sixte, septième mineure, chacune en paire longue-brève.", 3, 1,
          eighths([0, 0, 7, 7, 9, 9, 10, 10], [104, 80, 94, 80, 96, 80, 98, 84])),
    motif("3. Pompe : fondamentale, l'octave en contretemps des temps 2 et 4.", 3, 1,
          eighths([0] * 8, [104, 80, 90, 80, 98, 80, 90, 80], octaves=[0, 0, 1, 0, 0, 0, 1, 0])),
    motif("4. Marche en noires : fondamentale, quinte, sixte, quinte.", 3, 1,
          [note(s, 3, a, v) for s, a, v in zip(BEATS, [0, 7, 9, 7], [102, 88, 92, 88])]),
    motif("5. Fondamentale et quinte en alternance (rock'n'roll).", 3, 1,
          eighths([0, 7, 0, 7, 0, 7, 0, 7], [104, 82, 92, 82, 98, 82, 92, 82])),
    motif("6. Deux mesures : boogie, puis boogie qui monte à la septième mineure et à la sixte vers l'accord suivant.", 3, 2,
          eighths([0, 7, 9, 7, 0, 7, 9, 7]) + eighths([0, 7, 9, 7, 0, 7, 10, 9], off=12)),
    motif("7. Deux mesures : montée, puis redescente par la sixte et la quinte (la deuxième mesure commence sur la fondamentale).", 2, 2,
          eighths([0, 0, 7, 7, 9, 9, 10, 10], [104, 80, 94, 80, 96, 80, 98, 84])
          + eighths([0, 0, 9, 9, 7, 7, 0, 0], [100, 80, 92, 80, 96, 80, 94, 80], off=12)),
    motif("8. Triolets de fondamentale (blues lent, 12/8 : douze notes par mesure).", 2, 1,
          [note(s, 1, 0, 100 if s % 3 == 0 else 70 if s % 3 == 1 else 76) for s in range(12)]),
    motif("9. Tenue : une blanche par demi-mesure (chorus clairsemé, changement d'accord au temps 3).", 2, 1,
          [note(0, 6, 0, 98), note(6, 6, 0, 90)]),
]


def hit(step: int, dur: int, vel: int, direction: str | None = None, extra: str = "") -> str:
    d = f', direction = "{direction}"' if direction else ""
    return f"{{ step = {step}, duration = {dur}, velocity = {vel}{d}{extra} }}"


def comp(weight: int, tag: str, hits: list[str], comment: str) -> str:
    inner = ", ".join(hits) if len(hits) <= 5 else "\n    " + ",\n    ".join(hits) + ",\n  "
    return f'  # {comment}\n  {{ weight = {weight}, tag = "{tag}", hits = [{inner}] }},'


PAIRS = lambda vl, vs, dl="down", ds="up": [  # noqa: E731  (longue puis brève, quatre temps)
    h for b, (a, c) in zip(BEATS, zip(vl, vs)) for h in (hit(b, 2, a, dl), hit(b + 2, 1, c, ds))]

COMPING = [
    # 0-3 clairsemé
    comp(4, "sparse", [hit(0, 11, 70, "block")], "0. Accord tenu toute la mesure."),
    comp(3, "sparse", [hit(0, 5, 72, "block"), hit(6, 5, 64, "block")], "1. Deux tenues, une par demi-mesure."),
    comp(3, "sparse", [hit(0, 6, 74, "block"), hit(9, 2, 62, "down"), hit(11, 1, 66, "up", ", anticipate = true")],
         "2. Tenue puis relance du temps 4, la dernière note annonce l'accord suivant."),
    comp(1, "sparse", [], "3. Silence."),
    # 4-7 médian
    comp(4, "medium", [hit(0, 2, 78, "down"), hit(3, 2, 70, "down"), hit(6, 2, 76, "down"), hit(9, 2, 70, "down")],
         "4. Une frappe par temps (blues lent)."),
    comp(3, "medium", [hit(3, 2, 80, "down"), hit(5, 1, 60, "up"), hit(9, 2, 78, "down"), hit(11, 1, 60, "up", ", anticipate = true")],
         "5. Contretemps des temps 2 et 4, longue-brève."),
    comp(3, "medium", [hit(2, 1, 68, "up"), hit(5, 1, 62, "up"), hit(8, 1, 68, "up"), hit(11, 1, 62, "up")],
         "6. Contretemps de chaque temps (la brève du shuffle)."),
    comp(2, "medium", [hit(0, 3, 78, "block"), hit(3, 2, 68, "down"), hit(6, 3, 76, "block"), hit(9, 2, 68, "down"), hit(11, 1, 62, "up", ", anticipate = true")],
         "7. Accords sur les temps 1 et 3, rebond sur 2 et 4."),
    # 8-11 dense
    comp(4, "dense", PAIRS([82, 70, 80, 70], [62, 58, 62, 58]), "8. Paires longue-brève à chaque temps (shuffle plein)."),
    comp(3, "dense", PAIRS([84, 68, 80, 68], [64, 56, 62, 56], "down", "up")[:-1]
         + [hit(11, 1, 58, "up", ", probability = 0.6")], "9. Shuffle, dernière brève probabiliste."),
    comp(3, "dense", [hit(0, 2, 80, "down"), hit(2, 1, 60, "up"), hit(3, 2, 92, "down"), hit(5, 1, 60, "up"),
                      hit(6, 2, 78, "down"), hit(8, 1, 60, "up"), hit(9, 2, 92, "down"), hit(11, 1, 60, "up")],
         "10. Shuffle, accents du backbeat (temps 2 et 4)."),
    comp(2, "dense", [hit(s, 1, 76 if s % 3 == 0 else 54 if s % 3 == 1 else 62, "down" if s % 3 != 1 else "up",
                          ", probability = 0.8" if s % 3 == 1 else "") for s in range(12)],
         "11. Triolets de piano (douze frappes, celles du milieu à 80 %)."),
]

PHRASES = [
    (3, "sparse", [0, 0, 1, 0], ""), (3, "sparse", [1, 1, 2, 1], ""), (2, "sparse", [0, 3], ""), (2, "sparse", [2, 2], ""),
    (3, "sparse", [1, 1, 2, 0], "part_end"),
    (3, "medium", [4, 4, 5, 4], ""), (3, "medium", [5, 6, 5, 6], ""), (2, "medium", [4, 7, 4, 7], ""), (2, "medium", [6, 6], ""),
    (2, "medium", [4, 5], ""), (3, "medium", [4, 4, 5, 7], "part_end"),
    (3, "dense", [8, 8, 9, 8], ""), (3, "dense", [9, 10, 9, 10], ""), (2, "dense", [8, 11, 8, 11], ""), (2, "dense", [10, 10], ""),
    (2, "dense", [8, 9], ""), (3, "dense", [8, 8, 9, 11], "part_end"),
]


def phrases() -> str:
    rows = []
    for w, tag, motifs, at in PHRASES:
        a = f', at = "{at}"' if at else ""
        rows.append(f'  {{ weight = {w}, tag = "{tag}"{a}, motifs = {motifs} }},')
    return "\n".join(rows)


def chords_loop(weight: int, comment: str, chords: list[tuple[int, str, int]]) -> str:
    assert sum(s for *_, s in chords) == 144, comment
    rows = "\n".join(f'    {{ semitones = {a}, quality = "{q}", steps = {s} }},' for a, q, s in chords)
    return f"  # {comment}\n  {{ weight = {weight}, chords = [\n{rows}\n  ] }},"


LOOPS = [
    chords_loop(5, "Blues de douze mesures, forme commune : I7 (4), IV7 (2), I7 (2), V7, IV7, I7, V7 (retournement : la dernière mesure sur le V).",
                [(0, "7", 48), (5, "7", 24), (0, "7", 24), (7, "7", 12), (5, "7", 12), (0, "7", 12), (7, "7", 12)]),
    chords_loop(5, "Blues à changement rapide (quick change) : IV7 dès la deuxième mesure, V7 IV7 I7 V7 aux mesures 9 à 12.",
                [(0, "7", 12), (5, "7", 12), (0, "7", 24), (5, "7", 24), (0, "7", 24), (7, "7", 12), (5, "7", 12), (0, "7", 12), (7, "7", 12)]),
    chords_loop(3, "Blues à retournement I-VI-ii-V : VI7 à la mesure 8, ii7, V7, I7, V7 aux mesures 9 à 12.",
                [(0, "7", 12), (5, "7", 12), (0, "7", 24), (5, "7", 24), (0, "7", 12), (9, "7", 12), (2, "m7", 12), (7, "7", 12), (0, "7", 12), (7, "7", 12)]),
    chords_loop(2, "Blues mineur : Im7 (4), IVm7 (2), Im7 (2), bVI7, V7, Im7, V7.",
                [(0, "m7", 48), (5, "m7", 24), (0, "m7", 24), (8, "7", 12), (7, "7", 12), (0, "m7", 12), (7, "7", 12)]),
    chords_loop(2, "Blues qui se pose : comme la forme commune, mais les mesures 11 et 12 restent sur I7 (fin de chanson, sans retournement).",
                [(0, "7", 48), (5, "7", 24), (0, "7", 24), (7, "7", 12), (5, "7", 12), (0, "7", 24)]),
]

KEYS = [(4, 3), (9, 3), (7, 2), (0, 2), (10, 2), (2, 1)]  # mi, la, sol, do, si♭, ré (poids = tonalités courantes du blues)

TEMPLATE = (HERE / "profile.toml.tmpl").read_text()


def main() -> None:
    performance = (HERE / "preuves" / "performance.toml").read_text().rstrip("\n")
    text = TEMPLATE
    text = text.replace("@@KEY_MODES@@", "[" + ", ".join(f'{{ key = {k}, mode = "mixolydian", weight = {w} }}' for k, w in KEYS) + "]")
    text = text.replace("@@LOOPS@@", "\n".join(LOOPS))
    text = text.replace("@@BASS@@", "\n".join(BASS))
    text = text.replace("@@COMPING@@", "\n".join(COMPING))
    text = text.replace("@@PHRASES@@", phrases())
    text = text.replace("@@PERFORMANCE@@", performance)
    (PROFILE / "profile.toml").write_text(text)


if __name__ == "__main__":
    main()
