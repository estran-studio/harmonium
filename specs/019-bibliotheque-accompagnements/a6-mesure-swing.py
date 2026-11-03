#!/usr/bin/env python3
"""A6 : rapport de swing mesuré sur le MIDI exporté, par tempo et par piste, contre JTD.

Lance le CLI du moteur (`--export --record-midi`, tempo imposé par une automation `bpm`), lit le
MIDI avec `mido` et mesure, pour chaque piste, le rapport de swing comme le fait
`harmonium_lab/references/jtd-swing-tempo` : fraction de la noire où tombent les attaques de
contretemps (entre 0,45 et 0,85 de la noire), médiane sur les attaques. Compare à la courbe
`a6_table.curve` du labo (batterie, basse, piano) au tempo imposé.

Usage (depuis `~/Project/bascanada/harmonium`, pour `assets/`), avec le venv du labo :

    ~/Project/bascanada/harmonium_lab/.venv/bin/python \\
        specs/019-bibliotheque-accompagnements/a6-mesure-swing.py \\
        --cli target/release/harmonium-cli --out /tmp/harmonium-019/a6/mesures.json

Aucune lecture audio : le CLI rend hors ligne, sans sortie son.
"""
import argparse
import json
import shutil
import statistics
import subprocess
import tempfile
from pathlib import Path

import mido

LAB = Path.home() / "Project/bascanada/harmonium_lab/references/jtd-swing-tempo/measures-swing-tempo.json"
FIXTURES = Path(__file__).resolve().parents[2] / "tests/fixtures/profiles"
TEMPOS = [110, 150, 190, 230, 270]
# (canal MIDI, note ou None) de chaque piste mesurée ; la batterie est le ride (note 51).
TRACKS = {"batterie": (9, 51, "drums"), "basse": (2, None, "bass"), "comping": (1, None, "piano")}


def jtd_curve():
    """Points `tempo -> rapport` de `a6_table.curve` pour la batterie, la basse et le piano."""
    curve = json.loads(LAB.read_text())["a6_table"]["curve"]
    return {k: {p["tempo"]: p["swing_ratio"] for p in v["points"]} for k, v in curve.items()}


def without_voice_timing(profile, tmp):
    """Copie du profil sans `[performance.*]` : le rapport mesuré est celui du swing seul."""
    copy = Path(tmp) / "profil"
    shutil.copytree(profile, copy)
    text = (copy / "profile.toml").read_text()
    start = text.index("[performance.tracks")
    (copy / "profile.toml").write_text(text[:start] + text[text.index("[mix]", start):])
    return copy


def render(cli, profile, bpm, bars, seed, swing=None, strip_voices=False):
    with tempfile.TemporaryDirectory() as tmp:
        auto = Path(tmp) / "auto.toml"
        auto.write_text(f"[[keys]]\nbar = 1\nbpm = {bpm}\n")
        midi = Path(tmp) / "out.mid"
        cmd = [cli, "--export", "--bars", str(bars), "--seed", str(seed), "--automation", str(auto),
               "--record-midi", str(midi)]
        if strip_voices:
            profile = without_voice_timing(profile, tmp)
        cmd += ["--profile", str(profile)] if Path(profile).exists() else ["--style", str(profile)]
        if swing is not None:
            cmd += ["--swing", str(swing)]
        subprocess.run(cmd, check=True, capture_output=True)
        return mido.MidiFile(str(midi))


def attacks(midi):
    out = []
    for track in midi.tracks:
        tick = 0
        for msg in track:
            tick += msg.time
            if msg.type == "note_on" and msg.velocity > 0:
                out.append((tick, msg.channel, msg.note))
    return out


def swing_of(att, channel, note, ppq):
    fractions = [(t % ppq) / ppq for t, c, n in att if c == channel and (note is None or n == note)]
    fractions = [f for f in fractions if 0.45 <= f < 0.85]
    return (statistics.median(fractions), len(fractions)) if fractions else (None, 0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cli", default="target/release/harmonium-cli")
    ap.add_argument("--out", default="/tmp/harmonium-019/a6/mesures.json")
    ap.add_argument("--bars", type=int, default=96)
    ap.add_argument("--seeds", type=int, nargs="+", default=[1, 2, 3])
    args = ap.parse_args()
    table = jtd_curve()
    rows = []
    shuffle = FIXTURES / "grille_shuffle_ternaire"
    cases = [("grille_shuffle_ternaire (swing seul)", shuffle, True),
             ("grille_shuffle_ternaire (avec micro-timing de voix)", shuffle, False),
             ("grille_valse_jazz", FIXTURES / "grille_valse_jazz", False),
             ("wjazzd_swing (sans courbe)", "wjazzd_swing", False)]
    for name, profile, strip in cases:
        for bpm in TEMPOS:
            pooled = {k: [] for k in TRACKS}
            ppq = 480
            for seed in args.seeds:
                midi = render(args.cli, profile, bpm, args.bars, seed, strip_voices=strip)
                ppq = midi.ticks_per_beat
                att = attacks(midi)
                for label, (channel, note, _) in TRACKS.items():
                    f = [(t % ppq) / ppq for t, c, n in att if c == channel and (note is None or n == note)]
                    pooled[label] += [x for x in f if 0.45 <= x < 0.85]
            row = {"profil": name, "tempo": bpm, "ppq": ppq}
            for label, (_, _, key) in TRACKS.items():
                values = pooled[label]
                row[label] = {
                    "mesure": round(statistics.median(values), 4) if values else None,
                    "n": len(values),
                    "jtd": table[key].get(float(bpm)),
                }
            rows.append(row)
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text(json.dumps(rows, indent=1, ensure_ascii=False))
    print("| profil | tempo | batterie (mesuré / JTD) | basse (mesuré / JTD) | comping (mesuré / JTD) |")
    print("|---|---|---|---|---|")
    for r in rows:
        cells = []
        for label in TRACKS:
            c = r[label]
            cells.append("—" if c["mesure"] is None else f'{c["mesure"]:.3f} / {c["jtd"]:.3f} (n={c["n"]})')
        print(f'| {r["profil"]} | {r["tempo"]} | ' + " | ".join(cells) + " |")


if __name__ == "__main__":
    main()
