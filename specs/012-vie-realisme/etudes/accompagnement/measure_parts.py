"""Accords A/B et 1re/2e passe (reprises) sur les airs AABB annotés ; lecture seule."""
import json, sys
from collections import Counter, defaultdict
from pathlib import Path
sys.path.insert(0, "src")
sys.path.insert(0, "/tmp/accomp_etude")
from harmonium_lab.trad.form import analyze_form
from harmonium_lab.trad.features import parse_chord_symbol
from measure_chords import PROFILES, ROOT, SEMI_CLASS, ident

def cls(chord, tonic):
    return SEMI_CLASS.get((chord[0] - tonic) % 12, "autre")

res = {}
for name, (folder, trad, typ, k_want, meter_want) in PROFILES.items():
    manifest = [json.loads(l) for l in (ROOT / folder / "manifest.jsonl").read_text().splitlines() if l.strip()]
    st = defaultdict(Counter); n = 0
    for e in manifest:
        if e["type"] != typ: continue
        try:
            plan = analyze_form((ROOT / folder / "tunes" / typ / (e["id"] + ".abc")).read_text(errors="replace"))
        except Exception:
            continue
        if plan.eighths_per_bar != k_want or (meter_want and plan.meter != meter_want) or plan.scheme != "AABB": continue
        passes = [[i for i in p if plan.bars[i][3]] for p in plan.passes]
        if len(passes) < 4 or any(len(p) != 8 for p in passes[:4]): continue
        nsym = sum(len(plan.bars[i][1]) for p in passes[:4] for i in p)
        if nsym / 32 < 0.4: continue
        n += 1; current = None; tonic = plan.key_tonic_pc
        for pi, p in enumerate(passes[:4]):
            part = "AB"[pi // 2]; rep = "1" if pi % 2 == 0 else "2"
            changes = 0
            for j, bi in enumerate(p, 1):
                for off, sym in sorted(plan.bars[bi][1], key=lambda c: float(c[0])):
                    cid = ident(sym)
                    if cid is None: continue
                    if current is not None and cid != current:
                        changes += 1
                    current = cid
                    st[f"{part}_all"][cls(cid, tonic)] += 1
                if current and j in (4, 8):
                    st[f"{part}{rep}_m{j}"][cls(current, tonic)] += 1
            st[f"{part}_changes"]["bars"] += 8; st[f"{part}_changes"]["changes"] += changes
    res[name] = {"airs": n, **{k: dict(v) for k, v in st.items()}}
Path(sys.argv[1]).write_text(json.dumps(res, ensure_ascii=False, indent=1))
for name, v in res.items():
    print(name, "airs AABB annotés:", v["airs"])
    if not v["airs"]: continue
    for part in "AB":
        c = v[f"{part}_all"]; t = sum(c.values())
        ch = v[f"{part}_changes"]
        print("  ", part, "symboles", t, {k: round(100 * c.get(k, 0) / t) for k in ("I", "IV", "V", "vi", "ii", "bVII")}, "changements/mesure", round(ch["changes"] / ch["bars"], 2))
    for key in ("A1_m4", "A2_m4", "A1_m8", "A2_m8", "B1_m4", "B2_m4", "B1_m8", "B2_m8"):
        c = v.get(key, {}); t = sum(c.values())
        if t: print("  ", key, {k: round(100 * c[k] / t) for k in sorted(c, key=lambda x: -c[x])[:3]})
