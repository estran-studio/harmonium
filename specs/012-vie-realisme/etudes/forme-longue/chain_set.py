#!/usr/bin/env python3
"""Vérification pratique V7a : chaîne 3 exports CLI en un set hors moteur (aucun code du dépôt modifié).

1. règle de set proposée (poids de base du profil × facteur par classe de transition) -> 3 tonalités
2. pour chaque tonalité : une graine CLI dont l'air 1 (tune 0) est dans cette tonalité
3. export MIDI par air avec automation (arousal du jeu constant + rampe de set : tempo plafonné, intensité)
4. assemblage : air 1 complet (intro + tours 1-2) ; airs 2/3 = mesure de levée + tours 2-3 (toutes les pistes
   jouent dès A1, comme `intro_each_tune = false`) ; tempo/vélocités tels que les produit le moteur
5. rendu fluidsynth + GeneralUser-GS
"""
import collections, json, math, random, re, subprocess, sys
from pathlib import Path
import mido

sys.path.insert(0, '/tmp/set_etude')
from measure import classify  # même classification que les mesures

BIN = '/tmp/set_etude/target/release/harmonium-cli'
WORK = Path('/tmp/set_etude/work')
WORK.mkdir(exist_ok=True)
SF2 = '/Users/william.quintal/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2'
OUT = Path('/Users/william.quintal/Project/bascanada/harmonium_lab/target/set_etude')
CWD = '/Users/william.quintal/orca/workspaces/harmonium/vie-v7a-sets'

# ------------------------------------------------------------------ règle proposée (reel_quebecois)
KEY_MODES = [  # profiles/reel_quebecois/profile.toml : melody.key_modes
    ((2, 'major'), 3, 'D ionien'), ((7, 'major'), 3, 'G ionien'), ((9, 'major'), 2, 'A ionien'),
    ((9, 'mixolydian'), 1, 'A mixolydien'), ((2, 'mixolydian'), 1, 'D mixolydien')]
LIFT = {'même tonalité': 1.28, 'même tonique, autre mode': 1.00, 'relatif (même collection)': 1.02,
        'tonique +5te (dominante)': 0.97, 'tonique +4te (sous-dominante)': 1.01, 'tonique +1 ton': 0.91,
        'tonique −1 ton': 0.73, 'autre': 0.75}  # mesures The Session, reel→reel : rapport observé / produit des marginales
ENGINE_MODE = {'D ionien': 2, 'G ionien': 7, 'A ionien': 9, 'A mixolydien': 9, 'D mixolydien': 2}


def next_weights(prev):
    return [(name, w * LIFT[classify(prev, k)]) for k, w, name in KEY_MODES]


def draw(weights, u):
    tot = sum(w for _, w in weights)
    x = u * tot
    for name, w in weights:
        if x < w:
            return name
        x -= w
    return weights[-1][0]


KEY_OF = {n: k for k, _, n in KEY_MODES}


def plan_set(seed, n=3):
    rnd = random.Random(seed)
    us = [rnd.random() for _ in range(n)]  # tirages fixés au début du set (u_1..u_n), résolus ensuite
    names = [draw([(nm, w) for _, w, nm in KEY_MODES], us[0])]
    for u in us[1:]:
        names.append(draw(next_weights(KEY_OF[names[-1]]), u))
    return names


# ------------------------------------------------------------------ outils CLI
def tune0_key(seed):
    out = subprocess.run([BIN, '--export', '--bars', '32', '--tours', '1', '--seed', str(seed), '--style', 'reel_quebecois'],
                         capture_output=True, text=True, cwd=CWD, timeout=60).stdout
    m = re.search(r'Tune 1 .*?: (\S+) (\w+)', out)
    return f"{m.group(1)} {({'Ionian': 'ionien', 'Mixolydian': 'mixolydien'})[m.group(2)]}"


def find_seed(name, start=1):
    for s in range(start, start + 400):
        if tune0_key(s) == name:
            return s
    raise SystemExit(f'pas de graine pour {name}')


