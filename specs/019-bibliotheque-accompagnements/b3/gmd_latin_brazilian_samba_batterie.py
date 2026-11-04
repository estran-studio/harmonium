#!/usr/bin/env python3
"""B3 samba : `percussion.toml` de `gmd_latin_brazilian_samba` à partir du Groove MIDI Dataset (agrégats seulement).

Reproduire (venv du labo, lecture seule ; archive GMD vérifiée par SHA256) :

    ~/Project/bascanada/harmonium_lab/.venv/bin/python \
      specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_batterie.py \
      --out profiles/gmd_latin_brazilian_samba/percussion.toml \
      --stats /tmp/harmonium-019/b3/samba/mesures-generation.json

Ce qui est MESURÉ (GMD `latin/brazilian-samba`, 5 fichiers `beat` 4/4 droits d'un seul batteur, 322 mesures,
« juste ») : la grosse caisse (pas 0, 3, 4, 7, 8, 11, 12, 15), le tom grave (GM 43, pas 4 et 12), le charleston au
pied (pas 2, 6, 10, 14), la caisse claire (doubles croches continues, sauf aux pas 4 et 12) et le cross-stick ; leurs
vélocités relatives ; les deux gestes de caisse claire et leurs suites (phrases) ; les relances de `latin/samba`
`fill` (12 fichiers d'une mesure droits sur 17, « juste », autre label du même batteur).
Ce qui est RÈGLE DE MÉTIER (le GMD n'a aucune percussion latine : kit électronique Roland TD-11) : le surdo
étouffé au temps 1 et ouvert au temps 2, le tamborim (wood block GM 76 : le kit GM n'a pas de tamborim), l'agogô
(GM 67/68), le chocalho (shaker GM 82), le pandeiro (tambourin GM 54), l'orchestration par étiquette
(`surdo` -> `ensemble` -> `batucada`), les figures de tamborim.
Aucune mesure d'un enregistrement n'est copiée : un coup est un (rôle, pas) dont la probabilité par pas dans un
groupe de mesures atteint le seuil de la densité.
"""
from __future__ import annotations

import argparse
import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "b8"))
import gmd_batterie as B  # noqa: E402

STYLE = "latin/brazilian-samba"
FILL_STYLE = "latin/samba"
#: nom -> (note GM, vélocité de base, piste). Les vélocités sont des réglages de service (mixage mesuré, README).
VOIX = {
    "kick": (36, 64, "PercussionLow"),
    "surdo": (43, 78, "PercussionLow"),
    "caixa": (38, 62, "PercussionHigh"),
    "bois": (37, 66, "PercussionHigh"),
    "pedale": (44, 60, "PercussionHigh"),
    "tom_aigu": (50, 74, "PercussionHigh"),
    "tom_medium": (47, 74, "PercussionHigh"),
    "tamborim": (76, 66, "PercussionHigh"),
    "agogo_aigu": (67, 62, "PercussionHigh"),
    "agogo_grave": (68, 62, "PercussionHigh"),
    "shaker": (82, 42, "PercussionHigh"),
    "pandeiro": (54, 46, "PercussionHigh"),
}
#: Rôle du GMD -> voix du profil.
ROLE = {"kick": "kick", "snare": "caixa", "sidestick": "bois", "hat_pedal": "pedale", "tom_low": "surdo",
        "tom_high": "tom_aigu", "tom_mid": "tom_medium"}
TAGS = ("surdo", "ensemble", "batucada")
#: Seuils de probabilité par pas, par densité (famille de B4/B8 : ossature 0,75 ; 0,5 ; coups fantômes 0,25).
SEUIL = {
    "kick": [0.85, 0.5, 0.5], "hat_pedal": [0.5, 0.5, 0.5], "tom_low": [0.5, 0.5, 0.5],
    "snare": [0.75, 0.5, 0.25], "sidestick": [1.1, 1.1, 0.2],
}
#: Une probabilité par pas sous ce seuil reste un coup probabiliste ; au-dessus c'est l'ossature (jouée).
SOLIDE = 0.9
#: Figures de tamborim (règle de métier, une mesure = deux mesures de 2/4) : 3-3-2 puis 3-2-3 dans chaque 2/4.
TAMBORIM = ([0, 3, 6, 8, 11, 14], [0, 3, 5, 8, 11, 13])


