# Changements moteur demandés par harmonium_training (base : main)

Document reçu le 2026-10-30 de la session qui fait revivre `harmonium_training` (copie
conservée telle quelle ; décisions et ordre dans `plan.md`).

**Contexte.** harmonium_training devient un client externe de la crate harmonium
(`harmonium_host`) et suit main. Il pilote le moteur par une grille d'accords explicite, pas par
les émotions. L'entrée est MIDI d'abord, audio plus tard.

## A. Bloquants pour la v1 (impro MIDI sur une grille)

**E0 — Constitution : autoriser un client externe qui n'est pas un jeu**
- Constat : seuls le REPL, le studio et Bevy sont nommés comme hôtes, avec « un seul frontend »
  (constitution.md:100-116). La mission ne parle que de jeux pilotés par les émotions (:3-11).
- Changement : amender pour couvrir les clients externes qui utilisent l'API publique de
  harmonium_host hors du dépôt (training), pilotés par une grille plutôt que par les émotions.
  Aucun code de training n'entre dans le dépôt moteur.
- Acceptation : nouvelle version de la constitution (4.5.0) avec sa ligne d'amendement.

**E1 — Grille riche dans le modèle loop**
- Constats :
  - le modèle n'accepte que Major et Minor (models/chord_loop.rs:196) ;
  - les durées sont en mesures entières (chord_loop.rs:185-220) ;
  - le voicer renvoie au plus 3 hauteurs, `ArrayVec<u8, 3>` (trad/accomp.rs:451-479) ;
  - `Chord::from_name` rejette G7b9 et ne remplit pas `extensions` (theory/chord.rs:309-360,462-503).
- Besoin : le vocabulaire iReal (liste complète dans training,
  harmonium_practice/src/ireal.rs:224-283) :
  - accords de base : 6, maj7, m7, 7, m7b5, dim7, sus ;
  - extensions 9, 11, 13 et altérations b9, #9, #11, b13, alt ;
  - basse différente de la fondamentale (accords slash) ;
  - plusieurs accords par mesure.
- Changement : qualités en données, à partir de `theory::ChordType`, plus extensions et basse ;
  durées d'accord en pas (ou en fraction de mesure) ; voicer jusqu'à 4–5 hauteurs, sans
  allocation (ArrayVec sur la pile) ; parseur de symboles complet.
- Acceptation : un blues en fa en accords de 7ᵉ, un ii-V dans une seule mesure, C/E, G7b9 : les
  `chord_changes` tombent aux bons `start_step` sur 3 graines ; même résultat en natif et en
  wasm ; goldens des profils distribués inchangés.

**E2 — Tonalité verrouillée**
- Constats : la tonalité vient de `melody.key_modes` seulement à valence neutre, sinon de
  `dynamics.color` (trad/set.rs:424-446) ; le mode bascule avec la valence
  (trad/generator.rs:630-683) ; `UseDirectMode` ne fige rien (mapper.rs:65-105).
- Changement : un profil peut déclarer une tonique et un mode fixes que la valence ne modifie
  pas. La forme exacte des données reste à décider.
- Acceptation : valence balayée de −1 à 1, tonique et accords publiés identiques.
- La transposition ne demande pas de nouvelle commande : on reconstruit le profil et on l'envoie
  par `SetStyleProfile`, appliqué à la mesure suivante, sans seek ni reset de graine
  (harmonium_host/src/composer.rs:351-377).

**E3 — Identité d'accord complète dans le snapshot**
- Constats : `ChordContext` ne contient que `root_offset`, `is_minor`, `chord_name` et
  `scale_guidance` (timeline/mod.rs:179-199) ; le modèle loop publie toujours
  `scale_guidance = None` (trad/generator.rs:1921-1938).
- Pourquoi : la notation de l'impro dans training lit le symbole, la qualité, la fondamentale et
  les notes de l'accord (src/lib/state/scoring.svelte.ts:93-103, src/lib/scoring/classifier.ts).
- Changement : publier la qualité typée, les extensions, la basse et la fondamentale absolue
  (classe de hauteur) ; avec loop, publier `ScaleGuidance` (notes de l'accord, gamme) quand
  `scale_guidance = true`.
- Acceptation : rupture du contrat, donc bump de `TUNING_VERSION` et goldens régénérés dans le
  même commit (principe I).

**E4 — Notes jouées en direct (retour MIDI)**
- Constats : ni `EngineCommand` ni `PlaybackCommand` n'ont de note on/off (command.rs:13-120,
  playback.rs:125-193) ; `LiveMidiSender` a été supprimé (specs/015-studio/s2-hote-unique.md:68-71).
