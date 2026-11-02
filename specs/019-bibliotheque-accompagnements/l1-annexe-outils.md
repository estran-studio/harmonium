# L1 — annexe : outils d'écoute et de mesure (hors dépôt)

Scripts Python **jetables** qui ont produit les chiffres de `l1-banques-soundfont.md`. Ils ne sont pas dans le dépôt (aucun code
dans le diff de L1) : recopier chaque bloc dans `/tmp/harmonium-019/l1/work/<nom>` (même nom). Dépendances : Python 3.14 standard,
`ffmpeg`, `fluidsynth`, `oggenc` ; `latinkeys.py` utilise le venv du labo (`pretty_midi`). Le binaire est `harmonium-cli` du dépôt
moteur (`~/Project/bascanada/harmonium`, `cargo build --release -p harmonium-cli`), lancé depuis ce dossier pour `assets/`.

Le convertisseur SF3 est `harmonium_studio/scripts/sf2_to_sf3.py` (dépôt, inchangé).

## `sf2info.py`

Lecture des en-têtes SF2/SF3 : presets, nombre d'échantillons, plages de touches, couverture d'un kit.

```python
#!/usr/bin/env python3
"""Liste les presets d'un SF2/SF3 (banque, programme, nom, nb d'échantillons, plage de touches) — lecture seule, hors dépôt."""
import json
import struct
import sys


def chunks(data, pos, end):
    out = []
    while pos < end:
        cid = data[pos:pos + 4]
        size = struct.unpack_from('<I', data, pos + 4)[0]
        body = data[pos + 8:pos + 8 + size]
        out.append((cid, body[4:], body[:4]) if cid == b'LIST' else (cid, body, None))
        pos += 8 + size + (size & 1)
    return out


def load(path):
    data = open(path, 'rb').read()
    assert data[:4] == b'RIFF' and data[8:12] == b'sfbk', path
    top = chunks(data, 12, len(data))
    info = next(c for c in top if c[2] == b'INFO')
    pdta = next(c for c in top if c[2] == b'pdta')
    ver = None
    for cid, body, _ in chunks(info[1], 0, len(info[1])):
        if cid == b'ifil':
            ver = struct.unpack('<HH', body[:4])
    p = {cid: body for cid, body, _ in chunks(pdta[1], 0, len(pdta[1]))}
    return ver, p


def coverage(path, bank, program):
    """Touches (0..127) qui ont au moins une zone d'échantillon dans le preset (bank, program)."""
    ver, p = load(path)
    phdr = [struct.unpack_from('<20sHHHIII', p[b'phdr'], i * 38) for i in range(len(p[b'phdr']) // 38)]
    pbag = [struct.unpack_from('<HH', p[b'pbag'], i * 4) for i in range(len(p[b'pbag']) // 4)]
    pgen = [struct.unpack_from('<Hh', p[b'pgen'], i * 4) for i in range(len(p[b'pgen']) // 4)]
    inst = [struct.unpack_from('<20sH', p[b'inst'], i * 22) for i in range(len(p[b'inst']) // 22)]
    ibag = [struct.unpack_from('<HH', p[b'ibag'], i * 4) for i in range(len(p[b'ibag']) // 4)]
    igen = [struct.unpack_from('<Hh', p[b'igen'], i * 4) for i in range(len(p[b'igen']) // 4)]
    keys = set()
    for i in range(len(phdr) - 1):
        if phdr[i][2] != bank or phdr[i][1] != program:
            continue
        for b in range(phdr[i][3], phdr[i + 1][3]):
            pk = (0, 127)
            ii = None
            for g in range(pbag[b][0], pbag[b + 1][0]):
                if pgen[g][0] == 43:
                    pk = (pgen[g][1] & 0xFF, pgen[g][1] >> 8)
                if pgen[g][0] == 41:
                    ii = pgen[g][1]
            if ii is None:
                continue
            for bb in range(inst[ii][1], inst[ii + 1][1]):
                ik = (0, 127)
                has = False
                for g in range(ibag[bb][0], ibag[bb + 1][0]):
                    if igen[g][0] == 43:
                        ik = (igen[g][1] & 0xFF, igen[g][1] >> 8)
                    if igen[g][0] == 53:
                        has = True
                if has:
                    lo, hi = max(pk[0], ik[0]), min(pk[1], ik[1])
                    keys.update(range(lo, hi + 1))
    return keys


def parse(path):
    ver, p = load(path)
    phdr = [struct.unpack_from('<20sHHHIII', p[b'phdr'], i * 38) for i in range(len(p[b'phdr']) // 38)]
    pbag = [struct.unpack_from('<HH', p[b'pbag'], i * 4) for i in range(len(p[b'pbag']) // 4)]
    pgen = [struct.unpack_from('<Hh', p[b'pgen'], i * 4) for i in range(len(p[b'pgen']) // 4)]
    inst = [struct.unpack_from('<20sH', p[b'inst'], i * 22) for i in range(len(p[b'inst']) // 22)]
    ibag = [struct.unpack_from('<HH', p[b'ibag'], i * 4) for i in range(len(p[b'ibag']) // 4)]
    igen = [struct.unpack_from('<Hh', p[b'igen'], i * 4) for i in range(len(p[b'igen']) // 4)]
    shdr = [struct.unpack_from('<20sIIIIIBbHH', p[b'shdr'], i * 46) for i in range(len(p[b'shdr']) // 46)]
    presets = []
    for i in range(len(phdr) - 1):
        name, prog, bank, bidx, _, _, _ = phdr[i]
        nxt = phdr[i + 1][3]
        insts = []
        for b in range(bidx, nxt):
            for g in range(pbag[b][0], pbag[b + 1][0]):
                if pgen[g][0] == 41:
                    insts.append(pgen[g][1])
        samples = set()
        lo, hi = 127, 0
        for ii in insts:
            for b in range(inst[ii][1], inst[ii + 1][1]):
                sid = None
                kr = None
                for g in range(ibag[b][0], ibag[b + 1][0]):
                    if igen[g][0] == 53:
                        sid = igen[g][1]
                    if igen[g][0] == 43:
                        kr = igen[g][1]
                if sid is not None:
                    samples.add(sid)
                    if kr is not None:
                        lo = min(lo, kr & 0xFF)
                        hi = max(hi, kr >> 8)
                    else:
                        lo, hi = 0, 127
        pcm = sum((shdr[s][2] - shdr[s][1]) for s in samples)
        presets.append({'bank': bank, 'program': prog, 'name': name.split(b'\0')[0].decode('latin1').strip(),
                        'samples': len(samples), 'keys': [lo, hi] if hi >= lo else None, 'sample_points': pcm})
    vorbis = sum(1 for s in shdr[:-1] if s[9] & 0x10)
    return {'file': path, 'ifil': ver, 'n_presets': len(presets), 'n_samples': len(shdr) - 1,
            'n_vorbis_samples': vorbis, 'presets': presets}


if __name__ == '__main__':
    r = parse(sys.argv[1])
    if len(sys.argv) > 2 and sys.argv[2] == '--json':
        print(json.dumps(r))
    else:
        print(f"{r['file']} ifil={r['ifil']} presets={r['n_presets']} samples={r['n_samples']} vorbis={r['n_vorbis_samples']}")
        for pr in r['presets']:
            print(f"{pr['bank']:3d}:{pr['program']:3d} {pr['name']:<22} samples={pr['samples']:4d} keys={pr['keys']}")
```

