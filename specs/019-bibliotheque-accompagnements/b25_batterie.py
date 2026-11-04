#!/usr/bin/env python
"""B2 + B5 : `percussion.toml` des profils `wjazzd_ballad` (4/4) et `gmd_jazz_waltz` (3/4), construit
depuis le Groove MIDI Dataset v1.0.0 (CC BY 4.0, Gillick et al., ICML 2019).

Lance avec l'environnement du labo (lecture seule du labo : `harmonium_lab.backing.drums` et
`gmd_styles` lisent l'archive vérifiée par SHA256, `corpus/gmd/raw/`, jamais copiée) :

    cd ~/Project/bascanada/harmonium_lab
    .venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b25_batterie.py wjazzd_ballad \\
        > <moteur>/profiles/wjazzd_ballad/percussion.toml
    ... --report     # tableaux du README (stderr), même construction

Construction (aucune mesure d'un enregistrement n'est copiée : agrégats par groupe) :

1. **Mesures** : fichiers `beat` du sous-ensemble du profil (`SUBSETS`), ramenés à la grille swinguée de
   E7 (`drums.quantize` : pulsations de 4 pas, pas 2 retardé au rapport du fichier, mesuré sur ses
   contretemps de ride ; 16 pas en 4/4, 12 en 3/4). Vecteur de chaque mesure : grosse caisse, caisse
   claire, ride, pédale de charleston.
2. **Groupes** : k-moyennes (`random_state` fixe) ; un groupe est retenu s'il passe les seuils du profil
   (mesures, fichiers, batteurs, part maximale d'un fichier ; écrits dans le README du profil : la source est
   petite) et s'il est un jazz joué (règle `musical` : ride d'au moins trois coups).
3. **Patterns de ride** (étiquette `ride`) : par groupe retenu et par densité, un coup par (rôle, pas) dont
   la probabilité par pas dans le groupe atteint le seuil de la densité ; poids = effectif de mesures. Les
   vélocités gardent la FORME des accents du groupe (rapport à la médiane du ride), le niveau est celui de
   la voix (E7).
4. **Patterns de balais** (étiquette `brush`) et de **charleston** (`hat`) : RÈGLES DE MÉTIER, dites comme
   telles. Le GMD est joué aux baguettes sur un kit électronique : aucun balai n'y est mesuré. `brush` reprend
   la figure du ride du groupe (mêmes pas, mêmes accents) sur la frappe de balai (note 38 du kit 40,
   « Brushed Snares ») et ajoute le balayage (note 40) sur le un et le trois ; `hat` ne garde que le balayage
   et la pédale sur les contretemps (exposition, deux temps).
5. **Phrases** : par fichier, suite des groupes de ses mesures ; fenêtres de 2 et 4 mesures alignées sur
   la première mesure ; suite gardée si elle est vue au moins `MIN_PHRASE_COUNT` fois (batteurs : un
   seul dans la source 3/4 : dit) ; une suite qui change de groupe est aussi proposée en `part_end`.
   Les suites de `hat` sont écrites (groove tenu, règle de métier).
6. **Relances** : fichiers `fill` du style (`jazz` : 22 fichiers d'un batteur, 125 BPM) en 4/4 ; en 3/4
   aucun fichier : relances de métier (caisse claire sur le trois, grosse caisse sur le un), dites comme telles.
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

SEED = 0
TAGS = ("hat", "brush", "ride")

#: Sous-ensembles. `bar_beats` : pulsations par mesure ; `files` : prédicat sur (FileRow, rapport mesuré).
SUBSETS = {
    "wjazzd_ballad": {
        "bar_beats": 4,
        "styles": ("jazz", "jazz/swing"),
        "signature": "4-4",
        "max_bpm": 112,
        "min_ratio": 0.54,
        "fill_styles": ("jazz",),
        "k": 4,
        "min_bars": 10,
        "min_files": 2,
        "min_drummers": 1,
        "max_share": 0.9,
        "title": "wjazzd_ballad : GMD `jazz`, 4/4, 92 à 112 BPM, ride swingué (rapport mesuré ≥ 0,54)",
        "fills_none_ratio": 3.0,
    },
    "gmd_jazz_waltz": {
        "bar_beats": 3,
        "styles": ("jazz",),
        "signature": "3-4",
        "max_bpm": 999,
        "min_ratio": 0.54,
        "fill_styles": (),
        "k": 3,
        "min_bars": 12,
        "min_files": 1,
        "min_drummers": 1,
        "max_share": 0.85,
        "title": "gmd_jazz_waltz : GMD `jazz`, 3/4 (2 fichiers, un batteur, 120 et 200 BPM)",
        "fills_none_ratio": 1.5,
    },
}

#: Voix : (nom, note GM, piste). Les notes 38 et 40 sont « Brushed Snares » dans le kit 40 de
#: GeneralUser GS (vérifié dans le SF2 : touches 38, 39, 40) et « Jazz Snare 1 » / « Jazz Brushed Snare »
#: dans le kit 32.
VOICES = {
    "kick": ("kick", 36, "PercussionLow"),
    "hat_pedal": ("pedal", 44, "PercussionHigh"),
    "ride": ("ride", 51, "PercussionHigh"),
    "snare": ("snare", 38, "PercussionHigh"),
    "tom": ("tom", 43, "PercussionHigh"),
    "tap": ("tap", 38, "PercussionHigh"),
    "swirl": ("swirl", 40, "PercussionHigh"),
    "crash": ("crash", 49, "PercussionHigh"),
}
#: Gain par rôle sur la vélocité médiane du GMD (E7, `ROLE_GAIN`, kit jazz 32) ; les balais ne se
#: mesurent pas : niveaux de service.
ROLE_GAIN = {"kick": 0.5, "snare": 0.6, "tom": 0.6, "hat_pedal": 0.9, "ride": 1.0, "crash": 0.6}
#: Bornes de service par rôle (vélocité MIDI) : valeurs de métier, ni calibrées au LUFS ni écoutées. Le gain E7
#: donne une grosse caisse à 19 dans les fichiers lents (relevée à 30) ; la pédale est bornée à 58, niveau de E7.
ROLE_BOUNDS = {"kick": (30, 80), "snare": (24, 80), "tom": (36, 80), "hat_pedal": (36, 58), "ride": (30, 100), "crash": (40, 90)}


def level_of(role: str, raw: float) -> int:
    """Vélocité de rendu d'un coup du GMD : × gain du rôle (E7), dans les bornes de service."""
    lo, hi = ROLE_BOUNDS[role]
    return max(lo, min(hi, round(raw * ROLE_GAIN[role])))


