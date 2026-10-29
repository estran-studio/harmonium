# Phase 014 — Styles de jeu : Nuclear Throne et The Binding of Isaac

**Statut** : ouverte le 2026-10-27, sous la constitution 4.0.0 (moteur tous styles, fusion en
données, sources analysables). **Close le 2026-10-28 sur Nuclear Throne** (décision du
propriétaire : « une bonne expérimentation ») ; Isaac reste un essai d'écoute, la suite est hors
phase (voir « Clôture »). La phase 013 reste ouverte pour J5-min (dépôt 1837).

## Problème

Le propriétaire fait deux clones, l'un de Nuclear Throne (NT), l'autre de The Binding of Isaac.
Il veut tester Harmonium dans ces jeux, avec un genre et un son adaptés à chacun. Leur musique
repose sur des boucles et des ostinatos : basse et batterie en avant, harmonie modale ou sur
pédale. *Ce constat vient d'une connaissance générale des bandes originales ; il sera mesuré en
G0.* La grammaire du moteur sort des airs de danse trad. Constats dans le code (moteur `092b1a1`) :

- **Modes** : `trad::Mode` ne connaît que `Ionian`, `Mixolydian`, `Dorian`, `Aeolian`
  (`harmonium_core/src/trad/mode.rs`). Pas de phrygien, pas de mineur harmonique.
- **Harmonie** : la validation n'accepte que `trad_functional`, `jazz_functional` ou `none`
  (`StyleProfile::validate`, `style_profile.rs`). Aucun modèle de boucle d'accords fixe ni de
  pédale.
- **Basse** : `BassDegree` vaut `Root`, `Fifth` ou `FifthIfHeld` (`trad/accomp_profile.rs`). Les
  coups de basse suivent l'accord, donc un riff motivique est impossible.
- **Mélodie** : `has_track(Melody)` est toujours vrai, et les cellules `Run`, `Arp` et `Turn` sont
  exigées de tout profil (validation des rôles de cellule, `style_profile.rs`). Le moteur suppose
  donc qu'une mélodie mène.
- **Forme** : générique (« ni huit mesures, ni deux parties », `profiles/README.md`), mais les
  rôles sont `Open`, `CadHalf`, `CadFull`… Une boucle de 2 ou 4 mesures sans cadence n'a jamais
  été validée.
- **Percussion** : `FeetTable` porte des `voices` nommées, de l'euclidien et des motifs par
  densité, ce qui suffit pour une batterie. Les pistes gardent leurs noms `Heel`/`Toe` : les
  renommer romprait le contrat `MeasureSnapshot` (principe I).
- **Nappes** : aucune piste dédiée. `Chords` avec `stroke = "block"` en est le candidat ; la tenue
  au-delà de la barre de mesure reste à vérifier.
- **Son** : la banque par défaut est GeneralUser GS, en General MIDI. Le jeu passe déjà sa banque
  à `create_offline_engine(sf2_bytes, feet_sf2_bytes)`. Aucune banque propre à un jeu n'existe.
- **Labo** : l'analyse symbolique lit ABC, MIDI et MusicXML. `audio.py` (librosa) mesure tempo,
  spectre, tonalité et dynamique sur de l'audio, donc directement sur les bandes originales.
  `profile_draft.py` ne lit que l'ABC.

## Décisions du propriétaire (2026-10-27)

- **A. Constitution 4.0.0.** Tous les styles doivent pouvoir être générés et fusionnés (principe
  VIII). Les styles se nomment par jeu et zone : `nt_<zone>`, `isaac_<étage>`.
- **B. Les bandes originales servent de référence d'analyse** (principe VII). Le manifeste note
  l'œuvre, l'auteur et la source. Audio et transcriptions restent hors dépôt ; seuls les dérivés
  (mesures, cellules, profils) y entrent.
- **C. La liste des ajouts au moteur est validée** : modes, harmonie boucle ou pédale, basse
  motivique, forme en boucle, nappes, banque propre à chaque jeu.
- **D. NT d'abord, puis Isaac.** NT demande le moins de code ; Isaac ajoute le mineur harmonique,
  les nappes, l'orgue et le chœur.
- **E. À terme, un profil par zone ;** pour commencer, un seul profil par jeu.
- **F. Chaque algorithme ajouté se prouve aussi dans un autre style** (principe VIII), au moyen
  d'une fixture d'un profil trad qui l'utilise. Aucun nom de style dans le code.

## Tâches

