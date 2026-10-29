# T1 — Timbre Odin selon l’émotion

## Décisions

- Un seul schéma : `[[odin.<Piste>.zones]]`, une à quatre zones ordonnées,
  défaut final obligatoire sans `when`. Aucun ancien `preset` direct accepté.
  Les axes omis sont libres ; les axes renseignés se combinent par ET,
  bornes inclusives (`valence` −1..1, autres 0..1).
- Hystérésis générique de 0,03 : entrée resserrée, maintien élargi ; une zone
  antérieure peut préempter une zone moins prioritaire. Le resserrement est
  limité au milieu de l’intervalle et préserve les extrêmes du domaine.
- Le compositeur conserve la dernière sélection souhaitée et calcule le
  morphing de la zone entendue comme de la zone en attente. Le lecteur
  installe la transaction à l’entrée réelle d’une partie, reprises comprises
  (`bar_in_strain == 1`, pas 0). Ni préchargement, ni writehead, ni horloge
  murale ne déclenchent la bascule.
- Deux emplacements par piste. Les moteurs sont construits hors callback ;
  fondu audio linéaire de 300 ms, sans interpolation de paramètres entre
  presets. L’ancienne instance reçoit les `NoteOff` de ses notes et ne reçoit
  plus d’attaque ; la nouvelle monte depuis zéro. Le retrait et le retour SPSC
  n’ont lieu qu’après le fondu ; seul le thread principal détruit les moteurs.
- Les tenues sont relâchées sur l’ancienne instance puis réattaquées une fois
  sur la nouvelle, après les événements de début de pas : une fin exactement
  à la frontière supprime le transfert ; une attaque musicale du même pitch
  remplace le transfert. Les notes `Tied` dans une zone ne réattaquent pas.
- `level_db` (−60..12 dB, défaut 0) multiplie le mix de piste ; le master du
  preset n’est pas modifié pour compenser un changement de zone. Le chemin
  à gain unitaire conserve l’ordre des opérations du rendu antérieur.
- `isaac_burning` : A par défaut (Bass Crusher [RM] + Haunt [RM]) ; B sous
  `tension = [0.6, 1.0]` (Bass Low [RM], −5 dB ; Moonshade, −7 dB). Le morphing
  de B reprend celui de `isaac_burning_psyche`. Seuil choisi pour permettre
  l’essai, à régler par le propriétaire à l’écoute. Psyche reste distribué.
- Les autres profils ont une seule zone par défaut. La fixture `valse_nappe`
  utilise deux zones conjointes valence/tension, avec un même preset et un
  niveau distinct : mécanisme indépendant du style et du changement de preset.

## Propriété, contrepression et limites

`LiveHost::pump()` entretient l’expédition. Un hôte direct appelle
`MusicComposer::send_odin` périodiquement après la génération ; le CLI d’export
le fait à chaque bloc. Sans changement, aucun moteur n’est construit. Les
commandes de morphing sont séparées des transactions propriétaires : elles
ne font pas avancer la frontière d’installation. La dernière cible remplace
celle qui attend ; un choix reçu pendant un fondu attend une nouvelle frontière
après libération du second emplacement. Même règle si deux parties sont
séparées de moins de 300 ms. La contrepression SPSC suspend les commandes sans
libérer une boîte dans le callback. Seek et mute gardent l’arrêt net existant ;
une bascule de zone n’utilise jamais cet arrêt net.

Le diagnostic hors ligne est un instantané fixe des compteurs d’attaques/fins
et des positions par instance effectivement installée. L’historique est borné
à 128 retraits ; `history_overflow != 0` invalide un audit exhaustif. Ce n’est
pas un journal allouant dans le callback. Les moteurs préparés puis annulés,
jamais installés ni attaqués, ne figurent pas dans cet historique.

Aucun changement dans `odin-rs`, dans le studio, ni dans les sections musicales
des profils. Pas d’écoute automatique : les preuves sont des rendus hors ligne
silencieux ; le jugement musical appartient au propriétaire.

## Références avant changement

