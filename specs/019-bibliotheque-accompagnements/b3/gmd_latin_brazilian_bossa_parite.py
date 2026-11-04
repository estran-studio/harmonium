#!/usr/bin/env python3
"""B3 bossa : le GMD montre-t-il un cycle de deux mesures ? Probabilité par pas de la grosse caisse, de la pédale, du ride, du
cross-stick et de la caisse claire dans les mesures paires et impaires de chaque fichier `beat` de `latin/brazilian-bossa`.

    cd ~/Project/bascanada/harmonium_lab && .venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_parite.py

Lecture seule (cache `harmonium_lab` du GMD via `b8/gmd_batterie.py`). Écart maximal entre parités, par fichier et par rôle, sur les pas
à probabilité ≥ 0,5 dans l'une des deux parités : un écart faible dans les deux fichiers dirait un cycle commun ; ce n'est pas le cas.
"""
from __future__ import annotations

import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "b8"))
import gmd_batterie as B  # noqa: E402

lab = Path.home() / "Project/bascanada/harmonium_lab"
results = B.load_results(lab)
beats = [r for r in results if r.row.style == "latin/brazilian-bossa" and r.row.beat_type == "beat" and not r.reject
         and r.row.time_signature == "4-4" and r.feel == "straight" and r.measures]
ROLES = ("kick", "hat_pedal", "ride", "sidestick", "snare")
for n, r in enumerate(beats, 1):
    par = defaultdict(lambda: defaultdict(int))
    cnt = Counter()
    for i, m in enumerate(r.measures):
        cnt[i % 2] += 1
        for role in ROLES:
            for step in m.steps.get(role, {}):
                par[role, i % 2][step] += 1
    print(f"fichier {n} : {len(r.measures)} mesures ({cnt[0]} paires, {cnt[1]} impaires)")
    for role in ROLES:
        gaps = []
        for step in range(16):
            a, b = par[role, 0][step] / cnt[0], par[role, 1][step] / cnt[1]
            if max(a, b) >= 0.5:
                gaps.append((abs(a - b), step, a, b))
        worst = max(gaps) if gaps else None
        print(f"  {role:10} écart maximal entre mesures paires et impaires : " +
              (f"{worst[0]:.2f} (pas {worst[1]} : {worst[2]:.2f} contre {worst[3]:.2f})" if worst else "aucun pas ≥ 0,5"))
