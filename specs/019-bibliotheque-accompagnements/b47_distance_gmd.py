#!/usr/bin/env python
"""B4 + B7 : corpus de référence pour `make style-distance` à partir du Groove MIDI Dataset.

Le GMD est du MIDI de batterie (CC BY 4.0, Gillick et al., ICML 2019) : pas d'audio, pas de
transcription. Ce script fabrique, **hors dépôt** (principe VII, jamais commité), un corpus au format
de `harmonium_lab.styledist` :

- `manifest.jsonl` : une piste par fichier `beat` retenu (id = identifiant du GMD) ;
- `tracks/<id>.wav` : le fichier MIDI rendu par `fluidsynth` avec GeneralUser GS (batterie seule) ;
- `transcriptions/<id>_drums.json` : coups kick / snare / charleston en doubles croches
  (`pos_16th` depuis la mesure 0), boucle de 4 mesures ;
- `grids.json` : tempo du fichier.

Les deux voix absentes (basse, accords) donnent une référence vide : seuls les groupes `batterie` et
la mesure `loop_drums` du symbolique ont un sens ici ; l'audio compare un mixage complet à une batterie
seule, biais identique pour le profil calibré et la copie non calibrée.

    cd ~/Project/bascanada/harmonium_lab
    .venv/bin/python <moteur>/specs/019-bibliotheque-accompagnements/b47_distance_gmd.py \\
        gmd_funk /tmp/b47/gmd_corpus_funk
"""

from __future__ import annotations

import argparse
import collections
import json
import subprocess
import sys
from pathlib import Path

from harmonium_lab.backing import drums
from harmonium_lab.backing import gmd_styles as g

SOUNDFONT = Path("~/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2").expanduser()

CORPORA = {
    "gmd_funk": {"styles": ["funk"], "per_drummer": 2},
    "gmd_neworleans_funk_secondline": {"styles": ["neworleans/funk", "neworleans/secondline"], "per_drummer": 3},
}
MIN_BARS = 24
CLASSES = {"kick": "kick", "snare": "snare", "sidestick": "snare", "hat_closed": "hat", "hat_open": "hat", "hat_pedal": "hat"}


def pick(rows, spec) -> list:
    """Fichiers `beat` 4/4 d'au moins 24 mesures, `per_drummer` par batteur (ordre du chemin)."""
    sel = [r for r in rows if r.style in spec["styles"] and r.beat_type == "beat" and r.time_signature == "4-4"]
    chosen = []
    count: dict[str, int] = collections.Counter()
    for r in sel:
        if count[r.drummer] < spec["per_drummer"] and r.duration * r.bpm / 60 / 4 >= MIN_BARS:
            chosen.append(r)
            count[r.drummer] += 1
    return chosen


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("corpus", choices=sorted(CORPORA))
    ap.add_argument("out", type=Path)
    args = ap.parse_args(argv)
    spec = CORPORA[args.corpus]
    archive = drums.open_archive()
    rows = drums.parse_rows(drums.read_info_rows(archive))
    out = args.out.expanduser()
    for sub in ("tracks", "transcriptions", "midi"):
        (out / sub).mkdir(parents=True, exist_ok=True)
    manifest, grids = [], {}
    for r in pick(rows, spec):
        ident = f"{r.drummer}_{r.ident}".replace("/", "_")
        data = archive.read(f"groove/{r.path}")
        mid = out / "midi" / f"{ident}.mid"
        mid.write_bytes(data)
        wav = out / "tracks" / f"{ident}.wav"
        subprocess.run(["fluidsynth", "-ni", "-q", "-F", str(wav), "-r", "44100", str(SOUNDFONT), str(mid)],
                       check=True, capture_output=True)
        perf = g.read_performance(data)
        hits = []
        for h in perf.hits:
            role = g.fine_role(h.pitch)
            cls = CLASSES.get(role)
            if cls:
                index, sub, _ = g.quantize(h.beat, "straight", 0.5)
                hits.append({"class": cls, "pos_16th": index * 4 + sub})
        nbars = int(perf.end_beat // 4)
        (out / "transcriptions" / f"{ident}_drums.json").write_text(json.dumps(
            {"track_id": ident, "loop": "drums", "bars": [0, 4], "drums": {"hits": hits}}, indent=0), encoding="utf-8")
        manifest.append({"id": ident, "file": f"tracks/{ident}.wav", "style": r.style, "drummer": r.drummer,
                         "bpm": r.bpm, "bars": nbars,
                         "licence": "CC-BY-4.0 (Groove MIDI Dataset v1.0.0, Google LLC ; Gillick et al., ICML 2019)"})
        grids[ident] = {"bpm": float(r.bpm), "tatums_per_beat": 2, "beats_per_bar": 4}
    (out / "manifest.jsonl").write_text("\n".join(json.dumps(m, ensure_ascii=False) for m in manifest) + "\n",
                                        encoding="utf-8")
    (out / "grids.json").write_text(json.dumps(grids, indent=1), encoding="utf-8")
    print(",".join(m["id"] for m in manifest))
    print(f"{len(manifest)} pistes dans {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