## `sf2merge.py`

Fusion de presets de plusieurs SF2 (PCM 16 bits) dans une SF2 ; `--extend-keys` ouvre les plages de touches.

```python
#!/usr/bin/env python3
"""Fusionne des presets de plusieurs SF2 (PCM 16 bits) dans une seule SF2 — outil d'écoute L1, hors dépôt.

Usage : sf2merge.py sortie.sf2 base.sf2 [--drop B:P,B:P,…] [--extend-keys] [--add source.sf2 'bs:ps=>bd:pd' …]…
  - `base.sf2` : tous ses presets sont conservés, sauf ceux de `--drop` (banque:programme).
  - `--add source.sf2 'bs:ps=>bd:pd'` : copie le preset (banque bs, programme ps) de `source.sf2`
    à (banque bd, programme pd) ; plusieurs correspondances par source, séparées par des virgules.
    Un preset ajouté remplace celui de même (banque, programme) de la base.
`--extend-keys` : prolonge la zone la plus grave de chaque preset ajouté (hors kits) jusqu'à la touche 0 et la plus aiguë jusqu'à 127.
Seuls les échantillons, instruments et presets sont copiés avec leurs modulateurs ; le preset
reste lu par oxisynth/fluidsynth comme dans sa banque d'origine (mêmes générateurs).
Les échantillons utilisés seulement par des presets retirés sont élagués (taille de sortie réduite).
"""
import struct
import sys


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

    def sample(self, bank, k):
        key = (id(bank), k)
        if key in self._samples:
            return self._samples[key]
        h = bank.shdr[k]
        name, st, en, sl, el, rate, pitch, corr, link, typ = h
        new_st = len(self.smpl) // 2
        chunk = bank.smpl[st * 2:min(en + 46, len(bank.smpl) // 2) * 2]  # échantillon + 46 points de garde d'origine
        self.smpl += chunk + b'\0' * (((en - st + 46) * 2) - len(chunk))
        d = new_st - st
        idx = len(self.shdr)
        self._samples[key] = idx
        # sampleLink : recopié après coup (liens stéréo) ; ici index provisoire à résoudre
        self.shdr.append([name, st + d, en + d, sl + d, el + d, rate, pitch, corr, link, typ, bank, k])
        return idx

    def instrument(self, bank, j, extend=False):
        key = (id(bank), j, extend)
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
            g2 = []
            for g, v in gens:
                if g == 53:
                    v = self.sample(bank, v)
                if g == 43 and extend:
                    lo, hi = v & 0xFF, v >> 8
                    v = (0 if lo == lo_min else lo) | ((127 if hi == hi_max else hi) << 8)
                g2.append((g, v))
            self.ibag.append((len(self.igen), len(self.imod)))
            self.igen.extend(g2)
            for m in mods:
                self.imod.append(m)
        return idx

    def add_preset(self, bank, i, new_bank, new_prog, extend=False):
        zones = []
        for gens, mods in bank.preset_zones(i):
            g2 = []
            for g, v in gens:
                if g == 41:
                    v = self.instrument(bank, v, extend)
                if g == 43 and extend:
                    v = 127 << 8  # plage de preset ouverte : les limites restent celles des zones d'instrument
                g2.append((g, v))
            zones.append((g2, list(mods)))
        name = bank.phdr[i][0]
        self.presets = [p for p in self.presets if (p[1], p[2]) != (new_bank, new_prog)]
        self.presets.append((name, new_bank, new_prog, zones))

    def write(self, path):
        # résolution des sampleLink (même banque source)
        for h in self.shdr:
            if h[9] not in (1, 0x8001) and h[8] != 0:
                src, k = h[10], h[11]
                h[8] = self._samples.get((id(src), h[8]), 0)
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


def main():
    a = sys.argv[1:]
    out_path, base_path = a[0], a[1]
    rest = a[2:]
    drops = set()
    adds = []
    extend = False
    i = 0
    while i < len(rest):
        if rest[i] == '--drop':
            for t in rest[i + 1].split(','):
                b, p = t.split(':')
                drops.add((int(b), int(p)))
            i += 2
        elif rest[i] == '--extend-keys':
            extend = True
            i += 1
        elif rest[i] == '--add':
            adds.append((rest[i + 1], rest[i + 2]))
            i += 3
        else:
            raise SystemExit(__doc__)
    out = Out()
    base = Bank(base_path)
    for k in range(len(base.phdr) - 1):
        if (base.phdr[k][2], base.phdr[k][1]) in drops:
            continue
        out.add_preset(base, k, base.phdr[k][2], base.phdr[k][1])
    for src_path, maps in adds:
        src = Bank(src_path)
        for m in maps.split(','):
            s, d = m.split('=>')
            sb, sp = map(int, s.split(':'))
            db, dp = map(int, d.split(':'))
            out.add_preset(src, src.preset(sb, sp), db, dp, extend and db != 128)
    out.write(out_path)
    print(out_path, len(out.presets), 'presets', len(out.smpl) // 2, 'points')


if __name__ == '__main__':
    main()
```