def hit(step, voice, delta=0, probability=1.0):
    return {"step": step, "voice": voice, "delta_velocity": int(delta), "probability": float(probability)}


def aggregates(measures, role, threshold, med):
    """Coups d'un rôle : un par pas dont la probabilité par pas atteint le seuil, vélocité relative médiane."""
    out = []
    for step in range(16):
        present = [m.steps[role][step] for m in measures if step in m.steps.get(role, {})]
        p = len(present) / len(measures)
        if not present or p < threshold:
            continue
        rel = sorted(x[0] / med[role] for x in present)[len(present) // 2]
        voice = ROLE[role]
        base = VOIX[voice][1]
        delta = max(1 - base, min(127 - base, round(base * (rel - 1.0))))
        out.append(hit(step, voice, delta, 1.0 if p >= SOLIDE else round(p, 2)))
    return out


def shaker(density):
    """Chocalho (règle de métier) : croches à la densité 0, doubles croches ensuite, accents sur les temps."""
    steps = range(0, 16, 2) if density == 0 else range(16)
    return [hit(s, "shaker", {0: 6, 2: -2}.get(s % 4, -14)) for s in steps]


def orchestre(measures, med, density, tag, geste):
    """Une mesure de la tranche (densité, étiquette, geste) : liste de coups."""
    hits = [hit(4, "surdo"), hit(12, "surdo", -4)]  # surdo ouvert au temps 2 de chaque 2/4
    if tag == "surdo":
        # Surdo étouffé au temps 1 : grosse caisse douce (le kit GM n'a pas de surdo étouffé).
        hits += [hit(0, "kick", -26), hit(8, "kick", -30)]
        if density == 2:
            hits += [h for h in aggregates(measures, "kick", 0.5, med) if h["step"] in (3, 7, 11, 15)]
        return hits + shaker(density)
    hits += aggregates(measures, "kick", SEUIL["kick"][density], med)
    hits += aggregates(measures, "hat_pedal", SEUIL["hat_pedal"][density], med)
    hits += [hit(s, "tamborim", 0 if s % 8 == 0 else -6) for s in TAMBORIM[geste]]
    if density >= 1:
        hits += [hit(s, "agogo_aigu" if s in (0, 8) else "agogo_grave", -6) for s in (0, 6, 8, 14)]
    if density == 2:
        hits += [hit(s, "tamborim", -22, 0.5) for s in (2, 5, 10, 13)]
    hits += shaker(density)
    if tag == "batucada":
        hits += aggregates(measures, "snare", SEUIL["snare"][density], med)
        if density == 2:
            hits += aggregates(measures, "sidestick", SEUIL["sidestick"][density], med)
        if density >= 1:
            hits += [hit(s, "pandeiro", 4 if s % 8 == 2 else -8) for s in (2, 6, 10, 14)]
    return hits


def order(hits):
    # Un seul coup par (pas, voix) : le premier gagne (surdo ou kick posés avant l'agrégat).
    seen, out = set(), []
    for h in sorted(hits, key=lambda h: (h["step"], h["voice"])):
        if (h["step"], h["voice"]) not in seen:
            seen.add((h["step"], h["voice"]))
            out.append(h)
    return out


def val(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, str):
        return json.dumps(v, ensure_ascii=False)
    if isinstance(v, float):
        return repr(v)
    if isinstance(v, list):
        return "[" + ", ".join(val(x) for x in v) + "]"
    return str(v)


def hit_toml(h):
    parts = [f"step = {h['step']}", f'voice = "{h["voice"]}"']
    if h["delta_velocity"]:
        parts.append(f"delta_velocity = {h['delta_velocity']}")
    if h["probability"] < 1.0:
        parts.append(f"probability = {h['probability']}")
    return "{ " + ", ".join(parts) + " }"


def micro_timing(measures):
    """Médiane de l'écart à la grille (en pas) par rôle et place (sur le temps, hors temps) : GMD, un batteur."""
    out = {}
    for role in ("kick", "snare", "hat_pedal", "tom_low"):
        on, off = [], []
        for m in measures:
            for step, (_v, _n, offset) in m.steps.get(role, {}).items():
                (on if step % 4 == 0 else off).append(offset)
        out[role] = {"sur_le_temps": {"n": len(on), "mediane": round(statistics.median(on), 3) if on else None},
                     "hors_temps": {"n": len(off), "mediane": round(statistics.median(off), 3) if off else None}}
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--stats", type=Path, help="JSON des mesures lues (README du profil)")
    args = ap.parse_args()
    results = B.load_results(args.lab)
    beats = [r for r in results if r.row.style == STYLE and r.row.beat_type == "beat" and not r.reject
             and r.row.time_signature == "4-4" and r.feel == "straight"]
    ms = [m for r in beats for m in r.measures]
    med = __import__("harmonium_lab.backing.gmd_styles", fromlist=["role_velocity_medians"]).role_velocity_medians(ms)

    # Deux gestes de caisse claire : au moins douze doubles croches (jeu continu), ou un jeu plus aéré.
    def geste_of(m):
        return int(len(m.steps.get("snare", {})) >= 12)

    groups = defaultdict(list)
    for m in ms:
        groups[geste_of(m)].append(m)
    groups = {k: v for k, v in sorted(groups.items()) if len(v) >= 10 and len({m.file for m in v}) >= 3}
    if len(groups) != 2:
        raise SystemExit("Les deux gestes de caisse claire n'ont pas les effectifs requis")

    patterns, index = [], {}
    for density in range(3):
        for tag in TAGS:
            for geste, group in groups.items():
                index[density, tag, geste] = len(patterns)
                patterns.append({"density": density, "tag": tag, "weight": len(group),
                                 "hits": order(orchestre(group, med, density, tag, geste))})

    # Phrases : suites de gestes de 2 et 4 mesures vues >= 3 fois sur >= 2 fichiers (batucada, mesuré) ;
    # surdo et ensemble : mêmes suites (le même aller-retour de figures), règle de métier.
    windows = defaultdict(lambda: {"n": 0, "files": set()})
    for r in beats:
        labels = [geste_of(m) for m in r.measures]
        for size in (2, 4):
            for start in range(0, len(labels) - size + 1, size):
                seq = tuple(labels[start:start + size])
                windows[seq]["n"] += 1
                windows[seq]["files"].add(r.row.path)
    kept = [(seq, e) for seq, e in sorted(windows.items(), key=lambda kv: (-kv[1]["n"], kv[0]))
            if e["n"] >= 3 and len(e["files"]) >= 2 and len(set(seq)) > 1 or e["n"] >= 6 and len(e["files"]) >= 2][:10]
    phrases = []
    for density in range(3):
        for tag in TAGS:
            for seq, e in kept:
                phrases.append({"density": density, "tag": tag, "weight": e["n"],
                                "patterns": [index[density, tag, k] for k in seq]})

    # Relances : `latin/samba` `fill`, une mesure entière droite (12 fichiers sur 17 ; 4 fragments de moins de
    # trois temps et 1 fichier swing exclus).
    fs = [r for r in results if r.row.style == FILL_STYLE and r.row.beat_type == "fill" and not r.reject
          and r.row.time_signature == "4-4" and r.feel == "straight" and len(r.measures) == 1]
    fill_med = __import__("harmonium_lab.backing.gmd_styles", fromlist=["role_velocity_medians"]).role_velocity_medians(
        [r.measures[0] for r in fs])
    hits_by_file = sorted(sum(len(v) for v in r.measures[0].steps.values()) for r in fs)
    cut = 18  # coups par fichier : < 18 relance courte (9 fichiers), >= 18 relance chargée (3 fichiers)
    short = [r.measures[0] for r in fs if sum(len(v) for v in r.measures[0].steps.values()) < cut]
    loaded = [r.measures[0] for r in fs if sum(len(v) for v in r.measures[0].steps.values()) >= cut]
    fills = []

    def fill_hits(group, threshold):
        hs = []
        for role in ("snare", "kick", "tom_low", "tom_high", "tom_mid"):
            hs += aggregates(group, role, threshold, fill_med)
        return order(hs)

    for density, group, threshold in [(1, short, 0.5), (2, short, 0.3), (2, loaded, 0.5)]:
        fills.append({"density": density, "tag": "batucada", "weight": len(group), "start_step": 0,
                      "bars": [{"hits": fill_hits(group, threshold)}]})

    header = [
        "# GÉNÉRÉ : .venv/bin/python specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_samba_batterie.py "
        "--out profiles/gmd_latin_brazilian_samba/percussion.toml",
        "# Gillick, Roberts, Engel, Eck, Bamman, Learning to Groove with Inverse Sequence Transformations, ICML 2019 ;",
        "# Groove MIDI Dataset v1.0.0, Google LLC, CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/).",
        "# MESURÉ (GMD latin/brazilian-samba, 5 fichiers, 322 mesures, un batteur) : grosse caisse, tom grave (surdo), charleston",
        "# au pied, caisse claire, cross-stick ; deux gestes de caisse claire et leurs suites ; relances de latin/samba `fill`.",
        "# RÈGLE DE MÉTIER (le GMD n'a aucune percussion latine) : surdo étouffé/ouvert, tamborim (wood block GM 76, le kit GM",
        "# n'a pas de tamborim), agogô, chocalho, pandeiro, orchestration surdo -> ensemble -> batucada par étiquette d'arc.",
        "# Étiquettes que l'arc nomme : surdo (surdo, grosse caisse douce, chocalho), ensemble (+ tamborim, agogô, charleston",
        "# au pied, grosse caisse du kit), batucada (+ caisse claire, cross-stick, pandeiro, relances).",
        "# Un 4/4 de 16 pas = deux mesures de 2/4 de 8 pas : surdo ouvert aux pas 4 et 12, étouffé aux pas 0 et 8.",
    ]
    out = header + ["", "low_note = 36", "high_note = 38", "low_velocity = 64", "high_velocity = 64",
                    "percussion_gain = 1.0", "velocity_humanize = 3", "fills_none_weight = 10", ""]
    for name, (note, velocity, track) in VOIX.items():
        out += ["[[voices]]", f'name = "{name}"', f"note = {note}", f"velocity = {velocity}", f'track = "{track}"', ""]
    for i, p in enumerate(patterns):
        out += [f"# {i}", "[[patterns]]", f"density = {p['density']}", f'tag = "{p["tag"]}"', f"weight = {p['weight']}",
                "hits = ["] + ["  " + hit_toml(h) + "," for h in p["hits"]] + ["]", ""]
    for p in phrases:
        out += ["[[phrases]]", f"density = {p['density']}", f'tag = "{p["tag"]}"', f"weight = {p['weight']}",
                f"patterns = {p['patterns']}", ""]
    for f in fills:
        out += ["[[fills]]", f"density = {f['density']}", f'tag = "{f["tag"]}"', f"weight = {f['weight']}",
                f"start_step = {f['start_step']}", "bars = ["]
        for bar in f["bars"]:
            out += ["  { hits = ["] + ["    " + hit_toml(h) + "," for h in bar["hits"]] + ["  ] },"]
        out += ["]", ""]
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text("\n".join(out), encoding="utf8")

    # Mesures lues, pour le README (aucun enregistrement copié : comptes et probabilités par pas).
    per_step = {}
    for role in ("kick", "tom_low", "hat_pedal", "snare", "sidestick"):
        per_step[role] = [round(sum(1 for m in ms if s in m.steps.get(role, {})) / len(ms), 2) for s in range(16)]
    bpm = sorted(r.row.bpm for r in beats)
    stats = {
        "fichiers": len(beats), "mesures": len(ms), "tempos_fichiers": bpm,
        "tempos_p10_p50_p90": [round(statistics.quantiles(bpm, n=10, method="inclusive")[0], 1), statistics.median(bpm),
                               round(statistics.quantiles(bpm, n=10, method="inclusive")[-1], 1)],
        "par_pas": per_step,
        "velocite_mediane_gmd": {k: med[k] for k in ("kick", "tom_low", "hat_pedal", "snare", "sidestick")},
        "gestes": [{"geste": k, "mesures": len(v), "fichiers": len({m.file for m in v})} for k, v in groups.items()],
        "suites": [{"gestes": list(s), "occurrences": e["n"], "fichiers": len(e["files"])} for s, e in kept],
        "relances": {"fichiers_beat_fill_total": 17, "fichiers_une_mesure_droits": len(fs),
                     "coups_par_fichier": hits_by_file, "courtes": len(short), "chargees": len(loaded),
                     "tempo": sorted({r.row.bpm for r in fs})},
        "micro_timing_en_pas": micro_timing(ms),
        "patterns": len(patterns), "phrases": len(phrases), "relances_ecrites": len(fills),
    }
    if args.stats:
        args.stats.parent.mkdir(parents=True, exist_ok=True)
        args.stats.write_text(json.dumps(stats, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(stats, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
