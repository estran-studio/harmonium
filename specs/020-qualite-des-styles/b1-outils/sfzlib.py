#!/usr/bin/env python3
"""Lecteur SFZ minimal + convertisseur de régions en zones SF2 (outil jetable B1, hors dépôt).

Prend en charge : <control> default_path, #define, #include, <global>/<master>/<group>/<region>, héritage des opcodes,
key/lokey/hikey/pitch_keycenter, lovel/hivel, sample, offset/end, loop_mode/loop_start/loop_end, volume, tune,
ampeg_*, trigger (release ignorés), lorand (premier tour de round-robin), seq_position (1), sw_* et locc/hicc (régions
retenues seulement si une valeur par défaut les rend actives).
"""
import os
import re
import numpy as np
import soundfile as sf
from scipy.signal import resample_poly
from math import gcd

NOTE = {'c': 0, 'd': 2, 'e': 4, 'f': 5, 'g': 7, 'a': 9, 'b': 11}


def note_num(v):
    v = str(v).strip().lower()
    if re.fullmatch(r'-?\d+', v):
        return int(v)
    m = re.fullmatch(r'([a-g])(#|b)?(-?\d+)', v)
    if not m:
        raise ValueError(v)
    n = NOTE[m.group(1)] + (1 if m.group(2) == '#' else -1 if m.group(2) == 'b' else 0)
    return n + (int(m.group(3)) + 1) * 12


def strip_comments(t):
    t = re.sub(r'/\*.*?\*/', '', t, flags=re.S)
    return re.sub(r'//[^\n]*', '', t)


def load_text(path, defines, seen=()):
    t = strip_comments(open(path, encoding='utf8', errors='replace').read())
    out = []
    for line in t.split('\n'):
        s = line.strip()
        m = re.match(r'#define\s+(\$\w+)\s+(.*)', s)
        if m:
            defines[m.group(1)] = m.group(2).strip()
            continue
        m = re.match(r'#include\s+"([^"]+)"', s)
        if m:
            p = os.path.normpath(os.path.join(os.path.dirname(path), m.group(1).replace('\\', '/')))
            if os.path.exists(p) and p not in seen:
                out.append(load_text(p, defines, seen + (path,)))
            continue
        out.append(line)
    text = '\n'.join(out)
    for k in sorted(defines, key=len, reverse=True):
        text = text.replace(k, defines[k])
    return text


def parse_opcodes(s):
    ops = {}
    for m in re.finditer(r'([A-Za-z_][\w$]*)=(.*?)(?=\s+[A-Za-z_][\w$]*=|\s*$)', s, flags=re.S):
        ops[m.group(1)] = m.group(2).strip()
    return ops


def parse_sfz(path):
    """Renvoie la liste des régions (dict d'opcodes complets) et le default_path."""
    defines = {}
    text = load_text(path, defines)
    toks = re.split(r'<(control|global|master|group|region|curve|effect|midi)>', text)
    ctl, glob, mast, grp = {}, {}, {}, {}
    regions = []
    i = 1
    while i < len(toks):
        kind, body = toks[i], toks[i + 1]
        ops = parse_opcodes(body)
        if kind == 'control':
            ctl.update(ops)
        elif kind == 'global':
            glob, mast, grp = dict(ops), {}, {}
        elif kind == 'master':
            mast, grp = dict(ops), {}
        elif kind == 'group':
            grp = dict(ops)
        elif kind == 'region':
            r = {}
            for d in (glob, mast, grp, ops):
                r.update(d)
            regions.append(r)
        i += 2
    return regions, ctl.get('default_path', ''), ctl


