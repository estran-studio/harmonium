#!/usr/bin/env python3
"""B3 : mesures hors ligne d'un profil latin (aucun son, aucune lecture audio).

Deux sous-commandes, sur le CLI `harmonium-cli` de la branche (à lancer depuis `~/Project/bascanada/harmonium`
pour `assets/`) :

    mesures_latin.py habillages --cli <cli> --profile <dossier> [--seeds 16] [--bars 48] [--out j.json]
    mesures_latin.py basse      --cli <cli> --profile <dossier> [--seeds 8] [--bars 48] \
        --reference ~/Project/bascanada/harmonium_lab/references/wjazzd-latin/measures-bass.json [--out j.json]

`habillages` : par graine, la vérité terrain (`--export --record-truth`) donne, dans `MeasureSnapshot.session`
(une entrée par groupe), `instruments`, `composition_bpm` et `position.key` / `position.mode`, l'habillage de
la session. Le tableau compte les habillages distincts, les tempos à ± 8 BPM (grappes : un tempo rejoint la
grappe dont le premier membre est à moins de 8 BPM), les tonalités, les triplets de programmes (comping,
basse, kit), le kit et la famille de voicings (nom de la variante du groupe `voicing` s'il existe), et les
changements de programme d'un chorus à l'autre (orchestration de l'arc).

`basse` : la basse jouée (piste 2) contre `measures-bass.json` de WJazzD `LATIN` (ODbL, Weimar Jazz
Database) sur ce que la source **mesure** : registre (hauteur de basse, centiles), arrivée sur la basse de
l'accord à chaque changement (hors mesure muette), approche (dernière note de la mesure d'avant le changement
à la barre : demi-ton, ton, autre), rythme harmonique (changement au temps 1 ou 3, accords par mesure), tempo.
La source ne mesure ni croche ni silence de basse : rien d'autre n'est comparé.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

BASS = 2


def run_truth(cli: Path, profile: Path, seed: int, bars: int, work: Path) -> dict:
    work.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    for path in sorted(profile.glob("*.toml")):
        digest.update(path.name.encode())
        digest.update(path.read_bytes())
    out = work / f"{profile.name}_{digest.hexdigest()[:12]}_s{seed}_b{bars}.json"
    if not out.is_file():
        cmd = [str(cli), "--export", "--bars", str(bars), "--profile", str(profile), "--seed", str(seed),
               "--record-truth", str(out)]
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return json.loads(out.read_text())


def measures(truth: dict) -> list[dict]:
    # Le compositeur précharge au-delà de l'audio ; la dernière barre peut être partielle.
    starts = truth["bar_start_frames"]
    return [m["measure"] for m in truth["measures"]
            if starts.get(str(m["bar"] + 1), truth["rendered_frames"] + 1) <= truth["rendered_frames"]]


def clusters(values: list[float], gap: float) -> list[list[float]]:
    groups: list[list[float]] = []
    for v in sorted(values):
        if groups and v - groups[-1][0] <= gap:
            groups[-1].append(v)
        else:
            groups.append([v])
    return groups


def habillages(args: argparse.Namespace) -> dict:
    rows = []
    for seed in range(1, args.seeds + 1):
        ms = measures(run_truth(args.cli, args.profile, seed, args.bars, args.work))
        first = next(m for m in ms if m.get("session") is not None)
        session = tuple((s["group"], s["name"]) for s in first["session"])
        programs: dict[int, dict] = {}
        for m in ms:
            if m.get("instruments"):
                programs[m["position"]["tour"]] = m["instruments"]
        base = first["instruments"]
        rows.append({
            "seed": seed,
            "session": session,
            "bpm": first["composition_bpm"],
            "key": first["position"]["key"],
            "mode": first["position"]["mode"],
            "programs": (base["chords_program"], base["bass_program"], base["percussion_bank"],
                         base["percussion_program"]),
            "chorus_changes": sum(1 for a, b in zip(list(programs.values()), list(programs.values())[1:]) if a != b),
            "swing": first["swing_ratio"],
        })
    tempo = clusters([r["bpm"] for r in rows], 8.0)
    signatures = []
    for r in rows:
        tempo_group = next(i for i, members in enumerate(tempo) if r["bpm"] in members)
        signatures.append((tempo_group, r["key"], r["mode"], r["programs"],
                           dict(r["session"]).get("voicing")))
    distinct = set(signatures)
    groups = sorted({g for r in rows for g, _ in r["session"]})
    per_group = {g: len({dict(r["session"]).get(g) for r in rows}) for g in groups}
    result = {
        "profile": args.profile.name,
        "seeds": args.seeds,
        "habillages_distincts": len(distinct),
        "tempos": [round(r["bpm"]) for r in rows],
        "grappes_tempo_8bpm": len(tempo),
        "tonalites": len({(r["key"], r["mode"]) for r in rows}),
        "programmes_comping_basse_kit": len({r["programs"] for r in rows}),
        "kits": sorted({(r["programs"][2], r["programs"][3]) for r in rows}),
        "variantes_distinctes_par_groupe": per_group,
        "swing": sorted({r["swing"] for r in rows}),
        "seeds_avec_orchestration_par_chorus": sum(1 for r in rows if r["chorus_changes"]),
        "lignes": [{**r, "session": list(map(list, r["session"]))} for r in rows],
    }
    print(f"{result['profile']} : {result['habillages_distincts']} habillages distincts / {args.seeds} graines ; "
          f"{result['grappes_tempo_8bpm']} grappes de tempo (± 8 BPM) ; {result['tonalites']} tonalités ; "
          f"{result['programmes_comping_basse_kit']} triplets de programmes ; kits {result['kits']}")
    print("variantes distinctes par groupe :", per_group)
    print("tempos :", result["tempos"], "; swing :", result["swing"])
    return result


def percentile(values: list[float], q: float) -> float:
    ordered = sorted(values)
    pos = (len(ordered) - 1) * q
    lo = int(pos)
    hi = min(lo + 1, len(ordered) - 1)
    return ordered[lo] + (ordered[hi] - ordered[lo]) * (pos - lo)


def bass_at(measure: dict, step: int) -> dict | None:
    sounding = [n for n in measure["notes"] if n["track"] == BASS
                and n["start_step"] <= step < n["start_step"] + n["duration_steps"]]
    return sounding[0] if len(sounding) == 1 else None


def basse(args: argparse.Namespace) -> dict:
    ref = json.loads(args.reference.read_text())["groups"]["all"]
    pitches: list[int] = []
    arrival = Counter()
    approach = Counter()
    change_beat = Counter()
    chords_per_bar = Counter()
    bpms = []
    n_bars = 0
    missing_arrivals = 0
    for seed in range(args.seed_start, args.seed_start + args.seeds):
        ms = measures(run_truth(args.cli, args.profile, seed, args.bars, args.work))
        tour = [m for m in ms if m["position"].get("section") == "Tour"]
        for i, m in enumerate(tour):
            n_bars += 1
            bpms.append(m["tempo"])
            for step in (0, 4, 8, 12):
                note = bass_at(m, step)
                if note is not None:
                    pitches.append(note["pitch"])
            prev = tour[i - 1] if i else None
            last = prev["chord_changes"][-1]["context"] if prev else None
            distinct = 1
            for c in m["chord_changes"]:
                ctx = c["context"]
                same = last is not None and (ctx["root_pc"], ctx["bass_pc"], ctx["quality"]) == (
                    last["root_pc"], last["bass_pc"], last["quality"])
                last = ctx
                if c["start_step"] > 0 and not same:
                    distinct += 1
                if (i == 0 and c["start_step"] == 0) or same:
                    continue
                change_beat[c["start_step"] // 4 + 1] += 1
                at = bass_at(m, c["start_step"])
                if at is None:
                    missing_arrivals += 1
                    continue
                arrival["bass" if at["pitch"] % 12 == ctx["bass_pc"] else "autre"] += 1
                if c["start_step"] == 0 and prev is not None:
                    before = bass_at(prev, 12)
                    if before is not None:
                        delta = abs(before["pitch"] - at["pitch"])
                        approach["demi_ton" if delta == 1 else "deux_demi_tons" if delta == 2 else "autre"] += 1
            chords_per_bar[min(distinct, 4)] += 1
    def share(counter: Counter, key: str) -> float | None:
        total = sum(counter.values())
        return round(counter[key] / total, 3) if total else None

    reg = ref["register"]["pitch"]
    ar = ref["arrival"]["change"]["shares"]
    ap = ref["indicators"]
    hr = ref["harmonic_rhythm"]
    result = {
        "profile": args.profile.name,
        "mesures_lues": n_bars,
        "temps_avec_basse_active": len(pitches),
        "changements_sans_basse_active_exclus": missing_arrivals,
        "arrivees_comparees": sum(arrival.values()),
        "approches_comparees": sum(approach.values()),
        "registre_basse_p1_p50_p99": [percentile(pitches, q) for q in (0.01, 0.5, 0.99)] if pitches else None,
        "registre_WJazzD_LATIN_p1_p50_p99": [reg["p1"], reg["p50"], reg["p99"]],
        "etendue_basse": [min(pitches), max(pitches)] if pitches else None,
        "etendue_WJazzD_LATIN": ref["register"]["pitch_range"],
        "arrivee_sur_la_basse_au_changement": share(arrival, "bass"),
        "arrivee_WJazzD_LATIN": ar["bass"],
        "approche_a_un_demi_ton_avant_changement_a_la_barre": share(approach, "demi_ton"),
        "approche_a_deux_demi_tons": share(approach, "deux_demi_tons"),
        "approche_autre": share(approach, "autre"),
        "limite_approche": "Distances de hauteur au temps 4 ; deux demi-tons n'implique pas un degré diatonique. Aucun score agrégé.",
        "approche_WJazzD_LATIN_chromatique_diatonique_aucune": [
            ap["approach_chromatic_bar_change"]["estimate"], ap["approach_diatonic_bar_change"]["estimate"],
            ap["approach_none_bar_change"]["estimate"]],
        "changement_au_temps_3": share(change_beat, 3),
        "changement_au_temps_3_WJazzD_LATIN": hr["change_beat"]["shares"]["3"],
        "mesures_a_un_accord": share(chords_per_bar, 1),
        "mesures_a_un_accord_WJazzD_LATIN": hr["chords_per_bar"]["shares"]["1"],
        "tempo_min_max": [min(bpms), max(bpms)] if bpms else None,
        "tempo_WJazzD_LATIN_p10_p50_p90": [ref["structure"]["tempo"].get(k) for k in ("p10", "p50", "p90")],
    }
    for k, v in result.items():
        print(f"{k} : {v}")
    return result


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", choices=["habillages", "basse"])
    ap.add_argument("--cli", type=Path, required=True)
    ap.add_argument("--profile", type=Path, required=True, help="dossier du profil")
    ap.add_argument("--seeds", type=int, default=16)
    ap.add_argument("--seed-start", type=int, default=1)
    ap.add_argument("--bars", type=int, default=48)
    ap.add_argument("--reference", type=Path)
    ap.add_argument("--work", type=Path, default=Path("/tmp/harmonium-019/b3/truth"))
    ap.add_argument("--out", type=Path)
    args = ap.parse_args()
    if args.cmd == "basse" and not args.reference:
        sys.exit("--reference requis (measures-bass.json de wjazzd-latin)")
    result = habillages(args) if args.cmd == "habillages" else basse(args)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
