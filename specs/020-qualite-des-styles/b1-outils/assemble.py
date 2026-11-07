#!/usr/bin/env python3
"""Banque GM d'essai B1 : banque complète + presets échantillonnés libres.

Exécuter : H020_B1_ROOT=<sources> H020_B1_GU=<GeneralUser-GS.sf2> python assemble.py gu|msb
Dépend de sf2build.py, sfzlib.py (dans ce dossier), archives FreePats et dépôts sfzinstruments (voir b1-banque-sons.md § 3).
Variables : H020_B1_NORMALIZE=0 (banque brute, sans gains de niveau) ; H020_B1_SKIP=29,30 (ne pas importer ces programmes).
Sortie : <ROOT>/merged/b1-{gu,msb}.sf2, déterministe (même SHA-256 à la reconstruction).
"""
import collections
import glob
import os
import sys
from pathlib import Path
from sf2build import Bank, Out
from sfzlib import BankZone, sfz_to_zones, read_audio, note_num

ROOT = Path(os.environ.get('H020_B1_ROOT', '/tmp/h020-b1'))
GU = Path(os.environ.get('H020_B1_GU', str(Path.home() / 'Project/bascanada/harmonium/assets/GeneralUser-GS.sf2')))
BANK = sys.argv[1]
assert BANK in ('gu', 'msb')
base_path = GU if BANK == 'gu' else ROOT / 'dl/MS_Basic-v2.0.0.sf2'
base = Bank(str(base_path))
out = Out()


def fp(pattern):
    files = glob.glob(str(ROOT / 'sf2' / pattern))
    assert len(files) == 1, (pattern, files)
    return files[0]


def src(repo, pattern):
    files = glob.glob(str(ROOT / 'src' / repo / '*' / pattern))
    assert len(files) == 1, (repo, pattern, files)
    return files[0]


# Les échantillons communs à plusieurs presets sont copiés une seule fois par Out.
# H020_B1_SKIP=29,30 : ne pas importer ces programmes (la banque de base garde les siens) ; mesure du coût d'une partie (poids, décodage).
SKIP = {int(p) for p in os.environ.get('H020_B1_SKIP', '').split(',') if p}
replaced = {(0, p) for p in (4, 7, 16, 17, 18, 24, 26, 27, 29, 30, 32, 33, 34, 66) if p not in SKIP}
# Kits acoustiques seulement : les kits électroniques 24 (Electronic) et 25 (808/909) restent ceux de la banque de base.
KITS = (0, 8, 16, 32, 40)
replaced.update((128, p) for p in KITS)
for i in range(len(base.phdr) - 1):
    b, p = base.phdr[i][2], base.phdr[i][1]
    if (b, p) not in replaced:
        out.add_preset(base, i, b, p)

# Niveaux : `palette.py` (mêmes notes, un programme à la fois) donne l'écart de chaque preset importé à celui de GeneralUser GS ;
# on ramène à GeneralUser GS (que le mixage des profils suppose) les presets plus forts. Les presets plus faibles ne sont pas
# remontés (écrêtage du PCM 16 bits : Sneakybass a déjà une crête à 0,94 pour −5 LU, le kick de Swirly est à 0,85) : écart
# restant = proposition de mixage du compte rendu.
# H020_B1_NORMALIZE=0 : banque brute (c'est elle que `palette.py` mesure pour calculer ces gains).
GAIN_DB = {17: -2.1, 18: -1.6, 24: -2.3, 26: -1.4, 27: -6.1, 29: -5.0, 30: -5.0, 33: -1.6, 34: -1.4, 66: -8.9}
GAIN_EP_DB = {4: -2.6}
if os.environ.get('H020_B1_NORMALIZE', '1') == '0':
    GAIN_DB, GAIN_EP_DB = {}, {}

replacements = [
    (0, 16, 'DrawbarOrgan*/*.sf2'), (0, 17, 'PercussiveOrgan*/*.sf2'),
    (0, 18, 'RockOrgan*/*.sf2'), (0, 24, 'SpanishClassicalGuitar*/*.sf2'),
    (0, 26, 'EGuitarFSBS-jazz*/*.sf2'), (0, 27, 'EGuitarFSBS-clean*/*.sf2'),
    (0, 29, 'EGuitarFSBS-dist2*/*.sf2'), (0, 30, 'EGuitarFSBS-dist2*/*.sf2'),
    (0, 33, 'FingerBassYR*/*.sf2'), (0, 34, 'PickedBassYR*/*.sf2'),
    (0, 66, 'TenorSaxophone*/*.sf2'),
]
fonts = {}
for bank, prog, pattern in replacements:
    if prog in SKIP:
        continue
    font = fonts.get(pattern)
    if font is None:
        font = fonts[pattern] = Bank(fp(pattern))
    out.add_preset(font, font.preset(0, 0), bank, prog, extend=True, gain_db=GAIN_DB.get(prog, 0.0))
    print(f'SF2 {bank}:{prog} {pattern}', flush=True)

