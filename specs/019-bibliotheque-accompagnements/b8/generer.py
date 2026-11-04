#!/usr/bin/env python3
"""B8 : écrit `profiles/gmd_*` (profile, percussion, arc, dynamics, arrangement, set, cells, README).

Usage (venv du labo, qui lit le GMD en cache vérifié) :

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py \\
        [--lab ~/Project/bascanada/harmonium_lab] [--out profiles] [id ...]

Les fichiers écrits sont la source du profil distribué : ils se relisent et se corrigent à la main, mais
`percussion.toml` est GÉNÉRÉ (relancer ce script le réécrit).
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

sys.path.insert(0, str(HERE.parent / "r2-styles"))

import emit  # noqa: E402
import gmd_batterie as B  # noqa: E402
import habillage as H  # noqa: E402
from styles import PROFILES  # noqa: E402


def percentile(values: list[float], q: float) -> float:
    ordered = sorted(values)
    pos = (len(ordered) - 1) * q
    lo = int(pos)
    hi = min(lo + 1, len(ordered) - 1)
    return ordered[lo] + (ordered[hi] - ordered[lo]) * (pos - lo)


def header(p: dict, data: dict) -> str:
    s: emit.Style = p["style"]
    return f"""# Profil `{s.id}` (phase 019, B8) : {s.title}.
#
# Le nom est celui de la source qui définit le style (principe VII) : {p['source']}
# Ce profil est fait pour jouer sous une grille fournie par l'utilisateur : la basse (motifs), les accords
# (rythmes de comping, phrases) et la batterie (patterns, phrases, relances) ne dépendent que de l'accord
# publié et de la graine. Les deux parties par défaut (A et B de 8 mesures, quatre grilles chacune), quand
# l'utilisateur n'en fournit pas, sont des progressions d'accords de forme commune écrites pour ce profil
# (voir README, provenance et droits). La grille s'écrit relative à la tonique : la tonalité est tirée par la graine
# (`session.toml`, par défaut {p['key_name']}) et training la remplace par `[fixed_tonality]`.
#
# Ce qui vient de la source : la batterie, le tempo et le feel (`percussion.toml` est GÉNÉRÉ par
# `specs/019-bibliotheque-accompagnements/b8/generer.py`). Ce qui n'en vient pas, DIT ICI : la basse, le comping
# et les grilles sont des règles de métier (aucune mesure de basse ni de voicings pour ce style), le mixage et
# l'arc sont des réglages de service. L'habillage par graine (`session.toml`, A2 ; R2-styles) tire le tempo dans
# la plage de la source, la tonalité, les instruments, le kit, la famille de voicings, le registre, l'énergie et
# l'arc ; la variété vient aussi des boucles de grille, des phrases et relances, des motifs de basse et des
# patterns de batterie (coups probabilistes et micro-timing par voix mesurés dans le GMD).
#
# Aucune mélodie : la piste `Melody` est absente de l'arrangement."""


