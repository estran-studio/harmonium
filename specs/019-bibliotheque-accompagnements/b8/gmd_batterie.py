"""B8 : batterie d'un profil à partir du Groove MIDI Dataset (CC BY 4.0), construite sur des agrégats.

Lancé avec le venv du labo (`harmonium_lab/.venv`) ; lit `corpus/gmd/raw/` (cache gitignoré, SHA256
vérifié par `drums.open_archive`) et n'écrit rien dans le labo. Aucune mesure d'un enregistrement
n'est copiée : chaque coup est un (rôle, pas) dont la probabilité par pas dans un groupe de mesures
atteint un seuil (même construction que `gmd_styles.proposed_patterns`, L2), chaque phrase une suite
de groupes vue dans plusieurs fichiers, chaque relance la moyenne seuillée d'un groupe de fichiers
`fill`.

Ce qui diffère de `gmd_styles.proposed_patterns` (L2), et pourquoi :
- les figures `other` et `sparse` du timekeeper sont gardées (hip-hop, rock, reggae : la majorité des
  mesures y tombent, L2 les écartait faute d'« ossature commune » : les seuils de probabilité par pas
  la retrouvent) ; seule la figure `none` (aucun timekeeper) est écartée ;
- les seuils d'éligibilité d'un groupe (fichiers, mesures, part maximale d'un fichier) sont ceux du
  profil, écrits dans son README (les styles de peu de fichiers ne tiennent pas ceux de L2) ;
- la charleston ouverte et le crash entrent aux densités 1 et 2 (disco, afrobeat) ;
- phrases de 2 et 4 mesures (suites de groupes) et relances groupées par longueur et position de
  départ : mesure demandée par A1 (`a1-phrases.md`, « Phrases du GMD »).
"""

from __future__ import annotations

import pickle
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

#: Rôle du GMD → (nom de la voix du moteur, piste).
VOICES: dict[str, tuple[str, str]] = {
    "kick": ("kick", "PercussionLow"),
    "snare": ("snare", "PercussionHigh"),
    "sidestick": ("sidestick", "PercussionHigh"),
    "hat_closed": ("hat", "PercussionHigh"),
    "hat_pedal": ("pedal", "PercussionHigh"),
    "hat_open": ("open", "PercussionHigh"),
    "ride": ("ride", "PercussionHigh"),
    "ride_bell": ("bell", "PercussionHigh"),
    "crash": ("crash", "PercussionHigh"),
    "tom_high": ("tom_high", "PercussionHigh"),
    "tom_mid": ("tom_mid", "PercussionHigh"),
    "tom_low": ("tom_low", "PercussionHigh"),
}
ROLE_OF_VOICE = {name: role for role, (name, _t) in VOICES.items()}
#: Étiquette d'arc du timekeeper d'un groupe.
TAG_OF_TIMEKEEPER = {"hat_closed": "hat", "ride": "ride", "ride_bell": "ride", "hat_open": "open"}
#: Seuils de probabilité par pas, par densité (L2 `LEVELS`, plus charleston ouverte et crash).
LEVELS: dict[int, dict[str, float]] = {
    0: {"timekeeper": 0.5},
    1: {"timekeeper": 0.5, "hat_pedal": 0.5, "kick": 0.5, "snare": 0.5, "sidestick": 0.5, "hat_open": 0.5},
    2: {"timekeeper": 0.5, "hat_pedal": 0.5, "kick": 0.25, "snare": 0.25, "sidestick": 0.25, "hat_open": 0.35, "crash": 0.5},
}
#: Niveau de rendu de chaque voix (vélocité MIDI sur le kit de GeneralUser GS), réglage de service : les
#: vélocités du GMD sont celles des pads d'un Roland TD-11 (grosse caisse médiane 29 à 78, caisse claire
#: 91 à 122) et ne se transposent pas ; seule la FORME des accents (vélocité relative du groupe) est gardée.
RENDER_LEVEL: dict[str, int] = {
    "kick": 100, "snare": 96, "sidestick": 88, "hat_closed": 70, "hat_pedal": 62, "hat_open": 76, "ride": 74,
    "ride_bell": 78, "crash": 96, "tom_high": 92, "tom_mid": 92, "tom_low": 92,
}
COHERENT_FIGURES = ("eighths", "sixteenths", "quarters", "quarters_plus", "spang")
FILL_ROLES = ("kick", "snare", "sidestick", "tom_high", "tom_mid", "tom_low", "crash")
STEPS = 16