#: Niveau de service des balais (vélocité de la voix) et accent relatif du balayage.
BRUSH_LEVEL = {"tap": 56, "swirl": 46}
#: Rôles joués par un pattern, hors timekeeper.
BACKBEAT = ("kick", "snare", "tom")
ROLES = ("kick", "snare", "tom", "ride", "hat_pedal")
FILL_ROLES = ("kick", "snare", "tom", "crash")
FILL_THRESHOLD = 0.3
MIN_PHRASE_COUNT = 2
MAX_VELOCITY = 120


def threshold(role: str, density: int) -> float:
    if density == 0:
        return 0.5 if role in ("ride", "hat_pedal") else 0.75
    if density == 1:
        return 0.5
    return 0.3 if role in BACKBEAT else 0.5


@dataclass
class Bar:
    file: str
    drummer: str
    steps: dict[str, dict[int, int]]  # rôle → pas → vélocité


@dataclass
class Group:
    ident: int
    bars: list[Bar] = field(default_factory=list)
    keep: bool = False

    @property
    def files(self) -> collections.Counter:
        return collections.Counter(b.file for b in self.bars)

    @property
    def drummers(self) -> set[str]:
        return {b.drummer for b in self.bars}


def read_bars(arc, row, bar_beats: int) -> tuple[list[Bar], float]:
    """Règles de mesures entières de E7, étendues ici au 3/4 (E7 lui-même ne lit que le 4/4)."""
    perf = g.read_performance(arc.read("groove/" + row.path))
    measured, _n = drums.swing_ratio(perf.hits)
    ratio = drums.effective_ratio(measured)
    by: dict[int, dict[str, dict[int, int]]] = {}
    first: dict[int, float] = {}
    for hit in perf.hits:
        beat, sub = drums.quantize(hit.beat, ratio)
        bar, in_bar = divmod(beat, bar_beats)
        step = in_bar * 4 + sub
        role = drums.role_of(hit.pitch)
        if role == "hat_closed":
            role = "ride"  # réalisation du charleston fermé sur la voix de ride
        slot = by.setdefault(bar, {}).setdefault(role, {})
        first.setdefault(bar, hit.beat)
        slot[step] = max(slot.get(step, 0), hit.velocity)
    out: list[Bar] = []
    first_bar = min(by) if by else None
    for bar in sorted(by):
        start = bar * bar_beats
        complete = start + bar_beats <= perf.end_beat + drums.BAR_TOLERANCE_BEATS
        if bar == first_bar and first[bar] - start > drums.FIRST_BAR_LATEST_ATTACK_BEATS:
            complete = False
        if complete:
            out.append(Bar(row.path, row.drummer, by[bar]))
    return out, ratio


