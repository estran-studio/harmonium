# G7b — Sourdine par partie (`tracks_off`)

Tâche moteur de la phase 014 : couper des pistes sur une partie de la forme, en
données. Résout l'écart connu de G7 (batterie au pont de *Fläshyn*).

## Ce qui a changé

- `harmonium_core/src/style_profile.rs` : `ProfilePart.tracks_off: Vec<TrackId>`
  (`#[serde(default)]`, omission = aucune sourdine) + `ProfileForm::part_mutes` ;
  validation : doublon refusé (`form.parts[i].tracks_off`). Un nom de piste
  inconnu est refusé au chargement (désérialisation `TrackId`) ; une partie
  inexistante est impossible par construction (le masque vit dans la partie,
  pas dans une table à part).
- `harmonium_core/src/trad/generator.rs` : `track_active` lit le masque en plus
  de l'arrangement, des drapeaux du jeu et de la densité — une seule porte
  (principe IV). L'intro (sans partie) et l'outro tonique (hors forme) sont
  inchangés.
- `harmonium_core/src/trad/accomp_profile.rs` : une partie qui tait la basse
  est exemptée de l'exigence « un motif de basse par partie » ; un motif qui la
  nomme est refusé comme sans emploi (pas de donnée morte).
- `harmonium_core/src/trad/accomp.rs` : `BassMotifPlan` ne tire aucune tranche
  pour une partie muette (vecteur vide, accès protégés) ; une `CopyOfPart`
  depuis une partie muette tire une tranche fraîche au lieu de copier du vide.
  Comportement inchangé dès que la source existe.
- `profiles/nt_desert/profile.toml` : `tracks_off = ["Bass", "Heel", "Toe"]`
  sur le pont ; motif de basse du pont supprimé (devenu sans emploi).
- `tests/fixtures/profiles/gigue_bourdon/profile.toml` : partie B avec
  `tracks_off = ["Heel", "Toe"]` (principe VIII, autre style : gigue 6/8).
- `harmonium_core/tests/part_tracks_off_tests.rs` : 3 tests de contrat sur
  snapshots et lecteur (pont muet puis reprise sans orpheline ; B sans pieds
  de la gigue ; validation piste inconnue / doublon / partie inexistante /
  motif sans emploi / exemption).
- `harmonium_core/tests/nt_desert_profile_tests.rs` : `arche_48_melodie_tardive`
  fige le contrat inverse (silence Bass/Heel/Toe mes. 17–32, présence avant/après).
- `profiles/README.md` : ligne `[form]`, recette `parts.tracks_off`, exemption
  basse documentées.
- `specs/014-styles-de-jeu/g7-nt-desert.md` : écart marqué résolu (§ « Écart
  résolu », tableau du pont, minutage d'écoute, limites).

## Choix et pourquoi

- **Champ inline dans `[[form.parts]]`** (proposition du compte rendu G7
  confirmée) plutôt qu'une table à part : une partie inexistante ne peut pas
  être nommée ; pas de jointure, pas de deuxième validation.
- **Une seule porte (`track_active`)** : ni deuxième `allows`, ni drapeau
  dédié (principe IV). Les plans internes (mélodie pour le vote, tranches de
  basse des parties sonores) restent résolus comme en G4 : la densité et les
  entrées gardent leur sens, la sourdine tranche en dernier.
- **Fermeture/reprise sans code lecteur** : le générateur n'émet aucune note
  sur la partie muette ; la fin en vol précédente tombe à la barre, et la
  première attaque liée suivante retombe sur une attaque normale (repli sûr
  G5, la hauteur ne sonnant plus). Prouvé par le bilan `NoteOn == NoteOff`
  au lecteur sur 40 mesures + purge.
- **Motifs exigés sauf basse muette** : une partie muette n'a pas de matière
  basse à tirer ; exiger quand même une banque serait de la donnée morte.

## Fichiers touchés

Moteur : `harmonium_core/src/style_profile.rs`, `trad/generator.rs`,
`trad/accomp.rs`, `trad/accomp_profile.rs`. Données : `profiles/nt_desert/`,
fixture `gigue_bourdon`. Tests : `part_tracks_off_tests.rs` (nouveau),
`nt_desert_profile_tests.rs`. Docs : `profiles/README.md`, `g7-nt-desert.md`.

## Preuves de test

- `part_tracks_off_tests` : 3 tests verts (pont muet + reprise + bilan nul au
  lecteur ; B sans pieds sur deux tours de gigue ; validation ×5 cas).
- `nt_desert_profile_tests` : 6 tests verts, contrat inverse figé.
- Non-régression : exports `--bars 64 --seed 7 --record-truth` des 10 profils
  trad identiques entre `main` et la branche (0 mesure divergente) ;
  `TUNING_VERSION` inchangé, goldens non régénérés.
- `make quality` vert (fmt + clippy + tests).
- Smoke : `--export --bars 32 --profile tests/fixtures/profiles/gigue_bourdon
  --seed 1 --record-midi /tmp/g7b-pistes-par-partie.mid` (3 861 octets) :
  6 coups de pieds par mesure en A, 0 en B (mes. 9–16, 25–32),
  mélodie/accords/basse de B inchangés.
- WAV d'écoute G7 re-rendus (banque `assets/nt_desert.sf2`, graine 1,
  64 mesures) dans `corpus/nuclear_throne/ecoute_g7/` (noms inchangés) :
  pont sans batterie ni basse à 27,4–54,9 s sur `nt_desert_seed1.wav`.
  Stems solo `--solo-track Heel/Bass` : silence numérique mes. 18–32,
  seule la queue de réverb de la frontière sonne en mes. 17.

## Limites

- La sourdine est statique (données) : le jeu ne peut pas couper une piste sur
  une partie à chaud — les drapeaux `enable_*` et `density_tracks` gardent ce
  rôle global.
- Une `CopyOfPart` de basse depuis une partie muette tire une tranche fraîche
  au lieu de copier : cas pathologique documenté, déterministe, sans panique.
- Les approches de basse (`passing`) vers une mesure muette ne sont pas
  inhibées (aucun profil ne combine les deux ; le cas reste équilibré).

## Ce qu'il faut écouter

- `nt_desert_seed1.wav`, 27,4–54,9 s (mes. 17–32) : triades + sifflement seuls,
  **sans batterie ni basse** ; reprise complète à 54,9 s (mes. 33).
- `/tmp/g7b-pistes-par-partie.mid` : gigue 6/8, pieds en A, silence des pieds
  en B (mes. 9–16, 25–32), tout le reste inchangé.