@dataclass
class DrumSpec:
    """Ce que le profil demande à la batterie."""

    styles: list[str]
    #: Éligibilité d'un groupe (timekeeper, figure) : fichiers, mesures, part maximale d'un fichier.
    min_files: int = 3
    min_measures: int = 20
    max_share: float = 0.6
    max_groups: int = 7
    #: Fichiers `fill` d'un groupe (longueur, départ), et styles dont ils viennent.
    fill_styles: list[str] | None = None
    min_fill_files: int = 3
    #: Phrases : occurrences et fichiers minimaux d'une suite de groupes.
    min_phrase_count: int = 3
    min_phrase_files: int = 2
    #: Une suite qui change de groupe (une vraie phrase, pas un groove tenu) se garde dès ce nombre d'occurrences.
    min_change_count: int = 2
    #: Gain par rôle appliqué au niveau de rendu (`RENDER_LEVEL`), mesuré hors ligne sur le kit du profil.
    gain: dict[str, float] = field(default_factory=dict)
    #: Étiquettes que l'arc nomme : seules elles ont des relances (une copie par étiquette).
    arc_tags: list[str] = field(default_factory=list)
    #: Phrases gardées par densité et par longueur (les plus fréquentes).
    max_phrases: dict[int, int] = field(default_factory=lambda: {4: 8, 2: 4})
    #: Aucune relance : le style n'a aucun fichier `fill` dans le GMD.
    fills: bool = True
    #: Groupes (timekeeper, figure) écartés à la main, avec la raison (README).
    drop_groups: list[tuple[str, str]] = field(default_factory=list)


def load_results(lab: Path) -> list[Any]:
    """Analyse de tout le GMD (L2, `analyse_archive` + `settle`), en mémoire."""
    sys.path.insert(0, str(lab / "src"))
    from harmonium_lab.backing import drums, gmd_styles as G  # noqa: PLC0415

    cache = Path("/tmp/harmonium-019/b8/results.pkl")
    if cache.is_file():
        return pickle.loads(cache.read_bytes())
    archive = drums.open_archive()
    rows = drums.parse_rows(drums.read_info_rows(archive))
    results = G.analyse_archive(archive, rows)
    G.settle(results, archive)
    cache.parent.mkdir(parents=True, exist_ok=True)
    cache.write_bytes(pickle.dumps(results))
    return results


def _median(values: list[float]) -> float:
    ordered = sorted(values)
    mid = len(ordered) // 2
    return ordered[mid] if len(ordered) % 2 else (ordered[mid - 1] + ordered[mid]) / 2


@dataclass
class Group:
    key: tuple[str, str]
    measures: list[Any]
    files: int
    drummers: int
    index: int = -1

    @property
    def tag(self) -> str:
        return TAG_OF_TIMEKEEPER[self.key[0]]


