#!/usr/bin/env python3
"""Q0 (phase 020) : corrections rapides des profils d'accompagnement, appliquées APRÈS les générateurs de 019.

Les profils `gmd_*` et `wjazzd_*` sont écrits par des générateurs (`specs/019-*/b8/generer.py`, `b47_*`, `b3/*`, `b25_*`, `b6/*`,
`b1/*`). Q0 n'y touche pas : ce script part des fichiers de `main` à `BASE` (`git show BASE:chemin`), applique les points 1, 2 et 4
du brief et écrit le résultat dans `profiles/<id>/` ; il est idempotent (même sortie à chaque lancement). Le point 3 (groove de
batterie) est `q0/batterie.py`.

    python3 specs/020-qualite-des-styles/q0/appliquer.py [--profil <id>] [--out <dossier racine>]
"""
from __future__ import annotations

import argparse
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
# Point 2 : accents et notes fantômes, par style.
#
# Chaque coup garde la vélocité que son générateur lui a donnée (niveau du style, calé au mixage) ; Q0 y ajoute un écart
# selon la place du coup dans la mesure. Une forme (`SHAPES`) donne, par classe de place, un coefficient entre -1 et +1 ;
# l'amplitude (`AMPLITUDE`, en vélocités MIDI) le met à l'échelle. Les coups probabilistes (notes fantômes) reçoivent en plus
# `ghost` (coefficient, négatif). Classes (`spb` pas par pulsation : 4 en 16 pas, 3 en 12 pas ternaires) :
#   b1 premier temps · b3 troisième temps · bk contretemps 2 et 4 (backbeat) · and croche de contretemps (ternaire : dernier
#   triolet) · e, a doubles croches (ternaire : triolet du milieu) · ant attaque anticipée (`anticipate`, avant-dernière croche)
# `s` : pas précis (syncopes, clave) qui l'emportent sur la classe.
# Valse (3/4, 12 pas, `spb` = 4) : `bk` est le temps 2, `b3` le temps 3.

SHAPES = {
    # Jazz : charleston (et du 2, anticipation), temps légers, notes fantômes très douces.
    "jazz_comp": dict(b1=-0.3, b3=-0.5, bk=-0.4, and_=0.5, e=-0.8, a=-0.8, ant=1.0, s={6: 1.0}, ghost=-0.6),
    "ballade_comp": dict(b1=-0.2, b3=-0.5, bk=-0.5, and_=0.6, e=-0.8, a=-0.8, ant=1.0, s={6: 1.0}, ghost=-0.6),
    "valse_comp": dict(b1=-0.6, b3=-0.8, bk=1.0, and_=0.0, e=-0.8, a=-0.8, ant=0.2, s={}, ghost=-0.6),
    # Latin : bossa (syncope pas 6 et 14), samba (surdo : accent des 2/4, tamborim), baião (zabumba), afro-cubain (clave)
    "bossa_comp": dict(b1=0.4, b3=0.0, bk=-0.1, and_=-0.2, e=-0.6, a=-0.4, ant=0.8, s={6: 1.0, 14: 1.0}, ghost=-0.7),
    "samba_comp": dict(b1=-0.2, b3=0.3, bk=0.9, and_=-0.2, e=-0.7, a=0.6, ant=0.7, s={}, ghost=-0.7),
    "baiao_comp": dict(b1=0.9, b3=0.4, bk=-0.1, and_=0.0, e=-0.6, a=-0.2, ant=0.8, s={6: 0.7}, ghost=-0.7),
    "afrocubain_comp": dict(b1=0.0, b3=0.0, bk=0.0, and_=0.5, e=-0.6, a=-0.4, ant=0.9, s={6: 0.9, 14: 1.0, 2: 0.3}, ghost=-0.7),
    # Groove
    "funk_comp": dict(b1=1.0, b3=0.3, bk=0.2, and_=-0.2, e=-0.8, a=0.3, ant=0.7, s={}, ghost=-0.9),
    "soul_comp": dict(b1=0.8, b3=0.3, bk=0.5, and_=-0.2, e=-0.7, a=0.1, ant=0.7, s={}, ghost=-0.9),
    "reggae_comp": dict(b1=-0.8, b3=-0.5, bk=-0.4, and_=1.0, e=-0.8, a=-0.6, ant=0.8, s={6: 1.0, 14: 1.0}, ghost=-0.8),
    # Rock / pop / shuffle
    "rock_comp": dict(b1=0.9, b3=0.5, bk=0.7, and_=-0.4, e=-0.9, a=-0.9, ant=0.8, s={}, ghost=-0.9),
    "pop_comp": dict(b1=0.8, b3=0.4, bk=0.5, and_=-0.4, e=-0.8, a=-0.8, ant=0.7, s={}, ghost=-0.9),
    "shuffle_comp": dict(b1=0.9, b3=0.5, bk=0.8, and_=0.0, e=-0.9, a=-0.9, ant=0.5, s={}, ghost=-0.9),
    # Basse : racine accentuée au temps 1, reprises de croche plus légères, notes mortes aux doubles croches.
    "rock_bass": dict(b1=1.0, b3=0.5, bk=0.2, and_=-0.4, e=-0.9, a=-0.9, ant=0.5, s={}, ghost=-0.9),
    "pop_bass": dict(b1=0.9, b3=0.5, bk=0.2, and_=-0.4, e=-0.8, a=-0.8, ant=0.5, s={}, ghost=-0.9),
    "soul_bass": dict(b1=1.0, b3=0.3, bk=0.1, and_=-0.4, e=-0.9, a=0.3, ant=0.5, s={}, ghost=-0.9),
    "reggae_bass": dict(b1=0.5, b3=1.0, bk=0.2, and_=-0.2, e=-0.6, a=-0.6, ant=0.4, s={}, ghost=-0.8),
    "bossa_bass": dict(b1=1.0, b3=0.4, bk=0.1, and_=-0.2, e=-0.7, a=-0.7, ant=0.6, s={6: 0.4}, ghost=-0.8),
    "samba_bass": dict(b1=-0.3, b3=-0.1, bk=1.0, and_=-0.2, e=-0.8, a=-0.8, ant=0.4, s={}, ghost=-0.8),
    "afrocubain_bass": dict(b1=0.0, b3=0.0, bk=-0.3, and_=0.4, e=-0.6, a=-0.6, ant=1.0, s={6: 0.7, 12: 0.7}, ghost=-0.8),
    "baiao_bass": dict(b1=1.0, b3=0.5, bk=0.0, and_=0.0, e=-0.7, a=-0.5, ant=0.6, s={6: 0.4}, ghost=-0.8),
}