## `make_profiles.py`

Profils jetables d'écoute (batterie dense, kit balais, percussions latines), copiés de `profiles/wjazzd_swing`.

```python
#!/usr/bin/env python3
"""Génère les profils temporaires d'écoute L1 (hors dépôt) à partir de profiles/wjazzd_swing.

Usage : make_profiles.py <dossier profiles du dépôt> <dossier de sortie>
  - swing_dense  : wjazzd_swing, kit jazz (128:32), densité de batterie 2 constante (grosse caisse, caisse claire, toms, relances).
  - swing_balais : comme swing_dense, kit balais (banque 128, programme 40).
  - latin_gm     : wjazzd_swing, guitare nylon (24), basse acoustique (32), 120 BPM, droit ;
                   percussions latines aux touches du kit GM (congas 62-64, bongos 60-61,
                   claves 75, cloche 56, maracas 70, timbales 65-66, guiro 73, grosse caisse 36).
  - latin_fp     : idem, kit « World Percussion » de FreePats (banque 128, programme 64 après fusion) ;
                   touches propres à cette banque (bongos 51-53, congas 62-66, claves 60,
                   maracas 67, tambourin 57, cajón 48-50).
"""
import re
import shutil
import sys
from pathlib import Path

src_root, out_root = Path(sys.argv[1]), Path(sys.argv[2])
src = src_root / "wjazzd_swing"


def clone(name):
    dst = out_root / name
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    prof = (dst / "profile.toml").read_text(encoding="utf-8")
    prof = re.sub(r'(?m)^id = "wjazzd_swing"', f'id = "{name}"', prof, count=1)
    return dst, prof


def write(dst, prof):
    (dst / "profile.toml").write_text(prof, encoding="utf-8")


def dense(dst):
    """Densité de batterie 2 constante (kick, caisse claire, toms, relances) : percussion_intensity = +1."""
    p = dst / "dynamics.toml"
    t = p.read_text(encoding="utf-8")
    t = re.sub(r"(?m)^percussion_intensity = .*$", "percussion_intensity = [[0.0, 1.0], [1.0, 1.0]]", t, count=1)
    p.write_text(t, encoding="utf-8")


def set_instruments(prof, **kw):
    for k, v in kw.items():
        prof = re.sub(rf"(?m)^{k} = .*$", f"{k} = {v}", prof, count=1)
    return prof


# ── swing_dense : kit jazz (32), densité 2 constante ; swing_balais : kit balais (40), densité 2 ──
dst, prof = clone("swing_dense")
write(dst, prof)
dense(dst)
dst, prof = clone("swing_balais")
write(dst, set_instruments(prof, percussion_program=40))
dense(dst)

# ── latin ──
HEADER = """# Profil d'écoute L1 (jetable) : percussions latines pour comparer des banques. Pas un style du moteur.
low_note = {low}
high_note = {high}
low_velocity = 64
high_velocity = 64
percussion_gain = 1.0
velocity_humanize = 3
"""


def voices(vs):
    out = []
    for name, note, vel, track in vs:
        out.append(f'[[voices]]\nname = "{name}"\nnote = {note}\nvelocity = {vel}\ntrack = "{track}"\n')
    return "\n".join(out)


def pat(density, weight, hits):
    h = ",\n".join(f'  {{ step = {s}, voice = "{v}", delta_velocity = {d} }}' for s, v, d in hits)
    return f"[[patterns]]\ndensity = {density}\nweight = {weight}\nhits = [\n{h},\n]\n"


def latin(name, voice_defs, rows):
    """rows : dict densité -> liste de (pas, voix, delta)."""
    dst, prof = clone(name)
    prof = set_instruments(prof, chords_program=24, bass_program=32, percussion_bank=128,
                           percussion_program=0 if name == "latin_gm" else 64)
    prof = re.sub(r"(?m)^default_swing_ratio = .*$", "default_swing_ratio = 0.5", prof, count=1)
    prof = re.sub(r"(?m)^default_bpm = .*$", "default_bpm = 120.0", prof, count=1)
    prof = re.sub(r"(?m)^min_bpm = .*$", "min_bpm = 90.0", prof, count=1)
    write(dst, prof)
    dense(dst)
    text = HEADER.format(low=voice_defs[0][1], high=voice_defs[1][1]) + "\n" + voices(voice_defs) + "\n"
    for density, hits in rows.items():
        text += pat(density, 100, hits) + "\n"
    (dst / "percussion.toml").write_text(text, encoding="utf-8")


def tumbao(v):
    """Motif latin sur 16 pas ; v : noms de voix de la banque."""
    d0 = [(0, v["clave"], 0), (3, v["clave"], -4), (6, v["clave"], 0), (10, v["clave"], -4), (12, v["clave"], 0)]
    d0 += [(s, v["shaker"], -14 + (6 if s % 4 == 0 else 0)) for s in range(0, 16, 2)]
    d1 = list(d0) + [(s, v["bell"], -2) for s in (0, 4, 8, 12)]
    d1 += [(4, v["conga_mute"], 0), (6, v["conga_open"], 4), (7, v["conga_open"], -6), (12, v["conga_mute"], 0), (14, v["conga_open"], 4), (15, v["conga_open"], -6)]
    d2 = list(d1) + [(0, v["conga_low"], 6), (8, v["conga_low"], 6)]
    d2 += [(s, v["bongo_hi"], -4) for s in (2, 10)] + [(s, v["bongo_lo"], -2) for s in (0, 6, 8, 14)]
    return {0: d0, 1: d1, 2: d2}


gm_voices = [
    ("kick", 36, 34, "PercussionLow"),
    ("conga_low", 64, 50, "PercussionLow"),
    ("clave", 75, 55, "PercussionHigh"),
    ("bell", 56, 48, "PercussionHigh"),
    ("shaker", 70, 40, "PercussionHigh"),
    ("conga_mute", 62, 50, "PercussionHigh"),
    ("conga_open", 63, 52, "PercussionHigh"),
    ("bongo_hi", 60, 50, "PercussionHigh"),
    ("bongo_lo", 61, 50, "PercussionHigh"),
    ("timbale", 65, 50, "PercussionHigh"),
    ("guiro", 73, 40, "PercussionHigh"),
]
latin("latin_gm", gm_voices, tumbao({k: k for k in ("clave", "shaker", "bell", "conga_mute", "conga_open", "conga_low", "bongo_hi", "bongo_lo")}))

fp_voices = [
    ("cajon_low", 48, 50, "PercussionLow"),
    ("conga_low", 64, 50, "PercussionLow"),
    ("clave", 60, 55, "PercussionHigh"),
    ("bell", 57, 40, "PercussionHigh"),
    ("shaker", 67, 40, "PercussionHigh"),
    ("conga_mute", 65, 50, "PercussionHigh"),
    ("conga_open", 63, 52, "PercussionHigh"),
    ("bongo_hi", 52, 50, "PercussionHigh"),
    ("bongo_lo", 53, 50, "PercussionHigh"),
]
latin("latin_fp", fp_voices, tumbao({k: k for k in ("clave", "shaker", "bell", "conga_mute", "conga_open", "conga_low", "bongo_hi", "bongo_lo")}))
print("profils écrits dans", out_root)
```

