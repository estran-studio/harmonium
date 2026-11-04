#!/usr/bin/env python3
"""R2-trad : habillage par graine (`[session]`, A2) des dix profils trad distribués.

    python3 habillage.py            # écrit profiles/<id>/session.toml, `[files] session`, section de README
    python3 habillage.py --check    # ne vérifie que les programmes et imprime les tailles

Aucun mécanisme du moteur n'est ajouté : tout passe par des champs de `[session]` existants. Les airs (cellules, formes,
ornements, rôles) ne changent pas ; seule leur réalisation varie.

D'où vient chaque valeur :
- plages de tempo : sous-plages de `meter.min_bpm..max_bpm` du profil (la plage de la tradition, déjà bornée par les phases
  009 à 012 d'après le corpus et le labo) ; le moteur refuse un tempo hors de cette plage ;
- tonalités : celles de `melody.key_modes` du profil (poids du corpus du labo : `references/trad-*/`) plus quelques
  tonalités courantes des airs de violon de la tradition (do, fa, mi dorien, la dorien) ; modes atteignables par le profil ;
- instruments, accompagnement, voicings, pieds, énergie, registre, arcs : règles de métier de chaque tradition (aucun
  corpus distribuable ne les mesure : décision F3), dites comme telles dans chaque README.

Programmes General MIDI 0-based de GeneralUser GS (banque 0), vérifiés dans le `phdr` de `assets/GeneralUser-GS.sf2` ;
kits de la banque 128 : 0 standard, 8 room, 16 power, 40 brush. Les profils québécois et acadiens gardent leurs pieds
(`percussion_bank = 1`, `harmonium_trad.sf2`) : seule leur densité varie.
"""

from __future__ import annotations

import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PROFILES = ROOT / "profiles"
SF2 = Path.home() / "workspace/harmonium/assets/GeneralUser-GS.sf2"

NOTE = {0: "do", 2: "re", 4: "mi", 5: "fa", 7: "sol", 9: "la", 10: "si_bemol"}
NOTE_FR = {0: "do", 2: "ré", 4: "mi", 5: "fa", 7: "sol", 9: "la", 10: "si♭"}
MODE_FR = {"ionian": "ionien", "mixolydian": "mixolydien", "dorian": "dorien", "aeolian": "éolien"}


def phdr() -> dict[int, dict[int, str]]:
    data = SF2.read_bytes()
    i = data.find(b"phdr")
    size = struct.unpack("<I", data[i + 4:i + 8])[0]
    out: dict[int, dict[int, str]] = {}
    for k in range(i + 8, i + 8 + size - 38, 38):
        name = data[k:k + 20].split(b"\0")[0].decode("latin1")
        program, bank = struct.unpack("<HH", data[k + 20:k + 24])
        out.setdefault(bank, {})[program] = name
    return out


BANKS = phdr()


def check(fields: dict[str, int], where: str) -> None:
    for key in ("melody_program", "countermelody_program", "chords_program", "bass_program"):
        if key in fields and fields[key] not in BANKS[0]:
            raise SystemExit(f"{where} : programme {fields[key]} absent de GeneralUser GS")
    if fields.get("percussion_bank") == 128 and fields["percussion_program"] not in BANKS[128]:
        raise SystemExit(f"{where} : kit {fields['percussion_program']} absent de GeneralUser GS")


def inst(fields: dict[str, int], where: str) -> str:
    check(fields, where)
    return "instruments = { " + ", ".join(f"{k} = {v}" for k, v in sorted(fields.items())) + " }"


def variant(group: str, name: str, weight: int, *lines: str) -> str:
    out = ["[[variants]]", f'group = "{group}"', f'name = "{name}"']
    if weight != 1:
        out.append(f"weight = {weight}")
    out += [line for line in lines if line]
    return "\n".join(out)


