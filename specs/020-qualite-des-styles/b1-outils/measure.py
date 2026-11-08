#!/usr/bin/env python3
"""Rendus E1 : mix et pistes isolées, LUFS ebur128 comme L1 §8, mêmes notes/seed.

Exécuter : python measure.py nom banque.sf2|sf3 (graine de chaque style : `seeds.json` de seeds.py, 64 mesures). Stocke uniquement les mesures et SHA MIDI, pas les WAV.
"""
import concurrent.futures
import hashlib
import json
import re
import subprocess
import sys
import os
from pathlib import Path

repo = Path(os.environ.get('H020_B1_REPO', '.')).resolve()
base = Path(os.environ.get('H020_B1_ROOT', '/tmp/h020-b1'))
trad = Path(os.environ.get('H020_B1_TRAD', str(Path.home() / 'Project/bascanada/harmonium/assets/harmonium_trad.sf2')))
name, font = sys.argv[1], Path(sys.argv[2]).resolve()
bars = os.environ.get('H020_B1_BARS', '64')
seeds = {k: v['seed'] for k, v in json.loads((base / 'seeds.json').read_text())['best'].items()}  # seeds.py
styles = ('wjazzd_swing', 'wjazzd_ballad', 'gmd_jazz_waltz', 'gmd_latin_brazilian_bossa',
          'gmd_latin_brazilian_samba', 'gmd_latin_brazilian_baiao', 'gmd_afrocuban',
          'gmd_funk', 'gmd_soul', 'gmd_reggae', 'gmd_rock', 'gmd_pop', 'gmd_blues_shuffle')
tracks = tuple(os.environ.get('H020_B1_TRACKS', 'mix,Chords,Bass,PercussionLow,PercussionHigh').split(','))

def measure(style, track):
    file = base / 'render' / f'{name}-{style}-{track}'
    wav = file.with_suffix('.wav')
    midi = file.with_suffix('.mid')
    cmd = [str(repo / 'target/release/harmonium-cli'), '--export', '--bars', bars, '--style', style,
           '--seed', str(seeds[style]), '--soundfont', str(font),
           '--percussion-soundfont', str(trad),
           '--record-wav', str(wav), '--record-midi', str(midi)]
    if track != 'mix':
        cmd += ['--solo-track', track]
    p = subprocess.run(cmd, cwd=repo, capture_output=True, text=True)
    err = [x for x in (p.stdout + p.stderr).splitlines()
           if re.search(r'(?i)erreur|error|panic|warn|absent|introuvable', x) and 'Odin' not in x]
    md = hashlib.sha256(midi.read_bytes()).hexdigest()[:16] if midi.exists() else None
    midi.unlink(missing_ok=True)
    loud, peak = None, None
    if wav.exists():
        q = subprocess.run(['ffmpeg', '-nostats', '-v', 'info', '-i', str(wav), '-af',
                            'ebur128=peak=true', '-f', 'null', '-'], capture_output=True, text=True)
        summary = q.stderr.split('Summary:')[-1]
        m = re.search(r'I:\s+(-?[\d.]+|-inf) LUFS', summary)
        k = re.search(r'Peak:\s+(-?[\d.]+|-inf) dBFS', summary)
        if m: loud = float(m.group(1))
        if k: peak = float(k.group(1))
        wav.unlink(missing_ok=True)
    return {'style': style, 'track': track, 'rc': p.returncode, 'lufs': loud,
            'peak': peak, 'midi_sha': md, 'errors': err[:3]}

if __name__ == '__main__':
    (base / 'render').mkdir(exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(lambda pair: measure(*pair), ((s,t) for s in styles for t in tracks)))
    path = base / 'render' / f'{name}.json'
    path.write_text(json.dumps(results, indent=2))
    bad = [r for r in results if r['rc'] or r['lufs'] is None or r['errors']]
    print(name, 'rendus', len(results), 'erreurs', len(bad), 'sortie', path, flush=True)
    for r in bad:
        print('ÉCHEC', r, flush=True)
    if bad: sys.exit(1)