```mermaid
flowchart LR
  G0[G0 Références NT] --> G2[G2 Harmonie boucle / pédale]
  G0 --> G3[G3 Basse motivique]
  G0 --> G6[G6 Banque NT]
  G1[G1 Modes] --> G7
  G2 --> G7[G7 Profil nt_zone]
  G3 --> G7
  G4[G4 Forme en boucle] --> G7
  G5[G5 Nappes] --> G7
  G6 --> G7
  G7 --> G8[G8 Dans le clone NT]
  G8 --> G9[G9 Isaac : références, banque, profil, clone]
```

| Tâche | Contenu | Dépôt |
|---|---|---|
| G0 | Choisir la zone NT et ses pistes de référence ; écrire le manifeste. Mesures audio librosa : tempo, tonalité et chroma, brillance spectrale, dynamique. Transcrire en MIDI les boucles qui portent le style (riff de basse, groove de batterie, motif mélodique), soit 8 à 15 boucles. Audio et MIDI hors dépôt ; mesures dérivées versionnées. | labo |
| G1 | Modes phrygien et mineur harmonique dans `Mode` et partout où le mode est lu : accords candidats, gammes, bascule de mode par la valence. | moteur |
| G2 | Modèle d'harmonie générique de boucle d'accords fixe par partie, ou de pédale de tonique ; rythme harmonique lu dans le profil. | moteur |
| G3 | Basse motivique : une ligne de basse tirée d'une banque, avec degrés libres et octaves, répétée par la forme. La conception reste ouverte (étendre `BassVoice` ou ouvrir les cellules à plusieurs pistes) ; un seul chemin au bout (principe IV). | moteur |
| G4 | Forme en boucle : parties de 2 ou 4 mesures sans cadence ; mélodie optionnelle ou entrée tardive (arrangement, densité) ; cellules `Run`/`Arp`/`Turn` exigées seulement si la forme les emploie. | moteur |
| G5 | Nappes sur `Chords` : accords plaqués tenus. D'abord vérifier si les données suffisent, sinon ajouter la tenue au-delà de la barre de mesure. | moteur |
| G6 | Banque NT : basse synthé saturée, lead, boîte à rythmes. Source et licence au manifeste ; construction au labo, comme `make trad-sf2`. | labo |
| G7 | Profil `nt_<zone>` en données. Distance au corpus mesurée au labo (audio et symbolique), puis écoute. | moteur |
| G8 | Dans le clone NT : `create_offline_engine` avec sa banque et une émotion fixe (comme J5-min), puis une table de pilotage (signaux du jeu → 4 émotions) côté jeu. | clone NT |
| G9 | Isaac : G0, G6, G7 et G8 pour une première zone (`isaac_<étage>`), en réutilisant G1 à G5. | labo, moteur, clone Isaac |

G0 et G1 peuvent démarrer en parallèle, de même que G4 et G5. La conception de G2 et G3 attend
les transcriptions de G0.

### État (2026-10-27)