def read_audio(path, rate_out=44100):
    data, sr = sf.read(path, dtype='float32', always_2d=True)
    mono = data.mean(axis=1)
    if sr != rate_out:
        g = gcd(sr, rate_out)
        mono = resample_poly(mono, rate_out // g, sr // g).astype('float32')
    return mono


# ── SFZ → zones SF2 ──────────────────────────────────────────────────────────
from dataclasses import dataclass

@dataclass
class BankZone:
    sample_name: str
    audio: np.ndarray
    root_key: int
    key_lo: int
    key_hi: int
    vel_lo: int = 0
    vel_hi: int = 127
    loop: bool = False
    loop_start: int = 0
    loop_end: int = 0
    sample_mode: int | None = None
    attack_s: float = 0.002
    decay_s: float | None = None
    sustain_cb: int | None = None
    release_s: float = 0.25
    attenuation_cb: int = 0
    exclusive_class: int = 0
    sample_rate: int = 44100
    fine_tune_cents: int = 0
    pan: int = 0


def sfz_to_zones(sfz_path, *, max_seconds=None, ctl_cc=None, pick_rr=True, vel_filter=None, key_shift=0,
                 fade_s=0.05, keep=None, collect=None, mix=False, attack_s=None, release_s=None, gain_db=0.0, force_no_loop=False,
                 sample_dir=None):
    """Zones SF2 d'un fichier SFZ. `keep(region_dict)` : filtre supplémentaire. Retourne (zones, rejets)."""
    regions, dpath, ctl = parse_sfz(sfz_path)
    cc = {}
    for k, v in ctl.items():
        m = re.fullmatch(r'set_(hd)?cc(\d+)', k)
        if m:
            cc[int(m.group(2))] = float(v) * (127 if m.group(1) else 1)
    cc.update(ctl_cc or {})
    ctl_cc = cc
    base = os.path.dirname(sfz_path)
    if sample_dir:
        base = sample_dir
    # valeurs par défaut des CC (fichier <control> non conservé : lire à part)
    zones, dropped = [], []
    cache = {}
    for r in regions:
        why = None
        if 'sample' not in r or r['sample'].startswith('*'):
            continue
        if r.get('trigger', 'attack') not in ('attack', 'first'):
            why = 'trigger'
        if pick_rr and 'lorand' in r and float(r['lorand']) > 0:
            why = 'rr'
        if pick_rr and 'seq_position' in r and int(float(r['seq_position'])) != 1:
            why = 'seq'
        if 'sw_last' in r or 'sw_previous' in r or 'sw_down' in r or 'sw_up' in r:
            sd = r.get('sw_default')
            if 'sw_last' in r and (sd is None or note_num(sd) != note_num(r['sw_last'])):
                why = 'sw'
            if 'sw_previous' in r or 'sw_down' in r or 'sw_up' in r:
                why = 'sw'
        for k in list(r):
            m = re.fullmatch(r'(lo|hi)(?:hd)?cc(\d+)', k)
            if m:
                cc = int(m.group(2))
                val = (ctl_cc or {}).get(cc, 0)
                lo = float(r.get(f'locc{cc}', 0))
                hi = float(r.get(f'hicc{cc}', 127))
                if not (lo <= val <= hi):
                    why = f'cc{cc}'
        if keep and not keep(r):
            why = 'keep'
        if why:
            dropped.append((r.get('sample'), why))
            continue
        # clés
        try:
            if 'key' in r:
                lo = hi = root = note_num(r['key'])
            else:
                lo = note_num(r.get('lokey', 0))
                hi = note_num(r.get('hikey', 127))
                root = note_num(r['pitch_keycenter']) if 'pitch_keycenter' in r else lo
            if 'pitch_keycenter' in r:
                root = note_num(r['pitch_keycenter'])
        except Exception:
            dropped.append((r.get('sample'), 'key'))
            continue
        vlo, vhi = int(float(r.get('lovel', 0))), int(float(r.get('hivel', 127)))
        if vel_filter and not vel_filter(vlo, vhi):
            dropped.append((r.get('sample'), 'vel'))
            continue
        spath = os.path.normpath(os.path.join(base, dpath.replace('\\', '/'), r['sample'].replace('\\', '/')))
        if collect is not None:
            collect.append(spath)
            continue
        if not os.path.exists(spath):
            dropped.append((r['sample'], 'missing'))
            continue
        off = int(float(r.get('offset', 0)))
        end = int(float(r['end'])) if 'end' in r else None
        ck = (spath, off, end, max_seconds)
        if ck not in cache:
            try:
                a = read_audio(spath)
            except Exception:
                dropped.append((r['sample'], 'unreadable'))
                continue
            # offset/end sont en échantillons du fichier d'origine
            info = sf.info(spath)
            ratio = 44100 / info.samplerate
            o = int(off * ratio)
            e = int(end * ratio) if end else len(a)
            a = a[o:e]
            if max_seconds and len(a) > max_seconds * 44100:
                n = int(max_seconds * 44100)
                a = a[:n].copy()
                f = int(fade_s * 44100)
                a[-f:] *= np.linspace(1, 0, f, dtype='float32')
            cache[ck] = (a, ratio, o)
        a, ratio, o = cache[ck]
        z = BankZone(sample_name=os.path.splitext(os.path.basename(r['sample']))[0][:19], audio=a, root_key=root,
                     key_lo=max(0, lo + key_shift), key_hi=min(127, hi + key_shift), vel_lo=vlo, vel_hi=vhi)
        z.root_key = max(0, min(127, root + key_shift))
        if 'tune' in r:
            z.fine_tune_cents = int(round(float(r['tune'])))
        db = float(r.get('volume', 0)) + gain_db
        z.attenuation_cb = int(max(0, -db * 10))
        lm = r.get('loop_mode', 'no_loop' if 'loop_start' not in r else 'loop_continuous')
        if lm in ('loop_continuous', 'loop_sustain') and not force_no_loop and 'loop_start' in r and 'loop_end' in r:
            ls = int((int(float(r['loop_start'])) - off) * ratio)
            le = int((int(float(r['loop_end'])) - off) * ratio)
            if 8 <= ls < le <= len(a) - 8:
                z.loop, z.loop_start, z.loop_end = True, ls, le
                z.sample_mode = 1 if lm == 'loop_continuous' else 3
        z.attack_s = attack_s if attack_s is not None else float(r.get('ampeg_attack', 0.002) or 0.002)
        z.release_s = release_s if release_s is not None else float(r.get('ampeg_release', 0.25) or 0.25)
        if 'ampeg_sustain' in r and float(r['ampeg_sustain']) < 100 and 'ampeg_decay' in r:
            s = max(float(r['ampeg_sustain']), 0.01)
            z.decay_s = float(r['ampeg_decay'])
            z.sustain_cb = int(-200 * np.log10(s / 100))  # 10 cB = 1 dB
        zones.append(z)
    if collect is not None:
        return zones, dropped
    out = []
    for z in zones:
        same = [p for p in out if (p.key_lo, p.key_hi, p.vel_lo, p.vel_hi) == (z.key_lo, z.key_hi, z.vel_lo, z.vel_hi)]
        if mix and same:
            p = same[0]
            n = max(len(p.audio), len(z.audio))
            m = np.zeros(n, dtype=np.float32)
            m[:len(p.audio)] += p.audio
            m[:len(z.audio)] += z.audio * (10 ** (-z.attenuation_cb / 200) / max(10 ** (-p.attenuation_cb / 200), 1e-6))
            p.audio = m
            continue
        if any(max(z.key_lo, p.key_lo) <= min(z.key_hi, p.key_hi) and max(z.vel_lo, p.vel_lo) <= min(z.vel_hi, p.vel_hi)
               for p in out):
            dropped.append((z.sample_name, 'overlap'))
            continue
        out.append(z)
    return out, dropped
