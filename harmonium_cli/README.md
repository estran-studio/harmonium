# Harmonium CLI

REPL d'écoute et export offline du moteur Harmonium. Au démarrage, le profil
embarqué `reel_quebecois` est actif : 230 BPM à la noire, signature 2/2,
pulse dansée à la blanche et swing 0,56.

## Démarrage

```bash
cargo build --release -p harmonium-cli
cargo run -p harmonium-cli -- --style gigue_quebecoise
cargo run -p harmonium-cli -- --profile profiles/mon_style
```

`--style <id>` sélectionne un profil embarqué ; `--profile <dossier>` charge un
profil TOML validé avec ses banques. Les deux options sont exclusives.
`--soundfont <fichier.sf2>` choisit la banque audio. La résolution par défaut
cherche GeneralUser GS puis `assets/test.sf2` ; cette dernière ne permet pas
une validation des timbres traditionnels. `make soundfont` installe la banque GM.
`--feet-soundfont <fichier.sf2>` choisit la banque de pieds (défaut :
`assets/harmonium_trad.sf2` si présente, sinon le kit GM du canal 9).

`--osc` ouvre l'écoute OSC dans le REPL : un client UDP envoie
`/harmonium/params <arousal> <valence> <density> <tension>` (quatre flottants)
sur `127.0.0.1:8080` et les émotions changent, comme avec `emotion …`
(la commande passe par le même exécuteur que le REPL).

Le REPL pilote un `LiveHost` : chaque commande est analysée en `EngineCommand`
et exécutée par l'hôte, comme l'OSC et le studio.

## Commandes

### Profils et dynamique

```text
style reel_quebecois
style gigue_quebecoise
style valse_quebecoise
style profiles/mon_style
feet 0|1|2
intensity 0.8
seed 42
seed new
emotion
emotion 0.9 0.5 0.7 0.6
emotion a+0.1 v-0.2 d+0.05 t-0.1
direct
```

Les émotions sont arousal, valence, densité et tension. Elles agissent via
`[dynamics]` du profil actif, pas via des plages musicales codées dans le REPL.
La grille, la forme, les cellules, le modèle harmonique et les instruments se
règlent dans le profil. `reset` rétablit le profil embarqué par défaut.

### Transport, tempo et mixage

```text
set bpm 230
reset bpm
swing 0.6
tempo 200 8
tempo off
beat
seek 4
loop 4 8
loop off
set volume 0.8
set gain 0 0.8
set mute 3
set unmute 3
set route 2 0
set output mute
set output unmute
set rhythm off
set harmony on
set melody off
set lookahead 8
seek 2 soft
```

`set volume <0-1>` règle le volume d'écoute (amplitude linéaire, borné, rampe de
10 ms) : il agit sur la sortie audio, pas sur les enregistrements WAV/MIDI.
`set bpm` modifie le tempo nominal des prochaines mesures ; `tempo` pilote
le tempo audible à la prochaine pulse ou par rampe de N pulses (40–300 BPM).
`reset bpm` retire la surcharge nominale. Le swing est borné à 0,5–0,75 et
ignoré par la grille composée. `beat` lit `BeatInfo` ; Entrée, Échap, q ou
Ctrl+C reviennent au REPL. Les pistes sont Melody=0, Chords=1, Bass=2,
Heel=3 et Toe=4.

`seed new` tire une nouvelle graine au hasard (affichée) et repart de la mesure 1.
`seek <n>` rejoue le générateur jusqu'à la mesure n, de façon déterministe ;
`seek <n> soft` ne déplace que la tête de lecture (les mesures déjà composées
sont rejouées, rien n'est recomposé). `set rhythm|harmony|melody on|off`
active ou coupe un module de génération. `set route <canal> <banque>` envoie un
canal vers une banque SoundFont (`-1` = banque 0). `set output mute|unmute`
coupe la sortie sans arrêter la génération. `set lookahead <n>` règle
l'anticipation de génération (minimum 4 mesures).

### Enregistrement et état

```text
record wav
record midi
record musicxml
stop
stop musicxml
state
help
help style
quit
```

WAV et MIDI sont capturés au rendu. MusicXML est exporté depuis les mesures
complètes observées par le compositeur pendant la capture : pas de reconstruction
NoteOn/NoteOff, pas d'export de partition dans le callback audio. Les fichiers
finalisés sont écrits dans le répertoire courant. `state` affiche le profil,
la tonalité, la position dans la forme, le tempo audible, le swing et la pulse.
Les émotions relatives (`emotion a+0.1`) partent des émotions en vigueur sur
l'hôte, y compris celles posées par l'OSC. Tab complète les commandes et profils ; l'historique est dans `~/.harmonium_history`.

## Export offline

```bash
cargo run -p harmonium-cli --release -- --export --bars 64 --seed 1 \
  --record-wav demos/reel_seed1.wav --record-midi demos/reel_seed1.mid \
  --record-musicxml demos/reel_seed1.musicxml

cargo run -p harmonium-cli --release -- --export --bars 64 \
  --style valse_quebecoise --seed 1 --feet off
```

- `--bars N` borne la durée en mesures ; `--duration N` la borne en secondes.
- `--seed`, `--feet 0|1|2|off`, `--swing`, `--tours` et `--intensity` règlent le rendu.
- `--automation <fichier.toml>` applique les courbes par mesure.
- `--record-truth <fichier.json>` écrit un objet `{sample_rate, rendered_frames,
  bar_start_frames, measures, reports}` : `measures` porte les snapshots complets
  (décalages, gates et respirations d'interprétation compris), avec ou sans
  automation ; `reports` garde les rapports périodiques.

La génération et le playback sont identiques au temps réel, sans ouvrir de
périphérique audio. Voir `profiles/README.md` pour le format des profils.