def load(cfg):
    arc = drums.open_archive()
    rows = drums.parse_rows(drums.read_info_rows(arc))
    beats = []
    for row in rows:
        if row.beat_type != "beat" or row.style not in cfg["styles"] or row.time_signature != cfg["signature"]:
            continue
        if row.bpm > cfg["max_bpm"]:
            continue
        bars, ratio = read_bars(arc, row, cfg["bar_beats"])
        if ratio < cfg["min_ratio"] or len(bars) < 4:
            continue
        beats.append((row, bars, ratio))
    fills = []
    if cfg["fill_styles"]:
        res = g.analyse_archive(arc, [r for r in rows if r.style in cfg["fill_styles"] and r.beat_type == "fill"])
        g.settle(res, arc)
        fills = [r for r in res if not r.reject and r.measures and r.fill_position]
    return beats, fills


def steps_per_bar(cfg) -> int:
    return cfg["bar_beats"] * 4


def vec(bar: Bar, n: int) -> list[float]:
    out: list[float] = []
    for role in ("kick", "snare", "ride", "hat_pedal"):
        out += [1.0 if s in bar.steps.get(role, {}) else 0.0 for s in range(n)]
    return out


def stats(bars: list[Bar], role: str, n: int) -> tuple[list[float], list[float]]:
    prob, vel = [], []
    for s in range(n):
        present = [b.steps[role][s] for b in bars if s in b.steps.get(role, {})]
        prob.append(len(present) / len(bars) if bars else 0.0)
        vel.append(float(np.mean(present)) if present else 0.0)
    return prob, vel


def musical(bars: list[Bar], n: int) -> bool:
    """Un groupe est un jazz joué, pas un fragment : un ride d'au moins trois coups (probabilité ≥ 0,5)."""
    prob, _ = stats(bars, "ride", n)
    return sum(1 for p in prob if p >= 0.5) >= 3


def cluster(beats, cfg) -> list[Group]:
    n = steps_per_bar(cfg)
    items = [(row, b) for row, bars, _ in beats for b in bars]
    x = np.array([vec(b, n) for _, b in items])
    labels = KMeans(cfg["k"], n_init=10, random_state=SEED).fit(x).labels_
    groups = [Group(c) for c in range(cfg["k"])]
    for (_row, b), c in zip(items, labels):
        groups[c].bars.append(b)
    for grp in groups:
        files = grp.files
        grp.keep = (
            bool(grp.bars)
            and musical(grp.bars, n)
            and len(grp.bars) >= cfg["min_bars"]
            and len(files) >= cfg["min_files"]
            and len(grp.drummers) >= cfg["min_drummers"]
            and max(files.values()) / len(grp.bars) <= cfg["max_share"]
        )
    return groups


def role_medians(bars: list[Bar]) -> dict[str, float]:
    out = {}
    for role in ROLES + ("crash",):
        vs = [v for b in bars for v in b.steps.get(role, {}).values()]
        if vs:
            out[role] = float(np.median(vs))
    return out


