#!/usr/bin/env python3
"""Régénère les kits d'écoute de la phase 012 avec le moteur courant.

Chaque kit A/B isole **une** couche : A ou B = profil distribué complet, l'autre =
même profil avec la seule couche retirée (sections TOML supprimées dans une copie
temporaire du dossier de profil). Même seed, mêmes banques, même durée ; les deux
versions sont ramenées au même volume moyen (RMS −20 dBFS) et l'ordre est tiré au
hasard (graine fixe). La correspondance est écrite à part.

Le kit `emotions` (phase 013, J4 ; `--kits emotions`) est un comparateur des quatre émotions :
9 versions par air (neutre, puis bas / haut de chaque émotion, les trois autres au neutre), même
graine, même air, mêmes mesures ; page `emotions/comparateur.html` (voir `build_emotions`).

Exemple :
    cargo build --release -p harmonium-cli
    /Users/william.quintal/Project/bascanada/harmonium_lab/.venv/bin/python \
        specs/012-vie-realisme/kits_ecoute.py \
        --output /Users/william.quintal/Project/bascanada/harmonium_lab/target/ecoute
"""

from __future__ import annotations

import argparse
import json
import random
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import tomllib
import wave
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
TABLE_HEADER = re.compile(r"^\s*\[\[?\s*([A-Za-z0-9_.\-]+)\s*\]\]?\s*(#.*)?$")

# Couche → préfixes de sections TOML (dans profile.toml) retirées pour la version « sans ».
LAYERS: dict[str, tuple[str, ...]] = {
    "mixage": ("mix",),
    "interpretation": ("performance",),
    "ornements": ("ornaments",),
    "accompagnement": ("accompaniment.bass.passing", "accompaniment.chords.strum"),
    "deuxieme_voix": ("heterophony",),
}
KIT_PROFILES: dict[str, tuple[str, ...]] = {
    "mixage": ("reel_quebecois", "valse_quebecoise", "marche_britannique_coloniale"),
    "interpretation": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise"),
    "ornements": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise"),
    "accompagnement": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise"),
    "deuxieme_voix": ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise",
                      "marche_britannique_coloniale"),
}
QUESTIONS = {
    "mixage": "Salle crédible ? Violon devant ? Pieds assez présents ?",
    "interpretation": "Plus vivant ? Effets trop discrets, justes ou trop marqués ?",
    "ornements": "Assez d'ornements ? Trop ? Naturels ?",
    "accompagnement": "Basse et guitare plus humaines ? Grattage trop lent ou trop rapide ?",
    "deuxieme_voix": "La 2ᵉ voix (entrée au 2ᵉ tour) aide-t-elle ? Trop forte ?",
}
SET_PROFILES = ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise")
NEUTRAL_AUTOMATION = """[[keys]]
bar = 1
intensity = 0.5
arousal = 0.5
valence = 0.3
density = 0.5
tension = 0.4
"""


def strip_sections(text: str, prefixes: tuple[str, ...]) -> str:
    """Retire les tables TOML dont le nom commence par un des préfixes."""
    kept, skip = [], False
    for line in text.splitlines(keepends=True):
        header = TABLE_HEADER.match(line)
        if header:
            name = header.group(1)
            skip = any(name == p or name.startswith(p + ".") for p in prefixes)
        if not skip:
            kept.append(line)
    return "".join(kept)


def read_wav(path: Path) -> tuple[np.ndarray, int]:
    data = path.read_bytes()
    i, channels, rate, bits = 12, 2, 44100, 32
    while i < len(data):
        cid, size = data[i:i + 4], struct.unpack("<I", data[i + 4:i + 8])[0]
        body = data[i + 8:i + 8 + size]
        if cid == b"fmt ":
            channels, rate = struct.unpack("<HI", body[2:8])
            bits = struct.unpack("<H", body[14:16])[0]
        if cid == b"data":
            if bits == 16:
                samples = np.frombuffer(body, dtype="<i2").astype(np.float64) / 32768.0
            elif bits == 24:
                raw = np.frombuffer(body, dtype=np.uint8).reshape(-1, 3).astype(np.int32)
                val = raw[:, 0] | (raw[:, 1] << 8) | (raw[:, 2] << 16)
                samples = np.where(val & 0x800000, val - 0x1000000, val) / 8388608.0
            else:
                samples = np.frombuffer(body, dtype="<f4").astype(np.float64)
            return samples.reshape(-1, channels), rate
        i += 8 + size + (size & 1)
    raise ValueError(f"{path} : pas de bloc data")