# --- spécification d'un profil -------------------------------------------------------------------------------------
STD_ENERGY = [("doux", 1, 0.85), ("normal", 2, None), ("fort", 1, 1.12)]
STD_FEET = [("discrets", 1, 0.8), ("normal", 2, None), ("fournis", 1, 1.2)]
STD_REGISTER = [("normal", 5, 0), ("haut", 1, 1), ("grave", 1, -1)]
STD_VOICINGS = [("triades_serrees", 3, "close_triad"), ("triades_fixes", 2, "fixed_triad")]
GUITAR_PIANO = [("guitare_nylon", 3, 24), ("guitare_acier", 3, 25), ("piano", 3, 0), ("piano_bastringue", 1, 3)]
BASSES = [("contrebasse", 4, 32), ("basse_doigts", 2, 33)]
BRITISH_KITS = [("standard", 4, 0), ("room", 2, 8), ("brush", 1, 40)]
ARC_WEIGHTS = {"defaut": 2, "montee": 3, "sommet": 3, "orchestre": 2}

# Mélodie : (nom, poids, melody_program, countermelody_program ou None). 40 violon, 110 « fiddle », 21 accordéon,
# 22 harmonica, 73 flûte, 72 piccolo (fifre), 71 clarinette, 56 trompette.
FIDDLE_QC = [("violon", 6, 40, 40), ("fiddle", 3, 110, 40), ("accordeon", 2, 21, 40)]
FIDDLE_BRIT = [("violon", 6, 40, None), ("fiddle", 2, 110, None), ("flute", 2, 73, None), ("accordeon", 1, 21, None)]