def drum_header(p: dict, data: dict) -> str:
    s: emit.Style = p["style"]
    st = data["stats"]
    lines = [
        f"# Batterie du profil `{s.id}` (phase 019, B8) : GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b8/generer.py`",
        "# (`gmd_batterie.py`) depuis le Groove MIDI Dataset v1.0.0 (Google LLC, CC BY 4.0 ; Gillick, Roberts, Engel, Eck,",
        f"# Bamman, ICML 2019) : styles {', '.join(p['drum'].styles)}, 4/4 droit, {st['n_files']} fichiers `beat`, {st['n_measures']} mesures,",
        f"# {st['n_drummers']} batteur(s) ; {st['n_fill_files']} fichiers `fill`. Aucune mesure d'un enregistrement n'est copiée.",
        "# Un pattern = un groupe de mesures de même geste de timekeeper (mêmes pas exactement, ou même figure nette) ;",
        "# chaque coup est un (rôle, pas) dont la probabilité par pas dans le groupe atteint le seuil de la densité",
        "# (0 : timekeeper ≥ 0,5 ; 1 : + charleston pédale et ouverte, grosse caisse, caisse claire, cross-stick ≥ 0,5 ;",
        "# 2 : + crash ≥ 0,5). Sous le seuil certain (0,5), un coup de grosse caisse, de caisse claire, de cross-stick (dès",
        "# 0,25 à la densité 1, dès 0,08 à 0,10 à la densité 2) et de charleston ouverte (dès 0,20, densité 2) joue avec sa",
        "# fréquence mesurée dans le groupe (`probability`, A6 : tirage sans état, stable par graine). Poids = effectif",
        "# de mesures du groupe. `delta_velocity` = accent relatif du groupe sur le niveau de rendu de la voix",
        "# (vélocités du TD-11 non transposées, réglage de service). Étiquettes : `hat` charleston fermée, `ride`,",
        "# `open` charleston ouverte (le timekeeper du groupe), nommées par `arc.toml`.",
        "# `[[phrases]]` : suites de groupes de 2 ou 4 mesures vues dans plusieurs fichiers du corpus (" + st["phrase_note"] + ").",
    ]
    if data["fills"]:
        lines.append("# `[[fills]]` : relances de 1 ou 2 mesures, groupées par longueur et pas de départ, moyenne seuillée des fichiers `fill`")
        lines.append("# (au moins 2 fichiers sur 3 d'accord dans un petit groupe, 0,25 au-delà) ; `fills_none_weight` : valeur de service.")
    else:
        lines.append("# Aucun fichier `fill` pour ce style dans le GMD : pas de relance (la variation vient des phrases et de l'arc).")
    return "\n".join(lines)


