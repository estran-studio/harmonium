# G2 — Harmonie en boucle et pédale

## Changement et choix

- `harmony.model = "loop"` ajoute un plan à côté de `trad_functional`,
  `jazz_functional` et `none`, dans `HarmonyPlan`. Chaque nom de partie de
  forme déclare une pédale ou une boucle d'accords ; aucun nom de style dans
  l'algorithme, aucun tirage ajouté aux profils existants.
- Racine : exactement un `degree` modal numérique 1..7 ou `semitones` 0..11.
  Une boucle porte des qualités explicites `Major` / `Minor` et des durées
  entières `bars`, avec `default_bars_per_chord` (défaut 1). Sa longueur doit
  diviser la partie. Le chargeur refuse références, racines et durées invalides.
- Une pédale est **une note**, pas une triade ni une basse qui décroît.
  `neighbor_semitones` ajoute éventuellement un voisin chromatique (dyade).
  Le voicer existant traite une, deux ou trois hauteurs, sans allocation de
  sonorité (`ArrayVec` sur pile) ; les doublures `strings` restent réservées
  aux triades. Accords, basse, passages et hétérophonie gardent leurs voies
  d'accompagnement existantes.
- Les degrés sont résolus dans le mode de `TunePlan` ; G1 reconstruit le plan
  à sa frontière de bascule de valence, sans déplacer le calendrier de boucle.
  Les offsets chromatiques et les qualités écrites restent fixes. La tension
  ne remplace pas les accords déclarés par une dominante.
- La réalisation mélodique partage celle du jazz : rythme et contour viennent
  des cellules, les porteuses se rapprochent des hauteurs de l'accord courant
  avant interprétation/ornements. Le contexte est publié dans le snapshot et
  dans `chord_changes` au pas 0.
- La tenue ne crée aucun lecteur : coups `block` de mesure entière avec
  `tie = true`, puis `Articulation::Tied` et fusion G5 au lecteur/export MIDI.
  Aucun changement au callback audio, aucune allocation ou verrou ajouté.

## Données et fichiers

- `models/chord_loop.rs`, `models/mod.rs`, `style_profile.rs` : schéma,
  validation et pré-calcul par matériel de forme (secondes fins comprises).
- `trad/harmony.rs`, `trad/accomp.rs`, `trad/generator.rs` : sonorités,
  accompagnement commun, bascule de mode, publication et réalisation mélodique.
- `tests/fixtures/profiles/nt_pedale_boucle/` : exercice technique en 4/4,
  G♯ phrygien ; quatre mesures de pédale, deux mesures G majeur puis deux
  G♯ majeur, puis boucle modale i / II de deux mesures chacun. Scie grave
  **programme 81 0-based** (Lead 2 sawtooth), une note liée sur `Chords` ;
  mélodie omise via G4, basse et batterie séparées. La basse utilise un motif
  G3 de deux mesures (fondamentale/b2 puis fondamentale/quinte), référencé
  à l'accord courant. Le programme 30 rejeté par le propriétaire n'est pas utilisé.
- `tests/fixtures/profiles/gigue_bourdon/` : banques et mètre 6/8 du profil
  `gigue_quebecoise`, partie A sur bourdon, partie B sur I–IV–V–I ; mélodie
  présente. Même modèle, autre tradition (principe VIII).
- `harmonium_core/tests/loop_harmony_tests.rs` : contrats publics ; schéma
  documenté dans `profiles/README.md`. Aucun profil distribué modifié.

`git merge main` a intégré G3 (`98d0e10`) avant le commit final. Le conflit
dans `accomp.rs` a conservé le plan de motifs G3 et le voicer G2. Le chargeur
accepte les motifs sous `trad_functional` **et** `loop` ; la gigue conserve les
coups historiques, la fixture NT vérifie la composition des deux mécanismes.

Référence : labo `references/nt-desert/README.md`, §§2, 3, 6, 7 ; *Fläshyn*,
Jukio Kallio, 2015. Les fixtures sont des exercices dérivés, **pas un profil
NT validé à l'écoute**, ni une copie des transcriptions hors dépôt.

## Preuves