SPECS: dict[str, dict] = {
    "reel_quebecois": dict(
        label="Reel québécois", feet=True, default_bpm=230.0,
        bands=[("pose", 2, 220, 226), ("moyen", 3, 227, 233), ("vif", 2, 234, 240)],
        keys=[(2, "ionian", 3), (7, "ionian", 3), (9, "ionian", 2), (9, "mixolydian", 1), (2, "mixolydian", 1),
              (0, "ionian", 1), (9, "dorian", 1)],
        melody=FIDDLE_QC, comping=GUITAR_PIANO, bass=BASSES, kit=None,
        swing=[("droit", 1, 0.54), ("traditionnel", 3, 0.56), ("appuye", 1, 0.58)],
        orch=({"chords_program": 24}, {"chords_program": 25, "bass_program": 33}),
    ),
    "reel_acadien": dict(
        label="Reel acadien", feet=True, default_bpm=228.0,
        bands=[("pose", 2, 215, 222), ("moyen", 3, 223, 231), ("vif", 2, 232, 238)],
        keys=[(9, "ionian", 4), (2, "ionian", 4), (7, "ionian", 3), (9, "dorian", 2), (9, "mixolydian", 1),
              (4, "dorian", 1), (0, "ionian", 1)],
        melody=[("violon", 5, 40, 40), ("fiddle", 3, 110, 40), ("accordeon", 3, 21, 40)],
        comping=[("piano", 4, 0), ("piano_bastringue", 2, 3), ("guitare_nylon", 2, 24), ("guitare_acier", 2, 25)],
        bass=BASSES, kit=None,
        swing=[("droit", 1, 0.54), ("traditionnel", 3, 0.56), ("appuye", 1, 0.58)],
        orch=({"chords_program": 24}, {"chords_program": 0, "bass_program": 33}),
    ),
    "reel_britannique_coloniale": dict(
        label="Reel britannique colonial", feet=False, default_bpm=208.0,
        bands=[("pose", 2, 192, 201), ("moyen", 3, 202, 213), ("vif", 2, 214, 224)],
        keys=[(2, "ionian", 8), (7, "ionian", 7), (9, "ionian", 2), (0, "ionian", 2), (9, "aeolian", 1),
              (4, "aeolian", 1)],
        melody=FIDDLE_BRIT, comping=[("guitare_nylon", 3, 24), ("guitare_acier", 3, 25), ("piano", 3, 0)],
        bass=BASSES, kit=BRITISH_KITS, swing=None,
        orch=({"chords_program": 24}, {"chords_program": 25, "bass_program": 33}),
    ),
    "gigue_quebecoise": dict(
        label="Gigue québécoise", feet=True, default_bpm=172.5,
        bands=[("pose", 2, 160, 168), ("moyen", 3, 169, 177), ("vif", 2, 178, 185)],
        keys=[(2, "ionian", 3), (7, "ionian", 3), (9, "ionian", 2), (9, "mixolydian", 1), (2, "mixolydian", 1),
              (0, "ionian", 1)],
        melody=FIDDLE_QC, comping=GUITAR_PIANO, bass=BASSES, kit=None, swing=None,
        orch=({"chords_program": 24}, {"chords_program": 25, "bass_program": 33}),
    ),
    "gigue_acadienne": dict(
        label="Gigue acadienne", feet=True, default_bpm=180.0,
        bands=[("pose", 2, 168, 175), ("moyen", 3, 176, 184), ("vif", 2, 185, 192)],
        keys=[(2, "ionian", 6), (7, "ionian", 5), (9, "ionian", 5), (0, "ionian", 2), (4, "dorian", 2),
              (9, "dorian", 1)],
        melody=[("violon", 5, 40, 40), ("fiddle", 3, 110, 40), ("accordeon", 3, 21, 40)],
        comping=[("piano", 4, 0), ("piano_bastringue", 2, 3), ("guitare_nylon", 2, 24), ("guitare_acier", 2, 25)],
        bass=[("piano", 4, 0), ("contrebasse", 2, 32), ("basse_doigts", 1, 33)], kit=None, swing=None,
        orch=({"chords_program": 0}, {"chords_program": 3, "bass_program": 32}),
    ),
    "gigue_britannique_coloniale": dict(
        label="Gigue britannique coloniale", feet=False, default_bpm=156.0,
        bands=[("pose", 2, 144, 152), ("moyen", 3, 153, 160), ("vif", 2, 161, 168)],
        keys=[(2, "ionian", 8), (7, "ionian", 8), (9, "ionian", 4), (0, "ionian", 2), (9, "aeolian", 2),
              (4, "aeolian", 1)],
        melody=FIDDLE_BRIT, comping=[("guitare_nylon", 3, 24), ("guitare_acier", 3, 25), ("piano", 3, 0)],
        bass=BASSES, kit=BRITISH_KITS, swing=None,
        orch=({"chords_program": 24}, {"chords_program": 25, "bass_program": 33}),
    ),
    "hornpipe_britannique_coloniale": dict(
        label="Hornpipe britannique colonial", feet=False, default_bpm=92.0,
        bands=[("pose", 2, 80, 88), ("moyen", 3, 89, 96), ("vif", 2, 97, 104)],
        keys=[(7, "ionian", 8), (2, "ionian", 7), (9, "ionian", 3), (4, "aeolian", 1), (9, "aeolian", 1),
              (0, "ionian", 2)],
        melody=FIDDLE_BRIT, comping=[("guitare_nylon", 3, 24), ("guitare_acier", 3, 25), ("piano", 3, 0)],
        bass=BASSES, kit=BRITISH_KITS,
        swing=[("leger", 1, 0.62), ("hornpipe", 3, 0.65), ("appuye", 1, 0.68)],
        orch=({"chords_program": 24}, {"chords_program": 25, "bass_program": 33}),
    ),
    "marche_britannique_coloniale": dict(
        label="Marche britannique coloniale", feet=False, default_bpm=108.0, march=True,
        bands=[("pas_lent", 2, 90, 100), ("pas_ordinaire", 3, 101, 116), ("pas_rapide", 2, 117, 126)],
        keys=[(2, "ionian", 6), (7, "ionian", 6), (9, "ionian", 2), (2, "aeolian", 2), (0, "ionian", 2),
              (5, "ionian", 1)],
        melody=[("piccolo", 4, 72, 72), ("flute", 3, 73, 73), ("clarinette", 1, 71, 71), ("trompette", 1, 56, 56)],
        comping=None, bass=None, kit=[("standard", 4, 0), ("room", 2, 8), ("power", 1, 16)], swing=None,
        orch=None,
    ),
    "valse_quebecoise": dict(
        label="Valse québécoise", feet=True, default_bpm=145.0,
        bands=[("pose", 2, 140, 143), ("moyen", 3, 144, 147), ("vif", 2, 148, 150)],
        keys=[(2, "ionian", 3), (7, "ionian", 3), (9, "ionian", 2), (9, "mixolydian", 1), (2, "mixolydian", 1),
              (0, "ionian", 1), (5, "ionian", 1)],
        melody=[("violon_accordeon", 4, 40, 21), ("accordeon_violon", 2, 21, 40), ("fiddle_accordeon", 2, 110, 21),
                ("harmonica_violon", 1, 22, 40)],
        comping=GUITAR_PIANO, bass=BASSES, kit=None, swing=None,
        orch=({"chords_program": 24}, {"chords_program": 0, "bass_program": 33}),
    ),
    "valse_britannique_coloniale": dict(
        label="Valse britannique coloniale", feet=False, default_bpm=138.0,
        bands=[("lente", 2, 126, 134), ("moyenne", 3, 135, 142), ("vive", 2, 143, 150)],
        keys=[(7, "ionian", 8), (2, "ionian", 7), (0, "ionian", 2), (5, "ionian", 2), (9, "aeolian", 1),
              (9, "ionian", 1)],
        melody=FIDDLE_BRIT, comping=[("guitare_nylon", 3, 24), ("guitare_acier", 2, 25), ("piano", 4, 0)],
        bass=BASSES, kit=BRITISH_KITS, swing=None,
        orch=({"chords_program": 24}, {"chords_program": 0, "bass_program": 33}),
    ),
}