def voice_levels(medians: dict[str, float]) -> dict[str, int]:
    """Vélocité de chaque voix : médiane du GMD × gain du rôle (E7) ; balais : niveau de service."""
    levels = {role: level_of(role, medians.get(role, 60.0)) for role in ROLE_GAIN}
    levels.update(BRUSH_LEVEL)
    return levels


def build_ride(groups: list[Group], n: int, levels: dict[str, int]) -> list[dict]:
    """Patterns `ride` par groupe retenu et par densité ; coups = (pas, voix, vélocité absolue)."""
    patterns: list[dict] = []
    for density in (0, 1, 2):
        for grp in groups:
            if not grp.keep:
                continue
            raw = []
            for role in ROLES:
                prob, vel = stats(grp.bars, role, n)
                for s in range(n):
                    if prob[s] > 0 and prob[s] >= threshold(role, density):
                        raw.append((s, role, vel[s]))
            ride_vels = [v for _s, r, v in raw if r == "ride"]
            if not ride_vels:
                continue
            scale = float(np.mean(ride_vels))
            hits = []
            for s, role, v in raw:
                hits.append((s, role, round(v / scale * levels["ride"]) if role == "ride" else level_of(role, v)))
            patterns.append({"density": density, "weight": len(grp.bars), "tag": "ride", "group": grp.ident,
                             "hits": hits})
    return patterns


def derive(patterns: list[dict], n: int, bar_beats: int, levels: dict[str, int]) -> list[dict]:
    """Patterns `brush` (même groupe, même densité) et `hat` (règle de métier, un par densité)."""
    out: list[dict] = []
    swirl_steps = [0] if bar_beats == 3 else [0, 8]
    off_pedal = [4, 8] if bar_beats == 3 else [4, 12]
    for p in patterns:
        hits = []
        for s, role, v in p["hits"]:
            if role == "ride":
                hits.append((s, "tap", round(v / levels["ride"] * levels["tap"])))
            elif role == "snare":
                hits.append((s, "tap", round(levels["tap"] * 0.8)))
            else:
                hits.append((s, role, v))
        hits += [(s, "swirl", levels["swirl"]) for s in swirl_steps]
        # Ride et caisse claire peuvent tomber ensemble : une seule frappe de balai par pas.
        merged = {}
        for step, role, level in hits:
            merged[(step, role)] = max(merged.get((step, role), 0), level)
        hits = [(step, role, level) for (step, role), level in merged.items()]
        out.append({"density": p["density"], "weight": p["weight"], "tag": "brush", "group": p["group"], "hits": hits})
    weight = max(p["weight"] for p in patterns)
    for density in (0, 1, 2):
        hits = [(s, "swirl", levels["swirl"]) for s in swirl_steps] + [(s, "hat_pedal", levels["hat_pedal"]) for s in off_pedal]
        if density == 2:
            hits.append((0, "kick", levels["kick"]))
        out.append({"density": density, "weight": weight, "tag": "hat", "group": -1, "hits": hits})
    return out


def phrases(beats, groups: list[Group], n: int) -> list[dict]:
    label = {id(b): grp.ident for grp in groups if grp.keep for b in grp.bars}
    counts: dict[tuple[int, ...], collections.Counter] = collections.defaultdict(collections.Counter)
    for row, bars, _ in beats:
        seq = [label.get(id(b), -1) for b in bars]
        for length in (2, 4):
            for start in range(0, len(seq) - length + 1, length):
                window = tuple(seq[start:start + length])
                if -1 in window:
                    continue
                counts[window][row.drummer] += 1
    out = []
    for window, drummers in counts.items():
        total = sum(drummers.values())
        if total < MIN_PHRASE_COUNT:
            continue
        out.append({"window": window, "weight": total, "drummers": len(drummers),
                    "turnaround": window[-1] != window[0]})
    out.sort(key=lambda p: (-p["weight"], p["window"]))
    return out[:16]


