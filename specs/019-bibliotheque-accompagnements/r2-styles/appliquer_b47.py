#!/usr/bin/env python3
"""R2-styles : habille `gmd_funk` et `gmd_neworleans_funk_secondline` (profils écrits à la main par B4 + B7).

Idempotent : réécrit `session.toml` et remplace, dans `profile.toml`, `dynamics.toml` et `set.toml`, les passages que
R2 change (tonalité verrouillée → tonalités de session, micro-timing par voix, couleur et dérive des voicings).
`percussion.toml` reste celui de `b47_batterie.py` (coups probabilistes). Lancer avec le venv du labo :

    ~/Project/bascanada/harmonium_lab/.venv/bin/python specs/019-bibliotheque-accompagnements/r2-styles/appliquer_b47.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent))

import b47_batterie as B  # noqa: E402
import habillage as H  # noqa: E402

PROFILES = HERE.parents[2] / "profiles"
DEFAULT = {"gmd_funk": (96.0, 80.0, 130.0, "mi dorien"), "gmd_neworleans_funk_secondline": (93.0, 82.0, 124.0, "si♭ mixolydien")}


def sub(text: str, pattern: str, repl: str, flags: int = re.S) -> str:
    new, n = re.subn(pattern, lambda _m: repl, text, count=1, flags=flags)
    if n != 1 and repl not in text:
        raise SystemExit(f"motif introuvable : {pattern}")
    return new


def readme(pid: str, h: H.Habillage, bpm: float) -> None:
    path = PROFILES / pid / "README.md"
    text = path.read_text()
    perc = (PROFILES / pid / "percussion.toml").read_text()
    prob = perc.count("probability =")
    timing = H.voice_timing(h.families, h.styles, bpm)
    key_name = DEFAULT[pid][3]
    text = text.replace("`[fixed_tonality]`, pas de piste `Melody`", "pas de `[fixed_tonality]` (tonique tirée par `session.toml`), pas de piste `Melody`")
    text = re.sub(r"\| `performance\.tracks\.Percussion\*` \|[^\n]*\n",
                  "| `performance.tracks.Percussion*.voices` | une entrée par note GM jouée, `on_beat` et `off_beat` en fraction de pas, sans `jitter` | "
                  "GMD, médianes en ms de l'écart à la grille par rôle (groupe 4/4 droit, pondérées par l'effectif), converties au tempo par défaut ; table dans « Habillage par graine » |\n", text)
    text = re.sub(r"\(`fixed_tonality` [^)]*\)", f"({key_name} par défaut ; la session tire la tonique, voir « Habillage par graine »)", text)
    text = re.sub(r"rigide \(pas de coup probabiliste dans le schéma : L2 § 6 point 5\), les\n  notes fantômes de la caisse claire y sont des coups à vélocité faible\.",
                  f"rigide (les notes fantômes de la caisse claire et les grosses caisses rares sont des coups probabilistes, `probability` = fréquence mesurée, A6 : {prob} coups ; le reste est un seuil à 0,5).", text)
    text = re.sub(r"\n  probabiliste \(L2 § 6 point 5\), pas de roulé\.", "\n  fixe ; les notes fantômes sont probabilistes (R2), pas de roulé.", text)
    text = re.sub(r"- \*\*Micro-timing\*\*.*?(?=\n- \*\*)", "- **Micro-timing** : médianes par voix (A6, R2) ; la dispersion mesurée (0,11 à 0,20 pas) n'est pas reproduite : un `jitter` de batterie partage son tirage avec la basse à notes de passage et la déplacerait (contrat `passing_keeps_neighbour_tracks_and_strong_bass_unchanged`). Le décalage est mesuré en ms et converti au tempo par défaut : il dérive de la mesure aux autres tempos tirés.", text, flags=re.S)
    text = re.sub(r"- \*\*Habillage par graine\*\* \(A2, vague 4\).*?(?=\n- \*\*)",
                  "- **Habillage par graine** (R2) : groupes indépendants, aucun lien entre kit, tempo et instruments (un kit « power » peut tomber avec le tempo le plus lent) ; tonalités, instruments, kits, voicings, registre, énergie et arcs sont de métier, jugés par des preuves chiffrées seulement (D5). Les options de marche d'A4 (notes fantômes, croches, notes répétées, arrivée) sont exclusives de `motifs` et de `passing` : la basse garde ses riffs.", text, flags=re.S)
    text = re.sub(r"- \*\*Instruments\*\* : un seul par piste[^\n]*\n(?:  [^\n]*\n)*", "", text)
    text = re.sub(r"\n## Habillage par graine.*?(?=\n## )", "", text, flags=re.S)
    section = H.readme_section(h, bpm, timing, prob, key_name)
    text = text.replace("\n## Contrat et preuves", "\n" + section + "\n## Contrat et preuves", 1)
    path.write_text(text)


def main() -> int:
    for pid, (bpm, lo, hi, key_name) in DEFAULT.items():
        h = H.HABILLAGES[pid]
        d = PROFILES / pid
        text = (d / "profile.toml").read_text()
        text = sub(text, r'arc = "arc.toml"\n(?!session)', 'arc = "arc.toml"\nsession = "session.toml"\n')
        text = sub(text, r'\[fixed_tonality\]\nkey = \d+\nmode = "\w+"\n',
                   "# Pas de `[fixed_tonality]` : la tonalité vient de la variante de session (R2) ; training installe le sien\n"
                   "# (le verrou de l'hôte gagne sur la variante).\n")
        text = sub(text, r'key_modes = \[[^\n]*\]', "key_modes = " + h.key_modes())
        m = re.search(r'(\[accompaniment\.chords\]\n(?:[^\n]*\n)*?voicing = "\w+"\n)((?:color = [^\n]*\n)?(?:register_drift = [^\n]*\n)?)', text)
        text = text[:m.start()] + m.group(1) + f"color = {H.COLOR[pid]}\nregister_drift = {H.DRIFT[pid]}\n" + text[m.end():]
        timing = B.timing_text(pid).rstrip("\n")
        note = (f"# Micro-timing de la batterie par voix (A6, R2-styles) : médianes de l'écart à la grille mesurées dans le GMD\n"
                f"# ({', '.join(h.styles)}, 4/4 droit, pondérées par l'effectif), converties en fraction de pas au tempo par défaut\n"
                f"# ({bpm:g} BPM : un pas = 15 000 / BPM ms). Aucun `jitter` sur la batterie : le tirage de `jitter` partage son flux\n"
                "# entre les pistes et la basse à notes de passage le décalerait (contrat\n"
                "# `passing_keeps_neighbour_tracks_and_strong_bass_unchanged`).\n")
        text = sub(text, r'# Micro-timing de la batterie : .*?\[performance\.tracks\.PercussionHigh\]\noffsets = \[[^\n]*\]\n',
                   note + timing + "\n") if "offsets = [-0" in text else text
        text = sub(text, r"Tonique mi pour toutes : la tonalité est verrouillée, la grille s'écrit\n# relative\.",
                   "Tonique tirée par la graine (session.toml ; mi par défaut) : la grille s'écrit\n# relative.") \
            if pid == "gmd_funk" else text
        (d / "profile.toml").write_text(text)
        dyn = (d / "dynamics.toml").read_text()
        dyn = sub(dyn, r"# Aucune bascule de mode : la tonalité est verrouillée \(`fixed_tonality`\)\.",
                  "# Tonalité : tirée par la variante de session (`session.toml`) ; les seuils de bascule parallèle sont écartés de la\n"
                  "# valence neutre (0,3) pour que le mode tiré ne bascule pas de lui-même.")
        dyn = sub(dyn, r"dark_below = [\d.]+\nbright_above = [\d.]+\nparallel_modes = \[[^\n]*\]",
                  f"dark_below = 0.1\nbright_above = 0.9\nparallel_modes = {h.parallel_modes()}")
        dyn = sub(dyn, r"color = \[\{[^\n]*\n?$", "color = " + h.color() + "\n", flags=re.S | re.M)
        (d / "dynamics.toml").write_text(dyn)
        st = (d / "set.toml").read_text()
        st = sub(st, r"La tonalité est verrouillée \(`fixed_tonality`\) : les poids de changement de\n# tonalité n'ont pas d'effet\.",
                 "La tonalité est celle de la variante de session (`session.toml`) : un air par\n# set, les poids de changement de tonalité n'ont pas d'effet.")
        (d / "set.toml").write_text(st)
        header = ("# Habillage par graine de `%s` (A2 ; phase 019, R2-styles) : un tirage par groupe, les groupes se combinent\n"
                  "# (%d habillages possibles). Écrit par `specs/019-bibliotheque-accompagnements/r2-styles/appliquer_b47.py` depuis\n"
                  "# `habillage.py`.\n#\n"
                  "# Ce qui vient de la source (GMD, CC BY 4.0) : les plages de tempo (centiles des BPM des fichiers `beat`, %g à %g) et\n"
                  "# le micro-timing de la batterie. Le reste est de métier (tonalités courantes du style, instruments, kits, familles de\n"
                  "# voicings, registre, énergie, arcs nommés) : aucun corpus distribuable ne le mesure (décision F3). Programmes General\n"
                  "# MIDI (0-based) de GeneralUser GS, banque 0 ; kits de la banque 128 : 0 standard, 8 room, 16 power, 24 electronic,\n"
                  "# 32 jazz. Aucune zone Odin (D1). Le tempo de l'hôte (`SetBpm`), son swing et son `[fixed_tonality]` gagnent sur la\n"
                  "# variante." % (pid, h.possible(), lo, hi))
        (d / "session.toml").write_text(h.session_toml(header, "sparse", "dense"))
        readme(pid, h, bpm)
        print(pid, h.possible())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