## `render_l1.py`

Rendus par banque et par style (mix + solo par piste) et mesure LUFS (ffmpeg ebur128).

```python
#!/usr/bin/env python3
"""Rendus d'écoute L1 : (banque × style) en mix complet + solo par piste, LUFS mesurés par ffmpeg (ebur128).

Lancé depuis ~/Project/bascanada/harmonium (pour assets/ et le binaire harmonium-cli).
Sortie : /tmp/harmonium-019/l1/render/<style>/<banque>_{mix,solo-<Piste>}.wav + results.json
"""
import concurrent.futures
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ENGINE = Path.home() / "Project/bascanada/harmonium/target/release/harmonium-cli"
L1 = Path("/tmp/harmonium-019/l1")
BANKS = json.loads((L1 / "work/banks.json").read_text())  # alias -> chemin
STYLES = {
    # nom : (arguments de style, mesures, pistes solo)
    "wjazzd_swing": (["--style", "wjazzd_swing"], 44, ["Chords", "Bass", "PercussionLow", "PercussionHigh"]),
    "swing_dense": (["--profile", str(L1 / "profiles/swing_dense")], 44, ["Chords", "Bass", "PercussionLow", "PercussionHigh"]),
    "swing_balais": (["--profile", str(L1 / "profiles/swing_balais")], 44, ["Chords", "Bass", "PercussionLow", "PercussionHigh"]),
    "isaac_burning": (["--style", "isaac_burning"], 32, ["Melody", "Chords", "Bass", "PercussionLow", "PercussionHigh"]),
    "latin_gm": (["--profile", str(L1 / "profiles/latin_gm")], 32, ["Chords", "Bass", "PercussionLow", "PercussionHigh"]),
    "latin_fp": (["--profile", str(L1 / "profiles/latin_fp")], 32, ["Chords", "Bass", "PercussionLow", "PercussionHigh"]),
}
SEED = "7"


def lufs(path):
    p = subprocess.run(["ffmpeg", "-nostats", "-i", str(path), "-af", "ebur128=peak=true", "-f", "null", "-"],
                       capture_output=True, text=True)
    tail = p.stderr.split("Summary:")[-1]
    m = re.search(r"I:\s+(-?[\d.]+|-inf) LUFS", tail)
    pk = re.search(r"Peak:\s+(-?[\d.]+|-inf) dBFS", tail)
    return (float(m.group(1)) if m and m.group(1) != "-inf" else None,
            float(pk.group(1)) if pk and pk.group(1) != "-inf" else None)


def run(bank, style, track=None):
    args, bars, _ = STYLES[style]
    out_dir = L1 / "render" / style
    out_dir.mkdir(parents=True, exist_ok=True)
    stem = f"{bank}_{'mix' if track is None else 'solo-' + track}"
    wav = out_dir / f"{stem}.wav"
    mid = out_dir / f"{stem}.mid"
    cmd = [str(ENGINE), "--export", "--bars", str(bars), *args, "--seed", SEED,
           "--soundfont", BANKS[bank], "--record-wav", str(wav), "--record-midi", str(mid)]
    if track:
        cmd += ["--solo-track", track]
    p = subprocess.run(cmd, capture_output=True, text=True, cwd=ENGINE.parent.parent.parent)
    err = [l for l in (p.stdout + p.stderr).splitlines() if re.search(r"(?i)erreur|error|panic|Avertissement|WARN|absent", l)]
    il, pk = lufs(wav) if wav.exists() else (None, None)
    return {"bank": bank, "style": style, "track": track or "mix", "rc": p.returncode, "lufs": il, "peak": pk,
            "midi_sha": hashlib.sha256(mid.read_bytes()).hexdigest()[:12] if mid.exists() else None,
            "notes": [e for e in err if "Odin" not in e][:3], "wav": str(wav)}


def main():
    banks = sys.argv[1].split(",") if len(sys.argv) > 1 else list(BANKS)
    styles = sys.argv[2].split(",") if len(sys.argv) > 2 else list(STYLES)
    jobs = []
    for style in styles:
        for bank in banks:
            if style == "latin_gm" and bank.startswith("fp"):
                continue
            if style == "latin_fp" and not bank.startswith("fp"):
                continue
            jobs.append((bank, style, None))
            for t in STYLES[style][2]:
                jobs.append((bank, style, t))
    res = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
        for r in ex.map(lambda j: run(*j), jobs):
            res.append(r)
    out = L1 / "render" / "results.json"
    old = json.loads(out.read_text()) if out.exists() else []
    keep = {(r["bank"], r["style"], r["track"]): r for r in old}
    for r in res:
        keep[(r["bank"], r["style"], r["track"])] = r
    out.write_text(json.dumps(list(keep.values()), indent=1))
    bad = [r for r in res if r["rc"] != 0 or r["lufs"] is None]
    print(len(res), "rendus,", len(bad), "en échec")
    for r in bad:
        print("ÉCHEC", r["bank"], r["style"], r["track"], r["rc"], r["notes"])


if __name__ == "__main__":
    main()
```