def session_header(p: dict, style: emit.Style, hab: H.Habillage) -> str:
    lo, default, hi = style.bpm
    return f"""# Habillage par graine de `{style.id}` (A2 ; phase 019, R2-styles) : un tirage par groupe, les groupes se combinent
# ({hab.possible()} habillages possibles). GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/b8/generer.py` depuis
# `specs/019-bibliotheque-accompagnements/r2-styles/habillage.py`.
#
# Ce qui vient de la source (GMD, CC BY 4.0) : les plages de tempo (centiles des BPM des fichiers `beat`, {lo:g} à {hi:g}) et
# le swing léger (rapport médian des fichiers `swing`). Le reste est de métier (tonalités courantes du style, instruments,
# kits, familles de voicings, registre, énergie, arcs nommés) : aucun corpus distribuable ne le mesure (décision F3).
# Programmes General MIDI (0-based) de GeneralUser GS, banque 0 ; kits de la banque 128 : 0 standard, 8 room, 16 power,
# 24 electronic, 25 808/909. Aucune zone Odin (D1). Le tempo de l'hôte (`SetBpm`), son swing et son `[fixed_tonality]`
# gagnent sur la variante."""


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def readme(p: dict, data: dict, style: emit.Style) -> str:
    st = data["stats"]
    r = p["readme"]
    lo, default, hi = (f"{x:g}" for x in style.bpm)
    proofs = proofs_section(style.id)
    groups = "\n".join(
        f"| `{g['key']}` | {g['tag']} | {g['n_measures']} | {g['n_files']} | {g['n_drummers']} |" for g in st["groups"])
    skipped_rows = "\n".join(f"| `{g['group']}` | {g['n_measures']} | {g['n_files']} | {g['reason']} |"
                              for g in st["skipped"][:12])
    skipped = (f"\nGroupes écartés (extraits) :\n\n| Groupe | Mesures | Fichiers | Raison |\n|---|---|---|---|\n{skipped_rows}\n"
               if skipped_rows else "")
    fills = "\n".join(f"| {f['n_bars']} mesure(s) | {f['start_step']} | {f['n_files']} | {f['n_drummers']} | {f['threshold']} |"
                      for f in data["fills"]) or "| — | | | | |"
    loops = "\n".join(
        f"| {part.name} ({part.bars} mesures) | {loop.weight} | {loop.comment} |" for part in style.parts for loop in part.loops)
    n_phr = len({tuple(ph["patterns"]) for ph in data["phrases"]})
    return f"""# `{style.id}` — {style.title}

{r['intro']}

**Nom (principe VII).** Le profil nomme la source qui définit son style : {p['source']} Ce n'est pas « le {r['genre']} » :
{r['nom_limite']}

## Mentions de licence

- **Groove MIDI Dataset v1.0.0** (Google LLC) — CC BY 4.0, <https://creativecommons.org/licenses/by/4.0/> : J. Gillick,
  A. Roberts, J. Engel, D. Eck, D. Bamman, « Learning to Groove with Inverse Sequence Transformations », ICML 2019.
  Les patterns, phrases et relances sont construits sur des probabilités par pas agrégées ; aucune mesure
  d'enregistrement. SHA256 de l'archive (vérifié par le labo avant toute lecture) :
  `651cbc524ffb891be1a3e46d89dc82a1cecb09a57c748c7b45b844c4841dcc1e`.
- Aucune autre source ; aucune source NC ni SA n'entre dans ce profil. Pas d'ODbL : WJazzD n'est pas lu.

Provenance chiffrée : `harmonium_lab/references/l2-ingestion.md` (§ 4.2), `references/gmd-*/`, `corpus/gmd/manifest.jsonl`.

## Grilles par défaut : provenance et droits

Quand l'utilisateur ne fournit pas de grille, le profil joue la partie A puis la partie B (8 mesures chacune ; une
boucle par partie est tirée par air selon la graine et les poids, puis tenue : un chorus fait 16 mesures). Les
grilles sont des **progressions d'accords de forme commune**, écrites relativement à la tonique (**{p['key_name']}** par défaut ; la variante
de session en tire une autre, voir « Habillage par graine »), pas copiées d'une partition ni d'une œuvre. Le Compendium du US Copyright Office (§802.5(A), éd. du
2021-01-28) range les « chord symbols based on standard chord progressions » parmi le matériau non protégeable
(*common property*) ; aucune œuvre n'est reproduite, aucune n'est nommée.

| Partie | Poids | Progression (mesure par mesure) |
|---|---|---|
{loops}

{r.get('grilles', '')}

## Écriture

4/4, 16 pas par mesure, une pulsation de 4 pas, feel droit (rapport de swing 0,5 : le GMD mesure {r['swing']}). Tempo
{default} noires/min par défaut, plage {lo} à {hi} : {style.bpm_note} Un chorus = 16 mesures ; {style.tours} chorus par air
(un air par set) : le nombre de chorus est connu, la retombée de l'arc joue au dernier.

| Couche | Réglage |
|---|---|
| Basse | {r['basse']} |
| Accords | {r['comping']} |
| Batterie | kit GeneralUser GS banque 128, programme {style.kit_program} ({r['kit']}) ; {len(data['patterns'])} patterns, {n_phr} phrases distinctes, {len(data['fills'])} relances ; densité 0 timekeeper, 1 ossature, 2 notes fantômes |
| Arc | exposition, corps, sommet, retombée : `arc.toml` (valeurs de service) |
| Timing | {r['timing']} |
| Mixage | master {style.mix['master']} dB, accords {style.mix['Chords']} dB, basse {style.mix['Bass']} dB, batterie {style.mix['Percussion']} dB : réglages de service calés hors ligne (section Mixage) |

Odin n'est pas utilisé : aucune zone `[[odin.<Piste>.zones]]`, les timbres sont ceux de GeneralUser GS (D1).

## Mesures → paramètres

| Paramètre | Valeur | Mesure | Source |
|---|---|---|---|
| `meter.default_bpm` / `min_bpm` / `max_bpm` | {default} / {lo} / {hi} | tempo des {st['n_files']} fichiers `beat` (4/4 droit) : médiane et centiles 10 et 90 ; BPM du nom de fichier | GMD |
| `meter.default_swing_ratio` | 0,5 | rapport médian des fichiers retenus {r['swing']} | GMD |
| Patterns de batterie | {len(data['patterns'])} | groupes ci-dessous, seuils d'éligibilité du profil, seuils de probabilité de L2 | GMD |
| Phrases de batterie | {len(data['phrases'])} entrées (3 densités) | {st['phrase_note']} | GMD |
| Relances | {len(data['fills'])} groupes × {len(data['fill_tags'])} étiquettes × 2 densités | fichiers `fill` ({st['n_fill_files']}) | GMD |
| Basse, comping, grilles | — | **aucune mesure** : règles de métier ({r['regles']}) | règles |
| Plages de tempo de la session, swing léger | voir « Habillage par graine » | centiles des BPM du GMD ; rapport médian des fichiers `swing` | GMD |
| Micro-timing par voix, coups probabilistes | voir « Habillage par graine » et `percussion.toml` | médianes en ms par rôle ; probabilité par pas du groupe | GMD |
| Tonalités, instruments, kits, voicings, registre, énergie, arcs nommés, mixage, vélocités | — | réglages de service (règles de métier) | — |

### Batterie : groupes retenus

Fichiers `beat` : {st['n_files']}, mesures {st['n_measures']}, batteurs {st['n_drummers']}. Seuils d'éligibilité de ce profil :
au moins {p['drum'].min_files} fichiers, {p['drum'].min_measures} mesures, aucun fichier au-delà de {int(p['drum'].max_share * 100)} % du groupe ({p['drum_why']}).

| Groupe (timekeeper:figure ou pas exacts) | Étiquette | Mesures | Fichiers | Batteurs |
|---|---|---|---|---|
{groups}

{skipped}
Relances retenues :

| Longueur | Pas de départ | Fichiers | Batteurs | Seuil de probabilité |
|---|---|---|---|---|
{fills}

## Mixage

Aucune mesure de corpus ne donne un mixage : les niveaux de piste sont calés hors ligne, en silence
(`calibrer_mix.py` : rendus WAV de 48 mesures, graines 1 et 2, LUFS intégré en solo par piste) sur les mêmes cibles
pour tous les profils `gmd_*` : accords −26 LUFS, basse −25, batterie (grosse caisse + reste du kit) −25,5, soit un mix
d'environ −21,5 LUFS (`wjazzd_swing` : −22,2).
{proofs['mix']}

{H.readme_section(style.hab, style.bpm[1], getattr(style, 'timing', {}), data['stats'].get('probabilistic', 0), p['key_name'])}

## Variété entre graines et entre chorus (D6)

{r['variete']}
{proofs['variete']}


{proofs['distance']}
## Limites

{r['limites']}
- **Basse et comping : règles de métier**, jugées à l'écoute seulement (D5 : aucune écoute exigée ; des preuves chiffrées,
  compte rendu `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`). WJazzD ne mesure que la basse du jazz ; aucun corpus distribuable de
  voicings ni de rythmes de comping (décision F3).
- **Batterie** : kit électronique Roland TD-11, dix batteurs en tout ; les vélocités et le micro-timing sont ceux du
  kit (niveaux recalés au kit de rendu). Un groupe de peu de fichiers décrit un batteur, pas un style (L2 § 4.1).
- **Manques du schéma** (comptes rendus B8 et R2-styles) : accords de puissance (« 5 »), options de basse d'A4
  (`walking.*` : notes fantômes, croches, notes répétées, arrivée) exclusives de `motifs` et de `templates` donc
  inutilisables avec des riffs, feel de basse et étiquettes d'arc pour les motifs de basse (`bass_feel` exige la marche),
  groupes de session indépendants (un kit « power » avec un tempo lent n'est pas exclu), boucle de grille fixe par air.

## Régénérer

`percussion.toml` est généré : `{{venv du labo}}/bin/python specs/019-bibliotheque-accompagnements/b8/generer.py {style.id}`
(lit le GMD en cache vérifié, n'écrit rien dans le labo). Les autres valeurs sont portées à la main dans `profile.toml`
(`styles.py` les génère aussi : le script réécrit tout, README compris).
"""



