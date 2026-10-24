"""Mesures d'accompagnement dans le corpus ABC (lecture seule, parseur commun du labo).

Usage : .venv/bin/python measure_chords.py OUT.json  (depuis harmonium_lab)
Rythme harmonique, positions des changements, accords par position de phrase,
mouvements de fondamentale, notes de passage possibles, notes de mélodie sur le temps.
"""
import json, sys
from collections import Counter, defaultdict
from fractions import Fraction
from pathlib import Path
sys.path.insert(0, "src")
from harmonium_lab.trad.form import analyze_form
from harmonium_lab.trad.features import parse_chord_symbol, beat_eighths
from harmonium_lab.trad.model import mode_intervals, normalize_mode

ROOT = Path("corpus")
PROFILES = {
    "reel_quebecois": ("trad", "quebecoise", "reel", 8, None),
    "gigue_quebecoise": ("trad", "quebecoise", "jig", 6, "6/8"),
    "valse_quebecoise": ("trad", "quebecoise", "waltz", 6, "3/4"),
    "reel_acadien": ("acadienne", "acadienne", "reel", 8, None),
    "gigue_britannique_coloniale": ("britannique_coloniale", "britannique_coloniale", "jig", 6, "6/8"),
    "hornpipe_britannique_coloniale": ("britannique_coloniale", "britannique_coloniale", "hornpipe", 8, None),
    "marche_britannique_coloniale": ("britannique_coloniale", "britannique_coloniale", "march", 8, None),
}
SEMI_CLASS = {0: "I", 5: "IV", 7: "V", 9: "vi", 2: "ii", 10: "bVII", 3: "bIII", 8: "bVI", 4: "III", 11: "VII", 1: "b2", 6: "tri"}

def ident(sym):
    pc, q = parse_chord_symbol(sym)
    return None if pc is None else (pc, q)

def triad(pc, q):
    third = 3 if q in ("minor", "diminished") else 4
    fifth = 6 if q == "diminished" else 7
    return {pc % 12, (pc + third) % 12, (pc + fifth) % 12}

