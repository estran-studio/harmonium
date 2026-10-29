# G2 — Glissando Odin pour l’articulation Legato

## Problème et décision

G7e faisait chevaucher les notes de 0,2 pas, mais Odin réattaquait chaque
hauteur. Le lecteur transmet désormais l’enchaînement effectif dans
`AudioEvent::NoteOn.legato` ; les instants, les snapshots, le MIDI et le
chevauchement SoundFont restent inchangés. Aucun changement de
`TUNING_VERSION`, de golden musical ni de profil distribué.

Un enchaînement `Legato` d’une autre hauteur tenue utilise le mode
mono/legato natif : nouveau `note_on` pendant la tenue, puis fermeture de
l’ancienne touche. Sa fin G7e différée est ignorée sur Odin, sans couper la
nouvelle hauteur ni laisser de touche ancienne dans la pile de rappel.
Les enveloppes d’amplitude, de filtre et de modulation ne sont pas
réattaquées. Première note, reprise après silence, même hauteur répétée et
note `Normal` : vraie attaque, glide coupé. `Tied` reste une prolongation
sans événement d’attaque.

## API Odin vérifiée et absence de modification tierce

Dépendance conservée : `odin-rs` **4718189**. Dans `odin2-core` :

- `EngineConfig.play_mode` expose `Poly`, `Mono` et `Legato` ;
  `load_config` prépare les états et alloue, donc reste hors callback.
- `ParameterId::from_name("glide")` et `set_parameter` exposent le glide,
  sans allocation ni remise à zéro des enveloppes.
- Le catalogue de `ParameterId` **n’expose pas le mode de voix**. Recharger
  une configuration dans le callback serait à la fois allouant et destructif.
- `OdinEngine::start_note` alloue les voix fixes en mode non polyphonique ;
  `Voice::start(..., legato=true)` utilise `trigger_legato`, qui conserve une
  enveloppe tenue et réattaque après son relâchement.
- `note_on` suivi de `note_off` de l’ancienne touche ne relâche pas la
  nouvelle voix : sa hauteur a déjà été remplacée.

Contournement côté Harmonium, sans modifier le dépôt tiers : le compositeur
repère les pistes dont les données musicales contiennent du `Legato`
(actuellement les motifs de `Bass`, unique source de cette articulation).
Le dispatch prépare ces moteurs en `PlayMode::Legato` avant leur envoi SPSC.
Le mode fait partie de l’identité d’installation : changer de profil avec le
même preset ne réutilise pas une instance préparée dans le mauvais mode.
Une attaque `Normal` relâche explicitement la tenue mono avant `note_on`.
Les pistes sans `Legato` gardent leur mode, leur glide de preset et leur
chemin historique. Aucun interrupteur supplémentaire ni nom de style en code.

Aucun changement dans `odin-rs`, `harmonium_studio/`, `harmonium_bevy/` ou
les sections de zones des profils distribués. Le callback ne construit,
ne détruit ni n’alloue de moteur ; aucun verrou ajouté.

## Un seul réglage en données

`glide_ms` appartient à `[[odin.<Piste>.zones]]`, comme le niveau et le
morphing de T1. Défaut **80 ms** ; valeur finie, **0** pour désactiver le
glissement de hauteur sans réattaquer l’enveloppe, sinon **40 à 500 ms**.
Erreur localisée sur `odin.<Piste>.zones[i].glide_ms`.

Il s’agit du temps **nominal pour parcourir 90 % de l’écart fréquentiel à
44 800 Hz**, pas d’une rampe linéaire terminée à cette échéance. Le moteur
natif fait `f = cible × (1 − c) + f × c`, avec
`c = 0,9985 + 0,0014 × glide`. La banque inverse cette relation hors callback ;
Odin applique ensuite sa propre adaptation du contrôle au taux
d’échantillonnage. Le temps réellement mesuré peut donc différer du nominal.
La plage évite les extrêmes inatteignables du coefficient natif au taux de
référence. Le réglage ne dépend pas du tempo et s’applique au prochain
enchaînement, sans second chemin de morphing du glide.

Limites natives conservées : les phases/LFO suivent les règles d’Odin ; le
legato garantit la continuité d’enveloppe, pas l’immobilité de tous les états
DSP. Si une nouvelle cible arrive avant la fin du lissage, Odin repart de
la hauteur de la touche précédente, selon son API native.

## Traversée d’une bascule T1

Le fondu linéaire de **300 ms** conserve deux moteurs indépendants. Une
bascule ne peut pas transférer l’état d’enveloppe ni la fréquence courante :

