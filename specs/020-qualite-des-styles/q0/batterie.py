#!/usr/bin/env python3
"""Q0 : corrige les probabilités des coups de grosse caisse et de caisse claire sans changer les phrases.

À lancer après `appliquer.py` : écrit `percussion.toml` depuis 019 (`git show BASE`), jamais à partir

 d'un profil déjà transformé. Les phrases de 2/4 mesures portent le groove, les coups secondaires
 restent des ornements probabilistes (tirage sans état par mesure, reproductible au seek).
"""
from __future__ import annotations

import argparse
import re
from pathlib import Path

from appliquer import BASE, E1, REPO, git_show

# Réglages par profil, trouvés par `specs/020-qualite-des-styles/q0/tuner_batterie.py` (recherche coordonnée par coordonnée sur 16 graines, groove de la
# batterie rendue contre le groupe GMD du profil ; voir le compte rendu). Aucun morceau ni note MIDI d'un fichier du corpus n'est
# copié : les phrases et les patterns de 019 restent, Q0 ne touche qu'aux probabilités, à l'ajout de notes fantômes, aux poids
# des phrases tenues et à la part de fins de phrase sans relance.
#   fixed       probabilité des coups de grosse caisse et de caisse claire hors temps forts qui étaient certains (1 : inchangés) ;
#               le premier temps et le backbeat ne sont jamais touchés
#   extra       nombre de notes fantômes de caisse claire ajoutées (places libres : contretemps d'abord, puis doubles croches)
#   ghost_chance probabilité de chaque note fantôme
#   prob_exp    exposant des probabilités existantes des coups de grosse caisse et de caisse claire (> 1 : plus de variation)
#   hold        poids des phrases qui répètent un seul pattern (groove tenu sur la phrase), multiplicateur
#   fill_scale  multiplicateur de `fills_none_weight` (part des fins de phrase sans relance)
JAZZ = ('wjazzd_swing', 'wjazzd_ballad', 'gmd_jazz_waltz')
#: Voix à ne jamais rendre probabilistes, par profil.
SKIP: dict[str, set[str]] = {'gmd_latin_brazilian_baiao': {'kick'}}

PARAMS: dict[str, dict] = {
    "wjazzd_swing": dict(fixed=0.8, extra=3, ghost_chance=0.35, prob_exp=1, hold=1, fill_scale=1),
    "wjazzd_ballad": dict(fixed=0.7, extra=4, ghost_chance=0.2, prob_exp=1, hold=6, fill_scale=2),
    "gmd_jazz_waltz": dict(fixed=0.9, extra=5, ghost_chance=0.3, prob_exp=0.75, hold=6, fill_scale=4),
    "gmd_latin_brazilian_bossa": dict(fixed=1, extra=0, ghost_chance=0.5, prob_exp=1, hold=16, fill_scale=1),
    "gmd_latin_brazilian_samba": dict(fixed=1, extra=0, ghost_chance=0.45, prob_exp=1, hold=1, fill_scale=1),
    "gmd_latin_brazilian_baiao": dict(fixed=0.5, extra=8, ghost_chance=0.45, prob_exp=2, hold=24, fill_scale=1),
    "gmd_afrocuban": dict(fixed=0.5, extra=5, ghost_chance=0.1, prob_exp=0.5, hold=16, fill_scale=0.25),
    "gmd_funk": dict(fixed=0.95, extra=0, ghost_chance=0.45, prob_exp=1, hold=1, fill_scale=2),
    "gmd_soul": dict(fixed=0.95, extra=0, ghost_chance=0.45, prob_exp=3, hold=16, fill_scale=8),
    "gmd_reggae": dict(fixed=1, extra=3, ghost_chance=0.05, prob_exp=1, hold=64, fill_scale=1),
    "gmd_rock": dict(fixed=1, extra=1, ghost_chance=0.05, prob_exp=2, hold=2, fill_scale=4),
    "gmd_pop": dict(fixed=0.8, extra=2, ghost_chance=0.2, prob_exp=1, hold=2, fill_scale=16),
    "gmd_blues_shuffle": dict(fixed=0.95, extra=3, ghost_chance=0.2, prob_exp=2, hold=4, fill_scale=1),
}


