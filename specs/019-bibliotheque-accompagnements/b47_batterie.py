#!/usr/bin/env python
"""B4 + B7 : `percussion.toml` des profils `gmd_funk` et `gmd_neworleans_funk_secondline`,
construit depuis le Groove MIDI Dataset v1.0.0 (CC BY 4.0, Gillick et al., ICML 2019).

Lance avec l'environnement du labo (lecture seule du labo : `harmonium_lab.backing.drums` et
`gmd_styles` lisent l'archive vérifiée par SHA256, `corpus/gmd/raw/`, jamais copiée) :

    cd ~/Project/bascanada/harmonium_lab
    .venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b47_batterie.py gmd_funk \\
        > <moteur>/profiles/gmd_funk/percussion.toml
    ... --report     # tableaux du README (stderr), même construction

Construction (aucune mesure d'un enregistrement n'est copiée : agrégats par groupe) :

1. **Mesures** : celles de `gmd_styles.analyse_archive` (grille 16 pas droite, 4/4), styles `beat`
   du profil. Chaque mesure → vecteur (grosse caisse, caisse claire + cross-stick, charleston ou
   ride du timekeeper) de 48 bits.
2. **Groupes de mesures** : k-moyennes (`random_state` fixe, le rôle du timekeeper pèse comme quatre pas) ; un groupe est retenu s'il compte au
   moins 20 mesures, 3 fichiers, 2 batteurs, et aucun fichier au-delà de 60 % (règles d'éligibilité
   d'E7 et de L2, batteurs en plus). Les groupes écartés ne donnent aucun pattern.
   Un groupe est aussi écarté s'il n'est pas un groove (règle `musical` du profil : timekeeper et
   caisse claire au temps 2 ou 4 pour le funk ; grosse caisse et roulement ou timekeeper pour la
   second line).
3. **Patterns** : par groupe retenu et par densité, un coup par (rôle, pas) dont la probabilité par
   pas dans le groupe atteint le seuil de la densité ; vélocité = moyenne du (rôle, pas) × gain du
   rôle ; poids = effectif de mesures ; étiquette `sparse` / `medium` / `dense` = tiers du nombre de
   coups à la densité 1 (l'arc nomme ces étiquettes).
4. **Phrases** : par fichier, suite des groupes de ses mesures ; fenêtres de 2 et 4 mesures alignées
   sur la première mesure ; suite gardée si ≥ 3 occurrences sur ≥ 2 batteurs (A1) ; poids =
   effectif ; une suite dont la dernière mesure change de groupe est aussi proposée en `part_end`.
5. **Relances** : les fichiers `fill` du (des) style(s), groupés par longueur (1 ou 2 mesures) et
   par départ (mesure entière ou levée à partir du pas 8), scindés par k-moyennes quand le groupe
   compte ≥ 6 fichiers ; coup = (rôle, pas) de probabilité ≥ 0,3 dans le groupe ; `start_step` =
   plus petit pas porteur de la première mesure ; poids = effectif de fichiers.
"""

from __future__ import annotations

import argparse
import collections
import sys
from dataclasses import dataclass, field

import numpy as np
from sklearn.cluster import KMeans

from harmonium_lab.backing import drums
from harmonium_lab.backing import gmd_styles as g

# ── Profils ─────────────────────────────────────────────────────────────────────────────────

PROFILES = {
    "gmd_funk": {
        "beat": ["funk"],
        "fill": ["funk"],
        "k": 12,
        "rule": "groove",
        "title": "gmd_funk : GMD `funk` (36 fichiers beat, 4 batteurs ; 41 fills, 2 batteurs)",
        "kit": "kit standard de GeneralUser GS (banque 128, programme 0)",
    },
    "gmd_neworleans_funk_secondline": {
        "beat": ["neworleans/funk", "neworleans/secondline"],
        "fill": ["neworleans/funk"],
        "k": 8,
        "rule": "second_line",
        "title": (
            "gmd_neworleans_funk_secondline : GMD `neworleans/funk` (8 fichiers beat, 4 batteurs ; 40 fills) "
            "et `neworleans/secondline` (3 fichiers beat, 2 batteurs)"
        ),
        "kit": "kit standard de GeneralUser GS (banque 128, programme 0)",
    },
}