## `palette.py`

Balayage des programmes utiles avec le moteur : un profil jetable par programme, solo, LUFS.

```python
#!/usr/bin/env python3
"""Balayage des programmes utiles par banque, avec le moteur (oxisynth) : un profil jetable par programme,
la piste concernée en solo (--solo-track), LUFS intégré mesuré (ebur128). Hors dépôt.

Un programme « sonne » si le solo mesure plus de -50 LUFS ; il est « absent » si la banque n'a pas de preset
(banque, programme) — lu dans les en-têtes SF2 (sf2info) — et il tombe alors sur le preset de repli du synthé.
"""
import concurrent.futures
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, "/tmp/harmonium-019/l1/work")
import sf2info  # noqa: E402

ENGINE = Path.home() / "Project/bascanada/harmonium/target/release/harmonium-cli"
CWD = ENGINE.parent.parent.parent
SRC = Path.home() / "orca/workspaces/harmonium/l1-banques-sf2/profiles/wjazzd_swing"
L1 = Path("/tmp/harmonium-019/l1")
B = L1 / "banks"
BANKS = {
    "gu": B / "generaluser.sf2",
    "fluid": B / "fluidr3.sf2",
    "msb2": B / "dl/MS_Basic-v2.0.0.sf2",
    "fp-up": B / "merged/fp-up.sf2",
    "fp-sal": B / "merged/fp-sal.sf2",
}
FAMILIES = {
    "Chords": {
        "piano 0": 0, "piano clair 1": 1, "Rhodes 4": 4, "EP FM 5": 5, "vibraphone 11": 11,
        "orgue drawbar 16": 16, "orgue perc. 17": 17, "orgue rock 18": 18, "accordéon 21": 21,
        "nylon 24": 24, "acier 25": 25, "guitare jazz 26": 26, "guitare claire 27": 27, "harpe 46": 46,
        "trompette 56": 56, "trombone 57": 57, "trompette sourdine 59": 59, "cor 60": 60, "cuivres 61": 61,
        "sax alto 65": 65, "sax ténor 66": 66, "clarinette 71": 71, "flûte 73": 73,
    },
    "Bass": {"basse ac. 32": 32, "basse doigts 33": 33, "basse médiator 34": 34, "fretless 35": 35, "slap 36": 36, "contrebasse 43": 43},
}
KITS = {"standard 0": 0, "power 16": 16, "jazz 32": 32, "balais 40": 40, "orchestre 48": 48}


def make_profile(tag, chords=None, bass=None, kit=None):
    d = L1 / "profiles" / f"pal_{tag}"
    if d.exists():
        shutil.rmtree(d)
    shutil.copytree(SRC, d)
    p = (d / "profile.toml").read_text(encoding="utf-8")
    if chords is not None:
        p = re.sub(r"(?m)^chords_program = .*$", f"chords_program = {chords}", p, count=1)
    if bass is not None:
        p = re.sub(r"(?m)^bass_program = .*$", f"bass_program = {bass}", p, count=1)
    if kit is not None:
        dy = (d / "dynamics.toml").read_text(encoding="utf-8")
        dy = re.sub(r"(?m)^percussion_intensity = .*$", "percussion_intensity = [[0.0, 1.0], [1.0, 1.0]]", dy, count=1)
        (d / "dynamics.toml").write_text(dy, encoding="utf-8")
        p = re.sub(r"(?m)^percussion_program = .*$", f"percussion_program = {kit}", p, count=1)
    (d / "profile.toml").write_text(p, encoding="utf-8")
    return d


def lufs(path):
    q = subprocess.run(["ffmpeg", "-nostats", "-i", str(path), "-af", "ebur128", "-f", "null", "-"],
                       capture_output=True, text=True).stderr.split("Summary:")[-1]
    m = re.search(r"I:\s+(-?[\d.]+) LUFS", q)
    return float(m.group(1)) if m else None


def job(bank, family, label, prog):
    tag = f"{bank}_{family}_{prog}".replace("/", "_")
    kw = {"Chords": {"chords": prog}, "Bass": {"bass": prog}, "Kit": {"kit": prog}}[family]
    d = make_profile(tag, **kw)
    wav = L1 / "render/palette" / f"{tag}.wav"
    wav.parent.mkdir(parents=True, exist_ok=True)
    tracks = ["PercussionLow", "PercussionHigh"] if family == "Kit" else [family]
    best = []
    for track in tracks:
        cmd = [str(ENGINE), "--export", "--bars", "12", "--profile", str(d), "--seed", "7", "--soundfont", str(BANKS[bank]),
               "--record-wav", str(wav), "--solo-track", track]
        subprocess.run(cmd, capture_output=True, text=True, cwd=CWD)
        best.append(lufs(wav) if wav.exists() else None)
    shutil.rmtree(d, ignore_errors=True)
    wav.unlink(missing_ok=True)
    return bank, family, label, prog, best if family == "Kit" else best[0]


def main():
    sel = sys.argv[1].split(",") if len(sys.argv) > 1 else list(BANKS)
    presence = {}
    for bank, path in BANKS.items():
        if bank not in sel:
            continue
        info = sf2info.parse(str(path))
        presence[bank] = {(p["bank"], p["program"]) for p in info["presets"]}
    jobs = []
    for bank in sel:
        for fam, items in FAMILIES.items():
            for label, prog in items.items():
                jobs.append((bank, fam, label, prog))
        for label, prog in KITS.items():
            jobs.append((bank, "Kit", label, prog))
    out = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
        for bank, fam, label, prog, l in ex.map(lambda j: job(*j), jobs):
            key = (128, prog) if fam == "Kit" else (0, prog)
            out.setdefault(bank, {})[f"{fam}|{label}"] = {"lufs": l, "preset": key in presence[bank]}
    f = L1 / "render/palette.json"
    old = json.loads(f.read_text()) if f.exists() else {}
    old.update(out)
    f.write_text(json.dumps(old, indent=1))
    print("ok", sum(len(v) for v in out.values()))


if __name__ == "__main__":
    main()
```