| Tâche | État | Compte rendu |
|---|---|---|
| G0 | fusionnée (labo `5e50ddf`) : désert = 12/8 à 140 BPM, pédales phrygiennes G#/D/A, 10 boucles transcrites (hors dépôt) | labo `references/nt-desert/README.md` |
| G1 | fusionnée (`90acfb0`) | `g1-modes.md` |
| G4 | fusionnée (`dab7665`) | `g4-forme-boucle.md` |
| G5 | fusionnée (`69c1449`) | `g5-nappes.md` |
| G3 | fusionnée (`98d0e10`) : motifs pondérés dans `BassVoice`, pédale G#1 tenue en scie (GM 81, décision d'écoute) | `g3-basse-motivique.md` |
| G6 | fusionnée (labo `3fa7eaa`) : `nt_desert.sf2` procédurale CC0, scies 38/81 tenues (excursion < 0,1 dB), kit rock sec | labo `references/nt-desert/banque.md` |
| G2 | fusionnée (`12e0530`) : `harmony.model = "loop"`, pédales et boucles d'accords par partie, composée avec G3 | `g2-harmonie-boucle.md` |
| G7 | fusionnée (`40dcc0f`) : `profiles/nt_desert` en données (arche de Fläshyn, tempo et centre tonal reproduits au labo ; brillance −1,3 kHz) | `g7-nt-desert.md` |
| G7b | fusionnée (`c26c2c3`) : `tracks_off` par partie ; pont de `nt_desert` sans batterie ni basse | `g7b-pistes-par-partie.md` |
| G7c | fusionnée (`229dc34`) : réécriture de `nt_desert` après écoute (sans sifflement, une voix grave, nappe, groove de batterie) | `g7c-nt-desert-ecoute.md` |
| G6b | fusionnée (`113fe81`) : synthé Odin par piste derrière l'option `odin`, morphing par les 4 émotions, routage de percussion par profil | `g6b-odin.md` |
| G7d | fusionnée (`94ba523`) : mix de `nt_desert` (basse au premier plan, charleston réduit aux accents, Odin +17 dB) | `g7d-nt-desert-mix.md` |
| M1 | fusionnée (labo `0a1d77f`) : distance générique profil→corpus (`styledist`, audio + symbolique), première mesure sur `nt_desert` | labo `references/nt-desert/distance-nt_desert.json` |
| G9-G0 | fusionnée (labo `6a28971`) : références Basement d'Isaac (mineur harmonique sur pédale, 140 BPM, 4/4 ; propositions `isaac_basement` avec presets Odin candidats) | labo `references/isaac-basement/README.md` |
| G6c | fusionnée (`fe921d2`) : morphing préparé hors du thread audio, avertissement au chargement | `g6c-odin-finitions.md` |
| G6d | fusionnée (`6770b43`) : vrai moteur Odin fidèle (odin-rs `4718189`), morphing sur 45 paramètres du catalogue, instances hors callback, boost +17 dB retiré | `g6d-odin2.md` |
| G7f | fusionnée (`12bb1c7`) : dessus composé A5–F5–D5 (pad 88), deux tours, relief par section, sans sifflement | `g7f-nt-desert-riche.md` |
| G7e | fusionnée (`1f89f36`) : `Articulation::Legato` (relance et riff de basse liés, chevauchement de 0,2 pas au lecteur) ; oxisynth n'interprète ni mono ni portamento, glide Odin en suivi | `g7e-legato.md` |
| G7h | fusionnée (`d90ef38`) : lisibilité de `nt_desert` — relance et riff montés d'une octave (médiane basse MIDI 27 → 39, plus aucun demi-ton sous 70 Hz hors pédale G#1), figures ≥ 2 pas, dessus en lead 81 au lieu du pad 88 ; legato G7e conservé | `profiles/nt_desert/README.md` |
| M1 bis | fusionnée (labo `621f862`) : re-mesure de `nt_desert` au moteur `aadc812` (audio 59,6–60,9 vs *Fläshyn*, symbolique 70,7 → 72,6) | labo `references/nt-desert/README.md` §8 |
| G9 essai | `profiles/isaac_basement` en données seules (D mineur harmonique, pédale, montée/descente de *Diptera Sonata*, pont de *Periculum*), aucun ajout moteur ; distance symbolique 65,9–69,0, audio 38,7–40,9. Aimé à l'écoute (2026-10-29) ; suivis du même schéma : `isaac_cellar` (*Periculum*), `isaac_burning` (*Fundamentum*, F# phrygien), `isaac_downpour` (*River of Despair*), non mesurés | `profiles/isaac_basement/README.md`, labo `references/isaac-basement/README.md` §8 |
| G8 | reportée hors phase (décision du propriétaire, 2026-10-27 : on se concentre sur harmonium) | — |

Après G7h : 637 tests passent sans `odin`, 93 avec `odin` (`-p harmonium`). Labo après G0, G6,
M1 et G9 : 290 tests passent, 27 ignorés. Port Odin 2 fidèle dans `estran-studio/odin-rs`
(`4718189`), branché par G6d.

Prérequis G6d corrigé en amont (`odin-rs` `4718189`, poussé) : les tables dessinées immuables
utilisaient `Rc`, ce qui empêchait le transfert de l'instance prête vers le callback CPAL (`Send`).
Elles utilisent désormais `Arc`, cloné au chargement et non par échantillon. Preuves : 108 tests
avec `std`, 91 sans `std`, clippy strict, compilation wasm32 et rendu réel DeepBass de 4 s.

Écoute du propriétaire (2026-10-27) : la pédale de *Fläshyn* rendue en GM 38 décroît (−19 dB sur la
tenue) et laisse la batterie seule ; GM 30 (guitare saturée) « trop étrange » ; **GM 81 scie
retenue**.

Écoute de `nt_desert` G7 (2026-10-27) : « horrible » — buzz (deux scies 81 empilées dans le grave),
batterie en avant et rigide (charleston à tous les pas pairs, swing 0,50, aucun micro-timing),
solos sifflés sans rapport (mélodie de cellules génériques, programme 78). Constat : la structure est
juste, le son et l'écriture non ; la banque procédurale G6 est le maillon faible. Corrections
d'écriture : G7c.