#: (forme du comping, amplitude) et (forme de la basse, amplitude) ; `None` : la basse a déjà des accents (écart-type ≥ 8).
AMPLITUDE = {
    "wjazzd_swing": (("jazz_comp", 28, -10), None),
    "wjazzd_ballad": (("ballade_comp", 24, -1), None),
    "gmd_jazz_waltz": (("valse_comp", 20, -2), None),
    "gmd_latin_brazilian_bossa": (("bossa_comp", 28, -7), ("bossa_bass", 17, -10)),
    "gmd_latin_brazilian_samba": (("samba_comp", 36, -11), ("samba_bass", 28, -2)),
    "gmd_latin_brazilian_baiao": (("baiao_comp", 28, -14), ("baiao_bass", 14, -6)),
    "gmd_afrocuban": (("afrocubain_comp", 28, -8), ("afrocubain_bass", 14, -6)),
    "gmd_funk": (("funk_comp", 20, -3), None),
    "gmd_soul": (("soul_comp", 26, -8), ("soul_bass", 14, -3)),
    "gmd_reggae": (("reggae_comp", 22, -15), ("reggae_bass", 18, -6)),
    "gmd_rock": (("rock_comp", 24, -11), ("rock_bass", 15, -3)),
    "gmd_pop": (("pop_comp", 30, -10), ("pop_bass", 19, -6)),
    "gmd_blues_shuffle": (("shuffle_comp", 20, -8), None),
}
# Basse en marche (jazz) : la vélocité de base de la marche et le surcroît des arrivées (`accent`) sont dans `[accompaniment.bass.walking]` ;
# une courbe de phrase (A6 : `phrase`, delta de vélocité selon la phase de la phrase) nuance le deux-temps de la ballade et de la valse.
WALKING = {
    "wjazzd_swing": dict(velocity=72, accent=20),
    "wjazzd_ballad": dict(velocity=61, accent=20),
    "gmd_jazz_waltz": dict(velocity=66, accent=20),
}
PHRASE = {
    "wjazzd_swing": 18, "wjazzd_ballad": 18, "gmd_jazz_waltz": 18,
}
VEL_MIN, VEL_MAX = 32, 118  # plage audible : voir le compte rendu (modulateur de vélocité SF2 : -40 log10(v/127) dB)