# Wurlitzer réel (CC BY 3.0), au slot GM EP1 : pas un Rhodes ; Pianet au slot clavinet : pas un D6.
for prog, label, pattern in (
    (4, 'Wurlitzer EP200', 'Wurlitzer EP200/Wurlitzer EP200.sfz'),
    (7, 'Hohner Pianet T', 'Pianet T/Pianet T.sfz'),
):
    zones, dropped = sfz_to_zones(src('GregSullivan.E-Pianos', pattern), max_seconds=8)
    assert zones and all(reason == 'trigger' for _, reason in dropped), (prog, collections.Counter(reason for _, reason in dropped))
    out.add_zones_preset(label, 0, prog, zones, gain_db=GAIN_EP_DB.get(prog, 0.0))
    print(f'SFZ 0:{prog} {label}, {len(zones)} zones', flush=True)
    del zones

# Contrebasse pizzicato (CC0), 40 échantillons chromatiques ; étendre seulement les deux extrémités.
zz, drops = sfz_to_zones(src('karoryfer.sneakybass', 'Programs/02-sneakybass_pluck.sfz'), max_seconds=4)
assert len(zz) >= 35, collections.Counter(reason for _, reason in drops)
zz[0].key_lo = 0
zz[-1].key_hi = 127
out.add_zones_preset('Sneakybass pizz', 0, 32, zz)
print('SFZ 0:32 Sneakybass', len(zz), 'zones', flush=True)
del zz


# Kits : Swirly Drums CC0, frappes sèches, 1er round-robin ; profils GM de la base conservés
# pour les clés non couvertes. L'arrangement du jeu n'emploie pas les articulations SFZ (keyswitch/CC).
sw = src('karoryfer.swirly-drums', 'Programs/Full_kit.sfz')
kit_maps = {35: 35, 36: 36, 38: 38, 40: 40, 42: 42, 44: 44, 46: 46,
            41: 41, 43: 43, 45: 45, 47: 47, 48: 50, 49: 49, 51: 51, 53: 53}

def sw_zone(keys, dest=None, seconds=4):
    zz, drops = sfz_to_zones(sw, max_seconds=seconds, mix=True,
                              keep=lambda r: note_num(r.get('key', r.get('lokey', -1))) in keys)
    zz = [z for z in zz if z.key_lo in keys]
    if dest:
        for z in zz:
            z.root_key += dest - z.key_lo
            z.key_lo = z.key_hi = dest
    for z in zz:
        # Les coups doux de grosse caisse sont ~15 LU sous la référence si
        # normalisés comme les cymbales ; conserver le MIDI/vélocité, monter
        # seulement l'échantillon avec une marge sous la saturation PCM.
        peak = float(abs(z.audio).max())
        if peak:
            if z.key_lo in (35, 36):
                z.audio = z.audio * min(10 ** (36 / 20), 0.85 / peak)
            else:
                target = 0.15 + 0.5 * (z.vel_hi / 127)
                z.audio = z.audio * min(10 ** (18 / 20), target / peak)
        if z.key_lo in (42, 44, 46):
            z.exclusive_class = 1
        z.release_s = 0.1
    return zz

std = sw_zone(set(kit_maps), seconds=4)
for z in std:
    source = z.key_lo
    z.root_key += kit_maps[source] - source
    z.key_lo = z.key_hi = kit_maps[source]
print('Swirly kit zones', len(std), flush=True)
# Ride identique aux autres kits, kick/snare/hat changés. Brushes : véritable stir de Swirly sur 38/40.
brush = [z for z in std if z.key_lo not in (38, 40)] + sw_zone({60}, dest=38, seconds=3) + sw_zone({64}, dest=40, seconds=3)
# Percussions latines VCSL (CC0) dans les kits GM : les autres notes restent celles de la base.
# Timbales et cuica sans échantillon isolé vérifié : repli de la banque GM ; surdo hors GM.
V = ROOT / 'src/vcsl'
latin = {
    56: 'Cowbells', 60: 'Bongos', 61: 'Bongos', 62: 'Conga', 63: 'Conga', 64: 'Conga',
    67: 'Agogo Bells', 68: 'Agogo Bells', 70: 'Shaker, Small', 73: 'Guiro', 74: 'Guiro',
    75: 'Claves', 82: 'Shaker, Large',
}
percs = {}
for key, family in latin.items():
    files = sorted(V.glob(f'**/{family}/**/*.wav'))
    if not files:
        continue
    f = files[key % len(files)]
    a = read_audio(str(f))
    peak = float(abs(a).max())
    if peak:
        a *= min(10 ** (24 / 20), 0.5 / peak)
    if len(a) > 3 * 44100:
        a = a[:3 * 44100].copy()
    percs[key] = BankZone(sample_name=f'{family[:12]}-{key}', audio=a, root_key=key,
                           key_lo=key, key_hi=key, release_s=0.1)
assert len(percs) >= 10, sorted(percs)
print('Latin VCSL', sorted(percs), flush=True)
kit_base = base if BANK == 'gu' else Bank(str(GU))
for prog in KITS:
    zz = (brush if prog == 40 else std) + list(percs.values())
    out.overlay_kit(kit_base, kit_base.preset(128, prog), prog, zz, name=f'B1 Swirly {prog}')
    print('kit', prog, len(zz), flush=True)

output = ROOT / 'merged' / (f'b1-{BANK}.sf2' if os.environ.get('H020_B1_NORMALIZE', '1') != '0' else f'raw-{BANK}.sf2').replace('.sf2', ''.join(f'-sans{p}' for p in sorted(SKIP)) + '.sf2')
output.parent.mkdir(exist_ok=True)
out.write(str(output))
print(output, output.stat().st_size, len(out.presets), 'presets', flush=True)
