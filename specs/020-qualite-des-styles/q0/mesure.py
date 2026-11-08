#!/usr/bin/env python3
"""Mesures Q0 sur 16 graines : vélocités, groove de batterie, mise en place.  Usage : mesure.py <racine profils> <étiquette> [profil...]"""
import json, os, subprocess, sys, statistics as st, collections
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gmdref as G

CLI = os.environ.get('HARMONIUM_CLI', str(HERE.parents[2] / 'target/release/harmonium-cli'))
WORK = Path(os.environ.get('H020_WORK', '/tmp/h020-q0'))
OUT = WORK / 'runs'
E1 = ["wjazzd_swing","wjazzd_ballad","gmd_jazz_waltz","gmd_latin_brazilian_bossa","gmd_latin_brazilian_samba","gmd_latin_brazilian_baiao","gmd_afrocuban","gmd_funk","gmd_soul","gmd_reggae","gmd_rock","gmd_pop","gmd_blues_shuffle"]
SEEDS = range(1, 17)
BARS = 48
KICK, SNARE, HAT = {35, 36}, {37, 38, 40}, {42, 44, 46, 51, 53, 59}
# groupe GMD par profil : (styles exacts, signature, feels, bpm max)
REF = {
 'wjazzd_swing': (['jazz','jazz/swing'], '4-4', {'swing','ternary'}, 999),
 'wjazzd_ballad': (['jazz'], '4-4', {'swing','ternary'}, 112),
 'gmd_jazz_waltz': (['jazz'], '3-4', {'swing','ternary','straight'}, 999),
 'gmd_latin_brazilian_bossa': (['latin/brazilian-bossa'], '4-4', {'straight'}, 999),
 'gmd_latin_brazilian_samba': (['latin/brazilian-samba'], '4-4', {'straight'}, 999),
 'gmd_latin_brazilian_baiao': (['latin/brazilian-baiao'], '4-4', {'straight'}, 999),
 'gmd_afrocuban': (['afrocuban','afrocuban/rhumba'], '4-4', {'straight'}, 999),
 'gmd_funk': (['funk'], '4-4', {'straight'}, 999),
 'gmd_soul': (G.GROUPS['soul'][0], '4-4', {'straight'}, 999),
 'gmd_reggae': (['reggae'], '4-4', {'straight'}, 999),
 'gmd_rock': (['rock'], '4-4', {'straight'}, 999),
 'gmd_pop': (G.GROUPS['pop'][0], '4-4', {'straight'}, 999),
 'gmd_blues_shuffle': (['blues/shuffle','rock/shuffle','funk/purdieshuffle'], '4-4', {'ternary'}, 999),
}
MINBARS = 8

def render(root, tag, pid, seed):
    d = OUT / tag; d.mkdir(parents=True, exist_ok=True)
    f = d / f'{pid}_{seed}.json'
    if not f.exists():
        r = subprocess.run([CLI, '--export', '--bars', str(BARS), '--profile', f'{root}/{pid}', '--seed', str(seed), '--record-truth', str(f)], capture_output=True, text=True)
        if r.returncode: raise SystemExit(r.stderr)
    last = {}
    for rec in json.load(open(f))['measures']:
        last[rec['measure']['index']] = rec['measure']
    return [last[i] for i in sorted(last) if i <= BARS - 1]

def ref_stats(res, pid):
    styles, sig, feels, bpm = REF[pid]
    per = []; lens = []
    for r in res:
        if r.row.beat_type != 'beat' or r.reject or r.row.style not in styles or r.row.time_signature != sig or r.feel not in feels or r.row.bpm > bpm: continue
        sigs = [G.sig_gmd(m) for m in r.measures]
        if len(sigs) < MINBARS: continue
        s = G.stats(sigs)
        per.append(s); lens.append(len(sigs))
    if not per: return None
    med = lambda i: st.median(p[i] for p in per)
    return dict(files=len(per), bars=int(st.median(lens)), top=med(0), distinct=med(1), same=med(2))

def sig_eng(m):
    s = set()
    for n in m['notes']:
        if n['track'] in (3, 4) and n['velocity'] >= G.VMIN:
            if n['pitch'] in KICK: s.add((n['start_step'], 'k'))
            elif n['pitch'] in SNARE: s.add((n['start_step'], 's'))
    return frozenset(s)

def groove(measures, window):
    sigs = [sig_eng(m) for m in measures]
    out = []
    for a in range(0, len(sigs) - window + 1, window):
        s = G.stats(sigs[a:a + window])
        if s: out.append(s)
    return out

