#!/usr/bin/env python3
"""Niveau par programme : mêmes notes (profil jetable dérivé de `wjazzd_swing`, graine 7, 12 mesures), un programme à la fois,
la piste en solo, LUFS intégré ebur128. Même méthode que `palette.py` de L1 (`l1-annexe-outils.md`), banques de B1.

Exécuter : H020_B1_REPO=<worktree> python palette.py nom=banque.sf2 [nom=banque.sf2 …] ; écrit `palette.json` dans H020_B1_ROOT.
Le delta à la banque de référence `gu` d'un même programme donne la correction de niveau de la piste (−delta dB) si ce
programme joue ; il est indépendant du profil, ce que ne dit pas la mesure par profil de `measure.py`.
"""
import concurrent.futures
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

repo = Path(os.environ.get('H020_B1_REPO', '.')).resolve()
root = Path(os.environ.get('H020_B1_ROOT', '/tmp/h020-b1'))
FAMILIES = {
    'Chords': (0, 3, 4, 7, 16, 17, 18, 21, 24, 25, 26, 27, 28, 29, 30, 61, 66),
    'Bass': (32, 33, 34, 35, 36, 38),
}
KITS = (0, 8, 16, 24, 25, 32, 40)


def make_profile(tag, chords=None, bass=None, kit=None):
    d = root / 'profiles' / f'pal_{tag}'
    shutil.rmtree(d, ignore_errors=True)
    shutil.copytree(repo / 'profiles/wjazzd_swing', d)
    # L'habillage par graine (`session.toml`) impose ses programmes : sans ce retrait, `profile.toml` est ignoré.
    sess = (d / 'session.toml').read_text(encoding='utf-8')
    sess = re.sub(r'instruments = \{[^}]*\},[ \t]*', '', sess)
    sess = re.sub(r',[ \t]*instruments = \{[^}]*\}', '', sess)
    sess = re.sub(r'(?m)^instruments = \{[^}]*\}\n', '', sess)
    (d / 'session.toml').write_text(sess, encoding='utf-8')
    p = (d / 'profile.toml').read_text(encoding='utf-8')
    if chords is not None:
        p = re.sub(r'(?m)^chords_program = .*$', f'chords_program = {chords}', p, count=1)
    if bass is not None:
        p = re.sub(r'(?m)^bass_program = .*$', f'bass_program = {bass}', p, count=1)
    if kit is not None:  # densité maximale constante : grosse caisse, caisse claire et toms jouent
        dy = (d / 'dynamics.toml').read_text(encoding='utf-8')
        dy = re.sub(r'(?m)^percussion_intensity = .*$', 'percussion_intensity = [[0.0, 1.0], [1.0, 1.0]]', dy, count=1)
        (d / 'dynamics.toml').write_text(dy, encoding='utf-8')
        p = re.sub(r'(?m)^percussion_program = .*$', f'percussion_program = {kit}', p, count=1)
    (d / 'profile.toml').write_text(p, encoding='utf-8')
    return d


def lufs(path):
    q = subprocess.run(['ffmpeg', '-nostats', '-i', str(path), '-af', 'ebur128', '-f', 'null', '-'],
                       capture_output=True, text=True).stderr.split('Summary:')[-1]
    m = re.search(r'I:\s+(-?[\d.]+) LUFS', q)
    return float(m.group(1)) if m else None


def job(bank, font, family, prog):
    tag = f'{bank}_{family}_{prog}'
    d = make_profile(tag, **{'Chords': {'chords': prog}, 'Bass': {'bass': prog}, 'Kit': {'kit': prog}}[family])
    wav = root / 'render/palette' / f'{tag}.wav'
    wav.parent.mkdir(parents=True, exist_ok=True)
    tracks = ['PercussionLow', 'PercussionHigh'] if family == 'Kit' else [family]
    res = []
    for track in tracks:
        subprocess.run([str(repo / 'target/release/harmonium-cli'), '--export', '--bars', '12', '--profile', str(d), '--seed', '7',
                        '--soundfont', font, '--record-wav', str(wav), '--solo-track', track],
                       capture_output=True, text=True, cwd=repo)
        res.append(lufs(wav) if wav.exists() else None)
        wav.unlink(missing_ok=True)
    shutil.rmtree(d, ignore_errors=True)
    return bank, family, prog, res if family == 'Kit' else res[0]


if __name__ == '__main__':
    banks = dict(a.split('=', 1) for a in sys.argv[1:])
    jobs = [(b, f, fam, p) for b, f in banks.items() for fam, progs in FAMILIES.items() for p in progs]
    jobs += [(b, f, 'Kit', p) for b, f in banks.items() for p in KITS]
    out = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        for bank, family, prog, v in pool.map(lambda j: job(*j), jobs):
            out.setdefault(bank, {})[f'{family}:{prog}'] = v
    (root / 'palette.json').write_text(json.dumps(out, indent=1, sort_keys=True))
    print('ok', sum(len(v) for v in out.values()))