Révision de départ, identique à `main` au moment de la capture :
`03e772ba86610b4b89b27b3af88f3aafb024e41b`. CLI release compilé avant migration,
option `odin`, graine 1, 16 mesures, GeneralUser-GS de 32 319 396 octets et
`/tmp/harmonium-014/odin-presets` :

| Profil | WAV de référence | SHA-256 |
|---|---|---|
| `nt_desert` | `/tmp/harmonium-016/t1_nt_desert_main.wav` | `9fe03331c0210bd5a1aae0eb443b1f380af18b3258a27db0fc7fbd8739af8e49` |
| `isaac_cellar` | `/tmp/harmonium-016/t1_isaac_cellar_main.wav` | `05de5dbe8bc026e2ebfc4264046c4d264bd2dc8effc59f5b7709391e7f348c53` |

La SoundFont était absente du worktree ; copie locale depuis la sonde S1
existante, sans téléchargement ni modification des presets. Les deux rendus
font 9 676 868 octets chacun. Un rendu Burning de référence avec automation,
encore à preset fixe, est aussi conservé dans
`/tmp/harmonium-016/t1_isaac_burning_main_automation.wav`.

Le propriétaire a signalé ensuite l’intégration de V1 dans `main` (`1571654`),
avec changements musicaux des profils, ainsi que du studio S7. Cette branche
n’intègre ni ne rebase ces changements : les comparaisons ci-dessus portent
sur le `main` de départ ; le propriétaire conserve la fusion et les conflits
de profils/documentation.

## Preuves de rendu réel

Les deux rendus à zone unique après migration ont exactement les empreintes
ci-dessus ; `cmp` ne relève aucun octet différent. Le contrat SF2 ciblé
`emotion_zones_without_odin_keep_the_soundfont_audio_unchanged` passe sans
option `odin` : même audio avec ou sans zones, programme réellement audible.

Le CLI release du worktree produit le WAV demandé :

`~/Project/bascanada/harmonium_lab/corpus/isaac/ecoute/t1_isaac_burning_bascule.wav`

Automation versionnée :
[`t1-isaac-burning-automation.toml`](t1-isaac-burning-automation.toml).
Tension 0,4 jusqu’à la mesure 19, 0,85 de 20 à 43, puis 0,4 à 44 ;
tempo fixé à 140. Comme les parties font huit mesures, les frontières visées
sont **25 et 49**, pas 20 et 44. La vérité exportée donne respectivement
les frames **1 814 400 et 3 628 800**, à 44 100 Hz. Le premier échantillon
différent du rendu `main` avec la même automation est exactement à la
frame 1 814 400 : aucun changement avant la frontière de partie 25.

Mesure de continuité stéréo, maximum de `abs(x[n] - x[n-1])` par canal,
fenêtre de ±350 ms autour de chaque frontière :

| Frontière | Crête de différence |
|---|---:|
| Mesure 25 | 0,23988990 |
| Mesure 49 | 0,13098549 |

Borne observée pour ce rendu complet : **< 0,25** en amplitude normalisée
(la batterie participe à cette mesure). Ce n’est pas une borne universelle
pour tout preset ni un verdict d’écoute. `ffmpeg -af astats` relève une
crête globale de **−2,927743 dBFS**, zéro NaN et zéro infini. WAV float32
stéréo, 44 100 Hz, 4 838 400 frames, 109,714286 s, 38 707 268 octets.

Commande de reproduction silencieuse :

```sh
cargo build --release -p harmonium-cli --features odin
target/release/harmonium-cli --export --style isaac_burning --seed 1 --bars 64 \
  --soundfont assets/GeneralUser-GS.sf2 \
  --odin-presets /tmp/harmonium-014/odin-presets \
  --automation specs/016-variete-et-timbres/t1-isaac-burning-automation.toml \
  --record-wav ~/Project/bascanada/harmonium_lab/corpus/isaac/ecoute/t1_isaac_burning_bascule.wav
```

Le rejeu complet de l’automation d’écoute est identique octet pour octet (`cmp`),
SHA-256 commun :
`c0112e65a3d701d74a328c34610c0c390d33936d18fae344475c3de2a18f83e2`.

## Contrat audio avec presets synthétiques

