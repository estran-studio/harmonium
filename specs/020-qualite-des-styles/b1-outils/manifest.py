#!/usr/bin/env python3
"""Empreintes SHA256 des sources et des banques de B1 (manifeste, principe VII). Sortie Markdown.

Arbre de fichiers : on trie les chemins relatifs et on ajoute au hachage final, pour chaque fichier,
`chemin UTF-8 + NUL + SHA256(octets du fichier)` ; cela authentifie aussi README et licences, sans stocker les WAV dans Git.

Exécuter : H020_B1_ROOT=<sources> python manifest.py
"""
import hashlib
import os
from pathlib import Path

root = Path(os.environ.get('H020_B1_ROOT', '/tmp/h020-b1'))


def sha(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for block in iter(lambda: f.read(1 << 20), b''):
            h.update(block)
    return h.hexdigest()


def tree(path):
    files = sorted(p for p in path.rglob('*') if p.is_file())
    h = hashlib.sha256()
    total = 0
    for p in files:
        h.update(str(p.relative_to(path)).encode('utf-8') + b'\0' + sha(p).encode())
        total += p.stat().st_size
    return len(files), total, h.hexdigest()


print('| Archive FreePats (SF2 extrait) | SHA256 |\n|---|---|')
for p in sorted((root / 'sf2').glob('*/*.sf2')):
    print(f'| `{p.name}` | `{sha(p)}` |')
print('\n| Arbre | Fichiers ; octets | SHA256 de l\'arbre |\n|---|---|---|')
for p in sorted((root / 'src').glob('*/*')):
    if p.is_dir() and p.parent.name not in ('vcsl', 'vcsl-repo'):
        n, size, digest = tree(p)
        print(f'| `{p.parent.name}/{p.name}` | {n} ; {size} | `{digest}` |')
n, size, digest = tree(root / 'src/vcsl')
print(f'| `vcsl/` (sous-arbre sélectionné) | {n} ; {size} | `{digest}` |')
print('\n| Banque | Octets | SHA256 |\n|---|---:|---|')
for p in [root / 'dl/generaluser.sf2', root / 'dl/MS_Basic-v2.0.0.sf2', root / 'merged/b1-gu.sf2', root / 'merged/b1-msb.sf2'] + sorted((root / 'sf3').glob('*.sf3')):
    print(f'| `{p.name}` | {p.stat().st_size} | `{sha(p)}` |')
