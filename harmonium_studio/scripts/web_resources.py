#!/usr/bin/env python3
"""Ressources du studio web (S8) : banques et presets Odin, publiés en assets d'une release GitHub.

Le dépôt ne versionne aucune banque ni aucun preset : il versionne `harmonium_studio/web-resources.json`,
qui fixe la release, et pour chaque fichier son rôle, son nom, son nom d'asset, sa taille et son SHA-256.

Commandes :

  fetch --out DOSSIER [--from DOSSIER_LOCAL]
      Télécharge chaque ressource de la release (ou la copie depuis DOSSIER_LOCAL, qui contient les
      assets sous leur nom d'asset), contrôle taille et SHA-256, et la range sous DOSSIER/main/,
      DOSSIER/feet/ ou DOSSIER/odin/ sous son nom d'origine (le nom d'un preset est son nom de
      fichier). Une ressource qui ne correspond pas au fichier de verrou fait échouer la commande.

  check-profiles [--profiles DOSSIER]
      Échoue si un profil de `profiles/` nomme un preset Odin absent du verrou : la vitrine le jouerait
      en SoundFont.

  stage --tag TAG --main-bank F --feet-bank F --odin-dir D --license F --out DOSSIER [--profiles DOSSIER]
      Prépare une nouvelle release : copie sous leur nom d'asset la banque principale, la banque de
      pieds, les presets Odin que nomment les profils (et eux seuls) et la licence de la banque, puis
      réécrit le verrou. La publication reste à faire : `gh release create TAG DOSSIER/*`.

Python 3.11 ou plus (tomllib), sans dépendance.
"""
import argparse
import hashlib
import json
import re
import shutil
import sys
import tomllib
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / 'harmonium_studio' / 'web-resources.json'
PROFILES = ROOT / 'profiles'
REPOSITORY = 'estran-studio/harmonium'
# Sous-dossier de `fetch --out` par rôle.
ROLE_DIRS = {'main-bank': 'main', 'feet-bank': 'feet', 'odin': 'odin', 'license': 'license'}


def sha256(path):
    digest = hashlib.sha256()
    with open(path, 'rb') as f:
        for block in iter(lambda: f.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def asset_name(role, file):
    """Nom d'asset sans espace ni crochet (GitHub les remplace) : `odin-Bass-Crusher-RM.odin`."""
    stem, dot, ext = file.rpartition('.')
    safe = re.sub(r'[^A-Za-z0-9._-]+', '-', stem).strip('-')
    prefix = 'odin-' if role == 'odin' else ''
    return f'{prefix}{safe}{dot}{ext}'


def load_lock():
    return json.loads(LOCK.read_text())


def referenced_presets(profiles):
    """Presets Odin nommés par les profils (`[[odin.<piste>.zones]] preset`), avec les profils qui les citent."""
    presets = {}
    for profile in sorted(profiles.glob('*/profile.toml')):
        data = tomllib.loads(profile.read_text())
        for track in data.get('odin', {}).values():
            for zone in track.get('zones', []) if isinstance(track, dict) else []:
                users = presets.setdefault(zone['preset'], [])
                if profile.parent.name not in users:
                    users.append(profile.parent.name)
    if not presets:
        sys.exit(f'aucun preset Odin trouvé dans {profiles} : le format de `[odin]` a-t-il changé ?')
    return presets


def cmd_check_profiles(args):
    locked = {Path(r['file']).stem for r in load_lock()['resources'] if r['role'] == 'odin'}
    presets = referenced_presets(args.profiles)
    missing = {name: users for name, users in presets.items() if name not in locked}
    for name, users in sorted(missing.items()):
        print(f'preset Odin « {name} » ({", ".join(users)}) absent de {LOCK.name}', file=sys.stderr)
    if missing:
        sys.exit(1)
    print(f'{len(locked)} presets Odin dans {LOCK.name} ; les {len(presets)} presets nommés par les profils y sont')


def cmd_fetch(args):
    lock = load_lock()
    base = f'https://github.com/{lock["repository"]}/releases/download/{lock["release"]}'
    errors = []
    for resource in lock['resources']:
        target = args.out / ROLE_DIRS[resource['role']] / resource['file']
        target.parent.mkdir(parents=True, exist_ok=True)
        if not (target.exists() and target.stat().st_size == resource['bytes'] and sha256(target) == resource['sha256']):
            if args.source:
                shutil.copyfile(args.source / resource['asset'], target)
            else:
                url = f'{base}/{resource["asset"]}'
                print(f'downloading {url}')
                with urllib.request.urlopen(url) as response, open(target, 'wb') as out:
                    shutil.copyfileobj(response, out)
        size, digest = target.stat().st_size, sha256(target)
        if size != resource['bytes'] or digest != resource['sha256']:
            errors.append(f'{resource["asset"]}: {size} octets, sha256 {digest} ; '
                          f'verrou : {resource["bytes"]} octets, sha256 {resource["sha256"]}')
            target.unlink()
        else:
            print(f'{resource["role"]:<10} {size:>10} octets  {resource["file"]}  ok')
    if errors:
        print('ressources non conformes au verrou :', *errors, sep='\n  ', file=sys.stderr)
        sys.exit(1)


def cmd_stage(args):
    presets = referenced_presets(args.profiles)
    files = [('main-bank', args.main_bank), ('feet-bank', args.feet_bank), ('license', args.license)]
    for name in sorted(presets):
        path = args.odin_dir / f'{name}.odin'
        if not path.exists():
            sys.exit(f'preset « {name} » ({", ".join(presets[name])}) absent de {args.odin_dir}')
        files.append(('odin', path))
    args.out.mkdir(parents=True, exist_ok=True)
    resources = []
    for role, path in files:
        path = path.resolve()
        asset = asset_name(role, path.name)
        shutil.copyfile(path, args.out / asset)
        resources.append({'role': role, 'file': path.name, 'asset': asset,
                          'bytes': path.stat().st_size, 'sha256': sha256(path)})
        print(f'{role:<10} {path.stat().st_size:>10} octets  {asset}')
    LOCK.write_text(json.dumps({'repository': REPOSITORY, 'release': args.tag, 'resources': resources},
                               indent=2, ensure_ascii=False) + '\n')
    print(f'{LOCK.relative_to(ROOT)} réécrit ; publier : gh release create {args.tag} {args.out}/*')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest='command', required=True)
    fetch = sub.add_parser('fetch')
    fetch.add_argument('--out', type=Path, required=True)
    fetch.add_argument('--from', dest='source', type=Path)
    check = sub.add_parser('check-profiles')
    check.add_argument('--profiles', type=Path, default=PROFILES)
    stage = sub.add_parser('stage')
    stage.add_argument('--tag', required=True)
    stage.add_argument('--main-bank', type=Path, required=True)
    stage.add_argument('--feet-bank', type=Path, required=True)
    stage.add_argument('--odin-dir', type=Path, required=True)
    stage.add_argument('--license', type=Path, required=True)
    stage.add_argument('--out', type=Path, required=True)
    stage.add_argument('--profiles', type=Path, default=PROFILES)
    args = parser.parse_args()
    {'fetch': cmd_fetch, 'check-profiles': cmd_check_profiles, 'stage': cmd_stage}[args.command](args)


if __name__ == '__main__':
    main()