#: rôle GMD → (nom de voix, note GM du kit, piste). Les vélocités du GMD sont celles d'un Roland
#: TD-11 ; sur le kit GM de GeneralUser GS une même vélocité ne donne pas le même niveau selon le
#: rôle. Le niveau est calé par rôle (`gain`, plus bas), pas à l'oreille : LUFS intégré en solo
#: d'un rôle frappé 4 fois par mesure à 96 BPM, mesuré à vélocité 50 et 100 (`CALIBRATION`), droite
#: en log2 de la vélocité (≈ 10 à 16 LU par octave) ; la vélocité médiane du GMD du rôle est
#: ramenée au niveau visé (`TARGET`), la forme des accents (rapport à la médiane) est conservée.
VOICES = {
    "kick": ("kick", 36, "PercussionLow"),
    "snare": ("snare", 38, "PercussionHigh"),
    "sidestick": ("sidestick", 37, "PercussionHigh"),
    "hat_closed": ("hat", 42, "PercussionHigh"),
    "hat_pedal": ("pedal", 44, "PercussionHigh"),
    "hat_open": ("open", 46, "PercussionHigh"),
    "ride": ("ride", 51, "PercussionHigh"),
    "ride_bell": ("bell", 53, "PercussionHigh"),
    "crash": ("crash", 49, "PercussionHigh"),
    "tom_high": ("tom_high", 48, "PercussionHigh"),
    "tom_mid": ("tom_mid", 45, "PercussionHigh"),
    "tom_low": ("tom_low", 43, "PercussionHigh"),
}
#: LUFS intégré en solo (4 coups par mesure, 96 BPM, 16 mesures, kit 128/0 de GeneralUser GS,
#: mixage du profil `gmd_funk` : `/tmp/b47/calibrate.py`, `ffmpeg ebur128`) à vélocité 50 puis 100.
CALIBRATION = {
    "kick": (-32.8, -20.4), "snare": (-28.3, -16.2), "sidestick": (-33.6, -23.2),
    "hat_closed": (-44.0, -28.4), "hat_pedal": (-48.4, -32.5), "hat_open": (-33.2, -23.1),
    "ride": (-41.0, -30.9), "ride_bell": (-33.8, -23.7), "crash": (-31.5, -16.5),
    "tom_high": (-26.4, -16.3), "tom_mid": (-28.5, -18.4), "tom_low": (-29.8, -19.6),
}
#: Niveau visé en solo (mêmes conditions) : peaux et crash plus forts que les cymbales, charleston
#: et ride sous eux parce qu'ils jouent 8 à 16 coups par mesure.
TARGET = {
    "kick": -23.0, "snare": -23.0, "sidestick": -29.0, "hat_closed": -38.0, "hat_pedal": -41.0,
    "hat_open": -32.0, "ride": -37.0, "ride_bell": -32.0, "crash": -26.0,
    "tom_high": -27.0, "tom_mid": -27.0, "tom_low": -27.0,
}


def gain(role: str, median: float) -> float:
    """Facteur de vélocité qui ramène la vélocité médiane du rôle au niveau visé."""
    l50, l100 = CALIBRATION[role]
    per_octave = l100 - l50
    velocity = 50.0 * 2.0 ** ((TARGET[role] - l50) / per_octave)
    return velocity / median


ROLES = tuple(VOICES)
TIMEKEEPERS = ("hat_closed", "ride", "ride_bell", "hat_open")
BACKBEAT = ("kick", "snare", "sidestick")
FILL_ROLES = ("kick", "snare", "sidestick", "tom_high", "tom_mid", "tom_low", "crash", "hat_open")
TAGS = ("sparse", "medium", "dense")

# Seuils de probabilité par pas et par densité (README du profil, « mesure → valeur »).
#   densité 0 : ossature : timekeeper à 0,5 et temps forts de la grosse caisse et de la caisse claire à 0,75 ;
#   densité 1 : tous les rôles à 0,5 ;
#   densité 2 : grosse caisse, caisse claire, cross-stick à 0,3 (notes fantômes et syncopes), le reste à 0,5.
def threshold(role: str, density: int) -> float:
    if density == 0:
        return 0.5 if role in TIMEKEEPERS else 0.75
    if density == 1:
        return 0.5
    return 0.3 if role in BACKBEAT else 0.5


