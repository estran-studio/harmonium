"""Réglage de l'amplitude et du décalage de niveau des accents (point 2) : augmente l'amplitude jusqu'à un écart-type minimal de 12,3 sur
16 graines pour les accords (10,3 pour la basse) et recale le niveau moyen sur celui de 019 ; valeurs retenues dans `AMPLITUDE` de `appliquer.py`.

    python3 specs/020-qualite-des-styles/q0/tuner_velocites.py [profil...]   (profils de 019 extraits dans $H020_WORK/base/profiles)
"""
import sys, json, subprocess, statistics as st, tempfile, shutil, os
from pathlib import Path
from multiprocessing import Pool
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import appliquer as A
WORK = os.environ.get('H020_WORK', '/tmp/h020-q0')
CLI = os.environ.get('HARMONIUM_CLI', str(Path(__file__).resolve().parents[3] / 'target/release/harmonium-cli'))
REPO = A.REPO

def build_dir(pid, tag):
    root = Path(f'{WORK}/tv/{tag}'); 
    if root.exists(): shutil.rmtree(root)
    (root / 'profiles').mkdir(parents=True)
    shutil.copytree(REPO / 'profiles' / pid, root / 'profiles' / pid)
    A.build(pid, root)
    return root / 'profiles' / pid

def one(args):
    d, seed = args
    out = f'{d}/t{seed}.json'
    subprocess.run([CLI, '--export', '--bars', '48', '--profile', str(d), '--seed', str(seed), '--record-truth', out], capture_output=True, check=True)
    last = {}
    for rec in json.load(open(out))['measures']: last[rec['measure']['index']] = rec['measure']
    v = {1: [], 2: []}
    for i in sorted(last):
        if i > 47: continue
        for n in last[i]['notes']:
            if n['track'] in v: v[n['track']].append(n['velocity'])
    return {t: (st.pstdev(x) if len(x) > 1 else 0, min(x, default=0), max(x, default=0), st.mean(x) if x else 0) for t, x in v.items()}

def measure(pool, pid, tag):
    d = build_dir(pid, tag)
    r = pool.map(one, [(d, s) for s in range(1, 17)])
    return {t: (min(x[t][0] for x in r), min(x[t][1] for x in r), max(x[t][2] for x in r), st.mean(x[t][3] for x in r)) for t in (1, 2)}

def base_means(pool, pid):
    d = Path(f'{WORK}/base/profiles') / pid
    r = pool.map(one, [(d, s) for s in range(1, 17)])
    return {t: st.mean(x[t][3] for x in r) for t in (1, 2)}

if __name__ == '__main__':
    pids = sys.argv[1:] or A.E1
    with Pool(12) as pool:
        for pid in pids:
            ac, ab = A.AMPLITUDE[pid]
            bm = base_means(pool, pid)
            tc, tb = 12.3, 10.3
            for it in range(14):
                A.AMPLITUDE[pid] = (ac, ab)
                m = measure(pool, pid, pid)
                dc = bm[1] - m[1][3]; db = bm[2] - m[2][3]
                ok_c = m[1][0] >= tc and abs(dc) <= 1.5
                ok_b = True if (ab is None and pid not in A.WALKING) else (m[2][0] >= tb and abs(db) <= 1.5)
                print(pid, it, ac, ab, {t: tuple(round(x, 1) for x in v) for t, v in m.items()}, 'base', {t: round(v, 1) for t, v in bm.items()}, flush=True)
                if ok_c and ok_b: break
                name, amp, bias = ac
                if m[1][0] < tc: amp += 2
                ac = (name, amp, round(bias + dc * 0.9))
                if pid in A.WALKING:
                    A.WALKING[pid]['velocity'] += round(db * 0.9)
                if ab:
                    name, amp, bias = ab
                    if m[2][0] < tb: amp += 1
                    ab = (name, amp, round(bias + db * 0.9))
            print('FINAL', pid, repr(ac), repr(ab), flush=True)
