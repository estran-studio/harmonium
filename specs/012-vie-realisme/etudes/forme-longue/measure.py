#!/usr/bin/env python3
"""Mesures V7a : enchaînements de tonalités/types dans des sets réels.
Sources : TheSession-data (sets.json, recordings.json, tunes.json, ODbL) ;
manifestes du labo (distribution par type). Rien n'est écrit hors /tmp/set_etude.
"""
import collections, json, math, re, sys, unicodedata, random
from pathlib import Path

LAB = Path('/Users/william.quintal/Project/bascanada/harmonium_lab')
RAW = LAB / 'corpus/trad/raw/thesession'
OUT = Path('/tmp/set_etude')

PC = {'C': 0, 'D': 2, 'E': 4, 'F': 5, 'G': 7, 'A': 9, 'B': 11}
MODE_OFF = {'major': 0, 'mixolydian': 5, 'dorian': 10, 'minor': 3}  # tonique + off = tonique ionienne équivalente
MODE_NAME = {'major': 'ionian', 'minor': 'aeolian', 'dorian': 'dorian', 'mixolydian': 'mixolydian'}


def parse_mode(s):
    m = re.fullmatch(r'([A-G])([#b]?)(major|minor|dorian|mixolydian)', s or '')
    if not m:
        return None
    pc = PC[m.group(1)] + (1 if m.group(2) == '#' else -1 if m.group(2) == 'b' else 0)
    return pc % 12, m.group(3)


def collection(k):
    pc, mode = k
    return (pc + MODE_OFF[mode]) % 12


def sig_pos(c):  # position sur le cycle des quintes (C=0, G=1, D=2 ...)
    p = (c * 7) % 12
    return p if p <= 6 else p - 12


def classify(a, b):
    """Classe de transition (priorité : même, parallèle, relatif, intervalle de tonique)."""
    (pa, ma), (pb, mb) = a, b
    if (pa, ma) == (pb, mb):
        return 'même tonalité'
    if pa == pb:
        return 'même tonique, autre mode'
    if collection(a) == collection(b):
        return 'relatif (même collection)'
    iv = (pb - pa) % 12
    return {7: 'tonique +5te (dominante)', 5: 'tonique +4te (sous-dominante)', 2: 'tonique +1 ton',
            10: 'tonique −1 ton'}.get(iv, 'autre')


CLASSES = ['même tonalité', 'même tonique, autre mode', 'relatif (même collection)',
           'tonique +5te (dominante)', 'tonique +4te (sous-dominante)', 'tonique +1 ton',
           'tonique −1 ton', 'autre']


def dsig(a, b):
    d = (sig_pos(collection(b)) - sig_pos(collection(a)))
    d = (d + 6) % 12 - 6
    return d


def fold(s):
    return ''.join(c for c in unicodedata.normalize('NFD', s.lower()) if unicodedata.category(c) != 'Mn')


def table(counter, order=None, total=None):
    total = total or sum(counter.values())
    keys = order or [k for k, _ in counter.most_common()]
    return [(k, counter.get(k, 0), 100 * counter.get(k, 0) / total if total else 0) for k in keys]


def pr(title, rows):
    print(f'\n### {title}')
    print('| classe | n | % |')
    print('|---|---:|---:|')
    for k, n, p in rows:
        print(f'| {k} | {n} | {p:.1f} |')


def analyse(name, seqs, types_of=None):
    """seqs : liste de listes de (key, type). Retourne un dict de résultats et imprime les tableaux."""
    lens = collections.Counter(len(s) for s in seqs)
    print(f'\n## {name}')
    print(f'sets : {len(seqs)} ; longueurs : ' + ', '.join(f'{k}:{v}' for k, v in sorted(lens.items()) if k <= 8))
    tot = sum(len(s) for s in seqs)
    print(f'airs : {tot} ; longueur moyenne {tot/len(seqs):.2f}')
    pairs = [(s[i], s[i + 1]) for s in seqs for i in range(len(s) - 1)]
    print(f'transitions : {len(pairs)}')
    return pairs