MIN_BARS, MIN_FILES, MIN_DRUMMERS, MAX_SHARE = 20, 3, 2, 0.6
MAX_VELOCITY = 120  # laisse de la marge aux multiplicateurs de dynamique (jusqu'à ×1,2)
FILL_THRESHOLD = 0.3
MIN_PHRASE_COUNT = 3
SEED = 0


@dataclass
class Group:
    ident: int
    bars: list = field(default_factory=list)  # (FileResult, indice de mesure, Measure)
    tag: str = ""
    index: dict[int, int] = field(default_factory=dict)  # densité → indice dans [[patterns]]
    musical: bool = True  # règle du profil (`musical`)

    @property
    def files(self) -> collections.Counter:
        return collections.Counter(r.row.path for r, _, _ in self.bars)

    @property
    def drummers(self) -> set[str]:
        return {r.row.drummer for r, _, _ in self.bars}

    def eligible(self) -> bool:
        files = self.files
        return (
            self.musical
            and len(self.bars) >= MIN_BARS
            and len(files) >= MIN_FILES
            and len(self.drummers) >= MIN_DRUMMERS
            and max(files.values()) / len(self.bars) <= MAX_SHARE
        )


def step_vector(measure, role: str) -> list[float]:
    return [1.0 if s in measure.steps.get(role, {}) else 0.0 for s in range(16)]


def features(measure) -> np.ndarray:
    tk = g.timekeeper_of(measure)
    snare = [max(a, b) for a, b in zip(step_vector(measure, "snare"), step_vector(measure, "sidestick"))]
    tkv = step_vector(measure, tk) if tk != "none" else [0.0] * 16
    # Le rôle du timekeeper pèse comme quatre pas : un groupe ne mélange pas charleston et ride.
    role = [4.0 if tk == name else 0.0 for name in TIMEKEEPERS]
    return np.array(step_vector(measure, "kick") + snare + tkv + role)


def stats(measures, role: str) -> tuple[list[float], list[float]]:
    """Probabilité par pas et vélocité moyenne par pas d'un rôle dans un groupe de mesures."""
    prob, vel = [], []
    for s in range(16):
        present = [m.steps[role][s][0] for m in measures if s in m.steps.get(role, {})]
        prob.append(len(present) / len(measures) if measures else 0.0)
        vel.append(float(np.mean(present)) if present else 0.0)
    return prob, vel


def role_medians(measures) -> dict[str, float]:
    out = {}
    for role in ROLES:
        vs = [v[0] for m in measures for v in m.steps.get(role, {}).values()]
        if vs:
            out[role] = float(np.median(vs))
    return out


def load(cfg):
    arc = drums.open_archive()
    rows = [r for r in drums.parse_rows(drums.read_info_rows(arc)) if r.style in set(cfg["beat"]) | set(cfg["fill"])]
    res = g.analyse_archive(arc, rows)
    g.settle(res, arc)
    return [r for r in res if not r.reject and (r.row.beat_type == "fill" or r.feel == "straight")]


def musical(measures, rule: str) -> bool:
    """Un groupe de mesures est un groove, pas un fragment : règle du profil, lue à la densité 1.

    - `groove` (funk) : un timekeeper d'au moins 4 coups et une caisse claire ou un cross-stick sur
      le temps 2 ou 4 (pas 4 ou 12) ;
    - `second_line` (New Orleans) : une grosse caisse d'au moins 2 coups, et soit la caisse claire
      (roulement : au moins 8 coups), soit un timekeeper d'au moins 4 coups.
    """
    n = {}
    prob = {}
    for role in ROLES:
        prob[role], _ = stats(measures, role)
        n[role] = sum(1 for p in prob[role] if p >= threshold(role, 1))
    keeper = max(n[r] for r in TIMEKEEPERS)
    if rule == "groove":
        backbeat = any(prob[r][s] >= 0.5 for r in ("snare", "sidestick") for s in (4, 12))
        return keeper >= 4 and backbeat
    return n["kick"] >= 2 and (max(n["snare"], n["sidestick"]) >= 8 or keeper >= 4)


