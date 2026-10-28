# G6b — Synthé Odin par piste (option `odin`) et routage des pieds par le profil

Décision du propriétaire (constitution 4.2.0) : SoundFont par défaut ; synthé Odin
(`odin2-core`, GPLv3) en option de compilation `odin` de `harmonium_audio`, désactivée
par défaut. Un binaire compilé avec `odin` est GPL ; 1837 compile sans. Aucun fichier
GPL (code ou `.odin`) n'est dans ce dépôt.

## Ce qui a changé

1. **Option `odin`** : `harmonium_audio/odin` (dépendance git `odin2-core`, optionnelle),
   relayée par `harmonium/odin` (host) et `harmonium-cli/odin`. Sans l'option, aucun code
   Odin n'est compilé.
2. **Instrument par piste, en données** : `StyleProfile.odin: BTreeMap<TrackId,
   OdinTrackSpec>` (`[odin.<Piste>]`) avec `preset` (nom) et `morph` (règles
   `param + gains arousal/valence/density/tension`). Le programme SoundFont reste requis
   (validé) : c'est le rendu sans `odin` ou sans preset fourni. Aucun `match` sur un nom
   de style ni canal fixe : la clé est le `TrackId` (`harmonium_core/src/odin.rs`).
3. **Presets fournis par l'hôte** : `OdinPresetSource { name, bytes }`
   (`harmonium_audio::odin_source`, sans dépendance GPL) passé à `SynthBackend::new`,
   `create_offline_engine`, `create_timeline_stream`, `NativeHandle::start` /
   `start_paused`. CLI : `--odin-presets <dossier>` (fichiers `*.odin`, nom = tige),
   sous `--features odin`.
4. **Temps réel** (`harmonium_audio/src/odin_voices.rs`) : 6 voix (une par `TrackId`) et
   tampon de rendu créés à la construction ; presets analysés hors callback ; le morphing
   est du `f32` pur sur un tableau `Copy`. `Tied` : le lecteur G5 n'émet pas de `NoteOn`
   pour une liaison, donc pas de réattaque Odin. `NoteOn` sur une hauteur active referme
   d'abord l'ancienne ; `AllNotesOff` et les bascules SoundFont ↔ Odin (changement de
   profil) coupent les deux côtés.
5. **Routage de la percussion corrigé** : `percussion_bank_override`,
   `configure_gm_defaults` et le `feet_loaded` sont supprimés. `apply_profile_programs`
   est l'unique chemin : banque `128` → kit GM canal 9 ; sinon canaux propres 3/4 sur la
   banque du profil. Les 5 profils trad à podorythmie (reel/gigue/valse québécois, reel
   acadien, gigue acadienne) déclarent `percussion_bank = 1` (banque `harmonium_trad.sf2`) ;
   `nt_desert` et la marche gardent 128 et jouent leur kit même quand la banque de pieds
   est chargée.
6. **Données** : `nt_desert` déclare `Bass → basse_saturee` et `Chords → nappe` (G7c
   conservé, seules ces déclarations ajoutées). Principe VIII : fixtures `nappe_pad`
   (4/4) et `valse_nappe` (3/4, autre style) emploient le même mécanisme sur `Chords`.

## Choix et pourquoi

- **Gains additifs par émotion, pas le morphing bilinéaire à 4 quadrants de l'ancien
  `morph.rs`.** Les 4 émotions du moteur sont arousal, valence, density, tension : quatre
  axes, pas le plan valence/arousal ; des règles par paramètre le disent en données.
  L'ancien backend (4 canaux fixes Bass/Lead/Snare/Hat) n'est pas repris (principe V).
