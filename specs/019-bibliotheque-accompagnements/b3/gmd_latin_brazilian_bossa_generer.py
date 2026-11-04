#!/usr/bin/env python3
"""B3 bossa : régénère `percussion.toml` de `gmd_latin_brazilian_bossa` depuis le Groove MIDI Dataset.

Reproduire (labo en lecture seule, archive GMD vérifiée par SHA256 par `harmonium_lab`) :

    cd ~/Project/bascanada/harmonium_lab
    .venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/gmd_latin_brazilian_bossa_generer.py \
        --out <worktree>/profiles/gmd_latin_brazilian_bossa

Écrit `percussion.toml` et `<ce dossier>/gmd_latin_brazilian_bossa_mesures.json` (effectifs et micro-timing, repris par le README et
`[performance.tracks]` de `profile.toml`). Aucune mesure d'un enregistrement n'est copiée : agrégats par pas.

Source : GMD `latin/brazilian-bossa`, 2 fichiers `beat`, 1 batteur (drummer5), 631 mesures, 127 BPM ; « trop peu »
(L2). Rien d'autre n'existe : aucun `fill`, aucune percussion latine (kit Roland TD-11).

Ce qui est MESURÉ : groupes de mesures (timekeeper ride en croches, ride en noires plus, charleston fermé en
croches), probabilités par pas de la grosse caisse, de la pédale, de la caisse claire (ghost notes), du cross-stick
et du timekeeper dans chaque groupe, forme des accents (vélocité relative), micro-timing médian par rôle, suites de
groupes sur 2 et 4 mesures.
Ce qui est de MÉTIER (dit dans le fichier généré et le README) : niveaux de service des voix, phrases « groove tenu »
des groupes sans suite mesurée, relances (aucun fichier `fill` bossa dans le GMD), shaker GM à faible niveau.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "b8"))
import gmd_batterie as B  # noqa: E402

ID = "gmd_latin_brazilian_bossa"
STYLE = "latin/brazilian-bossa"
STEPS = 16

#: nom de voix → (note GM, vélocité de service, piste). Niveaux de service : la forme des accents est mesurée, l'amplitude est
#: celle du kit GM de GeneralUser GS (le TD-11 ne se transpose pas). Calés sur le LUFS intégré en solo avec le niveau de
#: `PercussionHigh` du profil (−9,6 dB, partagé avec la grosse caisse : canal 9) : grosse caisse −30,2, reste du kit −27,1 LUFS,
#: batterie −25,4 (`gmd_latin_brazilian_bossa_mix.py`) ; la grosse caisse est plus douce en vélocité que le reste du kit
#: parce que « Jazz Bass Drum » sonne plus fort que le ride et le charleston à vélocité égale.
VOIX = {
    "kick": (36, 47, "PercussionLow"),
    "sidestick": (37, 96, "PercussionHigh"),
    "snare": (38, 60, "PercussionHigh"),
    "hat": (42, 80, "PercussionHigh"),
    "pedal": (44, 78, "PercussionHigh"),
    "ride": (51, 86, "PercussionHigh"),
    "tom_mid": (47, 65, "PercussionHigh"),
    "shaker": (82, 39, "PercussionHigh"),
}
#: rôle du GMD → voix
ROLE = {"kick": "kick", "sidestick": "sidestick", "snare": "snare", "hat_closed": "hat", "hat_pedal": "pedal",
        "ride": "ride", "tom_mid": "tom_mid"}
#: groupes de mesures retenus : clé (timekeeper, figure) → (nom, étiquette d'arc)
GROUPES = {
    ("ride", "eighths"): ("ride_croches", "ride"),
    ("ride", "quarters_plus"): ("ride_noires", "ride"),
    ("hat_closed", "eighths"): ("charleston_croches", "hat"),
}


def objet(**valeurs) -> str:
    def val(v):
        if isinstance(v, bool):
            return str(v).lower()
        if isinstance(v, float):
            return repr(round(v, 4))
        if isinstance(v, str):
            return json.dumps(v, ensure_ascii=False)
        return str(v)

    return "{ " + ", ".join(f"{k} = {val(v)}" for k, v in valeurs.items()) + " }"


def groupes_de_mesures(beats, G):
    """Mesures par groupe. Un geste exact du ride (`pas:…`) qui coïncide avec « noires plus » rejoint ce groupe."""
    out: dict[str, list] = defaultdict(list)
    member: dict[int, str] = {}
    for r in beats:
        for m in r.measures:
            tk, fig = G.timekeeper_of(m), G.figure_of(m)
            key = (tk, fig)
            if key == ("ride", "other") and tuple(m.role_steps("ride")) == (0, 4, 6, 8, 10, 12, 14):
                key = ("ride", "quarters_plus")  # même geste que les « noires plus », classé « other » par L2
            if key in GROUPES:
                out[GROUPES[key][0]].append(m)
                member[id(m)] = GROUPES[key][0]
    return out, member


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--out", type=Path, required=True, help="dossier du profil (percussion.toml y est écrit)")
    ap.add_argument("--stats", type=Path, default=HERE / f"{ID}_mesures.json", help="effectifs et micro-timing mesurés")
    args = ap.parse_args()
    results = B.load_results(args.lab)
    sys.path.insert(0, str(args.lab / "src"))
    from harmonium_lab.backing import gmd_styles as G  # noqa: PLC0415

    beats = [r for r in results if r.row.style == STYLE and r.row.beat_type == "beat" and not r.reject
             and r.row.time_signature == "4-4" and r.feel == "straight" and r.measures]
    all_ms = [m for r in beats for m in r.measures]
    groups, member = groupes_de_mesures(beats, G)
    med = G.role_velocity_medians(all_ms)
    spec_info = {name: {"mesures": len(ms), "fichiers": len({m.file for m in ms}),
                        "batteurs": len({r.row.drummer for r in beats if r.row.path in {m.file for m in ms}})}
                 for name, ms in groups.items()}

    def stat(ms, role, step):
        present = [m.steps[role][step] for m in ms if step in m.steps.get(role, {})]
        if not present:
            return 0.0, 1.0
        rel = sorted(p[0] / med[role] for p in present)[len(present) // 2]
        return len(present) / len(ms), rel

    def hit(voice: str, step: int, rel: float = 1.0, probability: float = 1.0, facteur: float = 1.0) -> dict:
        velocity = VOIX[voice][1]
        # Les extrêmes isolés du TD-11 ne deviennent pas des accents de kit écrasants (0,55 à 1,4).
        delta = round(velocity * (min(1.4, max(0.55, rel)) * facteur - 1))
        delta = max(1 - velocity, min(127 - velocity, delta))
        h = {"step": step, "voice": voice}
        if delta:
            h["delta_velocity"] = delta
        if probability < 0.999:
            h["probability"] = round(probability, 3)
        return h

    def role_hits(ms, role, seuil, probabiliste=False, plafond=0.9, facteur=1.0):
        hits = []
        for step in range(STEPS):
            p, rel = stat(ms, role, step)
            if p >= seuil:
                hits.append(hit(ROLE[role], step, rel, min(plafond, p) if probabiliste else 1.0, facteur))
        return hits

    patterns: list[dict] = []
    index: dict[tuple[str, int], int] = {}
    for name, ms in sorted(groups.items()):
        tag = next(t for n, t in GROUPES.values() if n == name)
        tk = "hat_closed" if tag == "hat" else "ride"
        for density in (0, 1, 2):
            hits = role_hits(ms, tk, 0.5)
            # d0 : ossature (grosse caisse, pédale ≥ 0,75) ; d1 : ≥ 0,5 et cross-stick probabiliste ; d2 : ghost notes.
            hits += role_hits(ms, "kick", 0.75 if density == 0 else 0.5)
            hits += role_hits(ms, "hat_pedal", 0.75 if density == 0 else 0.5)
            if density >= 1:
                hits += [h for h in role_hits(ms, "kick", 0.2, True) if h["step"] % 2 == 1]
                hits += role_hits(ms, "sidestick", 0.15, True)
            if density == 2:
                hits += role_hits(ms, "snare", 0.2, True, 0.6)
                hits += role_hits(ms, "tom_mid", 0.1, True, 0.5)
                # Shaker GM : règle de métier (aucune percussion latine dans le GMD), très bas, croches.
                hits += [hit("shaker", s, 1.0, 1.0, 1.0 if s % 4 == 2 else 0.8) for s in range(0, STEPS, 2)]
            hits.sort(key=lambda h: (h["step"], VOIX[h["voice"]][2] != "PercussionLow", h["voice"]))
            # doublons (un pas porte une grosse caisse ≥ 0,5 et ghost) : le coup sûr gagne
            seen: dict[tuple[int, str], dict] = {}
            for h in hits:
                key = (h["step"], h["voice"])
                if key not in seen or "probability" not in h:
                    seen[key] = h
            hits = sorted(seen.values(), key=lambda h: (h["step"], VOIX[h["voice"]][2] != "PercussionLow", h["voice"]))
            index[name, density] = len(patterns)
            patterns.append({"density": density, "tag": tag, "weight": len(ms), "group": name, "hits": hits})

    # Phrases : suites de groupes sur 2 et 4 mesures, fenêtres alignées sur la première mesure du fichier.
    windows: dict[tuple[int, tuple], dict] = {}
    for r in beats:
        labels = [member.get(id(m)) for m in r.measures]
        for size in (2, 4):
            for start in range(0, len(labels) - size + 1, size):
                window = tuple(labels[start:start + size])
                if all(k is not None for k in window):
                    e = windows.setdefault((size, window), {"n": 0, "files": set()})
                    e["n"] += 1
                    e["files"].add(r.row.path)
    phrases: list[dict] = []
    mesurees = []
    for (size, window), e in sorted(windows.items(), key=lambda kv: (-kv[1]["n"], kv[0])):
        needed = 2 if len(set(window)) > 1 else 3
        if e["n"] < needed:
            continue
        tag = next(t for n, t in GROUPES.values() if n == window[0])
        for density in (0, 1, 2):
            phrases.append({"density": density, "tag": tag, "weight": e["n"], "patterns": [index[k, density] for k in window]})
        mesurees.append({"mesures": size, "suite": list(window), "occurrences": e["n"], "fichiers": len(e["files"])})
    covered = {i for p in phrases for i in p["patterns"]}
    tenues = 0
    # Groove tenu : un batteur garde son groupe 2 et 4 mesures (règle de métier, dite dans le fichier) pour les
    # groupes (et densités) que ni suite mesurée ni ancre n'atteignent.
    for name, ms in sorted(groups.items()):
        tag = next(t for n, t in GROUPES.values() if n == name)
        for density in (0, 1, 2):
            i = index[name, density]
            for size in (2, 4):
                if not any(p["density"] == density and p["patterns"] == [i] * size for p in phrases):
                    phrases.append({"density": density, "tag": tag, "weight": max(1, len(ms) // 4), "patterns": [i] * size})
                    tenues += 1

    # Relances : AUCUN fichier `fill` bossa dans le GMD : écrites à la main (métier), deux étiquettes (arc).
    relances = []
    for tag in ("ride", "hat"):
        relances += [
            {"density": 1, "tag": tag, "weight": 3, "start_step": 12,
             "bars": [[hit("snare", 12, 1.0, 1.0, 0.8), hit("snare", 13, 1.0, 1.0, 0.6), hit("tom_mid", 14), hit("tom_mid", 15, 1.0, 1.0, 0.9)]]},
            {"density": 1, "tag": tag, "weight": 2, "start_step": 8,
             "bars": [[hit("sidestick", 8), hit("sidestick", 10), hit("sidestick", 12), hit("kick", 14), hit("snare", 15, 1.0, 1.0, 0.7)]]},
            {"density": 2, "tag": tag, "weight": 3, "start_step": 12,
             "bars": [[hit("pedal", 12), hit("snare", 12, 1.0, 1.0, 1.0), hit("snare", 13, 1.0, 1.0, 0.7), hit("tom_mid", 14), hit("kick", 14), hit("tom_mid", 15)]]},
            {"density": 2, "tag": tag, "weight": 2, "start_step": 8,
             "bars": [[hit("sidestick", 8), hit("sidestick", 10), hit("sidestick", 12), hit("sidestick", 13), hit("kick", 14), hit("snare", 15)]]},
        ]

    def val(v):
        if isinstance(v, dict):
            return objet(**v)
        if isinstance(v, list):
            return "[" + ", ".join(val(x) for x in v) + "]"
        return json.dumps(v, ensure_ascii=False)

    out = [
        f"# Batterie du profil `{ID}` : GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b3/{ID}_generer.py` (labo en lecture seule) :",
        f"#   cd ~/Project/bascanada/harmonium_lab && .venv/bin/python <worktree>/specs/019-bibliotheque-accompagnements/b3/{ID}_generer.py --out <worktree>/profiles/{ID}",
        "# Source : Groove MIDI Dataset v1.0.0, style latin/brazilian-bossa : 2 fichiers beat, 1 batteur (drummer5), 631 mesures, 127 BPM,",
        "# fiabilité « trop peu » (L2). CC BY 4.0, Gillick, Roberts, Engel, Eck, Bamman, « Learning to Groove with Inverse Sequence",
        "# Transformations », ICML 2019 ; Google LLC. Agrégats par pas seulement : aucune mesure d'un enregistrement n'est copiée.",
        f"# MESURÉ : trois groupes de mesures (ride en croches {len(groups['ride_croches'])}, ride en noires plus {len(groups['ride_noires'])}, charleston fermé en croches {len(groups['charleston_croches'])}), probabilités par pas",
        "# de la grosse caisse [0, 6, 8, 14], de la pédale (temps 2 et 4), du timekeeper, du cross-stick et de la caisse claire (coups",
        "# probabilistes), forme des accents. Seuils : d0 timekeeper ≥ 0,5, grosse caisse et pédale ≥ 0,75 ; d1 tout ≥ 0,5, plus grosse",
        "# caisse fantôme et cross-stick ≥ 0,15 à leur probabilité mesurée ; d2 plus caisse claire ≥ 0,2 et tom médium ≥ 0,1 probabilistes.",
        "# DE MÉTIER (aucune mesure) : niveau de service de chaque voix (kit GM de GeneralUser GS, pas un TD-11 ; calé sur le LUFS en solo,",
        "# voir README), shaker GM (densité 2),",
        f"# {tenues} phrases « groove tenu » 2 et 4 mesures (aucune suite vue assez de fois), toutes les relances (le GMD n'a aucun",
        "# fichier fill bossa). Le GMD n'a aucune percussion latine : le kit seul est mesuré.",
        "low_note = 36", "high_note = 37", "low_velocity = 64", "high_velocity = 64",
        "percussion_gain = 1.0", "velocity_humanize = 2",
        "# Poids de « pas de relance » à une fin de phrase : valeur de service (aucun fichier fill dans la source).",
        "fills_none_weight = 8", "",
    ]
    for name, (note, velocity, track) in VOIX.items():
        out += ["[[voices]]", f'name = "{name}"', f"note = {note}", f"velocity = {velocity}", f'track = "{track}"', ""]
    for i, p in enumerate(patterns):
        out += [f"# {i} : groupe {p['group']} ({p['weight']} mesures), densité {p['density']}", "[[patterns]]",
                f"density = {p['density']}", f'tag = "{p["tag"]}"', f"weight = {p['weight']}", "hits = ["]
        out += ["  " + objet(**h) + "," for h in p["hits"]]
        out += ["]", ""]
    for p in phrases:
        out += ["[[phrases]]", f"density = {p['density']}", f'tag = "{p["tag"]}"', f"weight = {p['weight']}",
                f"patterns = {p['patterns']}", ""]
    out.append("# Relances de métier : le GMD n'a aucun fichier fill pour ce style.")
    for f in relances:
        out += ["[[fills]]", f"density = {f['density']}", f'tag = "{f["tag"]}"', f"weight = {f['weight']}",
                f"start_step = {f['start_step']}", "bars = [" + ", ".join("{ hits = " + val(b) + " }" for b in f["bars"]) + "]", ""]
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "percussion.toml").write_text("\n".join(out), encoding="utf8")

    # Micro-timing médian par rôle (écart à la grille, en pas), borne du schéma ±0,2 : un écart plus grand est omis.
    source = json.loads((args.lab / "references/gmd-latin-brazilian-bossa/measures-drums.json").read_text())
    brut = source["styles"][STYLE]["groups"]["4-4:straight"]["beat"]
    micro = {}
    for role, voice in ROLE.items():
        m = brut["micro_timing"].get(role)
        if not m:
            continue
        on = m.get("on_beat", {}).get("steps", {}).get("p50")
        off = m.get("off_beat", {}).get("steps", {}).get("p50", on)
        note, _v, track = VOIX[voice]
        entree = {"note": note, "piste": track, "on_beat": on, "off_beat": off,
                  "n_on": m.get("on_beat", {}).get("n"), "n_off": m.get("off_beat", {}).get("n")}
        micro[voice] = entree
    stats = {
        "fichiers": len(beats), "mesures": len(all_ms), "batteurs": len({r.row.drummer for r in beats}),
        "bpm": sorted(r.row.bpm for r in beats), "groupes": spec_info,
        "patterns": len(patterns), "phrases": len(phrases), "phrases_mesurees": mesurees, "phrases_groove_tenu": tenues,
        "relances": len(relances), "micro_timing_p50_pas": micro,
        "timekeeper": brut["timekeeper"]["roles"],
    }
    args.stats.write_text(json.dumps(stats, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(stats, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
