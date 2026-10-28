# G6d — Branchement du vrai synthé Odin 2 (odin-rs 4718189)

`harmonium` utilisait l'ancien `odin2-core` « simplifié analogique » (0e8f08b) : un
`PresetConfig` réduit à 3 oscillateurs + 1 filtre + enveloppes, sans effets. Le port
fidèle d'Odin 2 publié sur `estran-studio/odin-rs` est branché ici : chemin de voix
complet, matrice de modulation, effets (distorsion, delay, phaser, flanger, chorus,
réverb), parseur ValueTree complet et API hôte générique.

## Ce qui a changé

1. **Dépendance épinglée.** `harmonium_audio/Cargo.toml` pointe
   `https://github.com/estran-studio/odin-rs` au commit `4718189` ; `cargo update
   -p odin2-core` remplace `0e8f08b`. Features inchangées (`optional`, `odin =
   ["dep:odin2-core"]`) : sans `odin`, aucune source GPL n'est compilée et aucun
   fichier `.odin` n'entre au dépôt. `4718189` rend `OdinEngine: Send` en Rust sûr
   (les tables dessinées partagées passent de `Rc` à `Arc`) — prérequis au transfert
   de l'instance vers le callback CPAL.
2. **Chargement des presets complets.** `odin_bank::load_bank` décode chaque
   ValueTree (`OdinPreset::from_bytes`) et construit l'`EngineConfig` complet, qui
   conserve types d'oscillateurs, routage, matrice, effets, unissons et octaves.
   Un preset illisible, à nom invalide, à matrice rejetée ou portant un paramètre non
   fini est ignoré avec un avertissement (aucun repli implicite) ; son socle
   (`OdinVoiceParams`) et sa configuration partent ensemble ou pas du tout.
3. **Morphing sur le catalogue, pas sur un sous-ensemble.** `harmonium_core::odin`
   expose `OdinMorphParam` (45 identifiants stables du catalogue, serde exact :
   `osc{1,2,3}_{vol,detune}`, `fil{1,2,3}_{freq,res,env,saturation,gain}`,
   `env{1..4}_{attack,decay,sustain,release}`, `master`, `dist_boost`,
   `dist_drywet`, `delay_wet`, `phaser_drywet`, `flanger_drywet`, `chorus_drywet`,
   `rev_drywet`) avec `ALL`, `COUNT`, `index`, `name`, `bounds`.
   `OdinVoiceParams` est un tableau fixe `[f32; COUNT]` indexé par variante, sans
   dépendance GPL. `OdinDiscreteParams` et l'ancien `PresetConfig` disparaissent.
4. **Frontière GPL.** L'application des émotions passe par `OdinEngine::set_parameters`
   (un seul recalcul DSP par lot), jamais par une reconstruction de configuration ;
   les sélections non audio (type d'oscillateur, forme dessinée, ordre d'effets,
   unisson) restent celles du preset usine au chargement. Le tempo du transport est
   poussé au moteur (`set_tempo`) pour les LFO et effets synchronisés.
5. **Instances hors du callback.** `MusicComposer::send_odin` remplace l'ancien
   `Push(SetOdin(patches))` appauvri : une seule transaction `OdinUpdate` transporte
   les paramètres `Copy` et, seulement pour les pistes dont le rang de preset change,
   l'instance déjà chargée (`OdinEngine::new` + `load_config`, sur le thread
   principal). Le callback échange les instances (`mem::swap`) puis renvoie la boîte
   au compositeur par SPSC ; il ne fabrique ni ne détruit aucune instance. Une file
   pleine refuse la commande avant préparation lourde, et l'état « rang envoyé »
   n'avance qu'après insertion réussie.

## Choix et pourquoi

- **Une commande atomique, pas un train séparé.** Mettre paramètres et instances dans
  la même boîte `SetOdin` conserve l'ordre face aux autres commandes du ring et rend
  la bascule SoundFont↔Odin ou preset↔preset indivisible. La file de retour pleine
  suspend le dépilage des commandes (une boîte en attente) : aucune libération de
  moteur dans le callback, aucune perte d'installation.
- **Socle référencé par identifiant stable.** `ParameterId` est résolu une fois au
  chargement de la banque (et dans `OdinMixer::new`) ; le morphing ne manipule que des
  indices de variante et des `f32`. La borne physique de chaque paramètre est
  dupliquée dans `OdinMorphParam::bounds` (le catalogue peut changer de version) et
  vérifiée échantillon-par-échantillon par un test isolé contre le descripteur réel.
- **Boost de l'ancien moteur retiré.** Le « +17 dB » de G7d calibré sur le moteur
  simplifié n'est pas appliqué à `nt_desert` : avec les presets d'usine mesurés, le
  niveau crête reste à −2,71 dBFS sans compensation ; le master d'origine des presets
  (`master` usine) est conservé.

## Fichiers

