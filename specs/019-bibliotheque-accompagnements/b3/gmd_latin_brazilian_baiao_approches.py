#!/usr/bin/env python3
"""B3 : approches de basse avant un changement d'accord à la barre, profil contre WJazzD `LATIN` (hors ligne, en silence).

Commande (depuis `~/Project/bascanada/harmonium`, binaire de la branche) :

    python3 specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_baiao_approches.py \\
        --cli <worktree>/target/release/harmonium-cli --profile <worktree>/profiles/gmd_latin_brazilian_baiao \\
        [--seeds 16] [--bars 80] [--out approches.json]

`mesures_latin.py basse` compare la hauteur au temps 4 à celle de l'arrivée, sans savoir si la note est une approche : dans
un vamp I7 / bVII la fondamentale seule est à deux demi-tons de l'arrivée. Ici une **approche** est une note de basse de la
mesure d'avant le changement qui n'est ni la fondamentale ni la quinte de l'accord qui sonne (les notes des motifs) ;
elle est **chromatique** à un demi-ton de la note de basse suivante (la suivante de la mesure, ou l'arrivée), **de degré**
à deux demi-tons, sinon « autre ». Sorties sur les changements d'accord à la barre (mesure d'avant à un seul accord, arrivée
au pas 0) : part des changements précédés d'au moins une approche, et part chromatique / de degré des notes d'approche.
Référence WJazzD `LATIN` (`measures-bass.json`) : « approche du 4ᵉ temps avant un changement à la barre », 1205 événements,
22 solos : aucune 60 %, chromatique 22 %, diatonique 19 %, soit 40 % d'événements avec approche. Les unités diffèrent
(WJazzD : la note du 4ᵉ temps ; ici : toute note de passage de la mesure), la comparaison est d'ordre de grandeur.
Les mesures de break et de stop-time (deux dernières de chaque chorus après le premier) sont exclues.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import mesures_latin as M  # noqa: E402

BASS = 2


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--cli", type=Path, required=True)
    ap.add_argument("--profile", type=Path, required=True)
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--bars", type=int, default=80)
    ap.add_argument("--work", type=Path, default=Path("/tmp/harmonium-019/b3/baiao/truth"))
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    count: Counter = Counter()
    for seed in range(1, args.seeds + 1):
        ms = [m for m in M.measures(M.run_truth(args.cli, args.profile, seed, args.bars, args.work))
              if m["position"].get("section") == "Tour"]
        for prev, cur in zip(ms, ms[1:]):
            # Mesure d'avant à un seul accord, changement à la barre ; hors break et stop-time (deux dernières mesures d'un
            # chorus après le premier, où la basse ne garde que le temps 1).
            def coupe(m: dict) -> bool:
                return (m["index"] - 1) % 16 >= 14 and m["index"] > 16

            if not prev.get("chord_changes") or not cur.get("chord_changes"):
                continue
            if len(prev["chord_changes"]) != 1 or cur["chord_changes"][0]["start_step"] != 0 or coupe(prev) or coupe(cur):
                continue
            before, after = prev["chord_changes"][-1]["context"], cur["chord_changes"][0]["context"]
            if (before["root_pc"], before["bass_pc"]) == (after["root_pc"], after["bass_pc"]):
                continue
            notes = sorted((n["start_step"], n["pitch"]) for n in prev["notes"] if n["track"] == BASS)
            arrival = sorted((n["start_step"], n["pitch"]) for n in cur["notes"] if n["track"] == BASS)
            if not notes or not arrival or arrival[0][0] != 0:
                continue
            count["changements"] += 1
            passages = [(i, n) for i, n in enumerate(notes) if (n[1] - before["bass_pc"]) % 12 not in (0, 7)]
            count["avec_approche"] += bool(passages)
            for i, (_step, pitch) in passages:
                suivante = notes[i + 1][1] if i + 1 < len(notes) else arrival[0][1]
                distance = abs(pitch - suivante)
                count["chromatique" if distance == 1 else "degre" if distance == 2 else "autre"] += 1
                count["approches"] += 1
    n = count["changements"]
    ref = json.loads((Path.home() / "Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json").read_text())
    ind = ref["groups"]["all"]["indicators"]
    k = max(count["approches"], 1)
    result = {"changements_a_la_barre": n, "approches_notes": count["approches"],
              "changements_precedes_d_une_approche": round(count["avec_approche"] / n, 3),
              "approche_chromatique_part_des_notes": round(count["chromatique"] / k, 3),
              "approche_de_degre_part_des_notes": round(count["degre"] / k, 3),
              "approche_autre_part_des_notes": round(count["autre"] / k, 3),
              "WJazzD_LATIN_aucune_chromatique_diatonique_au_4e_temps": [
                  ind["approach_none_bar_change"]["estimate"], ind["approach_chromatic_bar_change"]["estimate"],
                  ind["approach_diatonic_bar_change"]["estimate"]]}
    print(json.dumps(result, indent=2, ensure_ascii=False))
    if args.out:
        args.out.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
