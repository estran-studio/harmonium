#!/usr/bin/env python3
"""Lecteur et assembleur SF2 pour l'essai de banques B1.

Adapté de sf2merge.py de l'annexe L1 : copies de presets et échantillons 16 bits,
fusion avec les zones converties de SFZ et remplacement des clés de kit GM.
"""
import struct
import math
import numpy as np


def _tc(sec):
    return max(-12000, min(8000, round(1200 * math.log2(max(float(sec), 0.001)))))


def chunks(data, pos, end):
    out = []
    while pos < end:
        cid = data[pos:pos + 4]
        size = struct.unpack_from('<I', data, pos + 4)[0]
        body = data[pos + 8:pos + 8 + size]
        out.append((cid, body[4:], body[:4]) if cid == b'LIST' else (cid, body, None))
        pos += 8 + size + (size & 1)
    return out


def rec(fmt, buf):
    s = struct.calcsize(fmt)
    return [struct.unpack_from(fmt, buf, i * s) for i in range(len(buf) // s)]


class Bank:
    def __init__(self, path):
        data = open(path, 'rb').read()
        assert data[:4] == b'RIFF' and data[8:12] == b'sfbk', path
        top = chunks(data, 12, len(data))
        sdta = next(c for c in top if c[2] == b'sdta')
        pdta = next(c for c in top if c[2] == b'pdta')
        self.smpl = next(c for c in chunks(sdta[1], 0, len(sdta[1])) if c[0] == b'smpl')[1]
        p = {c[0]: c[1] for c in chunks(pdta[1], 0, len(pdta[1]))}
        self.phdr = rec('<20sHHHIII', p[b'phdr'])
        self.pbag = rec('<HH', p[b'pbag'])
        self.pmod = rec('<HHhHH', p[b'pmod'])
        self.pgen = rec('<Hh', p[b'pgen'])
        self.inst = rec('<20sH', p[b'inst'])
        self.ibag = rec('<HH', p[b'ibag'])
        self.imod = rec('<HHhHH', p[b'imod'])
        self.igen = rec('<Hh', p[b'igen'])
        self.shdr = rec('<20sIIIIIBbHH', p[b'shdr'])
        self.path = path

    def preset(self, bank, prog):
        for i in range(len(self.phdr) - 1):
            if self.phdr[i][2] == bank and self.phdr[i][1] == prog:
                return i
        raise KeyError((self.path, bank, prog))

    def preset_zones(self, i):
        """[(gens, mods)] d'un preset."""
        zs = []
        for b in range(self.phdr[i][3], self.phdr[i + 1][3]):
            gens = self.pgen[self.pbag[b][0]:self.pbag[b + 1][0]]
            mods = self.pmod[self.pbag[b][1]:self.pbag[b + 1][1]]
            zs.append((list(gens), list(mods)))
        return zs

    def inst_zones(self, j):
        zs = []
        for b in range(self.inst[j][1], self.inst[j + 1][1]):
            gens = self.igen[self.ibag[b][0]:self.ibag[b + 1][0]]
            mods = self.imod[self.ibag[b][1]:self.ibag[b + 1][1]]
            zs.append((list(gens), list(mods)))
        return zs


class Out:
    def __init__(self):
        self.smpl = bytearray()
        self.shdr = []
        self.inst = []
        self.ibag = []
        self.igen = []
        self.imod = []
        self.presets = []  # (name, bank, prog, [(gens, mods)])
        self._samples = {}
        self._insts = {}
        self._new_samples = {}
        self._names = set()

    def _unique_name(self, name):
        """Nom d'échantillon unique dans la banque de sortie. oxisynth retrouve l'échantillon d'une zone PAR SON NOM (premier
        trouvé, `core/soundfont/instrument.rs`) : deux échantillons de même nom, venus de banques sources différentes, se
        confondent à la lecture (le second preset joue les échantillons du premier). Un suffixe `~n` les distingue."""
        base = bytes(name).split(b'\0')[0]
        out, i = base, 0
        while out in self._names:
            i += 1
            tag = b'~' + (b'%d' % i)
            out = base[:19 - len(tag)] + tag
        self._names.add(out)
        return out.ljust(20, b'\0')

    def sample(self, bank, k, gain_db=0.0):
        key = (id(bank), k, round(gain_db, 2))
        if key in self._samples:
            return self._samples[key]
        h = bank.shdr[k]
        name, st, en, sl, el, rate, pitch, corr, link, typ = h
        new_st = len(self.smpl) // 2
        chunk = bank.smpl[st * 2:min(en + 46, len(bank.smpl) // 2) * 2]  # échantillon + 46 points de garde d'origine
        chunk = chunk + b'\0' * (((en - st + 46) * 2) - len(chunk))
        if gain_db:
            a = np.frombuffer(chunk, dtype='<i2').astype(np.float32) * (10 ** (gain_db / 20))
            chunk = np.clip(np.round(a), -32768, 32767).astype('<i2').tobytes()
        self.smpl += chunk
        d = new_st - st
        idx = len(self.shdr)
        self._samples[key] = idx
        self.shdr.append([self._unique_name(name), st + d, en + d, sl + d, el + d, rate, pitch, corr, link, typ, bank, k, gain_db])
        return idx

    def new_sample(self, name, pcm_i16, rate, root, loop=None):
        """Échantillon mono issu d'un tableau int16 (loop = (début, fin) en points ou None)."""
        pcm = np.asarray(pcm_i16, dtype='<i2').copy()
        n = len(pcm)
        if loop:
            pcm = np.concatenate([pcm, np.zeros(8, dtype='<i2')])
            pcm[loop[1]:loop[1] + 8] = pcm[loop[0]:loop[0] + 8]
            body = pcm[:max(n, loop[1] + 8)]
            n = len(body)
            pcm = body
        st = len(self.smpl) // 2
        self.smpl += pcm.tobytes() + b'\0' * (46 * 2)
        ls, le = (st + loop[0], st + loop[1]) if loop else (st, st + n)
        idx = len(self.shdr)
        self.shdr.append([self._unique_name(name[:19].encode('latin1', 'replace')), st, st + n, ls, le, rate, root, 0, 0, 1, None, None, 0.0])
        return idx

    def instrument(self, bank, j, extend=False, gain_db=0.0, keymap=None, drop_keys=None):
        """keymap : {touche source: [touches de destination]} (kits) ; drop_keys : touches sources retirées."""
        key = (id(bank), j, extend, round(gain_db, 2), tuple(sorted((k, tuple(v)) for k, v in keymap.items())) if keymap else None, tuple(sorted(drop_keys)) if drop_keys else None)
        if key in self._insts:
            return self._insts[key]
        idx = len(self.inst)
        self._insts[key] = idx
        self.inst.append((bank.inst[j][0], len(self.ibag)))
        zones = bank.inst_zones(j)
        if extend:  # prolonge la zone la plus grave jusqu'à 0 et la plus aiguë jusqu'à 127 (le synthé ne joue rien hors zone)
            rngs = [(v & 0xFF, v >> 8) for gens, _ in zones for g, v in gens if g == 43]
            lo_min, hi_max = min(r[0] for r in rngs), max(r[1] for r in rngs)
        for gens, mods in zones:
            variants = [gens]
            if keymap is not None or drop_keys is not None:
                kr = [v for g, v in gens if g == 43]
                if any(g == 53 for g, _ in gens):
                    lo, hi = (kr[0] & 255, kr[0] >> 8) if kr else (0, 127)
                    if keymap is not None:
                        if lo != hi or lo not in keymap:
                            continue
                        variants = []
                        for dst in keymap[lo]:
                            g3 = [(g, dst | (dst << 8) if g == 43 else v) for g, v in gens if g != 58]
                            if not kr:
                                g3.insert(0, (43, dst | (dst << 8)))
                            g3.insert(1, (58, dst))
                            variants.append(g3)
                    elif drop_keys:
                        runs = []
                        start = None
                        for k in range(lo, hi + 2):
                            if k <= hi and k not in drop_keys:
                                if start is None:
                                    start = k
                            elif start is not None:
                                runs.append((start, k - 1))
                                start = None
                        variants = []
                        for left, right in runs:
                            g3 = [(g, left | (right << 8) if g == 43 else v) for g, v in gens]
                            if not kr:
                                g3.insert(0, (43, left | (right << 8)))
                            variants.append(g3)
            for gv in variants:
                g2 = []
                for g, v in gv:
                    if g == 53:
                        v = self.sample(bank, v, gain_db)
                    if g == 43 and extend:
                        lo, hi = v & 0xFF, v >> 8
                        v = (0 if lo == lo_min else lo) | ((127 if hi == hi_max else hi) << 8)
                    g2.append((g, v))
                self.ibag.append((len(self.igen), len(self.imod)))
                self.igen.extend(g2)
                for m in mods:
                    self.imod.append(m)
        return idx

    def add_preset(self, bank, i, new_bank, new_prog, extend=False, gain_db=0.0, keymap=None, drop_keys=None, name=None):
        zones = []
        for gens, mods in bank.preset_zones(i):
            g2 = []
            for g, v in gens:
                if g == 41:
                    v = self.instrument(bank, v, extend, gain_db, keymap, drop_keys)
                if g == 43 and extend:
                    v = 127 << 8  # plage de preset ouverte : les limites restent celles des zones d'instrument
                if g == 43 and keymap is not None:
                    v = 127 << 8
                g2.append((g, v))
            zones.append((g2, list(mods)))
        nm = name.encode('latin1', 'replace').ljust(20, b'\0')[:20] if name else bank.phdr[i][0]
        self.presets = [p for p in self.presets if (p[1], p[2]) != (new_bank, new_prog)]
        self.presets.append((nm, new_bank, new_prog, zones))

    def _append_zone(self, z, gain_db=0.0):
        key = (id(z.audio), gain_db, z.sample_name, z.root_key, z.loop_start, z.loop_end)
        sid = self._new_samples.get(key)
        if sid is None:
            a = np.asarray(z.audio, dtype=np.float32) * (10 ** (gain_db / 20))
            pcm = np.clip(np.round(a * 32767), -32768, 32767).astype('<i2')
            loop = (z.loop_start, z.loop_end) if z.loop else None
            sid = self.new_sample(z.sample_name, pcm, z.sample_rate, z.root_key, loop)
            self._new_samples[key] = sid
        gens = [(43, z.key_lo | (z.key_hi << 8)), (44, z.vel_lo | (z.vel_hi << 8)), (34, _tc(z.attack_s))]
        if z.decay_s is not None:
            gens.append((36, _tc(z.decay_s)))
        if z.sustain_cb is not None:
            gens.append((37, max(0, min(1440, int(z.sustain_cb)))))
        gens.append((38, _tc(z.release_s)))
        if z.attenuation_cb:
            gens.append((48, max(0, min(1440, int(z.attenuation_cb)))))
        if z.sample_mode:
            gens.append((54, z.sample_mode))
        elif z.loop:
            gens.append((54, 1))
        if z.fine_tune_cents:
            gens.append((52, int(max(-99, min(99, z.fine_tune_cents)))))
        if z.exclusive_class:
            gens.append((57, int(z.exclusive_class)))
        if getattr(z, 'pan', 0):
            gens.append((17, int(z.pan)))
        gens.append((58, z.root_key))
        gens.append((53, sid))
        self.ibag.append((len(self.igen), len(self.imod)))
        self.igen.extend(gens)

    def add_zones_preset(self, name, new_bank, new_prog, zones, gain_db=0.0):
        """Preset construit à partir de BankZone (sfzlib) : un instrument, une zone par BankZone."""
        inst_idx = len(self.inst)
        nm = name.encode('latin1', 'replace').ljust(20, b'\0')[:20]
        self.inst.append((nm, len(self.ibag)))
        for z in sorted(zones, key=lambda z: (z.key_lo, z.vel_lo)):
            self._append_zone(z, gain_db)
        self.presets = [p for p in self.presets if (p[1], p[2]) != (new_bank, new_prog)]
        self.presets.append((nm, new_bank, new_prog, [([(41, inst_idx)], [])]))
        return inst_idx

    def overlay_kit(self, base, base_idx, new_prog, zones, gain_db=0.0, name=None, base_gain_db=0.0, drop=()):
        """Kit 128:new_prog = kit de base hors touches couvertes + zones importées."""
        covered = set(drop)
        for z in zones:
            covered.update(range(z.key_lo, z.key_hi + 1))
        pzs = []
        for gens, mods in base.preset_zones(base_idx):
            g2 = []
            for g, v in gens:
                if g == 41:
                    v = self.instrument(base, v, False, base_gain_db, None, covered)
                g2.append((g, v))
            pzs.append((g2, list(mods)))
        inst_idx = len(self.inst)
        nm = name.encode('latin1', 'replace').ljust(20, b'\0')[:20] if name else base.phdr[base_idx][0]
        self.inst.append((nm, len(self.ibag)))
        for z in zones:
            self._append_zone(z, gain_db)
        pzs.append(([(41, inst_idx)], []))
        self.presets = [p for p in self.presets if (p[1], p[2]) != (128, new_prog)]
        self.presets.append((nm, 128, new_prog, pzs))


    def write(self, path):
        # résolution des sampleLink (même banque source)
        for h in self.shdr:
            if h[9] not in (1, 0x8001) and h[8] != 0:
                src, k = h[10], h[11]
                h[8] = self._samples.get((id(src), h[8], round(h[12], 2)), 0)
            else:
                h[8] = 0
        presets = sorted(self.presets, key=lambda p: (p[1], p[2]))
        phdr, pbag, pgen, pmod = b'', b'', b'', b''
        nb = ng = nm = 0
        pb = []
        for name, bank, prog, zones in presets:
            pb.append(struct.pack('<20sHHHIII', name, prog, bank, nb, 0, 0, 0))
            for gens, mods in zones:
                pbag += struct.pack('<HH', ng, nm)
                nb += 1
                for g, v in gens:
                    pgen += struct.pack('<Hh', g, v)
                    ng += 1
                for m in mods:
                    pmod += struct.pack('<HHhHH', *m)
                    nm += 1
        pb.append(struct.pack('<20sHHHIII', b'EOP'.ljust(20, b'\0'), 0, 0, nb, 0, 0, 0))
        pbag += struct.pack('<HH', ng, nm)
        pgen += struct.pack('<Hh', 0, 0)
        pmod += struct.pack('<HHhHH', 0, 0, 0, 0, 0)
        phdr = b''.join(pb)
        inst = b''.join(struct.pack('<20sH', n, b) for n, b in self.inst) + struct.pack('<20sH', b'EOI'.ljust(20, b'\0'), len(self.ibag))
        ibag = b''.join(struct.pack('<HH', a, b) for a, b in self.ibag) + struct.pack('<HH', len(self.igen), len(self.imod))
        igen = b''.join(struct.pack('<Hh', g, v) for g, v in self.igen) + struct.pack('<Hh', 0, 0)
        imod = b''.join(struct.pack('<HHhHH', *m) for m in self.imod) + struct.pack('<HHhHH', 0, 0, 0, 0, 0)
        shdr = b''.join(struct.pack('<20sIIIIIBbHH', *h[:10]) for h in self.shdr) + struct.pack('<20sIIIIIBbHH', b'EOS'.ljust(20, b'\0'), 0, 0, 0, 0, 0, 0, 0, 0, 0)

        def ck(cid, body):
            return cid + struct.pack('<I', len(body)) + body + (b'\0' if len(body) & 1 else b'')

        def lst(kind, parts):
            body = kind + b''.join(parts)
            return ck(b'LIST', body)

        info = lst(b'INFO', [ck(b'ifil', struct.pack('<HH', 2, 1)), ck(b'isng', b'EMU8000\0'), ck(b'INAM', b'harmonium L1 merge\0\0')])
        sdta = lst(b'sdta', [ck(b'smpl', bytes(self.smpl))])
        pdta = lst(b'pdta', [ck(b'phdr', phdr), ck(b'pbag', pbag), ck(b'pmod', pmod), ck(b'pgen', pgen), ck(b'inst', inst), ck(b'ibag', ibag), ck(b'imod', imod), ck(b'igen', igen), ck(b'shdr', shdr)])
        body = b'sfbk' + info + sdta + pdta
        open(path, 'wb').write(b'RIFF' + struct.pack('<I', len(body)) + body)


