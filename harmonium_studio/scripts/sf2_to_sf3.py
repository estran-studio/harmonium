#!/usr/bin/env python3
"""Convertit une SoundFont SF2 (PCM 16 bits) en SF3 (échantillons Ogg Vorbis), comme sf3convert de MuseScore.

Banque principale du studio web (S8, décision du propriétaire : SF3). Chaque échantillon du chunk
`smpl` est encodé séparément par `oggenc` (mono, qualité 4 par défaut), les flux sont concaténés,
`shdr` est réécrit (début/fin en OCTETS dans `smpl`, boucles relatives au début, drapeau 0x10
Vorbis) et `ifil` passe à la version 3.0. Les présets, instruments et générateurs sont copiés tels
quels. Le numéro de série de chaque flux Ogg est le rang de l'échantillon (`oggenc -s`) : la même
entrée et la même version d'`oggenc` donnent le même fichier, octet pour octet.

Repris de la sonde S1 (`probes/s1-web/scripts/sf2_to_sf3.py`, branche `wquintal/s1-sonde-web`).

Usage : sf2_to_sf3.py entrée.sf2 sortie.sf3 [qualité_vorbis=4]
"""
import concurrent.futures
import os
import struct
import subprocess
import sys


def read_chunks(data, pos, end):
    """Liste des (id, contenu, type_liste) d'une zone RIFF."""
    out = []
    while pos < end:
        cid = data[pos:pos + 4]
        size = struct.unpack_from('<I', data, pos + 4)[0]
        body = data[pos + 8:pos + 8 + size]
        if cid == b'LIST':
            out.append((cid, body[4:], body[:4]))
        else:
            out.append((cid, body, None))
        pos += 8 + size + (size & 1)
    return out


def chunk(cid, body):
    pad = b'\0' if len(body) & 1 else b''
    return cid + struct.pack('<I', len(body)) + body + pad


def lst(kind, chunks):
    body = kind + b''.join(chunks)
    return chunk(b'LIST', body)


def encode(pcm_bytes, rate, quality, serial):
    p = subprocess.run(
        ['oggenc', '-Q', '-q', str(quality), '-s', str(serial), '-r', '-B', '16', '-C', '1', '-R', str(rate),
         '--raw-endianness', '0', '-o', '-', '-'],
        input=pcm_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
    return p.stdout


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    src, dst = sys.argv[1], sys.argv[2]
    quality = float(sys.argv[3]) if len(sys.argv) > 3 else 4
    data = open(src, 'rb').read()
    if data[:4] != b'RIFF' or data[8:12] != b'sfbk':
        sys.exit(f'{src} : pas une SoundFont (RIFF sfbk)')
    top = read_chunks(data, 12, len(data))
    info = next(c for c in top if c[2] == b'INFO')
    sdta = next(c for c in top if c[2] == b'sdta')
    pdta = next(c for c in top if c[2] == b'pdta')
    smpl = next(c for c in read_chunks(sdta[1], 0, len(sdta[1])) if c[0] == b'smpl')[1]
    pdta_chunks = read_chunks(pdta[1], 0, len(pdta[1]))
    shdr = next(c for c in pdta_chunks if c[0] == b'shdr')[1]
    n = len(shdr) // 46
    recs = [struct.unpack_from('<20sIIIIIBbHH', shdr, i * 46) for i in range(n)]

    jobs = []
    for i, (name, start, end, sl, el, rate, pitch, corr, link, typ) in enumerate(recs[:-1]):
        jobs.append((i, smpl[start * 2:end * 2], rate))
    streams = [None] * len(jobs)
    bad = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count()) as ex:
        futs = {ex.submit(encode, pcm, rate, quality, i): i for i, pcm, rate in jobs}
        for f in concurrent.futures.as_completed(futs):
            i = futs[f]
            try:
                streams[i] = f.result()
            except subprocess.CalledProcessError as e:
                bad += 1
                print(f'échantillon {i} ({recs[i][0]!r}, {len(jobs[i][1]) // 2} points) : oggenc a échoué : {e.stderr[:100]!r}',
                      file=sys.stderr)
    if bad:
        sys.exit(f'{bad} échantillons non encodés')

    new_smpl = bytearray()
    new_shdr = bytearray()
    for (name, start, end, sl, el, rate, pitch, corr, link, typ), ogg in zip(recs[:-1], streams):
        off = len(new_smpl)
        new_smpl += ogg
        new_shdr += struct.pack('<20sIIIIIBbHH', name, off, off + len(ogg), max(sl - start, 0), max(el - start, 0), rate,
                                pitch, corr, link, typ | 0x10)
    name, start, end, sl, el, rate, pitch, corr, link, typ = recs[-1]
    new_shdr += struct.pack('<20sIIIIIBbHH', name, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    if len(new_smpl) & 1:  # oxisynth lit `smpl` comme des i16 : une longueur impaire tronquerait le dernier octet
        new_smpl.append(0)

    info_chunks = []
    for cid, body, kind in read_chunks(info[1], 0, len(info[1])):
        if cid == b'ifil':
            body = struct.pack('<HH', 3, 0)
        info_chunks.append(chunk(cid, body))
    pdta_out = []
    for cid, body, kind in pdta_chunks:
        pdta_out.append(chunk(cid, bytes(new_shdr) if cid == b'shdr' else body))
    riff_body = (b'sfbk' + lst(b'INFO', info_chunks) + lst(b'sdta', [chunk(b'smpl', bytes(new_smpl))])
                 + lst(b'pdta', pdta_out))
    open(dst, 'wb').write(b'RIFF' + struct.pack('<I', len(riff_body)) + riff_body)
    print(f'{n - 1} échantillons ; SF2 {len(data)} octets -> SF3 {os.path.getsize(dst)} octets '
          f'(smpl {len(smpl)} -> {len(new_smpl)})')


if __name__ == '__main__':
    main()
