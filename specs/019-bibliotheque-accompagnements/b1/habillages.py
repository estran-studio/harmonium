#!/usr/bin/env python3
"""B1 : habillages distincts sur N graines, lus dans la vérité terrain du CLI (`--record-truth`).

    python3 specs/019-bibliotheque-accompagnements/b1/habillages.py --cli target/release/harmonium-cli \\
        --profile profiles/wjazzd_swing [--seeds 16] [--work /tmp/harmonium-019/b1/hab] [--bars 2]

Lancé depuis `~/Project/bascanada/harmonium` (pour `assets/`). Aucune lecture audio : rendu hors ligne.
Habillage = (grappe de tempo à ± 8 BPM, tonalité et mode, programmes comping / basse / kit, famille de voicings).
"""
import argparse
import json
import subprocess
from pathlib import Path


def truth(cli, profile, seed, bars, out):
    if not out.is_file():
        out.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run([cli, "--export", "--bars", str(bars), "--profile", str(profile), "--seed", str(seed),
                        "--record-truth", str(out)], check=True, capture_output=True)
    return json.loads(out.read_text())["measures"][0]["measure"]


def clusters(values, width=16):
    """Grappes de largeur `width` BPM (± 8) : une valeur ouvre une grappe, les suivantes dans `width` la rejoignent."""
    out = []
    for v in sorted(values):
        if out and v - out[-1][0] < width:
            out[-1].append(v)
        else:
            out.append([v])
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cli", default="target/release/harmonium-cli")
    ap.add_argument("--profile", required=True)
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=2)
    ap.add_argument("--work", type=Path, default=Path("/tmp/harmonium-019/b1/hab"))
    args = ap.parse_args()
    rows = []
    for seed in range(1, args.seeds + 1):
        m = truth(args.cli, args.profile, seed, args.bars, args.work / f"{Path(args.profile).name}_s{seed}.json")
        s = {e["group"]: e["name"] for e in m.get("session") or []}
        i = m.get("instruments") or {}
        rows.append({"seed": seed, "bpm": m["composition_bpm"], "key": m["position"]["key"], "mode": m["position"]["mode"],
                     "comping": i.get("chords_program"), "bass": i.get("bass_program"),
                     "kit": i.get("percussion_program"), "voicing": s.get("voicing"), "rythme": s.get("rythme"),
                     "session": s})
    grappes = clusters([r["bpm"] for r in rows])
    index = {v: n for n, g in enumerate(grappes) for v in g}
    keys = set()
    print("| graine | BPM | tonalité | comping | basse | kit | voicings | rythme (tempo, kit, arc) |")
    print("|---:|---:|---|---:|---:|---:|---|---|")
    for r in rows:
        keys.add((index[r["bpm"]], r["key"], r["mode"], r["comping"], r["bass"], r["kit"], r["voicing"]))
        print(f'| {r["seed"]} | {r["bpm"]:.0f} | {r["key"]} {r["mode"]} | {r["comping"]} | {r["bass"]} | {r["kit"]} | {r["voicing"]} | {r["rythme"]} |')
    print()
    print(f"habillages distincts : {len(keys)} / {len(rows)} ; grappes de tempo : {len(grappes)} "
          f"({min(r['bpm'] for r in rows):.0f} à {max(r['bpm'] for r in rows):.0f} BPM) ; "
          f"tonalités : {len({(r['key'], r['mode']) for r in rows})} ; comping : {len({r['comping'] for r in rows})} ; "
          f"basse : {len({r['bass'] for r in rows})} ; kit : {len({r['kit'] for r in rows})} ; "
          f"voicings : {len({r['voicing'] for r in rows})}")
    (args.work / "resume.json").write_text(json.dumps(rows, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