def classify(step: int, spb: int, anticipate: bool, steps_per_bar: int) -> str:
    if anticipate or step == steps_per_bar - 2:
        return "ant"
    r = step % spb
    if r == 0:
        return "b1" if step == 0 else ("bk" if (step // spb) % 2 == 1 else "b3")
    if spb == 4:
        return {1: "e", 2: "and_", 3: "a"}[r]
    return "and_" if r == 2 else "e"


def vel(shape: dict, amp: float, bias: float, base: int, step: int, spb: int, spbar: int, *, anticipate=False, prob=1.0) -> int:
    coef = shape["s"].get(step)
    if coef is None:
        coef = shape[classify(step, spb, anticipate, spbar)]
    if prob < 1.0:
        coef += shape["ghost"]
    return max(VEL_MIN, min(VEL_MAX, round(base + bias + amp * coef)))


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
    name, amp, bias = AMPLITUDE[pid][0]
    shape = SHAPES[name]
    a, b = region(text, "[accompaniment.chords]", (r"^\[accompaniment\.chords\.", r"^\[\[accompaniment\.chords\.", r"^\[instruments\]"))

    def sub(m: re.Match) -> str:
        step, dur, base, rest = int(m[1]), m[2], int(m[3]), m[4]
        prob = re.search(r"probability = ([0-9.]+)", rest)
        v = vel(shape, amp, bias, base, step, spb, spbar, anticipate="anticipate = true" in rest,
                prob=float(prob[1]) if prob else 1.0)
        return f"{{ step = {step}, duration = {dur}, velocity = {v}{rest}}}"

    return text[:a] + HIT.sub(sub, text[a:b]) + text[b:]


def retouch_bass(pid: str, text: str, spb: int, spbar: int) -> str:
    name, amp, bias = AMPLITUDE[pid][1]
    shape = SHAPES[name]
    a, b = region(text, "[accompaniment.bass]", (r"^\[accompaniment\.chords\]",))
    seg = text[a:b]

    def note(m: re.Match) -> str:
        step = int(m[1]) % spbar
        v = vel(shape, amp, bias, int(m[4]), step, spb, spbar)
        return f"{{ step = {m[1]}, duration = {m[2]}, {m[3] or ''}velocity = {v}{m[5]}}}"

    def tpl(m: re.Match) -> str:
        return f"{{ step = {m[1]}, duration = {m[2]}, velocity = {vel(shape, amp, bias, int(m[3]), int(m[1]) % spbar, spb, spbar)} }}"

    def pickup(m: re.Match) -> str:
        return f"{m[1]}{vel(shape, amp, bias, int(m[2]), spbar - 2, spb, spbar)}"

    out = []
    for line in seg.splitlines(keepends=True):
        s = line.lstrip()
        if s.startswith("rhythm = "):
            line = BASS_TPL.sub(tpl, line)
        elif s.startswith("{") or s.startswith("motifs") or "notes = [" in line:
            line = BASS_NOTE.sub(note, line)
        if "pickup = {" in line:
            line = re.sub(r"(pickup = \{[^}]*velocity = )(\d+)", pickup, line)
        out.append(line)
    return text[:a] + "".join(out) + text[b:]


def retouch_walking(pid: str, text: str) -> str:
    w = WALKING[pid]
    return re.sub(r"(\[accompaniment\.bass\.walking\]\nvelocity = )\d+(\naccent = )\d+",
                  lambda m: f"{m[1]}{w['velocity']}{m[2]}{w['accent']}", text, count=1)


# --------------------------------------------------------------------------------------------------------------------
# Point 4 : mise en place. La basse suit la grosse caisse et le comping la caisse claire et le timekeeper (moyenne des
# décalages par voix de la batterie, A6), pas par pas ; la dispersion propre de la basse et du comping est réduite à presque rien
# (celle de la batterie, mesurée, reste) ; le geste d'égrenage commence sur la référence (`anchor = 0`).
# Jazz : la batterie n'a aucun décalage par voix ; le retard de +20 ms du piano mesuré par la JTD est retiré (aucun instrument
# ne dérive de la grille plus que la batterie).

BASS_JITTER, CHORDS_JITTER = 0.02, 0.02
# Dispersion de la batterie seule : les quatre profils de B8 portaient le plafond de l'A6 (0,1 pas, soit un écart-type de 0,058 pas)
# partout. Le GMD donne, DANS un fichier, un écart-type de 0,03 à 0,05 pas par voix (grosse caisse, caisse claire, charleston :
# médiane des fichiers `beat` droits du groupe) ; la dispersion mesurée entre fichiers (0,14 pas) est celle des batteurs, pas d'un
# batteur. Amplitude de la dispersion uniforme = √3 × écart-type médian des trois voix (arrondi à 0,005).
DRUM_JITTER = {"gmd_rock": 0.07, "gmd_soul": 0.045, "gmd_reggae": 0.06, "gmd_pop": 0.06}


def voices(profile: dict) -> dict[int, tuple[float, float]]:
    out: dict[int, tuple[float, float]] = {}
    for track in profile.get("performance", {}).get("tracks", {}).values():
        for v in track.get("voices", []):
            on = v.get("on_beat", 0.0)
            out[v["note"]] = (on, v.get("off_beat", on))
    return out


def clamp(x: float) -> float:
    return round(max(-0.2, min(0.2, x)), 4)


def placement(profile: dict) -> tuple[list[float], list[float]]:
    spbar = profile["meter"]["steps_per_bar"]
    pulse = profile["meter"]["steps_per_pulse"]
    v = voices(profile)
    kick = v.get(36)
    snare = v.get(38) or v.get(37)
    tk = next((v[n] for n in (42, 51, 44) if n in v), None)
    bass, chords = [], []
    for s in range(spbar):
        i = 0 if s % pulse == 0 else 1
        bass.append(clamp(kick[i]) if kick else 0.0)
        refs = [x[i] for x in (snare, tk) if x]
        chords.append(clamp(sum(refs) / len(refs)) if refs else 0.0)
    return bass, chords


def retouch_performance(pid: str, text: str, profile: dict) -> str:
    bass, chords = placement(profile)
    fmt = lambda xs: "[" + ", ".join(f"{x:g}" for x in xs) + "]"
    phrase = f"phrase = [[0.0, {-PHRASE[pid]:g}], [0.5, {PHRASE[pid]:g}], [1.0, {-PHRASE[pid]:g}]]\n" if pid in PHRASE else ""
    bound = f"velocity_min = {VEL_MIN}\nvelocity_max = {VEL_MAX}\n"
    new_bass = "[performance.tracks.Bass]\n" + (f"offsets = {fmt(bass)}\n" if any(bass) else "") + f"jitter = {BASS_JITTER}\n{phrase}{bound}"
    new_chords = "[performance.tracks.Chords]\n" + (f"offsets = {fmt(chords)}\n" if any(chords) else "") + f"jitter = {CHORDS_JITTER}\n{bound}"
    text, n1 = re.subn(r"\[performance\.tracks\.Bass\]\n(?:(?!\[)[^\n]*\n)*?(?=\n|\[|\Z)", lambda m: new_bass, text, count=1)
    text, n2 = re.subn(r"\[performance\.tracks\.Chords\]\n(?:(?!\[)[^\n]*\n)*?(?=\n|\[|\Z)", lambda m: new_chords, text, count=1)
    assert n1 == n2 == 1, (pid, n1, n2)
    if pid in DRUM_JITTER:
        a = text.index("[performance.tracks.PercussionLow]")
        b = text.index("[performance.tracks.Bass]")
        text = text[:a] + text[a:b].replace("jitter = 0.1", f"jitter = {DRUM_JITTER[pid]:g}") + text[b:]
    text = re.sub(r"(\[accompaniment\.chords\.spread\]\n(?:[^\[\n][^\n]*\n)*?)anchor = [0-9.]+", r"\1anchor = 0.0", text)
    return text


# --------------------------------------------------------------------------------------------------------------------


def session(text: str) -> str:
    """Point 1 : retire le groupe `feel` (variantes `droit` et `swing_leger`) : le feel droit du profil reste seul."""
    out = re.sub(r"\[\[variants\]\]\ngroup = \"feel\"\nname = \"(?:droit|swing_leger)\"\n(?:[a-z_]+ = [^\n]*\n)*\n", "", text)
    assert 'group = "feel"' not in out
    out = re.sub(r"\(([0-9]+) habillages possibles\)", lambda m: f"({int(m[1]) // 2} habillages possibles)", out, count=1)
    out = re.sub(r"(centiles des BPM des fichiers `beat`, [0-9 à]+\)) et\n# le swing léger \(rapport médian des fichiers `swing`\)\.",
                 r"\1.", out, count=1)
    return out


def build(pid: str, out_root: Path) -> None:
    rel = f"profiles/{pid}"
    out = out_root / rel
    out.mkdir(parents=True, exist_ok=True)
    if pid in E1:
        prof_text = git_show(f"{rel}/profile.toml")
        profile = tomllib.loads(prof_text)
        spbar = profile["meter"]["steps_per_bar"]
        spb = spbar // profile["meter"]["time_signature"]["numerator"]
        prof_text = retouch_comping(pid, prof_text, spb, spbar)
        if AMPLITUDE[pid][1]:
            prof_text = retouch_bass(pid, prof_text, spb, spbar)
        if pid in WALKING:
            prof_text = retouch_walking(pid, prof_text)
        prof_text = retouch_performance(pid, prof_text, profile)
        (out / "profile.toml").write_text(prof_text)
    if pid in SANS_SWING_LEGER:
        (out / "session.toml").write_text(session(git_show(f"{rel}/session.toml")))


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
