# G7f — `nt_desert` : motifs et relief, sans sifflement

## Décision

Réécriture **en données seulement**, après les corrections du propriétaire du
2026-10-28 : A1 est sur **G♯**, A2 localement sur **D**. Aucun sifflement 78,
aucun solo générique, aucune présence supposée de sifflet dans *Fläshyn*
affirmée. La transcription `19_a_way_lead_motif` (~7 % de voix utile) n'est
pas une source assez fiable pour transcrire un thème : le dessus proposé est
**une composition originale**, pas une restitution de l'OST.

- Un appel de trois attaques (`8 — 6 — 5`), puis une réponse espacée ; au
  neutre graine 1, l'appel en A2 est **A5–F5–D5**. Pad doux **88**, disponible
  dans GeneralUser et la banque NT, à −8 dB ; aucun preset absent ni variante
  muette nécessaire. B reprend le matériau avec un centre abaissé de cinq
  demi-tons avant réalisation harmonique.
- Premier A1 : basse et nappe seules. A2 : entrée du dessus et de la batterie.
  Pont : nappes seules, `tracks_off` sur les cinq autres pistes. B : alternance
  G/G♯ et retour du motif. A1' : retour avec dessus et batterie. Au second
  tour, les couches restent entrées ; la doublure à l'octave inférieure
  n'arrive qu'aux **mesures 89–96**, à −14 dB.
- Deux tours explicites par air. `repeat_variation = 0.15`,
  `tour_variation = 0.20`, opérations `varied_end`/`repeated_notes` : quelques
  fins changent, pas de déplacements d'octave générateurs de solos.
- `phrase_ends = [8]` sur A1/A2/B pilote les `cadence_patterns` existants :
  descente de toms **48–45–41** au dernier temps, sans imposer de cadence
  harmonique. Pas de fill dans le pont ni dans l'ouverture sans batterie.
- Triades complètes liées, doublées à l'octave (`strings = 4`) dans les sections
  d'accords ; **pédale d'A1 inchangée, une seule note**. Basse motivique et
  relances chromatiques d'A1 préservées. Les copies de rôles ont été écartées
  après smoke : elles copiaient aussi la basse, effaçaient ses relances ou
  importaient le riff d'A2 en B. La répétition est portée par l'appel unique.

Les quatre axes restent distincts : arousal = tempo/vélocités/batterie ;
density = couches (dessus ≥ 0,40, doublure ≥ 0,50, nappe seule à 0,20) ;
valence = mode publié/couleur ; tension = contretemps et durée du dessus.
L'harmonie chromatique explicitement écrite n'est pas remplacée à forte tension.

## Avant/après : protocole et résultats

Dossier hors dépôt :
`/Users/william.quintal/Project/bascanada/harmonium_lab/corpus/nuclear_throne/ecoute_g7/`.
Baseline de `fe921d2`, profil copié avant édition, deux premiers tours ; son
arrangement omettait `tours_per_tune` et utilisait en réalité le défaut **3**,
malgré l'ancien commentaire « un seul tour ». Après : deux tours explicites.

Les quatre rendus ont **96 mesures, 7 198 976 frames à 44 100 Hz,
163,242086 s**, graine 1, émotions constantes **0,5/0,3/0,5/0,5** via
`g7f_neutre.toml`. L'automation évite l'arrêt/outro implicite de l'export fixe.
Même master +9 dB, même banque de pieds `assets/harmonium_trad.sf2`, aucun Odin
chargé (avertissements attendus), aucune normalisation des rendus bruts.
Les programmes requis sont effectivement présents dans les deux banques.

Notes/mesure = notes **publiées**, segments `Tied` inclus, pas attaques sonores.
Centroïde = moyenne des fenêtres de mesures, mono 22 050 Hz, magnitude STFT
2048/hop 512, fenêtres délimitées par `bar_start_frames` réellement rendu.
Les pistes ci-dessous sont celles observées après ; avant, hors pont :
nappe/basse/pieds, sans aucune voix de dessus ; au pont : nappe seule.

Abréviations : N nappe, B basse, D batterie (Heel + Toe), M dessus, C doublure.

| Tour / section | Notes/mes. avant → après | Centroïde NT, Hz avant → après | Centroïde GeneralUser, Hz avant → après | Pistes après |
|---|---:|---:|---:|---|
| 1 / A1 | 11,00 → 3,00 | 1284 → 867 | 1493 → 1386 | N B |
| 1 / A2 | 16,25 → 21,75 | 1231 → 1382 | 1212 → 1272 | N B D M |
| 1 / Pont | 3,00 → 4,00 | 988 → 1042 | 521 → 548 | N |
| 1 / B | 11,50 → 15,75 | 1462 → 1633 | 1173 → 1290 | N B D M |
| 1 / A1' | 11,00 → 14,25 | 1257 → 1501 | 1488 → 1555 | N B D M |
| 2 / A1 | 11,00 → 14,25 | 1255 → 1483 | 1527 → 1594 | N B D M |
| 2 / A2 | 16,25 → 21,88 | 1230 → 1381 | 1215 → 1273 | N B D M |
| 2 / Pont | 3,00 → 4,00 | 988 → 1041 | 518 → 546 | N |
| 2 / B | 11,50 → 15,75 | 1461 → 1645 | 1183 → 1321 | N B D M |
| 2 / A1' | 11,00 → 17,25 | 1258 → 1506 | 1488 → 1565 | N B D M C |

