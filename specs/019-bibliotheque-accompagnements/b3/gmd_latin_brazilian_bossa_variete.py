#!/usr/bin/env python3
"""B3 bossa : tableau de variété (`harmonium-lab seed-variety --chorus 4`, 16 graines) du profil livré, de la copie sans
`session.toml` (« avant ») et de V0 `wjazzd_swing` remesuré avec le même binaire.

    harmonium-lab seed-variety --profile <dossier> --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium \
        --engine-bin <cli> --label B3 --work-dir /tmp/harmonium-019/b3/bossa/var/w_<nom> -o /tmp/harmonium-019/b3/bossa/var/<nom>.json
    python3 gmd_latin_brazilian_bossa_variete.py livre.json avant.json v0.json [--out resume.json]

Définitions : `harmonium_lab/references/v0-variance/README.md`. Par couche : mesures changées d'un chorus au suivant (intra,
`adjacent.exact`), paires de chorus identiques, chorus distincts par graine, suites distinctes sur 16 graines (chorus égal, place
égale) et distance graduée entre graines (`inter.graded`). `orchestration` : suites de programmes (MIDI, `program_change`).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

LAYERS = [("accords", "chords"), ("comping : rythme", "comping_rhythm"), ("comping : voicings", "comping_voicing"),
          ("basse", "bass"), ("batterie", "drums"), ("orchestration", "orchestration"), ("accompagnement", "all")]


def summarize(profile: dict) -> dict:
    out = {}
    for label, key in LAYERS:
        intra, inter = profile["intra"][key], profile["inter"][key]
        out[label] = {"intra_mesures_changees": round(intra["adjacent"]["exact"], 3),
                      "intra_paires_identiques": round(intra["identical_pairs"], 3),
                      "chorus_distincts_par_graine": round(intra["distinct_choruses_mean"], 2),
                      "inter_suites_distinctes": inter["distinct_runs"],
                      "inter_distance_graduee": round(inter["graded"], 3)}
    return out


def main() -> int:
    names = ["livre", "avant", "v0"]
    files = [Path(a) for a in sys.argv[1:4]]
    out_path = Path(sys.argv[sys.argv.index("--out") + 1]) if "--out" in sys.argv else None
    table = {}
    raw = {}
    for name, path in zip(names, files):
        data = json.loads(path.read_text())
        profile = data["profiles"][0]
        raw[name] = profile
        table[name] = summarize(profile)
    heads = {"livre": "livré (avec session)", "avant": "sans session.toml (avant)", "v0": "wjazzd_swing (V0 remesuré)"}
    print("| Couche | mesure | " + " | ".join(heads[n] for n in names) + " |")
    print("|---|---|---:|---:|---:|")
    for label, _ in LAYERS:
        for field, text, fmt in (("intra_mesures_changees", "mesures changées d'un chorus au suivant", "{:.0%}"),
                                 ("inter_suites_distinctes", "suites distinctes / 16", "{}"),
                                 ("inter_distance_graduee", "distance graduée entre graines", "{:.0%}")):
            print(f"| {label} | {text} | " + " | ".join(fmt.format(table[n][label][field]) for n in names) + " |")
    for n in names:
        keys = [k for k in raw[n] if k not in ("intra", "inter")]
        extra = {k: raw[n][k] for k in keys if k in ("orchestration",)}
        if extra:
            print(n, json.dumps(extra)[:600])
    if out_path:
        out_path.write_text(json.dumps(table, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