# courbes du profil reel_quebecois (dynamics.toml)
def interp(points, x):
    if x <= points[0][0]:
        return points[0][1]
    for (x0, y0), (x1, y1) in zip(points, points[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0)
    return points[-1][1]


TEMPO_OFFSET = [[0.0, -10.0], [0.5, 0.0], [1.0, 15.0]]
AROUSAL_INTENSITY = [[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]]
CEILING_BPM = 240.0  # meter.max_bpm
BASE_BPM = 230.0
RAMP_TEMPO = [[0.0, 0.0], [1.0, 8.0]]      # bpm ajoutés selon la progression du set (0 → 1)
RAMP_INTENSITY = [[0.0, 0.0], [1.0, 0.08]]  # ajouté à l'intensité (via l'arousal)
GATE = [[0.0, 0.0], [0.2, 1.0], [0.8, 1.0], [1.0, 0.0]]  # selon l'arousal du jeu


def set_params(progress, game_arousal):
    gate = interp(GATE, game_arousal)
    arousal = min(1.0, game_arousal + interp(RAMP_INTENSITY, progress) * gate)
    emo_tempo = BASE_BPM + interp(TEMPO_OFFSET, interp(AROUSAL_INTENSITY, arousal))
    base_tempo = BASE_BPM + interp(TEMPO_OFFSET, interp(AROUSAL_INTENSITY, game_arousal))
    lift = interp(RAMP_TEMPO, progress) * gate
    final = emo_tempo + min(lift, max(0.0, CEILING_BPM - emo_tempo))
    nominal = final - interp(TEMPO_OFFSET, interp(AROUSAL_INTENSITY, arousal))  # `bpm` d'automation (tempo nominal)
    return arousal, final, nominal, base_tempo


def export_air(idx, seed, arousal, nominal):
    auto = WORK / f'auto{idx}.toml'
    auto.write_text(f'[[keys]]\nbar = 1\narousal = {arousal:.4f}\nbpm = {nominal:.3f}\n')
    mid = WORK / f'air{idx}.mid'
    truth = WORK / f'air{idx}.json'
    r = subprocess.run([BIN, '--export', '--bars', '100', '--tours', '3', '--seed', str(seed), '--style', 'reel_quebecois',
                        '--automation', str(auto), '--record-midi', str(mid), '--record-truth', str(truth)],
                       capture_output=True, text=True, cwd=CWD, timeout=120)
    if r.returncode:
        raise SystemExit(r.stderr)
    t = json.load(open(truth))
    pos = {m['bar']: m['measure']['position'] for m in t['measures']}
    tempo_of = {m['bar']: m['measure']['tempo'] for m in t['measures']}
    intro = sorted(b for b, p in pos.items() if p['tune_index'] == 0 and p['section'] == 'Intro')
    tour_bars = sorted(b for b, p in pos.items() if p['tune_index'] == 0 and p['section'] == 'Tour')
    return mid, len(intro), tour_bars[0], tour_bars[-1], tempo_of[tour_bars[0] + 1]


def abs_events(mid):
    out = []
    for tr in mid.tracks:
        a = 0
        for m in tr:
            a += m.time
            out.append((a, m))
    out.sort(key=lambda x: x[0])
    return out


def build(segments, tempos, path):
    """segments : [(midi, [(tick0, tick1), ...])] ; un tempo par segment. Coupe les notes tenues aux bornes."""
    new = mido.MidiFile(type=0, ticks_per_beat=480)
    tr = mido.MidiTrack()
    new.tracks.append(tr)
    ev = []  # (tick, ordre, msg)
    cursor = 0
    for n, ((mid, ranges), tempo) in enumerate(zip(segments, tempos)):
        evs = abs_events(mid)
        if n == 0:
            for a, m in evs:
                if a == 0 and m.type in ('program_change', 'control_change', 'time_signature'):
                    ev.append((0, 0, m))
        ev.append((cursor, 0, mido.MetaMessage('set_tempo', tempo=mido.bpm2tempo(tempo))))
        for rng in ranges:
            t0, t1 = rng[0], rng[1]
            chans = rng[2] if len(rng) > 2 else None
            active = collections.Counter()
            for a, m in evs:
                if not (t0 <= a < t1) or (chans is not None and getattr(m, 'channel', -1) not in chans):
                    continue
                if m.type == 'note_on' and m.velocity > 0:
                    active[(m.channel, m.note)] += 1
                    ev.append((cursor + a - t0, 2, m.copy()))
                elif m.type == 'note_off' or (m.type == 'note_on' and m.velocity == 0):
                    k = (m.channel, m.note)
                    if active[k] > 0:
                        active[k] -= 1
                        ev.append((cursor + a - t0, 1, m.copy()))
            for (ch, note), c in active.items():
                for _ in range(c):
                    ev.append((cursor + t1 - t0, 1, mido.Message('note_off', channel=ch, note=note, velocity=0)))
            cursor += t1 - t0
    ev.sort(key=lambda x: (x[0], x[1]))
    last = 0
    for a, _, m in ev:
        m.time = a - last
        last = a
        tr.append(m)
    tr.append(mido.MetaMessage('end_of_track', time=0))
    new.save(path)


def stats(path):
    mid = mido.MidiFile(path)
    tempo = 500000
    abst = 0
    rows = collections.defaultdict(list)
    seg_tempo = []
    notes = 0
    t = 0
    for m in mido.merge_tracks(mid.tracks):
        t += m.time
        if m.type == 'set_tempo':
            seg_tempo.append((t, mido.tempo2bpm(m.tempo)))
        if m.type == 'note_on' and m.velocity > 0:
            rows[m.channel].append((t, m.velocity))
    return seg_tempo, rows


if __name__ == '__main__':
    set_seed = int(sys.argv[1]) if len(sys.argv) > 1 else 1
    names = plan_set(set_seed)
    print('règle : set de 3 airs, tirage hors moteur (graine de set', set_seed, ') ->', ' → '.join(names))
    for a, b in zip(names, names[1:]):
        print(f'  {a} → {b} : {classify(KEY_OF[a], KEY_OF[b])}')
    game_arousal = 0.5
    seeds, used = [], 0
    segs, tempos, info = [], [], []
    for i, nm in enumerate(names):
        p = i / (len(names) - 1)
        seed = find_seed(nm, seeds[-1] + 1 if seeds else 1)
        seeds.append(seed)
        arousal, final, nominal, base_tempo = set_params(p, game_arousal)
        mid, intro_bars, t0, t1, eng = export_air(i, seed, arousal, nominal)
        m = mido.MidiFile(mid)
        bt = 1920
        tour = 32
        if i == 0:
            ranges = [(0, (intro_bars + 2 * tour) * bt)]  # intro + tours 1 et 2
        else:
            # levée seule (mélodie, 2 derniers temps de la mesure d'intro), puis tours 2 et 3
            ranges = [(intro_bars * bt - 960, intro_bars * bt, {0}), ((intro_bars + tour) * bt, (intro_bars + 3 * tour) * bt)]
        segs.append((m, ranges))
        # tempo effectivement joué (snapshot de la 2ᵉ mesure du tour)
        tempos.append(eng)
        info.append((nm, seed, p, arousal, final, eng, intro_bars))
        print(f'air {i+1} : {nm}, graine CLI {seed}, progression {p:.2f}, arousal {arousal:.3f}, tempo visé {final:.1f}, tempo moteur {eng:.1f}, intro {intro_bars} mesure(s)')
    out = WORK / 'set_demo.mid'
    build(segs, tempos, out)
    json.dump({'names': names, 'info': info}, open(WORK / 'set_demo.json', 'w'))
    print('MIDI :', out)
