"""Statistiques de groove par fichier du GMD (médiane sur les fichiers `beat`), lues dans le cache vérifié du labo (lecture seule).

Signature d'une mesure : l'ensemble des (pas, grosse caisse | caisse claire) de vélocité >= 40 (grosse caisse 35/36, caisse claire 37/38/40,
cross-stick compris). Trois nombres par fichier : part de la mesure la plus fréquente, mesures distinctes / mesures, part des mesures
identiques à la précédente ; médiane sur les fichiers `beat` 4/4 d'au moins `MINBARS` mesures (voir `mesure.py`).
"""
import sys, os, statistics, collections
from pathlib import Path
REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / 'specs/019-bibliotheque-accompagnements/b8'))
os.environ.setdefault('H019_GMD_CACHE', '/tmp/h020-q0/gmd_results.pkl')
import gmd_batterie as B
LAB = Path.home() / 'Project/bascanada/harmonium_lab'
VMIN = 40

def sig_gmd(m):
    s = set()
    for role, tag in (('kick', 'k'), ('snare', 's'), ('sidestick', 's')):
        for step, v in m.steps.get(role, {}).items():
            if v[0] >= VMIN: s.add((step, tag))
    return frozenset(s)

def stats(sigs):
    n = len(sigs)
    if n < 2: return None
    c = collections.Counter(sigs)
    top = max(c.values()) / n
    distinct = len(c) / n
    same = sum(1 for a, b in zip(sigs, sigs[1:]) if a == b) / (n - 1)
    return top, distinct, same

# profil -> (styles, signature, feels)
GROUPS = {
 'rock': (['rock'], '4-4', {'straight'}),
 'funk': (['funk'], '4-4', {'straight'}),
 'pop': (['pop', 'pop/soft', 'pop/groove7'], '4-4', {'straight'}),
 'soul': (['soul', 'soul/motown', 'soul/groove3', 'soul/groove4', 'soul/groove9', 'soul/groove10'], '4-4', {'straight'}),
 'reggae': (['reggae', 'reggae/slow'], '4-4', {'straight'}),
 'latin': (['latin'] , '4-4', {'straight'}),
}

def files(results, styles, sig, feels, family=None):
    out = []
    for r in results:
        if r.row.beat_type != 'beat' or r.reject: continue
        if r.row.time_signature != sig or r.feel not in feels: continue
        st = r.row.style
        if (family and st.split('/')[0] == family) or st in styles:
            out.append(r)
    return out

def group_stats(results, styles, sig, feels, family=None):
    fs = files(results, styles, sig, feels, family)
    per = []; lens = []
    for r in fs:
        sigs = [sig_gmd(m) for m in r.measures]
        s = stats(sigs)
        if s: per.append(s); lens.append(len(sigs))
    if not per: return None
    med = lambda i: statistics.median(p[i] for p in per)
    return dict(files=len(per), bars=int(statistics.median(lens)), top=med(0), distinct=med(1), same=med(2))

if __name__ == '__main__':
    res = B.load_results(LAB)
    for fam in ('rock', 'funk', 'pop', 'latin'):
        print(fam, group_stats(res, [], '4-4', {'straight'}, family=fam))