def vel_stats(measures, track):
    v = [n['velocity'] for m in measures for n in m['notes'] if n['track'] == track]
    return v

def timing(measures):
    """asynchronies (ms) aux pas où deux voix attaquent ensemble ; dérive de la grille par piste."""
    bk, cs, ct, ccs = [], [], [], []   # basse-grosse caisse, accord(1re corde)-caisse claire, accord-timekeeper, accord(centroïde)-caisse claire
    drift = collections.defaultdict(list)
    for m in measures:
        spp = m['pulse']['steps_per_pulse']; ms = 60000.0 / m['tempo'] / spp
        by = collections.defaultdict(lambda: collections.defaultdict(list))
        for n in m['notes']:
            off = n.get('performance', {}).get('offset', 0) / 1000.0 * ms
            kind = None
            if n['track'] == 1: kind = 'chord'
            elif n['track'] == 2: kind = 'bass'
            elif n['track'] in (3, 4):
                if n['pitch'] in KICK: kind = 'kick'
                elif n['pitch'] in SNARE and n['velocity'] >= 40: kind = 'snare'
                elif n['pitch'] in HAT: kind = 'tk'
            if kind:
                by[n['start_step']][kind].append(off)
                drift[kind].append(abs(off))
        for step, d in by.items():
            if 'bass' in d and 'kick' in d: bk.append(d['bass'][0] - st.mean(d['kick']))
            if 'chord' in d:
                first = min(d['chord'])
                if 'snare' in d: cs.append(first - st.mean(d['snare'])); ccs.append(st.mean(d['chord']) - st.mean(d['snare']))
                if 'tk' in d: ct.append(first - st.mean(d['tk']))
    return dict(bk=bk, cs=cs, ct=ct, ccs=ccs, drift=drift)

def pct(xs, q):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))] if xs else None

def run(root, tag, pids):
    res = G.B.load_results(G.LAB)
    table = {}
    for pid in pids:
        ref = ref_stats(res, pid)
        g = []; vs = {1: [], 2: []}; vstd = {1: [], 2: []}
        tm = collections.defaultdict(list); drift = collections.defaultdict(list)
        window = min(ref['bars'], 47) if ref else 16
        for seed in SEEDS:
            ms = render(root, tag, pid, seed)
            g += groove(ms, window)
            for t in (1, 2):
                v = vel_stats(ms, t); vs[t] += v
                if len(v) > 1: vstd[t].append(st.pstdev(v))
            t = timing(ms)
            for k in ('bk', 'cs', 'ct', 'ccs'): tm[k] += t[k]
            for k, v in t['drift'].items(): drift[k] += v
        row = dict(ref=ref, window=window,
                   groove=dict(top=st.median(x[0] for x in g), distinct=st.median(x[1] for x in g), same=st.median(x[2] for x in g)),
                   vel={('chords' if t == 1 else 'bass'): dict(std_min=min(vstd[t]), std_med=st.median(vstd[t]), vmin=min(vs[t]), vmax=max(vs[t])) for t in (1, 2) if vs[t]},
                   timing={k: dict(n=len(v), med=st.median(v), sd=st.pstdev(v), p95abs=pct([abs(x) for x in v], .95)) for k, v in tm.items() if v},
                   drift={k: dict(med=st.median(v), p95=pct(v, .95)) for k, v in drift.items()})
        table[pid] = row
    return table

if __name__ == '__main__':
    root, tag = sys.argv[1], sys.argv[2]
    pids = sys.argv[3:] or E1
    t = run(root, tag, pids)
    (WORK / f'{tag}.json').write_text(json.dumps(t, indent=1))
    for pid, r in t.items():
        rf = r['ref']; g = r['groove']
        print(f"{pid:28} groove top/dist/same {g['top']:.2f}/{g['distinct']:.2f}/{g['same']:.2f}"
              + (f"  GMD {rf['top']:.2f}/{rf['distinct']:.2f}/{rf['same']:.2f} (n={rf['files']}, {rf['bars']}b)" if rf else ' GMD -'))
        v = r['vel']
        print('   vel ' + ' '.join(f"{k}: sd {x['std_min']:.1f}..{x['std_med']:.1f} [{x['vmin']},{x['vmax']}]" for k, x in v.items()))
        print('   t   ' + ' '.join(f"{k}: med {x['med']:+.1f} sd {x['sd']:.1f} p95 {x['p95abs']:.1f} (n{x['n']})" for k, x in r['timing'].items()))
        print('   drift p95 ' + ' '.join(f"{k} {x['p95']:.1f}" for k, x in r['drift'].items()))