## `loadtest.py`

Chargement de chaque banque dans le moteur (code retour, avertissements, durée, LUFS).

```python
import subprocess,time,re,sys,os,json
ENGINE=os.path.expanduser('~/Project/bascanada/harmonium/target/release/harmonium-cli')
cwd=os.path.expanduser('~/Project/bascanada/harmonium')
B='/tmp/harmonium-019/l1/banks/'
files={
 'GeneralUser GS (sf2)':B+'generaluser.sf2','GeneralUser GS (sf3)':B+'sf3/generaluser.sf3',
 'FluidR3 (sf2)':B+'fluidr3.sf2','FluidR3 (sf3 q4)':B+'sf3/fluidr3.sf3',
 'MuseScore_General 0.2 (sf2)':B+'musescore.sf2','MuseScore_General 0.2 (sf3 officiel)':B+'dl/MuseScore_General.sf3','MuseScore_General 0.2 (sf3 q4)':B+'sf3/musescore.sf3',
 'MS Basic = HQ 0.2 (sf3 MuseScore main)':B+'dl/MS_Basic_HQ_0.2.sf3',
 'MS Basic 2.0.0 (sf2)':B+'dl/MS_Basic-v2.0.0.sf2','MS Basic 2.0.0 (sf3 miroir)':B+'dl/MS_Basic-v2.0.0.sf3','MS Basic 2.0.0 (sf3 q4)':B+'sf3/MS_Basic-v2.0.0-q4.sf3',
 'fp-up (sf2)':B+'merged/fp-up.sf2','fp-up (sf3)':B+'sf3/fp-up.sf3','fp-sal (sf2)':B+'merged/fp-sal.sf2','fp-sal (sf3)':B+'sf3/fp-sal.sf3',
 'Upright KW (sf3)':B+'sf3/UprightPianoKW-20220221.sf3','Disklavier YDP (sf3)':B+'sf3/YDP-GrandPiano-20160804.sf3','Salamander (sf3)':B+'sf3/Salamander.sf3',
 'MuldjordKit (sf3)':B+'sf3/MuldjordKit_20201018.sf3','World Percussion (sf3)':B+'sf3/WorldPercussion_20200905.sf3',
}
files['MS Basic = HQ 0.2 (sf3 MuseScore main)']=B+'dl/MS_Basic_HQ_0.2.sf3'
out={}
for k,f in files.items():
    wav='/tmp/harmonium-019/l1/render/t/load.wav'
    t=time.time()
    p=subprocess.run([ENGINE,'--export','--bars','8','--style','wjazzd_swing','--seed','7','--soundfont',f,'--record-wav',wav],capture_output=True,text=True,cwd=cwd)
    dt=time.time()-t
    txt=p.stdout+p.stderr
    warn=[l for l in txt.splitlines() if re.search(r'(?i)avertissement|erreur|error|panic|échec|failed|WARN',l) and 'Odin' not in l]
    lines=[l for l in txt.splitlines() if l.startswith('Loaded')]
    q=subprocess.run(['ffmpeg','-nostats','-i',wav,'-af','ebur128','-f','null','-'],capture_output=True,text=True).stderr.split('Summary:')[-1]
    m=re.search(r'I:\s+(-?[\d.]+) LUFS',q)
    out[k]={'rc':p.returncode,'s':round(dt,1),'lufs':m and float(m.group(1)),'loaded':lines[:1],'warn':warn[:2]}
    print(k,out[k],flush=True)
json.dump(out,open('/tmp/harmonium-019/l1/render/loadtest.json','w'),indent=1)
```