def fills_from_files(fill_files, cfg) -> list[dict]:
    """Relances 4/4 : fichiers `fill` du style groupés par longueur et départ."""
    buckets: dict[tuple[int, bool], list] = collections.defaultdict(list)
    for r in fill_files:
        n = 1 if len(r.measures) == 1 else 2
        pickup = r.fill_position["first_step"] >= 8 and r.fill_position["first_bar_index"] == 0
        buckets[(n, pickup)].append(r)
    out = []
    for key in sorted(buckets):
        members = buckets[key]
        if len(members) < 3:
            continue
        n = key[0]
        bars = []
        for i in range(n):
            ms = [r.measures[-n:][i] for r in members if len(r.measures) >= n]
            hits = []
            for role in ("kick", "snare", "tom_high", "tom_mid", "tom_low", "crash"):
                for s in range(16):
                    present = [m.steps[role][s][0] for m in ms if s in m.steps.get(role, {})]
                    if present and len(present) / len(ms) >= FILL_THRESHOLD:
                        hits.append((s, {"tom_high": "tom", "tom_mid": "tom", "tom_low": "tom"}.get(role, role),
                                     level_of({"tom_high": "tom", "tom_mid": "tom", "tom_low": "tom"}.get(role, role), float(np.mean(present)))))
            # un rôle de tom fusionné : un seul coup par (pas, voix)
            seen: dict[tuple[int, str], int] = {}
            for s, role, v in hits:
                seen[(s, role)] = max(seen.get((s, role), 0), v)
            bars.append(sorted(((s, r, v) for (s, r), v in seen.items()),
                               key=lambda h: (h[0], VOICES[h[1]][2] != "PercussionLow", h[1])))
        if not bars[0]:
            continue
        start = min(s for s, _, _ in bars[0])
        bars[0] = [h for h in bars[0] if h[0] >= start]
        out.append({"weight": len(members), "start_step": start, "bars": bars, "n_hits": sum(len(b) for b in bars),
                    "kind": f"{n} mesure(s), {'levée' if key[1] else 'mesure entière'}", "files": len(members)})
    out.sort(key=lambda f: (f["n_hits"], f["start_step"]))
    return out


def rule_fills(bar_beats: int) -> list[dict]:
    """Relances de métier pour la valse (aucun fichier `fill` en 3/4 dans le GMD)."""
    snare_three = [(8, "snare", 70), (10, "snare", 56)]
    return [
        {"weight": 3, "start_step": 8, "bars": [snare_three], "n_hits": 2, "kind": "règle : caisse claire sur le trois", "files": 0},
        {"weight": 2, "start_step": 6, "bars": [[(6, "snare", 56), (8, "snare", 70), (10, "tom", 64)]], "n_hits": 3,
         "kind": "règle : caisse claire et tom sur le deux et le trois", "files": 0},
        {"weight": 1, "start_step": 8, "bars": [snare_three, [(0, "kick", 60)]], "n_hits": 3,
         "kind": "règle : deux mesures, grosse caisse sur le un", "files": 0, "part_end": True},
    ]


# ── Émission TOML ───────────────────────────────────────────────────────────────────────────

def hit_line(step: int, role: str, level: int, voices: dict[str, int]) -> str:
    delta = max(-127, min(127, min(MAX_VELOCITY, level) - voices[role]))
    parts = [f"step = {step}", f'voice = "{VOICES[role][0]}"']
    if delta:
        parts.append(f"delta_velocity = {delta}")
    return "  { " + ", ".join(parts) + " },"


def brush_fill(role: str, level: int, levels: dict[str, int]) -> tuple[str, int]:
    """Relance aux balais : la caisse claire devient la frappe de balai."""
    return ("tap", round(levels["tap"] * 1.15)) if role == "snare" else (role, level)


