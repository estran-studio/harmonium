#!/usr/bin/env python3
"""B1 : rapport de swing de `wjazzd_swing` mesuré sur le MIDI exporté, par tempo et par piste, contre JTD.

Réutilise `a6-mesure-swing.py` (même définition que `references/jtd-swing-tempo` : fraction de la noire des
attaques de contretemps entre 0,45 et 0,85, médiane). Tempo imposé par une automation (l'hôte gagne sur la
variante de session). Lancé depuis `~/Project/bascanada/harmonium` avec le venv du labo :

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b1/mesure_swing.py \\
        --cli <worktree>/target/release/harmonium-cli --profile <worktree>/profiles/wjazzd_swing

Aucune lecture audio : le CLI rend hors ligne.
"""
import argparse
import importlib.util
import json
import statistics
from pathlib import Path

A6 = Path(__file__).resolve().parents[1] / "a6-mesure-swing.py"
spec = importlib.util.spec_from_file_location("a6", A6)
a6 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(a6)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cli", default="target/release/harmonium-cli")
    ap.add_argument("--profile", required=True)
    ap.add_argument("--bars", type=int, default=96)
    ap.add_argument("--seeds", type=int, nargs="+", default=[1, 2, 3, 4, 5, 6])
    ap.add_argument("--tempos", type=int, nargs="+", default=[110, 150, 190, 230, 270])
    ap.add_argument("--out", default="/tmp/harmonium-019/b1/swing.json")
    args = ap.parse_args()
    table = a6.jtd_curve()
    rows = []
    print("| tempo | batterie (ride) mesuré / JTD | basse mesuré / JTD | comping mesuré / JTD |")
    print("|---:|---|---|---|")
    for bpm in args.tempos:
        pooled = {k: [] for k in a6.TRACKS}
        for seed in args.seeds:
            midi = a6.render(args.cli, args.profile, bpm, args.bars, seed)
            ppq = midi.ticks_per_beat
            att = a6.attacks(midi)
            for label, (channel, note, _) in a6.TRACKS.items():
                f = [(t % ppq) / ppq for t, c, n in att if c == channel and (note is None or n == note)]
                pooled[label] += [x for x in f if 0.45 <= x < 0.85]
        row = {"tempo": bpm}
        cells = []
        for label, (_, _, key) in a6.TRACKS.items():
            v = pooled[label]
            med = round(statistics.median(v), 4) if v else None
            ref = table[key].get(float(bpm))
            row[label] = {"mesure": med, "n": len(v), "jtd": ref}
            cells.append("—" if med is None else f"{med:.3f} / {ref:.3f} (n={len(v)})")
        rows.append(row)
        print(f"| {bpm} | " + " | ".join(cells) + " |")
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out).write_text(json.dumps(rows, indent=1, ensure_ascii=False))


if __name__ == "__main__":
    main()