## `families.py`

Tableau programme × banque : nom de preset, échantillons, mégaoctets PCM, plage de touches.

```python
import sys,json
sys.path.insert(0,'.')
import sf2info
B='/tmp/harmonium-019/l1/banks/'
banks={'gu':B+'generaluser.sf2','fluid':B+'fluidr3.sf2','msb2':B+'dl/MS_Basic-v2.0.0.sf2'}
fp={'Upright KW':B+'sf2/UprightPianoKW-20220221.sf2','Disklavier':B+'sf2/YDP-GrandPiano-20160804.sf2','Salamander':B+'sf2/Salamander.sf2',
    'Drawbar':B+'sf2/DrawbarOrganEmulation-20190712.sf2','Percussive':B+'sf2/PercussiveOrganEmulation-20190715.sf2','Rock organ':B+'sf2/RockOrganEmulation-20190715.sf2',
    'Nylon':B+'sf2/SpanishClassicalGuitar-20190618.sf2','FSBS jazz':B+'sf2/EGuitarFSBS-jazz_bridge_small_20260807.sf2','FSBS clean':B+'sf2/EGuitarFSBS-clean_bridge_small_20260807.sf2',
    'Finger YR':B+'sf2/FingerBassYR_20190930.sf2','Picked YR':B+'sf2/PickedBassYR_20190930.sf2','Tenor sax':B+'sf2/TenorSaxophone-20200717.sf2','World perc':B+'sf2/WorldPercussion_20200905.sf2','Muldjord':B+'sf2/MuldjordKit_20201018.sf2'}
progs=[(0,0),(0,4),(0,5),(0,16),(0,17),(0,18),(0,24),(0,25),(0,26),(0,27),(0,32),(0,33),(0,34),(0,43),(0,56),(0,57),(0,59),(0,60),(0,61),(0,65),(0,66),(0,71),(128,0),(128,32),(128,40)]
out={}
for n,f in banks.items():
    r=sf2info.parse(f); d={(p['bank'],p['program']):p for p in r['presets']}
    out[n]={}
    for k in progs:
        p=d.get(k)
        out[n][k]=None if not p else {'name':p['name'],'samples':p['samples'],'mb':round(p['sample_points']*2/1e6,1),'keys':p['keys']}
o2={}
for n,f in fp.items():
    r=sf2info.parse(f); p=r['presets'][0]
    o2[n]={'name':p['name'],'samples':p['samples'],'mb':round(p['sample_points']*2/1e6,1),'keys':p['keys']}
json.dump({'gm':{n:{f'{a}:{b}':v for (a,b),v in d.items()} for n,d in out.items()},'fp':o2},open('/tmp/harmonium-019/l1/render/families.json','w'),indent=1)
for k in progs:
    print(k,[ (n, out[n][k] and (out[n][k]['name'],out[n][k]['samples'],out[n][k]['mb'])) for n in banks])
print(o2)
```

## `provenance.py`

Part des échantillons d'un preset présents à l'identique dans une autre banque (SHA-1 des échantillons).

