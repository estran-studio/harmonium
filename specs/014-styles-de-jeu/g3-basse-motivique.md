# G3 — Basse motivique

## Changement et choix

**Étendre `BassVoice`, pas les cellules mélodiques.** Celles-ci imposent degrés,
cadences et continuité mélodique ; elles ne conviennent pas aux chromatismes,
silences et tenues multi-mesures d'un riff. `hits` et `motifs` sont deux sources
exclusives de la même voix : réalisation des notes/doublures, dynamique,
interprétation et lecteur communs, aucun second chemin audio ni nom de style en code.

Banque pondérée de motifs de 1 à 4 mesures : pas/durées sur la grille du profil,
demi-tons **ou** degrés du mode actif, octave relative, vélocité, doublures et
liaison. Filtrage optionnel par partie. Plan fixé par air sur un ChaCha8 dérivé
du matériel, sans déplacer les tirages de session ; reprises, tours et retours
d'airs signature conservent les choix. `CopyOf`/`CopyOfPart` copient une mesure
réharmonisée à destination, sans suspendre l'avancée du motif. Seconde fin :
copie, motif d'une mesure ou tranche finale conservée. Fin de partie : motif
entier préféré, sinon tronqué à la frontière, pas de silence ajouté.

`chord_root` suit la fondamentale à l'attaque ; `tonic` permet le bourdon.
Une tenue franchissant une barre publie des segments `Tied` : **mécanisme G5**
réutilisé dans le lecteur et le MIDI, sans modification du callback.

## Fixtures, source, fichiers

- `tests/fixtures/profiles/nt_basse_motivique/` : D phrygien, 12/8, 24 pas,
  pulse 6, **210 noires/min = 140 noires pointées/min**. Pédale tonique +6 =
  **G#1 (MIDI 32)** tenue trois mesures ; relance D–E–F#–G–G# aux pas
  0/5/6/7/11 de la quatrième. **Saw Lead `bass_program = 81`, 0-based**
  (décision du propriétaire, pas la guitare). Mélodie réellement omise via G4 ;
  cellules cachées pour le vote harmonique. Riff D/D# sur deux mesures,
  variante analytique de poids 3 et variante synthétique espacée de poids 1.
- `tests/fixtures/profiles/valse_basse_motivique/` : grille 3/4 de la valse
  québécoise, marche modale 1–2–3 / 5–4–2, copies locales/interparties. Cellules
  et arrangement synthétiques, pas des mesures historiques du corpus (VIII).
- Source NT : Jukio Kallio, *Fläshyn*, Nuclear Throne OST (2015), analyse G0
  `harmonium_lab/references/nt-desert/README.md` §6.1/7.2 et JSON
  `02_flashyn_bass_*` consultés hors dépôt. Seuls les **dérivés analytiques**
  entrent ici : positions regroupées, vélocités synthétiques. Frontières rapides
  et octave restent à valider à l'oreille selon les réserves de G0.
- Code : `trad/accomp_profile.rs`, `trad/accomp.rs`, `trad/generator.rs`,
  `trad/bass_passing.rs`, `style_profile.rs`. Deux suites `bass_motif*_tests.rs` ;
  schéma et règles détaillés dans `profiles/README.md`.

## Preuves

- `make quality` : fmt, clippy et tests workspace passent ; goldens existants
  passent sans régénération. Les avertissements historiques du dépôt restent.
- Deux suites G3 : **13 tests passent**. Snapshots exacts/transposition,
  pondération, copie interrompant une cellule, frontière de partie, marche
  interparties, dynamique/silence, cibles chromatiques préservées avec `passing`,
  invalidation/rejeu ; lecteur sans réattaque ni note orpheline ; chargeur
  refusant ambiguïtés, chevauchements, poids/références/grille/MIDI invalides.
- Comparaison publique avec HEAD initial `69c1449` : **10 profils × graines
  1/7 × 128 mesures = 2 560 snapshots identiques**, avec changement des quatre
  émotions et invalidation à la mesure 33. Sonde temporaire retirée ; profils
  distribués et `TUNING_VERSION = 21` inchangés.
- Smoke réel GeneralUser-GS, seed 1, 32 mesures :
  `/tmp/g3-basse-motivique.mid` (**2 812 octets**, programme 81,
  **106 attaques / 106 fins**) ; `/tmp/g3-valse-basse-motivique.mid`
  (**4 539 octets**, programme 32, **94 attaques / 94 fins**).
  Aucun recouvrement de même hauteur ni note orpheline. Première pédale G#1 :
  **tick 0 → 8 640 à 480 PPQ**, trois mesures, aucune attaque aux deux barres.
- WAVs : `/tmp/g3-basse-motivique.wav`, `/tmp/g3-valse-basse-motivique.wav`.
  Solo Bass : `/tmp/g3-pedale-saw.wav`, sustain −38,66 dBFS sur 2–2,5 s et
  −37,46 dBFS sur 4–4,5 s. Pas de validation subjective d'écoute revendiquée.

## Limites et écoute

Intégration G2 : les motifs acceptent désormais `trad_functional` et `loop` ;
la fixture `nt_pedale_boucle` vérifie leur transposition sur ses accords fixes.
Les degrés suivent le mode de la tonalité depuis la référence ; octaves relatives,
pas de repli du contour ; refus MIDI conservateur jusqu'à trois demi-tons/pas modal.
`SequenceOf`, `VariedEndOf`, résolution par passe, variations et préférences
rythmiques restent mélodiques : écrire les variantes de basse dans sa banque.

Arrangement, seuils de densité, `bass_velocity` et interprétation Bass restent actifs.
La densité ne clairseme pas le riff écrit ; l'arousal ne retire pas le motif en cours.
`passing` garde densité/phrase/arousal et approche les cibles réellement écrites ;
seuls les ajouts subissent le veto, mais une porteuse peut être écourtée. Les fixtures
l'omettent. Gate/respiration/décalage peuvent rompre une liaison comme en G5 ; pas
de CC11/aftertouch dans la tenue. L'outro existante reste prioritaire, même au CLI.

```sh
cargo run -q -p harmonium-cli --release -- --export --bars 32 \
  --profile tests/fixtures/profiles/nt_basse_motivique --seed 1 \
  --record-midi /tmp/g3-basse-motivique.mid --record-wav /tmp/g3-basse-motivique.wav
```

Écouter la scie sans creux aux mesures 2/3, la relance mesure 4, puis le demi-ton
phrygien du riff dès la mesure 9 et sa transposition. La valse doit marcher sur
trois noires sans adopter les chromatismes NT. Fixtures G3, pas profil de jeu G7.