Le contrat de nappe garde les notes pré-générées fixes et automatise seulement
le timbre : entrée en B à 3, retour en A à 12. Les installations observées
ont lieu aux frames 940 800 (mesure 9) et 1 881 600 (mesure 17), sans aucune
différence d’échantillon avant la première frontière. Le même scénario
rejoué rend les mêmes échantillons. Le callback entier tourne sous la garde
temps réel d’allocation/libération.

| Instance | Zone | Installation | NoteOn | NoteOff | Notes actives après mute |
|---|---|---|---:|---:|---:|
| 1 | A | Initiale, frame 0 | 7 | 7 | 0 |
| 2 | B | Mesure 9, frame 940 800 | 7 | 7 | 0 |
| 3 | A | Mesure 17, frame 1 881 600 | 7 | 7 | 0 |

Les 21 attaques correspondent exactement à 15 attaques musicales enregistrées
en MIDI, plus six vraies tenues traversant les frontières. Les fins exactement
sur la barre sont exclues de ce calcul indépendant : pas de réattaque fantôme.

Chaque retrait intervient exactement 13 230 frames après l’installation
suivante (300 ms à 44 100 Hz). L’audit couvre toutes les instances, sans
débordement. Les fenêtres ±350 ms des frontières ont des crêtes de différence
de 0,06996348 et 0,070160374, sous la borne de régression **0,1**.
Un second contrat fait osciller la tension entre 0,59 et 0,61 à chaque mesure :
audio identique au défaut constant, donc aucune rafale d’installations.
Le gain de zone est vérifié sur le rendu (facteur −6 dB extérieur au synthé,
erreur absolue < 10⁻⁶), pas en recopiant les valeurs des profils.

## Écoute par le propriétaire

```sh
make studio STUDIO_ODIN=/tmp/harmonium-014/odin-presets
```

Choisir `isaac_burning`, graine 1 ; monter la tension à 0,85, puis la ramener
à 0,4. Écouter le changement à la partie suivante, le relâchement des anciennes
notes et l’attaque unique des tenues transférées. Le WAV avec automation permet
de comparer le même parcours sans geste manuel. Cette session n’a lancé ni
studio ni REPL sonore.

## Vérification finale

Les suites complètes ci-dessous ont été exécutées une seule fois en fin de
travail, après les tests ciblés et les rendus réels :

| Commande | Résultat |
|---|---|
| `cargo test --workspace -q --no-fail-fast` | 676 tests réussis, 89 suites, 2 ignorés |
| `cargo test -p harmonium --features odin` | 98 tests réussis, 24 suites |
| `cargo check --target wasm32-unknown-unknown -p harmonium_core -p harmonium_audio -p harmonium` | Réussi |
| `cargo check --target wasm32-unknown-unknown -p harmonium_audio -p harmonium --features odin` | Réussi |
| `cargo +nightly fmt --all -- --check` | Réussi |
| Clippy workspace, toutes cibles, avec Odin, options de lint de la CI | Réussi avec avertissements `unwrap`/`expect`/`panic`, sans prétention de zéro avertissement |

Commande Clippy employée :

```sh
cargo +stable clippy --workspace --all-targets --features odin -- \
  -A clippy::all -W clippy::unwrap_used -W clippy::panic \
  -W clippy::todo -W clippy::expect_used
```

Le contrat existant de contrepression supposait une bascule audible sous
huit secondes à 90 BPM, donc avant la nouvelle frontière permise. Il a été
adapté au contrat de partie : fixture à 300 BPM (partie de 6,4 s), entretien
régulier de la génération et de `send_odin`, mêmes preuves d’audibilité et
d’absence de notes orphelines après saturation/rafale/retour SoundFont.
Le scénario ciblé corrigé passe, ainsi que les deux suites finales.

Les contrats couvrent aussi la priorité des zones chevauchantes, les zones
étroites/ponctuelles, les extrêmes de chaque axe, le rejet de l’ancien schéma,
la localisation des erreurs TOML et les ressources manquantes des zones
inactives. Aucun changement de `TUNING_VERSION` ni de golden musical.
