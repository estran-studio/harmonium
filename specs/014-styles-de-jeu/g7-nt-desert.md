# G7 — Profil `nt_desert` (données seulement)

Tâche données de la phase 014 : la musique du désert de *Nuclear Throne* en
profil de style, sans ajouter de code moteur. Tout le moteur vient de
G1 (phrygien, bascule par valence), G2 (`harmony.model = "loop"`, pédale liée
sur `Chords`), G3 (motifs de `BassVoice`, pédale liée), G4 (forme en boucle,
sifflement à entrée tardive) et G5 (`tie`, segments `Tied`). Les fixtures
`nt_pedale_boucle`, `nt_basse_motivique`, `boucle_test` et `nappe_pad` ont
servi de patrons ; aucun mécanisme nouveau n'a été nécessaire, sauf un,
depuis résolu en G7b : la sourdine par partie (pont sans batterie ni basse),
impossible en données au moment de G7 (voir « Écart résolu » ci-dessous).

## Ce qui a changé

- `profiles/nt_desert/` : `profile.toml` (méta, mètre, forme, mélodie,
  harmonie `loop`, accompagnement, instruments 78/81/81 + kit 128/0, mix),
  `cells.toml` (sifflement open/run), `feet.toml` (grille §6.2 : kick 0/8/20,
  snare 2/10/14, charleston 42 à tous les pas pairs), `arrangement.toml`
  (air de 48 mesures, sifflement au pont), `dynamics.toml` (4 émotions),
  `set.toml` (un air par set, boucle sans fin), `README.md`.
- `harmonium_core/tests/nt_desert_profile_tests.rs` : 6 tests de contrat sur
  snapshots et lecteur public (chargement par nom, arche 48 mesures,
  pédale liée à une attaque, grille batterie, presets, calme à density 0.2).
- `harmonium_core/tests/nappe_contract_tests.rs` : le garde-fou `tie` couvre
  11 profils (`nt_desert` seul autorisé à lier).
- `harmonium_cli/tests/export_terminates.rs` : le contrat d'export couvre
  tous les distribués sans pinner leur nombre (10 → N).
- `profiles/README.md` : ligne `nt_desert` au tableau des distribués.

Aucun fichier moteur touché (`harmonium_*/src` inchangé) ; les 10 profils
existants sont identiques octet pour octet (`git status` : que des ajouts +
3 tests/docs). `TUNING_VERSION` inchangé, goldens non régénérés.

## Choix par section de *Fläshyn*

Tonique G# phrygien au neutre ; tout est relatif (degrés modaux,
demi-tons) donc la couleur transpose la structure entière.

| Section | Mesures | Harmonie | Basse | Choix et pourquoi |
|---|---|---|---|---|
| A1 pédale G# | 1–8 | pédale degré 1 | G#1 tenu 3 mes. (`tie`), relance D–E–F#–G–G# absolus (G0 §6.1) | Scie 81 liée, une attaque (décision d'écoute ; jamais de 30) |
| A2 riff D/D# | 9–16 | boucle 8 mes. : D mineur (+6 demi-tons) | riff D1/D#1 (b2) + montée F–G–G#, variante espacée (poids 3:1) | La superposition D + D# + A n'est pas une triade : D mineur l'approche, tierce documentée |
| Pont nappes | 17–32 | 16 accords tenus (C C E E F B C C / F C E E F B C# C#) | tue (`tracks_off`, G7b) | Sifflement 78 entre au pont (G4 ; bribes du pont) ; chute F–C#–D de fin d'A1 omise (pédale pure) |
| B G/G# | 33–40 | I–bII phrygien, 2 + 2 mes. ×2 | 1 note douce / 2 mes. (quasi-absente, comme le stem) | Scie liée sur les triades |
| A1' reprise | 41–48 | rejouée d'A1 (`sequence [0,1,2,3,0]`) | rejouée d'A1 (bit à bit aux graines testées) | Reprise littérale (r = 0,99 dans l'original) |

Batterie : grille §6.2 exacte à densité 1 (vérifiée au test). Sifflement :
tenues + traits 1-2-3-2-1, porteuses snapped sur l'accord (G2). Set d'un
seul air : boucle de jeu sans fin, entrées rejouées à chaque air, pas
d'intro ni d'outro. Mix propositionnel (salle petite et sèche, scies en
retrait, charleston bas) : à valider à l'oreille, il ne change aucune note.