def update(pid: str, text: str, fixed: float, extra: int, prob_exp: float = 1.0,
           hold: float = 1.0, fill_scale: float = 1.0, ghost_chance: float = 0.45) -> str:
    """Transforme les patterns et les poids de phrases, sans changer leur séquence."""
    import tomllib
    data = tomllib.loads(text)
    voice = {v['name']: v for v in data.get('voices', [])}
    spbar = 12 if pid in ('gmd_jazz_waltz', 'gmd_blues_shuffle') else 16
    # Les deux premiers temps sont la charpente : ne pas retirer le temps 1 et le backbeat.
    protected = {0, 4, 8, 12} if spbar == 16 else {0, 3, 6, 9}
    # Notes de complément à faible vélocité : la caisse claire de jazz (22) est trop faible
    # pour entrer dans la mesure de signatures >= 40, `tap` est la frappe de balai à 52.
    ghost_voice = 'tap' if pid in JAZZ else next(
        (v['name'] for v in data.get('voices', []) if v['note'] in (37, 38, 40) and v['velocity'] >= 44),
        None,
    )
    if extra and ghost_voice is None:
        raise ValueError(f'{pid}: aucune caisse claire audible pour les coups fantômes')
    ghost_vel = voice[ghost_voice]['velocity'] if ghost_voice else 0
    ghost_delta = max(-24, min(0, 48 - ghost_vel))
    # 16 pas : contretemps 6 et 14 d'abord, puis 2 et 10, puis les doubles croches 3, 11, 7, 15 ; 12 pas : fins de triolets 5, 11, 2, 8 puis 1, 4, 7, 10.
    ghost_steps = [6, 14, 2, 10, 3, 11, 7, 15] if spbar == 16 else [5, 11, 2, 8, 1, 4, 7, 10]
    pattern = re.compile(r'(?m)^\[\[patterns\]\]\n.*?(?=^\[\[(?:patterns|phrases|fills)\]\]|\Z)', re.S | re.M)
    # Voix de la charpente du style, jamais rendues probabilistes : le contrat de leur profil l'exige (la grosse caisse du baião est la zabumba).
    hi = {name for name, v in voice.items() if v['note'] in (35, 36, 37, 38, 40) and name not in SKIP.get(pid, ())}
    def sub(m):
        block = m.group(0)
        present = set()
        # Seule la densité 1 (l'ossature) est retouchée : la densité 0 reste le timekeeper seul et la densité 2 garde ses notes
        # fantômes mesurées (contrats des tests de profil : les coups montent de densité en densité).
        density = int(re.search(r'(?m)^density = (\d)$', block)[1])
        def hit(mm):
            nonlocal present
            line = mm.group(0)
            step = int(mm[1]); vname = mm[2]
            if vname in hi:
                present.add((step, 'k' if voice[vname]['note'] in (35, 36) else 's'))
            if density != 1:
                return line
            if vname in hi and 'probability' in line and prob_exp != 1.0:
                line = re.sub(r'probability = ([0-9.]+)',
                              lambda p: f'probability = {max(0.001, min(1.0, float(p[1]) ** prob_exp)):.3f}', line)
            if fixed >= 1 or vname not in hi or step in protected or 'probability' in line:
                return line
            # Le geste du batteur reste dans le même pattern sur toute la phrase ;
            # seule sa frappe secondaire varie d'une mesure à l'autre.
            return line[:-2] + f', probability = {fixed:g} }}'
        block = re.sub(r'\{ step = (\d+), voice = "([^"]+)"[^}]*\}', hit, block)
        # La note fantôme ne s'ajoute qu'aux patterns qui jouent déjà cette voix : l'orchestration d'une étiquette (surdo seul de
        # l'exposition de la samba) reste celle de 019. Jazz : `tap` (note 38, frappe de balai ou caisse claire des baguettes à 52)
        # va dans tous les patterns de densité 1 et 2 ; le balayage `swirl` (40), lui, n'est jamais ajouté (le contrat interdit
        # le balayage aux baguettes).
        if extra and 'hits = [' in block and density in (1, 2) and (
                pid in JAZZ or f'voice = "{ghost_voice}"' in block):
            add = []
            for step in ghost_steps[:extra]:
                if (step, 's') not in present:
                    add.append(f'  {{ step = {step}, voice = "{ghost_voice}", delta_velocity = {ghost_delta}, probability = {ghost_chance:g} }},\n')
            block = block.replace('hits = [\n', 'hits = [\n' + ''.join(add), 1)
        return block
    text = pattern.sub(sub, text)
    if hold != 1.0:
        phrase = re.compile(r'(?m)^\[\[phrases\]\]\n.*?(?=^\[\[(?:phrases|fills)\]\]|\Z)', re.S | re.M)
        def weight(m):
            block = m.group(0)
            seq = re.search(r'^patterns = \[([0-9, ]+)\]', block, re.M)
            if seq and len(set(re.findall(r'\d+', seq[1]))) == 1:
                block = re.sub(r'(?m)^weight = (\d+)$',
                               lambda w: f'weight = {max(1, round(int(w[1]) * hold))}', block, count=1)
            return block
        text = phrase.sub(weight, text)
    if fill_scale != 1.0:
        text = re.sub(r'(?m)^fills_none_weight = (\d+)$',
                      lambda m: f'fills_none_weight = {max(1, round(int(m[1]) * fill_scale))}',
                      text, count=1)
    return text


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--profil', action='append')
    ap.add_argument('--out', type=Path, default=REPO)
    a = ap.parse_args()
    for pid in a.profil or E1:
        p = PARAMS[pid]
        path = a.out / 'profiles' / pid / 'percussion.toml'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(update(pid, git_show(f'profiles/{pid}/percussion.toml'), p['fixed'], p['extra'], p['prob_exp'],
                               p['hold'], p['fill_scale'], p['ghost_chance']))
        print('ok', pid, p)


if __name__ == '__main__':
    main()