def cluster(beats, k, rule) -> list[Group]:
    bars = [(r, i, m) for r in beats for i, m in enumerate(r.measures)]
    x = np.array([features(m) for _, _, m in bars])
    labels = KMeans(k, n_init=10, random_state=SEED).fit(x).labels_
    groups = [Group(c) for c in range(k)]
    for (r, i, m), c in zip(bars, labels):
        groups[c].bars.append((r, i, m))
    for grp in groups:
        grp.musical = bool(grp.bars) and musical([m for _, _, m in grp.bars], rule)
    return groups


def timekeeper_hits_density1(measures) -> int:
    n = 0
    for role in ROLES:
        prob, _ = stats(measures, role)
        n += sum(1 for p in prob if p >= threshold(role, 1))
    return n


def build_patterns(groups: list[Group], medians) -> list[dict]:
    """Patterns par groupe retenu et par densité, étiquetés par tiers d'activité."""
    kept = [grp for grp in groups if grp.eligible()]
    order = sorted(kept, key=lambda grp: (timekeeper_hits_density1([m for _, _, m in grp.bars]), grp.ident))
    for rank, grp in enumerate(order):
        grp.tag = TAGS[min(len(TAGS) - 1, rank * len(TAGS) // len(order))]
    patterns: list[dict] = []
    for density in (0, 1, 2):
        for grp in sorted(kept, key=lambda grp: grp.ident):
            measures = [m for _, _, m in grp.bars]
            hits = []
            for role in ROLES:
                prob, vel = stats(measures, role)
                for s in range(16):
                    if prob[s] > 0 and prob[s] >= threshold(role, density):
                        hits.append((s, role, vel[s]))
            if not hits:
                continue
            grp.index[density] = len(patterns)
            patterns.append({
                "density": density,
                "weight": len(grp.bars),
                "tag": grp.tag,
                "group": grp.ident,
                "files": len(grp.files),
                "hits": sorted(hits, key=lambda h: (h[0], VOICES[h[1]][2] != "PercussionLow", h[1])),
            })
    return patterns


def phrases(beats, groups: list[Group], patterns: list[dict]) -> list[dict]:
    label = {id(m): grp.ident for grp in groups for _, _, m in grp.bars if grp.eligible()}
    counts: dict[tuple[int, ...], collections.Counter] = collections.defaultdict(collections.Counter)
    for r in beats:
        seq = [label.get(id(m), -1) for m in r.measures]
        for length in (2, 4):
            for start in range(0, len(seq) - length + 1, length):
                window = tuple(seq[start:start + length])
                if -1 in window:
                    continue
                counts[window][r.row.drummer] += 1
    by_ident = {grp.ident: grp for grp in groups}
    out = []
    for window, drummers in counts.items():
        total = sum(drummers.values())
        if total < MIN_PHRASE_COUNT or len(drummers) < MIN_DRUMMERS:
            continue
        # Une suite d'un seul groupe répété compte pour la phrase « tenue » ; elle reste proposée.
        tags = collections.Counter(by_ident[c].tag for c in window)
        out.append({"window": window, "weight": total, "drummers": len(drummers),
                    "tag": tags.most_common(1)[0][0], "turnaround": window[-1] != window[0]})
    out.sort(key=lambda p: (-p["weight"], p["window"]))
    return out[:24]


def fills(fill_files) -> list[dict]:
    """Relances : fichiers `fill` groupés par (longueur, départ), scindés quand ils sont nombreux."""
    buckets: dict[tuple[int, bool], list] = collections.defaultdict(list)
    for r in fill_files:
        if not r.measures or not r.fill_position:
            continue
        n = 1 if len(r.measures) == 1 else 2
        pickup = r.fill_position["first_step"] >= 8 and r.fill_position["first_bar_index"] == 0
        buckets[(n, pickup)].append(r)
    groups: list[list] = []
    for key in sorted(buckets):
        members = buckets[key]
        n = key[0]
        if len(members) >= 6:
            def vec(r):
                bars = r.measures[-n:]
                out = []
                for m in bars:
                    for role in FILL_ROLES:
                        out += step_vector(m, role)
                return out
            labels = KMeans(2, n_init=10, random_state=SEED).fit([vec(r) for r in members]).labels_
            for c in (0, 1):
                sub = [r for r, lab in zip(members, labels) if lab == c]
                groups.append(sub)
        else:
            groups.append(members)
    out = []
    for members in groups:
        if len(members) < MIN_FILES:
            continue
        n = 1 if all(len(r.measures) == 1 for r in members) else 2
        bars = []
        for i in range(n):
            ms = [r.measures[-n:][i] for r in members if len(r.measures) >= n]
            med = role_medians(ms)
            hits = []
            for role in FILL_ROLES:
                if role not in med:
                    continue
                prob, vel = stats(ms, role)
                for s in range(16):
                    if prob[s] >= FILL_THRESHOLD:
                        hits.append((s, role, vel[s]))
            bars.append(sorted(hits, key=lambda h: (h[0], VOICES[h[1]][2] != "PercussionLow", h[1])))
        first = bars[0]
        if not first:
            continue
        start = min(s for s, _, _ in first)
        bars[0] = [h for h in first if h[0] >= start]
        out.append({"weight": len(members), "drummers": len({r.row.drummer for r in members}), "start_step": start,
                    "bars": bars, "n_hits": sum(len(b) for b in bars)})
    out.sort(key=lambda f: (f["n_hits"], f["start_step"]))
    for rank, fill in enumerate(out):
        fill["tag"] = TAGS[min(len(TAGS) - 1, rank * len(TAGS) // len(out))]
    return out


# ── Émission TOML ───────────────────────────────────────────────────────────────────────────

def voice_levels(medians, used: set[str]) -> dict[str, int]:
    return {role: max(1, min(127, round(medians[role] * gain(role, medians[role])))) for role in used}


def hit_line(step: int, role: str, velocity: float, levels: dict[str, int], gains: dict[str, float]) -> str:
    name = VOICES[role][0]
    delta = max(1, min(MAX_VELOCITY, round(velocity * gains[role]))) - levels[role]
    parts = [f"step = {step}", f'voice = "{name}"']
    if delta:
        parts.append(f"delta_velocity = {delta}")
    return "  { " + ", ".join(parts) + " },"


def emit(name: str, cfg, patterns, phr, fl, medians) -> str:
    used = {h[1] for p in patterns for h in p["hits"]} | {h[1] for f in fl for b in f["bars"] for h in b}
    levels = voice_levels(medians, used)
    gains = {role: gain(role, medians[role]) for role in used}
    out = [
        f"# Batterie du profil `{name}` : GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b47_batterie.py {name}`",
        "# depuis le Groove MIDI Dataset v1.0.0 (CC BY 4.0, Gillick, Roberts, Engel, Eck, Bamman, ICML 2019 ;",
        f"# Google LLC), {cfg['title']}.",
        "# Chaque pattern est construit sur les probabilités PAR PAS d'un groupe de mesures de même",
        "# configuration (grosse caisse, caisse claire, charleston ou ride) : k-moyennes, groupes d'au moins",
        "# 20 mesures, 3 fichiers et 2 batteurs ; aucun n'est une mesure d'un enregistrement.",
        "# Seuils : densité 0 timekeeper ≥ 0,5 et grosse caisse / caisse claire ≥ 0,75 ; densité 1 tous ≥ 0,5 ;",
        "# densité 2 grosse caisse, caisse claire, cross-stick ≥ 0,3 (notes fantômes), le reste ≥ 0,5.",
        "# Poids = effectif de mesures du groupe ; étiquette = tiers d'activité (sparse, medium, dense).",
        "# Les phrases sont les suites de groupes vues ≥ 3 fois sur ≥ 2 batteurs ; les relances sont les",
        "# fichiers `fill`, groupés par longueur et départ, coups de probabilité ≥ 0,3.",
        f"# Kit : {cfg['kit']}. Vélocités ramenées au kit par un gain par rôle, calé sur le LUFS en solo (voir le script).",
        "low_note = 36",
        "high_note = 42",
        "low_velocity = 64",
        "high_velocity = 64",
        "percussion_gain = 1.0",
        "velocity_humanize = 4",
        "# Poids de « pas de relance » dans le tirage d'une fin de phrase (valeur de service : le GMD isole",
        "# les relances, la part de fins de phrase sans relance n'y est pas mesurable).",
        "fills_none_weight = 3",
        "",
    ]
    for role in sorted(used, key=lambda r: (VOICES[r][2] != "PercussionLow", VOICES[r][0])):
        name_v, note, track = VOICES[role]
        out += ["[[voices]]", f'name = "{name_v}"', f"note = {note}", f"velocity = {levels[role]}",
                f'track = "{track}"', ""]
    for p in patterns:
        out += ["[[patterns]]", f"density = {p['density']}", f"weight = {p['weight']}", f'tag = "{p["tag"]}"',
                "hits = ["]
        out += [hit_line(s, role, v, levels, gains) for s, role, v in p["hits"]]
        out += ["]", ""]
    for density in (0, 1, 2):
        for ph in phr:
            idx = []
            ok = True
            for ident in ph["window"]:
                grp = next((x for x in ph["groups"] if x.ident == ident), None)
                if grp is None or density not in grp.index:
                    ok = False
                    break
                idx.append(grp.index[density])
            if not ok:
                continue
            for at in ph["places"]:
                out += ["[[phrases]]", f"density = {density}", f"weight = {ph['weight']}", f'tag = "{ph["tag"]}"']
                if at != "any":
                    out.append(f'at = "{at}"')
                out += [f"patterns = {idx}", ""]
    for density in (1, 2):
        for f in fl:
            out += ["[[fills]]", f"density = {density}", f"weight = {f['weight']}", f'tag = "{f["tag"]}"',
                    f"start_step = {f['start_step']}", "bars = ["]
            for bar in f["bars"]:
                out.append("  { hits = [")
                out += ["  " + hit_line(s, role, v, levels, gains) for s, role, v in bar]
                out.append("  ] },")
            out += ["]", ""]
    return "\n".join(out).rstrip("\n") + "\n"


def report(name, cfg, groups, patterns, phr, fl) -> str:
    lines = [f"# {name}", ""]
    lines.append("| Groupe | Mesures | Fichiers | Batteurs | Part max d'un fichier | Retenu | Étiquette |")
    lines.append("|---|---:|---:|---:|---:|---|---|")
    for grp in groups:
        files = grp.files
        share = max(files.values()) / len(grp.bars) if grp.bars else 0
        lines.append(f"| {grp.ident} | {len(grp.bars)} | {len(files)} | {len(grp.drummers)} | {share:.2f} | "
                     f"{'oui' if grp.eligible() else 'non'} | {grp.tag or '—'} |")
    lines += ["", f"patterns : {len(patterns)} ; phrases : {len(phr)} suites ; relances : {len(fl)}", ""]
    for f in fl:
        lines.append(f"- relance {f['n_hits']} coups, {len(f['bars'])} mesure(s), départ pas {f['start_step']}, "
                     f"{f['weight']} fichiers, {f['drummers']} batteurs, étiquette {f['tag']}")
    for p in phr:
        lines.append(f"- phrase {p['window']} poids {p['weight']} ({p['drummers']} batteurs) {p['tag']}")
    return "\n".join(lines)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("profile", choices=sorted(PROFILES))
    ap.add_argument("--report", action="store_true", help="écrit les tableaux du README sur stderr")
    args = ap.parse_args(argv)
    cfg = PROFILES[args.profile]
    res = load(cfg)
    beats = [r for r in res if r.row.beat_type == "beat" and r.row.style in cfg["beat"]]
    fill_files = [r for r in res if r.row.beat_type == "fill" and r.row.style in cfg["fill"]]
    groups = cluster(beats, cfg["k"], cfg["rule"])
    medians = role_medians([m for r in beats for m in r.measures])
    patterns = build_patterns(groups, medians)
    phr = phrases(beats, groups, patterns)
    kept = [grp for grp in groups if grp.eligible()]
    for ph in phr:
        ph["groups"] = kept
        ph["places"] = ["any"] + (["part_end"] if ph["turnaround"] else [])
    fl = fills(fill_files)
    sys.stdout.write(emit(args.profile, cfg, patterns, phr, fl, medians))
    if args.report:
        sys.stderr.write(report(args.profile, cfg, groups, patterns, phr, fl) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
