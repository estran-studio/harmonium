# G6c — Finitions Odin : morphing préparé hors du callback, avertissement de ressources

Suite de G6b (section « Limites »). Aucun flux de snapshots modifié : notes des 10 profils
distribués et goldens inchangés, `TUNING_VERSION` inchangé.

## Ce qui a changé

1. **Morphing préparé sur le thread principal.** Le compositeur connaît la banque des
   presets de l'hôte (`OdinBank`, socles analysés à la construction) et, à chaque
   changement d'émotion ou de profil, calcule le preset morphé de chaque piste
   (`MusicComposer::odin_patches`). Il l'envoie au callback comme charge `Copy`
   (`PlaybackCommand::SetOdin(OdinPatches)` → `AudioEvent::SetOdin`) ; le callback copie la
   charge dans le moteur (`odin_voices::preset_config`) : aucun calcul, allocation ni
   verrou. `EmotionUpdate` / `SetEmotions`, `OdinTrackDesc`, `PlaybackParams.odin` et le
   calcul côté audio sont supprimés (un seul chemin, principe IV).
2. **Avertissement au chargement.** `StyleProfile::missing_resources(&HostResources)`
   (`harmonium_core::host_resources`) liste ce que le profil désigne et que l'hôte n'a pas
   chargé : banque de percussion (`instruments.percussion_bank`, comparée aux banques que
   `SynthBackend::percussion_banks` annonce : 128 si le kit GM existe, 1 si la banque de
   pieds s'est chargée) et presets Odin déclarés. `MusicComposer::set_tuning` /
   `set_host_resources` journalisent un `WARN` clair par ressource (une fois par profil
   tant que l'absence ne change pas) ; `missing_resources()` l'expose (`LiveHost`
   aussi). Le CLI (export, démarrage REPL, commande `style`) l'imprime sur la sortie
   standard car `HARMONIUM_CLI` coupe le journal sur stderr. Aucun repli implicite ajouté.
3. **Conversion émotions → paramètres isolée.** Tout est dans `harmonium_core/src/odin.rs` :
   `OdinVoiceParams` (socle et valeur morphée = un seul type), `slot_mut` (unique table
   paramètre → champ), `clamp`. Les côtés GPL se réduisent à deux fonctions : lecture
   `odin_bank::read_preset`, écriture `odin_voices::preset_config`. Ajouter drive ou
   effets : un champ + sa borne, une variante `OdinMorphParam` + son bras `slot_mut`,
   et une ligne dans chacune des deux fonctions (mode d'emploi en doc de
   `OdinVoiceParams`). Schéma des profils inchangé.

## Choix et pourquoi

- **Charge par piste = `Option<OdinTrackPatch>`** (paramètres morphés, champs discrets,
  rang du preset dans la banque) : le rang permet au callback de couper les notes quand une
  piste change de preset sans comparer de noms ni allouer.
- **Analyse des presets hors de `SynthBackend`** (`odin_bank::load_bank`, appelé par
  `build_engine`) : `SynthBackend::new` perd son paramètre `odin_presets` (cutover, tous les
  appelants migrés). Les API d'hôte (`create_offline_engine`, `LiveHost::start`…) gardent
  leurs signatures.
- **Sans `odin`** la banque est vide : les presets déclarés sont signalés comme non chargés,
  ce qui est exact (build sans synthé).
- **Compositeur hors hôte audio** (`host_resources = None`) : rien n'est vérifié ni morphé.
- **Émotions hors mode émotion** : le contrôleur garde les valeurs par défaut, comme avant.

## Fichiers

Core : `odin.rs`, `host_resources.rs` (nouveau), `events.rs`, `params.rs`, `lib.rs`.
Audio : `odin_bank.rs` (nouveau), `odin_voices.rs`, `backend/{synth_backend,midi_backend}.rs`,
`voice_manager.rs`, `lib.rs`. Host : `composer.rs`, `audio.rs`, `live_host.rs`, `playback.rs`.
CLI : `export.rs`, `main.rs`, `repl.rs`. Docs : `profiles/README.md`.

## Preuves

- `make quality` vert (fmt, clippy, tests sans `odin`, studio, contrat `odin`).
- Sans `odin` : `host_resources_tests` (5) — avertissement de banque de percussion
  (reel québécois sans banque de pieds), absence d'avertissement avec banque, avertissement
  de preset Odin pour un autre style (valse 3/4), une seule fois par profil, **snapshots
  identiques avec ou sans banque**. `odin_morph_tests` (core, 4) : valeurs exactes du
  morphing et bornes, charges préparées seulement pour les presets chargés, déterminisme.
  `synth_backend` : `percussion_banks()`.
- Avec `odin` : `odin_contract_tests` (6) sous `RTCheckAllocator` : morphing déterministe,
  chaque émotion change le rendu sans changer les notes, **changement d'émotion en cours de
  lecture** (nouvelle charge `SetOdin` appliquée dans le callback sans allocation), liaisons
  `Tied`, coupure de piste.
- Non-régression : `nt_desert`, seed 1, 32 mesures, presets de smoke : MIDI et WAV
  **identiques octet pour octet** (`cmp`) entre HEAD et cette branche (mêmes émotions → même
  morphing). Reel québécois avec/sans banque de pieds : MIDI identique à HEAD.
- Smoke CLI : `nt_desert` sur `test.sf2` imprime les 3 avertissements (banque 128, presets
  `nappe` et `basse_saturee`) ; reel québécois sans banque de pieds imprime l'avertissement
  banque 1, pas avec `--feet-soundfont`.

## Principe VIII

Le mécanisme (preset morphé par piste) est employé par `nt_desert`, `nappe_pad` (4/4) et
`valse_nappe` (3/4) ; l'avertissement par un profil à kit (`nt_desert`), un reel à banque 1
et la valse.

## Limites

- Une banque de percussion autre que 128/1 est toujours signalée absente : l'hôte ne
  charge que ces deux.
- `NativeHandle::start` ne charge toujours pas de banque de pieds.
- Le moteur d'`odin2-core` n'expose ni drive ni effets : à venir avec le portage, via la
  procédure ci-dessus.
- L'avertissement du `set_host_resources` initial porte sur le profil intégré ; hors CLI,
  seul le journal le montre.

## Ce qu'il faut écouter

`nt_desert` avec `--odin-presets` : rendu identique à G6b (basse et nappe), puis changer
valence/tension au REPL (`--features odin`) : la brillance suit toujours, maintenant calculée
côté principal. Lancer un profil trad sans `--feet-soundfont` : message d'avertissement, pieds
silencieux comme annoncé.
