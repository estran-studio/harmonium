"""Count first/final written endings without modifying the lab or its corpus.

Run with the lab's Python environment and --lab-root pointing to that checkout.
FormPlan completes partial ending bars with a pickup, then removes source indices.
A temporary profiling hook reads the parser's existing source mapping at return:
no repeat/volta parser is duplicated, and no artificial pickup is counted as a
written final note. This research command deliberately checks that private map;
it fails rather than silently changing the measurement if the parser changes.
Only aggregate counts leave the corpus.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys


class OriginMapChanged(RuntimeError):
    pass


def analyze_with_origins(text, analyze_form):
    origins = {}

    def capture(frame, event, _result):
        if event == "return" and frame.f_code is analyze_form.__code__:
            for key in ("segments", "passes", "kept"):
                if key in frame.f_locals:
                    origins[key] = frame.f_locals[key]

    previous_profile = sys.getprofile()
    sys.setprofile(capture)
    try:
        plan = analyze_form(text)
    finally:
        sys.setprofile(previous_profile)
    if set(origins) != {"segments", "passes", "kept"}:
        raise OriginMapChanged("form.analyze_form source map changed")
    flattened = [index for group in origins["passes"] for index in group]
    segments = [origins["segments"][flattened[index]] for index in origins["kept"]]
    if len(segments) != len(plan.bars):
        raise OriginMapChanged("written and completed bar maps no longer align")
    return plan, segments


def category(pitch, tonic):
    interval = (pitch - tonic) % 12
    return "tonic" if interval == 0 else "fifth" if interval == 7 else "other"


def measure(root, kind, analyze_form):
    counts = defaultdict(Counter)
    errors = Counter()
    tunes = paired = eight = 0
    paths = sorted((root / "tunes" / kind).glob("*.abc"))
    for path in paths:
        text = path.read_text(encoding="utf-8")
        try:
            plan, original = analyze_with_origins(text, analyze_form)
        except OriginMapChanged:
            raise
        except Exception as error:
            errors["parse:" + type(error).__name__] += 1
            continue
        if (
            plan.scheme == "croche"
            or sum(plan.parts) != len(plan.bars)
            or len(plan.scheme) != len(plan.parts)
        ):
            errors["irregular"] += 1
            continue
        groups = defaultdict(list)
        cursor = 0
        for label, size in zip(plan.scheme, plan.parts):
            end = cursor + size - 1
            cursor += size
            if size <= 0:
                continue
            played = next((p for _, _, p in reversed(plan.bars[end][0]) if p is not None), None)
            written = next((p for _, _, p in reversed(original[end].notes) if p is not None), None)
            if played is None or written is None:
                errors["empty_ending"] += 1
                continue
            if played != written:
                counts["diagnostics"]["written_vs_completed_final_pitch_changes"] += 1
            groups[label].append((category(played, plan.key_tonic_pc), category(written, plan.key_tonic_pc), size))
        tunes += 1
        for label, endings in groups.items():
            if len(endings) < 2:
                counts["single_final_written"][endings[-1][1]] += 1
                continue
            paired += 1
            is_eight = endings[0][2] == endings[-1][2] == 8
            eight += is_eight
            for name, ending in (("first", endings[0]), ("second", endings[-1])):
                counts[name + "_played"][ending[0]] += 1
                counts[name + "_written"][ending[1]] += 1
                counts[label + ":" + name + "_written"][ending[1]] += 1
                if is_eight:
                    counts["eight_" + name + "_written"][ending[1]] += 1
                    counts["eight_" + label + ":" + name + "_written"][ending[1]] += 1
    return {
        "input_tunes": len(paths),
        "tunes": tunes,
        "paired_parts": paired,
        "eight_bar_pairs": eight,
        "counts": dict(counts),
        "excluded": dict(errors),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lab-root", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    sys.path.insert(0, str(args.lab_root.resolve() / "src"))
    from harmonium_lab.trad.form import analyze_form

    results = {}
    for tradition, folder in (("quebecoise", "trad"), ("acadienne", "acadienne"), ("britannique_coloniale", "britannique_coloniale")):
        for kind in ("reel", "jig", "waltz", "march", "hornpipe"):
            root = args.lab_root / "corpus" / folder
            if (root / "tunes" / kind).exists():
                results[f"{tradition}-{kind}"] = measure(root, kind, analyze_form)
    output = json.dumps(results, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.write_text(output, encoding="utf-8")
    else:
        print(output, end="")


if __name__ == "__main__":
    main()