def load_proof(name: str) -> dict:
    path = HERE / "preuves" / name
    return json.loads(path.read_text()) if path.is_file() else {}


def proofs_section(pid: str) -> dict[str, str]:
    """Tableaux chiffrés lus dans `preuves/` (écrits par `calibrer_mix.py`, `resumer_variete.py`, `distance_gmd.py`)."""
    out = {"mix": "", "variete": "", "distance": ""}
    mix = load_proof("mix-lufs.json").get(pid)
    if mix:
        out["mix"] = (f"\n| Piste en solo | LUFS intégré |\n|---|---:|\n| accords | {mix['Chords']:.1f} |\n| basse | {mix['Bass']:.1f} |\n"
                      f"| grosse caisse | {mix['PercussionLow']:.1f} |\n| reste du kit | {mix['PercussionHigh']:.1f} |\n"
                      f"| batterie (somme en puissance) | {mix['kit']:.1f} |\n| mix complet | {mix['mix']:.1f} |\n")
    var = load_proof("variete.json").get(pid)
    if var:
        rows = "\n".join(f"| {label} | {m['intra_mesures_changees']:.0%} | {m['intra_paires_identiques']:.0%} | {m['chorus_distincts_par_graine']} | "
                          f"{m['inter_suites_distinctes']} / 16 | {m['inter_distance_graduee']:.0%} |" for label, m in var.items())
        out["variete"] = ("\nMesuré par `harmonium-lab seed-variety --chorus 4` (16 graines × 4 chorus, `resumer_variete.py`) :\n\n"
                          "| Couche | Mesures changées d'un chorus au suivant | Paires de chorus identiques | Chorus distincts / graine | "
                          "Suites distinctes (16 graines) | Distance graduée entre graines |\n|---|---:|---:|---:|---:|---:|\n" + rows + "\n")
    dist = load_proof("distance-gmd.json").get(pid)
    if dist:
        rows = "\n".join(f"| {name} | {v['profil']:.3f} | {v['sans_arc']:.3f} | {v['non_calibre']:.3f} |" for name, v in dist.items())
        out["distance"] = ("\n## Distance de la batterie au GMD\n\n`distance_gmd.py` : fréquence d'attaque par pas de chaque voix (Bray-Curtis, 0 = identique), "
                           "16 graines × 47 mesures, contre TOUTES les mesures `beat` 4/4 droites du style. « Non calibrée » : le même profil avec la "
                           "batterie de `wjazzd_swing` et sans arc.\n\n| Voix | Profil | Profil sans arc | Copie non calibrée |\n|---|---:|---:|---:|\n" + rows + "\n")
    return out