- l’ancienne instance relâche ses notes et ne reçoit plus d’attaque ;
- une tenue traversante est réattaquée **une fois** sur la nouvelle ;
- une hauteur `Legato` arrivant exactement à la frontière ouvre cette tenue
  avant son nouvel `note_on` legato : une attaque d’enveloppe, puis glide
  selon le réglage de la **zone entrante** ;
- une attaque `Normal` remplace le transfert mono, une fin exactement sur
  la frontière l’annule ; aucun doublon musical ;
- si le glide a commencé avant la frontière, la nouvelle instance démarre
  à la **hauteur cible** tenue. L’ancienne poursuit sa sortie pendant le
  fondu ; on ne prétend pas prolonger un même état DSP entre deux moteurs.

Les diagnostics restent des compteurs de touches `NoteOn`/`NoteOff`, pas un
compteur d’attaques d’enveloppe. La preuve de non-réattaque doit venir du
signal rendu, avec une enveloppe dont attaque et sustain se distinguent.

## Preuves de contrat et de compatibilité

`cargo test -p harmonium --features odin --test odin_contract_tests -- --nocapture` :
**18 contrats réussis**, dont six nouveaux. Le callback complet tourne sous
`RTCheckAllocator`. Preset JUCE synthétique sinus, sustain à 0,2 : la vraie
attaque dépasse deux fois le RMS du plateau, contrairement aux enchaînements
Legato. La hauteur est mesurée par les passages zéro interpolés du signal,
pas déduite du MIDI ni d’un compteur logiciel d’attaques.

- Montée C4 → G4 → C5, glide 400 ms : fondamentale intermédiaire à 60 ms,
  progression à 220 ms, cible à 850 ms ; seule l’attaque initiale dépasse
  le seuil d’attaque. Rejeu identique échantillon pour échantillon.
- `Normal` après Legato et répétition d’une même hauteur : vraies réattaques
  mesurées, cible immédiate ; les anciennes fins ne coupent pas la note.
- `glide_ms = 0` : cible immédiate mais enveloppe toujours liée.
- `Tied` à la barre : hauteur et enveloppe conservées, une seule attaque
  musicale pour la tenue.
- Legato exactement à T1 : deux instances, un seul transfert en plus des
  attaques musicales, aucune différence avant la frontière, glide encore
  audible après celle-ci.
- T1 pendant un glide de 500 ms commencé 250 ms avant la frontière :
  fondamentale encore intermédiaire juste avant, cible sur la nouvelle
  instance après le fondu. Pas d’attaque musicale supplémentaire pour `Tied`.

Chaque nouvelle prise vérifie `NoteOn == NoteOff` par instance, zéro note
active après mute, zéro débordement d’historique, silence final. Les deux
cas T1 vérifient le retrait après exactement 13 230 frames, le maintien du
signal dans le fondu et une différence inter-échantillon inférieure à 0,1
sur le sinus de contrat. Cette borne n’est pas une garantie universelle de
continuité pour n’importe quel preset.

Référence exacte : archive de `main` **0efe475c8d80bbc894b3e06bcef4d44e38e3ee10**,
compilée séparément en release avec Odin, sans merge ni autre worktree.
**15 rendus de 16 mesures**, graine 1, sont identiques **octet pour octet,
en-tête WAV compris** :

- les dix profils traditionnels distribués, non modifiés ;
- les cinq profils de jeu, copiés temporairement hors dépôt avec les seuls
  `legato = true` remplacés par `false`, afin d’exercer leurs vraies pistes
  Odin sans Legato. Leurs zones, presets, morphings et autres données ne
  changent pas ; les profils distribués ne sont pas modifiés.

Références et rendus comparés :
`/tmp/harmonium-017/g2-{main,apres}-sans-legato-<profil>.wav` ; copies des
profils de jeu dans `/tmp/harmonium-017/sans-legato/`.
Même GeneralUser-GS de 32 319 396 octets et mêmes presets que les écoutes.

## WAV et repères d’écoute

Rendus silencieux du CLI release avec `odin`, graine 1, sans automation ni
normalisation ajoutée. WAV float32 stéréo, 44 100 Hz ; crêtes mesurées par
`ffmpeg -af astats`, toutes strictement sous 0 dBFS.

Dossiers hors dépôt :

- NT : `~/Project/bascanada/harmonium_lab/corpus/nuclear_throne/ecoute_g7/`
- Isaac : `~/Project/bascanada/harmonium_lab/corpus/isaac/ecoute/`

