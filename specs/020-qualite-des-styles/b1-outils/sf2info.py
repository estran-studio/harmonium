#!/usr/bin/env python3
"""Liste les presets d'un SF2/SF3 (banque, programme, nom, nb d'échantillons, plage de touches) et la couverture de touches
d'un preset — lecture seule, hors moteur. Repris de l'annexe de L1 (`l1-annexe-outils.md`).

Usage : sf2info.py banque.sf2 [--json] ; ou, en module : coverage(path, bank, program) -> ensemble de touches avec échantillon.
"""
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


def _tables(p):
    def rec(fmt, key):
        size = struct.calcsize(fmt)
        return [struct.unpack_from(fmt, p[key], i * size) for i in range(len(p[key]) // size)]
    return (rec('<20sHHHIII', b'phdr'), rec('<HH', b'pbag'), rec('<Hh', b'pgen'), rec('<20sH', b'inst'),
            rec('<HH', b'ibag'), rec('<Hh', b'igen'), rec('<20sIIIIIBbHH', b'shdr'))


def coverage(path, bank, program):
    """Touches (0..127) qui ont au moins une zone d'échantillon dans le preset (bank, program)."""
    _, p = load(path)
    phdr, pbag, pgen, inst, ibag, igen, _ = _tables(p)
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
                    keys.update(range(max(pk[0], ik[0]), min(pk[1], ik[1]) + 1))
    return keys


def parse(path):
    ver, p = load(path)
    phdr, pbag, pgen, inst, ibag, igen, shdr = _tables(p)
    presets = []
    for i in range(len(phdr) - 1):
        name, prog, bank, bidx, _, _, _ = phdr[i]
        insts = [pgen[g][1] for b in range(bidx, phdr[i + 1][3]) for g in range(pbag[b][0], pbag[b + 1][0]) if pgen[g][0] == 41]
        samples = set()
        lo, hi = 127, 0
        for ii in insts:
            for b in range(inst[ii][1], inst[ii + 1][1]):
                sid = kr = None
                for g in range(ibag[b][0], ibag[b + 1][0]):
                    if igen[g][0] == 53:
                        sid = igen[g][1]
                    if igen[g][0] == 43:
                        kr = igen[g][1]
                if sid is not None:
                    samples.add(sid)
                    if kr is not None:
                        lo, hi = min(lo, kr & 0xFF), max(hi, kr >> 8)
                    else:
                        lo, hi = 0, 127
        pcm = sum(shdr[s][2] - shdr[s][1] for s in samples)
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