Audio : `Cargo.toml`, `odin_bank.rs` (chargement complet + transaction + expéditeur),
`odin_voices.rs` (mixeur RT : `set_parameters`, plus de `preset_config`),
`backend/{mod,recorder,synth_backend,wav_backend,midi_backend}.rs` (trait
`apply_odin`), `voice_manager.rs`, `lib.rs`. Host : `audio.rs` (câblage des rings),
`composer.rs` (`send_odin`, dispatch), `playback.rs` (commande `SetOdin` propriétaire,
pression de retour), `live_host.rs`. Core : `odin.rs`, `style_profile.rs` (validation
bordures/unicité), `events.rs` (suppression de `SetOdin`/l'ancienne voie). CLI :
`export.rs`. Données : `profiles/nt_desert/profile.toml` (presets usine, suppression
du boost), `profiles/nt_desert/README.md`, `tests/fixtures/profiles/{nappe_pad,
valse_nappe}/profile.toml`, `profiles/README.md`. Compte rendu : `g6d-odin2.md`.

## Preuves

- **Contrat `odin` (RTCheckAllocator actif, debug) : 9 tests verts.** Déterminisme du
  morphing sans changement des snapshots/MIDI, changement d'émotion en vol, presets
  audibles distincts, tenues `Tied`, coupure de piste, **bascule SF↔Odin et
  remplacements multiples en cours de lecture** (reprise après file pleine, notes non
  orphelines), rejet d'un preset à paramètre non fini avant le callback. Tous les
  rendus passent par `process_buffer` sous la garde de l'allocateur de test.
- **Fidélité du chemin hôte.** Rendu isolé (compositeur `send_odin` → `apply_update` →
  `set_parameters` → `render_add`) comparé au moteur `odin2-core` direct : écart
  absolu ≤ 7,7e-7 sur `Sausage Bassener` et `Darken Nature [RM]`. Contribution
  effets+distorsion (référence sèche vs complète) : L2 relatif 1,12 et 1,54 — les
  effets du vrai moteur sonnent, ce qui restait inaudible sur l'ancien moteur.
- **Notes inchangées.** Rendu `--profile profiles/nt_desert --seed 1 --bars 32` avec
  et sans `odin` (mêmes émotions) : 79 `MeasureSnapshot` identiques et MIDI identique
  octet pour octet (SHA256 `2ea820c9…`). Aucun bump de `TUNING_VERSION`, aucun golden
  régénéré.
- **Niveaux réels (float32, aucun écrêtage dans l'encodeur).** Rendu usine 32 mesures :
  pic Odin −2,71 dBFS / RMS −19,67 dBFS, 0 échantillon ≥ 1 ; pic SF2 −5,69 dBFS.
  À niveau RMS égal (atténuation seule, marge commune de 1 dB), Odin rend 2,54 dB plus
  fort que le programme SoundFont du même profil ; l'écoute comparative s'entend sur
  `ecoute_g7/g6d_comparaison_{odin,sf2}.wav`.
- **Compilation wasm32** : `make wasm/check` passe sans (`harmonium`) et avec
  (`--features odin`) le synthé — le moteur reste sans fil ni `Rc` au chargement.
- `make quality` vert (fmt + clippy + tests, hors et avec `odin`).

## Rendus d'écoute (hors dépôt)

`harmonium_lab/corpus/nuclear_throne/ecoute_g7/` :
- `g6d_odin.wav` — 32 mesures `nt_desert` (Sausage Bassener + Darken Nature [RM]).
- `g6d_sf2.wav` — même partition en SoundFont (`nt_desert.sf2`).
- `g6d_comparaison_odin.wav` / `g6d_comparaison_sf2.wav` — comparaison à niveau égal.
- `g6d_basse_transition.wav` — solo de basse (piste `Bass`) sur les mesures 7–10,
  le passage A1 → A2 (pédale G# → riff D/D#).
- `g6d_niveaux.json` — mesures crête/RMS des fichiers ci-dessus.

## Limites

- La réverb `SurgeReverb` amont n'est pas portée (l'amont ne l'emploie pas) ; l'arp
  `Random` et les tirages de phase du port sont déterministes (graine fixe) là où le
  C++ tirait du hasard système.
- Les sélections non audio (types d'oscillateur, formes dessinées, ordre d'effets,
  unisson, mode de jeu) ne se morphent pas : un changement de preset les applique au
  chargement ; les émotions ne pilotent que les paramètres numériques du catalogue.
- La bascule de preset coupe les voix du preset précédent (contrat G6c) : une tenue
  Tied prolongeant une hauteur à cheval sur un changement de preset ne réattaque pas
  tant que l'accord n'a pas changé (comportement voulu, pas une note coincée).
- La destruction des moteurs encore installés s'effectue avec la destruction du
  `PlaybackEngine`/de la closure CPAL ; aucune libération n'a lieu dans `process_buffer`.

## Ce qu'il faut écouter

Basse saturée de `Sausage Bassener` (distorsion, sustain) et nappe `Darken Nature [RM]`
(réverb, chorus, fondu lent), sans l'ancien moteur qui lissait tout : la basse doit
suivre le riff chromatique de la section A2 avec du grain, la nappe doit respirer dans
le pont. Changer `arousal`/`tension` au REPL : le filtre et la brillance suivent sans
réattaque des nappes. Comparer `g6d_comparaison_odin.wav` et `g6d_comparaison_sf2.wav`
à niveau égal pour juger le gain à choisir dans `[mix]`, plutôt que le master d'usine.