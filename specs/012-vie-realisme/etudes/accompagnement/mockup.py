"""Maquette hors moteur : remplace l'accompagnement d'un reel exporté par le CLI.

Jetable. Lit reel_sN.mid + reel_sN.json (--record-midi / --record-truth), recompose les canaux
1 (accords) et 2 (basse) selon la variante, rend via fluidsynth + GeneralUser GS, égalise le
volume moyen (RMS) de toutes les variantes d'un même seed.
Aucune des constantes ci-dessous n'est mesurée : ordres de grandeur à valider à l'oreille.
"""
import json, random, subprocess, sys
from pathlib import Path
import mido, numpy as np, soundfile as sf

SRC = Path("/tmp/accomp_etude/src"); OUT = Path(sys.argv[1]); OUT.mkdir(parents=True, exist_ok=True)
SF2 = "/Users/william.quintal/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2"
PPQ, STEP, BAR = 480, 120, 1920
MS_TICKS = None  # ticks par ms, fixé par le tempo
NAMES = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}
MAJOR = [0, 2, 4, 5, 7, 9, 11]

def pc_of(name):
    return (NAMES[name[0]] + name.count("#") - name.count("b")) % 12

def load(seed):
    mid = mido.MidiFile(SRC / f"reel_s{seed}.mid")
    global MS_TICKS
    tempo = next(m.tempo for m in mid.tracks[0] if m.type == "set_tempo")
    MS_TICKS = PPQ / (tempo / 1000)
    t, open_, notes = 0, {}, {0: [], 1: [], 2: []}
    for m in mid.tracks[0]:
        t += m.time
        if m.type == "note_on" and m.velocity > 0:
            open_[(m.channel, m.note)] = (t, m.velocity)
        elif m.type in ("note_off", "note_on") and (m.channel, m.note) in open_:
            s, v = open_.pop((m.channel, m.note))
            if m.channel in notes:
                notes[m.channel].append((s, t, m.note, v))
    truth = json.load(open(SRC / f"reel_s{seed}.json"))
    chords = {}
    for s in truth:
        chords[(s["bar"] - 1) * 2 + (0 if s["step"] < 8 else 1)] = (pc_of(s["chord"]), s["chord_is_minor"])
    return mid, notes, chords, tempo

def key_scale(melody):
    weight = np.zeros(12)
    for s, e, p, v in melody:
        weight[p % 12] += e - s
    best = max(range(12), key=lambda k: sum(weight[(k + i) % 12] for i in MAJOR))
    return {(best + i) % 12 for i in MAJOR}

def half_of(tick):
    return int(round(tick / STEP)) // 8  # index de demi-mesure

def chord_at(chords, tick):
    h = half_of(tick)
    while h >= 0 and h not in chords:
        h -= 1
    return chords.get(h, (0, False))

def place(pc, low, high):
    p = low + (pc - low) % 12
    return p if p <= high else p - 12

def scale_neighbor(p, scale, direction):
    q = p + direction
    while q % 12 not in scale:
        q += direction
    return q

def groups(notes):
    g = {}
    for s, e, p, v in sorted(notes):
        g.setdefault(int(round(s / STEP)), []).append((s, e, p, v))
    return g

def triad_voicing(root, minor, prev, lo=48, hi=67):
    tones = [root % 12, (root + (3 if minor else 4)) % 12, (root + 7) % 12]
    cands = []
    for inv in range(3):
        order = tones[inv:] + tones[:inv]
        for base in range(lo - 12, hi):
            v, last = [], base - 1
            for pc in order:
                n = last + 1 + (pc - last - 1) % 12
                v.append(n); last = n
            if lo <= v[0] and v[-1] <= hi and v[-1] - v[0] < 12:
                cands.append(v)
    ref = prev or [57, 60, 64]
    return min(cands, key=lambda v: sum(abs(a - b) for a, b in zip(v, ref)) + abs(sum(v) / 3 - 58) * 0.3)

class Ev(list):
    def add(self, ch, tick, dur, pitch, vel):
        self.append((float(tick), float(tick + max(dur, 8)), ch, int(pitch), max(1, min(127, int(vel)))))