def mix_overrides() -> dict:
    path = HERE / "mix.json"
    return json.loads(path.read_text()) if path.is_file() else {}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lab", type=Path, default=Path.home() / "Project/bascanada/harmonium_lab")
    ap.add_argument("--out", type=Path, default=HERE.parents[2] / "profiles")
    ap.add_argument("ids", nargs="*")
    args = ap.parse_args()
    results = B.load_results(args.lab)
    for p in PROFILES:
        style: emit.Style = p["style"]
        if args.ids and style.id not in args.ids:
            continue
        data = B.build(results, p["drum"], args.lab)
        bpms = data["stats"]["bpm"]
        style.bpm = (float(round(percentile(bpms, 0.10))), float(round(percentile(bpms, 0.50))),
                     float(round(percentile(bpms, 0.90))))
        style.bpm_note = (f"tempo des {len(bpms)} fichiers `beat` du GMD (BPM du nom de fichier) : centile 10 = {style.bpm[0]:g}, "
                          f"médiane = {style.bpm[1]:g}, centile 90 = {style.bpm[2]:g}.")
        style.mix.update(mix_overrides().get(style.id, {}))
        hab = H.HABILLAGES[style.id]
        style.hab = hab
        style.timing = H.voice_timing(hab.families, hab.styles, style.bpm[1])
        style.voices_toml = H.voices_toml(style.timing, {v["note"] for v in data["voices"].values()}, hab.jitter)
        style.color = H.COLOR[style.id]
        style.register_drift = H.DRIFT[style.id]
        style.corpus = p["corpus"]
        style.comments = p.get("comments", {})
        out = args.out / style.id
        write(out / "profile.toml", emit.profile_toml(style, header(p, data)))
        write(out / "percussion.toml", B.percussion_toml(data, drum_header(p, data), p["fills_none_weight"]))
        write(out / "arc.toml", emit.arc_toml(style, data["tags"]))
        write(out / "dynamics.toml", emit.dynamics_toml(style))
        write(out / "arrangement.toml", emit.arrangement_toml(style))
        write(out / "set.toml", emit.SET_TOML)
        loud = "ride" if "ride" in data["tags"] else "open" if "open" in data["tags"] else None
        write(out / "session.toml", hab.session_toml(session_header(p, style, hab), "hat", loud))
        write(out / "cells.toml", emit.CELLS_TOML)
        write(out / "README.md", readme(p, data, style))
        print(f"{style.id} : {len(data['patterns'])} patterns, {len(data['phrases'])} phrases, {len(data['fills'])} relances, "
              f"tempo {style.bpm}, étiquettes {data['tags']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
