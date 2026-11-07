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

# Probabilités des coups secondaires et ajouts discrets ; valeurs de métier après mesure
# des signatures kick/snare (GMD : tempo, feel et batterie uniquement, CC BY 4.0).
# `fixed` : les coups déjà présents aux pas secondaires deviennent optionnels ; le premier
# temps et le backbeat du batteur ne sont pas touchés. `extra` : notes fantômes ajoutées à
# des places libres. Aucun morceau ni note MIDI d'un fichier du corpus n'est copié.
PARAMS = {
    "wjazzd_swing": (0.65, 4),
    "wjazzd_ballad": (0.7, 4),
    "gmd_jazz_waltz": (0.75, 3),
    "gmd_latin_brazilian_bossa": (1.0, 0),
    "gmd_latin_brazilian_samba": (0.80, 2),
    "gmd_latin_brazilian_baiao": (0.72, 3),
    "gmd_afrocuban": (0.90, 1),
    "gmd_funk": (1.0, 1),
    "gmd_soul": (1.0, 0),
    "gmd_reggae": (1.0, 0),
    "gmd_rock": (1.0, 0),
    "gmd_pop": (1.0, 0),
    "gmd_blues_shuffle": (1.0, 0),
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
    ghost_voice = 'tap' if pid == 'wjazzd_swing' else next(
        (v['name'] for v in data.get('voices', []) if v['note'] in (37, 38, 40) and v['velocity'] >= 44),
        None,
    )
    if extra and ghost_voice is None:
        raise ValueError(f'{pid}: aucune caisse claire audible pour les coups fantômes')
    ghost_vel = voice[ghost_voice]['velocity'] if ghost_voice else 0
    ghost_delta = max(-24, min(0, 48 - ghost_vel))
    # 2, 6, 10, 14 (16 pas) : syncopes entre temps ; 2, 5, 8, 11 (12 pas) : fin des triolets.
    ghost_steps = [2, 6, 10, 14] if spbar == 16 else [2, 5, 8, 11]
    pattern = re.compile(r'(?m)^\[\[patterns\]\]\n.*?(?=^\[\[(?:patterns|phrases|fills)\]\]|\Z)', re.S | re.M)
    hi = {name for name, v in voice.items() if v['note'] in (35, 36, 37, 38, 40)}
    def sub(m):
        block = m.group(0)
        present = set()
        def hit(mm):
            nonlocal present
            line = mm.group(0)
            step = int(mm[1]); vname = mm[2]
            if vname in hi:
                present.add((step, 'k' if voice[vname]['note'] in (35, 36) else 's'))
            if vname in hi and 'probability' in line and prob_exp != 1.0:
                line = re.sub(r'probability = ([0-9.]+)',
                              lambda p: f'probability = {max(0.001, min(1.0, float(p[1]) ** prob_exp)):.3f}', line)
            if fixed >= 1 or vname not in hi or step in protected or 'probability' in line:
                return line
            # Le geste du batteur reste dans le même pattern sur toute la phrase ;
            # seule sa frappe secondaire varie d'une mesure à l'autre.
            return line[:-2] + f', probability = {fixed:g} }}'
        block = re.sub(r'\{ step = (\d+), voice = "([^"]+)"[^}]*\}', hit, block)
        if extra and 'hits = [' in block:
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
    ap.add_argument('--fixed', type=float)
    ap.add_argument('--extra', type=int)
    ap.add_argument('--prob-exp', type=float)
    ap.add_argument('--hold', type=float)
    ap.add_argument('--fill-scale', type=float)
    ap.add_argument('--ghost-chance', type=float)
    a = ap.parse_args()
    for pid in a.profil or E1:
        fixed, extra = PARAMS[pid]
        if a.fixed is not None: fixed = a.fixed
        if a.extra is not None: extra = a.extra
        path = a.out / 'profiles' / pid / 'percussion.toml'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(update(pid, git_show(f'profiles/{pid}/percussion.toml'), fixed, extra,
                               a.prob_exp if a.prob_exp is not None else 1.0,
                               a.hold if a.hold is not None else 1.0,
                               a.fill_scale if a.fill_scale is not None else 1.0,
                               a.ghost_chance if a.ghost_chance is not None else 0.45))
        print('ok', pid, fixed, extra)

if __name__ == '__main__':
    main()