def bass_line(bass_orig, chords, scale, rng, passing, lo=40, hi=50, wlo=38, whi=52):
    """Basse du moteur (fondamentale, quinte si accord inchangé) + notes de passage pondérées."""
    out, by_step = [], groups(bass_orig)
    bars = sorted({k // 16 for k in by_step})
    for b in range(bars[0], bars[-1] + 1) if bars else []:
        h0, h1 = chords.get(2 * b), chords.get(2 * b + 1, chords.get(2 * b))
        if h0 is None:
            continue
        p0 = place(h0[0], lo, hi)
        same = h1 == h0
        p8 = place((h1[0] + (7 if same else 0)) % 12, lo, hi)
        out += [(b * BAR, p0, 78, 4 * STEP), (b * BAR + 8 * STEP, p8, 78, 4 * STEP)]
        if not passing:
            continue
        nxt = chords.get(2 * b + 2)
        if h1 and h0 != h1 and rng.random() < 0.5:  # approche mi-mesure
            pa = scale_neighbor(place(h1[0], lo, hi), scale, 1 if p8 < place(h1[0], lo, hi) else -1)
            pa = pa if p0 != pa else p0
            out.append((b * BAR + 6 * STEP, pa, 66, 2 * STEP))
        if nxt and nxt[0] != h1[0] and rng.random() < 0.75:  # marche vers la mesure suivante
            tgt = place(nxt[0], lo, hi)
            direction = -1 if p8 < tgt else 1  # approche par le côté de départ
            if scale_neighbor(tgt, scale, direction) == p8:
                direction = -direction  # pas de note répétée : approche par l'autre côté
            leap = abs(tgt - p8)
            if rng.random() < 0.3:
                out.append((b * BAR + 12 * STEP, tgt - direction, 66, 2 * STEP))  # chromatique
            elif leap >= 5 and rng.random() < 0.5:
                n2 = scale_neighbor(tgt, scale, direction)
                n1 = scale_neighbor(n2, scale, direction)
                out += [(b * BAR + 12 * STEP, n1, 64, STEP), (b * BAR + 14 * STEP, n2, 68, STEP)]
            else:
                out.append((b * BAR + 12 * STEP, scale_neighbor(tgt, scale, direction), 66, 2 * STEP))
    return out

def build(variant, seed, notes, chords, rng):
    scale = key_scale(notes[0])
    chop = groups(notes[1]); ev = Ev()
    for s, e, p, v in notes[0]:
        ev.add(0, s, e - s, p, v)
    programs = {0: 40, 1: 24, 2: 32}
    use_new_bass = variant in ("passing", "strum_passing", "piano", "accordion")
    bass_src = notes[2]
    if variant in ("baseline",) or not use_new_bass:
        for s, e, p, v in bass_src:
            ev.add(2, s, e - s, p, v)
    elif variant in ("passing", "strum_passing"):
        for t, p, v, d in bass_line(bass_src, chords, scale, rng, True):
            ev.add(2, t, d, p, v)
    # accords
    if variant in ("baseline", "passing"):
        for s, e, p, v in notes[1]:
            ev.add(1, s, e - s, p, v)
    elif variant in ("strum", "strum_passing"):
        d_down, d_up, nstr = 7.0, 5.0, 4  # ms entre cordes : mesure Freire et al. 2018 (voir étude)
        for step, g in sorted(chop.items()):
            g = sorted(g, key=lambda x: x[2]); base = g[0][0]
            ps = [x[2] for x in g] + [g[0][2] + 12]
            for i, p in enumerate(ps):  # descente : graves -> aigus, centrée sur le temps
                off = (i - (len(ps) - 1) / 2) * d_down * rng.uniform(0.7, 1.3) * MS_TICKS
                ev.add(1, base + off, g[0][1] - g[0][0], p, g[0][3] + rng.randint(-5, 5) + (4 if i == 0 else 0))
            if step % 8 == 4 and rng.random() < 0.4:  # coup montant doux, croche suivante
                up = base + 2 * STEP
                for i, p in enumerate(reversed(ps[:3])):
                    off = (i - 1) * d_up * rng.uniform(0.7, 1.3) * MS_TICKS
                    ev.add(1, up + off, STEP, p, g[0][3] - 18 + rng.randint(-4, 4))
    elif variant == "piano":
        programs = {0: 40, 1: 0, 2: 0}
        for t, p, v, d in bass_line(bass_src, chords, scale, rng, True, 36, 47, 34, 50):
            ev.add(2, t, d, p, v + 4); ev.add(2, t, d, p + 12, v - 8)
        prev = None
        for step, g in sorted(chop.items()):
            root, minor = chord_at(chords, g[0][0])
            prev = triad_voicing(root, minor, prev)
            for p in prev:
                ev.add(1, g[0][0] + rng.randint(-3, 3), 2 * STEP, p, 58 + rng.randint(-5, 5) + (6 if step % 8 == 4 and step % 16 == 4 else 0))
    elif variant == "accordion":
        programs = {0: 40, 1: 21, 2: 21}
        for b in sorted({k // 16 for k in groups(bass_src)}):
            h0, h1 = chords.get(2 * b), chords.get(2 * b + 1, chords.get(2 * b))
            if h0 is None: continue
            r0 = place(h0[0], 36, 47); r1 = place((h1[0] + (7 if h1 == h0 else 0)) % 12, 36, 47)
            ev.add(2, b * BAR, 3 * STEP, r0, 70); ev.add(2, b * BAR + 8 * STEP, 3 * STEP, r1, 70)
        for step, g in sorted(chop.items()):
            root, minor = chord_at(chords, g[0][0])
            v = triad_voicing(root, minor, None, 52, 64)  # bouton d'accord : voicing fixe, sans conduite de voix
            for p in v:
                ev.add(1, g[0][0], 2 * STEP, p, 56)
    return ev, programs

def write_midi(ev, programs, tempo, path):
    mid = mido.MidiFile(type=0, ticks_per_beat=PPQ); tr = mido.MidiTrack(); mid.tracks.append(tr)
    tr.append(mido.MetaMessage("set_tempo", tempo=tempo, time=0))
    tr.append(mido.MetaMessage("time_signature", numerator=2, denominator=2, time=0))
    for ch, pr in programs.items():
        tr.append(mido.Message("program_change", channel=ch, program=pr, time=0))
    msgs = []
    for s, e, ch, p, v in ev:
        s = max(0, round(s)); e = max(s + 4, round(e))
        msgs.append((s, 1, mido.Message("note_on", channel=ch, note=p, velocity=v)))
        msgs.append((e, 0, mido.Message("note_off", channel=ch, note=p, velocity=0)))
    msgs.sort(key=lambda m: (m[0], m[1]))
    last = 0
    for t, _, m in msgs:
        tr.append(m.copy(time=t - last)); last = t
    mid.save(path)

def render(mid_path, wav_path):
    subprocess.run(["fluidsynth", "-ni", "-q", "-r", "44100", "-F", str(wav_path), SF2, str(mid_path)], check=True, capture_output=True)

VARIANTS = ["baseline", "strum", "passing", "strum_passing", "piano", "accordion"]

def main():
    stats = {}
    for seed in map(int, sys.argv[2:]):
        mid, notes, chords, tempo = load(seed)
        wavs = {}
        for variant in VARIANTS:
            rng = random.Random(1000 * seed + 7)  # même tirage de base pour toutes les variantes
            ev, programs = build(variant, seed, notes, chords, rng)
            m = OUT / f"reel_s{seed}_{variant}.mid"; write_midi(ev, programs, tempo, m)
            w = OUT / f"reel_s{seed}_{variant}.wav"; render(m, w); wavs[variant] = w
        data = {v: sf.read(w)[0] for v, w in wavs.items()}
        sr = 44100
        n = max(len(d) for d in data.values())
        rms = {v: float(np.sqrt(np.mean(np.square(d)))) for v, d in data.items()}
        peak = {v: float(np.max(np.abs(d))) for v, d in data.items()}
        # RMS commun : le plus fort possible sans que la crête de la variante la plus pointue dépasse 0,891 (−1 dBFS)
        target = min(0.891 * rms[v] / peak[v] for v in data)
        for v, d in data.items():
            g = target / rms[v]
            sf.write(OUT / f"reel_s{seed}_{v}.wav", np.clip(d * g, -1, 1), sr, subtype="PCM_16")
            stats[f"s{seed}/{v}"] = dict(rms_before_dbfs=20 * np.log10(rms[v]), peak_before_dbfs=20 * np.log10(peak[v]),
                gain_db=20 * np.log10(g), rms_after_dbfs=20 * np.log10(target), peak_after_dbfs=20 * np.log10(peak[v] * g),
                seconds=len(d) / sr)
    (OUT / "levels.json").write_text(json.dumps(stats, indent=1))

main()