def emit(name: str, cfg, patterns, phr, fl, levels, meta) -> str:
    n = steps_per_bar(cfg)
    used = {h[1] for p in patterns for h in p["hits"]}
    for tag_fills in fl:
        for bar in tag_fills["bars"]:
            used |= {h[1] for h in bar}
    used.add("tap")
    voices = {role: levels[role] for role in used}
    out = [
        f"# Batterie du profil `{name}` : GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b25_batterie.py {name}`",
        "# depuis le Groove MIDI Dataset v1.0.0 (CC BY 4.0, Gillick, Roberts, Engel, Eck, Bamman, ICML 2019 ;",
        f"# Google LLC), {cfg['title']} : {meta['files']} fichiers `beat`, {meta['bars']} mesures, {meta['drummers']} batteur(s).",
        f"# Mesures ramenées à la grille swinguée de E7 ({n} pas : pulsations de quatre pas, pas 2 retardé au rapport du fichier).",
        "# Étiquette `ride` : patterns MESURÉS. Chaque pattern = un groupe de mesures (k-moyennes sur grosse caisse, caisse",
        "# claire, ride, pédale) ; chaque coup est un (rôle, pas) dont la probabilité par pas dans le groupe atteint le seuil",
        "# de la densité (0 : ride et pédale ≥ 0,5, grosse caisse / caisse claire / tom ≥ 0,75 ; 1 : tous ≥ 0,5 ; 2 : grosse",
        "# caisse, caisse claire, tom ≥ 0,3). Poids = effectif de mesures. Vélocités : forme des accents du groupe,",
        "# niveau de la voix (gain par rôle de E7).",
        "# Étiquettes `brush` et `hat` : RÈGLES DE MÉTIER (le GMD est joué aux baguettes ; aucun balai n'y est mesuré) :",
        "# `brush` = la figure du ride du groupe jouée à la frappe de balai `tap` (note 38) plus `swirl` (note 40) au pas 0",
        "# (et 8 en 4/4). Ces deux touches sont « Brushed Snares » dans le kit 40, sans preuve de deux gestes acoustiques",
        "# distincts ; la note 40 du kit 32 est « Jazz Brushed Snare ». `hat` = `swirl` et pédale de charleston aux temps",
        "# 2 et 4 (3/4 : 2 et 3), plus grosse caisse en densité 2.",
        "# Phrases : suites de groupes vues dans le GMD (fenêtres de 2 et 4 mesures) ; relances : " + meta["fills_note"],
        "low_note = 36",
        "high_note = 51",
        "low_velocity = 64",
        "high_velocity = 64",
        "percussion_gain = 1.0",
        "velocity_humanize = 3",
        f"# Part de fins de phrase sans relance : {meta['none_pct']} % (valeur de service : le GMD isole les relances, la part de",
        "# fins de phrase sans relance n'y est pas mesurable).",
        f"fills_none_weight = {max(1, round(cfg['fills_none_ratio'] * meta['total_fill']))}",
        "",
    ]
    for role in sorted(used, key=lambda r: (VOICES[r][2] != "PercussionLow", VOICES[r][0])):
        nm, note, track = VOICES[role]
        out += ["[[voices]]", f'name = "{nm}"', f"note = {note}", f"velocity = {voices[role]}", f'track = "{track}"', ""]
    index: dict[tuple[str, int], dict[int, int]] = {}
    for i, p in enumerate(patterns):
        index.setdefault((p["tag"], p["density"]), {})[p["group"]] = i
        out += ["[[patterns]]", f"density = {p['density']}", f"weight = {p['weight']}", f'tag = "{p["tag"]}"', "hits = ["]
        for s, role, level in sorted(p["hits"], key=lambda h: (h[0], VOICES[h[1]][2] != "PercussionLow", h[1])):
            out.append(hit_line(s, role, level, voices))
        out += ["]", ""]
    for tag in TAGS:
        for density in (0, 1, 2):
            by_group = index.get((tag, density), {})
            if tag == "hat":
                idx = by_group.get(-1)
                if idx is None:
                    continue
                for window, weight, at in (([idx] * 4, 3, None), ([idx] * 2, 2, None), ([idx] * 2, 2, "part_end")):
                    out += ["[[phrases]]", f"density = {density}", 'tag = "hat"', f"weight = {weight}"]
                    if at:
                        out.append(f'at = "{at}"')
                    out += [f"patterns = {window}", ""]
                continue
            for ph in phr:
                try:
                    idx = [by_group[ident] for ident in ph["window"]]
                except KeyError:
                    continue
                for at in ["any"] + (["part_end"] if ph["turnaround"] else []):
                    out += ["[[phrases]]", f"density = {density}", f'tag = "{tag}"', f"weight = {ph['weight']}"]
                    if at != "any":
                        out.append(f'at = "{at}"')
                    out += [f"patterns = {idx}", ""]
    for tag in ("brush", "ride"):
        for density in (1, 2):
            for f in fl:
                out += ["[[fills]]", f"density = {density}", f'tag = "{tag}"', f"weight = {f['weight']}"]
                if f.get("part_end"):
                    out.append('at = "part_end"')
                out += [f"start_step = {f['start_step']}", "bars = ["]
                for bar in f["bars"]:
                    out.append("  { hits = [")
                    for s, role, level in bar:
                        if tag == "brush":
                            role, level = brush_fill(role, level, levels)
                        out.append("  " + hit_line(s, role, level, voices))
                    out.append("  ] },")
                out += ["]", ""]
    return "\n".join(out).rstrip("\n") + "\n"