Retour d'écoute suivant : le propriétaire privilégie la fluidité des notes, puis juge le morceau
« ultra vide et répétitif ». Le manque de fluidité est observé, sa cause n'est pas encore isolée :
ne pas conclure sur la seule lecture des hauteurs qu'une octave est fausse, ni qu'un glissando est
nécessaire. Comparer d'abord transitions, enveloppes et chevauchements. `nt_desert_seed1.wav`
reste une ancienne version ; la page LAN sépare désormais les derniers témoins des archives.

**Décision (2026-10-27) : synthé Odin.** Le propriétaire veut un vrai synthé pour les jeux de type NT :
le port Rust `odin-rs` (GPLv3, dérivé d'Odin 2), déjà branché avant `77a26d4`. Constitution v4.2.0 :
Odin en option de compilation `odin`, instrument par piste en données, désactivée par défaut ;
Harmonium et alacod passent sous `MIT OR Apache-2.0` ; seuls les jeux publics GPL (clones dans
`alacod/games/`) activent `odin`, 1837 compile sans (SoundFont, aucun code GPL).

## Acceptation de la phase

1. `nt_desert` rendu par harmonium (CLI, banque ou presets Odin) : écoute du propriétaire « ça sonne
   comme du NT ».
2. Même critère pour `isaac_<étage>`.
3. Pour chaque profil, le labo publie la distance au corpus de référence, en audio et en
   symbolique.
4. Chaque algorithme ajouté est employé par au moins une fixture d'un autre style ; aucun nom de
   style dans `harmonium_*/src`.
5. Les notes des 10 profils distribués restent identiques à `main` (même graine, mêmes émotions).
   Sinon, bump de `TUNING_VERSION` et goldens régénérés dans le même commit (principe I).
6. `cargo test --workspace`, clippy sans nouvel avertissement, fmt.

## Clôture (2026-10-28)

| Critère | État |
|---|---|
| 1. `nt_desert` « ça sonne comme du NT » | accepté : G7h validé à l'écoute, phase close sur NT |
| 2. `isaac_<étage>` | non atteint : `isaac_basement` livré comme essai d'écoute, sans validation |
| 3. Distance au corpus | publiée pour `nt_desert` (re-mesure) et `isaac_basement` |
| 4. Algorithmes employés par un autre style | tenu (fixtures G1–G7e, principe VIII) |
| 5. Notes des 10 profils trad | identiques |
| 6. Tests | 637 sans `odin`, 93 avec `odin` |

## Hors phase

- **Mécanisme de fusion** (un profil qui reprend les couches d'un autre, principe VIII) : il vient
  après deux styles non trad, pour qu'il se conçoive sur des cas réels.
- Test à l'aveugle réel/généré BT2 (annoncé en 013) ; J1b, J2 et J3 de la phase 013.
- Brancher la musique dans les clones (`alacod/games/throne`, `games/isaac`), ex-G8 et partie jeu de
  G9 : plus tard, la phase reste dans harmonium.
- Isaac au-delà de l'essai : valider `isaac_basement` à l'écoute, puis son (basse saturée, batterie
  plus présente, voisine chromatique D# de *Periculum*) et autres étages.
- Glide Odin pour l'articulation `Legato` (G7e ne couvre que le lecteur SoundFont).

## G0 — Corpus NT (2026-10-27)

Source : *Nuclear Throne OST (Full) — Jukio Kallio — Official*, publiée par la chaîne du
compositeur (YouTube `Jk5jogaEsz8`, 2015-12-13, 69 min 28 s, opus 251). Tracklist du propriétaire.

- Dépôt labo, `corpus/nuclear_throne/` : seuls `manifest.jsonl` (35 pistes : zone, type, partie
  A/B, début/fin en secondes, auteur, URL horodatée, licence) et `VERSION` sont versionnés ;
  `raw/` (webm + info.json) et `tracks/NN_<titre>.flac` (44,1 kHz, 836 Mo) sont gitignorés
  (`corpus/*/*`).
- Types : 20 pistes `zone` (A et B, dont les zones secrètes), 11 `boss`, plus `theme`, `ambient`,
  `credits` et `secret` (1 chacun).
- Recréer : `yt-dlp -f bestaudio --write-info-json -o 'corpus/nuclear_throne/raw/%(id)s.%(ext)s'
  <url>`, puis, pour chaque ligne du manifeste, `ffmpeg -ss <start_s> -to <end_s> -i raw/…webm
  -ar 44100 -c:a flac <file>`.
- Contrôle des coupes : `silencedetect` (−50 dB) trouve les silences entre pistes à 2:05,7–2:07,8
  et 35:47,3–35:48+, ce qui encadre les horodatages 2:07 et 35:50.
- Suite de G0 : choisir la zone de départ, mesurer l'audio (`audio.py`) sur ses pistes A et B,
  transcrire les boucles en MIDI.