def measure(entries, k_want, meter_want):
    st = defaultdict(Counter)
    pos = defaultdict(lambda: defaultdict(Counter))
    n_tunes = n_ann = 0
    syms = slash = sevenths = 0
    for path in entries:
        n_tunes += 1
        try:
            plan = analyze_form(path.read_text(encoding="utf-8", errors="replace"))
        except Exception:
            continue
        if plan.eighths_per_bar != k_want or (meter_want and plan.meter != meter_want):
            n_tunes -= 1
            continue
        k = plan.eighths_per_bar
        beat = beat_eighths(plan.meter)
        full = [i for i, b in enumerate(plan.bars) if b[3]]
        nsym = sum(len(plan.bars[i][1]) for i in full)
        if not full or not plan.bars[full[0]][1] or nsym / len(full) < 0.4:
            continue
        n_ann += 1
        mode = normalize_mode(plan.mode)
        intervals = mode_intervals(mode)
        tonic = plan.key_tonic_pc
        current = None
        prev_bar_last = None
        # position 1..8 dans les unités de 8 mesures pleines
        position = {}
        if plan.scheme != "croche":
            for p in plan.passes:
                f = [i for i in p if plan.bars[i][3]]
                if len(f) == 8:
                    for j, bi in enumerate(f, 1):
                        position[bi] = j
        for bi in full:
            notes, bar_chords, _, _ = plan.bars[bi]
            ordered = sorted(bar_chords, key=lambda c: float(c[0]))
            changes = []
            start_chord = current
            for off, sym in ordered:
                syms += 1
                slash += "/" in sym
                sevenths += any(ch in sym for ch in ("7",))
                cid = ident(sym)
                if cid is None:
                    continue
                if current is None or cid != current:
                    changes.append((off, current, cid))
                    current = cid
            if start_chord is None and not ordered:
                continue
            n_ch = len(changes)
            st["changes_per_bar"][str(min(n_ch, 3)) + ("+" if n_ch >= 3 else "")] += 1
            st["written_per_bar"][str(min(len(ordered), 4))] += 1
            st["bars"]["n"] += 1
            for off, a, b in changes:
                o = float(off)
                if a is None:
                    continue
                if o == 0:
                    cls = "temps 1 (début de mesure)"
                elif abs(o - k / 2) < 1e-9 and meter_want != "3/4":
                    cls = "demi-mesure"
                elif o % beat == 0:
                    cls = "autre temps"
                else:
                    cls = "contretemps"
                st["change_position"][cls] += 1
                st["change_offset_eighths"][f"{o:g}"] += 1
                semis = (b[0] - a[0]) % 12
                st["root_motion_semitones_up"][str(semis)] += 1
                ia = intervals.index((a[0] - tonic) % 12) if (a[0] - tonic) % 12 in intervals else None
                ib = intervals.index((b[0] - tonic) % 12) if (b[0] - tonic) % 12 in intervals else None
                if ia is None or ib is None:
                    st["root_motion_diatonic"]["chromatique"] += 1
                else:
                    d7 = (ib - ia) % 7
                    st["root_motion_diatonic"][str(d7)] += 1
                    st["boundary" if o == 0 else "mid"]["x"] += 1
                    steps = min(d7, 7 - d7)
                    st["passing_available"][str(max(steps - 1, 0))] += 1
                    st["passing_by_position"][("bar_boundary" if o == 0 else "intra") + f":{max(steps-1,0)}"] += 1
            j = position.get(bi)
            if j:
                last = current
                changed_in = n_ch > 0
                first_off = float(ordered[0][0]) if ordered else None
                entry_change = bool(changes and float(changes[0][0]) == 0)
                intra = sum(1 for c in changes if float(c[0]) != 0)
                pos[j]["n"]["bars"] += 1
                pos[j]["entry"]["change"] += entry_change
                pos[j]["intra"]["change"] += intra > 0
                if last is not None:
                    pos[j]["final"][SEMI_CLASS.get((last[0] - tonic) % 12, "autre")] += 1
                if start_chord is not None:
                    pos[j]["start"][SEMI_CLASS.get((start_chord[0] - tonic) % 12 if not entry_change else (changes[0][2][0] - tonic) % 12, "autre")] += 1
            # note de mélodie sur le temps vs accord actif
            cur_at = start_chord
            timeline = sorted(((float(o), ident(s)) for o, s in ordered if ident(s)), key=lambda t: t[0])
            for onset, _dur, pitch in notes:
                if pitch is None:
                    continue
                o = float(onset)
                if o % beat != 0:
                    continue
                active = start_chord
                for to, tid in timeline:
                    if to <= o:
                        active = tid
                if active is None:
                    continue
                kind = "temps 1" if o == 0 else ("demi-mesure" if abs(o - k / 2) < 1e-9 else "autre temps")
                st["melody_on_beat"][kind + (":ct" if pitch % 12 in triad(*active) else ":nct")] += 1
            prev_bar_last = current
    return {
        "tunes_total": n_tunes, "tunes_annotated": n_ann, "symbols": syms,
        "slash_symbols": slash, "seventh_symbols": sevenths,
        "stats": {k: dict(v) for k, v in st.items()},
        "by_position": {str(j): {a: dict(b) for a, b in v.items()} for j, v in sorted(pos.items())},
    }

def main(out):
    result = {}
    for name, (folder, trad, typ, k, meter) in PROFILES.items():
        manifest = [json.loads(l) for l in (ROOT / folder / "manifest.jsonl").read_text().splitlines() if l.strip()]
        paths = [ROOT / folder / "tunes" / e["type"] / (e["id"] + ".abc") for e in manifest if e["type"] == typ]
        result[name] = measure(paths, k, meter)
        print(name, result[name]["tunes_total"], result[name]["tunes_annotated"], file=sys.stderr)
    Path(out).write_text(json.dumps(result, ensure_ascii=False, indent=1))

if __name__ == "__main__":
    main(sys.argv[1])