HEADER = """\
# Habillage par graine de `{id}` (A2 ; phase 019, R2-trad) : un tirage par groupe, les groupes se combinent
# ({possible} habillages possibles). GÉNÉRÉ par `specs/019-bibliotheque-accompagnements/r2-trad/habillage.py`.
#
# Ce qui vient du profil et de son corpus : les sous-plages de tempo (dans `meter.min_bpm..max_bpm`) et les tonalités
# (`melody.key_modes`, plus quelques tonalités courantes des airs de violon de la tradition). Le reste est de métier
# (instrument de mélodie, accompagnement, voicings de triade, pieds, énergie, registre, arcs sur les tours d'un air) :
# aucun corpus distribuable ne le mesure (décision F3). Programmes General MIDI (0-based) de GeneralUser GS, banque 0,
# vérifiés dans son `phdr`. Les airs ne changent pas : cellules, rôles, formes et ornements sont ceux du profil ; seule
# leur réalisation varie. Le tempo de l'hôte (`SetBpm`), son swing et son `[fixed_tonality]` gagnent sur la variante.
"""


def arcs_text(spec: dict, name: str) -> str:
    """Trois arcs nommés sur les chorus d'un set (un chorus = un tour de la forme ; trois tours par air). Chaque arc
    reprend à son début à chaque air (`repeat_from = 0`) : exposition douce, sommet, et un dernier tour qui se détache
    (`fall`, nombre de chorus connu)."""
    orch = spec["orch"]

    def i(fields: dict[str, int] | None, where: str) -> str:
        return ", " + inst(fields, where) if fields else ""

    out: list[str] = []
    out += ["[arcs.montee]", "repeat_from = 0", "",
            "[[arcs.montee.stages]]", "choruses = 1", "variants = [",
            "  { intensity = 0.8, density = 0.85 },", "]", "",
            "[[arcs.montee.stages]]", "choruses = 1", "variants = [",
            "  { weight = 2, intensity = 0.95, density = 0.95 },", "  { weight = 1, intensity = 1.0 },", "]", "",
            "[[arcs.montee.stages]]", "choruses = 1", "variants = [",
            "  { weight = 2, intensity = 1.1, density = 1.1 },", "  { weight = 1, intensity = 1.15, density = 1.05 },", "]", "",
            "[[arcs.montee.fall]]", "intensity = 1.25", "density = 1.2", ""]
    out += ["[arcs.sommet]", "repeat_from = 0", "",
            "[[arcs.sommet.stages]]", "choruses = 2", "variants = [",
            "  { weight = 2, intensity = 0.95, density = 0.95 },", "  { weight = 1, intensity = 1.0, density = 1.05 },", "]", "",
            "[[arcs.sommet.stages]]", "choruses = 1", "variants = [",
            "  { intensity = 1.2, density = 1.15 },", "]", "",
            "[[arcs.sommet.fall]]", "intensity = 1.3", "density = 1.2", ""]
    if orch:
        expo, top = orch
        out += ["[arcs.orchestre]", "repeat_from = 0", "",
                "[[arcs.orchestre.stages]]", "choruses = 1", "variants = [",
                f"  {{ intensity = 0.85, density = 0.9{i(expo, name + ' orchestre expo')} }},", "]", "",
                "[[arcs.orchestre.stages]]", "choruses = 1", "variants = [",
                "  { intensity = 1.0, density = 1.0 },", "]", "",
                "[[arcs.orchestre.stages]]", "choruses = 1", "variants = [",
                f"  {{ intensity = 1.15, density = 1.1{i(top, name + ' orchestre sommet')} }},", "]", "",
                "[[arcs.orchestre.fall]]", "intensity = 1.25", "density = 1.2",
                inst(top, name + " orchestre fall"), ""]
    return "\n".join(out)