Réactions : arousal → tempo (180–240) + batterie (vélocité, densités 0–2) ;
density → couches (sifflement ≥ 0,35, basse ≥ 0,25, pieds ≥ 0,3/0,35 ; à 0,2
ne reste que la scie, bourdon façon *A Way*) ; tension → contretemps et notes
plus sèches (dominante à 0, la boucle n'est pas substituée) ; valence →
G# phrygien au neutre, A phrygien en bas (bourdon calme), G# mixolydien
en haut (bascule G1, degrés recalculés).

## Écart résolu : le pont tait la batterie et la basse (G7b)

La sourdine par partie (`form.parts[].tracks_off`, tâche G7b) coupe `Bass`,
`Heel` et `Toe` sur le pont, en données : le pont (mes. 17–32) est sans
batterie ni basse, comme l'original (silence total mes. 17–30). Le motif de
basse du pont (E1 épars) est supprimé : une partie qui tait la basse n'exige
plus aucun motif (exemption validée) et aucun motif ne peut la nommer. Le test
`arche_48_melodie_tardive` fige désormais le contrat inverse (silence au pont,
présence avant/après). Voir `g7b-pistes-par-partie.md`.

Autres écarts : A2 en D mineur (tierce ajoutée) ; une seule scie 81 pour
pédale et triades (pas de pad 88/89 par partie) ; pont de 16 mesures en un
air de 48 (coda omise) ; charleston explicite là où le mix l'enfouit.

## Mesures labo (64 mesures, banque nt_desert, vs `tracks/02_flashyn.flac`)

Rendus neutres graines 1–5 + script `measure_track` (`/tmp/g7_measures.py`,
labo `.venv`) : mêmes chiffres aux 5 graines (structure déterministe).

| Mesure | *Fläshyn* (réf.) | `nt_desert` (graines 1–5) |
|---|---:|---:|
| Tempo `beat_track` (BPM) | 69,8 (÷2 de 140) | **69,8**, stabilité 0,99 (0,96 réf.) |
| Tonalité (`key_strength`) | 1,41 (G# dominant) | **1,90**, G# dominant (chroma[8] = 0,63) |
| Gabarit global | C# mineur harm., marge 0,008 (ambigu, cf. G0) | **C# mineur harm., marge 0,006** (même ambiguïté !) |
| Entropie de chroma (bits) | 3,56 | 3,52 |
| Centroïde spectral (Hz) | 3 093 | **1 749** (déficit connu, cf. G6 : 1 536) |
| Rolloff 85 % (Hz) | 6 309 | 3 875 |
| RMS moyen | 0,231 | 0,180 |
| Part harmonique / percussive | 80,7 % / 19,3 % | 90,7 % / 9,3 % |
| Taux d'attaques (/s) | 0,62 | 6,95 (charleston explicite à tous les pas pairs) |

Lecture : tempo, centre tonal et ambiguïté C# reproduits à l'identique ;
brillance encore à −1,3 kHz (même limite qu'en G6 : le kit et la scie
restent sous l'original saturé) ; le charleston écrit ressort là où le mix
l'enfouit — point d'écoute n° 1.

## Quoi écouter (`corpus/nuclear_throne/ecoute_g7/`, hors dépôt)

64 mesures, graine 1 (≈ 110 s ; mesure = 1,714 s) : `nt_desert_seed1.wav`
(neutre), `nt_desert_arousal.wav` (arousal 1 : 240 BPM, densité 2),
`nt_desert_valence.wav` (valence −1 : **bourdon en A**, tempo 210),
`nt_desert_density.wav` (density 0.2 : **scie seule**, façon *A Way*),
`nt_desert_tension.wav` (tension 1 : contretemps, 216 BPM).

| Minutage | Écouter |
|---|---|
| 0,0–13,7 s (mes. 1–8) | Scie G# continue, **aucune réattaque** aux barres ; relance mes. 4 et 8 |
| 13,7–27,4 s (mes. 9–16) | Riff D/D# (demi-ton phrygien), montée F–G–G# |
| 27,4–54,9 s (mes. 17–32) | Pont : triades C–E–F–B…, sifflement entré, **sans batterie ni basse (G7b)** |
| 54,9–68,6 s (mes. 33–40) | Alternance G / G# toutes les 2 mes., basse quasi absente |
| 68,6–82,3 s (mes. 41–48) | Reprise A1 à l'identique |
| `density` : tout le fichier | Scie seule, sans pieds ni basse (valider le calme *A Way*) |
| `valence` : tout le fichier | Transposition en A (couleur par la valence) |

## Preuves de test

- `nt_desert_profile_tests` : 6 tests verts (nom, arche, pédale 1 attaque /
  bilan nul au lecteur, grille exacte, presets ⊂ banque, calme à 0,2).
- Adaptations de gardes à 11 profils, sans pinner de noms : `nappe` (seul
  `nt_desert` lie), `export_terminates` (tous les distribués, nombre non
  pinné) ; trois suites trad restreintes par structure, pas par nom :
  entropie rythmique (profils à tables `rhythm`), poids de rôles (rôles
  probabilistes en mes. 4/7 des deux premières parties), secondes fins
  (profils en déclarant deux). Aucune hypothèse de ces tests (mélodie dès
  la mes. 1, 32 mes./tour, secondes fins) ne tient pour une boucle.
- `make quality` vert : fmt, clippy, workspace (dont goldens et références
  des 10 profils, sans régénération ; `TUNING_VERSION` inchangé).
- Smoke : `--export --bars 64 --profile profiles/nt_desert --seed 1
  --automation neutre --record-midi` : 64/64 mesures pleines, 0 orpheline
  (bilan `NoteOn == NoteOff` par hauteur au lecteur sur l'air 1).
- Rappel : `--export --bars N` sans automation ne rend que l'air 1 (+ outro),
  la suite est silence (comportement existant, pas un bug du profil) : les
  rendus 64 mesures passent par une automation constante.

## Limites

- Écart structurel résolu en G7b : pont sans batterie ni basse (voir ci-dessus).
- Mix et vélocités propositionnels, non réglés à l'oreille contre la banque.
- Les 5 graines neutres donnent la même couleur (G#) : la variété vient des
  graines × émotions, pas du tirage neutre.
- Fixture d'un AUTRE style pour un algorithme ajouté (principe VIII) : sans
  objet, aucun algorithme ajouté (données seulement).
