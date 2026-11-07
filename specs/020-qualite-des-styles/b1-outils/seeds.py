#!/usr/bin/env python3
"""Choisit, pour chaque profil d'E1, la graine dont l'habillage expose le plus de programmes à juger.

Exécuter : H020_B1_REPO=<worktree> python seeds.py [graines max]. Lit `--record-truth` (`measures[].measure.instruments`,
programmes de chaque mesure) sur 64 mesures, sans écouter ni écrire de son : seul le JSON de vérité est utilisé.
Score = nombre de couples (rôle, programme) distincts parmi les programmes que B1 remplace ou juge, plus un point par kit
différent. Sortie : `seeds.json` (couples exposés et mesure de première apparition) et une ligne par profil.
"""
import concurrent.futures
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

repo = Path(os.environ.get('H020_B1_REPO', '.')).resolve()
GU = os.environ.get('H020_B1_GU', '/tmp/h020-b1/dl/generaluser.sf2')
out_dir = Path(os.environ.get('H020_B1_ROOT', '/tmp/h020-b1'))
styles = ('wjazzd_swing', 'wjazzd_ballad', 'gmd_jazz_waltz', 'gmd_latin_brazilian_bossa',
          'gmd_latin_brazilian_samba', 'gmd_latin_brazilian_baiao', 'gmd_afrocuban',
          'gmd_funk', 'gmd_soul', 'gmd_reggae', 'gmd_rock', 'gmd_pop', 'gmd_blues_shuffle')
weak = {'melody_program': {66}, 'chords_program': {4, 7, 16, 17, 18, 24, 26, 27, 28, 29, 30, 61, 25},
        'bass_program': {32, 33, 34, 35, 36}}
roles = ('melody_program', 'chords_program', 'bass_program')
n_seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 40


def exposed(style, seed):
    with tempfile.TemporaryDirectory() as tmp:
        truth = Path(tmp) / 'truth.json'
        subprocess.run([str(repo / 'target/release/harmonium-cli'), '--export', '--bars', '64', '--style', style,
                        '--seed', str(seed), '--soundfont', GU, '--record-truth', str(truth)],
                       cwd=repo, capture_output=True, check=True)
        measures = json.loads(truth.read_text())['measures']
    first = {}
    ins = {}
    for m in measures:
        ins.update(m['measure'].get('instruments') or {})  # les mesures sans changement n'ont pas de clé : reporter l'état
        for role in roles:
            p = ins.get(role)
            if p in weak[role]:
                first.setdefault(f'{role}:{p}', m['bar'])
        if ins.get('percussion_program') is not None:
            first.setdefault(f"kit:{ins['percussion_bank']}:{ins['percussion_program']}", m['bar'])
    return {'style': style, 'seed': seed, 'first_bar': first, 'score': len(first)}


if __name__ == '__main__':
    jobs = [(s, k) for s in styles for k in range(1, n_seeds + 1)]
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        res = list(pool.map(lambda j: exposed(*j), jobs))
    best = {}
    for r in res:
        b = best.get(r['style'])
        if b is None or (r['score'], -r['seed']) > (b['score'], -b['seed']):
            best[r['style']] = r
    (out_dir / 'seeds.json').write_text(json.dumps({'all': res, 'best': best}, indent=1))
    for s in styles:
        b = best[s]
        print(s.ljust(28), 'graine', b['seed'], 'score', b['score'], b['first_bar'])