```python
import hashlib,struct,sys,json
sys.path.insert(0,'.')
import sf2info
B='/tmp/harmonium-019/l1/banks/'
def sigs(path, progs):
    ver,p=sf2info.load(path)
    data=open(path,'rb').read()
    # smpl offset
    top=sf2info.chunks(data,12,len(data))
    sdta=next(c for c in top if c[2]==b'sdta')
    smpl=next(c for c in sf2info.chunks(sdta[1],0,len(sdta[1])) if c[0]==b'smpl')[1]
    r=sf2info.parse(path)
    shdr=[struct.unpack_from('<20sIIIIIBbHH',p[b'shdr'],i*46) for i in range(len(p[b'shdr'])//46)]
    # re-run sample collection per preset
    phdr=[struct.unpack_from('<20sHHHIII',p[b'phdr'],i*38) for i in range(len(p[b'phdr'])//38)]
    pbag=[struct.unpack_from('<HH',p[b'pbag'],i*4) for i in range(len(p[b'pbag'])//4)]
    pgen=[struct.unpack_from('<Hh',p[b'pgen'],i*4) for i in range(len(p[b'pgen'])//4)]
    inst=[struct.unpack_from('<20sH',p[b'inst'],i*22) for i in range(len(p[b'inst'])//22)]
    ibag=[struct.unpack_from('<HH',p[b'ibag'],i*4) for i in range(len(p[b'ibag'])//4)]
    igen=[struct.unpack_from('<Hh',p[b'igen'],i*4) for i in range(len(p[b'igen'])//4)]
    out={}
    for i in range(len(phdr)-1):
        key=(phdr[i][2],phdr[i][1])
        if key not in progs: continue
        hs=set()
        for b in range(phdr[i][3],phdr[i+1][3]):
            for g in range(pbag[b][0],pbag[b+1][0]):
                if pgen[g][0]==41:
                    ii=pgen[g][1]
                    for bb in range(inst[ii][1],inst[ii+1][1]):
                        for gg in range(ibag[bb][0],ibag[bb+1][0]):
                            if igen[gg][0]==53:
                                s=shdr[igen[gg][1]]
                                hs.add(hashlib.sha1(smpl[s[1]*2:s[2]*2]).hexdigest()[:10])
        out[key]=hs
    return out
progs={(0,k) for k in (0,4,5,16,18,24,25,26,27,32,33,34,43,56,57,59,60,61,65,66,71)}|{(128,k) for k in (0,16,32,40)}
S={n:sigs(B+f,progs) for n,f in [('gu','generaluser.sf2'),('fluid','fluidr3.sf2'),('msg','musescore.sf2'),('msb2','dl/MS_Basic-v2.0.0.sf2')]}
res={}
for key in sorted(progs):
    row={}
    for a,b in [('msb2','fluid'),('msb2','msg'),('gu','fluid'),('gu','msb2')]:
        A=S[a].get(key,set()); Bs=S[b].get(key,set())
        if not A or not Bs: row[f'{a}~{b}']=None; continue
        row[f'{a}~{b}']=round(len(A&Bs)/len(A),2)
    res[key]=row
for k,v in res.items(): print(k,v)
```

## `latinkeys.py`

Présence et crête de chaque touche latine d'un kit GM (fluidsynth).

```python
import pretty_midi, subprocess, re, json, sys
B='/tmp/harmonium-019/l1/banks/'
banks={'gu':B+'generaluser.sf2','fluid':B+'fluidr3.sf2','msb2':B+'dl/MS_Basic_v2.sf2' if False else B+'dl/MS_Basic-v2.0.0.sf2'}
KEYS={54:'tambourin',56:'cloche (cowbell)',58:'vibraslap',60:'bongo aigu',61:'bongo grave',62:'conga sourde',63:'conga ouverte',64:'conga grave',65:'timbale aiguë',66:'timbale grave',67:'agogo aigu',68:'agogo grave',69:'cabasa',70:'maracas',73:'guiro court',74:'guiro long',75:'claves',76:'woodblock aigu',77:'woodblock grave',78:'cuíca sourde',79:'cuíca ouverte',80:'triangle sourd',81:'triangle ouvert',82:'shaker'}
def measure(sf,prog,key):
    pm=pretty_midi.PrettyMIDI(initial_tempo=120); i=pretty_midi.Instrument(program=prog,is_drum=True); i.notes.append(pretty_midi.Note(velocity=100,pitch=key,start=0.05,end=0.3)); pm.instruments.append(i); pm.write('/tmp/harmonium-019/l1/render/t/d.mid')
    subprocess.run(['fluidsynth','-ni','-q','-F','/tmp/harmonium-019/l1/render/t/d.wav','-r','44100',sf,'/tmp/harmonium-019/l1/render/t/d.mid'],capture_output=True)
    q=subprocess.run(['ffmpeg','-nostats','-i','/tmp/harmonium-019/l1/render/t/d.wav','-af','astats=metadata=0','-f','null','-'],capture_output=True,text=True).stderr
    m=re.search(r'Peak level dB:\s+(-?[\d.inf]+)',q)
    return float(m.group(1)) if m and 'inf' not in m.group(1) else -99
out={}
for n,sf in banks.items():
    out[n]={KEYS[k]:measure(sf,0,k) for k in KEYS}
    print(n,{k:round(v) for k,v in out[n].items()})
# banques FreePats world percussion + Muldjord (touches propres)
json.dump(out,open('/tmp/harmonium-019/l1/render/latinkeys.json','w'),indent=1)
```

## `banks.json`

Banques finalistes pour `render_l1.py` (SF3 produits par `sf2_to_sf3.py`, qualité 4 : celle du studio).

```json
{"gu": "/tmp/harmonium-019/l1/banks/sf3/generaluser.sf3",
 "fluid": "/tmp/harmonium-019/l1/banks/sf3/fluidr3.sf3",
 "msb2": "/tmp/harmonium-019/l1/banks/sf3/MS_Basic-v2.0.0-q4.sf3",
 "fp-up": "/tmp/harmonium-019/l1/banks/sf3/fp-up.sf3",
 "fp-sal": "/tmp/harmonium-019/l1/banks/sf3/fp-sal.sf3"}
```