def build(pid: str, spec: dict) -> tuple[str, dict[str, int]]:
    blocks: list[str] = []
    sizes: dict[str, int] = {}

    def add(group: str, items: list[str]) -> None:
        blocks.extend(items)
        sizes[group] = len(items)

    add("tempo", [variant("tempo", n, w, f"bpm = [{lo:.1f}, {hi:.1f}]") for n, w, lo, hi in spec["bands"]])
    if spec.get("swing"):
        add("swing", [variant("swing", n, w, f"swing = {s}") for n, w, s in spec["swing"]])
    add("tonalite", [variant("tonalite", f"{NOTE[k]}_{m}", w, f'tonality = {{ key = {k}, mode = "{m}" }}')
                     for k, m, w in spec["keys"]])
    add("melodie", [variant("melodie", n, w, inst({"melody_program": m} | ({"countermelody_program": c} if c else {}),
                                                  f"{pid} melodie {n}"))
                    for n, w, m, c in spec["melody"]])
    if spec.get("comping"):
        add("comping", [variant("comping", n, w, inst({"chords_program": p}, f"{pid} comping {n}"))
                        for n, w, p in spec["comping"]])
        add("basse", [variant("basse", n, w, inst({"bass_program": p}, f"{pid} basse {n}")) for n, w, p in spec["bass"]])
        add("voicing", [variant("voicing", n, w, f'voicing = "{v}"') for n, w, v in STD_VOICINGS])
        add("registre", [variant("registre", n, w, f"register_octaves = {{ Chords = {o} }}" if o else "")
                         for n, w, o in STD_REGISTER])
    if spec.get("kit"):
        add("kit", [variant("kit", n, w, inst({"percussion_bank": 128, "percussion_program": p}, f"{pid} kit {n}"))
                    for n, w, p in spec["kit"]])
    add("energie", [variant("energie", n, w, f"intensity = {v}" if v else "") for n, w, v in STD_ENERGY])
    add("pieds", [variant("pieds", n, w, f"density = {v}" if v else "") for n, w, v in STD_FEET])
    add("arc", [variant("arc", n, w, f'arc = "{n}"' if n != "defaut" else "")
                for n, w in ARC_WEIGHTS.items() if n != "orchestre" or spec["orch"]])
    possible = 1
    for n in sizes.values():
        possible *= n
    text = HEADER.format(id=pid, possible=f"{possible:,}".replace(",", " ")) + "\n" + "\n\n".join(blocks) + "\n\n" \
        + arcs_text(spec, pid)
    return text.rstrip() + "\n", sizes


def possible_text(sizes: dict[str, int]) -> str:
    total = 1
    for n in sizes.values():
        total *= n
    return f"{total:,}".replace(",", " ")


