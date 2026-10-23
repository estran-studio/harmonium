"""Compare first-pass snapshot material with the lab's unchanged DS-T9 metrics.

Input: JSON produced by the phrase_metrics example (command in plan.md).
Onsets are expressed in cell units; durations end at the next onset or bar,
exactly as engine_material excludes MIDI gate and swing. No corpus rewriting.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--lab-root", type=Path, required=True)
    parser.add_argument("--snapshots", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    sys.path.insert(0, str(args.lab_root / "src"))
    from harmonium_lab.trad.model import Note
    from harmonium_lab.trad.structure import Material, analyze_materials

    grouped = defaultdict(list)
    for row in json.loads(args.snapshots.read_text()):
        length = row["units"]
        bars = []
        for notes in row["bars"]:
            bars.append([
                Note(pitch, onset, (notes[i + 1][1] if i + 1 < len(notes) else length) - onset)
                for i, (pitch, onset) in enumerate(notes)
            ])
        material = Material([("A", bars[:8]), ("B", bars[8:16])], bars,
                            row["key"], row["mode"], row["meter"], length,
                            key_source="MeasureSnapshot")
        grouped[(row["style"], row["variant"])].append(material)
    reports = {}
    for (style, variant), materials in grouped.items():
        stats = analyze_materials(materials)
        structures = {r["pattern"]: r["share"] for r in stats["structures"]}
        reports.setdefault(style, {})[variant] = {
            "airs": len(materials),
            "entropy_bits": stats["rhythm"]["entropy_bits"],
            "m5_6_copy": stats["phrases"]["opening_reprise_rate"],
            "a_b_a_c": structures.get("a b a c", 0),
            "a_b_c_d": structures.get("a b c d", 0),
            "A_m8_tonic": stats["phrases"]["rest_by_position"]["A:m8"]["tonic_cadence_rate"],
            "B_m7_A_m7": stats["transformations"]["B:m7/A:m7"]["rates"]["exact"],
        }
    args.output.write_text(json.dumps(reports, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(reports, indent=2, ensure_ascii=False))


    for style, variants in reports.items():
        before, after = variants["before_p3t3"], variants["after"]
        if after["entropy_bits"] < variants["before_p3t1"]["entropy_bits"]:
            raise ValueError(f"{style}: entropie sous le contrat pré-phrase")
        for metric in ("m5_6_copy", "a_b_a_c", "a_b_c_d", "A_m8_tonic", "B_m7_A_m7"):
            if abs(after[metric] - before[metric]) > 0.10:
                raise ValueError(f"{style}: {metric} dévie de plus de dix points")
if __name__ == "__main__":
    main()