- Changement : commande note on/off (canal, hauteur, vélocité) jusqu'au callback ; voie séparée
  des pistes générées, avec son propre programme ; hors du snapshot (déterminisme intact) ; la
  même commande sert au studio (clavier MIDI) et aux jeux.
- Acceptation : `RTCheckAllocator` silencieux ; latence d'au plus un buffer audio ; snapshots
  identiques avec et sans notes jouées ; le moteur compile toujours en wasm.

## B. Bloquants pour un accompagnement crédible

**E5 — Accompagnement jazz en données**
- Constat : les voicers jazz (notes guides 3-7, block, comping) ont été supprimés en 35b5d92,
  alors que l'inventaire disait de les porter ou de les garder en réserve
  (specs/010-phase2-style-profiles/legacy-inventory.md:38). `theory/voice_leading.rs` n'a aucun
  consommateur.
- Changement : options d'accompagnement dans le profil (notes guides, voicings sans
  fondamentale, drop 2, motifs de comping), enchaînées par `voice_leading`.
- Acceptation : tests de contrat sur les notes de la piste Chords, puis écoute.

**E6 — Walking bass sur une grille qui change en cours de mesure**
- Constat : motifs et notes de passage existent pour loop et trad (trad/accomp_profile.rs:36-86,
  trad/bass_passing.rs). Le chemin jazz_functional n'utilise que `bass.hits` et joue la
  fondamentale (trad/generator.rs:930-953,993-1020).
- Changement : après E1, motifs référés à l'accord courant au pas près, avec approche vers la
  fondamentale suivante ; rendu de basse de jazz_functional unifié sur ce mécanisme (principe IV).
- Acceptation : quatre noires par mesure, arrivée sur la fondamentale à chaque changement
  d'accord, approche chromatique au 4ᵉ temps.

**E7 — Profil jazz/blues distribué**
- Principes V et VII : corpus avec manifeste de provenance, nom par source précise, validation
  au labo puis à l'écoute.
- Existe déjà : swing (`SetSwing`), patterns de batterie à plusieurs notes (ride, charleston ;
  trad/feet.rs, exemple profiles/nt_desert/feet.toml:16-89).
- Usage training : l'accompagnement joue sous la grille fournie par l'utilisateur, qui remplace
  l'harmonie du profil. Plus tard : banques de cellules jazz pour la lecture à vue.

## C. Non bloquants

- **E8 — Composition déclarative de profils** (principe VIII) : aucun champ du type `extends`,
  et le schéma refuse les champs inconnus (style_profile.rs:523-552). En attendant, training peut
  copier les couches dans la struct.
- **E9 — Doc** : le commentaire de command.rs:15-18 annonce 70 à 180 BPM, le moteur accepte
  40 à 300 (mapper.rs:112-118).

## D. Déjà dans main

- `LiveHost` s'utilise comme bibliothèque, banques fournies au démarrage (live_host.rs:80-130).
- Profil en mémoire avec `StyleProfile::from_toml` ou `from_files`, champs publics
  (style_profile.rs:702-721) ; `SetStyleProfile` s'applique à la mesure suivante.
- Tempo, swing, boucle, seek, graine, mélodie on/off, mute et gain par canal, lookahead,
  `transport_position`, `pump`, pause et reprise.
- Mélodie omissible (G4) ; parties de 1 à 255 mesures.
- `ClockReader` est `Send` et sans verrou (harmonium_host/src/clock.rs:159-177).
- Instruments : `[instruments]` du profil ; `SetChannelRoute` ne change que la banque, et le
  changement de programme s'applique immédiatement (live_host.rs:485-503).

## E. Côté training (hors moteur)

- Réécrire `harmonium_practice/src/engine.rs` sur `LiveHost` et `EngineCommand` : une grille
  devient un profil loop, une seule partie de N mesures, tonalité verrouillée.
- Garder les accords en cours de mesure à l'import iReal (aujourd'hui aplatis :
  ChartPresetPicker.svelte:89-90, ireal.rs:745-748).
- Retirer les sélecteurs d'algorithmes de l'ancien moteur, l'éditeur JSON de `TuningParams` et
  les modes « émotion » et « technique ».
- Pont wasm-bindgen propre à training (le studio n'expose que `run`,
  harmonium_studio/src/lib.rs:23-39).
- Compiler sans odin si training est vendu en code fermé (GPL).

## F. Décisions demandées à l'orchestrateur

1. E1 : symbole texte (« Dm7 ») ou structure `{semitones, type, extensions, bass}` ?
2. E4 : `EngineCommand` ou canal direct de `LiveHost` vers le moteur audio ? Critère : latence.
3. E7 : quel corpus de référence, sous quelle licence, quel nom de profil (principe VII).
