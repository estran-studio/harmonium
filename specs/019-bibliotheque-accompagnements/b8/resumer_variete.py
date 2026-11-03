#!/usr/bin/env python3
"""B8 : résumé des sorties `harmonium-lab seed-variety --chorus` (16 graines × 4 chorus) des profils `gmd_*`.

    harmonium-lab seed-variety --profile profiles/<id> --seeds 16 --chorus 4 --engine-dir ~/Project/bascanada/harmonium \\
        --engine-bin target/release/harmonium-cli --label B8 -o /tmp/harmonium-019/b8/var/<id>.json

Puis `resumer_variete.py /tmp/harmonium-019/b8/var preuves/variete.json` : par profil et par couche, la part de mesures
changées d'un chorus au suivant, les paires de chorus identiques, les chorus distincts par graine, les suites
distinctes sur 16 graines (chorus égal, place égale) et la distance graduée entre graines. La référence de V0
(`harmonium_lab/references/v0-variance/wjazzd_swing.json`, grille par défaut, avant A3 et S0) est ajoutée si présente.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

LAYERS = [("accords", "chords"), ("comping : rythme", "comping_rhythm"), ("comping : voicings", "comping_voicing"),
          ("basse", "bass"), ("batterie", "drums"), ("accompagnement", "all")]


def summarize(profile: dict) -> dict:
    out = {}
    for label, key in LAYERS:
        intra, inter = profile["intra"][key], profile["inter"][key]
        out[label] = {
            "intra_mesures_changees": round(intra["adjacent"]["exact"], 3),
            "intra_paires_identiques": round(intra["identical_pairs"], 3),
            "chorus_distincts_par_graine": round(intra["distinct_choruses_mean"], 2),
            "inter_suites_distinctes": inter["distinct_runs"],
            "inter_distance_graduee": round(inter["graded"], 3),
        }
    return out


def main() -> int:
    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    table = {}
    for path in sorted(src.glob("gmd_*.json")):
        data = json.loads(path.read_text())
        table[data["profiles"][0]["profile"]] = summarize(data["profiles"][0])
    ref = Path.home() / "Project/bascanada/harmonium_lab/references/v0-variance/wjazzd_swing.json"
    if ref.is_file():
        data = json.loads(ref.read_text())
        profile = data["profiles"][0] if "profiles" in data else data
        table["wjazzd_swing (V0, avant A3 et S0)"] = summarize(profile)
    dst.write_text(json.dumps(table, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    print("| Profil | Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | Suites distinctes (16 graines) | Distance graduée entre graines |")
    print("|---|---|---:|---:|---:|---:|---:|")
    for pid, layers in table.items():
        for label, m in layers.items():
            print(f"| `{pid}` | {label} | {m['intra_mesures_changees']:.0%} | {m['intra_paires_identiques']:.0%} | "
                  f"{m['chorus_distincts_par_graine']} | {m['inter_suites_distinctes']} / 16 | {m['inter_distance_graduee']:.0%} |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