| Sur les 96 mesures | Avant | Après |
|---|---:|---:|
| Notes publiées | 892 | 1119 |
| Attaques MIDI (`note_on`, vélocité > 0) | 736 | 931 |
| Notes de dessus / doublure | 0 / 0 | 163 / 24 |
| Motifs de snapshot distincts (piste, hauteur, pas, durée) | 15 | 31 |
| Répétition maximale du même motif de snapshot | 24 mesures | 12 mesures |
| Crête NT, dBFS | −4,61 | −3,80 |
| Crête GeneralUser, dBFS | −7,19 | −6,37 |

Aucun écrêtage. Ces chiffres ne prouvent ni ressemblance à NT ni qualité
musicale ; **aucun jugement d'écoute humaine revendiqué**.

## Écoute comparative à niveau égal

Bruts demandés : `g7f_sf2.wav`, `g7f_generaluser.wav` ; avant :
`g7f_avant_sf2.wav`, `g7f_avant_generaluser.wav`.
Pour comparer sans biais de niveau, écouter plutôt les quatre
`g7f_comparaison_{avant,apres}_{sf2,generaluser}.wav` : même durée et
**−24 dBFS RMS stéréo global**, gain constant par fichier, pas de limiteur
ni de normalisation de crête. Crêtes toutes sous −4,7 dBFS. Ce n'est pas une
égalité de sonie perceptive LUFS. Gains et mesures : `g7f_niveaux.json`.

Repères exacts importables comme labels Audacity : `g7f_sections.txt`.
Premier tour : **0 / 13,60 / 27,21 / 54,41 / 68,02 / 81,62 s**
(A1 / A2 / Pont / B / A1' / tour suivant). Deuxième pont : **108,83–136,03 s** ;
doublure finale : **149,64–163,24 s**. Écouter l'entrée nette d'A2, le retrait
au pont, les fins de section aux toms, puis le retour final doublé — pas une
augmentation uniforme de densité.

Commande du rendu après (adapter `BANQUE` et `SORTIE`) :

```sh
cargo run -q -p harmonium-cli --release -- --export --bars 96 \
  --profile profiles/nt_desert --seed 1 --automation "$ECOUTE/g7f_neutre.toml" \
  --soundfont "$BANQUE" --feet-soundfont assets/harmonium_trad.sf2 \
  --record-wav "$SORTIE.wav" --record-midi "$SORTIE.mid" \
  --record-truth "$SORTIE.json"
```

`ECOUTE` désigne le dossier hors dépôt ci-dessus. Scripts et résultats détaillés
conservés avec les rendus : `g7f_metrics.py`, `g7f_niveaux.py`,
`g7f_mesures_{avant,apres}.json`, MIDI et truth JSON.

## Périmètre et limites

Fichiers : les six TOML et README de `profiles/nt_desert`, catalogue
`profiles/README.md`, tests `nt_desert_profile_tests` et injections d'erreurs
`profile_files_tests`, état du plan 014 et ce compte rendu. Aucun schéma changé,
aucun `harmonium_*/src`, preset Odin, profil trad distribué ou golden modifié.
Aucun algorithme ajouté : le principe VIII n'exige donc pas une nouvelle fixture ;
les mécanismes réutilisés ont déjà leurs fixtures d'autres styles.

Le modèle `loop` rapproche toutes les porteuses de la sonorité déclarée :
sur la pédale pure, le dessus reste sur G♯, pas une ligne modale libre avec b2.
Une mélodie modale libre indépendante sur pédale demanderait un réglage moteur
hors périmètre ; **non implémenté**. Le motif en A2 est adapté à D mineur, pas
présenté comme une transcription complète de D phrygien. Le pont garde la
réduction harmonique G7 (triades, pas accords slash) et une durée de 16 mesures.
Programmes uniques par piste ; aucune extension legato G7e ni API Odin G6d.

## Vérification réelle

- Smoke CLI via `cargo run -q -p harmonium-cli --release -- --export --bars 32
  --profile tests/fixtures/profiles/gigue_bourdon --seed 1 --record-midi
  /tmp/g7f-nt-desert-riche.mid` : 3 861 octets, 153 attaques mélodiques,
  registre MIDI 62–79, bilan NoteOn/NoteOff nul par hauteur/canal.
- Smoke du profil NT, 32 mesures avec automation neutre : 1 734 octets,
  21 attaques du dessus (programme 88, MIDI 74–81), 66 attaques de basse,
  45 de nappe, 66 de kit. Aucune note orpheline.
- MIDI final 96 mesures : 7 611 octets, 163 attaques du dessus et 24 de
  doublure (G♯4, MIDI 68), aucune note orpheline. Programmes observés
  88/89/81/88 ; aucune piste en programme 78.
- Les quatre WAV de comparaison ont été relus : durée identique et RMS
  −24 dBFS à moins de 0,00001 dB, aucun écrêtage. Les ponts et l'absence
  de doublure précoce ont aussi été vérifiés dans les truth exportées.
- Tests ciblés : **20 réussis** (`nt_desert_profile_tests` 6,
  `part_tracks_off_tests` 4, `profile_files_tests` 10). Contrats : entrées,
  pont, relances, variation des tours sans variation de l'accompagnement,
  doublure tardive, fills, liaisons sans orpheline et quatre émotions.
- **`make quality` vert** : fmt nightly, les deux clippy, workspace sans
  studio, tests studio et les six contrats Odin du host. Les références de
  notes trad passent sans régénération (`profile_d2_main_note_reference`,
  `profile_without_heterophony_keeps_main_v20_notes`,
  `fixed_roles_gigue_valse_notes_match_the_reference`, contrats des profils
  coloniaux et R1). Les dix profils trad et les goldens sont inchangés ;
  aucun bump de version de tuning.