- **Paramètres morphables = ceux de `PresetConfig`** (filtre, enveloppes, détune, volumes
  d'oscillateur, master). `odin2-core` n'expose ni drive ni effets dans cette
  configuration : le « drive » de l'ancien `modulation.rs` n'est pas atteignable.
- **Niveau** : le gain master du profil (+14 dB pour `nt_desert`) est calibré sur les
  samples SF2 ; appliqué à Odin il rendait +10 LUFS. Odin est mixé au niveau de piste du
  profil seulement ; le niveau propre est le `master` du preset.
- **Presets de smoke générés hors dépôt** (`/tmp/odin_presets`, octets au format `.odin`
  écrits par un script jetable) : aucun `.odin` n'est commité.

## Fichiers

Code : `harmonium_core/src/{odin,params,events,mapper,style_profile,lib}.rs`,
`timeline/mod.rs` (`TrackId: Ord`) ; `harmonium_audio/src/{odin_source,odin_voices,
voice_manager,lib}.rs`, `backend/{synth_backend,midi_backend}.rs`, `Cargo.toml` ;
`harmonium_host/src/{audio,composer,native_handle,playback,main}.rs`, `Cargo.toml` ;
`harmonium_cli/src/{main,export,repl}.rs`, `Cargo.toml` ; `harmonium_lab/src/render.rs`.
Données : `profiles/{nt_desert,reel_quebecois,gigue_quebecoise,valse_quebecoise,
reel_acadien,gigue_acadienne}/profile.toml`, `tests/fixtures/profiles/{nappe_pad,
valse_nappe}/profile.toml`. Docs : `profiles/README.md`.
Tests : `harmonium_core/tests/odin_profile_tests.rs`,
`harmonium_host/tests/odin_contract_tests.rs` (feature `odin`).

## Preuves

- Sans `odin` : `cargo test --workspace` vert (0 échec), notes des profils distribués et
  goldens inchangés, `TUNING_VERSION` inchangé (aucun flux de snapshots modifié).
- Avec `odin` : `cargo test -p harmonium --features odin --test odin_contract_tests` =
  5 tests : (a) une piste avec preset fourni produit du son, sans preset ou nom absent
  elle retombe sur SoundFont (silence ici, sans banque) ; (b) la valse 3/4 emploie le
  même mécanisme ; (c) morphing déterministe (deux rendus identiques) et chacune des 4
  émotions change le rendu sans changer les notes publiées ; (d) accords `Tied` : jamais
  une hauteur attaquée deux fois sans `NoteOff` (MIDI enregistré) ; (e) coupure de piste :
  aucune voix coincée. Le callback complet tourne sous `RTCheckAllocator` (panique sur
  toute allocation).
- Smoke : `cargo run -q -p harmonium-cli --release [--features odin] -- --export --bars
  32 --profile profiles/nt_desert --seed 1 --soundfont nt_desert.sf2 --feet-soundfont
  harmonium_trad.sf2 [--odin-presets /tmp/odin_presets]` : MIDI identiques octet pour
  octet (`cmp`, 2140 o) avec et sans Odin ; Odin −23,0 LUFS contre −19,8 LUFS SoundFont.
  Basse Odin seule : −32,8 LUFS. WAV dans
  `harmonium_lab/corpus/nuclear_throne/ecoute_g7/` : `odin_nt_desert_seed1.wav` et
  `sf2_nt_desert_seed1.wav` (témoin).

## Limites

- **Morphing calculé sur le fil audio, pas préparé sur le fil principal.** Il s'applique
  à la réception d'une commande `SetEmotions` (une copie `f32`, ≤ 8 règles) : aucune
  allocation ni verrou, vérifié sous `RTCheckAllocator`, mais pas pré-calculé hors
  callback comme demandé. Le pré-calcul demanderait que le fil principal connaisse les
  socles des presets.
- **Retour au SoundFont non audible sans banque** : si une banque de percussion
  demandée est absente (ex. profil trad sans `harmonium_trad.sf2`, avec GeneralUser
  seul), les pieds sont silencieux : l'ancien repli GM implicite est supprimé. **1837 doit
  déclarer `percussion_bank = 1`** dans ses profils à pieds. Pas d'avertissement au
  chargement pour l'instant.
- Le moteur Odin de `odin2-core` est simplifié : voix mono-signal analogique, pas de
  drive, d'effets ni de modulations du preset ; les `.odin` riches sonnent moins que dans
  Odin 2. Pas de pitch bend ni de sustain pédale.
- `NativeHandle::start` ne charge pas de banque de pieds (inchangé : `None`).
- Les presets de smoke sont synthétiques ; les presets d'usine du dépôt `odin-rs` ne sont
  pas livrés (aucun `.odin` n'y a été trouvé à jour).

## Ce qu'il faut écouter

`odin_nt_desert_seed1.wav` contre `sf2_nt_desert_seed1.wav` : pédale G# (basse scie
saturée-filtrée) et nappes du pont (attaque 0,35 s) ; la basse Odin doit tenir sans
réattaque aux barres (liaisons), la nappe ne doit pas pomper. Niveau Odin ~3 dB sous le
témoin. Changer valence/tension dans le REPL (`--features odin --odin-presets …`) :
la brillance de la basse et la nappe suivent.