def write_wav16(path: Path, samples: np.ndarray, rate: int) -> None:
    pcm = (np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as out:
        out.setnchannels(samples.shape[1])
        out.setsampwidth(2)
        out.setframerate(rate)
        out.writeframes(pcm.tobytes())


def db(value: float) -> float:
    return 20.0 * float(np.log10(value + 1e-12))


def export(binary: Path, profile_dir: Path, wav: Path, args: argparse.Namespace,
           bars: int, automation: Path | None = None, seed: int | None = None,
           truth: Path | None = None) -> None:
    seed = args.seed if seed is None else seed
    cmd = [str(binary), "--export", "--profile", str(profile_dir), "--seed", str(seed),
           "--bars", str(bars), "--tours", "2", "--soundfont", str(args.soundfont),
           "--feet-soundfont", str(args.feet_soundfont), "--record-wav", str(wav)]
    if automation is not None:
        cmd += ["--automation", str(automation)]
    if truth is not None:
        cmd += ["--record-truth", str(truth)]
    subprocess.run(cmd, check=True, capture_output=True, timeout=900)


def build_ab_kits(args: argparse.Namespace, tmp: Path) -> list[dict]:
    rng = random.Random(args.seed * 1000 + 12)
    report = []
    for kit, prefixes in LAYERS.items():
        out = args.output / kit
        out.mkdir(parents=True, exist_ok=True)
        mapping = []
        for profile in KIT_PROFILES[kit]:
            source = ROOT / "profiles" / profile
            stripped = tmp / kit / profile
            shutil.copytree(source, stripped)
            toml = stripped / "profile.toml"
            before_text = toml.read_text(encoding="utf-8")
            after_text = strip_sections(before_text, prefixes)
            if after_text == before_text:
                raise SystemExit(f"{kit}/{profile} : aucune section {prefixes} à retirer")
            toml.write_text(after_text, encoding="utf-8")
            # Seed où la couche s'entend le plus (écart d'énergie avec/sans), parmi `--seeds` seeds.
            best = None
            for seed in range(args.seed, args.seed + args.seeds):
                full_wav = tmp / f"{kit}-{profile}-{seed}-avec.wav"
                without_wav = tmp / f"{kit}-{profile}-{seed}-sans.wav"
                export(args.binary, source, full_wav, args, args.bars, seed=seed)
                export(args.binary, stripped, without_wav, args, args.bars, seed=seed)
                with_samples, rate = read_wav(full_wav)
                without_samples, _ = read_wav(without_wav)
                frames = min(len(with_samples), len(without_samples))
                a, b = with_samples[:frames], without_samples[:frames]
                change = db(float(np.sqrt(np.mean((a - b) ** 2)))) - db(float(np.sqrt(np.mean(a ** 2))))
                if best is None or change > best[0]:
                    best = (change, seed, rate, a, b, db(float(np.abs(with_samples).max())))
            change, seed, rate, with_part, without_part = best[:5]
            if change < -60.0:
                raise SystemExit(f"{kit}/{profile} : la couche ne change rien au rendu")
            versions = {"avec": with_part, "sans": without_part}
            order = ["avec", "sans"]
            rng.shuffle(order)
            for letter, version in zip("AB", order):
                samples = versions[version]
                gain = 10.0 ** ((-20.0 - db(float(np.sqrt(np.mean(samples ** 2))))) / 20.0)
                write_wav16(out / f"{profile}-{letter}.wav", samples * gain, rate)
                mapping.append(f"{profile} {letter} = {version} {kit} (seed {seed})")
            report.append({"kit": kit, "profile": profile, "seed": seed,
                           "seconds": round(len(with_part) / rate, 1),
                           "difference_db": round(change, 1), "peak_with_dbfs": round(best[5], 2)})
        (out / "correspondance.txt").write_text("\n".join(mapping) + "\n", encoding="utf-8")
        (out / "question.txt").write_text(
            f"{QUESTIONS[kit]}\nÉcouter A puis B ; noter la préférence avant d'ouvrir correspondance.txt.\n",
            encoding="utf-8")
    return report


def build_sets(args: argparse.Namespace, tmp: Path) -> list[dict]:
    out = args.output / "sets"
    out.mkdir(parents=True, exist_ok=True)
    automation = tmp / "neutre.toml"
    automation.write_text(NEUTRAL_AUTOMATION, encoding="utf-8")
    report = []
    for profile in SET_PROFILES:
        wav = tmp / f"set-{profile}.wav"
        export(args.binary, ROOT / "profiles" / profile, wav, args, args.set_bars, automation)
        samples, rate = read_wav(wav)
        write_wav16(out / f"{profile}.wav", samples, rate)
        report.append({"kit": "sets", "profile": profile, "seconds": round(len(samples) / rate, 1),
                       "peak_dbfs": round(db(float(np.abs(samples).max())), 2)})
    return report


# Ordre d'ajout des couches dans la progression (du moteur nu au moteur complet).
PROGRESSION = ("mixage", "interpretation", "ornements", "accompagnement", "deuxieme_voix")
PROGRESSION_PROFILES = ("reel_quebecois", "valse_quebecoise")


def build_progression(args: argparse.Namespace, tmp: Path) -> list[dict]:
    """Même air, couches ajoutées une à une : 0 = nu, 5 = complet (fichiers nommés, pas à l'aveugle)."""
    out = args.output / "progression"
    out.mkdir(parents=True, exist_ok=True)
    report = []
    for profile in PROGRESSION_PROFILES:
        source = ROOT / "profiles" / profile
        text = (source / "profile.toml").read_text(encoding="utf-8")
        for step in range(len(PROGRESSION) + 1):
            removed = tuple(p for layer in PROGRESSION[step:] for p in LAYERS[layer])
            variant = tmp / "progression" / profile / str(step)
            shutil.copytree(source, variant)
            (variant / "profile.toml").write_text(strip_sections(text, removed), encoding="utf-8")
            wav = tmp / f"prog-{profile}-{step}.wav"
            export(args.binary, variant, wav, args, args.bars)
            samples, rate = read_wav(wav)
            gain = 10.0 ** ((-20.0 - db(float(np.sqrt(np.mean(samples ** 2))))) / 20.0)
            label = "nu" if step == 0 else "+" + PROGRESSION[step - 1]
            write_wav16(out / f"{profile}-{step}-{label.lstrip('+')}.wav", samples * gain, rate)
            report.append({"kit": "progression", "profile": profile, "step": step, "label": label,
                           "seconds": round(len(samples) / rate, 1)})
    (out / "question.txt").write_text(
        "Écouter 0 → 5 dans l'ordre : 0 = moteur nu, chaque fichier ajoute une couche "
        "(mixage, interprétation, ornements, accompagnement, 2ᵉ voix). "
        "Laquelle s'entend ? Laquelle ne change rien pour toi ?\n", encoding="utf-8")
    return report


COMPARATOR_PROFILES = ("reel_quebecois", "valse_quebecoise", "gigue_quebecoise")
LAYER_LABELS = {
    "mixage": "Mixage (salle, placement, niveaux)",
    "interpretation": "Interprétation (décalages, liaisons, nuances)",
    "ornements": "Ornements",
    "accompagnement": "Accompagnement (basse qui marche, accords grattés)",
    "deuxieme_voix": "2ᵉ voix",
}


def seamless_loop(samples: np.ndarray, start: int, end: int, rate: int, fade_ms: float = 40.0) -> np.ndarray:
    """Boucle sans clic : le début reçoit, en fondu à puissance constante, la suite réelle de la fin.

    À la reprise, le lecteur passe de `samples[end - 1]` à `loop[0]` ; comme `loop[0:n]` commence par
    `samples[end:end + n]` (la musique qui suivait vraiment), il n'y a pas de saut d'échantillon.
    """
    loop = samples[start:end].copy()
    n = min(int(rate * fade_ms / 1000.0), len(samples) - end, len(loop))
    if n <= 0:
        raise SystemExit("boucle sans suite disponible : rendre quelques mesures de plus")
    phase = np.linspace(0.0, np.pi / 2.0, n)[:, None]
    loop[:n] = loop[:n] * np.sin(phase) + samples[end:end + n] * np.cos(phase)
    return loop


def _fmt(value: object) -> str:
    """Valeur TOML littérale (nombres, chaînes, tableaux imbriqués)."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return repr(round(value, 6)) if isinstance(value, float) else str(value)
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    return "[" + ", ".join(_fmt(v) for v in value) + "]"


def _clamp(x: float, lo: float, hi: float) -> float:
    return max(lo, min(hi, x))


# Version « marquée » des couches les plus discrètes, dans les bornes de validation du moteur :
# pour faire entendre ce que fait chaque couche, pas une proposition de valeurs.
def _mark_table(name: str, table: dict) -> dict:
    changed = {}
    if name == "performance.tracks.Melody":
        if "offsets" in table:
            changed["offsets"] = [_clamp(x * 5.0, -0.2, 0.2) for x in table["offsets"]]
        if "jitter" in table:
            changed["jitter"] = _clamp(table["jitter"] * 4.0, 0.0, 0.1)
        if "phrase" in table:
            changed["phrase"] = [[p, _clamp(d * 4.0, -20.0, 20.0)] for p, d in table["phrase"]]
        if "breath" in table:
            changed["breath"] = _clamp(table["breath"] * 3.0, 0.0, 0.5)
    elif name == "performance.tracks.Melody.rules" and table.get("articulation", "Normal") != "Normal":
        changed["weight"] = _clamp(table.get("weight", 0.0) * 6.0, 0.0, 1_000_000.0)
        if "shorten" in table:
            changed["shorten"] = _clamp(table["shorten"] * 2.0, 0.0, 0.8)
        if "accent" in table:
            changed["accent"] = _clamp(float(table["accent"]) * 3.0, 0.0, 20.0)
    elif name == "performance.tracks.Melody.emotions":
        for key, factor, hi in (("tension_advance", 3.0, 0.2), ("tension_detach", 2.5, 0.4),
                                ("arousal_accent", 3.0, 20.0)):
            if key in table:
                changed[key] = _clamp(table[key] * factor, 0.0, hi)
    elif name == "ornaments" and "rate_scale" in table:
        changed["rate_scale"] = 10.0
    elif name == "mix.room":
        changed = {"level": _clamp(table.get("level", 0.0) * 2.2, 0.0, 1.0),
                   "room_size": _clamp(table.get("room_size", 0.0) + 0.25, 0.0, 1.0)}
    elif name.startswith("mix.tracks."):
        if "reverb_send" in table:
            changed["reverb_send"] = _clamp(table["reverb_send"] * 2.5, 0.0, 1.0)
        if "pan" in table:
            changed["pan"] = _clamp(table["pan"] * 2.5, -0.9, 0.9)
    elif name == "mix.eq" and "presence_db" in table:
        changed["presence_db"] = _clamp(table["presence_db"] * 2.0, -12.0, 12.0)
    return changed


def mark_profile(text: str) -> str:
    """Applique `_mark_table` table par table, en réécrivant seulement les lignes `clé = valeur` touchées."""
    out, block, name = [], [], ""

    def flush() -> None:
        if block:
            try:
                table = tomllib.loads("".join(block))
            except tomllib.TOMLDecodeError:
                table = {}
            changed = _mark_table(name, table)
            for line in block:
                key = line.split("=", 1)[0].strip() if "=" in line and not line.lstrip().startswith("#") else ""
                out.append(f"{key} = {_fmt(changed[key])}\n" if key in changed else line)
            block.clear()

    for line in text.splitlines(keepends=True):
        header = TABLE_HEADER.match(line)
        if header:
            flush()
            name = header.group(1)
            out.append(line)
            continue
        block.append(line)
    flush()
    return "".join(out)


def build_comparator(args: argparse.Namespace, tmp: Path) -> list[dict]:
    """Comparateur aux valeurs distribuées."""
    return _build_comparator(args, tmp, "comparateur", marked=False)


def build_comparator_marked(args: argparse.Namespace, tmp: Path) -> list[dict]:
    """Même comparateur, mixage / interprétation / ornements nettement exagérés (pour les entendre)."""
    return _build_comparator(args, tmp, "comparateur-marque", marked=True)


def _build_comparator(args: argparse.Namespace, tmp: Path, kit: str, marked: bool) -> list[dict]:
    """Une boucle par combinaison de couches (2⁵), toutes alignées à l'échantillon près.

    Les décalages d'interprétation ne déplacent pas les barres (grille publiée) : la même fenêtre de
    mesures, découpée aux `bar_start_frames` de la vérité, est donc alignée dans toutes les versions.
    La page HTML les joue en même temps et n'entend qu'une combinaison : on bascule sans perdre le fil.
    """
    out = args.output / kit
    out.mkdir(parents=True, exist_ok=True)
    report, index = [], {"layers": [{"id": layer, "label": LAYER_LABELS[layer]} for layer in PROGRESSION],
                         "profiles": []}
    for profile in COMPARATOR_PROFILES:
        source = ROOT / "profiles" / profile
        text = (source / "profile.toml").read_text(encoding="utf-8")
        if marked:
            text = mark_profile(text)
        renders = {}
        for mask in range(1 << len(PROGRESSION)):
            removed = tuple(p for bit, layer in enumerate(PROGRESSION) if not mask >> bit & 1
                            for p in LAYERS[layer])
            variant = tmp / kit / profile / str(mask)
            shutil.copytree(source, variant)
            (variant / "profile.toml").write_text(strip_sections(text, removed), encoding="utf-8")
            wav, truth = tmp / f"{kit}-{profile}-{mask}.wav", tmp / f"{kit}-{profile}-{mask}.json"
            export(args.binary, variant, wav, args, args.bars, truth=truth)
            renders[mask] = (read_wav(wav), json.loads(truth.read_text(encoding="utf-8")))
        full = len(renders) - 1
        (_, rate), truth_full = renders[full]
        # Fenêtre : à partir de la première mesure où la 2ᵉ voix joue (toutes les couches audibles).
        second_voice = [m["bar"] for m in truth_full["measures"]
                        if any(n["track"] == 5 for n in m["measure"]["notes"])]
        first = min(second_voice) if second_voice else 1
        starts = truth_full["bar_start_frames"]
        window = (starts[str(first)], starts[str(first + args.loop_bars)])
        files = {}
        for mask, ((samples, _), truth) in renders.items():
            if (truth["bar_start_frames"][str(first)], truth["bar_start_frames"][str(first + args.loop_bars)]) != window:
                raise SystemExit(f"{profile}/{mask} : barres décalées, boucles non alignées")
            loop = seamless_loop(samples, window[0], window[1], rate)
            gain = 10.0 ** ((-20.0 - db(float(np.sqrt(np.mean(loop ** 2))))) / 20.0)
            name = f"{profile}-{mask:02d}.wav"
            write_wav16(out / name, loop * gain, rate)
            files[str(mask)] = name
        index["profiles"].append({"id": profile, "bars": [first, first + args.loop_bars - 1],
                                  "seconds": round((window[1] - window[0]) / rate, 2), "files": files})
        report.append({"kit": kit, "profile": profile, "bars": [first, first + args.loop_bars - 1],
                       "seconds": round((window[1] - window[0]) / rate, 2), "versions": len(files)})
    (out / "index.json").write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    html = COMPARATOR_HTML if not marked else COMPARATOR_HTML.replace(
        "<h1>Comparateur de couches</h1>",
        "<h1>Comparateur de couches — version marquée</h1><p><b>Mixage, interprétation et ornements "
        "sont volontairement exagérés</b> pour entendre ce que fait chaque couche ; ce ne sont pas les "
        "réglages du jeu.</p>")
    (out / "comparateur.html").write_text(html, encoding="utf-8")
    return report


COMPARATOR_HTML = """<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><title>Harmonium — comparateur de couches</title>
<style>
body{font-family:system-ui,sans-serif;max-width:720px;margin:2em auto;padding:0 1em;line-height:1.5}
button{font-size:1.1em;padding:.4em 1.2em;margin-right:.5em}
label{display:block;font-size:1.15em;margin:.35em 0;cursor:pointer}
input[type=checkbox]{transform:scale(1.4);margin-right:.6em}
#etat{color:#666;margin-top:1em}
</style></head><body>
<h1>Comparateur de couches</h1>
<p>La même boucle tourne en continu. Coche ou décoche une couche : le changement est immédiat, au même
moment de la musique. Toutes les versions ont le même volume moyen.</p>
<p><label for="profil" style="display:inline">Air : </label><select id="profil"></select></p>
<div id="couches"></div>
<p><button id="jouer">▶ Jouer</button><button id="arreter">■ Arrêter</button>
<button id="tout">Tout</button><button id="rien">Rien</button></p>
<div id="etat">Chargement…</div>
<script>
const ctx = new AudioContext();
let index, voices = [], playing = false, startAt = 0;
const etat = document.getElementById('etat');
function mask() {
  return index.layers.reduce((m, l, i) => m | (document.getElementById(l.id).checked ? 1 << i : 0), 0);
}
function applyMask() {
  const m = mask(), t = ctx.currentTime;
  voices.forEach((v, i) => { v.gain.gain.cancelScheduledValues(t);
    v.gain.gain.setTargetAtTime(i === m ? 1 : 0, t, 0.015); });
  etat.textContent = playing ? 'Version ' + m + ' — ' + index.layers.filter((l, i) => m >> i & 1)
    .map(l => l.label.split(' (')[0]).join(' + ') : etat.textContent;
}
async function load(profileId) {
  stop();
  const p = index.profiles.find(x => x.id === profileId);
  etat.textContent = 'Chargement de ' + profileId + '… (32 versions)';
  const buffers = await Promise.all(Object.keys(p.files).sort((a, b) => a - b).map(async k =>
    ctx.decodeAudioData(await (await fetch(p.files[k])).arrayBuffer())));
  voices = buffers.map(b => ({buffer: b, gain: ctx.createGain(), src: null}));
  voices.forEach(v => { v.gain.gain.value = 0; v.gain.connect(ctx.destination); });
  etat.textContent = 'Prêt : mesures ' + p.bars[0] + ' à ' + p.bars[1] + ' (' + p.seconds + ' s en boucle).';
}
function play() {
  if (playing || !voices.length) return;
  ctx.resume(); startAt = ctx.currentTime + 0.1;
  voices.forEach(v => { v.src = ctx.createBufferSource(); v.src.buffer = v.buffer; v.src.loop = true;
    v.src.connect(v.gain); v.src.start(startAt); });
  playing = true; applyMask();
}
function stop() {
  voices.forEach(v => { if (v.src) { v.src.stop(); v.src.disconnect(); v.src = null; } });
  playing = false;
}
(async () => {
  index = await (await fetch('index.json')).json();
  const box = document.getElementById('couches');
  index.layers.forEach(l => { const lab = document.createElement('label');
    lab.innerHTML = '<input type="checkbox" id="' + l.id + '" checked>' + l.label; box.appendChild(lab);
    lab.querySelector('input').addEventListener('change', applyMask); });
  const sel = document.getElementById('profil');
  index.profiles.forEach(p => sel.add(new Option(p.id.replace('_', ' '), p.id)));
  sel.addEventListener('change', () => load(sel.value));
  document.getElementById('jouer').onclick = play;
  document.getElementById('arreter').onclick = stop;
  document.getElementById('tout').onclick = () => { index.layers.forEach(l => document.getElementById(l.id).checked = true); applyMask(); };
  document.getElementById('rien').onclick = () => { index.layers.forEach(l => document.getElementById(l.id).checked = false); applyMask(); };
  await load(sel.value);
})();
</script></body></html>
"""


# ---------------------------------------------------------------- kit « emotions » (phase 013, J4)

EMOTION_PROFILES = ("reel_quebecois", "gigue_quebecoise", "valse_quebecoise",
                    "marche_britannique_coloniale")
# La caisse claire (Toe, notes GM 38 et 40) de la marche est muette dans `harmonium_trad.sf2`
# (mesuré : solo Toe à −130 dBFS) : ses pieds sont rendus avec la banque GM, comme son profil
# l'attend (`percussion_bank = 128`). Sans cela, densité et arousal ne s'entendraient pas aux pieds.
GM_FEET_PROFILES = ("marche_britannique_coloniale",)
EMOTIONS = {
    "arousal": "Arousal (énergie : tempo, force, remplissage)",
    "valence": "Valence (clair ↔ sombre)",
    "density": "Densité (texture : pistes, pieds, accords)",
    "tension": "Tension (nervosité : contretemps, notes sèches)",
}
# Niveau bas / haut de chaque axe : les bornes de l'API (valence : −1 / +1, les autres : 0 / 1).
EMOTION_EXTREMES = {"arousal": (0.0, 1.0), "valence": (-1.0, 1.0), "density": (0.0, 1.0), "tension": (0.0, 1.0)}
EMOTION_KEYS = ("arousal", "valence", "density", "tension")
LOUD_LIMIT = 10.0 ** (-1.0 / 20.0)  # crête maximale du kit : −1 dBFS


def neutral_emotions(profile_dir: Path) -> dict[str, float]:
    """Neutre du profil : les courbes de `dynamics.toml` y valent leur valeur « sans effet »."""
    dynamics = tomllib.loads((profile_dir / "dynamics.toml").read_text(encoding="utf-8"))
    knee = next(x for x, y in dynamics["tension_duration"] if abs(y - 1.0) < 1e-9)
    return {"arousal": 0.5, "valence": dynamics["neutral_valence"], "density": 0.5, "tension": knee}


def emotion_automation(base: dict[str, float], target: dict[str, float]) -> str:
    """Neutre à la mesure 1 (l'air est construit au neutre), cible dès la mesure 2.

    L'air (matériel mélodique, tonalité, registre) se fige à sa construction avec les émotions du
    moment ; ce que fait ensuite une émotion en cours d'air, c'est ce que le jeu fera entendre. Toutes
    les versions partagent donc exactement la même mélodie, et seules les couches pilotées changent.
    """
    return "".join(f"[[keys]]\nbar = {bar}\n" + "".join(f"{k} = {values[k]}\n" for k in EMOTION_KEYS)
                   for bar, values in ((1, base), (2, target)))


def melody_notes(truth: dict, first: int, last: int) -> set[tuple[int, int, int]]:
    return {(e["measure"]["index"], n["start_step"], n["pitch"]) for e in truth["measures"]
            if first <= e["measure"]["index"] < last for n in e["measure"]["notes"] if n["track"] == 0}


def build_emotions(args: argparse.Namespace, tmp: Path) -> list[dict]:
    """Une boucle par (émotion, niveau) : 9 versions par air, même graine, même air, mêmes mesures.

    Le tempo change avec l'arousal (et un peu avec la tension) : les boucles de 16 mesures n'ont donc
    pas la même durée en secondes, et les jouer en parallèle, comme le comparateur de couches, les
    ferait dériver. La page ne joue qu'**une** version à la fois et, à chaque changement, reprend la
    nouvelle version à la **même position musicale** (même mesure, même fraction de mesure) d'après les
    débuts de mesure publiés dans la vérité (`bar_start_frames`) : soit tout de suite, soit au début
    de la mesure suivante (réglage par défaut, jonction sans saut de pulse). Le volume n'est pas égalisé
    version par version (il fait partie de l'effet) : un seul gain par air, choisi pour que la plus
    forte version reste sous −1 dBFS ; une case « volume égalisé » permet d'écouter le reste.
    """
    out = args.output / "emotions"
    out.mkdir(parents=True, exist_ok=True)
    report = []
    index = {"emotions": [{"id": e, "label": label} for e, label in EMOTIONS.items()],
             "levels": ["bas", "neutre", "haut"], "profiles": []}
    for profile in EMOTION_PROFILES:
        source = ROOT / "profiles" / profile
        base = neutral_emotions(source)
        render_args = args
        if profile in GM_FEET_PROFILES:
            render_args = argparse.Namespace(**{**vars(args), "feet_soundfont": args.soundfont})
        versions: dict[str, dict[str, float]] = {"neutre": base}
        for emotion, (low, high) in EMOTION_EXTREMES.items():
            versions[f"{emotion}-bas"] = {**base, emotion: low}
            versions[f"{emotion}-haut"] = {**base, emotion: high}
        renders = {}
        for key, values in versions.items():
            automation = tmp / f"emotions-{profile}-{key}.toml"
            automation.write_text(emotion_automation(base, values), encoding="utf-8")
            wav, truth = tmp / f"emotions-{profile}-{key}.wav", tmp / f"emotions-{profile}-{key}.json"
            export(args.binary, source, wav, render_args, args.bars, automation=automation, truth=truth)
            renders[key] = (read_wav(wav), json.loads(truth.read_text(encoding="utf-8")))
        (_, rate), reference = renders["neutre"]
        tour2 = [e["measure"]["index"] for e in reference["measures"]
                 if e["measure"]["position"] and e["measure"]["position"]["section"] == "Tour"
                 and e["measure"]["position"]["tour"] == 2]
        first, last = min(tour2), min(tour2) + args.loop_bars
        if last > max(tour2) + 1:
            raise SystemExit(f"{profile} : le 2ᵉ tour est plus court que la boucle")
        loops, stats = {}, {}
        for key, ((samples, _), truth) in renders.items():
            starts = truth["bar_start_frames"]
            window = (starts[str(first)], starts[str(last)])
            loop = seamless_loop(samples, window[0], window[1], rate)
            tempi = [e["measure"]["tempo"] for e in truth["measures"] if first <= e["measure"]["index"] < last]
            wrap = float(np.max(np.abs(loop[0] - samples[window[1] - 1])))
            inside = float(np.max(np.abs(np.diff(loop, axis=0))))
            loops[key] = loop
            stats[key] = {
                "seconds": (window[1] - window[0]) / rate, "bpm": sum(tempi) / len(tempi),
                "bars": [(starts[str(b)] - window[0]) / rate for b in range(first, last + 1)],
                "rms": float(np.sqrt(np.mean(loop ** 2))), "peak": float(np.max(np.abs(loop))),
                "wrap_jump": wrap, "max_step": inside,
                "same_air": len(melody_notes(truth, first, last) & melody_notes(reference, first, last))
                            / len(melody_notes(reference, first, last)),
                "measures": sum(1 for e in truth["measures"] if first <= e["measure"]["index"] < last)}
            if stats[key]["measures"] != args.loop_bars:
                raise SystemExit(f"{profile}/{key} : {stats[key]['measures']} mesures au lieu de {args.loop_bars}")
            if wrap > inside:
                raise SystemExit(f"{profile}/{key} : saut de {wrap:.4f} à la reprise (> {inside:.4f} dans la boucle)")
        gain = 10.0 ** ((-20.0 - db(stats["neutre"]["rms"])) / 20.0)
        gain = min(gain, LOUD_LIMIT / max(s["peak"] for s in stats.values()))
        files = {}
        for key, loop in loops.items():
            name = f"{profile}-{key}.wav"
            write_wav16(out / name, loop * gain, rate)
            written, _ = read_wav(out / name)  # vérifie le fichier, pas le tampon en mémoire
            file_peak = float(np.max(np.abs(written)))
            if file_peak > LOUD_LIMIT * 1.002 or np.count_nonzero(np.abs(written) >= 32767 / 32768) > 0:
                raise SystemExit(f"{name} : écrêtage ({db(file_peak):.2f} dBFS)")
            if np.max(np.abs(written[0] - written[-1])) > np.max(np.abs(np.diff(written, axis=0))):
                raise SystemExit(f"{name} : clic à la reprise de la boucle")
            s = stats[key]
            eq = min(stats["neutre"]["rms"] / s["rms"], LOUD_LIMIT / (s["peak"] * gain))
            files[key] = {"file": name, "seconds": round(s["seconds"], 3), "bpm": round(s["bpm"], 1),
                          "bars": [round(t, 4) for t in s["bars"]], "eq_gain": round(eq, 4)}
            report.append({"kit": "emotions", "profile": profile, "version": key, "measures": s["measures"],
                           "bars": [first, last - 1], "seconds": round(s["seconds"], 2),
                           "bpm": round(s["bpm"], 1), "peak_dbfs": round(db(file_peak), 2),
                           "rms_dbfs": round(db(s["rms"] * gain), 2),
                           "wrap_jump_over_max_step": round(s["wrap_jump"] / s["max_step"], 3),
                           "same_melody": round(s["same_air"], 3)})
        index["profiles"].append({"id": profile, "bars": [first, last - 1], "loop_bars": args.loop_bars,
                                  "neutral": base, "versions": files})
    (out / "index.json").write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    (out / "comparateur.html").write_text(EMOTIONS_HTML, encoding="utf-8")
    return report


EMOTIONS_HTML = """<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><title>Harmonium — comparateur des 4 émotions</title>
<style>
body{font-family:system-ui,sans-serif;max-width:760px;margin:2em auto;padding:0 1em;line-height:1.5}
button{font-size:1.1em;padding:.4em 1.2em;margin-right:.5em}
fieldset{margin:.8em 0;padding:.4em .9em}
label{font-size:1.1em;margin-right:1.2em;cursor:pointer}
input[type=radio],input[type=checkbox]{transform:scale(1.3);margin-right:.4em}
#etat,#tempos{color:#555;margin-top:.8em}
</style></head><body>
<h1>Comparateur des 4 émotions</h1>
<p>Le même air (même graine, mêmes mesures) tourne en boucle. Choisis une émotion et un niveau
(<b>bas</b> / <b>neutre</b> / <b>haut</b>) ; les trois autres restent au neutre. Le tempo change avec
l'arousal : les boucles n'ont pas la même durée, donc <b>une seule version joue à la fois</b> et la
nouvelle reprend à la <b>même position musicale</b> (même mesure, même fraction de mesure).</p>
<p><label for="profil">Air : </label><select id="profil"></select></p>
<fieldset><legend>Émotion</legend><div id="emotions"></div></fieldset>
<fieldset><legend>Niveau</legend><div id="niveaux"></div></fieldset>
<fieldset><legend>Réglages</legend>
<label><input type="radio" name="bascule" value="mesure" checked>Bascule au début de la mesure suivante</label>
<label><input type="radio" name="bascule" value="tout">Bascule immédiate (même fraction de mesure)</label><br>
<label><input type="checkbox" id="egal">Volume égalisé (sinon la force fait partie de l'effet)</label></fieldset>
<p><button id="jouer">▶ Jouer</button><button id="arreter">■ Arrêter</button></p>
<div id="etat">Chargement…</div><div id="tempos"></div>
<script>
const ctx = new AudioContext();
let index, prof = null, buffers = {}, active = null, pending = null, playing = false;
const $ = id => document.getElementById(id);
const FADE = 0.012;
const checked = name => document.querySelector('input[name=' + name + ']:checked').value;
const keyOf = () => checked('niveau') === 'neutre' ? 'neutre' : checked('emotion') + '-' + checked('niveau');
const gainOf = key => $('egal').checked ? prof.versions[key].eq_gain : 1;
function commit(now) {
  if (pending && now >= pending.when) { active = pending; pending = null; }
}
function position(voice, now) {
  const v = prof.versions[voice.key];
  return (((voice.offset + now - voice.when) % v.seconds) + v.seconds) % v.seconds;
}
function locate(v, t) {
  let i = 0;
  while (i < v.bars.length - 2 && t >= v.bars[i + 1]) i++;
  return [i, (t - v.bars[i]) / (v.bars[i + 1] - v.bars[i])];
}
function startVoice(key, when, offset) {
  const src = ctx.createBufferSource(), gain = ctx.createGain();
  src.buffer = buffers[key]; src.loop = true; gain.gain.value = 0;
  src.connect(gain); gain.connect(ctx.destination);
  src.start(when, offset);
  gain.gain.setTargetAtTime(gainOf(key), when, FADE);
  return {key, src, gain, when, offset};
}
function stopVoice(voice, when) {
  voice.gain.gain.setTargetAtTime(0, when, FADE);
  voice.src.stop(when + 0.25);
  voice.timer = setTimeout(() => voice.gain.disconnect(), (when - ctx.currentTime + 0.5) * 1000);
}
function cancelPending(now) {
  if (!pending) return;
  pending.src.stop(); pending.gain.disconnect(); pending = null;
  clearTimeout(active.timer); active.src.stop(now + 7200);
  active.gain.gain.cancelScheduledValues(now);
  active.gain.gain.setValueAtTime(gainOf(active.key), now);
}
function select() {
  const key = keyOf();
  showTempos();
  if (!playing) return;
  let now = ctx.currentTime; commit(now); cancelPending(now);
  if (key === active.key) return;
  const from = prof.versions[active.key], to = prof.versions[key];
  let when = now + 0.005, offset;
  const [bar, phase] = locate(from, position(active, when));
  if (checked('bascule') === 'tout') {
    offset = to.bars[bar] + phase * (to.bars[bar + 1] - to.bars[bar]);
  } else {
    when += (1 - phase) * (from.bars[bar + 1] - from.bars[bar]);
    offset = to.bars[bar + 1];
  }
  offset %= to.seconds;
  pending = startVoice(key, when, offset);
  stopVoice(active, when);
}
function showTempos() {
  const e = checked('emotion'), v = prof.versions;
  const label = k => v[k].bpm.toFixed(0) + ' BPM, ' + v[k].seconds.toFixed(1) + ' s';
  $('tempos').textContent = e + ' — bas : ' + label(e + '-bas') + ' · neutre : ' + label('neutre') +
    ' · haut : ' + label(e + '-haut') + ' (' + prof.loop_bars + ' mesures chacune)';
}
function refresh() {
  if (!playing) return;
  const now = ctx.currentTime; commit(now);
  const v = prof.versions[active.key], [bar] = locate(v, position(active, now));
  $('etat').textContent = 'Joue : ' + active.key + ' — mesure ' + (bar + 1) + '/' + prof.loop_bars +
    ' (' + v.bpm.toFixed(0) + ' BPM)' + (pending ? ' → ' + pending.key + ' au prochain début de mesure' : '');
}
async function load(id) {
  stop();
  prof = index.profiles.find(p => p.id === id);
  $('etat').textContent = 'Chargement de ' + id + '… (9 versions)';
  buffers = {};
  await Promise.all(Object.entries(prof.versions).map(async ([k, v]) =>
    buffers[k] = await ctx.decodeAudioData(await (await fetch(v.file)).arrayBuffer())));
  $('etat').textContent = 'Prêt : mesures ' + prof.bars[0] + ' à ' + prof.bars[1] + ' de l\\'air.';
  showTempos();
}
function play() {
  if (playing || !prof) return;
  ctx.resume();
  active = startVoice(keyOf(), ctx.currentTime + 0.1, 0);
  playing = true; refresh();
}
function stop() {
  if (!playing) return;
  [active, pending].forEach(v => { if (v) { v.src.stop(); v.gain.disconnect(); } });
  active = pending = null; playing = false;
}
function radios(box, name, items, current) {
  items.forEach(([value, text]) => {
    const l = document.createElement('label');
    l.innerHTML = '<input type="radio" name="' + name + '" value="' + value + '"' +
      (value === current ? ' checked' : '') + '>' + text;
    l.querySelector('input').addEventListener('change', select); box.appendChild(l);
  });
}
(async () => {
  index = await (await fetch('index.json')).json();
  radios($('emotions'), 'emotion', index.emotions.map(e => [e.id, e.label]), 'arousal');
  radios($('niveaux'), 'niveau', index.levels.map(l => [l, l]), 'neutre');
  document.querySelectorAll('input[name=bascule]').forEach(r => r.addEventListener('change', () => {}));
  $('egal').addEventListener('change', () => { if (playing) {
    const now = ctx.currentTime; commit(now);
    [active, pending].forEach(v => v && v.gain.gain.setTargetAtTime(gainOf(v.key), now, FADE)); } });
  const sel = $('profil');
  index.profiles.forEach(p => sel.add(new Option(p.id.replaceAll('_', ' '), p.id)));
  sel.addEventListener('change', () => load(sel.value));
  $('jouer').onclick = play; $('arreter').onclick = stop;
  setInterval(refresh, 100);
  await load(sel.value);
})();
</script></body></html>
"""


def build_fiddles(args: argparse.Namespace, tmp: Path) -> list[dict]:
    """Page d'écoute synchronisée du comparatif de violons (`make fiddle-compare`), numéros seuls."""
    del tmp
    tunes = sorted(p for p in args.fiddle_dir.iterdir() if p.is_dir() and any(p.glob("0*.wav")))
    if not tunes:
        raise SystemExit(f"{args.fiddle_dir} : lancer d'abord `make fiddle-compare` dans le labo")
    index = {"profiles": [{"id": t.name, "files": {w.stem: f"{t.name}/{w.name}"
                                                   for w in sorted(t.glob("0*.wav"))}} for t in tunes]}
    (args.fiddle_dir / "index.json").write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n",
                                                encoding="utf-8")
    (args.fiddle_dir / "comparateur.html").write_text(FIDDLE_HTML, encoding="utf-8")
    return [{"kit": "violons", "profile": t.name, "versions": len(index["profiles"][i]["files"])}
            for i, t in enumerate(tunes)]


FIDDLE_HTML = """<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><title>Harmonium — comparateur de violons</title>
<style>
body{font-family:system-ui,sans-serif;max-width:720px;margin:2em auto;padding:0 1em;line-height:1.5}
button{font-size:1.1em;padding:.4em 1.2em;margin-right:.5em}
label{display:inline-block;font-size:1.3em;margin:.3em .8em .3em 0;cursor:pointer}
input[type=radio]{transform:scale(1.4);margin-right:.4em}
#etat{color:#666;margin-top:1em}
</style></head><body>
<h1>Comparateur de violons</h1>
<p>Le même vrai reel joue en boucle avec tous les violons en même temps ; on n'en entend qu'un.
Change de numéro : le violon change au même moment de la musique. Les 8 premières mesures sont le
violon seul, puis guitare et basse entrent. Même volume pour chaque violon. Note ton numéro préféré
<b>avant</b> d'ouvrir <code>correspondance.md</code>.</p>
<p><label for="air" style="display:inline">Air : </label><select id="air"></select></p>
<div id="violons"></div>
<p><button id="jouer">▶ Jouer</button><button id="arreter">■ Arrêter</button></p>
<div id="etat">Chargement…</div>
<script>
const ctx = new AudioContext();
let index, voices = [], names = [], playing = false;
const etat = document.getElementById('etat');
function chosen() { const r = document.querySelector('input[name=v]:checked'); return r ? names.indexOf(r.value) : 0; }
function apply() { const c = chosen(), t = ctx.currentTime;
  voices.forEach((v, i) => { v.gain.gain.cancelScheduledValues(t); v.gain.gain.setTargetAtTime(i === c ? 1 : 0, t, 0.015); });
  if (playing) etat.textContent = 'Violon n° ' + names[c]; }
function stop() { voices.forEach(v => { if (v.src) { v.src.stop(); v.src.disconnect(); v.src = null; } }); playing = false; }
async function load(id) {
  stop(); const p = index.profiles.find(x => x.id === id); names = Object.keys(p.files).sort();
  etat.textContent = 'Chargement…';
  const box = document.getElementById('violons'); box.innerHTML = '';
  names.forEach((n, i) => { const lab = document.createElement('label');
    lab.innerHTML = '<input type="radio" name="v" value="' + n + '"' + (i ? '' : ' checked') + '>' + n;
    box.appendChild(lab); lab.querySelector('input').addEventListener('change', apply); });
  const buffers = await Promise.all(names.map(async n => ctx.decodeAudioData(await (await fetch(p.files[n])).arrayBuffer())));
  voices = buffers.map(b => ({buffer: b, gain: ctx.createGain(), src: null}));
  voices.forEach(v => { v.gain.gain.value = 0; v.gain.connect(ctx.destination); });
  etat.textContent = 'Prêt : ' + names.length + ' violons.';
}
function play() { if (playing || !voices.length) return; ctx.resume(); const at = ctx.currentTime + 0.1;
  voices.forEach(v => { v.src = ctx.createBufferSource(); v.src.buffer = v.buffer; v.src.loop = true;
    v.src.connect(v.gain); v.src.start(at); }); playing = true; apply(); }
(async () => {
  index = await (await fetch('index.json')).json();
  const sel = document.getElementById('air');
  index.profiles.forEach(p => sel.add(new Option(p.id.replace(/-[0-9a-f]{8}$/, '').replaceAll('-', ' '), p.id)));
  sel.addEventListener('change', () => load(sel.value));
  document.getElementById('jouer').onclick = play; document.getElementById('arreter').onclick = stop;
  await load(sel.value);
})();
</script></body></html>
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/harmonium-cli")
    parser.add_argument("--soundfont", type=Path, default=ROOT / "assets/GeneralUser-GS.sf2")
    parser.add_argument("--feet-soundfont", type=Path, default=ROOT / "assets/harmonium_trad.sf2")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=int, default=3, help="première seed essayée")
    parser.add_argument("--seeds", type=int, default=10,
                        help="nombre de seeds essayées par kit (on garde la plus audible)")
    parser.add_argument("--bars", type=int, default=66)
    parser.add_argument("--set-bars", type=int, default=300)
    parser.add_argument("--loop-bars", type=int, default=16, help="longueur de la boucle du comparateur")
    parser.add_argument("--kits", default="ab,progression,comparateur,sets",
                        help="kits à produire, séparés par des virgules (aussi : violons, "
                             "comparateur-marque, emotions)")
    parser.add_argument("--fiddle-dir", type=Path,
                        default=ROOT.parent / "harmonium_lab/target/fiddle_compare",
                        help="sortie de `make fiddle-compare` (kit violons)")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="harmonium-kits-") as name:
        tmp = Path(name)
        builders = {"ab": build_ab_kits, "progression": build_progression,
                    "comparateur": build_comparator, "comparateur-marque": build_comparator_marked,
                    "sets": build_sets, "violons": build_fiddles, "emotions": build_emotions}
        report = [row for kit in args.kits.split(",") for row in builders[kit](args, tmp)]
    path = args.output / "kits.json"
    produced = {row["kit"] for row in report}
    kept = [row for row in json.loads(path.read_text(encoding="utf-8")) if row["kit"] not in produced] \
        if path.exists() else []  # un kit régénéré remplace ses lignes, les autres restent
    path.write_text(json.dumps(kept + report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    for row in report:
        print(row)
    return 0


if __name__ == "__main__":
    sys.exit(main())