- `cargo +stable test -p harmonium_core --test loop_harmony_tests` : six tests
  verts ; accords et notes exacts sur trois graines, déterminisme, transpositions,
  voisin optionnel, changement phrygien → mixolydien → phrygien aux frontières,
  gigue/mélodie, rejets du chargeur et lecteur sans réattaque ni note orpheline.
  Le cas voisin protège aussi l'intervalle réel (b2 au-dessus, sans renversement)
  et le registre fixe : fenêtre trop étroite refusée plutôt que pédale muette.
- Après fusion, les trois suites `loop_harmony_tests`, `bass_motif_tests` et
  `bass_motif_loading_tests` passent : **19 tests**, sans modifier les tests G3.
- `make quality` **après fusion G3** : fmt, clippy configuré et tests workspace
  passent ; `profile_d2_main_note_reference`, référence V20 sans hétérophonie
  et goldens gigue/valse verts sans régénération. Les avertissements historiques
  restent ; aucun avertissement dans les nouveaux module et tests G2.
- Comparaison avec `69c1449` avant fusion, puis avec `main` G3 (`98d0e10`) après
  fusion, via une sonde temporaire
  de l'API publique : **10 profils × 2 graines (1, 7) × 2 scénarios × 256 mesures
  = 10 240 snapshots identiques**. Scénarios constant et réactif (valence,
  tension, densité, invalidations aux mesures 65/129/193). Comparaison binaire
  JSON, SHA-256 commun `47199d569f3d3f5e4d47c2810c6e661a9958f9e2e611a7ae3d1647708f58f21d`.
  Sonde supprimée ; `TUNING_VERSION = 21`, goldens non régénérés.
- Smoke CLI release, graine 1, 32 mesures NT, MIDI/WAV/vérité :
  `/tmp/g2-harmonie-boucle.mid` (2 160 octets), `.wav` et `.jsonl`.
  MIDI PPQ 480 : **une seule attaque G♯2 (44) au tick 0**, fin au tick 7680
  (quatre mesures), aucune réattaque intermédiaire ; canal 1 : 29 attaques,
  29 fins, bilan nul pour chaque hauteur. Programme 81 publié sur ce canal.
  Basse motivique : 80 attaques, bilan final nul ; ses notes et transpositions
  sont exactes dans les 32 mesures rendues.
- Second smoke : `/tmp/g2-gigue-bourdon.mid` (4 611 octets, 32 mesures),
  153 attaques de mélodie ; accompagnement et percussion présents.
- Solo `Chords`, huit mesures NT : `/tmp/g2-pedale-solo.wav`. La scie tient
  pendant les huit premières secondes : RMS par fenêtre d'une seconde après
  l'attaque compris entre 0,00555 et 0,00806, sans extinction aux barres.

Les rendus sonores ont employé la banque GM existante du labo
`samples/fiddles/raw/generaluser.sf2` via `--soundfont` : le seul `assets/test.sf2`
local n'offre pas les timbres GM demandés. Banque non copiée dans le dépôt.

## Limites et écoute

- Durées entières en mesures ; ni accords slash ni turnaround implicite.
  Une section distincte exprime un raccord. Les intros vamp et l'outro tonique
  restent prioritaires et indépendantes des accords du corps de l'air.
- Qualités explicites inchangées par la valence ; seuls les degrés modaux
  bougent (II : 1 demi-ton phrygien, 2 mixolydien). Sur une pédale pure, les
  porteuses mélodiques sont limitées à sa seule classe de hauteur ; ornements
  et hétérophonie conservent leurs mécanismes modaux existants.
- Conditions de liaison G5 inchangées : pas de gate raccourci, respiration ou
  décalage d'attaque si la tenue continue est voulue. La vélocité reste celle
  de l'attaque initiale jusqu'au prochain changement de hauteur.
- Écouter d'abord `g2-pedale-solo.wav` : aucun pompage aux mesures 2–4, puis
  vraie rupture vers G. Dans le mix NT, ne pas confondre la basse réattaquée
  avec la scie continue. La gigue est une preuve de réemploi, non un réglage
  musical distribué ; aucune appréciation stylistique à l'oreille revendiquée.