def report(name, cfg, groups, patterns, phr, fl, beats) -> str:
    lines = [f"# {name}", "", "| Fichier | BPM | Batteur | Rapport mesuré | Mesures |", "|---|---:|---|---:|---:|"]
    for row, bars, ratio in beats:
        lines.append(f"| {row.path} | {row.bpm} | {row.drummer} | {ratio:.3f} | {len(bars)} |")
    lines += ["", "| Groupe | Mesures | Fichiers | Batteurs | Part max d'un fichier | Retenu | Ride (pas ≥ 0,5) |",
              "|---|---:|---:|---:|---:|---|---|"]
    n = steps_per_bar(cfg)
    for grp in groups:
        files = grp.files
        share = max(files.values()) / len(grp.bars) if grp.bars else 0
        prob, _ = stats(grp.bars, "ride", n)
        lines.append(f"| {grp.ident} | {len(grp.bars)} | {len(files)} | {len(grp.drummers)} | {share:.2f} | "
                     f"{'oui' if grp.keep else 'non'} | {[s for s, p in enumerate(prob) if p >= 0.5]} |")
    lines += ["", f"patterns : {len(patterns)} ; phrases : {len(phr)} suites ; relances : {len(fl)}", ""]
    for f in fl:
        lines.append(f"- relance {f['n_hits']} coups, {f['kind']}, départ pas {f['start_step']}, poids {f['weight']}")
    for p in phr:
        lines.append(f"- phrase {p['window']} poids {p['weight']} ({p['drummers']} batteurs)")
    return "\n".join(lines)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("profile", choices=sorted(SUBSETS))
    ap.add_argument("--report", action="store_true", help="écrit les tableaux du README sur stderr")
    args = ap.parse_args(argv)
    cfg = SUBSETS[args.profile]
    beats, fill_files = load(cfg)
    n = steps_per_bar(cfg)
    groups = cluster(beats, cfg)
    all_bars = [b for _, bars, _ in beats for b in bars]
    medians = role_medians(all_bars)
    levels = voice_levels(medians)
    ride = build_ride(groups, n, levels)
    patterns = ride + derive(ride, n, cfg["bar_beats"], levels)
    phr = phrases(beats, groups, n)
    if cfg["fill_styles"]:
        fl = fills_from_files(fill_files, cfg)
        fills_note = (f"{len(fill_files)} fichiers `fill` ({', '.join(cfg['fill_styles'])}) groupés par longueur et départ, "
                      "coups de probabilité ≥ 0,3.")
    else:
        fl = rule_fills(cfg["bar_beats"])
        fills_note = "aucun fichier `fill` en 3/4 dans le GMD : relances de métier (caisse claire sur le trois, grosse caisse sur le un)."
    total = sum(f["weight"] for f in fl)
    none_pct = round(100 * cfg["fills_none_ratio"] / (1 + cfg["fills_none_ratio"]))
    meta = {"files": len(beats), "bars": len(all_bars), "drummers": len({row.drummer for row, _, _ in beats}),
            "fills_note": fills_note, "none_pct": none_pct, "total_fill": total}
    sys.stdout.write(emit(args.profile, cfg, patterns, phr, fl, levels, meta))
    if args.report:
        sys.stderr.write(report(args.profile, cfg, groups, patterns, phr, fl, beats) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
