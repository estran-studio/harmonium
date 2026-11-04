#!/usr/bin/env python3
"""R2-trad : habillages distincts sur 16 graines, lus dans la vérité terrain du CLI (`--record-truth`).

    python3 habillage_mesure.py <harmonium-cli> [--out f.json] [profil ...]

Lancé depuis `~/Project/bascanada/harmonium` (pour `assets/`). Par profil et par graine : tempo des snapshots,
tonalité et mode (`position`), programmes qui sonnent (`instruments`), famille de voicings et variantes de
session (`session`). Un habillage = (grappe de tempo à ± 8 BPM, tonalité, mode, programmes des accords, de la basse,
du kit, variante de voicing, swing). Aucune lecture audio.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

PROFILES = ["reel_quebecois", "reel_acadien", "reel_britannique_coloniale", "gigue_quebecoise", "gigue_acadienne",
            "gigue_britannique_coloniale", "hornpipe_britannique_coloniale", "marche_britannique_coloniale", "valse_quebecoise",
            "valse_britannique_coloniale"]
WORK = Path("/tmp/h019-r2-trad/habillage")


def clusters(values: list[float], width: float = 8.0) -> list[int]:
    """Grappes de tempo : un tempo à moins de `width` BPM du premier de la grappe y entre."""
    firsts: list[float] = []
    out = []
    for v in values:
        for i, f in enumerate(firsts):
            if abs(v - f) <= width:
                out.append(i)
                break
        else:
            firsts.append(v)
            out.append(len(firsts) - 1)
    return out


def run(cli: Path, profile: str, seed: int, bars: int) -> dict:
    truth = WORK / f"{profile}_{seed}.json"
    if not truth.is_file():
        subprocess.run([str(cli), "--export", "--style", profile, "--seed", str(seed), "--bars", str(bars),
                        "--record-truth", str(truth)], check=True, capture_output=True,
                       cwd=Path.home() / "workspace/harmonium")
    last = {}
    for rec in json.loads(truth.read_text())["measures"]:
        last[rec["measure"]["index"]] = rec["measure"]
    first = last[min(last)]
    session = {s["group"]: s["name"] for s in (first.get("session") or [])}
    instruments = first.get("instruments") or {}
    pos = first["position"]
    return {"tempo": first["tempo"], "key": pos["key"], "mode": pos["mode"], "swing": round(first["swing_ratio"], 3),
            "melody": instruments.get("melody_program"), "chords": instruments.get("chords_program"), "bass": instruments.get("bass_program"),
            "kit": instruments.get("percussion_program"), "voicing": session.get("voicing"), "arc": session.get("arc"),
            "session": session}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("cli", type=Path)
    ap.add_argument("--out", type=Path)
    ap.add_argument("--bars", type=int, default=16)
    ap.add_argument("profiles", nargs="*")
    args = ap.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    table = {}
    for profile in args.profiles or PROFILES:
        rows = [run(args.cli, profile, seed, args.bars) for seed in range(1, 17)]
        grape = clusters([r["tempo"] for r in rows])
        habs = {(g, r["key"], r["mode"], r["melody"], r["chords"], r["bass"], r["kit"], r["voicing"], r["swing"])
                for g, r in zip(grape, rows)}
        table[profile] = {
            "habillages_distincts": len(habs), "grappes_de_tempo": len(set(grape)),
            "tempo_min_max": [min(r["tempo"] for r in rows), max(r["tempo"] for r in rows)],
            "tonalites": len({(r["key"], r["mode"]) for r in rows}),
            "programmes_melodie": sorted({r["melody"] for r in rows if r["melody"] is not None}),
            "programmes_comping": sorted({r["chords"] for r in rows if r["chords"] is not None}),
            "programmes_basse": sorted({r["bass"] for r in rows if r["bass"] is not None}),
            "kits": sorted({r["kit"] for r in rows if r["kit"] is not None}),
            "voicings": sorted({r["voicing"] for r in rows if r["voicing"]}),
            "arcs": dict(Counter(r["arc"] for r in rows if r["arc"])),
            "swings": sorted({r["swing"] for r in rows}),
            "par_graine": rows,
        }
        t = table[profile]
        print(f"{profile:34} {t['habillages_distincts']:2}/16 habillages, {t['grappes_de_tempo']} grappes de tempo "
              f"({t['tempo_min_max'][0]:g}..{t['tempo_min_max'][1]:g}), {t['tonalites']} tonalités, "
              f"melodie {len(t['programmes_melodie'])}, comping {len(t['programmes_comping'])}, basse {len(t['programmes_basse'])}, kit {len(t['kits'])}, "
              f"voicings {len(t['voicings'])}")
    if args.out:
        args.out.write_text(json.dumps(table, indent=1, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