def transition_tables(name, pairs, marginal_pool=None, n_perm=20000, seed=1):
    cls = collections.Counter(classify(a[0], b[0]) for a, b in pairs)
    pr(f'{name} : classe de transition de tonalité', table(cls, CLASSES))
    ds = collections.Counter(dsig(a[0], b[0]) for a, b in pairs)
    print('\n| Δ armure (quintes, + = dièses) | n | % |')
    print('|---|---:|---:|')
    for d in range(-5, 7):
        if ds.get(d):
            print(f'| {d:+d} | {ds[d]} | {100*ds[d]/len(pairs):.1f} |')
    near = sum(v for d, v in ds.items() if abs(d) <= 1)
    print(f'\nvoisines (|Δ armure| ≤ 1) : {100*near/len(pairs):.1f} %')
    # base d'indépendance : tirage indépendant dans la distribution marginale des airs de même type
    rnd = random.Random(seed)
    pool = marginal_pool or [p[1] for p in pairs]
    bytype = collections.defaultdict(list)
    for k, t in pool:
        bytype[t].append(k)
    base = collections.Counter()
    nsim = 0
    for _ in range(n_perm):
        a, b = rnd.choice(pairs)
        ka = rnd.choice(bytype[a[1]])
        kb = rnd.choice(bytype[b[1]])
        base[classify(ka, kb)] += 1
        nsim += 1
    print('\n| classe | observé % | indépendance % | rapport |')
    print('|---|---:|---:|---:|')
    for c in CLASSES:
        o = 100 * cls.get(c, 0) / len(pairs)
        e = 100 * base.get(c, 0) / nsim
        print(f'| {c} | {o:.1f} | {e:.1f} | {o/e if e else float("nan"):.2f} |')
    up = collections.Counter()
    for a, b in pairs:
        iv = (b[0][0] - a[0][0]) % 12
        if iv:
            up['monte (1..6 demi-tons)' if iv <= 6 else 'descend'] += 1
    print(f'\nmouvement de tonique (hors même tonique) : {dict(up)}')


def type_matrix(name, seqs, top=8):
    c = collections.Counter()
    for s in seqs:
        for i in range(len(s) - 1):
            c[(s[i][1], s[i + 1][1])] += 1
    tot = sum(c.values())
    print(f'\n### {name} : transitions de type (n={tot})')
    print('| de → vers | n | % |')
    print('|---|---:|---:|')
    for (a, b), n in c.most_common(top):
        print(f'| {a} → {b} | {n} | {100*n/tot:.1f} |')


# ---------------------------------------------------------------- The Session : sets de membres
def sessions_sets():
    data = json.load(open(OUT / 'sets.json'))
    by = collections.defaultdict(list)
    for r in data:
        k = parse_mode(r['mode'])
        by[r['tuneset']].append((int(r['settingorder']), k, r['type'], r['tune_id']))
    seqs, skipped = [], 0
    for sid, mem in by.items():
        mem.sort(key=lambda x: x[0])
        if any(m[1] is None for m in mem):
            skipped += 1
            mem = [m for m in mem if m[1] is not None]
        if len(mem) >= 2:
            seqs.append([(m[1], m[2]) for m in mem])
    print(f'sets.json : {len(by)} sets, {skipped} avec mode inconnu (membres écartés)')
    return seqs


# ---------------------------------------------------------------- enregistrements d'artistes
def artist_names(path):
    d = json.load(open(path))
    s = set()
    for a in d['artists']:
        for n in a['session_names']:
            s.add(fold(n))
    return s


def tune_keys():
    t = json.load(open(RAW / 'tunes.json'))
    per = collections.defaultdict(list)
    for r in t:
        k = parse_mode(r['mode'])
        if k:
            per[r['tune_id']].append((k, r['type']))
    out = {}
    for tid, lst in per.items():
        c = collections.Counter(lst)
        out[tid] = c.most_common(1)[0][0]
    return out


def recordings_sets(names, tk, label):
    rec = json.load(open(RAW / 'recordings.json'))
    tracks = collections.defaultdict(list)
    albums = collections.defaultdict(set)
    for r in rec:
        if fold(r['artist']) in names:
            tracks[(r['id'], r['track'])].append((int(r['number']), r['tune_id'], r['tune']))
            albums[r['id']].add(r['recording'])
    lens = collections.Counter(len(v) for v in tracks.values())
    print(f'\n## {label}')
    print(f'enregistrements : {len(albums)} ; pistes : {len(tracks)} ; longueur des pistes (nb d\'airs) : '
          + ', '.join(f'{k}:{v}' for k, v in sorted(lens.items()) if k <= 8))
    seqs = []
    mapped = unmapped = 0
    for key, v in tracks.items():
        v.sort()
        if len(v) < 2:
            continue
        seq = []
        for _, tid, _ in v:
            if tid in tk:
                seq.append(tk[tid])
                mapped += 1
            else:
                seq.append(None)
                unmapped += 1
        # on découpe aux airs inconnus : seules les paires consécutives connues comptent
        cur = []
        for x in seq:
            if x is None:
                if len(cur) >= 2:
                    seqs.append(cur)
                cur = []
            else:
                cur.append(x)
        if len(cur) >= 2:
            seqs.append(cur)
    print(f'airs de pistes ≥2 airs : mappés {mapped}, sans tune_id/mode {unmapped}')
    return seqs, tracks


def signature(tracks_by_artist, names, tk, label):
    """Airs qui reviennent : même tune_id dans ≥2 enregistrements distincts d'un même artiste."""
    rec = json.load(open(RAW / 'recordings.json'))
    per = collections.defaultdict(lambda: collections.defaultdict(set))
    for r in rec:
        if fold(r['artist']) in names and r['tune_id']:
            per[r['artist']][r['tune_id']].add(r['id'])
    tot_tunes = ret = 0
    best = []
    for a, d in per.items():
        n = len(d)
        k = sum(1 for v in d.values() if len(v) >= 2)
        tot_tunes += n
        ret += k
        best.append((n, k, a))
    best.sort(reverse=True)
    print(f'\n### {label} : retours d\'airs entre enregistrements d\'un même artiste')
    print(f'airs distincts (artiste, air) : {tot_tunes} ; présents dans ≥2 enregistrements : {ret} ({100*ret/tot_tunes:.1f} %)')
    for n, k, a in best[:6]:
        print(f'  {a}: {n} airs, {k} reviennent ({100*k/n:.0f} %)')


