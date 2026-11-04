#!/usr/bin/env python3
"""B6 : habillages distincts sur 16 graines de `gmd_blues_shuffle`, lus dans la vérité terrain du CLI (`--record-truth`).

    python3 habillage_mesure.py <harmonium-cli> [--out f.json]

Lancé avec le CLI du worktree ; le CLI tourne depuis `~/workspace/harmonium` (pour `assets/`) sur le dossier du profil.
Même définition que `r2-styles/habillage_mesure.py` : un habillage = (grappe de tempo à ± 8 BPM, tonalité, mode, programmes
des accords, de la basse, du kit, famille de voicings). Aucune lecture audio.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "r2-styles"))
from habillage_mesure import clusters  # noqa: E402

PROFILE = Path(__file__).resolve().parents[3] / "profiles" / "gmd_blues_shuffle"
ENGINE = Path.home() / "workspace/harmonium"
WORK = Path("/tmp/h019-b6/habillage")


def run(cli: Path, seed: int, bars: int) -> dict:
    truth = WORK / f"gmd_blues_shuffle_{seed}.json"
    if not truth.is_file():
        subprocess.run([str(cli), "--export", "--profile", str(PROFILE), "--seed", str(seed), "--bars", str(bars),
                        "--record-truth", str(truth)], check=True, capture_output=True, cwd=ENGINE)
    last = {}
    for rec in json.loads(truth.read_text())["measures"]:
        last[rec["measure"]["index"]] = rec["measure"]
    first = last[min(last)]
    session = {s["group"]: s["name"] for s in (first.get("session") or [])}
    instruments = first.get("instruments") or {}
    pos = first["position"]
    return {"tempo": first["tempo"], "key": pos["key"], "mode": pos["mode"], "swing": round(first["swing_ratio"], 3),
            "chords": instruments.get("chords_program"), "bass": instruments.get("bass_program"),
            "kit": instruments.get("percussion_program"), "voicing": session.get("voicing"), "arc": session.get("arc"),
            "register": session.get("registre"), "energy": session.get("energie"), "session": session}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("cli", type=Path)
    ap.add_argument("--out", type=Path)
    ap.add_argument("--bars", type=int, default=12)
    args = ap.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    rows = [run(args.cli, seed, args.bars) for seed in range(1, 17)]
    grape = clusters([r["tempo"] for r in rows])
    habs = {(g, r["key"], r["mode"], r["chords"], r["bass"], r["kit"], r["voicing"]) for g, r in zip(grape, rows)}
    table = {
        "habillages_distincts": len(habs), "grappes_de_tempo": len(set(grape)),
        "tempo_min_max": [min(r["tempo"] for r in rows), max(r["tempo"] for r in rows)],
        "tonalites": sorted({r["key"] for r in rows}),
        "programmes_comping": sorted({r["chords"] for r in rows if r["chords"] is not None}),
        "programmes_basse": sorted({r["bass"] for r in rows if r["bass"] is not None}),
        "kits": sorted({r["kit"] for r in rows if r["kit"] is not None}),
        "voicings": sorted({r["voicing"] for r in rows if r["voicing"]}),
        "arcs": dict(Counter(r["arc"] for r in rows if r["arc"])),
        "swings": sorted({r["swing"] for r in rows}),
        "par_graine": rows,
    }
    print(f"{table['habillages_distincts']:2}/16 habillages, {table['grappes_de_tempo']} grappes de tempo "
          f"({table['tempo_min_max'][0]:g}..{table['tempo_min_max'][1]:g}), {len(table['tonalites'])} tonalités, "
          f"comping {len(table['programmes_comping'])}, basse {len(table['programmes_basse'])}, kit {len(table['kits'])}, "
          f"voicings {len(table['voicings'])}, arcs {table['arcs']}")
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(table, indent=1, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
