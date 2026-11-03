#!/usr/bin/env python3
"""A6 : les touches des rôles de percussion latine existent dans le kit GM de GeneralUser GS.

Lit `assets/GeneralUser-GS.sf2` (RIFF `pdta` : presets, zones, instruments) sans la charger dans le
moteur, et liste, pour chaque programme de kit (banque 128), l'instrument qui répond à chaque
touche des rôles de `profiles/README.md` (cloche 56, bongos 60-61, congas 62-64, timbales 65-66,
cabasa 69, maracas 70, claves 75, shaker 82). Sortie : une ligne par rôle et par programme.

    python3 specs/019-bibliotheque-accompagnements/a6-roles-latins-sf2.py [chemin.sf2]
"""
import struct
import sys
from pathlib import Path

ROLES = {56: "cloche", 60: "bongo aigu", 61: "bongo grave", 62: "conga sourde", 63: "conga ouverte",
         64: "conga grave", 65: "timbale aiguë", 66: "timbale grave", 69: "cabasa", 70: "maracas",
         75: "claves", 82: "shaker"}
KITS = {0: "Standard 1", 32: "Jazz", 40: "Brush"}


def chunks(data, off, end):
    while off < end:
        cid, size = data[off:off + 4], struct.unpack("<I", data[off + 4:off + 8])[0]
        yield cid, off + 8, size
        off += 8 + size + (size & 1)


def read_pdta(data):
    for cid, off, size in chunks(data, 12, len(data)):
        if cid == b"LIST" and data[off:off + 4] == b"pdta":
            return {c: data[o:o + s] for c, o, s in chunks(data, off + 4, off + size)}
    raise SystemExit("pdta absent")


def records(blob, fmt):
    step = struct.calcsize(fmt)
    return [struct.unpack(fmt, blob[i:i + step]) for i in range(0, len(blob), step)]


def main():
    path = Path(sys.argv[1] if len(sys.argv) > 1 else "assets/GeneralUser-GS.sf2")
    pdta = read_pdta(path.read_bytes())
    phdr, pbag, pgen = (records(pdta[k], f) for k, f in ((b"phdr", "<20sHHHIII"), (b"pbag", "<HH"), (b"pgen", "<Hh")))
    inst, ibag, igen = (records(pdta[k], f) for k, f in ((b"inst", "<20sH"), (b"ibag", "<HH"), (b"igen", "<Hh")))

    def inst_zones(index):
        zones = []
        for bag in range(inst[index][1], inst[index + 1][1]):
            gens = igen[ibag[bag][0]:ibag[bag + 1][0]]
            keys = next(((g[1] & 0xFF, (g[1] >> 8) & 0xFF) for g in gens if g[0] == 43), None)
            zones.append((keys, any(g[0] == 53 for g in gens)))
        return zones

    missing = 0
    for program, kit in KITS.items():
        keys = {}
        for p, header in enumerate(phdr[:-1]):
            name, preset, bank, first = header[0], header[1], header[2], header[3]
            if bank != 128 or preset != program:
                continue
            for bag in range(first, phdr[p + 1][3]):
                gens = pgen[pbag[bag][0]:pbag[bag + 1][0]]
                zone = next(((g[1] & 0xFF, (g[1] >> 8) & 0xFF) for g in gens if g[0] == 43), None)
                target = next((g[1] for g in gens if g[0] == 41), None)
                if target is None:
                    continue
                for instrument_keys, has_sample in inst_zones(target):
                    if not has_sample:
                        continue
                    lo, hi = instrument_keys or (0, 127)
                    if zone:
                        lo, hi = max(lo, zone[0]), min(hi, zone[1])
                    for key in range(lo, hi + 1):
                        keys.setdefault(key, inst[target][0].split(b"\0")[0].decode())
        for key, role in ROLES.items():
            found = keys.get(key)
            missing += found is None
            print(f"kit {program:>3} ({kit:<10}) touche {key:>3} {role:<14} -> {found or 'ABSENTE'}")
    raise SystemExit(1 if missing else 0)


if __name__ == "__main__":
    main()
