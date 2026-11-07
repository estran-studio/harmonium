#!/usr/bin/env python3
"""Q0 (phase 020) : corrections rapides des profils d'accompagnement, appliquées APRÈS les générateurs de 019.

Les profils `gmd_*` et `wjazzd_*` sont écrits par des générateurs (`specs/019-*/b8/generer.py`, `b47_*`, `b3/*`, `b25_*`, `b6/*`,
`b1/*`). Q0 n'y touche pas : ce script part des fichiers de `main` à `BASE` (`git show BASE:chemin`), applique les points 1, 2 et 4
du brief et écrit le résultat dans `profiles/<id>/` ; il est idempotent (même sortie à chaque lancement). Le point 3 (groove de
batterie) est `q0/batterie.py`, qui lit `PARAMS` plus bas via `q0/batterie_params.json`.

    python3 specs/020-qualite-des-styles/q0/appliquer.py [--profil <id>] [--out <dossier racine>]
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

BASE = "448f996"
REPO = Path(__file__).resolve().parents[3]
E1 = ["wjazzd_swing", "wjazzd_ballad", "gmd_jazz_waltz", "gmd_latin_brazilian_bossa", "gmd_latin_brazilian_samba",
      "gmd_latin_brazilian_baiao", "gmd_afrocuban", "gmd_funk", "gmd_soul", "gmd_reggae", "gmd_rock", "gmd_pop",
      "gmd_blues_shuffle"]
#: Profils de session à feel droit seul (point 1) : hors E1, `gmd_hiphop` porte la même variante.
SANS_SWING_LEGER = ["gmd_rock", "gmd_hiphop"]

# --------------------------------------------------------------------------------------------------------------------
# Point 2 : accents du comping et de la basse, par style.
#
# Classe d'un pas dans la pulsation (`spb` pas par pulsation : 4 en 16 pas, 3 en 12 pas ternaire) :
#   b1 premier temps · b3 troisième temps · bk contretemps 2 et 4 (backbeat) · and croche de contretemps · e, a doubles croches
#   (ternaire : and = deuxième triolet, e = triolet du milieu) · ant attaque anticipée (`anticipate`, ou pas 14 en 16 pas).
# Un coup de probabilité < 1 est une note fantôme : sa vélocité est plafonnée à `ghost`. Une corde montante (`up`) perd `up`.
# Les valeurs sont des vélocités MIDI avant l'intensité et la dynamique du moteur (± 4 de jitter et 4 de pente de strum).

COMPING = {
    # Jazz : charleston (and du 2, anticipation), temps légers, notes fantômes très douces.
    "wjazzd_swing": dict(b1=76, b3=62, bk=58, and_=64, e=46, a=46, ant=88, s={6: 92}, ghost=44, up=0),
    "wjazzd_ballad": dict(b1=80, b3=60, bk=54, and_=68, e=44, a=44, ant=72, s={6: 78}, ghost=42, up=0),
    "gmd_jazz_waltz": dict(b1=76, b3=60, bk=58, and_=66, e=46, a=46, ant=84, s={}, ghost=44, up=0),
    # Latin
    "gmd_latin_brazilian_bossa": dict(b1=84, b3=70, bk=68, and_=64, e=50, a=54, ant=92, s={6: 92, 14: 92}, ghost=48, up=6),
    "gmd_latin_brazilian_samba": dict(b1=80, b3=82, bk=98, and_=68, e=48, a=88, ant=90, s={}, ghost=46, up=4),
    "gmd_latin_brazilian_baiao": dict(b1=96, b3=84, bk=70, and_=70, e=50, a=56, ant=92, s={6: 90}, ghost=48, up=0),
    "gmd_afrocuban": dict(b1=84, b3=74, bk=74, and_=88, e=60, a=60, ant=100, s={6: 98, 14: 100, 2: 80}, ghost=54, up=0),
    # Groove
    "gmd_funk": dict(b1=112, b3=88, bk=78, and_=70, e=44, a=64, ant=98, s={}, ghost=44, up=6),
    "gmd_soul": dict(b1=100, b3=82, bk=84, and_=72, e=44, a=70, ant=96, s={}, ghost=44, up=4),
    "gmd_reggae": dict(b1=60, b3=64, bk=66, and_=84, e=50, a=54, ant=96, s={6: 104, 14: 104}, ghost=48, up=0),
    # Rock / pop
    "gmd_rock": dict(b1=106, b3=96, bk=88, and_=72, e=58, a=58, ant=100, s={}, ghost=46, up=8),
    "gmd_pop": dict(b1=98, b3=86, bk=78, and_=66, e=52, a=52, ant=92, s={}, ghost=46, up=6),
    "gmd_blues_shuffle": dict(b1=96, b3=84, bk=92, and_=66, e=48, a=48, ant=76, s={}, ghost=46, up=6),
    "gmd_hiphop": None,
}
# Basse : accent du premier temps et des temps, croches plus légères, notes mortes aux doubles croches.
BASS = {
    "gmd_latin_brazilian_bossa": dict(b1=102, b3=92, bk=86, and_=84, e=60, a=60, ant=88, s={6: 86}),
    "gmd_latin_brazilian_samba": dict(b1=90, b3=88, bk=106, and_=76, e=58, a=66, ant=82, s={}),
    "gmd_latin_brazilian_baiao": dict(b1=104, b3=96, bk=88, and_=82, e=58, a=62, ant=88, s={6: 90}),
    "gmd_afrocuban": dict(b1=84, b3=76, bk=84, and_=84, e=60, a=60, ant=108, s={6: 102, 12: 108}),
    "gmd_funk": dict(b1=114, b3=94, bk=88, and_=80, e=50, a=82, ant=98, s={}),
    "gmd_soul": dict(b1=104, b3=90, bk=86, and_=74, e=56, a=78, ant=92, s={}),
    "gmd_reggae": dict(b1=92, b3=106, bk=88, and_=78, e=66, a=66, ant=92, s={}),
    "gmd_rock": dict(b1=108, b3=98, bk=94, and_=80, e=70, a=70, ant=100, s={}),
    "gmd_pop": dict(b1=102, b3=92, bk=88, and_=76, e=66, a=66, ant=92, s={}),
    "gmd_blues_shuffle": dict(b1=110, b3=96, bk=94, and_=76, e=64, a=64, ant=88, s={}),
}
# Basse en marche (jazz) : vélocité de base et surcroît des arrivées ; les croches fantômes gardent leur vélocité.
# Marche : accents d'arrivée, et courbe de phrase pour la ballade et la valse à une note
# par mesure (une vélocité unique par note de la marche ne peut nuancer autrement le deux-temps).
WALKING = {
    "wjazzd_swing": dict(velocity=72, accent=20),
    "wjazzd_ballad": dict(velocity=66, accent=20),
    "gmd_jazz_waltz": dict(velocity=72, accent=20),
}
VEL_MIN, VEL_MAX = 32, 118  # plage audible : voir le compte rendu (modulateur de vélocité SF2 : -40 log10(v/127) dB)

# Les variantes clairsemées n'atteignent pas 12 malgré les accents de position : élargissement
# des seules tables concernées, sans modifier les notes déjà fantômes du funk et du shuffle.
COMPING_BOOST = set(E1) - {"gmd_funk", "gmd_reggae", "gmd_blues_shuffle"}
BASS_BOOST = {"gmd_latin_brazilian_bossa", "gmd_latin_brazilian_samba", "gmd_latin_brazilian_baiao",
              "gmd_soul", "gmd_reggae", "gmd_rock", "gmd_pop"}

def boost_table(rule: dict, factor: float) -> None:
    for k in ("b1", "b3", "bk", "and_", "e", "a", "ant", "ghost"):
        if k in rule:
            rule[k] = max(VEL_MIN, min(112, round(70 + (rule[k] - 70) * factor)))
    for s, v in rule["s"].items():
        rule["s"][s] = max(VEL_MIN, min(112, round(70 + (v - 70) * factor)))

for _pid in COMPING_BOOST:
    boost_table(COMPING[_pid], 1.42)
for _pid in BASS_BOOST:
    boost_table(BASS[_pid], 1.45)


def classify(step: int, spb: int, anticipate: bool, steps_per_bar: int) -> str:
    if anticipate or step == steps_per_bar - 2:
        return "ant"
    r = step % spb
    if r == 0:
        return "b1" if step == 0 else ("bk" if (step // spb) % 2 == 1 else "b3")
    if spb == 4:
        return {1: "e", 2: "and_", 3: "a"}[r]
    return "and_" if r == 2 else "e"


def vel(rule: dict, step: int, spb: int, spbar: int, *, anticipate=False, prob=1.0, up=False) -> int:
    cls = classify(step, spb, anticipate, spbar)
    v = rule["s"].get(step) or rule[cls]
    if prob < 1.0:
        v = min(v, rule.get("ghost", v))
    if up and rule.get("up"):
        v -= rule["up"]
    return max(VEL_MIN, min(VEL_MAX, v))


def git_show(rel: str) -> str:
    return subprocess.run(["git", "show", f"{BASE}:{rel}"], cwd=REPO, check=True, capture_output=True, text=True).stdout


HIT = re.compile(r"\{ step = (\d+), duration = (\d+), velocity = (\d+)([^}]*)\}")
BASS_NOTE = re.compile(r"\{ step = (\d+), duration = (\d+), (semitones = -?\d+, )?velocity = (\d+)([^}]*)\}")
BASS_TPL = re.compile(r"\{ step = (\d+), duration = (\d+), velocity = (\d+) \}")


def region(text: str, start: str, ends: tuple[str, ...]) -> tuple[int, int]:
    a = text.index(start)
    b = len(text)
    for e in ends:
        m = re.compile(e, re.M).search(text, a + len(start))
        if m:
            b = min(b, m.start())
    return a, b


def retouch_comping(pid: str, text: str, spb: int, spbar: int) -> str:
    rule = COMPING[pid]
    a, b = region(text, "[accompaniment.chords]", (r"^\[accompaniment\.chords\.", r"^\[\[accompaniment\.chords\.", r"^\[instruments\]"))

    def sub(m: re.Match) -> str:
        step, dur, _v, rest = int(m[1]), m[2], m[3], m[4]
        prob = re.search(r"probability = ([0-9.]+)", rest)
        v = vel(rule, step, spb, spbar, anticipate="anticipate = true" in rest,
                prob=float(prob[1]) if prob else 1.0, up='direction = "up"' in rest)
        return f"{{ step = {step}, duration = {dur}, velocity = {v}{rest}}}"

    return text[:a] + HIT.sub(sub, text[a:b]) + text[b:]


def retouch_bass(pid: str, text: str, spb: int, spbar: int) -> str:
    rule = BASS[pid]
    a, b = region(text, "[accompaniment.bass]", (r"^\[accompaniment\.chords\]",))
    seg = text[a:b]

    def note(m: re.Match) -> str:
        step = int(m[1])
        v = vel(rule, step % spbar, spb, spbar)
        return f"{{ step = {step}, duration = {m[2]}, {m[3] or ''}velocity = {v}{m[5]}}}"

    def tpl(m: re.Match) -> str:
        return f"{{ step = {m[1]}, duration = {m[2]}, velocity = {vel(rule, int(m[1]) % spbar, spb, spbar)} }}"

    out = []
    for line in seg.splitlines(keepends=True):
        s = line.lstrip()
        if s.startswith("rhythm = "):
            line = BASS_TPL.sub(tpl, line)
        elif s.startswith("{") or s.startswith("motifs") or "notes = [" in line:
            line = BASS_NOTE.sub(note, line)
        if "pickup = {" in line:
            line = re.sub(r"(pickup = \{[^}]*velocity = )(\d+)", lambda m: f"{m[1]}{rule['ant']}", line)
        out.append(line)
    return text[:a] + "".join(out) + text[b:]


def retouch_walking(pid: str, text: str) -> str:
    w = WALKING[pid]
    text = re.sub(r"(\[accompaniment\.bass\.walking\]\nvelocity = )\d+(\naccent = )\d+",
                  lambda m: f"{m[1]}{w['velocity']}{m[2]}{w['accent']}", text, count=1)
    return text


# --------------------------------------------------------------------------------------------------------------------
# Point 4 : mise en place. La basse suit la grosse caisse, le comping suit la caisse claire et le timekeeper (moyenne des
# décalages par voix de la batterie), pas par pas ; gigue réduite ; jazz : pas de décalage systématique.

BASS_JITTER, CHORDS_JITTER = 0.02, 0.03
COMPING_CORRECTION = {
    "gmd_rock": 0.08,                    # première corde du strum ~11 ms avant la caisse claire
    "gmd_latin_brazilian_bossa": 0.06,   # première corde du strum ~12 ms avant la cross-stick
}
JAZZ_COMPING_LAG = 0.0
OFFSET_BASS_LAG = 0.0  # la basse sur la grosse caisse, sans retard systématique


def voices(profile: dict) -> dict[int, tuple[float, float]]:
    out: dict[int, tuple[float, float]] = {}
    for track in profile.get("performance", {}).get("tracks", {}).values():
        for v in track.get("voices", []):
            on = v.get("on_beat", 0.0)
            out[v["note"]] = (on, v.get("off_beat", on))
    return out


def clamp(x: float) -> float:
    return round(max(-0.2, min(0.2, x)), 4)


def placement(pid: str, profile: dict) -> tuple[list[float], list[float]]:
    spbar = profile["meter"]["steps_per_bar"]
    pulse = profile["meter"]["steps_per_pulse"]
    v = voices(profile)
    kick = v.get(36)
    snare = v.get(38) or v.get(37)
    tk = next((v[n] for n in (42, 51, 44) if n in v), None)
    bass, chords = [], []
    for s in range(spbar):
        i = 0 if s % pulse == 0 else 1
        bass.append(clamp(kick[i] + OFFSET_BASS_LAG) if kick else 0.0)
        refs = [x[i] for x in (snare, tk) if x]
        chords.append(clamp((sum(refs) / len(refs) if refs else JAZZ_COMPING_LAG)
                            + COMPING_CORRECTION.get(pid, 0.0)))
    return bass, chords


def retouch_performance(pid: str, text: str, profile: dict) -> str:
    bass, chords = placement(pid, profile)
    fmt = lambda xs: "[" + ", ".join(f"{x:g}" for x in xs) + "]"
    if pid in WALKING:
        phrase = "phrase = [[0.0, -18.0], [0.5, 16.0], [1.0, -18.0]]\n"
    elif pid in {"gmd_latin_brazilian_bossa", "gmd_reggae", "gmd_pop", "gmd_soul"}:
        phrase = "phrase = [[0.0, -16.0], [0.5, 16.0], [1.0, -16.0]]\n"
    else:
        phrase = ""
    bound = "velocity_min = 32\nvelocity_max = 120\n"
    new_bass = f"[performance.tracks.Bass]\noffsets = {fmt(bass)}\njitter = {BASS_JITTER}\n{phrase}{bound}" if any(bass) else f"[performance.tracks.Bass]\njitter = {BASS_JITTER}\n{phrase}{bound}"
    new_chords = f"[performance.tracks.Chords]\noffsets = {fmt(chords)}\njitter = {CHORDS_JITTER}\n{bound}"
    text, n1 = re.subn(r"\[performance\.tracks\.Bass\]\n(?:(?!\[)[^\n]*\n)*?(?=\n|\[|\Z)", lambda m: new_bass, text, count=1)
    text, n2 = re.subn(r"\[performance\.tracks\.Chords\]\n(?:(?!\[)[^\n]*\n)*?(?=\n|\[|\Z)", lambda m: new_chords, text, count=1)
    assert n1 == n2 == 1, (pid, n1, n2)
    return text


# --------------------------------------------------------------------------------------------------------------------


def swing_leger(text: str) -> str:
    """Retire la variante `swing_leger` et le groupe `feel` qui n'a plus qu'un seul choix (feel droit : celui du profil)."""
    out = re.sub(r"\[\[variants\]\]\ngroup = \"feel\"\nname = \"(?:droit|swing_leger)\"\n(?:[a-z_]+ = [^\n]*\n)*\n", "", text)
    assert 'group = "feel"' not in out
    return out


def build(pid: str, out_root: Path) -> None:
    rel = f"profiles/{pid}"
    out = out_root / rel
    out.mkdir(parents=True, exist_ok=True)
    prof_text = git_show(f"{rel}/profile.toml")
    profile = tomllib.loads(prof_text)
    spbar = profile["meter"]["steps_per_bar"]
    num = profile["meter"]["time_signature"]["numerator"]
    spb = spbar // num
    if pid in COMPING and COMPING[pid]:
        prof_text = retouch_comping(pid, prof_text, spb, spbar)
    if pid in BASS:
        prof_text = retouch_bass(pid, prof_text, spb, spbar)
    if pid in WALKING:
        prof_text = retouch_walking(pid, prof_text)
    if pid in E1:
        prof_text = retouch_performance(pid, prof_text, profile)
    (out / "profile.toml").write_text(prof_text)
    if pid in SANS_SWING_LEGER:
        (out / "session.toml").write_text(swing_leger(git_show(f"{rel}/session.toml")))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--profil", action="append")
    ap.add_argument("--out", type=Path, default=REPO)
    args = ap.parse_args()
    for pid in args.profil or sorted(set(E1) | set(SANS_SWING_LEGER)):
        build(pid, args.out)
        print("ok", pid)
    return 0


if __name__ == "__main__":
    sys.exit(main())
