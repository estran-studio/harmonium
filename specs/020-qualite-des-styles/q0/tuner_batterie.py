"""Recherche des réglages de batterie (point 3) pour un profil : `n` tirages aléatoires de (fixed, extra, ghost_chance, prob_exp, hold,
fill_scale) puis une passe de coordonnées ; coût = somme des carrés des écarts, en unités de 0,05, des trois statistiques de groove au
groupe GMD du profil (16 graines). Les valeurs retenues sont dans `PARAMS` de `batterie.py`.

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/020-qualite-des-styles/q0/tuner_batterie.py <profil> [n] ['{"fixed": 0.9}']
"""
import sys, os, json, shutil, subprocess, statistics as st, itertools
from pathlib import Path
Q0 = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, Q0)
import batterie as Bt
import appliquer as A
import mesure as M
import gmdref as G
from concurrent.futures import ThreadPoolExecutor

KNOBS = dict(fixed=[1.0, .95, .9, .85, .8, .7, .6, .5, .4], extra=[0, 1, 2, 3, 4, 5, 6, 7, 8], ghost_chance=[.05, .1, .15, .2, .3, .45, .6, .8],
             prob_exp=[.5, .75, 1.0, 1.5, 2.0, 3.0], hold=[1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 64], fill_scale=[.25, .5, 1, 1.5, 2, 3, 4, 6, 8, 12, 16, 24, 32, 64, 128])
DEFAULT = dict(fixed=1.0, extra=0, ghost_chance=.45, prob_exp=1.0, hold=1, fill_scale=1)

def _render(args):
    d, seed, window = args
    out = Path(d) / f't{seed}.json'
    subprocess.run([M.CLI, '--export', '--bars', '48', '--profile', str(d), '--seed', str(seed), '--record-truth', str(out)], capture_output=True, check=True)
    last = {}
    for rec in json.load(open(out))['measures']: last[rec['measure']['index']] = rec['measure']
    ms = [last[i] for i in sorted(last) if i <= 47]
    out.unlink()
    return M.groove(ms, window)

POOL = None
def evaluate(pid, params, ref, tag):
    d = Path(os.environ.get('H020_WORK', '/tmp/h020-q0')) / 'dt' / tag / pid
    if d.exists(): shutil.rmtree(d)
    shutil.copytree(A.REPO / 'profiles' / pid, d)
    (d / 'percussion.toml').write_text(Bt.update(pid, A.git_show(f'profiles/{pid}/percussion.toml'), params['fixed'], params['extra'],
        params['prob_exp'], params['hold'], params['fill_scale'], params['ghost_chance']))
    window = min(ref['bars'], 47)
    g = [x for r in POOL.map(_render, [(str(d), s, window) for s in range(1, 17)]) for x in r]
    return tuple(st.median(x[i] for x in g) for i in range(3))

def cost(g, ref):
    t = (ref['top'], ref['distinct'], ref['same'])
    return sum(((a - b) / 0.05) ** 2 for a, b in zip(g, t))

if __name__ == '__main__':
    import random
    from multiprocessing import Pool
    POOL = Pool(int(os.environ.get('H020_JOBS', '8')))
    pid = sys.argv[1]
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    res = G.B.load_results(G.LAB)
    ref = M.ref_stats(res, pid)
    cur = dict(DEFAULT)
    if len(sys.argv) > 3: cur.update(json.loads(sys.argv[3]))
    best = evaluate(pid, cur, ref, pid); bc = cost(best, ref)
    print(pid, 'ref', ref, 'start', [round(x, 3) for x in best], round(bc, 2), flush=True)
    rnd = random.Random(7)
    rng_vals = dict(fixed=[1, .95, .9, .8, .7, .6, .5], extra=[0, 1, 2, 3, 4, 5, 6], ghost_chance=[.05, .1, .2, .3, .5],
                    prob_exp=[.5, .75, 1, 1.5, 2, 3], hold=[1, 2, 4, 8, 16, 32], fill_scale=[.5, 1, 2, 4, 8, 16, 64])
    for i in range(n):
        trial = {k: rnd.choice(v) for k, v in rng_vals.items()}
        g = evaluate(pid, trial, ref, pid); c = cost(g, ref)
        if c < bc - 1e-9:
            bc, best, cur = c, g, trial
            print(pid, 'rand', i, json.dumps(trial), [round(x, 3) for x in g], round(c, 2), flush=True)
    for k in KNOBS:
        for v in KNOBS[k]:
            if v == cur[k]: continue
            trial = dict(cur); trial[k] = v
            g = evaluate(pid, trial, ref, pid); c = cost(g, ref)
            if c < bc - 1e-9:
                bc, best, cur = c, g, trial
                print(pid, k, v, [round(x, 3) for x in g], round(c, 2), flush=True)
    print('FINAL', pid, json.dumps(cur), [round(x, 3) for x in best], 'ref', [round(ref[k], 3) for k in ('top', 'distinct', 'same')], round(bc, 2), flush=True)
