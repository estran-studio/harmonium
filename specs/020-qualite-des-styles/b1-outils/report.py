#!/usr/bin/env python3
"""Tableaux du compte rendu B1 à partir de `render/*.json` (measure.py) : LUFS par piste en solo, écarts à GeneralUser GS,
crête du mix et correction de `mix.tracks.<piste>.level_db` proposée (−écart ; non appliquée).

Exécuter : H020_B1_REPO=<worktree> python report.py ; sortie Markdown sur stdout.
"""
import json
import os
import re
from pathlib import Path

repo = Path(os.environ.get('H020_B1_REPO', '.')).resolve()
root = Path(os.environ.get('H020_B1_ROOT', '/tmp/h020-b1'))
seeds = {k: v['seed'] for k, v in json.loads((root / 'seeds.json').read_text())['best'].items()}
REF = 'gu'
CANDS = ('ms-basic', 'b1-gu', 'b1-msb')
TRACKS = ('mix', 'Chords', 'Bass', 'PercussionLow', 'PercussionHigh')
SHORT = {'Chords': 'Accords', 'Bass': 'Basse', 'PercussionLow': 'Perc. grave', 'PercussionHigh': 'Perc. aiguë'}
MUTE = -60.0  # sous ce seuil la piste est muette (−70 LUFS = silence) : pas de gain utile


def load(name):
    return {(r['style'], r['track']): r for r in json.loads((root / 'render' / f'{name}.json').read_text())}


def level_db(style, track):
    text = (repo / 'profiles' / style / 'profile.toml').read_text(encoding='utf-8')
    m = re.search(rf'\[mix\.tracks\.{track}\]\s*\nlevel_db = (-?[\d.]+)', text)
    return float(m.group(1)) if m else None


def fmt(v):
    return '—' if v is None or v < MUTE else f'{v:.1f}'


def delta(cand, ref):
    if cand is None or ref is None or cand < MUTE or ref < MUTE:
        return None
    return cand - ref


data = {n: load(n) for n in (REF,) + CANDS}
styles = list(seeds)

print('### LUFS par piste en solo et mix (64 mesures, graine de la colonne 2)\n')
print('| Profil | Graine | Piste | GeneralUser GS | ms-basic | b1-gu | b1-msb |')
print('|---|---:|---|---:|---:|---:|---:|')
for s in styles:
    for t in TRACKS:
        row = [fmt(data[n][(s, t)]['lufs']) for n in (REF,) + CANDS]
        print(f"| {s if t == 'mix' else ''} | {seeds[s] if t == 'mix' else ''} | {SHORT.get(t, 'mix')} | " + ' | '.join(row) + ' |')

print('\n### Écarts à GeneralUser GS (LU, candidate − référence ; « — » = piste muette d\'un côté) et crête du mix (dBFS)\n')
print('| Profil | Banque | mix | Accords | Basse | Perc. grave | Perc. aiguë | crête mix |')
print('|---|---|---:|---:|---:|---:|---:|---:|')
for s in styles:
    for n in CANDS:
        ds = []
        for t in TRACKS:
            d = delta(data[n][(s, t)]['lufs'], data[REF][(s, t)]['lufs'])
            ds.append('—' if d is None else f'{d:+.1f}')
        print(f"| {s if n == CANDS[0] else ''} | {n} | " + ' | '.join(ds) + f" | {data[n][(s, 'mix')]['peak']:.1f} (réf. {data[REF][(s, 'mix')]['peak']:.1f}) |")

print('\n### Correction de mix proposée : nouveau `level_db` (actuel → proposé), pistes jouées seulement\n')
print('| Profil | Banque | Chords | Bass | PercussionLow | PercussionHigh |')
print('|---|---|---|---|---|---|')
for s in styles:
    for n in CANDS:
        cells = []
        for t in TRACKS[1:]:
            cur = level_db(s, t)
            d = delta(data[n][(s, t)]['lufs'], data[REF][(s, t)]['lufs'])
            cells.append('—' if d is None or cur is None else f'{cur:.1f} → {cur - d:.1f}')
        print(f"| {s if n == CANDS[0] else ''} | {n} | " + ' | '.join(cells) + ' |')

same = all(data[n][k]['midi_sha'] == data[REF][k]['midi_sha'] for n in CANDS for k in data[REF])
print(f"\nNotes MIDI identiques entre banques sur les {len(data[REF])} rendus : {'oui' if same else 'NON'} ; "
      f"erreurs de rendu : {sum(1 for n in data for r in data[n].values() if r['rc'] or r['errors'])}.")