def readme(pid: str, spec: dict, sizes: dict[str, int]) -> str:
    keys = ", ".join(f"{NOTE_FR[k]} {MODE_FR[m]} ({w})" for k, m, w in spec["keys"])
    bands = ", ".join(f"{n} {lo}–{hi}" for n, _, lo, hi in spec["bands"])
    mel = ", ".join(f"{n} ({m}" + (f" / {c}" if c else "") + ")" for n, _, m, c in spec["melody"])
    lines = [
        "", "## Habillage par graine (019, R2-trad)", "",
        f"`session.toml` : une graine tire un habillage dans les bornes de la tradition, sans toucher aux airs (cellules, "
        f"rôles, formes et ornements sont ceux du profil ; seule leur réalisation varie). {' × '.join(str(n) for n in sizes.values())} "
        f"= {possible_text(sizes)} habillages possibles.", "",
        "| Groupe | Variantes (poids) | Source |", "|---|---|---|",
        f"| `tempo` | {bands} BPM | sous-plages de `meter.min_bpm..max_bpm` ({int(spec['bands'][0][2])}–{int(spec['bands'][-1][3])}) |",
    ]
    if spec.get("swing"):
        lines.append("| `swing` | " + ", ".join(f"{n} {s}" for n, _, s in spec["swing"]) +
                     f" | autour du swing du profil ; `SetSwing` de l'hôte gagne |")
    lines.append(f"| `tonalite` | {keys} | `melody.key_modes` du profil et tonalités courantes des airs de violon de la tradition |")
    lines.append(f"| `melodie` | {mel} | métier : instrument de mélodie (et de contre-mélodie) |")
    if spec.get("comping"):
        lines.append("| `comping` | " + ", ".join(f"{n} ({p})" for n, _, p in spec["comping"]) + " | métier : piano en « vamp » ou guitare |")
        lines.append("| `basse` | " + ", ".join(f"{n} ({p})" for n, _, p in spec["bass"]) + " | métier |")
        lines.append("| `voicing` | " + ", ".join(f"`{v}` ({w})" for _, w, v in STD_VOICINGS) + " | triades simples seulement |")
        lines.append("| `registre` | normal (5), haut (1), grave (1) : octave des accords | métier |")
    if spec.get("kit"):
        lines.append("| `kit` | " + ", ".join(f"{n} ({p})" for n, _, p in spec["kit"]) + " | kits de GeneralUser GS, banque 128 |")
    lines.append("| `energie` | doux 0,85 (1), normal (2), fort 1,12 (1) : multiplicateur d'intensité | métier |")
    feet = "pieds (podorythmie, `harmonium_trad.sf2`)" if spec["feet"] else (
        "caisses (marche)" if spec.get("march") else "percussions (kit de la banque 128)")
    lines.append(f"| `pieds` | discrets 0,8 (1), normal (2), fournis 1,2 (1) : multiplicateur de densité des {feet} | métier |")
    arcs = ", ".join(f"`{n}` ({w})" for n, w in ARC_WEIGHTS.items() if n != "orchestre" or spec["orch"])
    lines.append(f"| `arc` | {arcs} | arcs nommés sur les tours d'un air (un chorus = un tour) |")
    lines += [
        "",
        "Arcs nommés (chaque arc reprend à son début à chaque air d'un set ; le dernier tour du set se détache) : `defaut` laisse "
        "l'énergie plate ; `montee` (0,8 → 0,95 → 1,1) ; `sommet` (deux tours posés, un sommet à 1,2) ; "
        + ("`orchestre` (exposition à une guitare ou un piano plus doux, sommet à un autre comping et une autre basse) ; "
           if spec["orch"] else "") + "le dernier tour du set monte à 1,25 ou 1,3 (`fall`). "
        "Sans `fall` (set sans fin), le dernier chorus n'est pas détaché.",
        "",
        "L'hôte garde la main : `SetBpm`, `SetSwing` et `[fixed_tonality]` (grille de training) gagnent sur la variante. "
        "Les valeurs ci-dessus sont des règles de métier de la tradition, à valider à l'oreille ; aucune n'est mesurée.",
    ]
    if spec.get("march"):
        lines.insert(-1, "La marche n'a pas d'accompagnement : l'habillage ne porte que la mélodie (fifre, flûte), le kit "
                         "de la caisse claire et de la grosse caisse, le tempo, la tonalité, l'énergie et l'arc.\n")
    return "\n".join(lines) + "\n"


def main() -> None:
    only_check = "--check" in sys.argv
    for pid, spec in SPECS.items():
        text, sizes = build(pid, spec)
        print(f"{pid:34s} {sizes}")
        if only_check:
            continue
        (PROFILES / pid / "session.toml").write_text(text)
        prof = PROFILES / pid / "profile.toml"
        body = prof.read_text()
        if "session =" not in body.split("[meta]")[0]:
            body = re.sub(r"(\[files\]\n(?:[a-z_]+ = \"[^\"]+\"\n)+)", lambda m: m.group(1) + 'session = "session.toml"\n',
                          body, count=1)
            prof.write_text(body)
        rd = PROFILES / pid / "README.md"
        old = rd.read_text()
        old = old.split("\n## Habillage par graine (019, R2-trad)")[0].rstrip("\n") + "\n"
        rd.write_text(old + readme(pid, spec, sizes))


if __name__ == "__main__":
    main()