def build(results: list[Any], spec: DrumSpec, lab: Path) -> dict[str, Any]:
    sys.path.insert(0, str(lab / "src"))
    from harmonium_lab.backing import gmd_styles as G  # noqa: PLC0415

    beats = [r for r in results if r.row.style in spec.styles and r.row.beat_type == "beat" and not r.reject
             and r.row.time_signature == "4-4" and r.feel == "straight" and r.measures]
    measures = [m for r in beats for m in r.measures]
    drummer_of = {r.row.path: r.row.drummer for r in beats}

    # Groupes. Deux familles, toutes deux cohérentes (un seul geste de timekeeper par groupe) :
    #  - `pas:<rôle>:<pas>` : mesures dont le timekeeper occupe exactement les mêmes pas ;
    #  - `<rôle>:<figure>` : figures nettes (croches, doubles croches, noires, noires plus) comme L2.
    # Les figures `sparse` et `other` (hétérogènes : la moyenne seuillée n'a pas d'ossature) et `none`
    # (aucun timekeeper) sont écartées.
    by_key: dict[tuple[str, str], list[Any]] = defaultdict(list)
    for m in measures:
        tk = G.timekeeper_of(m)
        if tk == "none":
            continue
        by_key[(tk, "pas:" + ",".join(map(str, m.role_steps(tk))))].append(m)
        figure = G.figure_of(m)
        if figure in COHERENT_FIGURES:
            by_key[(tk, figure)].append(m)
    candidates: list[Group] = []
    skipped: list[dict[str, Any]] = []
    for key, ms in sorted(by_key.items()):
        files = Counter(m.file for m in ms)
        info = {"group": f"{key[0]}:{key[1]}", "n_measures": len(ms), "n_files": len(files)}
        if key in spec.drop_groups:
            skipped.append({**info, "reason": "écarté (README)"})
        elif len(ms) < spec.min_measures or len(files) < spec.min_files or max(files.values()) / len(ms) > spec.max_share:
            continue  # trop de groupes exacts sous les seuils : seuls les groupes retenus sont comptés
        else:
            candidates.append(Group(key, ms, len(files), len({drummer_of[f] for f in files})))
    # Deux groupes de mêmes mesures (figure nette et pas exacts) n'en font qu'un : la figure le nomme.
    unique: dict[frozenset[int], Group] = {}
    for g in sorted(candidates, key=lambda g: (g.key[1].startswith("pas:"), g.key)):
        unique.setdefault(frozenset(id(m) for m in g.measures), g)
    candidates = sorted(unique.values(), key=lambda g: (-len(g.measures), g.key))
    groups = candidates[: spec.max_groups]
    for extra in candidates[spec.max_groups:]:
        skipped.append({"group": f"{extra.key[0]}:{extra.key[1]}", "n_measures": len(extra.measures),
                        "n_files": extra.files, "reason": f"au-delà des {spec.max_groups} groupes les plus fournis"})
    if not groups:
        raise SystemExit(f"aucun groupe éligible pour {spec.styles}")

    medians = G.role_velocity_medians(measures)
    gm_counts: dict[str, Counter] = defaultdict(Counter)
    for m in measures:
        for role, slots in m.steps.items():
            for _, note, _ in slots.values():
                gm_counts[role][note] += 1

    def fill_pool() -> list[Any]:
        styles = spec.fill_styles if spec.fill_styles is not None else spec.styles
        return [r for r in results if r.row.style in styles and r.row.beat_type == "fill" and not r.reject
                and r.row.time_signature == "4-4" and r.feel == "straight" and r.measures and r.fill_position]

    fills_src = fill_pool() if spec.fills else []
    for r in fills_src:
        for m in r.measures:
            for role, slots in m.steps.items():
                for _, note, _ in slots.values():
                    gm_counts[role][note] += 1
    fill_medians = G.role_velocity_medians([m for r in fills_src for m in r.measures]) if fills_src else {}

    # Voix : les rôles utilisés ; note GM la plus fréquente, vélocité médiane × gain.
    used: set[str] = set()

    def subset_hits(ms: list[Any], thresholds: dict[str, float], steps: int) -> list[dict[str, Any]]:
        hits = []
        total = len(ms)
        for role, thr in thresholds.items():
            for step in range(steps):
                present = [m.steps[role][step] for m in ms if step in m.steps.get(role, {})]
                if total and len(present) / total >= thr:
                    rel = sum(v / medians[role] for v, _, _ in present) / len(present) if medians.get(role) else 1.0
                    note = sorted(Counter(n for _, n, _ in present).items(), key=lambda kv: (-kv[1], kv[0]))[0][0]
                    hits.append({"step": step, "role": role, "note": note, "rel": rel})
        hits.sort(key=lambda h: (h["step"], VOICES[h["role"]][1] != "PercussionLow", h["role"]))
        return hits

    patterns: list[dict[str, Any]] = []
    index_of: dict[tuple[tuple[str, str], int], int] = {}
    for g in groups:
        for density, thr in LEVELS.items():
            thresholds = {(g.key[0] if k == "timekeeper" else k): v for k, v in thr.items()}
            hits = subset_hits(g.measures, thresholds, STEPS)
            if not hits:
                continue
            dup = next((i for i, p in enumerate(patterns) if p["density"] == density and p["tag"] == g.tag
                        and [(h["step"], h["role"], h["note"]) for h in p["hits"]]
                        == [(h["step"], h["role"], h["note"]) for h in hits]), None)
            if dup is not None:
                patterns[dup]["weight"] += len(g.measures)
                patterns[dup]["sources"].append(f"{g.key[0]}:{g.key[1]}")
                index_of[(g.key, density)] = dup
                continue
            index_of[(g.key, density)] = len(patterns)
            patterns.append({"density": density, "weight": len(g.measures), "tag": g.tag, "hits": hits,
                             "sources": [f"{g.key[0]}:{g.key[1]}"]})
            used.update(h["role"] for h in hits)
    for density in (0, 1, 2):
        if not any(p["density"] == density for p in patterns):
            raise SystemExit(f"{spec.styles} : aucun pattern de densité {density}")

    # Phrases : suites de groupes (2 et 4 mesures, alignées sur la première mesure du fichier). Une mesure
    # appartient à un ou deux groupes (son geste exact et sa figure) : le plus fourni la désigne.
    rank = {g.key: i for i, g in enumerate(groups)}
    member: dict[int, tuple[str, str]] = {}
    for g in groups:
        for m in g.measures:
            current = member.get(id(m))
            if current is None or rank[g.key] < rank[current]:
                member[id(m)] = g.key
    key_to_group = {g.key: g for g in groups}
    windows: dict[tuple[int, tuple[tuple[str, str], ...]], dict[str, Any]] = {}
    for r in beats:
        labels = [member.get(id(m)) for m in r.measures]
        for size in (2, 4):
            for start in range(0, len(labels) - size + 1, size):
                window = tuple(labels[start:start + size])
                if all(k is not None for k in window):
                    entry = windows.setdefault((size, window), {"n": 0, "files": set(), "drummers": set()})
                    entry["n"] += 1
                    entry["files"].add(r.row.path)
                    entry["drummers"].add(r.row.drummer)
    phrases: list[dict[str, Any]] = []
    kept_by_size: Counter = Counter()
    for (size, window), entry in sorted(windows.items(), key=lambda kv: (-kv[1]["n"], kv[0])):
        needed = spec.min_change_count if len(set(window)) > 1 else spec.min_phrase_count
        if entry["n"] < needed or len(entry["files"]) < spec.min_phrase_files:
            continue
        if kept_by_size[size] >= spec.max_phrases[size]:
            continue
        kept_by_size[size] += 1
        tags = Counter(key_to_group[k].tag for k in window)
        tag = max(tags, key=lambda t: (tags[t], t == key_to_group[window[0]].tag))
        for density in (0, 1, 2):
            idx = [index_of.get((k, density)) for k in window]
            if None in idx:
                continue
            phrases.append({"density": density, "weight": entry["n"], "tag": tag, "bars": size, "patterns": idx,
                            "n_files": len(entry["files"]), "n_drummers": len(entry["drummers"])})
    phrase_note = "mesurées : suites de groupes vues ≥ %d fois (≥ %d si la suite change de groupe) dans ≥ %d fichiers" % (
        spec.min_phrase_count, spec.min_change_count, spec.min_phrase_files)
    # Un groupe sans aucune suite mesurée joue tout de même : un batteur tient son groove quatre mesures (règle de
    # métier, dite dans le README) ; sans cela le groupe n'est jamais tiré dès qu'une phrase existe à sa densité.
    covered = {i for ph in phrases for i in ph["patterns"]}
    held = 0
    for g in groups:
        for density in (0, 1, 2):
            i = index_of.get((g.key, density))
            if i is not None and i not in covered:
                phrases.append({"density": density, "weight": len(g.measures), "tag": g.tag, "bars": 4,
                                "patterns": [i] * 4, "n_files": g.files, "n_drummers": g.drummers, "held": True})
                held += 1
    if held:
        phrase_note += " ; %d phrases de groove tenu 4 mesures ajoutées pour les groupes sans suite mesurée (règle de métier)" % held

    # Relances : groupes de fichiers `fill` par (mesures, pas de départ).
    fill_groups: dict[tuple[int, int], list[Any]] = defaultdict(list)
    for r in fills_src:
        n = len(r.measures)
        if n > 2:
            continue
        first = r.fill_position["first_step"]
        start = 0 if first < 3 else 4 if first < 6 else 8 if first < 10 else 12
        fill_groups[(n, start)].append(r)
    fills: list[dict[str, Any]] = []
    fills_skipped: list[dict[str, Any]] = []
    for (n_bars, start), rs in sorted(fill_groups.items()):
        if len(rs) < spec.min_fill_files:
            fills_skipped.append({"bars": n_bars, "start_step": start, "n_files": len(rs)})
            continue
        thr = max(0.25, min(0.5, 2 / len(rs)))
        bars = []
        for i in range(n_bars):
            ms = [r.measures[i] for r in rs if len(r.measures) == n_bars]
            hits = []
            for role in FILL_ROLES:
                for step in range(STEPS):
                    if i == 0 and step < start:
                        continue
                    present = [m.steps[role][step] for m in ms if step in m.steps.get(role, {})]
                    if len(present) / len(ms) >= thr:
                        rel = sum(v / fill_medians[role] for v, _, _ in present) / len(present)
                        note = sorted(Counter(n for _, n, _ in present).items(), key=lambda kv: (-kv[1], kv[0]))[0][0]
                        hits.append({"step": step, "role": role, "note": note, "rel": rel})
                        used.add(role)
            hits.sort(key=lambda h: (h["step"], VOICES[h["role"]][1] != "PercussionLow", h["role"]))
            bars.append(hits)
        if all(bars):
            fills.append({"weight": len(rs), "start_step": start, "n_bars": n_bars, "bars": bars,
                          "n_files": len(rs), "n_drummers": len({r.row.drummer for r in rs}), "threshold": round(thr, 2)})
        else:
            fills_skipped.append({"bars": n_bars, "start_step": start, "n_files": len(rs), "reason": "mesure vide"})

    voices = {}
    for role in sorted(used, key=lambda r: (VOICES[r][1] != "PercussionLow", VOICES[r][0])):
        name, track = VOICES[role]
        note = sorted(gm_counts[role].items(), key=lambda kv: (-kv[1], kv[0]))[0][0]
        voices[role] = {"name": name, "note": note, "track": track,
                        "velocity": max(1, min(127, round(RENDER_LEVEL[role] * spec.gain.get(role, 1.0))))}

    stats = {
        "n_files": len(beats), "n_measures": len(measures), "n_drummers": len({r.row.drummer for r in beats}),
        "bpm": sorted(r.row.bpm for r in beats),
        "n_fill_files": len(fills_src),
        "groups": [{"key": f"{g.key[0]}:{g.key[1]}", "tag": g.tag, "n_measures": len(g.measures),
                    "n_files": g.files, "n_drummers": g.drummers} for g in groups],
        "skipped": skipped, "fills_skipped": fills_skipped, "phrase_note": phrase_note,
    }
    return {"voices": voices, "patterns": patterns, "phrases": phrases, "fills": fills, "stats": stats,
            "tags": sorted({g.tag for g in groups}),
            "fill_tags": [t for t in spec.arc_tags if t in {g.tag for g in groups}] or sorted({g.tag for g in groups})}