| Fichier | Mesures | Frames | Durée (s) | Crête (dBFS) |
|---|---:|---:|---:|---:|
| `g2_nt_desert_glide.wav` | 32 | 2 419 200 | 54,857143 | −3,487599 |
| `g2_nt_desert_basse_glide.wav` | 32 | 2 419 200 | 54,857143 | −6,590187 |
| `g2_isaac_basement_glide.wav` | 48 | 3 628 800 | 82,285714 | −5,931532 |

Empreintes SHA-256 :

```text
24c1d61cf4ec7ab4f3ae208a9654fa840b4d2a876069430d467fa0c8e9e30c7d  g2_nt_desert_glide.wav
195515719b5e286d3a1e1edd1acabfb7c0467dc158def63eaf1f200a7b85126e  g2_nt_desert_basse_glide.wav
1011dd0f069a81213ae018993a9a45d35c047427cd367d7c954eb4904aad8b15  g2_isaac_basement_glide.wav
```

À écouter, en **minutes:secondes** :

- NT, complet ou solo Bass : **0:05–0:07**, relance D–E–F♯–G–G♯
  (départ exact 0:05,143), puis **0:12–0:14** pour sa reprise.
  **0:14–0:17**, riff D/D♯ et montée ; les répétitions de même hauteur
  gardent volontairement leur attaque. La basse se tait à partir de
  **0:27,429** dans le pont : la seconde moitié du solo est donc silencieuse.
- Isaac : **0:14–0:20**, montée F–G–A–B♭, puis **0:22–0:27**,
  descente A–G–F–E. La répétition `Normal` à **0:15** reste attaquée ;
  reprise du même parcours vers **0:28–0:41**.

Repères calculés depuis `bar_start_frames` et les notes des vérités exportées
dans `/tmp/harmonium-017/g2_*_glide.json`, pas depuis l’horloge murale.
Cette session n’a ouvert aucun périphérique sonore ; le jugement musical
reste celui du propriétaire. Pour l’essai interactif :

```sh
make studio STUDIO_ODIN="$HOME/Project/bascanada/odin-presets"
```

Choisir `nt_desert` ou `isaac_basement`, graine 1 ; pour NT, isoler `Bass`.
Reproduction des trois fichiers :

```sh
cargo build --release -p harmonium-cli --features odin
ODIN="$HOME/Project/bascanada/odin-presets"
NT="$HOME/Project/bascanada/harmonium_lab/corpus/nuclear_throne/ecoute_g7"
ISAAC="$HOME/Project/bascanada/harmonium_lab/corpus/isaac/ecoute"
target/release/harmonium-cli --export --style nt_desert --seed 1 --bars 32 \
  --soundfont assets/GeneralUser-GS.sf2 --odin-presets "$ODIN" \
  --record-wav "$NT/g2_nt_desert_glide.wav"
target/release/harmonium-cli --export --style nt_desert --seed 1 --bars 32 \
  --solo-track Bass --soundfont assets/GeneralUser-GS.sf2 --odin-presets "$ODIN" \
  --record-wav "$NT/g2_nt_desert_basse_glide.wav"
target/release/harmonium-cli --export --style isaac_basement --seed 1 --bars 48 \
  --soundfont assets/GeneralUser-GS.sf2 --odin-presets "$ODIN" \
  --record-wav "$ISAAC/g2_isaac_basement_glide.wav"
```

## Validation finale

| Commande | Résultat observé |
|---|---|
| `cargo test --workspace -q --no-fail-fast` | Réussi, aucun échec ; deux tests ignorés |
| `cargo test -p harmonium --features odin` | Réussi, dont les 18 contrats Odin |
| `cargo check --target wasm32-unknown-unknown -p harmonium_audio --features odin` | Réussi |
| `cargo +stable clippy --workspace --all-targets --features odin -- -A clippy::all -W clippy::unwrap_used -W clippy::panic -W clippy::todo -W clippy::expect_used` | Réussi avec avertissements `unwrap`/`expect`/`panic`, sans prétention de zéro avertissement |
| `cargo +nightly fmt --all -- --check` | Réussi |
| CLI release, trois exports réels et `ffmpeg -af astats` | Réussi ; zéro NaN, zéro infini et aucune crête à 0 dBFS |

Les options Clippy reprennent la convention `make lint` du dépôt, avec Odin
et toutes les cibles du workspace. La première compilation globale a révélé
des constructeurs `AudioEvent::NoteOn` encore incomplets dans les fixtures du
lab ; ils ont été migrés avec `legato = false`, puis toute la chaîne ci-dessus
a réussi. Aucun test supprimé, ignoré ou assoupli pour masquer cet échec.

Les rendus, références et vérités restent hors dépôt ; aucun script temporaire
ni preset tiers ajouté au commit. La SoundFont locale vient de la copie déjà
présente dans le checkout principal, sans téléchargement.