# ---------------------------------------------------------------- distribution par type (corpus du labo)
def corpus_marginals():
    FR = {'majeur': 'major', 'mineur': 'minor', 'dorien': 'dorian', 'mixolydien': 'mixolydian'}
    out = {}
    for d in ['trad', 'acadienne', 'britannique_coloniale']:
        rows = [json.loads(l) for l in open(LAB / f'corpus/{d}/manifest.jsonl')]
        pool = []
        for r in rows:
            m = re.fullmatch(r'([A-G])([#b]?)', r['key'])
            if not m or r['mode'] not in FR:
                continue
            pc = (PC[m.group(1)] + (1 if m.group(2) == '#' else -1 if m.group(2) == 'b' else 0)) % 12
            pool.append(((pc, FR[r['mode']]), r['type']))
        out[d] = pool
    return out


def marg_tables(out):
    NAMES = {0: 'C', 1: 'C#', 2: 'D', 3: 'Eb', 4: 'E', 5: 'F', 6: 'F#', 7: 'G', 8: 'Ab', 9: 'A', 10: 'Bb', 11: 'B'}
    for d, pool in out.items():
        bytype = collections.defaultdict(collections.Counter)
        for (pc, m), t in pool:
            bytype[t][f'{NAMES[pc]} {MODE_NAME[m]}'] += 1
        print(f'\n### {d} : tonalité × mode par type (corpus du labo)')
        for t, c in sorted(bytype.items(), key=lambda kv: -sum(kv[1].values())):
            n = sum(c.values())
            if n < 10:
                continue
            top = ', '.join(f'{k} {100*v/n:.0f} %' for k, v in c.most_common(6))
            print(f'- {t} (n={n}) : {top}')
        # probabilité de transition « voisine » sous indépendance, par type
        print('\n| type | n | P(même tonalité) | P(relatif) | P(+5te) | P(+4te) | P(|Δ armure|≤1) |')
        print('|---|---:|---:|---:|---:|---:|---:|')
        for t, c in sorted(bytype.items(), key=lambda kv: -sum(kv[1].values())):
            keys = [k for (k, tt) in pool if tt == t]
            n = len(keys)
            if n < 10:
                continue
            cnt = collections.Counter()
            tot = 0
            for a in keys:
                for b in keys:
                    cnt[classify(a, b)] += 1
                    if abs(dsig(a, b)) <= 1:
                        cnt['near'] += 1
                    tot += 1
            print(f'| {t} | {n} | {100*cnt["même tonalité"]/tot:.1f} | {100*cnt["relatif (même collection)"]/tot:.1f} | '
                  f'{100*cnt["tonique +5te (dominante)"]/tot:.1f} | {100*cnt["tonique +4te (sous-dominante)"]/tot:.1f} | {100*cnt["near"]/tot:.1f} |')


if __name__ == '__main__':
    which = sys.argv[1] if len(sys.argv) > 1 else 'all'
    if which in ('all', 'sessions'):
        seqs = sessions_sets()
        pairs = analyse('The Session — sets de membres (tous répertoires, majorité irlandaise)', seqs)
        transition_tables('The Session sets', pairs)
        type_matrix('The Session sets', seqs, 10)
        # sous-ensemble reel→reel et jig→jig
        for t in ('reel', 'jig'):
            sub = [(a, b) for a, b in pairs if a[1] == t and b[1] == t]
            print(f'\n#### The Session sets : {t}→{t} seulement (n={len(sub)})')
            transition_tables(f'The Session {t}→{t}', sub)
        # set de 3 airs : tonique du 3e par rapport au 1er
        three = [s for s in seqs if len(s) == 3]
        fl = [(s[0], s[2]) for s in three]
        cls = collections.Counter(classify(a[0], b[0]) for a, b in fl)
        pr(f'sets de 3 airs : 1er → 3e (n={len(fl)})', table(cls, CLASSES))
    if which in ('all', 'recordings'):
        tk = tune_keys()
        for label, f in (('Québec', 'quebec_artists.json'), ('Acadie', 'acadian_artists.json')):
            names = artist_names(LAB / 'src/harmonium_lab/trad' / f)
            seqs, tracks = recordings_sets(names, tk, f'Enregistrements d\'artistes {label} (The Session)')
            pairs = analyse(f'Pistes ≥2 airs, artistes {label}', seqs)
            if pairs:
                transition_tables(f'artistes {label}', pairs, marginal_pool=[p for s in seqs for p in s])
                type_matrix(f'artistes {label}', seqs, 8)
            signature(tracks, names, tk, label)
    if which in ('all', 'corpus'):
        marg_tables(corpus_marginals())