def delta(voice: dict[str, Any], rel: float) -> int:
    """Écart de vélocité d'un coup à la vélocité de sa voix (vélocité relative du groupe)."""
    base = voice["velocity"]
    return max(1 - base, min(127 - base, round(base * (rel - 1.0))))


def percussion_toml(data: dict[str, Any], header: str, fills_none_weight: int, humanize: int = 3) -> str:
    voices = data["voices"]

    def hit(h: dict[str, Any]) -> str:
        voice = voices[h["role"]]
        parts = [f"step = {h['step']}", f'voice = "{voice["name"]}"']
        if h["note"] != voice["note"]:
            parts.append(f"note = {h['note']}")
        d = delta(voice, h["rel"])
        if d:
            parts.append(f"delta_velocity = {d}")
        return "{ " + ", ".join(parts) + " }"

    out = [header.rstrip("\n"), "", "low_note = 36", "high_note = 38", "low_velocity = 64", "high_velocity = 64",
           "percussion_gain = 1.0", f"velocity_humanize = {humanize}", ""]
    if data["fills"]:
        out += [f"fills_none_weight = {fills_none_weight}", ""]
    for role, v in voices.items():
        out += ["[[voices]]", f'name = "{v["name"]}"', f"note = {v['note']}", f"velocity = {v['velocity']}",
                f'track = "{v["track"]}"', ""]
    for i, p in enumerate(data["patterns"]):
        out += [f"# {i} : " + ", ".join(p["sources"]), "[[patterns]]", f"density = {p['density']}",
                f'tag = "{p["tag"]}"', f"weight = {p['weight']}", "hits = ["]
        out += ["  " + hit(h) + "," for h in p["hits"]]
        out += ["]", ""]
    for p in data["phrases"]:
        out += ["[[phrases]]", f"density = {p['density']}", f'tag = "{p["tag"]}"', f"weight = {p['weight']}",
                f"patterns = {p['patterns']}", ""]
    for density in (1, 2):
        for f in data["fills"]:
            for tag in data["fill_tags"]:
                at = "any" if f["n_bars"] == 1 else "part_end"
                out += ["[[fills]]", f"density = {density}", f'tag = "{tag}"', f"weight = {f['weight']}"]
                if at != "any":
                    out.append(f'at = "{at}"')
                if f["start_step"]:
                    out.append(f"start_step = {f['start_step']}")
                out.append("bars = [")
                for bar in f["bars"]:
                    out.append("  { hits = [")
                    out += ["    " + hit(h) + "," for h in bar]
                    out.append("  ] },")
                out += ["]", ""]
    return "\n".join(out).rstrip("\n") + "\n"
