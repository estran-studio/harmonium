# Écrire un profil de style

Un dossier contient `profile.toml` et ses banques TOML. Le moteur embarque les profils
distribués : aucun fichier installé n'est nécessaire. `build.rs` en inclut les fichiers
d'origine tels quels (`profile.toml` puis chaque fichier de son `[files]`, commentaires
compris) ; ils se relisent avec `StyleProfile::distributed_files()`. Un seul décodeur lit
un profil multi-fichiers, quelle que soit la source :

- `StyleProfile::load(dossier)` (ou le chemin du `profile.toml`) : les fichiers viennent du disque ;
- `StyleProfile::from_files(fichiers)` : les fichiers viennent d'une liste `(nom, contenu)`
  en mémoire, avec les noms que `[files]` emploie (profils embarqués, atelier web) ;
- `StyleProfile::from_toml(texte)` : profil autonome, toutes les banques inline.

Une erreur s'affiche `fichier:ligne: champ: message` (par exemple
`cells.toml:9: melody.cells.cells[3]: jeton de note invalide…`) : le fichier est celui qui
porte la valeur fautive, la ligne celle de sa clé. Les champs de `ProfileError` sont `field`,
`message`, `file`, `line` et `column` (cette dernière pour les erreurs de syntaxe).

| Section | Champs / sens |
## Profils distribués

| Profil | Tradition et corpus | Mètre, tempo | Instruments | Percussion | 2ᵉ voix |
|---|---|---|---|---|---|
| `reel_quebecois` | Québec ; 574 reels | 2/2, 230 | violon, guitare, basse | oui | violon |
| `gigue_quebecoise` | Québec ; 88 gigues | 6/8, 172,5 | violon, guitare, basse | oui | violon |
| `valse_quebecoise` | Québec ; 67 valses | 3/4, 145 | violon, guitare, basse | oui | accordéon |
| `reel_acadien` | Acadie ; 169 reels | 2/2, 228 | violon, guitare, basse | oui | violon |
| `marche_britannique_coloniale` | britannique colonial ; 25 marches | 2/2, 108 | fifre | caisse | fifre |
| `gigue_britannique_coloniale` | britannique colonial ; 365 gigues | 6/8, 156 | violon, guitare, basse | non | non |
| `hornpipe_britannique_coloniale` | britannique colonial ; 78 hornpipes | 4/4, 92 | violon, guitare, basse | non | non |
| `gigue_acadienne` (R1) | Acadie ; 55 gigues | 6/8, 180 | violon, **piano** | oui | violon |
| `reel_britannique_coloniale` (R1) | britannique colonial ; 493 reels | 2/2, 208 | violon, guitare, basse | non | non |
| `valse_britannique_coloniale` (R1) | britannique colonial ; 60 valses | 3/4, 138 | violon, guitare, basse | non | non |
| `nt_desert` | Nuclear Throne, désert (Jukio Kallio, 2015) ; Fläshyn / A Way | 12/8, 210 (= 140 pointées) | dessus lead 81, nappe 89, scie 81, kit rock sec | oui (36, 38, 42 et fills 48/45/41) | lead 81, dernier retour du second tour (G7h) |
| `isaac_basement` (essai) | The Binding of Isaac, Basement (Ridiculon) ; Diptera Sonata / Periculum | 4/4, 140 | chœur 52 (descente et pont), nappe chœur 91, basse 33, kit rock sec | oui (36, 38, 42 et fills 48/45/43/41) | non |
| `isaac_cellar` (essai) | The Binding of Isaac, Cellar (Ridiculon) ; Periculum | 4/4, 140 | voix 53 (A' et pont), nappe chœur 91, synth bass 38 en croches, kit rock sec | oui (stomp de `isaac_basement`) | non |
| `isaac_burning` (essai) | The Binding of Isaac, Burning Basement (Ridiculon) ; Fundamentum | 4/4, 140 | chœur 52 (sections d'accords), orgue rock 18, basse médiator 34 | oui (backbeat en demi-temps, 42 aux temps 2 et 4) | non |
| `isaac_downpour` (essai) | The Binding of Isaac, Downpour (Ridiculon) ; River of Despair | 4/4, 144 | chœur 52 (dès B), nappe chaude 89, basse doigts 33 tenue | oui (clairsemé, caisse au temps 3) | non |
| `wjazzd_swing` (E7) | Weimar Jazz Database v2.1, sous-ensemble `rhythmfeel = SWING` (361 solos) ; batterie du Groove MIDI Dataset, timing de la Jazz Trio Database | 4/4, 150 (swing 0,71) | piano 0 (voicings sans fondamentale), basse acoustique 32 (marche), kit jazz 128/32 | oui (ride 51, pédale 44, caisse claire 38, grosse caisse 36) | non (pas de mélodie) |
| `gmd_rock` (B8) | Groove MIDI Dataset, style rock : 154 fichiers 4/4 droits, 9 batteurs ; basse, guitare, grilles : règles de métier | 4/4, 105 (90–130), droit | guitare saturée 29 (triades doublées, grattées), basse au médiator 34, kit Room 128/8 | oui (patterns par densité, phrases, relances du GMD ; ride, charleston) | non (pas de mélodie) |
| `gmd_pop` (B8) | GMD, famille pop (pop, pop/soft, pop/groove7) : 14 fichiers, 4 batteurs | 4/4, 135 (83–142), droit | piano 0 (triades), basse au doigt 33, kit standard 128/0 | oui | non |
| `gmd_hiphop` (B8) | GMD, famille hiphop : 34 fichiers droits, 5 batteurs | 4/4, 91 (70–100), droit | piano électrique 4 (`drop2`), basse synthétique 38, kit 808/909 128/25 | oui (charleston seule) | non |
| `gmd_soul` (B8) | GMD, famille soul (soul, motown, groove3/4/9/10) : 28 fichiers, 4 batteurs | 4/4, 102 (80–105), droit | piano électrique 4 (`drop2`), basse au doigt 33, kit standard | oui | non |
| `gmd_reggae` (B8) | GMD, style reggae : 2 fichiers 4/4 droits, 2 batteurs (« juste » pour le style, 3 fichiers) | 4/4, 102 (83–121), droit | guitare claire 27 (skank vers le haut), basse au doigt 33, kit standard | oui (trois groupes de charleston) | non |
| `gmd_dance_disco` (B8) | GMD, style dance/disco : 5 fichiers, 2 batteurs | 4/4, 120 (120–137), droit | guitare claire 27 (contretemps), basse en octaves 33, kit standard | oui (sans relance : aucun fichier `fill`) | non |
| `gmd_afrobeat` (B8) | GMD, style afrobeat : 13 fichiers, 2 batteurs | 4/4, 98 (86–113), droit | guitare claire 27 (`guide_tones`), basse au doigt 33 en ostinato, kit standard | oui (sans relance) | non |
| `gmd_funk` (019, B4) | Groove MIDI Dataset v1.0.0, style `funk` (36 fichiers beat, 41 fills) ; basse : WJazzD `FUNK` (20 solos) | 4/4, 96 (80 à 130), droit | guitare claire 27 (notes guides), basse aux doigts 33 (riffs), kit standard 128/0 | oui (patterns, phrases et relances du GMD) | non (pas de mélodie) |
| `gmd_neworleans_funk_secondline` (019, B7) | Groove MIDI Dataset v1.0.0, styles `neworleans/funk` (8 fichiers beat, 40 fills) et `neworleans/secondline` (3 fichiers) ; basse et comping par règles de métier | 4/4, 93 (82 à 124), droit | piano 0 (drop 2), tuba 58 (riffs), kit standard 128/0 | oui (patterns, phrases et relances du GMD) | non (pas de mélodie) |
| `gmd_latin_brazilian_bossa` (019, B3) | Groove MIDI Dataset v1.0.0, style `latin/brazilian-bossa` (2 fichiers beat, 1 batteur, « trop peu ») ; basse : WJazzD `LATIN` (23 solos) ; comping et basse en règles de métier | 4/4, 127 (104 à 180, tiré par graine), droit | guitare nylon 24 (comping syncopé), basse acoustique 32 (fondamentale-quinte anticipée), kit jazz 128/32 ; habillage par graine (guitares, piano, basses, kits 0/8/32/40) | oui (cross-stick, charleston, ride ; sans relance : aucun fichier `fill`) | non (pas de mélodie) |
| `gmd_latin_brazilian_samba` (019, B3) | GMD, style `latin/brazilian-samba` (5 fichiers beat, 1 batteur, « juste ») et `fill` de `latin/samba` ; basse : WJazzD `LATIN` pour le registre ; surdo, tamborim, agogô, chocalho, pandeiro : règles de métier | 4/4 (deux mesures de 2/4), 89 (85 à 120), droit | guitare nylon 24, basse acoustique 32 (deux appuis), kit standard 128/0 ; habillage par graine | oui (surdo = tom 43, voix latines GM, relances du GMD) | non (pas de mélodie) |
| `gmd_afrocuban` (019, B3) | GMD, style `afrocuban` droit et `afrocuban/rhumba` (1 fichier chacun, « trop peu » par groupe) ; basse : WJazzD `LATIN` pour le registre ; **clave son 3-2 / 2-3, montuno, tumbao, congas, cloche : règles de métier** | 4/4, 105 (105 à 128), droit | piano 0 (montuno), basse acoustique 32 (tumbao), kit standard 128/0 ; habillage par graine | oui (claves, congas, cloche, timbales, shaker ; phrases alignées sur la clave) | non (pas de mélodie) |
| `gmd_latin_brazilian_baiao` (019, B3) | GMD, style `latin/brazilian-baiao` (3 fichiers beat, 2 batteurs, « juste » ; 29 fills) ; zabumba, triangle, agogô : règles de métier | 4/4, 95 (95 à 110), droit | accordéon 21 (comping), basse aux doigts 33, kit standard 128/0 ; habillage par graine | oui (ride, charleston, caixa, triangle, relances du GMD) | non (pas de mélodie) |
| `wjazzd_ballad` (019, B2) | WJazzD `BALLAD` (10 solos, 48,8 à 67,7 BPM : basse, tempo) ; batterie : GMD `jazz` lent (5 fichiers, 3 batteurs) ; swing : JTD (plateau, rien sous 102 BPM) ; comping, grilles, arc, habillage : règles de métier | 4/4, 58 (44–84), swing 0,72 | piano 0, guitare de jazz 26 ou orgue 16 ; contrebasse 32 (ou 35, 33) ; kit balais 128/40 ou jazz 128/32 (tirés par graine) | oui (ride mesuré ; balais et charleston : règles) | non (pas de mélodie) |
| `gmd_jazz_waltz` (019, B5) | GMD `jazz` 3/4 (2 fichiers, 120 et 200 BPM, un batteur) ; swing : JTD (4/4 porté au 3/4) ; basse, comping, grilles, arc, habillage : règles de métier | 3/4 sur 12 pas, 160 (100–230), swing 0,70 | piano 0 (ou 4, 26, 16) ; contrebasse 32 (ou 33, 35) ; kit jazz 128/32 ou balais 128/40 | oui (ride mesuré ; relances : règles) | non (pas de mélodie) |

Les « sept profils » cités plus bas désignent les sept premiers, réglés pendant les phases 009–012. Les trois
derniers (phase 013, R1) sont construits par le même chemin — références du labo, `mine-cells`, formules de
[Source des réglages distribués](#source-des-réglages-distribués-p3-t1-contrat-11) — par
`specs/013-moteur-dans-le-jeu/r1_profils.py` (`selftest` rejoue les formules sur quatre profils existants) ;
mesures et limites dans le plan 013, section R1.
Le onzième, `nt_desert` (phase 014, G7 à G7h), est le premier profil de jeu vidéo : désert de *Nuclear Throne*, voir `profiles/nt_desert/README.md` et `specs/014-styles-de-jeu/g7f-nt-desert-riche.md` (écriture composée sans sifflement, comparaisons avant/après) ; G7e et G7h (legato, registre, longueurs et timbre lisibles) sont résumés dans le README du profil et le tableau d'état du plan 014.
Les profils `isaac_*` sont des essais d'écoute de fin de phase 014 : mêmes mécanismes que `nt_desert`, aucun ajout moteur, voir `profiles/isaac_basement/README.md` (les trois autres suivent le même schéma ; leur en-tête de `profile.toml` résume l'écriture).
Phase 019, B8 : sept profils d'accompagnement sous grille construits sur le Groove MIDI Dataset (CC BY 4.0), `gmd_rock`, `gmd_pop`, `gmd_hiphop`, `gmd_soul`, `gmd_reggae`, `gmd_dance_disco`, `gmd_afrobeat` ; batterie générée par `specs/019-bibliotheque-accompagnements/b8/generer.py`, basse, comping et grilles en règles de métier (aucune source mesurée), arc sur quatre chorus, phrases et relances ; voir leurs README et `specs/019-bibliotheque-accompagnements/b8-profils-gmd.md`.
Phase 019, R1 : `nt_desert`, `isaac_basement`, `isaac_burning` et `isaac_downpour` embarquent des alternatives issues des propositions P1 (boucles générées, pédale à alternatives, gabarits de basse) et P2 (boucles, motifs et patterns extraits des transcriptions), avec leur provenance en commentaire TOML ; `TUNING_VERSION` 25. `isaac_cellar` est inchangé. Voir `specs/019-bibliotheque-accompagnements/r1-integration-p1-p2.md` et les README `nt_desert` et `isaac_basement`.
Phase 019, B4 et B7 : `gmd_funk` et `gmd_neworleans_funk_secondline` sont des profils sous grille (`loop`, sans piste `Melody`) qui activent l'arc (A3), les phrases et les relances (A1) : batterie en patterns, phrases et relances tirés du Groove MIDI Dataset (`specs/019-bibliotheque-accompagnements/b47_batterie.py`), basse en riffs (WJazzD `FUNK` pour le registre et les approches de `gmd_funk`), comping en règles de métier, plusieurs boucles de grille par défaut tirées par la graine. Voir leur `README.md` (provenance, mesure → valeur, limites) et `specs/019-bibliotheque-accompagnements/b47-funk-neworleans.md`.
Phase 019, R2-styles (`TUNING_VERSION` 29) : les neuf profils `gmd_*` (rock, pop, hip-hop, soul, reggae, disco, afrobeat, funk, New Orleans) n'ont plus de `[fixed_tonality]` et portent un `session.toml` : une graine tire un tempo dans une sous-plage de la source (centiles des BPM du GMD), une tonalité courante du style, le programme des accords et de la basse, le kit, la famille de voicings (avec `chords.color` et `register_drift`), le registre, l'énergie et l'arc (`[arc]` ou un arc nommé qui change les instruments de chorus en chorus) ; hip-hop et rock tirent aussi un swing léger (0,59 et 0,60, rapports médians des fichiers `swing` du GMD). La batterie joue des coups probabilistes (`probability` = fréquence mesurée) et un micro-timing par voix mesuré dans le GMD (`performance.tracks.Percussion*.voices`). Les valeurs de métier (tonalités, instruments, kits, voicings) sont dites comme telles dans chaque README. Le tempo de l'hôte, son swing et son `[fixed_tonality]` (la grille de training) gagnent sur la variante. Compte rendu : `specs/019-bibliotheque-accompagnements/r2-styles.md` ; données : `specs/019-bibliotheque-accompagnements/r2-styles/habillage.py`.
Phase 019, B3 : quatre profils latins sous grille (`loop`, sans `[fixed_tonality]` pour que la tonalité tirée par la session prime, sans piste `Melody`), tous à habillage par graine (`session.toml` : tempo, tonalité, comping, basse, kit, voicings, registre, énergie, arc nommé ; 28 à 33 variantes en 9 groupes), arc (A3), phrases et relances (A1) : `gmd_latin_brazilian_bossa`, `gmd_latin_brazilian_samba`, `gmd_afrocuban` (seul à `[clave]` : phrases de comping et de batterie alignées sur la clave son 3-2 / 2-3 tirée par air), `gmd_latin_brazilian_baiao`. Le Groove MIDI Dataset ne donne que la batterie d'un kit électronique (aucune percussion latine) : clave, montuno, tumbao, surdo, tamborim, triangle, comping et grilles sont des **règles de métier** dites comme telles dans chaque README ; la basse n'est calibrée sur WJazzD `LATIN` que pour le registre, le rythme harmonique et les approches. Scripts, preuves et comptes rendus : `specs/019-bibliotheque-accompagnements/b3/` ; mesures hors ligne : `mesures_latin.py`.

Phase 019, B2 et B5 : `wjazzd_ballad` (ballade AABA de 32 mesures, 48 à 78 BPM, balais dominants) et `gmd_jazz_waltz` (valse jazz 3/4, AABA de 32 mesures, 118 à 206 BPM) sont des profils sous grille avec `[session]` (habillage tiré par graine : tempo et swing, tonalité, programmes de comping, de basse et de kit, famille de voicings, registre, énergie, arc nommé), `[arc]`, `[swing]` (courbes JTD), phrases et relances. La batterie vient du GMD (`specs/019-bibliotheque-accompagnements/b25_batterie.py`) : patterns `ride` mesurés, patterns `brush` et `hat` de règles de métier (le GMD n'a pas de balais). Voir leur `README.md` et `specs/019-bibliotheque-accompagnements/b25-ballade-valse.md`. Manque de schéma : pas de feel de basse à une attaque par mesure (blanche pointée de la valse).
Phase 019, R2-jeux : les cinq profils de jeu (`nt_desert`, `isaac_basement`, `isaac_burning`, `isaac_cellar`, `isaac_downpour`) activent l'habillage par graine (`session.toml` : tempo en plages autour du tempo du niveau, timbres SoundFont et zones Odin permutées avec leur `level_db`, kit, famille de voicings, registre, énergie, tags, arc nommé), un arc sans retombée (`arc.toml`, `repeat_from`) et des phrases et relances de batterie tirées des patterns existants ; `isaac_cellar` reçoit en plus une pédale à alternatives et deux boucles générées. Une seule tonalité par profil : les références du niveau n'en attestent qu'une, ou la seconde (A phrygien de `nt_desert`) coûte plus d'un point de distance symbolique. `TUNING_VERSION` 27. Voir `specs/019-bibliotheque-accompagnements/r2-jeux.md`.

|---|---|
| `[meta]` | `id` ASCII minuscule, `name`, `tradition` précise (région, époque), `tags`, `corpus` (référence au manifeste du labo et aux droits), `aliases` optionnels : noms courts pour `--style` (ex. `["gigue", "jig"]`), uniques parmi les profils distribués. Les tags ne servent pas à choisir un style. |
| `[files]` | `cells`, `percussion`, `arrangement`, `dynamics`, `set`, `arc`, `session` : chemins relatifs au dossier, sans `..`. Omettre pour écrire les sections inline. |
| `[meter]` | `time_signature = {numerator, denominator}`, `steps_per_bar`, `steps_per_unit` (unité des cellules), `steps_per_pulse`, `pulses_per_bar`, `default_swing_ratio`, `default_bpm`, `min_bpm`, `max_bpm`. Tempo à la noire ; pulse dansée indépendante. |
| `[fixed_tonality]` | Optionnel : `key` (classe MIDI 0..11, do = 0) et `mode` (`ionian`, `mixolydian`, `dorian`, `aeolian`, `phrygian`, `harmonic_minor`) obligatoires. Verrouille la tonalité de tous les airs : priorité sur `melody.key_modes`, `dynamics.color`, pondération du set et bascules parallèles. Absent : comportement émotionnel inchangé. Voir « Tonalité verrouillée ». |
| `[form]` | `sequence` : indices 0-based des parties jouées (AABB = `[0,0,1,1]`). Chaque `[[form.parts]]` donne `name`, `melody_center_offset` en demi-tons et `roles`, un rôle fixe ou pondéré par mesure (voir « Règles de phrase »), plus `tracks_off` optionnel : pistes tues sur cette partie (`["Bass", "PercussionLow", "PercussionHigh"]`, défaut : aucune), lues par la même porte que l'arrangement et la densité. |
| `[melody]` | `cell_units` unités par cellule (= une mesure), `strong_units`, `weak_units` (accents 0-based), `key_modes = [{key, mode, weight}]` (classe MIDI 0..11). `params` règle tessiture, registre, continuité, vélocités et poids de rôles. `cells` contient une liste `[[cells]]` avec `role`, `notes`, `weight` ; seuls les rôles effectivement employés par la forme (et les rôles libres de poids positif) sont exigés. Notes : degrés `1..14`, octave grave `1,`..`7,`, `-` pour prolonger. |
| `[melody.difficulty]` | Optionnel (E10). Bornes de difficulté de la mélodie : saut en demi-tons, attaques par mesure, plus petite durée, ambitus d'une mesure, gamme, ornements, ajustement aux accords. Voir « Difficulté de la mélodie ». |
| `[harmony]` | `model = "melody_functional"`, `"rewrite_functional"`, `"loop"` ou `"none"`, `steps_per_chord` diviseur de la mesure. `melody_functional` : `[[harmony.candidates]]` par mode, `chords = [{degree, quality}]` ; chaque mode atteignable (`ionian`, `mixolydian`, `dorian`, `aeolian`, `phrygian`, `harmonic_minor`) est requis avec I et V. V majeur en ionien, phrygien et mineur harmonique (sensible pour les deux derniers), mineur en mixolydien, dorien et éolien. Le II suit la seconde du mode (bII phrygien). Jazz : `[[harmony.rules]]`, `max_recursion` (0..16, défaut 0), `scale_guidance` (défaut false). Boucle : `parts` par nom de partie, accords relatifs à la tonique avec qualité, basse et durée en pas, voir « Harmonie en boucle et pédale ». |
| `[accompaniment]` | Vote harmonique (`strong_steps`, poids, `tie_priority`), voix `[accompaniment.bass]` (`hits`, `motifs`, `passing` ou `walking`) et `[accompaniment.chords]` (`patterns` ou `comping`, `voicing`, `spread`) décrites ci-dessous. |
| `[percussion]` | Notes MIDI, vélocités, gain, humanisation et `[[patterns]]` par densité 0, 1, 2 (`percussion.toml` externe) : `weight` entier strictement positif (défaut 1), `hits = [{step, track="PercussionLow"\|"PercussionHigh", delta_velocity}]` **ou** `euclid = {pulses, steps, rotation}` avec `track = "PercussionLow"` ou `"PercussionHigh"` (défaut PercussionLow). Plusieurs patterns d'une densité forment les alternatives pondérées ; voir « Batterie par graine ». Rotation entière, positive vers la droite ; `steps` égale la grille, `0 <= pulses <= steps`. Durée zéro = déclenchement de sample. |
| `[instruments]` | Programmes SoundFont 0-based : `melody_program`, `countermelody_program` optionnel, `chords_program`, `bass_program`, `percussion_program` et `percussion_bank`. |
| `[[odin.<Piste>.zones]]` | Optionnel. Zones d'émotion ordonnées d'une piste (`Melody`, `Chords`, `Bass`, `Countermelody`) : `when`, `preset`, `level_db`, `glide_ms`, `transpose` et `morph`. Dernière zone sans `when` obligatoire. Voir « Instrument Odin par piste ». |
| `[arrangement]` | `tours_per_tune`, `entries` par piste (`tour`, `part`, `pass`). `PercussionLow` et `PercussionHigh` sont requis ; toute autre piste (`Melody` comprise) peut entrer plus tard ou être omise. Une piste entrée continue à jouer. Les entrées ne s'appliquent qu'au premier air de chaque set (voir « Forme longue »). |
| `[arrangement.alternation]` | Optionnel (E11). Par piste déclarée dans `[entries]`, une alternance de mesures jouées et muettes : `period` (cycle en mesures), `play` (mesures jouées, défaut la moitié), `phase`. Voir « Alternance de mesures (question-réponse) ». |
| `[arrangement.intro]` / `[intro]` du fichier d'arrangement | Formes pondérées d'intro : rôle `percussion`, `tail` ou `vamp`, longueur en mesures, levée en unités. Intro complète au premier air de chaque set ; raccords des suivants dans `[set.transitions]`. |
| `[dynamics]` | Courbes par axe émotionnel : intensité, tempo, vélocités, densité, longueur de note et substitutions harmoniques ; seuils et frontière de changement de mode. Sans `fixed_tonality`, chaque mode atteignable (`key_modes`, `color`, `parallel_modes`) doit avoir sa bascule claire ↔ sombre : phrygien ↔ mixolydien, mineur harmonique ↔ ionien (majeur le plus proche, degrés altérés minimaux). Chaque courbe est une liste de points `[entrée, sortie]`, ordonnée ; extrémités constantes. Voir `dynamics.toml` pour les noms complets. |
| `[mix]` | Gain de bus, salle, niveaux/placement/envois par piste et EQ stéréo ; schéma et bornes ci-dessous. |
| `[set]` (`set.toml`) | Longueur, tonalités, raccords, clôture, rampe par air et mémoire signature ; section « Forme longue (set) » ci-dessous. |
| `[arc]` (`arc.toml`) | Optionnel. Arc sur plusieurs chorus ; section « Arc sur plusieurs chorus ». |
| `[session]` (`session.toml`) | Optionnel. Variantes tirées par graine (tempo, tonalité, instruments, voicings, swing, registre, tags, arc, Odin) ; section « Habillage par graine ». |

Le chargeur vérifie les références, bornes, copies antérieures, métrique et banques
avant l'installation. Les erreurs nomment le chemin du champ fautif. Un modèle
d'harmonie non implémenté est refusé : aucune substitution silencieuse.

## Tonalité verrouillée

Pour un accompagnement piloté par une grille relative à la tonique, ajouter au
`profile.toml` (ou au profil autonome) :

```toml
[fixed_tonality]
key = 5
mode = "ionian"
```

Tous les airs restent ici en fa ionien, même quand la valence change. Le verrou ne
demande pas de mélodie active et ne remplace pas les autres sections du profil.
Ses deux champs sont requis ; une tonique hors de 0..11, un mode inconnu ou un
champ supplémentaire sont refusés. En harmonie `melody_functional`, les candidats
d'accords du mode fixé restent requis ; aucune bascule parallèle de ce mode n'est
nécessaire. Les banques et courbes déclarées restent validées.

Le verrou fixe la **tonalité**, pas toutes les réactions émotionnelles : registre,
vélocités, timbres et autres axes restent actifs. Une grille `loop` conserve ainsi
ses accords relatifs, chromatiques ou par degrés ; le verrou ne transforme pas
une grammaire `rewrite_functional` sensible à la valence en grille écrite.

Pour transposer, reconstruire le profil avec une autre valeur de
`fixed_tonality.key` et l'envoyer par `EngineCommand::SetStyleProfile` : le host
l'applique à la prochaine mesure, sans seek ni réinitialisation de la graine.
Aucune nouvelle commande ; aucune modification des profils distribués.

## Batterie par graine

Dans `percussion.toml`, `[[patterns]]` correspond à `percussion.patterns` du profil
chargé. Chaque densité 0, 1, 2 doit être couverte ; plusieurs entrées d'une
même densité constituent une banque, non une superposition :

```toml
[[patterns]]
density = 1
weight = 2
hits = [
  { step = 0, track = "PercussionLow" },
  { step = 8, track = "PercussionHigh" },
]

[[patterns]]
density = 1
weight = 1
hits = [
  { step = 0, track = "PercussionLow" },
  { step = 6, track = "PercussionLow", delta_velocity = -18 },
  { step = 8, track = "PercussionHigh" },
]
```

`weight` vaut 1 s'il est omis et doit être strictement positif. Les
`[[cadence_patterns]]` restent uniques par densité : ils remplacent la grille
ordinaire aux fins de phrase et ne participent pas au tirage pondéré.
Les choix pondérés sont stables pour chaque partie et densité d'un air ;
un changement de densité puis un retour retrouve donc la même grille.
Les flux dérivés de la graine sont séparés pour batterie et harmonie :
ajouter une variante de batterie ne choisit pas un autre accord. Un
candidat unique n'entraîne aucun tirage. La fixture `gigue_bourdon`
emploie ces alternatives hors des styles de jeu.

## Harmonie en boucle et pédale

`model = "loop"` impose l'harmonie par partie, sans vote mélodique ni cadence
ajoutée. Chaque nom de `form.parts` doit avoir exactement une entrée dans
`harmony.parts` ; les noms inconnus sont refusés. Les autres modèles refusent
une table `parts` non vide.

```toml
[harmony]
model = "loop"
steps_per_chord = 8

[harmony.parts.A]
kind = "pedal"
degree = 1
# neighbor_semitones = 1

[harmony.parts.B]
kind = "chord_loop"
loops = [
  { weight = 2, chords = [
    { degree = 1, quality = "m", steps = 32 },
    { semitones = 1, quality = "", steps = 32 },
  ] },
  { weight = 1, chords = [
    { degree = 1, quality = "m", steps = 32 },
    { degree = 5, quality = "7", steps = 32 },
  ] },
]
```

### Un accord : racine, qualité, basse, durée

Un accord s'écrit **relativement à la tonique** : aucun symbole absolu (`G7`)
dans un profil. La tonique vient de la tonalité du profil ou de
`[fixed_tonality]` ; changer l'une transpose toute la grille.

```toml
{ semitones = 2, quality = "m7",  steps = 8 }            # ii  : Dm7 en do
{ semitones = 7, quality = "7b9", bass = 11, steps = 8 } # V7b9 sur la sensible (G7b9/B)
{ semitones = 0, quality = "",    bass = 4,  steps = 16 } # I sur la tierce (C/E)
```

- **Racine** : **exactement un** champ, `degree` numérique modal `1..7` (1 =
  tonique) ou `semitones` chromatique `0..11` au-dessus de la tonique. Les
  degrés suivent les intervalles du mode actif : le degré 2 vaut un demi-ton
  en phrygien, deux en ionien ; les offsets chromatiques restent fixes. Lors
  d'une bascule de couleur par `dynamics.color` ou `parallel_modes`, les degrés
  sont recalculés dans le nouveau mode ; la qualité et la basse ne changent pas.
- **`quality`** : suffixe de symbole, texte requis (`""` = triade majeure).
  C'est le même texte que `Chord::from_name` lit, parseur unique
  (`theory/quality.rs`). Tout autre texte est refusé au chargement.
- **`bass`** (optionnel) : basse d'un accord slash, en demi-tons `0..11` au-dessus
  de la tonique.
- **`steps`** : durée **en pas**, positive et **multiple de
  `harmony.steps_per_chord`** ; omission : un créneau (`steps_per_chord`). La somme
  des durées de **chaque alternative** doit être exactement la longueur de la
  partie en pas (`nombre de rôles × meter.steps_per_bar`).

**Qualités.** Tête, puis altérations facultatives :

| Famille | Suffixes |
|---|---|
| Triades | `""` (majeure), `m`, `dim`, `+`, `sus2`, `sus4`, `add9` |
| Sixtes | `6`, `m6`, `6/9`, `m6/9` |
| Septièmes | `7`, `maj7`, `m7`, `m7b5`, `dim7`, `mMaj7`, `7#5`, `7b5`, `7sus4`, `7sus2` |
| Tensions naturelles | `9`, `11`, `13`, `maj9`, `maj13`, `m9`, `m11`, `mMaj9`, `m(add9)`, `9sus4`, `13sus4` |
| Altérations d'une tête dominante `7`, `9`, `11`, `13` | `b9`, `#9`, `#11`, `b13`, suivies de `b5` ou `#5` après `7` seul : `7b9`, `7#9#11`, `9#11`, `7b9b5`, `7#9#5` |
| Altéré | `7alt` (fondamentale, tierce, septième et les altérations b9, #9, #11, b13) |
| Synonymes lus, jamais écrits | `maj`, `aug`, `add2`, `m(b5)`, `sus` |

`9`, `11` et `13` sous-entendent la pile naturelle (`13` = 7ᵉ, 9ᵉ, 13ᵉ ; `11` =
7ᵉ, 9ᵉ, 11ᵉ). Sont refusés : une altération qui double la pile ou la quinte
(`9b9`, `13b13`, `7b5#5`, `7#5b13`), une altération sur une tête non dominante
(`m7b9`, `maj7#11`), `b5`/`#5` sur une pile (`9b5`) et tout texte inconnu
(`Major`, `69`, `7 b9`). L'écriture `Major` / `Minor` des anciennes boucles est
retirée sans alias : `""` et `m`.

**Voicing.** Une triade garde exactement ses trois notes et la conduite de voix
serrée existante. Une septième ou une qualité à tensions se joue à **quatre ou
cinq hauteurs** (au plus 5, sur la pile) : les tensions s'empilent au-dessus de
l'octave (9 = 14 demi-tons, 13 = 21) ; au-delà de cinq notes la quinte tombe
d'abord, puis les tensions les plus hautes. L'accord altéré se joue fondamentale,
tierce, septième, b9, #9. Pas de doublure d'octave pour ces voicings :
`chords.note_count` ne double que les triades.

**Basse.** `Bass` joue la **basse** de l'accord : la fondamentale, ou `bass`
pour un accord slash. Un coup `degree = "fifth"` reste la quinte de la
**fondamentale**. Un motif `reference = "chord_root"` prend la basse de l'accord
comme référence (les décalages en demi-tons ou en degrés s'y ajoutent tels
quels). `Chords` joue un accord slash **sur sa basse** : si la basse est une note
du voicing, seules les inversions qui la mettent au grave sont retenues ; sinon la
basse s'ajoute sous un voicing limité à quatre notes. Les accords non slash
choisissent librement leur renversement. `voicing = "fixed_triad"` refuse les
qualités autres que `""` et `m` nues et les accords slash : il demande
`close_triad`.

**Plusieurs accords par mesure.** La grille harmonique est celle de
`steps_per_chord` : un créneau par `steps_per_chord` pas. Un ii–V dans une seule
mesure de 16 pas demande `steps_per_chord = 8` (ou 4) et deux accords de
`steps = 8`. Les changements d'accord sont publiés dans `chord_changes` à
chaque pas où l'accord change, plus au pas 0 de chaque mesure. Avec
`steps_per_chord` égal à la mesure (cas des profils distribués), chaque mesure
porte un seul accord comme avant.

`loops` et `generated` (voir « Procédural par règles ») forment la liste, non vide, des
alternatives de la partie. Chaque `weight` est un entier
strictement positif (défaut 1) ; chaque `chords` est non vide. L'ancien
`chords` directement dans `harmony.parts.<nom>` est refusé, sans alias, comme
l'ancienne durée en mesures (`bars`, `default_bars_per_chord`).
Une boucle est tirée par partie et par air, selon la graine et les poids ;
les reprises de la partie gardent ce choix. Une seule alternative ne
consomme aucun tirage. Les reprises, secondes fins et variations de forme
utilisent la même position dans la boucle. Pas de turnaround implicite :
une section distincte porte un éventuel raccord explicite.

`kind = "pedal"` n'accepte ni `quality` ni liste d'accords : c'est une vraie
fondamentale seule, sans tierce ni quinte, jouée sur `Chords` par les coups
habituels. `neighbor_semitones = 1..11` ajoute facultativement une seule note,
à cet intervalle chromatique au-dessus de la fondamentale. Pour une pédale
tenue, employer un coup `direction = "block"`, `step = 0`, une `duration` égale
à `meter.steps_per_bar` et `tie = true` dans chaque motif sélectionnable de
`accompaniment.chords` (voir « Accords grattés »). Les notes contiguës de même
hauteur sont liées sans réattaque ; un changement de fondamentale ferme la
précédente. Éviter gate raccourci, respiration ou décalage sur `Chords` pour
préserver cette coïncidence. Le registre vient de `accompaniment.chords.center`
et le timbre de `instruments.chords_program` ; par exemple Saw Lead GM 82
s'écrit `chords_program = 81` (0-based), avec un registre grave.

Les triades gardent la polyphonie `chords.note_count` et la conduite de voix
existantes ; une pédale seule reste une note, son voisin une dyade, sans
doublure automatique en octaves. `Bass` suit la fondamentale (ou la quinte
selon ses coups), sans lui imposer une tenue : la scie continue est sur
`Chords`, pas sur une basse percussive qui décroît.
Le voisin reste au-dessus de la pédale, sans renversement de la dyade.
Avec `voicing = "fixed_triad"`, la fenêtre `low..high` doit couvrir les
douze fondamentales et ce voisin (`high - low >= 11 + neighbor_semitones`).

Le contexte du premier accord est publié dans les champs habituels du
snapshot ; `chord_changes` porte le pas 0 et chaque changement d'accord de la mesure. Les cellules mélodiques fournissent
toujours rythme et contour ; comme pour `rewrite_functional` sans `scale_guidance`, leurs
porteuses sont rapprochées des hauteurs de la sonorité courante avant
interprétation et ornements. Une pédale pure contraint donc les porteuses à
sa seule classe de hauteur ; ses ornements restent ceux du mode actif.
La tension ne substitue pas de dominante aux accords déclarés. Les intros
vamp et l'outro tonique restent les raccords explicites de l'arrangement ;
la boucle décrit les mesures du corps de l'air.

## Procédural par règles (option, phase 018 P1)

Quatre mécanismes **optionnels** s'ajoutent aux données écrites sans les remplacer. Un profil
qui n'en active aucun garde ses notes octet pour octet. Depuis R1 (phase 019), `nt_desert` (boucles générées, pédale à
alternatives) et `isaac_burning` (boucles générées, gabarits de basse) en activent, ainsi que `isaac_cellar` depuis R2 (pédale à
alternatives, boucles générées) ; les autres profils distribués n'en activent aucun. Tous sont nommés par le rôle musical, tirent par le `ChaCha8Rng` injecté (entiers
`u64`, principe II) et sont refusés au chargement avec le champ en cause s'ils sont
insatisfaisables. Compte rendu, preuves et recommandations par profil :
`specs/018-grille-et-moteur-neutre/p1-procedural-regles.md`.

### Boucles d'accords générées par contraintes (`generated`)

Dans une partie `kind = "chord_loop"`, `generated` s'écrit **à côté** de `loops` (l'une des deux
au moins est non vide) et partage le même tirage pondéré : **une** alternative par partie et
par air, selon la graine et les poids, qui se rejoue à l'identique aux reprises et aux tours.
Une boucle générée est tirée **une fois**, puis **répétée** jusqu'à la fin de la partie : la
forme reste lisible.

```toml
[harmony.parts.A2]
kind = "chord_loop"
# Quatre accords mineurs, mouvement chromatique, sur pédale de ré (tonique + 6) :
generated = [{ weight = 1, bars = 4, count = 4, qualities = ["m"],
               motion = { kind = "chromatic" }, start = 6, pedal = 6 }]
```

| Champ | Sens |
|---|---|
| `weight` | poids du tirage (défaut 1, strictement positif) |
| `bars` | longueur de la boucle en mesures, **diviseur** de la longueur de la partie (omis : la partie entière, aucune répétition) |
| `count` | nombre d'accords, de 1 au nombre de créneaux `steps_per_chord` de la boucle ; les créneaux se répartissent aussi également que possible, les `créneaux mod count` premiers accords en recevant un de plus |
| `qualities` | qualités permises (texte de suffixe F1 : `""`, `m`, `7`, `m7`, `7b9`…) ; une qualité est tirée uniformément pour chaque accord (répéter une qualité la pondère) |
| `motion` | mouvement de la **fondamentale** d'un accord au suivant, voir ci-dessous |
| `start` | fondamentale du premier accord, demi-tons au-dessus de la tonique (omis : la tonique) |
| `pedal` | pédale de basse : demi-tons au-dessus de la tonique, basse de **tous** les accords (accords slash quand la fondamentale s'écarte de la pédale) |
| `key_fit` | accords communs avec la tonalité : `free` (défaut), `within` (toutes les notes de l'accord, basse comprise, dans la gamme du mode actif), `outside` (au moins une note hors gamme) |

`motion` (champ `kind`) :

| `kind` | Intervalle de fondamentale | Options |
|---|---|---|
| `chromatic` | un demi-ton | `direction = "up" \| "down" \| "either"` (défaut) |
| `fifths` | une quinte juste (+7 ou −7) | `direction` |
| `scale_steps` | une note de la gamme du mode actif (depuis une fondamentale hors gamme : la note de gamme voisine) | `direction` |
| `intervals` | intervalles permis et pondérés : `intervals = [{ semitones = 5, weight = 3 }, { semitones = -3 }]` (demi-tons signés `-11..=11`, 0 répète la fondamentale ; poids défaut 1) | |

**Algorithme.** Une table de faisabilité est calculée à rebours (quelles fondamentales laissent
une suite complète possible à chaque position, compte tenu de `motion` et de `key_fit`) ; le tirage
avance ensuite sans rejet : qualité uniforme parmi celles que `key_fit` permet sur la
fondamentale, puis fondamentale suivante pondérée parmi les successeurs faisables. Aucune
boucle partielle, aucune boucle impossible. Le mode actif est celui de la tonalité ; une bascule
de couleur (`dynamics.color`, `parallel_modes`) régénère la boucle avec la même graine dans le
nouveau mode, comme les degrés d'une boucle écrite se recalculent. Avec `fixed_tonality`, le mode
est fixe.

**Validation** (champ nommé dans l'erreur) : `weight`, `bars` (diviseur de la partie), `count`
(1..créneaux), `qualities` (1..64), `start`, `pedal` (0..11), `motion.intervals` (non vide, poids
> 0), `fixed_triad` (qualités majeures et mineures nues, sans pédale), puis la **faisabilité dans
chaque mode atteignable** : `key_fit` si aucune qualité permise ne convient à la fondamentale de
départ, `motion` si aucune suite de `count` accords ne respecte le mouvement et `key_fit`.

### Pédale à alternatives

Une partie `kind = "pedal"` accepte `weight` (défaut 1) et `alternatives` : d'autres pédales
(`degree` ou `semitones`, `neighbor_semitones`, `weight`), tirées **une fois par partie et par
air**. La pédale écrite directement dans la partie est la première alternative.

```toml
[harmony.parts.R]
kind = "pedal"
degree = 1
weight = 3
alternatives = [{ weight = 1, semitones = 7 }]
```

Sans `alternatives`, aucun tirage. La voix de basse d'une partie lit la pédale tirée.

### Basse par gabarits (`accompaniment.bass.templates`)

Option **à côté** des `motifs` écrits : les gabarits entrent dans le même pool pondéré (mêmes
poids, même portée `part`, mêmes règles de longueur multi-mesures, copies et secondes fins). Sans
gabarit, le pool est exactement la liste des motifs et le flux de tirages est inchangé. Un
gabarit est un **rythme d'une mesure** réalisé sur l'accord du pas, hauteur par hauteur, sans
tirage à la réalisation : une mesure se rejoue à l'identique. Exclusif de `hits` et de `walking`.

```toml
[[accompaniment.bass.templates]]
part = "C"          # optionnel, comme `motifs[].part`
weight = 3
bars = 1            # mesures du cycle de `ramp`, 1..=16
rhythm = [          # attaques de la mesure : pas, durée, vélocité (sans chevauchement)
  { step = 0, duration = 2, velocity = 96 },
  { step = 8, duration = 2, velocity = 96 },
]
cycle = [{ degree = "root", octave = 1 }, { degree = "fifth", octave = 1 }]
pickup = { lead = 2, duration = 2, velocity = 90 }
```

| Champ | Sens |
|---|---|
| `rhythm` | attaques d'une mesure ; la hauteur de chacune vient de l'accord du **créneau** qui contient son pas (plusieurs accords par mesure possibles) |
| `cycle` | hauteurs jouées tour à tour par les attaques : l'attaque `n` de la mesure `b` du cycle joue `cycle[(b × attaques + n) mod longueur]`. `degree` : `root` (basse de l'accord, accord slash compris), `fifth` (quinte de la **fondamentale**), `fifth_if_held` (quinte si l'accord n'a pas changé depuis le créneau précédent, ou au premier créneau de la mesure) ; `octave` : −2..2. Omis : la basse de l'accord. Octaves : `[{root}, {root, octave = 1}]` ; quinte-fondamentale : `[{root}, {fifth}]` |
| `ramp` | montée ou descente : `stride` (signé, non nul, ≤ 12) × rang de la mesure dans le cycle, en `motion = "semitone"` (défaut) ou `"scale_step"` (notes de la gamme du mode actif). Le rang repart à 0 tous les `bars` mesures |
| `pickup` | **relance vers la fondamentale** : une note courte (`duration` ≤ `lead`, `velocity`) jouant la basse de l'accord **suivant** (`degree` : `root` ou `fifth`) `lead` pas avant le changement ; les attaques qu'elle recouvre sont retirées ou raccourcies. `when = "chord_change"` (défaut) : avant chaque changement d'accord, y compris au passage à la mesure suivante ; `"bar_end"` : avant chaque barre, même si l'accord est tenu, sans relance interne. Aucune relance après la dernière mesure. `lead` < `harmony.steps_per_chord` |

Une montée d'un demi-ton par mesure sur une tenue : `bars = 4`, `rhythm = [{ step = 0, duration = 22,
velocity = 92 }]`, `ramp = { stride = 1 }`. La hauteur résultante doit rester dans 0..127 pour
toutes les classes de hauteur du registre `low..high` (validation, champ du gabarit).

### Batterie par partie ou par tour (`percussion.patterns[].part` / `.tour`)

Un pattern ordinaire accepte une **portée** optionnelle : `part = "<nom de form.parts>"` et/ou
`tour = n` (tour 1-based de l'air). Pour une partie, un tour et une densité, ne concourent que
les patterns de la densité dont la portée convient ; parmi eux, **seuls les plus spécifiques**
(deux portées, puis une, puis aucune) sont tirés par poids, une fois par partie, tour et densité
et par air. Deux portées de même rang (une `part` et un `tour` concernant la même mesure) se
partagent donc le tirage. Chaque densité 0, 1, 2 requiert au moins un pattern **sans portée** (le
repli) ; l'intro et les tours sans pattern dédié l'utilisent. Les `[[cadence_patterns]]` n'ont pas
de portée. Sans portée dans la table, le choix est celui de `Batterie par graine`.

```toml
[[patterns]]            # repli
density = 1
hits = [{ step = 0, track = "PercussionLow" }, { step = 8, track = "PercussionHigh" }]
[[patterns]]            # seulement dans la partie C2
density = 1
part = "C2"
hits = [{ step = 0, track = "PercussionLow" }, { step = 6, track = "PercussionLow" }, { step = 8, track = "PercussionHigh" }]
[[patterns]]            # seulement au tour 2
density = 1
tour = 2
hits = [{ step = 4, track = "PercussionLow" }, { step = 12, track = "PercussionHigh" }]
```

Un pattern ordinaire accepte aussi `tag = "..."` : une étiquette libre, sans effet sur le choix
ordinaire, que `[arc]` nomme pour choisir les patterns d'un chorus (charleston, puis ride). Les
suites de patterns sur plusieurs mesures (`[[phrases]]`) et les relances de fin de phrase
(`[[fills]]`, qui remplacent `[[cadence_patterns]]` à la densité qu'elles déclarent) sont décrites
dans « Phrases d'accompagnement sur plusieurs mesures ».

Fixture de contrat : `tests/fixtures/profiles/regles_procedurales/` (les quatre mécanismes ;
tests : `harmonium_core/tests/procedural_rules_tests.rs`).

## Identité d'accord publiée dans le snapshot

Tous les modèles d'harmonie (`loop`, `melody_functional`, `rewrite_functional`, `none`)
publient l'identité complète de l'accord en vigueur, en classes de hauteur **absolues**
(`0..=11`, `0` = do, tonalité verrouillée comprise). Les champs de la mesure décrivent le
premier accord ; chaque entrée de `chord_changes[].context` porte les mêmes valeurs sous les
noms sans préfixe :

| `MeasureSnapshot` | `chord_changes[].context` | Contenu |
|---|---|---|
| `chord_root_pc` | `root_pc` | fondamentale |
| `chord_bass_pc` | `bass_pc` | basse ; la fondamentale hors accord slash |
| `chord_quality` | `quality` | qualité typée, en texte de suffixe (`""`, `m7`, `7b9`, `7alt`…), celui de `chord_name` ; `null` sans qualité |
| `chord_extensions` | `extensions` | intervalles des tensions au-dessus de la fondamentale (`G7b9` → `[1]`, `C13` → `[2, 9]`), croissants |
| `chord_pitch_classes` | `pitch_classes` | notes de l'accord, **tensions comprises**, fondamentale en tête puis intervalles croissants |

`chord_name`, `chord_root_offset` et `chord_is_minor` restent publiés, inchangés.

- `pitch_classes` contient les tensions : `G7b9` donne `[7, 8, 11, 2, 5]`. Un joueur qui
  tombe sur le b9 joue une note de l'accord tel qu'il est écrit, et c'est ce que le
  générateur considère dans l'accord pour la mélodie. Le noyau sans tension se retrouve en
  retirant les `extensions` (`(root_pc + intervalle) % 12`).
- La basse d'un accord slash n'est dans `pitch_classes` que si elle appartient à l'accord
  (`C/E` : `[0, 4, 7]` ; `D/C` : `[2, 6, 9]`, la basse `0` est seulement dans `bass_pc`).
- `quality = null` : pédale (`pitch_classes` = la hauteur tenue, plus son voisin
  éventuel) et mesures sans accord joué (intro, outro, harmonie `none` : `root_pc` et
  `bass_pc` = la tonique de l'air, `pitch_classes` vide).
- `quality = ""` est la triade majeure ; `"m"` la mineure.

`scale_guidance = true` publie les gammes (`ScaleGuidance`, Lydian Chromatic) pour `loop` et
`melody_functional` comme pour `rewrite_functional` : même calcul, à partir de l'accord typé
et de la tension harmonique de la mesure. `chord_tones` y reste le **type seul**
(`G7b9` → `[7, 11, 2, 5]`), sans les tensions. Absent ou `false` : `null`. Les gammes d'un
accord de boucle ne modifient pas les notes ; une pédale n'a pas de gammes.

Les clients lisent les nouveaux champs ; un snapshot sans eux se relit (valeurs par défaut).
Rupture de contrat : `TUNING_VERSION` 23.


## Instrument Odin par piste (option `odin`)

Chaque piste garde son **programme SoundFont**, toujours requis : c'est le rendu
de tout binaire compilé sans `odin` (1837, par défaut). Une piste peut en plus
déclarer de une à quatre zones Odin, en données (aucun `preset` direct sur la piste) :

```toml
[[odin.Bass.zones]]
when = { tension = [0.6, 1.0] }
preset = "basse_psychedelique"
level_db = -5.0
glide_ms = 80.0
morph = [
  { param = "fil1_freq", arousal = 3000.0, tension = 1500.0 },
]

[[odin.Bass.zones]]
preset = "basse_saturee"
morph = [
  { param = "fil1_freq", arousal = 3500.0, tension = 1500.0 },
  { param = "fil1_res", tension = 0.3 },
]
```

- **Zones et priorité.** La dernière zone est obligatoirement le défaut sans
  `when` ; toutes les précédentes ont un `when` non vide. Chaque axe renseigné
  est une paire de bornes inclusives, finies et ordonnées : `valence` dans
  `-1..1`, `arousal`, `density` et `tension` dans `0..1`. Les axes sont
  conjoints (ET), les axes omis ne contraignent rien. Au chargement, la
  première zone admissible gagne, sinon le défaut final.
- **Hystérésis de 0,03.** Avec une zone déjà active, entrer dans une nouvelle
  zone exige d'en franchir les bornes resserrées de 0,03, y compris pour
  quitter le défaut. Une zone bornée active reste dans ses bornes élargies
  de 0,03 ; une zone plus prioritaire peut toutefois la remplacer dès que
  ses bornes resserrées sont satisfaites. Le resserrement ne dépasse jamais
  le point médian de chaque intervalle (même une zone ponctuelle reste
  accessible) et ne resserre pas les extrêmes du domaine.
  Ainsi `tension = [0.6, 1.0]` entre initialement à 0,6, entre depuis le
  défaut à 0,63, et reste active jusqu'à 0,57 inclus.
- **Niveau propre à la zone.** `level_db` vaut 0 par défaut ; valeur finie dans
  `-60..12` dB, appliquée en gain linéaire `10^(level_db/20)` en plus du mix
  de piste, sans modifier les paramètres du preset. Deux zones peuvent
  partager un preset et différer par leur niveau ou leur morphing.
  `isaac_burning` réunit A par défaut (Bass Crusher [RM] + Haunt [RM]) et B
  sous tension (Bass Low [RM] −5 dB + Moonshade −7 dB).
- **Registre du preset.** `transpose` (demi-tons, `-24..24`, défaut 0) décale
  seulement les notes envoyées au preset de la zone ; partition, MIDI et
  SoundFont ne changent pas. Beaucoup de presets de basse d'usine sonnent une
  ou deux octaves sous la note jouée (Growly FM Bass, Bass Low, Sausage
  Bassener, BS PWM Bass : deux ; DeepBass, Analog Bass, Bass Simple PM : une,
  mesuré par suivi de hauteur sur la relance de `nt_desert`) : sans
  correction, une ligne écrite en D2–G#2 tombe sous 50 Hz et ses notes ne se
  distinguent plus. La touche jouée est figée à l'attaque : un morphing ou une
  bascule de zone ne change pas la fin d'une note en cours.
- **Legato mono et glissando (G2).** Une note `Legato` enchaînant une autre
  hauteur tenue réutilise les enveloppes Odin, sans nouvelle attaque. La
  première note, une note après silence, une répétition de même hauteur et
  toute note `Normal` réattaquent, sans glissando depuis la note précédente.
  `Tied` reste une prolongation sans `NoteOn`. Sur SoundFont, rien ne change :
  le lecteur conserve son chevauchement de 0,2 pas, sans portamento.
  `glide_ms`, **par zone**, vaut **80 ms par défaut** ; `0` désactive seulement
  le glissement de hauteur, pas la liaison d'enveloppe. Sinon : **40 à 500 ms**,
  valeur finie. C'est le temps **nominal pour parcourir 90 % de l'écart en Hz
  à 44 800 Hz**, pas une rampe linéaire terminée à cette échéance : Odin
  utilise un lissage exponentiel borné, dont la durée réelle dépend aussi de
  son adaptation au taux d'échantillonnage. Le réglage s'applique au prochain
  enchaînement, indépendamment du tempo ; il ne se morphe pas par émotion.
  La commande native est calculée hors callback, jamais dans le rendu.
  Les pistes ayant du `Legato` dans leurs données musicales sont préparées
  hors callback en mode mono/legato (actuellement, les motifs de `Bass` sont
  l'unique source de cette articulation). Les autres gardent leur mode de
  preset et leur rendu antérieur, sans activation implicite du glide.
  Aucun second interrupteur dans les profils.
- **Legato à une bascule de zone.** Les enveloppes de deux instances ne se
  transfèrent pas : T1 garde son fondu audio de **300 ms**, avec une seule
  réattaque de la tenue sur la nouvelle instance. Si une nouvelle hauteur
  `Legato` arrive exactement à la frontière, cette tenue est d'abord ouverte,
  puis glisse vers la nouvelle hauteur, sans seconde attaque d'enveloppe.
  Une attaque `Normal` remplace le transfert mono ; une fin exactement à la
  frontière l'annule. Si un glide est déjà en cours à la frontière, la nouvelle
  instance repart de la **hauteur cible** tenue, pas de la fréquence
  intermédiaire de l'ancienne. Le fondu mélange leurs sorties, jamais leurs
  états DSP. Les anciennes fins différées ne coupent pas la nouvelle hauteur.
  Preuves et limites : [`g2-glissando-odin.md`](../specs/016-variete-et-timbres/g2-glissando-odin.md).
- **Choix à l'exécution.** Sans `odin` compilé, ou si l'hôte ne fournit pas le
  preset de la zone active, la piste joue son programme SoundFont. Avec `odin`
  et le preset présent, elle joue Odin. Aucun nom de style ni canal fixe : la clé est la
  piste (`Melody`, `Chords`, `Bass`, `Countermelody`) ; `PercussionLow` et `PercussionHigh` restent
  SoundFont.
- **Presets fournis par l'hôte**, comme les octets SF2 : `create_offline_engine`
  / `LiveHost::start` reçoivent des `OdinPresetSource { name, bytes }`
  (`name` = nom du fichier sans `.odin`) ; CLI : `--odin-presets <dossier>`,
  sous `--features odin`. **Aucun fichier `.odin` ni code GPL dans ce dépôt**
  (licence MIT OR Apache-2.0) ; `odin2-core` (GPLv3) est une dépendance git
  optionnelle : un binaire compilé avec `odin` est GPL.
- **Morphing par les 4 émotions.** `valeur = socle du preset + arousal×a +
  valence×v + density×d + tension×t` (`valence` dans `-1..1`, autres axes `0..1`), puis bornée.
  Noms exacts du catalogue odin-rs, sans alias :
  `osc1_vol`/`osc2_vol`/`osc3_vol` (−60 à +12 dB),
  `osc1_detune`/`osc2_detune`/`osc3_detune` (0 à 1) ;
  pour chacun des filtres `fil1`, `fil2`, `fil3`, suffixes `_freq`
  (20 à 20 000 Hz), `_res` (0 à 1), `_env` (−1 à 1),
  `_saturation` (0 à 1), `_gain` (−60 à +12 dB) ;
  pour chacune des enveloppes `env1` à `env4`, suffixes `_attack`,
  `_decay`, `_release` (0,001 à 10 s), `_sustain` (0 à 1) ;
  `master` (−50 à +12 dB), `dist_boost`, `dist_drywet`, `delay_wet`,
  `phaser_drywet`, `flanger_drywet`, `chorus_drywet`, `rev_drywet`
  (tous 0 à 1). Le catalogue n'expose pas de seuil de distorsion.
  Au plus 16 règles par zone, un paramètre une seule fois dans cette zone. Les états
  marche/arrêt, types, routages et octaves restent ceux du preset d'usine.
  `OdinMorphParam::{ALL, COUNT, index, name, bounds}` décrit cette table ;
  `OdinVoiceParams.values` est son tableau fixe, indépendant du code GPL.
  Aucun boost master de compensation de l'ancien moteur n'est appliqué.
- **Temps réel.** Le morphing est préparé sur le thread principal : le compositeur
  connaît les socles des presets fournis par l'hôte, calcule le preset morphé de
  chaque piste à chaque changement d'émotion et envoie au callback une charge
  `Copy` préparée hors callback. L'application audio ne change ni type ni
  routage du preset et n'alloue pas. Dans une zone, les notes `Tied`
  (nappes, pédales) n'émettent pas de nouvelle attaque.
  La sélection initiale est immédiate. Ensuite, les bascules de zone sont
  installées à la prochaine frontière de partie (`bar_in_part == 1`,
  reprises incluses), en fondu linéaire de **300 ms** entre deux instances,
  même si elles partagent le même preset ; le morphing continue dans la
  zone active entre ces frontières.
  À la bascule, chaque note ancienne reçoit sa fin ; seules les notes
  encore tenues après les fins de note à la frontière sont réattaquées une
  fois sur la nouvelle instance. Une nouvelle attaque musicale du même
  pitch remplace ce transfert, sans doublon. Le fondu porte sur l'audio
  des deux instances, jamais sur une interpolation de presets.
  Les instances voyagent déjà chargées dans une transaction unique
  (`MusicComposer::send_odin`). Les anciennes
  instances reviennent par SPSC au thread principal ; une file de retour
  pleine suspend le dépilage des commandes, sans libération dans le callback
  ni perte d'installation.
- **Preset non chargé.** Un preset déclaré que l'hôte n'a pas fourni (ou illisible,
  ou build sans `odin`) n'a pas de repli implicite autre que le programme SoundFont
  déclaré ; le chargement du profil journalise un avertissement clair.
- **Niveau.** Un preset Odin est mixé au niveau de piste du profil (`[mix]`),
  multiplié par le gain `level_db` de sa zone. Le gain master du profil,
  calibré pour les samples SF2, ne s'y applique pas ; le `master` du preset
  reste un paramètre artistique fixe ou morphé, pas une compensation de zone.

## Percussion : routage par le profil

PercussionLow et PercussionHigh suivent `instruments.percussion_bank`, un seul chemin
(`apply_profile_programs`) : banque `128` = kit GM sur le canal MIDI 9 (marche,
`nt_desert`) ; toute autre banque = canaux propres 3 et 4 sur cette banque. La
banque de percussion `harmonium_trad.sf2` est chargée en banque `1` : les profils
à percussion en banque dédiée déclarent `percussion_bank = 1`. Un profil à kit déclare 128 :
charger la banque de percussion ne le détourne plus. Une banque désignée et non
chargée par l'hôte (ni kit GM en 128, ni banque de percussion en 1) n'a aucun repli :
la percussion est silencieuse et le chargement du profil journalise un avertissement
(`MusicComposer::missing_resources`, `harmonium_core::host_resources`).

## Accompagnement

Le vote harmonique est inchangé. Deux voix remplacent les anciens champs plats
et les motifs de `dynamics` ; aucun alias de compatibilité. Migration V5b-1 :
les sept profils, dont la marche sans section harmonique, gardent le même flux.

```toml
[accompaniment.bass]
low = 40
high = 50
hits = [
  { step = 0, degree = "root", duration = 4, velocity = 78 },
  { step = 8, degree = "fifth_if_held", duration = 4, velocity = 78 },
]
[accompaniment.chords]
center = 60
max_drift = 6
default_pattern = 1
patterns = [
  { hits = [{ step = 4, duration = 1, velocity = 72 }] },
  { hits = [{ step = 4, duration = 1, velocity = 72 }, { step = 12, duration = 1, velocity = 72 }] },
]
```

`degree` : `root`, `fifth`, `fifth_if_held` (quinte si l'accord n'a pas
changé depuis le créneau précédent, ou au premier créneau de la mesure).
Les coups sont ordonnés par `step`, uniques dans chaque voix/motif, dans la
mesure ; durée positive entièrement dans la mesure, vélocité 1..127.
Le registre basse est MIDI ordonné ; la marche exige douze hauteurs consécutives (les douze
classes de hauteur). `center` et `max_drift` pilotent la conduite de voix des triades.
`default_pattern` est utilisé lorsque les courbes `chords_intensity + density_chords`
valent zéro ; sinon l'index est `round(1 + offset)`, borné à la table.

### Graine d'accompagnement (S0, `TUNING_VERSION` 26)

Les couches de l'accompagnement qui tirent au sort (comping, marche de basse, notes de passage,
motifs et gabarits de basse, gratté) ne lisent **pas** la mélodie écrite : chaque air tire une graine
d'accompagnement dans le RNG de session (deux entiers de 32 bits, une fois par air), et chaque mesure
en dérive la sienne par `mix(graine, indice de mesure)`, sans état d'une mesure à l'autre (un seek
recalcule une mesure depuis son indice et les graines de l'air). Avant S0, la graine dérivait des
notes de la mélodie : sous une grille d'accords fixe, la banque de cellules minimale la réduisait à
deux valeurs, et seize graines de session donnaient deux comping et deux basses.

- Le tirage a lieu **après** la mélodie, l'intro et le matériel de l'air : la mélodie, l'intro et
  les ornements du premier air ne changent pas. Les airs suivants d'un même set, eux, reçoivent un
  RNG décalé de deux tirages par air : ils tirent d'autres cellules (un profil à plusieurs airs par
  set n'a donc plus les mêmes airs 2 et suivants qu'avant S0).
- Tout profil avec `[accompaniment]` tire, que ses couches tirent ou non (ajouter ou retirer une
  couche ne décale aucun autre tirage). Un profil sans `[accompaniment]` garde son flux.
- Les ornements et l'hétérophonie gardent la graine de matériel (dérivée de la mélodie écrite).

### Motifs de basse

`bass.motifs` remplace `bass.hits` : les deux sources sont exclusives, puis
passent par la même réalisation de notes et de doublures. Aucun changement
pour un profil sans banque de motifs. Modèles compatibles : tous ceux qui jouent des
accords (`melody_functional`, `loop`, `rewrite_functional`) ; les motifs suivent aussi les accords et pédales G2.

```toml
[accompaniment.bass]
low = 26
high = 37
motifs = [
  { bars = 2, weight = 3, reference = "chord_root", notes = [
    { step = 0, duration = 4, semitones = 0, octave = 0, velocity = 95 },
    { step = 4, duration = 2, semitones = 1, octave = 0, velocity = 85 },
    { step = 6, duration = 18, semitones = 0, octave = 0, velocity = 90 },
    { step = 24, duration = 24, degree = 1, octave = 0, velocity = 95 },
  ] },
]
```

Exemple sur une grille de **24 pas par mesure**. `bars` vaut 1..4 (défaut 1) ;
`step` est absolu depuis le début du motif, `duration` est en pas de la grille
du profil. Les notes principales sont ordonnées, sans chevauchement ; les
silences sont les trous entre leurs plages. Chaque note fournit exactement
un intervalle : `semitones` (−24..24, chromatisme conservé) ou `degree`
(−14..−1 ou 1..14 : 1 = référence, 2 = prochaine note du mode actif,
−1 = précédente). Les degrés suivent le mode **de la tonalité**, même si
la référence est une autre fondamentale. `octave` (−2..2, défaut 0) est
**relatif**, pas un numéro d'octave MIDI. La référence est d'abord placée par
`low..high`, puis les intervalles/octaves sont ajoutés, sans repli du contour.
Le chargeur refuse les dépassements MIDI ; pour les degrés, sa borne
conservatrice couvre jusqu'à trois demi-tons par pas modal.

`reference = "chord_root"` (défaut) suit la basse de l'accord au pas d'attaque
(la fondamentale, ou `bass` d'un accord slash du modèle `loop`) ;
`"tonic"` garde la tonique pour un bourdon. Une tenue franchissant une barre
est découpée en segments publiés avec `Articulation::Tied` sur les suivants :
le lecteur et l'export MIDI G5 prolongent la même hauteur sans nouvelle attaque.
Si la référence change, la nouvelle hauteur attaque normalement. `tie = true`
sur une note demande aussi une liaison avec la fin coïncidente précédente,
y compris à l'intérieur d'une mesure. Un timbre à sustain et aucune
interprétation raccourcissant le gate sont nécessaires pour une vraie pédale ;
la fixture NT utilise **Saw Lead, `bass_program = 81` (0-based)**.
`legato = true` (G7e) enchaîne cette note à la hauteur précédemment tenue :
chevauchement au lecteur SoundFont ; glissando mono sans réattaque sur Odin
(G2, réglage `glide_ms` décrit plus haut). La première note attaque normalement.
`doubling = ["octave", "tenth"]` suit les mêmes règles que les coups.

Banque : au plus 256 motifs, poids `u32` (défaut 1), somme positive sans
débordement ; poids nul = jamais tiré. `part = "A"` filtre par nom de partie ;
les motifs spécifiques éligibles ont priorité sur les motifs sans `part`.
Chaque partie doit disposer d'au moins un motif de poids positif, sauf celles
qui taisent la basse (`tracks_off`, voir `[form]`) : aucun motif ne peut les
nommer (refusé comme sans emploi), et le plan n'y tire aucune tranche.
Le tirage préfère les motifs entiers qui tiennent avant la fin de partie ;
sinon il tronque un motif à cette frontière, jamais au-delà.

La sélection est fixée une fois par air, dans un domaine ChaCha8 dérivé de la graine
d'accompagnement de l'air (voir « Graine d'accompagnement »). Les passes et tours
rejouent les mêmes tranches. `CopyOf` et `CopyOfPart` copient **une mesure**,
puis transposent ses intervalles sur l'harmonie de destination. Une copie
interrompt la tranche courante sans suspendre l'avancée du motif. La seconde fin
copie sa référence, tire un motif d'une mesure ou conserve la tranche finale.
`SequenceOf`, `VariedEndOf`, `resolve`, les variations et les préférences
rythmiques mélodiques ne transforment pas la basse : écrire ses variantes
dans la banque pondérée. Aucun deuxième moteur de variation mélodique.

Émotions : arrangement et seuils `density_tracks` gardent le contrôle de
la piste ; `bass_velocity` et `[performance.tracks.Bass]` s'appliquent aux
notes publiées. La densité ne retire pas d'attaques du riff écrit, et l'arousal
ne retire pas le motif en cours. `bass.passing`, si présent, garde ses courbes
densité/phrase/arousal : il approche les cibles du motif, y compris la première
de la mesure suivante, sans modifier les hauteurs/positions écrites ni les
soumettre au veto mélodique réservé aux ajouts. Il peut écourter une porteuse.
Omettre `passing` pour conserver un riff ou une pédale exactement écrits.

### Notes de passage de basse

`walk_low` / `walk_high` élargissent `low` / `high` pour les seules approches ;
omission : fenêtre des coups. `[accompaniment.bass.passing]` est optionnel :
absence, poids tous nuls ou courbe nulle ⇒ aucun tirage, flux inchangé.

```toml
[accompaniment.bass.passing]
none_weight = 4
density = [[0.0, 0.0], [0.5, 0.6], [1.0, 1.0]]
phrase = [[0.0, 0.7], [0.75, 1.0], [1.0, 1.3]]
arousal_low_multiplier = 0.5
arousal_high_multiplier = 1.5
avoid_melody_intervals = [1, 6, 11]
rules = [
  { when = "next_target_differs", before = [4], motion = "scale_step", weight = 3, duration = 2, velocity_delta = -12, min_leap = 0 },
  { when = "next_target_differs", before = [4, 2], motion = "scale_walk", weight = 2, duration = 1, velocity_delta = -12, min_leap = 5 },
  { when = "next_target_differs", before = [2], motion = "chromatic", weight = 1, duration = 2, velocity_delta = -12, min_leap = 0 },
]
```

Les poids applicables sont multipliés par `density(rhythm_density) × phrase(phase)`
et le multiplicateur interpolé à l'arousal brut (défaut 1/1) ; `none_weight`
reste fixe. Les courbes couvrent 0..1, sorties finies 0..4. `before` est une
liste strictement décroissante de distances en pas avant la prochaine cible
de basse, y compris au-delà de la barre. `scale_step` : voisin diatonique
de la cible côté départ ; `chromatic` : demi-ton ; `scale_walk` : voisins
successifs dans le mode actif. Aucun détour ni répétition de hauteur.
`chord_change_inside_bar` restreint la règle à un changement d'accord intérieur.
Les durées ne recouvrent ni la note suivante ni la cible ; le coup précédent
est écourté si nécessaire. Pas de passage sur un coup de basse ni un temps
fort. Le veto contre les intervalles modulo 12 lit la mélodie qui sonne ;
il est revérifié après les ornements. Les passages reçoivent la dynamique et
l'interprétation V2 de la piste Bass comme les autres notes.

Poids entiers : somme positive dans u32, au plus 64 règles ; `duration > 0`,
`velocity_delta` −127..127, `min_leap` 0..127. Règle impossible : abstention.
Les tirages dérivent de la graine d'accompagnement de l'air (« Graine d'accompagnement »), dans
un domaine basse distinct par mesure et attaque : ni mélodie, ni percussion déplacées.
Les valeurs distribuées viennent du §7 de l'étude, **à valider à l'oreille** :
marche sans harmonie ; valse seulement dans le dernier quart de phrase ;
hornpipe plus rare (`none_weight = 10`), reel québécois `5`, autres reels/gigues `4`.

### Marche de basse

`[accompaniment.bass.walking]` est une troisième source de la voix de basse, **exclusive** de
`hits`, `motifs` et `passing` (le chargeur refuse la combinaison : deux sources d'approches
se piétineraient). Une note par pulsation (`meter.steps_per_pulse` : quatre noires en 4/4),
référée à l'accord courant **au pas près** : un changement d'accord à une pulsation donnée
est lu à cette pulsation. Valable pour tous les modèles qui jouent des accords, dont
`loop` et `rewrite_functional` (même mécanisme que les coups et les motifs).

```toml
[harmony]
steps_per_chord = 4          # multiple de meter.steps_per_pulse : un changement tombe sur une pulsation
[accompaniment.bass]
low = 36                     # registre des arrivées ; high - low >= 11
high = 47
walk_low = 33                # fenêtre des notes de passage et d'approche (défaut : low / high)
walk_high = 55
[accompaniment.bass.walking]
velocity = 76
accent = 8                   # ajouté à la vélocité des arrivées (défaut 0)
duration = 4                 # en pas, de 1 à une pulsation
max_leap = 5                 # demi-tons au plus entre deux notes de passage (1..12)
toward_target_weight = 3     # défaut 3 : poids ajouté (à 1) aux notes qui se rapprochent de la cible
keep_direction_weight = 2    # défaut 2 : poids ajouté aux notes qui poursuivent le sens précédent
approach = { none_weight = 1, chromatic_weight = 4, diatonic_weight = 2 }
passing = { chord_tone_weight = 3, scale_step_weight = 3 }
```

| Pulsation | Note |
|---|---|
| **Arrivée** : le premier temps de la mesure et chaque pulsation où l'accord change | la basse de l'accord (`bass` d'un accord slash, sinon la fondamentale), placée dans `low..high` ; vélocité + `accent` |
| **Approche** : la pulsation qui précède une arrivée, si elle n'en est pas une | tirage `none` / `chromatic` / `diatonic` par poids. Chromatique : un demi-ton sous ou sur la cible. Diatonique : la première note de la gamme du mode (un, deux ou trois demi-tons). Côté de l'arrivée précédente de la ligne (au-dessus si elle part plus haut), sinon de l'autre côté si la fenêtre l'impose. `none` : note de passage |
| **Passage** : les pulsations entre les deux | tirage `chord_tone` (note de l'accord du pas, tensions comprises) ou `scale_step` (voisin de la gamme de la note précédente), parmi les notes à `max_leap` de la précédente qui laissent la cible atteignable ; jamais un retour sur l'avant-dernière note tant qu'un autre choix existe |

La cible de la dernière pulsation de la mesure est l'arrivée du premier temps de la mesure
suivante : après la dernière mesure du dernier tour il n'y a pas de cible, donc pas
d'approche. Une mesure se joue à l'identique d'une génération à l'autre : ses tirages
dépendent de la graine d'accompagnement de l'air et de la mesure, jamais de la note de la mesure
précédente. Poids `u32` ; une somme nulle (`approach`, `passing`) est refusée.

**Deux temps.** Une variante de `[arc]` peut demander `bass_feel = "two_feel"` : la même ligne,
avec une attaque toutes les deux pulsations (temps 1 et 3 en 4/4) et à chaque changement
d'accord. L'arrivée est celle de la marche ; l'attaque suivante dans l'étendue de l'accord est
l'approche de la cible suivante (tirage `approach`, sinon une note de passage qui la rend
atteignable), tenue jusqu'à l'attaque d'après. Les poids de `[accompaniment.bass.walking]` sont
ceux de la marche.

#### Basse plus riche (phase 019, A4)

Six options **absentes par défaut** (aucune n'ajoute de tirage à la ligne tant qu'elle n'est pas
déclarée : un profil sans elles garde ses notes), toutes dans `[accompaniment.bass.walking]`. Les
poids sont des `u32` ; une somme nulle est refusée. Fixture : `tests/fixtures/profiles/grille_basse_voicings/`.

```toml
approach = { none_weight = 70, chromatic_weight = 10, diatonic_weight = 6, side = { from_weight = 50, other_weight = 15, above_weight = 35 } }
passing  = { chord_tone_weight = 40, scale_step_weight = 60, chromatic_weight = 30, repeat_weight = 40 }
arrival  = { root_weight = 63, third_weight = 12, fifth_weight = 12, seventh_weight = 13 }
eighths  = { none_weight = 84, chromatic_weight = 3, diatonic_weight = 3, ghost_weight = 10, velocity = 58, ghost_velocity = 34 }
two_feel = { passing_weight = 1, fifth_weight = 2 }
```

| Option | Effet |
|---|---|
| `approach.side` | côté de l'approche, tiré à chaque approche : `from_weight` (côté de l'arrivée précédente, règle historique), `other_weight` (côté opposé), `above_weight` (au-dessus de la cible), `below_weight` (dessous). La fenêtre de marche qui n'offre pas le côté tiré laisse l'autre côté |
| `passing.chromatic_weight` | chromatisme de passage : un demi-ton de la note précédente, hors de la gamme du mode et de l'accord ; genre de note tiré à côté des notes d'accord et des pas de gamme |
| `passing.repeat_weight` | note répétée : la note précédente rejouée (au passage seulement, jamais à l'arrivée ni à l'approche) |
| `arrival` | l'arrivée (changement d'accord, premier temps) joue la tierce, la quinte ou la septième de l'accord au lieu de sa basse. **Jamais la grille contredite (D4)** : un accord slash, une pédale ou un accord sans le degré tiré gardent la basse publiée ; la note reste une note de l'accord publié, dans `low..high`. Tirée avec la graine de la mesure et le rang de la pulsation, **indépendamment** du reste de la ligne : la mesure précédente retrouve l'arrivée de la suivante (graine de la mesure suivante) pour y mener son approche |
| `eighths` | après la note d'une pulsation dont la suivante tombe à la pulsation suivante (feel de marche ; pas au deux temps), une **croche de passage** (approche chromatique ou diatonique de la note suivante, vélocité `velocity`) ou une **note fantôme** (même hauteur, `ghost_velocity`), à la demi-pulsation ; la note de la pulsation est alors coupée à la demi-pulsation. Pas d'ajout après une approche. Demande `meter.steps_per_pulse` pair. Ne change aucune note de pulsation (test de contrat) |
| `two_feel` | feel à deux temps : la deuxième attaque d'une étendue d'accord qui n'est pas une approche est la **quinte** de l'accord (la plus proche de la note précédente) avec `fifth_weight`, une note de passage avec `passing_weight` |

Calibrage (`make backing-validate` du labo, `grille_arc_chorus` avant, `grille_basse_voicings` après,
blues de douze mesures sans arc, 88 mesures par graine, 16 graines inconnues 11 à 26) : distance
agrégée à WJazzD swing **0,224 → 0,133** (blues 0,217 → 0,133) ; chromatisme de passage 28 % (WJazzD
27 %), notes répétées 20 % (20 %), arrivée sur la basse 64 % (63 % du groupe `reliable`, 41 % de `all`).
Les **croches de basse n'ont pas de source** (WJazzD ne note que la hauteur par temps, L2 § 6
point 8) : leurs poids sont de métier (7 à 8 % de notes hors pulsation, dont trois quarts de
fantômes). Détail : `specs/019-bibliotheque-accompagnements/a4-a5-basse-voicings.md`.

**Gabarits non swing.** `accompaniment.bass.templates` (« Basse par gabarits ») exprime déjà
fondamentale-octave (`cycle = [{ degree = "root" }, { degree = "root", octave = 1 }]` sur huit
croches), fondamentale-quinte en croches (`cycle = [{ degree = "root" }, { degree = "fifth" }]`) et
les motifs syncopés (`rhythm` à pas libres) ; rien n'est ajouté au moteur (test de contrat
`les_gabarits_expriment_octave_quinte_en_croches_et_motifs_syncopes`).

### Accords grattés

`chords.note_count` (3..6, défaut 3) complète la triade par octaves en montant.
Chaque coup porte `direction = "block" | "down" | "up"` (défaut `block`),
`probability` (0..1, défaut 1) et `tie = true | false` (défaut `false`). Zéro/1 :
aucun tirage de présence ; entre les deux : un tirage dérivé par attaque. `down`
va du grave à l'aigu ; `up` ne joue que les `up_note_count` les plus aiguës, dans
l'ordre inverse. Dans le snapshot, l'ordre du Vec et des identifiants est
toujours l'ordre d'attaque. Un coup `tie = true` publie `articulation = "Tied"` :
si une hauteur sonne encore au moment exact où l'attaque suivante commence
(même canal, même hauteur, fin coïncidente), le lecteur et l'export MIDI la
prolongent sans nouvelle attaque ; sinon, attaque normale. Une nappe s'écrit
avec un coup `block` d'une mesure entière et `tie = true`.

```toml
[accompaniment.chords.spread]
down_gap_ms = 6.0
up_gap_ms = 4.0
up_note_count = 3
gap_jitter = 0.3
anchor = 0.5
tempo_exponent = 0.0
ref_bpm = 230.0
max_spread_ratio = 0.4
velocity_slope = 4
velocity_jitter = 5
```

| Champ | Bornes / mécanique |
|---|---|
| `down_gap_ms`, `up_gap_ms` | 0..40 ms entre cordes ; 0 : plaqué. |
| `gap_jitter` | 0..0,8, uniforme symétrique par écart, sur le RNG dérivé de l'attaque. |
| `up_note_count` | 2..`note_count`. |
| `anchor` | 0..1 : première corde / centre de l'étalement / dernière corde sur l'attaque V2. |
| `tempo_exponent` | 0..1 : écart × `(ref_bpm / tempo)^exposant` ; 0 : ms fixes, 1 : geste relatif au tempo. |
| `ref_bpm` | 1..1000, fini. |
| `max_spread_ratio` | 0..0,4 de la distance au coup suivant du motif, retour à la mesure suivante compris. |
| `velocity_slope`, `velocity_jitter` | 0..20 unités MIDI ; bonus décroissant de la première à la dernière corde, puis jitter individuel, **avant V2**. |

Le grattage est composé **après V2** en millièmes de pas. Pour conserver l'ordre,
le geste entier est réduit si nécessaire pour tenir dans ±490 ou le plafond
de voisinage ; aucune corde n'est clampée indépendamment des autres.
`spread` absent ou coup `block` : aucun décalage ni jitter V5, mais `note_count`
reste la polyphonie écrite. Mécanique historique : trois sons, pas de `spread`,
probabilités 1, mêmes vélocités/motifs. V2 tire désormais articulation et jitter
**par attaque** sur chaque piste (contexte = hauteur la plus grave de l'attaque
précédente) ; gate/respiration communs, vélocités individuelles conservées.
La validation croisée jitter/écart de V5a est donc inutile : un jitter V2 commun
ne peut pas inverser les cordes.

Les ms sont converties avec le tempo nominal de la mesure ; swing et rampes
du lecteur réutilisent ensuite ce décalage dans la durée gelée de chaque pas,
comme V2 (pas d'horloge murale ni de second ordonnanceur audio).
Les six ensembles harmoniques gardent guitare nylon et basse acoustique :
écarts descendants 6/7/8/9/10 ms selon le profil, valeurs de départ de l'étude.
Reels : montées douces optionnelles à haute densité ; gigues : bas-haut-bas ;
valse et hornpipe : montée rare. La marche reste sans accompagnement tonal.

### Voicings et comping

`chords.voicing` choisit la façon de jouer l'accord ; tous respectent les notes de l'accord
**publié** (`chord_pitch_classes`, tensions comprises) et ne jouent jamais la basse d'un
accord slash (la piste de basse la joue). Une pédale (sans qualité typée) garde ses voicings
serrés.

| `voicing` | Notes |
|---|---|
| `close_triad` (défaut), `fixed_triad` | inchangés : triade ou voicing serré, inversions libres ; `note_count` double la triade |
| `guide_tones` | tierce et septième (sixte, quinte d'une triade) ; plus une tension de la qualité (9, 13, #11, b9, #9, b13, 11 par priorité) si elle en a une : deux ou trois sons |
| `rootless` | quatre sons sans la fondamentale, **type A** (tierce en bas : tierce, quinte, septième, tension) ou **type B** (septième en bas : septième, tension, tierce, quinte), la conduite de voix choisit celui qui bouge le moins. Sans tension dans la qualité, la pile se prolonge à l'octave supérieure (tierce, quinte, septième, tierce). Triades : type A seulement |
| `rootless_a`, `rootless_b` | un seul type (B retombe sur A pour une triade) |
| `drop2` | quatre sons en position serrée dont la deuxième voix depuis l'aigu descend d'une octave ; une triade double sa fondamentale |
| `drop3` | quatre sons en position serrée dont la troisième voix depuis l'aigu descend d'une octave (ambitus de 17 à 19 demi-tons sur un accord de septième) |
| `fourths` | quartes justes empilées (*So What*) : quatre sons, trois quartes d'écart, dans les sons de l'accord (type, tensions publiées) et des tensions naturelles que sa qualité permet ; la pile garde la tierce ou la septième. Fa 7 : tierce, treizième, neuvième, quinte ; sol m 7 : quinte, fondamentale, onzième, septième. Sans pile pour la qualité (triade, diminué), retombe sur `rootless` |
| `upper_structure` | structure supérieure : une triade majeure ou mineure bâtie sur un degré autre que la fondamentale (la basse porte la fondamentale), dans tous ses renversements ; ses trois sons sont ceux de l'accord et de la tension de couleur, et elle garde la tierce ou la septième (ex. sol m sur do 7 avec la neuvième, mi m sur do maj7). Sans structure pour la qualité (do 7 sans tension), retombe sur `rootless` |
| `shell` | *shell* à deux voix : la tierce et la septième (sixte, quinte d'une triade), dans les deux dispositions |
| `open` | *spread* : drop 2 et 4 de la position serrée, quatre sons sur plus d'une octave (ambitus de 20 à 22 demi-tons sur un accord de septième) |

**Tension de couleur tirée par graine** (`chords.color`, absente par défaut : la tension historique,
9, 13, #11, b9, #9, b13, 11 par priorité). Par accord joué, un tirage pondéré avec la graine de la
mesure et le créneau d'accord :

```toml
color = { none_weight = 2, published_weight = 2, ninth_weight = 3, eleventh_weight = 1, thirteenth_weight = 3 }
register_drift = { step = 2, max = 6 }
```

`none` : aucune tension ; `published` : la tension publiée par la qualité (priorité historique) ;
`ninth`, `eleventh`, `thirteenth` : une tension naturelle que la qualité permet **sans la publier**.
Elle n'est tirée que si le tableau de métier de `voicing.rs` l'autorise pour le type (9 et 13 sur
majeur, maj7, 7 et 7sus4 ; 9 et 11 sur mineur et m7 ; 9 sur 6, m6 et mMaj7 ; rien sur m7b5, dim7,
augmenté, sus2, altéré), si aucun son de l'accord (type et tensions publiées) ne la double ni
ne l'altère (b9 ou #9 pour la 9, #11 pour la 11, b13 pour la 13) et si elle ne tombe pas **un demi-ton
au-dessus d'un son de l'accord** (note à éviter : 11 sur une tierce majeure, 13 sur une quinte
augmentée). **Jamais en conflit avec la grille (D4)** ; un poids inéligible vaut 0 ; sans poids
éligible, la tension historique (pédale comprise). Valable pour `guide_tones`, `rootless*`, `drop2`,
`drop3`, `open` (la tension remplace les tensions publiées dans la pile) et `upper_structure`
(les sons permis de la triade) ; `fourths` préfère les piles qui portent la tension tirée ;
`shell` n'a pas de tension.

**Dérive de registre** (`chords.register_drift`, absente par défaut) : à chaque mesure, le centre
des voicings se déplace de `step` demi-tons au plus (haut, bas ou rien, tiré par graine) sans jamais
s'éloigner de plus de `max` demi-tons de `center` (`step` 1..12, `max` `step`..24). Il repart de
zéro au début de chaque passe de partie, d'où le rejeu à l'identique après un seek (les voicings
sont rejoués depuis ce point). Le choix suit la même conduite de voix et la même fenêtre `low..high`.

Les voicings jazz se placent dans la fenêtre `low..high` (au moins douze demi-tons) :
de tous les renversements et de toutes les octaves, le voicing retenu est, dans l'ordre,
celui qui sort le moins de la fenêtre, dont la hauteur moyenne dérive le moins de `center`
(au plus `max_drift`), qui **bouge le moins** depuis le voicing précédent
(`theory::voice_leading::voicing_movement`, voix appariées par rang depuis le grave), puis
le plus proche de `center` et le plus grave. Au premier accord il n'y a pas de précédent :
le voicing le plus proche de `center`. Le choix est déterministe.

```toml
[accompaniment.chords]
center = 62
max_drift = 8
default_pattern = 0
voicing = "rootless"
low = 52
high = 76
patterns = [{ hits = [{ step = 0, duration = 8, velocity = 62 }, { step = 8, duration = 8, velocity = 62 }] }]
comping = [
  { weight = 3, hits = [{ step = 0, duration = 6, velocity = 66 }, { step = 6, duration = 4, velocity = 58 }] },
  { weight = 2, hits = [{ step = 4, duration = 3, velocity = 60 }, { step = 10, duration = 3, velocity = 56 }, { step = 14, duration = 2, velocity = 60, anticipate = true }] },
  { weight = 1, hits = [] },
]
```

`comping` est une banque de rythmes d'**une mesure**, tirés par poids (`u32`, défaut 1, somme
positive, au plus 256 motifs) avec la graine d'accompagnement et le numéro de mesure : une mesure se
rejoue à l'identique. Elle remplace le choix du motif d'intensité (`patterns`, toujours requis,
joué aussi par l'intro). Un motif sans attaque est un silence. Les attaques ont les mêmes
champs que celles de `patterns` (`direction`, `probability`, `tie`) plus `anticipate = true` :
l'attaque joue l'accord en vigueur à la **fin** de sa durée (`step + duration`), donc l'accord
qui commence à cette fin ; sans changement, l'accord du pas. Une attaque qui finit sur la barre
joue le premier accord de la mesure suivante et, après la dernière mesure du dernier tour,
l'accord du pas. Un profil sans `comping` ni `anticipate` garde son flux.

Un motif de `comping` accepte un `tag = "..."` (étiquette libre, sans effet seule) que
`[arc]` nomme pour restreindre le tirage d'un chorus à certains rythmes (clairsemés, denses).
Voir « Arc sur plusieurs chorus ». Les suites de motifs sur plusieurs mesures
(`accompaniment.chords.phrases`) sont décrites dans « Phrases d'accompagnement sur plusieurs
mesures ».

La fixture technique `../tests/fixtures/profiles/grille_accompagnee/` active la marche et le
comping sans fondamentale sur un blues de douze mesures en fa (ii-V dans la mesure 9) puis un
ii-V-I de quatre mesures aux changements à la noire. Elle n'est pas distribuée : aucune
validation à l'écoute ; le profil distribué qui les emploie est `wjazzd_swing` (E7).

### Recettes piano et accordéon

Ces snippets **remplacent les deux voix** et les deux programmes correspondants
dans un profil complet ; ils ne s'ajoutent pas aux tables existantes.
Les tests chargent ces snippets exacts avec le chargeur public. Les sept profils
distribués gardent leurs instruments : aucune mesure de corpus ni écoute ne
justifie encore de remplacer leur guitare/basse par un autre ensemble.

**Piano main gauche, basse-accord (« stride » de veillée)** :

```toml
# recette: piano_main_gauche
[instruments]
chords_program = 0
bass_program = 0

[accompaniment.bass]
low = 36
high = 47
walk_low = 34
walk_high = 50
hits = [
  { step = 0, degree = "root", duration = 3, velocity = 82, doubling = ["octave"] },
  { step = 8, degree = "fifth_if_held", duration = 3, velocity = 82, doubling = ["octave"] },
]
[accompaniment.bass.passing]
none_weight = 3
density = [[0.0, 0.0], [0.5, 0.8], [1.0, 1.2]]
phrase = [[0.0, 0.7], [0.75, 1.0], [1.0, 1.3]]
avoid_melody_intervals = [1, 6, 11]
rules = [
  { when = "next_target_differs", before = [4], motion = "scale_step", weight = 3, duration = 2, velocity_delta = -12, min_leap = 0 },
  { when = "next_target_differs", before = [4, 2], motion = "scale_walk", weight = 4, duration = 1, velocity_delta = -12, min_leap = 5 },
  { when = "next_target_differs", before = [2], motion = "chromatic", weight = 1, duration = 2, velocity_delta = -12, min_leap = 0 },
]
[accompaniment.chords]
voicing = "close_triad"
center = 58
max_drift = 5
note_count = 3
default_pattern = 0
patterns = [{ hits = [
  { step = 4, direction = "block", duration = 2, velocity = 62 },
  { step = 12, direction = "block", duration = 2, velocity = 58 },
] }]
```

`doubling` est une liste de doublures uniques `octave` / `tenth`, vide par
défaut. `tenth` remplace l'octave par la dixième diatonique au-dessus de la
note de basse (deux voisins dans le mode, puis une octave). Les doublures
partagent attaque, durée, vélocité et interprétation ; la marche reste une
ligne simple entre les cibles **graves**, pas entre leurs doublures aiguës.
Le chargeur rejette une tessiture qui ferait dépasser MIDI 127.
Pour la variante basse-dixième, remplacer `["octave"]` par `["tenth"]`.
Pour une gigue de Cape Breton : coups de basse 0/6, accords 2/4/8/10 ;
adapter durées et grille au profil, pas au nom d'une recette.

**Accordéon basse-accord, mélodéon à deux basses** :

```toml
# recette: accordeon_basse_accord
[instruments]
chords_program = 21
bass_program = 21

[accompaniment.bass]
low = 36
high = 47
hits = [
  { step = 0, degree = "root", duration = 3, velocity = 70 },
  { step = 8, degree = "fifth_if_held", duration = 3, velocity = 70 },
]
[accompaniment.chords]
voicing = "fixed_triad"
center = 58
max_drift = 0
low = 52
high = 64
note_count = 3
default_pattern = 0
patterns = [{ hits = [
  { step = 4, direction = "block", duration = 2, velocity = 56 },
  { step = 12, direction = "block", duration = 2, velocity = 56 },
] }]
```

`fixed_triad` choisit toujours le même renversement le plus grave dans
`low..high`, indépendamment de l'accord précédent ; fenêtre MIDI ordonnée
couvrant au moins douze hauteurs. `close_triad` reste le défaut, avec conduite
de voix autour de `center` / `max_drift`. Le mélodéon n'a ni grattage ni marche ;
pour un accordéon à davantage de basses, une règle de passage faible suffit.
Sources, prudence sur les extrapolations et repères historiques :
[étude V5a, §2 et §5.3](../specs/012-vie-realisme/accompagnement-etude.md).
Ces programmes GM ont une énergie différente : mesurer à nouveau les pistes
seules avec `measure_mix.py` avant de distribuer un profil qui adopte la recette.





## Hétérophonie

`Countermelody` est une seconde lecture de `Melody`, canal logique/MIDI **5**,
pas une nouvelle mélodie tirée. `TrackId::ALL` est la liste canonique des six pistes
pour snapshots, audio, export, solo et commandes de transport ; les canaux 0..4 ne bougent pas.
Le REPL agit sur le canal 5 avec `set gain 5 <gain>`, `set mute 5` / `set unmute 5`.
`--solo-track Countermelody` isole cette voix sans changer les notes générées.

**Valeur neutre : section `[heterophony]` absente**, donc aucune note secondaire,
aucun tirage supplémentaire et flux des autres pistes identique à main V20.
Une section explicite demande programme et entrée d'arrangement ; les champs
sont obligatoires sauf les filtres de chaque règle. Valeur vivante prudente :
entrée au deuxième tour, unisson dominant, rares notes simplifiées/ornées et
niveau mesuré au moins 4 dB RMS sous le violon.

```toml
[instruments]
melody_program = 40
countermelody_program = 40
percussion_bank = 128
percussion_program = 0

[heterophony]
source = "Melody"
octave = 0
low = 55
high = 88
arousal_low = 0.5
arousal_high = 1.5
density_low = 0.7
density_high = 1.3
operations = [
  { operation = "unison", weight = 80 },
  { operation = "plain", weight = 8 },
  { operation = "ornament", weight = 8, phrase_end = "never" },
  { operation = "simplify", weight = 5, beats = ["weak"], phrase_end = "never" },
  { operation = "octave", weight = 2 },
  { operation = "third", weight = 3, phrase_end = "only" },
  { operation = "sixth", weight = 2, phrase_end = "only" },
  { operation = "rest", weight = 2, beats = ["weak"], bars = [2, 6] },
]

[mix.tracks.Countermelody]
level_db = -12.0
pan = -0.22
reverb_send = 0.16

[performance.tracks.Countermelody]
jitter = 0.025
phrase = [[0.0, 0.98], [0.7, 1.02], [1.0, 0.94]]
breath = 0.08
```

Ajouter `Countermelody = { tour = 2, part = 0, pass = 1 }` à `[entries]` du
fichier d'arrangement, ou `[arrangement.entries]` en inline. Les entrées sont
réappliquées au premier air de chaque set, comme pour les autres pistes.
Pas de doublure pendant une intro ; une clôture tenue peut être doublée.

| Champ | Bornes et effet |
|---|---|
| `source` | `Melody` seulement ; pas de suivi cyclique ou harmonisation de basse. |
| `octave` | −2..2, décalage de base en octaves ; repli par octaves dans la tessiture. |
| `low`, `high` | MIDI 0..127, fenêtre ordonnée couvrant au moins une octave ; les ornements V3b respectent la même fenêtre. |
| `arousal_low/high`, `density_low/high` | Finis 0..4, non décroissants ; facteur linéaire de chaque axe brut, produit borné à 16 sur les poids autres qu'unisson. L'unisson garde son poids fixe. |
| `operations` | 1..64 règles, somme des poids positive dans u32 ; unisson inconditionnel de poids positif obligatoire. |
| `operation` | `unison`, `plain`, `ornament`, `simplify`, `octave`, `third`, `sixth`, `rest`. |
| `weight` | Poids relatif parmi les règles dont tous les filtres sont vrais, pas un pourcentage de modifications garanties. |
| `beats` | `strong` / `weak` selon la pulse du profil ; vide = tous. |
| `phrase_end` | `any`, `never`, `only` : dernière mesure de la phrase déclarée dans le profil. |
| `bars` | Positions 1-based 1..64 dans la partie ; vide = toutes. |

`unison` conserve la porteuse et réutilise les règles V3b avec le domaine de graine
commun ; `plain` explicite l'abstention d'ornement. `ornament` utilise ces **mêmes
règles V3b**, mais une graine de matériel dérivée distincte pour sa porteuse.
Les changements du matériel voisin et d'éligibilité peuvent modifier le tirage
d'unisson : seule une mesure identique conserve exactement son choix d'agrément.
Un tirage peut s'abstenir (durée, contexte, taux) ; pas de second moteur d'ornements.
`simplify` tient la note jusqu'à la prochaine pulse/changement d'accord en omettant
le trait intermédiaire, hors cadence. `octave` propose +12 demi-tons.
`third` / `sixth` montent de deux/cinq degrés du mode actif, en cadence seulement
et sur un son de l'accord présent ; la **dernière porteuse de phrase reste commune**.
`rest` omet une attaque faible seulement. Une opération impossible laisse l'unisson.

Variations avant dynamique et interprétation V2 ; ornements après V2 ; veto final
contre toute seconde mineure/majeure (y compris composée) avec la mélodie
autour des temps forts, tenues et notes ajoutées comprises : la voix secondaire
s'abstient, jamais de modification de l'harmonie ou de la mélodie principale.
Le timing vient uniquement de `[performance.tracks.Countermelody]`, appliqué sur
le chemin V2 existant. Tous les choix sont faits avant publication sur le principal ;
aucune allocation ni RNG ajouté au callback.

Les poids distribués sont des propositions d'arrangement à écouter, **pas des taux
d'hétérophonie mesurés dans le corpus monodique ABC**. Reels québécois/acadien et gigue
québécoise : second violon ; valse : accordéon de veillée moderne (anachronisme explicite
pour 1837) ; marche : second fifre ; gigue/hornpipe britanniques : pas de seconde voix,
faute de source d'ensemble suffisante. Provenance et réserves dans chaque README.

## Mixage

Le mixage ne change aucune note ni aucun tirage. Les sept profils déclarent
tous leurs paramètres ; **section absente** : salle sèche (`level = 0`),
pistes à 0 dB / centre / sans envoi, bus 0 dB, coupe-bas et présence neutres.
Les paramètres d'une section explicite sont requis, pas remplis en secret.

```toml
[mix]
master_db = 16.0
[mix.room]
room_size = 0.35
damping = 0.6
width = 0.7
level = 0.28
[mix.tracks]
Melody = { level_db = 0.0, pan = 0.04, reverb_send = 0.16 }
Chords = { level_db = -5.0, pan = -0.35, reverb_send = 0.22 }
Bass = { level_db = -8.0, pan = 0.0, reverb_send = 0.08 }
PercussionLow = { level_db = -6.0, pan = -0.06, reverb_send = 0.10 }
PercussionHigh = { level_db = -6.0, pan = 0.06, reverb_send = 0.10 }
[mix.eq]
highpass_hz = 40.0
presence_hz = 2800.0
presence_db = 1.5
presence_q = 0.8
```

| Paramètre | Bornes (fini) | À l'oreille : sec / ample |
|---|---|---|
| `master_db` | −60..48 dB ; gain amplitude `10^(dB/20)` | 0 : unité ; augmenter rapproche la crête de 0 dBFS, sans limiteur ni saturation volontaire. Les valeurs distribuées sont calibrées sur les banques de référence, pas une garantie toutes banques/graines. |
| `room.room_size` | 0..1 (`ReverbParams.roomsize`) | 0 : décroissance courte ; 0,6..1 : longue salle. |
| `room.damping` | 0..1 (`damp`) | 0 : réverbération brillante ; 1 : absorption maximale des aigus. |
| `room.width` | 0..1 | 0 : retour mono ; 1 : retour stéréo large. |
| `room.level` | 0..1 | 0 : sec ; 0,3..0,9 : retour de salle croissant. |
| `tracks.<piste>.level_db` | ≤0 dB, relativement à CC 7 = 127 | 0 : niveau maximal du canal ; négatif : accompagnement en retrait, jusqu'à la quantification à CC 7 = 0. |
| `tracks.<piste>.pan` | −1..1 → CC 10 | −1 gauche, 0 centre (64), +1 droite ; au centre, l'EQ ne déplace pas la piste. |
| `tracks.<piste>.reverb_send` | 0..1 → `round(127 × envoi)`, CC 91 | 0 : aucun envoi ajouté ; 1 : envoi maximal. Les envois intrinsèques SF2 peuvent s'y ajouter. |
| `eq.highpass_hz` | 0 (bypass), ou 20..1000 Hz | Coupe-bas Butterworth 12 dB/octave, Q fixe `1/√2` ; enlève le sous-grave, ne rend pas le signal plus large. |
| `eq.presence_hz` | 20..20000 Hz | Centre d'une bande en cloche (RBJ), p. ex. 2,8 kHz pour la lisibilité de l'archet. |
| `eq.presence_db` | −12..12 dB | 0 : neutre ; positif : présence, négatif : adoucir. |
| `eq.presence_q` | 0,1..10 | Petit Q : large bande ; grand Q : correction étroite. |

Conversion SF2 du niveau : l'atténuation du modulateur CC 7 concave est
`−40 log10(CC7 / 127)` dB ; donc **`CC7 = round(127 × 10^(level_db/40))`**,
pas une conversion linéaire `127 × 10^(dB/20)`. Quantification à 7 bits
(0 = silence, atteint aux très bas niveaux) et aux centibels du synthé.
Le pan est arrondi avec 64 pas à gauche et 63 à droite : centre exactement
64, extrêmes 0/127. Les coefficients des deux biquads et le gain master sont
préparés sur le thread principal au taux de l'hôte ; fréquences limitées à
0,45 × ce taux si Nyquist est inférieur au réglage demandé.

Un seul état par piste dans la lecture : le profil initialise niveau/pan/envoi,
`set gain` modifie le niveau (amplitude 0..1), `mute` filtre les attaques.
Un nouvel air restaure les niveaux du profil sans lever les mutes, donc
`--solo-track Melody` reste une sélection de rendu et ne change pas la génération.
En GM, PercussionLow et PercussionHigh partagent le canal 9 : **les réglages de PercussionHigh gagnent**,
également dans le MIDI. La banque dédiée les sépare sur 3/4. Les profils ont
les mêmes niveaux/envois aux deux voix de percussion, mais leur léger écart de pan est
réduit à celui de PercussionHigh dans le repli GM.

Choix d'espace proposés (pas des mesures historiques) : cuisine amortie pour
le reel québécois, petite salle pour sa gigue, salle communautaire pour le reel
acadien, salle paroissiale plus large pour la valse ; salles de danse courtes
pour gigue/hornpipe britanniques, plein air quasi sec pour fifre-et-tambour.
La guitare est écartée, la basse centrale, la percussion resserrée ; la mélodie
reste près du centre et au-dessus de chaque accompagnement mesuré.
Les README individuels donnent les intentions ; les mesures avant/après et
leurs limites sont dans [Mesures V1](../specs/012-vie-realisme/plan.md#mesures-v1).

## Interprétation

`[performance.tracks.<piste>]` fonctionne pour `Melody`, `Chords`, `Bass`,
`PercussionLow` et `PercussionHigh`. Les profils à airs distribués n'ajoutent d'interprétation qu'à
la mélodie ; `nt_desert` y ajoute le micro-timing et l'accent de la percussion
(jitter et règle `Accent` sur `PercussionLow`/`PercussionHigh`, G7c) ; les durées écrites de
basse/guitare et les samples de percussion restent ceux des banques. **Section absente
ou tous les réglages à zéro : aucun tirage supplémentaire**, mêmes hauteurs,
vélocités et durées qu'avant V2.
Les champs omis sont neutres ; `velocity_min = velocity_max = 0` désactive
le bornage supplémentaire.

L'unité de timing est la **fraction de pas**, pas la milliseconde : elle suit
le tempo, le swing et les rampes sans réécrire la grille. À 230 BPM et quatre
pas/noire, `0.02` vaut environ 1,3 ms avant swing. Les décisions sont publiées
dans chaque note : `performance.offset` en millièmes de pas signés,
`performance.gate` en millièmes de durée (1000 = plein),
`performance.breath` en millièmes de pas soustraits **après** le gate.
La résolution de décision est donc 0,001 pas ; la lecture arrondit ensuite
chaque événement à la frame la plus proche dans le pas effectivement gelé.
`start_step`, `BeatInfo` et le transport restent sur la grille.

```toml
[performance.tracks.Melody]
offsets = [-0.02, 0.0, 0.01, 0.0, -0.02, 0.0, 0.01, 0.0,
           -0.02, 0.0, 0.01, 0.0, -0.02, 0.0, 0.01, 0.0]
jitter = 0.02
phrase = [[0.0, -2.0], [0.5, 2.0], [1.0, -3.0]]
velocity_min = 30
velocity_max = 120
breath = 0.08
[[performance.tracks.Melody.rules]]
articulation = "Normal"
weight = 90.0
[[performance.tracks.Melody.rules]]
articulation = "Detached"
weight = 10.0
steps = [0, 4, 8, 12]
min_duration = 1
max_duration = 4
context = "repeated"
shorten = 0.2
[performance.tracks.Melody.emotions]
tension_advance = 0.04
tension_detach = 0.10
arousal_accent = 4.0
valence_velocity = 0.0
density_detach = 0.0
```

Exemple de syntaxe, **pas une estimation de taux traditionnel**. Les README
individuels citent les comptages utilisés et les choix non mesurables.

| Paramètre | Bornes / effet audible | Mécanique | Vivant, point de départ |
|---|---|---|---|
| `offsets` | Vide ou exactement un nombre fini par pas, chacun −0,2..0,2 ; négatif = en avant, positif = en arrière. | `[]` ou zéros | ±0,01..0,025 pas |
| `jitter` | 0..0,1 pas ; uniforme symétrique bornée. Tirée par un `ChaCha8Rng` dérivé d'une graine tirée **une fois par air** sur le ChaCha8 du générateur (uniquement si l'interprétation n'est pas neutre), un flux par mesure : la mesure `n` se joue pareil après un seek, sans horloge. | `0` | 0,01..0,025 |
| `rules[].articulation` | `Normal`, `Detached`, `Linked`, `Accent` ; l'étiquette est publiée, jamais interprétée secrètement dans l'audio. | Aucune règle positive | Détaché : gate court ; lié : gate plein, aucun chevauchement ajouté ; accent : vélocité |
| `rules[].weight` | Poids fini 0..1 000 000 ; tirage parmi **toutes** les règles applicables, addition des poids, pas de priorité implicite. Aucun poids positif : note inchangée. | `0` | Poids de marques écrites conditionnés par position/durée/contexte |
| `rules[].steps` | Positions 0-based dans la mesure ; vide = toutes. | `[]` | Positions de temps/demi-temps du corpus, ramenées à la grille du profil |
| `rules[].min_duration`, `max_duration` | Durée écrite en pas, bornes inclusives ; minimum omis = 0, maximum omis = sans plafond. | `0`, omis | Réserver une règle liée aux tenues, détachée aux notes courtes |
| `rules[].context` | `any`, `repeated`, `ascending`, `descending` ; intervalle entre les hauteurs les plus graves de deux attaques de la même piste **dans la mesure**, première attaque = `any`. Un accord partage ce contexte, sans intervalle fictif entre ses cordes. | `any` | Poids conditionnels sur répétitions ou contour |
| `rules[].shorten` | Fraction 0..0,8 retranchée à la durée ; pas de gate caché dans l'articulation. | `0` | Détaché : 0,15..0,25 ; lié : 0 |
| `rules[].accent` | Ajout de vélocité 0..20 unités MIDI, après la dynamique existante. | `0` | 2..4 pour `Accent` |
| `phrase` | Points `[phase 0..1, delta −20..20]`, phases strictement croissantes ; interpolation linéaire, extrémités constantes. Phase calculée à l'attaque sur la phrase configurée, pas sur tout l'air. | `[]` ou deltas nuls | −2 → +2 → −3 |
| `velocity_min`, `velocity_max` | Bornes MIDI ordonnées 0..127, appliquées après phrase/accent ; 0/0 = pas de bornage ajouté, vélocité toujours 1..127. | `0`, `0` | 30, 120 |
| `breath` | Fraction 0..0,5 de la durée de la dernière attaque avant chaque frontière `phrase_ends` ; en leur absence, frontières cadentielles du plan. Sans étirer la mesure ni déplacer les suivantes. | `0` | 0,08..0,12 |
| `emotions.tension_advance` | 0..0,2 pas soustraits proportionnellement à la tension ; borne finale de l'attaque ±0,49 pas. | `0` | 0,04 |
| `emotions.tension_detach` | 0..0,4 ajoutés au raccourcissement ; gate final ≥0,1 avant respiration. | `0` | 0,10 |
| `emotions.arousal_accent` | 0..20 unités MIDI ajoutées aux seules notes `Accent` à arousal 1 ; axe brut, distinct de l'intensité combinée. | `0` | 4 |
| `emotions.valence_velocity` | −10..10 unités multipliées par la valence signée. | `0` | 0 si aucune justification d'écoute |
| `emotions.density_detach` | 0..0,3 ajoutés au raccourcissement à densité 1. | `0` | 0 si aucune justification d'écoute |

La durée jouée vaut `max(0.001 pas, durée × gate / 1000 − breath / 1000)`.
Les triggers de durée écrite zéro ont une durée MIDI de référence d'un pas.
Les attaques négatives à la barre sont jouées dans le pas précédent depuis la
mesure préchargée ; au début de lecture/seek ou après sous-alimentation, elles
sont bornées au premier échantillon disponible, pas perdues. Une répétition de
la même hauteur ferme d'abord la note active ; son ancienne fin ne coupe pas
la nouvelle note. Les notes ne sont jamais déplacées dans la partition.

Les ABC donnent des **marques écrites**, pas des gates mesurés ni des timings
de violoneux : absence de marque ≠ absence de jeu expressif. Micro-timing,
gates, courbe de phrase et émotions restent **à valider à l'oreille**. Repères
généraux, non preuves historiques des sept traditions :
[Friberg & Sundberg, 1995, discrimination temporelle](https://pubs.aip.org/asa/jasa/article/98/5/2524/798047/Time-discrimination-in-a-monotonic-isochronous),
[Friberg, Bresin & Sundberg, 2006, règles KTH de phrasé/articulation](https://doi.org/10.2478/v10053-008-0052-x).
Mesures reproductibles : `specs/012-vie-realisme/measure_performance.py`,
MIDI + vérité pour 20 seeds/profil, tension et arousal à 0/1.

## Ornements

`[ornaments]` décrit les agréments idiomatiques ; un seul chemin va du taux du profil
aux notes publiées, la note de passage de densité émotionnelle en fait partie
(`[ornaments.types.passing]`, qui remplace `dynamics.density_passing` et
`form.ornament_min_steps`). **Section absente, types absents ou `enabled = false`,
`rate_scale = 0` : aucun tirage, flux identique à celui d'un profil sans ornements.**
Les ornements sont réalisés **après** l'interprétation ci-dessus : ils composent avec son
décalage, son gate et sa respiration, sans les remplacer.

```toml
[ornaments]
rate_scale = 4.0          # facteur global sur tous les taux écrits
repeat_variation = 0.0    # 0 : une mesure copiée/rejouée porte les mêmes ornements

[ornaments.types.roll]
written_per_100_notes = 0.689826
per_100_notes_by_bar = [1.023166, 0.583255, 1.167532, 0.410754, 1.060606, 0.797922, 0.689299, 0.2083]
min_carrier_eighths = 2.0
arousal_low_multiplier = 0.5   # à arousal 0 ; 1,0 : neutre
arousal_high_multiplier = 1.5  # à arousal 1

[ornaments.types.double_stop]
written_per_100_notes = 0.344225
min_carrier_eighths = 2.0
strings = [55, 62, 69, 76]     # cordes à vide (MIDI) du violon

[ornaments.types.passing]      # densité : aucun tirage
min_carrier_eighths = 1.5
density_curve = [[0.0, 0.0], [0.5, 0.0], [1.0, 1.0]]
```

Exemple de syntaxe : les valeurs distribuées sont celles du labo (DS-T10), voir les README
de profil. Types : `grace_single`, `grace_multiple`, `roll`, `turn`, `triplet_repeated`,
`double_stop`, `chord`, `trill`, `passing`.

| Paramètre | Bornes / effet audible | Mécanique | Vivant, point de départ |
|---|---|---|---|
| `rate_scale` | 0..10, multiplie tous les taux. Les transcriptions sous-notent : taux écrit = plancher | `1` | `4` aux sept profils, à valider à l'oreille |
| `repeat_variation` | 0..1 : probabilité qu'une mesure d'une passe ou d'un tour 2+ tire ses ornements indépendamment de sa source. À 0, les copies (`CopyOf`, `CopyOfPart`) et les reprises ont les mêmes ornements tant que la mélodie écrite est la même | `0` | `0` ; 0,3 pour un jeu qui ne se répète pas |
| `written_per_100_notes` | 0..100 pour 100 notes de la mélodie de la mesure | `0` | Corpus ABC (plancher écrit) |
| `per_100_notes_by_bar` | Jusqu'à 64 taux, mesure 1 de la partie en premier ; au-delà ou vide : `written_per_100_notes`. Une copie exacte emprunte la mesure de sa source | `[]` | Forme du corpus positionné (≥ 30 occurrences dans des parties de huit mesures), niveau ramené au taux global ; sinon taux global seul |
| `min_carrier_eighths` | 0..64 croches, durée **écrite** minimale de la porteuse (durée totale remplacée pour les figures) | `0` (un pas) | Minimum observé si peu d'occurrences ; sinon plus petite durée dont le taux par note atteint 10 % du meilleur |
| `arousal_low_multiplier`, `arousal_high_multiplier` | 0..4, taux × interpolation linéaire à l'arousal brut (0 → 1) | `1`, `1` | Neutres : le corpus n'annote pas l'arousal |
| `intervals` | `repeated`, `step` (≤ 2 demi-tons), `leap` : intervalle avec la note précédente de la mesure ; vide = tous ; la première note de la mesure est exclue dès que la liste n'est pas vide | `[]` | — |
| `beats` | `strong` (début de pulse du profil), `weak` | `[]` | — |
| `phrase_end` | `any`, `never`, `only` : dernière mesure de la phrase (`parts.phrase_ends`) | `any` | — |
| `velocity_scale` | 0,1..1, vélocité des notes ajoutées (appogiatures, voisines, seconde corde) | `0.8` | 0,7..0,9 |
| `span` | Appogiatures : durée totale avant la porteuse, 0,05..0,49 pas | `0.3` | 0,4 (simple), 0,49 (multiple) |
| `count` | `grace_multiple` : 2..4 notes | `2` | `2` |
| `strings` | `double_stop`, `chord` : 2..8 cordes à vide MIDI croissantes | requis | `[55,62,69,76]` (violon) ; type désactivé pour un fifre |
| `min_interval`, `max_interval` | `double_stop`, `chord` : demi-tons entre la porteuse et chaque note ajoutée | `3`, `12` | `chord` : `max_interval = 24` |
| `density_curve` | `passing` seulement : part de la mesure (0..1) qui reçoit la note de passage selon la densité | requis | `[[0,0],[0.5,0],[1,1]]` |

Réalisation, dans la durée jouée de la porteuse (millièmes de pas, ± 0,49 pas de la grille,
donc dans les bornes et le chemin du lecteur de l'interprétation) :

- **Appogiature(s)** : avant la porteuse, qui reste **sur la grille** ; elles prennent leur
  temps sur la note précédente de la mesure, écourtée d'autant (respiration composable), ou
  sur un silence ; si la note précédente n'a pas ≥ 0,25 pas à rendre, ou si elle est dans une
  autre mesure, l'ornement n'est pas posé (une mesure déjà publiée ne change pas). Hauteur :
  voisine diatonique du mode actif, dans la tessiture, différente de la note précédente.
- **Rouleau** : porteuse, voisine haute, porteuse, voisine basse, porteuse (36 / 16 / 16 / 16 /
  16 %). **Turn** : voisine haute, porteuse, voisine basse, porteuse. **Trille** : porteuse et
  voisine haute en alternance, nombre impair de notes (≥ 5, une note ≈ 0,2 pas).
- **Triolet de notes répétées** : trois notes égales de la porteuse, séparées par 10 % de silence.
- **Double corde / accord** : seconde (et troisième) note simultanée, même piste, même durée et
  même timing que la porteuse : corde à vide voisine de la corde de la porteuse, dans le mode,
  l'intervalle demandé et la tessiture ; la double corde essaie aussi une tierce diatonique
  sous la porteuse. Sans note jouable, la porteuse reste seule.
- **Note de passage** : inchangée (dernier pas de la tenue, voisine diatonique haute, sans tirage).
- Une note porte **au plus un** ornement ; la note de passage passe avant les types tirés ; une
  figure impossible (voisine hors mode ou tessiture) laisse la porteuse intacte. Les mesures
  d'intro et d'outro et les mesures protégées n'en reçoivent pas.

**Probabilité.** Pour chaque mesure, l'espérance du type vaut
`taux (par mesure) × rate_scale × multiplicateur d'arousal / 100 × notes de la mesure`, répartie
également sur les porteuses permises (durée écrite, contextes) ; une porteuse tire au plus une fois.
Le taux reste donc « pour 100 notes de la mélodie » quelle que soit la part de notes éligibles,
comme dans le corpus. Les tirages dérivent d'une graine propre à l'air, calculée **sans tirage**
depuis son matériel écrit : ni l'intro, ni les tirages suivants, ni la mélodie écrite ne changent
quand on active les ornements, et un même air porte les mêmes ornements quelle que soit la mesure
déjà générée (seek). Les tirages d'une note ne dépendent que de sa mesure écrite (hauteurs,
positions), pas de la durée jouée : à émotions constantes une reprise a les mêmes ornements ; si
les émotions changent entre deux passes, la même figure s'étire avec la durée jouée.

Mesures reproductibles : `specs/012-vie-realisme/measure_ornaments.py` (`corpus`, `rates`,
`wav`, `peaks`, `midi`).

## Forme longue (set)

Un **set** est une suite d'airs d'un même profil (même mètre, même forme), jouée sans pause de
session. `set.toml`, référencé par `[files] set = "set.toml"`, règle la longueur du set et le
choix de la tonalité d'un air au suivant. Avec `fixed_tonality`, tous les airs gardent la
tonique et le mode imposés. Sinon, le jeu garde la priorité : la couleur (tonalité, mode)
reste décidée par la valence ; le set ne fait que **pondérer** des candidats que la valence
autorise (`melody.key_modes` à `neutral_valence`, `[[color]]` évalué à la valence ailleurs).

- **Premier air d'un set** : intro complète (`[intro]`, formes pondérées et levée) et entrées
  progressives des pistes (`[arrangement.entries]`). Les **airs suivants** utilisent le raccord
  pondéré (`direct`, `call`, `held`, `silence`) puis toutes les pistes dès A1.
- **Section absente** : un set sans fin ; tous les facteurs à 1, chaque air se tire indépendamment
  du précédent, intro complète et entrées au seul premier air de la session.
- `set.toml` contient `length`, `[key]`, `[ramp]`, `[signature]`, `[[transitions]]`, `[end]`
  à la racine du fichier. Les anciens `form.next_same_weight`, `form.next_relative_weight`,
  `arrangement.intro_each_tune` et `intro.each_tune` sont refusés au chargement (champ inconnu) :
  `length = [{ tunes = 1, weight = 1 }]` redonne une intro complète à chaque air.
- **Tirages.** Au début de chaque set, **avant l'air 1**, le moteur tire la longueur du set puis un
  tirage par famille (tonalité, raccord, signature) pour chaque air 2..L, enfin la pause.
  Un set sans fin ne tire rien d'avance. La tonalité se résout **à la construction de l'air** :
  si le jeu change la valence entre deux airs, la tonalité la suit, mais le plan du set ne
  change pas. Le premier air d'un set se tire sans facteur, comme tout air d'un profil sans `[set]`.
  Le poids de chaque candidat de l'air suivant est son poids de base × le facteur de la classe
  (précédent → candidat), classes testées dans l'ordre `same`, `parallel`, `relative`, puis
  l'intervalle de tonique ; **aucune tonalité n'est publiée hors des candidats**.

```toml
# set.toml
length = [
  { tunes = 2, weight = 45 }, { tunes = 3, weight = 44 },
  { tunes = 4, weight = 7 },  { tunes = 5, weight = 4 },
]

[key]
same      = 1.28
parallel  = 1.00
relative  = 1.02
fifth_up  = 0.97
fourth_up = 1.01
step_up   = 0.91
step_down = 0.73
other     = 0.75
last_dark = 0.80
```

| Paramètre | Bornes | À l'oreille | Neutre | Vivante (reel québécois) |
|---|---|---|---|---|
| `length` | Liste `{ tunes, weight }` : `tunes` 1..=16, au plus 64 entrées, somme des poids positive dans u32. Tirée **au début du set** | Nombre d'airs avant la prochaine annonce complète. 1 : chaque air est annoncé en entier (marche) ; 2–3 : un enchaînement court ; 16 : presque une session sans coupure | Omettre : set sans fin | 2·3·4·5 airs aux poids 45·44·7·4 |
| `key.same` | 0..=1000, fini | Garder la tonalité et le mode de l'air précédent : > 1 favorise la continuité, 0 l'interdit | `1` | `1.28` |
| `key.parallel` | idem | Même tonique, autre mode (ré ionien → ré mixolydien) | `1` | `1.00` |
| `key.relative` | idem | Même collection diatonique, autre tonique (ré ionien → si éolien) | `1` | `1.02` |
| `key.fifth_up`, `fourth_up` | idem | Tonique +7 ou +5 demi-tons (ré → la ; ré → sol) | `1` | `0.97`, `1.01` |
| `key.step_up`, `step_down` | idem | Tonique +2 ou −2 demi-tons ; descendre d'un ton est rare dans les sets réels | `1` | `0.91`, `0.73` |
| `key.other` | idem | Toute autre transition | `1` | `0.75` |
| `key.last_dark` | idem | Dernier air d'un set fini : facteur des modes mineurs (dorien, éolien) ; < 1 finit plus souvent en majeur | `1` | `0.80` |

Au moins un facteur de classe doit être positif. Les facteurs multiplient le poids de base de
chaque candidat ; si tous les candidats tombent à 0, le moteur retire d'abord les facteurs de
classe, puis `last_dark`, avant le poids de base seul : le jeu ne se retrouve jamais sans tonalité.

Les facteurs distribués sont des **rapports observé / indépendance** mesurés sur The Session
(reel → reel, jig → jig, hornpipe, valse ; voir
[l'étude V7a](../specs/012-vie-realisme/forme-longue-etude.md#32-sets-de-membres-the-session)).
Presque tous valent entre 0,75 et 1,3 : l'essentiel de la logique de tonalités est la concentration
du répertoire (`key_modes`), pas l'enchaînement. Le reel acadien prend la racine carrée des rapports
mesurés sur 205 transitions de pistes enregistrées (échantillon dominé par le Cap-Breton).
Longueurs de set : The Session, sets de 2 airs ou plus. Rien n'est mesuré sur la pratique québécoise
des sets : les valeurs sont des bornes de répertoire, l'écoute fixera le réglage.

**Position publiée.** `MeasureSnapshot.position` (`TunePosition`) porte `set_index` (0-based, numéro
du set dans la session), `tune_in_set` (**1-based**, « air n° k du set » ; 1 pour le premier air) et
`tunes_in_set` (longueur tirée, constante dans le set ; **0** pour un set sans fin, où `tune_in_set`
compte les airs de la session). `tune_index` reste l'indice global. Les trois champs ont une valeur
par défaut à la lecture : un JSON de snapshot antérieur à V7b se relit. L'export CLI les affiche
(« set 2, air 1/3 ») et les écrit dans `measures[].measure.position` du JSON.

### Raccords et clôture

`[[transitions]]` dans `set.toml` (`[[set.transitions]]` en profil inline) contient
`kind`, `weight` et, seulement pour `call` / `silence`, `bars` (1..=4).
Les poids sont entiers, somme positive dans u32, au plus 64 entrées ; table absente :
`direct`. Les tirages de raccord sont fixés dans le bloc initial du set, indépendamment
des émotions et de la génération spéculative.

| Forme | Rendu |
|---|---|
| `direct` | Levée seule, une mesure si `pickup_eighths > 0`, sinon aucune. |
| `call` | `bars` mesures de percussion seule, puis la levée dans une mesure séparée. |
| `silence` | `bars` mesures vides, puis la levée dans une mesure séparée. |
| `held` | Dernière mesure de l'air précédent rendue par l'outro tonique existante ; puis levée. Aucun arrêt du lecteur. |

`[end] outro = true` applique cette outro à la dernière mesure du dernier air du set.
`gap_bars = [{ bars = 0, weight = 1 }]` est la distribution des pauses (0..=4 mesures),
tirée au début du set ; elle précède l'intro complète du set suivant. Toutes ces mesures
portent `Section::Intro` et la position de l'air suivant ; `held` reste la dernière mesure
de l'air précédent. La grille et la pulse ne s'interrompent pas.
Section `[end]` absente : aucune outro ni pause. Sans longueur de set : aucun raccord
autre que la levée, aucune clôture. `end_after_tour` reste un arrêt demandé par le jeu
à la fin de l'air courant, indépendant de cette clôture.

Les six profils de médley ont une outro ; les pauses restent à zéro faute d'écoute.
Reels et gigue québécoise : direct/call/held aux poids 70/20/10 ; valse : direct seul.
Gigue et hornpipe britanniques : direct/held 90/10, **pas de call**, leur table de
percussion étant volontairement vide. L'étude proposait 80/10/10 : cela produisait
un silence, pas un appel ; aucun instrument ni pattern n'a été ajouté pour le masquer.
La marche conserve son appel complet à chaque air, sans outro supplémentaire.
Ces propositions **ne sont pas un ADR d'écoute**.

### Rampe par air et priorité du jeu

`[ramp]` (`[set.ramp]` inline) est optionnel. Les courbes `tempo_bpm` et
`intensity_lift` couvrent 0..1, avec sorties finies non décroissantes, respectivement
0..300 et 0..1. `gate` couvre 0..1, sorties 0..1, **zéro aux deux extrêmes**.
`ceiling_bpm` est optionnel (défaut `meter.max_bpm`), entre `meter.min_bpm` et 300.
Un set d'un seul air ou sans fin ne reçoit aucune rampe.

La progression est `(tune_in_set − 1)/(tunes_in_set − 1)` : même palier pendant
tout l'air, intro incluse, tant que les entrées du jeu restent constantes.
L'intensité effective est `clamp(i + intensity_lift(p) × gate(i), 0, 1)` ;
elle pilote les mêmes lectures de dynamique que l'intensité du jeu.
Le tempo reçoit l'offset émotionnel puis `tempo_bpm(p) × gate(i)`, borné au plafond.
Ni le lift d'intensité ni celui de tempo ne peuvent faire dépasser ce plafond ;
le tempo des émotions seules n'est **jamais abaissé**, même s'il le dépasse déjà.
À intensité 0 ou 1 : mêmes tempo et notes qu'en l'absence de rampe.

`MusicComposer::set_bpm` marque `MusicalParams::bpm_explicit = true` :
la valeur demandée gagne exactement, sans offset émotionnel ni lift de set.
L'automation CLI `bpm` emprunte cette même API. `reset_bpm` rétablit le pilotage
émotionnel et la rampe ; charger un profil remet aussi ce drapeau à `false`.
Une commande tempo ou émotionnelle du jeu reste applicable pendant un air :
le palier concerne la progression du **set**, pas le verrouillage du jeu.

Valeurs proposées dans l'étude §4.5 : +8 bpm / +0,08 pour les reels, +6 / +0,08
pour la gigue québécoise, +5 / +0,06 pour la gigue britannique, +3 / +0,05 pour
le hornpipe, +2 / +0,05 pour la valse ; marche sans rampe. Borne proxy
cap-bretonnaise, **pas une mesure québécoise ni une validation d'écoute**.

### Air signature

`[signature] return_rate = 0.17, memory = 3` (`[set.signature]` inline) reprend
le taux de 16,7 % d'airs déjà posés par le même membre dans un set antérieur
(The Session, étude §3.2). Probabilité finie 0..1, mémoire 0..16 airs ; absence,
taux zéro ou mémoire zéro : aucun retour.

Comme le bloc de l'étude §4.3, un tirage par air **2..L** est fixé au début du set.
Le premier air est neuf ; le premier set n'a pas encore de mémoire. Le taux s'applique
donc aux occasions éligibles, **pas à tous les airs de session** (mesurer séparément
les deux dénominateurs). Les `memory` dernières occurrences des sets précédents
sont examinées, puis filtrées par le poids de tonalité/mode à la valence courante
(strictement positif). Un matériel déjà joué dans le set courant est exclu aussi.
L'index vient de `floor((u / return_rate) × nombre_éligible)`, sans nouveau tirage.

Le moteur réutilise le `TunePlan` résolu et la `MelodyLine` écrite, **pas une graine** :
un changement d'intensité ou de registre émotionnel ne réécrit pas la mélodie.
La couleur parallèle et la dynamique restent pilotées par le jeu ; harmonie,
accompagnement, intro et interprétation sont recalculés pour l'état courant.
La fenêtre de mémoire référence les airs déjà conservés pour la position et le seek :
pas de deuxième copie, et `invalidate_future` retire automatiquement les futurs retours.
Le rejeu depuis la mesure 1 reconstruit la même mémoire.

## Arc sur plusieurs chorus (`[arc]`, phase 019 A3)

Un **chorus** est une passe complète de la forme (`TunePosition.tour`) : pour un profil `loop`
sous une grille fournie, la partie de N mesures répétée. `[arc]` (ou `arc.toml`, `[files] arc =
"arc.toml"`) fait varier d'un chorus à l'autre l'intensité, la densité, le feel de basse, le
comping, la batterie, les breaks et la retombée finale. Section absente : tous les chorus se
jouent pareil, et le flux de snapshots est celui d'avant (aucun tirage de plus). Aucun nom de
style dans l'algorithme ; la musique est dans les données. Aucune réharmonisation : les accords
publiés ne changent jamais.

**Numéro de chorus.** `MeasureSnapshot.position` expose déjà `tour` et `tours` (chorus de l'air),
`tune_in_set` et `tunes_in_set` (airs du set). Le chorus du set se lit `TunePosition::chorus()` =
`(tune_in_set - 1) × tours + tour` (0 en intro) et le nombre de chorus `TunePosition::choruses()` =
`tunes_in_set × tours`, `None` pour un **set sans fin** (`tunes_in_set == 0`). Un set d'un air
(`length = [{ tunes = 1 }]`) de `tours_per_tune` chorus est donc un arc **de fin connue** ; un set
sans `length` est un arc **sans fin**. Aucun champ de snapshot n'est ajouté.

```toml
# arc.toml
repeat_from = 1              # absent : la dernière étape se tient sans fin

[[stages]]                   # étape 1 (indice 0) : un chorus
choruses = 1                 # 1..=1024, défaut 1
variants = [                 # chaque chorus de l'étape tire sa variante par poids
  { weight = 3, intensity = 0.6, density = 0.5, bass_feel = "two_feel",
    comping = [{ tag = "sparse", weight = 3 }, { tag = "medium" }],
    percussion = [{ tag = "hat" }] },
  { weight = 1, intensity = 0.7, bass_feel = "two_feel", comping = [{ tag = "medium" }] },
]

[[stages]]                   # étape 2 : deux chorus
choruses = 2
variants = [{ bass_feel = "walking", percussion = [{ tag = "ride" }],
              breaks = [{ start = -1, bars = 1, chance = 0.5, tracks = ["Chords", "Bass"] }] }]

[[fall]]                     # retombée : variantes du dernier chorus quand le nombre est connu
intensity = 0.5
bass_feel = "two_feel"
breaks = [{ start = -2, bars = 2, tracks = ["Chords", "Bass"], keep_steps = [0] }]   # stop-time
```

| Champ | Bornes | Effet |
|---|---|---|
| `stages` | 1..=16 étapes | Dans l'ordre des chorus. Une étape couvre `choruses` chorus consécutifs (1..=1024) |
| `stages[].variants` | 1..=16, somme des poids dans u32 | Chaque chorus de l'étape tire **sa** variante par poids : un flux ChaCha8 dérivé de la graine de l'arc et du numéro de chorus |
| `repeat_from` | indice d'étape 0-based | Passé la dernière étape, l'arc reprend à cette étape en boucle. Absent : la dernière étape se tient (set sans fin ou plus de chorus que d'étapes) |
| `fall` | 0..=16 variantes | Variantes du **dernier chorus du set**, qui remplacent celles de l'étape. Jouées seulement si le nombre de chorus est connu **et d'au moins deux** ; set sans fin ou un seul chorus : jamais |
| `weight` | ≥ 1 (défaut 1) | Poids du tirage de la variante |
| `intensity` | 0..=2, défaut 1 | Multiplicateur de l'intensité lue par les courbes de `dynamics` (vélocités par piste, choix du motif de comping par intensité, niveau de densité de la batterie), borné à 0..1 après le produit. **Ne change pas le tempo** |
| `density` | 0..=2, défaut 1 | Multiplicateur de la densité rythmique lue par `density_percussion`, `density_chords` et les notes de passage de la basse (`accompaniment.bass.passing`), borné à 0..1. N'ouvre ni ne ferme une piste (`density_tracks`) |
| `bass_feel` | `walking` ou `two_feel`, absent : le profil | Exige `accompaniment.bass.walking`. `two_feel` : une attaque toutes les deux pulsations (temps 1 et 3 en 4/4), tenue jusqu'à l'attaque suivante, plus une arrivée à chaque changement d'accord |
| `comping` | 0..=8 `{ tag, weight }` | Restreint le tirage du comping aux motifs de `accompaniment.chords.comping` et aux phrases de `accompaniment.chords.phrases` dont le `tag` est nommé, au poids du motif (ou de la phrase) × le poids du tag. Aucun motif concerné : le tirage du profil ; aucune phrase concernée : les mesures se tirent une à une |
| `percussion` | 0..=8 `{ tag, weight }` | Même mécanisme sur les `[[patterns]]`, les `[[phrases]]` et les `[[fills]]` de `percussion.toml` (de la densité et de la partie de la mesure) ; les mesures de relance de `[[cadence_patterns]]` gardent leur pattern |
| `breaks` | 0..=8 | Voir ci-dessous |

**Tags.** `tag = "..."` (optionnel) sur un motif de `accompaniment.chords.comping` et sur un
`[[patterns]]` de batterie. Le tag n'est pas une portée : il ne change rien sans `[arc]`. Un tag
nommé par l'arc doit exister dans sa banque (erreur à `arc.stages[i].variants[j].comping[k].tag`).

**Breaks et stop-time.** `{ start, bars, chance, tracks, keep_steps }` : `start` est la première
mesure de la fenêtre, **1-based dans le chorus**, négatif compté depuis la fin (`-2` : l'avant-
dernière) ; `bars` mesures (défaut 1, la fenêtre reste dans le chorus) ; `chance` (0..=1, défaut 1)
tirée une fois par chorus et par break ; `tracks` (défaut vide : toutes les pistes) ; `keep_steps` :
les attaques des pistes touchées qui ne démarrent pas à l'un de ces pas sont retirées. Vide :
silence complet de ces pistes (break) ; `[0]` : stop-time sur le premier temps. Les accords publiés
(`chord_changes`) ne changent pas. Ni intro ni mesure de clôture du set ne portent d'arc.

**Tirage et seek.** La graine de l'arc est tirée une fois par air (deux tirages de 32 bits, **seulement si
le profil a un arc** : sans arc le flux ne change pas) ; celle du **premier air du set** sert à tout
le set, donc un arc qui traverse plusieurs airs reste cohérent. L'état d'une mesure ne dépend que de
sa position et de cette graine (variante, break, motif de comping et pattern de batterie par flux
dérivés) : une mesure se rejoue à l'identique après un seek.

Fixtures de contrat : `tests/fixtures/profiles/grille_arc_chorus/` (blues de douze mesures, quatre
chorus : exposition à deux temps, corps à la marche, retombée à deux temps) et
`grille_arc_chorus_aaba/` (forme AABA de trente-deux mesures, trois chorus) ; tests :
`harmonium_core/tests/arc_chorus_tests.rs`. Compte rendu :
[`specs/019-bibliotheque-accompagnements/a3-arc-chorus.md`](../specs/019-bibliotheque-accompagnements/a3-arc-chorus.md).

## Phrases d'accompagnement sur plusieurs mesures (phase 019, A1)

Le comping et la batterie se tirent **par phrase** de 1 à 8 mesures au lieu d'une mesure à la fois,
et la batterie peut jouer des **relances** de 1 à 4 mesures en fin de phrase. Tout est optionnel :
un profil sans `phrases` ni `fills` garde son flux, ses dumps et ses rendus (aucune des clés
n'est sérialisée, aucun tirage de plus). Aucun nom de style dans l'algorithme ; la musique est dans
les données.

**Découpage d'une passe.** Chaque passe d'une partie (`form.sequence` : une entrée = une passe) se
découpe en phrases par un tirage pondéré. Le découpage **ne franchit jamais la frontière d'une
partie, donc ni celle d'un chorus** (un chorus est un nombre entier de parties). Une phrase `at =
"part_end"` ou `"chorus_end"` se pose en **fin** de passe (la **mise en place** avant le changement
de section) ; le reste se découpe depuis le début de la passe, par tirages successifs parmi les
phrases qui tiennent dans ce qui reste. Une mesure sans phrase qui tienne (reste plus court que
la plus courte phrase) se joue comme sans phrase : motif tiré seul, pattern ordinaire.

| `at` | Place |
|---|---|
| `any` (défaut) | n'importe où dans la passe |
| `part_start` | première phrase de la passe (comping et batterie ; pas pour une relance) |
| `part_end` | dernière phrase de **chaque** passe de partie : mise en place avant un changement de section |
| `chorus_end` | dernière phrase de la dernière partie du chorus ; **l'emporte** sur `part_end` à cet endroit (la classe la plus précise qui a un candidat gagne) |

Dans un profil à une partie par chorus (grille utilisateur), `part_end` et `chorus_end` désignent la
même mesure. Une fin de passe n'est jamais optionnelle : si une phrase de fin concourt, elle joue ;
pour une fin facultative, déclarer aussi une phrase de fin neutre.

### Phrases de comping

```toml
[accompaniment.chords]
comping = [                                   # banque de motifs d'une mesure, indices 0..
  { weight = 4, tag = "medium", hits = [...] },     # 0
  { weight = 1, tag = "sparse", hits = [] },        # 1 (silence)
]

[[accompaniment.chords.phrases]]
weight = 3                  # défaut 1, > 0
tag = "medium"              # optionnel : étiquette que [arc] nomme, comme pour un motif
at = "part_end"             # défaut any
part = "B"                  # optionnel : nom de form.parts[], la phrase ne concourt que là
motifs = [0, 0, 1, 0]       # un motif de comping par mesure, 1..=8 ; indices dans `comping`
```

| Champ | Bornes | Effet |
|---|---|---|
| `phrases` | 0..=256 | Banque de phrases ; chaque mesure d'une phrase joue le motif désigné, à la place du tirage d'un motif par mesure |
| `weight` | ≥ 1 | Poids du tirage de la phrase |
| `motifs` | 1..=8 indices dans `comping` | Une mesure par entrée ; le motif joue comme en tirage ordinaire (même `hits`, même voicing et conduite de voix) |
| `at`, `part`, `tag` | voir plus haut | Place, portée, étiquette |

### Phrases et relances de batterie

```toml
# percussion.toml
fills_none_weight = 3       # poids de « pas de relance » dans le tirage d'une fin de phrase (défaut 0)

[[phrases]]                 # suite de patterns de même densité, indices de [[patterns]]
density = 1                 # 0, 1 ou 2 : ne concourt qu'à cette densité
weight = 3
tag = "ride"
at = "any"
part = "B"                  # optionnel
patterns = [3, 4, 3, 5]     # 1..=8 patterns de la densité, sans portée `part` ni `tour`

[[fills]]                   # relance de fin de phrase
density = 2
weight = 3
tag = "ride"
at = "part_end"             # any (défaut), part_end ou chorus_end
start_step = 12             # pas de la première mesure où la relance prend la main (levée au temps 4)
bars = [                    # 1..=4 mesures : la relance s'achève avec la phrase
  { hits = [{ step = 12, voice = "snare" }, { step = 15, voice = "snare" }] },
]
```

- **Phrases de batterie** : le découpage ci-dessus, à la densité de la mesure (celle de la
  batterie après l'intensité et la densité de l'arc). Un `[[patterns]]` référencé n'a ni `part`
  ni `tour` (erreur à `percussion.phrases[i].patterns[j]`). Sans phrase à la densité de la mesure,
  ou sous une étiquette d'arc qu'aucune phrase ne porte : pattern de l'arc ou ordinaire.
  Une mesure de `[[cadence_patterns]]` garde son pattern.
- **Relance** : tirée à **chaque fin de phrase**, avec le flux de la passe et de la mesure. La fin
  est celle de la phrase de batterie qui couvre la mesure ; hors des phrases de la table (densité
  sans phrase, reste de passe), c'est une fin de phrase de la forme (`form.parts[].phrase_ends`
  ou les rôles cadentiels). Une relance de `n` mesures couvre les `n` dernières de la phrase
  (jamais plus que la phrase) : dans la première, le pattern joue jusqu'à `start_step` puis la
  relance prend la main (ses coups sont à `start_step` ou après) ; les suivantes sont celles de la
  relance seule. Les relances de la densité concourent par poids (× le poids du tag sous un arc),
  parmi la classe de place la plus précise qui en a une, avec « pas de relance »
  (`fills_none_weight`). Sous un arc qui nomme des tags, seules les relances de ces tags
  concourent ; aucune : pas de relance.
- **Remplace `[[cadence_patterns]]`** à la densité qu'elle déclare : une relance et un pattern de
  cadence pour la même densité sont refusés (`percussion.fills[i].density`). Une table sans
  `fills` garde son comportement (le pattern de cadence remplace la mesure de fin de phrase).

**Graine et seek.** Les phrases se tirent avec `ActiveTune::accompaniment_seed` (tirée aussi quand le
profil n'a que des phrases ou relances de batterie, sans `[accompaniment]`) et l'indice de la
première mesure de la passe, par des flux ChaCha8 distincts (comping, batterie ; fin de passe,
ouverture, relance). La phrase d'une mesure se recalcule depuis sa seule position : aucun état
d'une mesure à l'autre, donc le même rejeu après un seek. Le découpage se calcule une fois par
mesure générée (jamais par mesure rejouée de la conduite de voix), côté compositeur : rien de plus
dans le callback audio.

**Erreurs** : chemin du champ (`accompaniment.chords.phrases[2].motifs[1]`,
`percussion.fills[0].bars[0].hits[1].step`, `percussion.phrases[0].patterns[1]`…). Champ inconnu :
refusé.

Fixtures de contrat : `tests/fixtures/profiles/grille_phrases/` (blues de douze mesures, quatre
chorus) et `grille_phrases_aaba/` (AABA, `part_start`, `part_end`, `chorus_end`) ; tests :
`harmonium_core/tests/phrases_tests.rs`. Compte rendu :
[`specs/019-bibliotheque-accompagnements/a1-phrases.md`](../specs/019-bibliotheque-accompagnements/a1-phrases.md).

## Swing, métriques et feels latins (phase 019, A6)

Tout est **optionnel en données** : un profil sans `[swing]`, sans `[clave]`, sans `voices` ni
`probability` garde exactement ses dumps, ses WAV et son MIDI (`TUNING_VERSION` inchangé).

### Swing selon le tempo et par piste (`[swing]`, ou `swing.toml` via `[files]`)

```toml
reference = "PercussionHigh"            # piste dont le rapport est celui de la grille (défaut)
[tracks.PercussionHigh]                  # une courbe par piste : points [BPM, rapport]
points = [[140.0, 0.7213], [299.3089, 0.5658]]
[tracks.Bass]
points = [[130.0, 0.7148], [299.3089, 0.6405]]
[tracks.Chords]
points = [[150.0, 0.7021], [299.3089, 0.5934]]
```

- **Courbe** : interpolation linéaire entre points (1 à 16, BPM strictement croissants dans
  20..400, rapport dans `0.5..0.75`), **valeur de l'extrémité** hors des points : aucune
  extrapolation hors des tempos mesurés (JTD : 102 à 299 BPM). Les valeurs ci-dessus sont celles
  de `references/jtd-swing-tempo` du labo (`a6_table.curve`, régression robuste, plateau puis
  décroissance) : batterie charnière 140 BPM, pente −0,0976 par 100 BPM ; basse 130 BPM, −0,0439 ;
  piano (comping) 150 BPM, −0,0728. Les bornes `0.5..0.75` du rapport suffisent aux mesures
  (0,566 à 0,721) : elles ne sont pas élargies.
- **Grille et pistes** : le rapport de la piste `reference` est celui de la **grille**
  (`Measure::swing_ratio`, lu par le lecteur : `BarTiming::step_offset`, `time_plan`). Une autre
  piste dont le rapport diffère est replacée par un décalage de chaque attaque en millièmes de pas
  (`performance.offset`) calculé en inversant la géométrie du swing : sa croche de contretemps
  tombe exactement à son rapport, les temps ne bougent pas, la grille et la longueur des mesures
  non plus. Écart maximal à la référence : 0,12 (le décalage reste sous le demi-pas que borne
  l'interprétation). Une piste sans courbe suit la grille. Le décalage est celui du rapport, pas
  du tempo : seek, export MIDI et plan partagé le reproduisent.
- **Priorité de l'hôte** : `SetSwing` (REPL, `--swing`, API `MusicComposer::set_swing_ratio`)
  pose `MusicalParams::swing_explicit` : le rapport demandé s'applique à **toutes** les pistes et
  **la courbe est ignorée**, jusqu'à `reset` ou au chargement d'un profil. Sans commande directe,
  l'écart entre le rapport demandé (`MusicalParams::swing_ratio`) et `meter.default_swing_ratio`
  décale toutes les courbes (un dosage tiré par graine, A2, passe par là). Un seul point d'entrée :
  `SwingByTempo::resolve`, appelé par `TuneGenerator::swing_for`.
- **Tempo lu** : le tempo de la mesure à la génération (`tempo_for`) ; un `SetTempo` audible qui
  ne régénère pas les mesures ne déplace pas le swing déjà publié.

Mesures sur le MIDI exporté (`specs/019-bibliotheque-accompagnements/a6-mesure-swing.py`,
compte rendu `a6-swing-metriques.md`) : le rapport de chaque piste suit sa courbe JTD à ±0,003.

### Grilles ternaires et métriques

Le swing partage chaque noire en deux segments, jusqu'au pas `⌈pas par noire / 2⌉` : le milieu sur
quatre pas par noire (croches), **le troisième pas sur trois** (grille de 12 pas en 4/4, triolets :
`steps_per_bar = 12`, `steps_per_unit = 3`, `steps_per_pulse = 3`). Sur une grille de triolets, `0,6667`
ne change rien et `0.5` laisse la grille telle quelle ; un rapport entre les deux la resserre, un
rapport plus grand la durcit (shuffle 12/8 par tempo). Une grille qui ne tombe pas sur des noires
entières, une noire d'un seul pas ou une pulse qui n'est pas un nombre entier de noires (6/8 sur
12 pas, pulse de six pas ; 12/8 sur 24 pas) **ne swingue jamais**, quel que soit le rapport : le
ternaire est la grille (aucune mesure de swing composé n'existe dans JTD). 3/4 sur 12 pas (quatre
pas par noire) swingue comme le 4/4. `BarTiming::swing_unit_steps` dit si une grille swingue.

Le MIDI enregistré et l'export `timeline_to_midi` comptent **480 ticks par noire** quelle que soit la
grille (`events::midi_ticks_per_step` : 120 par pas sur quatre pas par noire, 160 sur 12 pas en
4/4) ; avant A6 ils comptaient 120 ticks par pas, soit une mesure de 12 pas en 4/4 de 1440 ticks
au lieu de 1920. Hors de ce cas, rien ne change. Limites : l'export MusicXML (`divisions = 4`) et le
champ `beat` du transport (`TICKS_PER_BEAT = 4`) supposent encore quatre pas par noire.

### Micro-timing par voix de batterie (`performance.tracks.<piste>.voices`)

```toml
[performance.tracks.PercussionHigh]
voices = [
  { note = 51, on_beat = 0.045, off_beat = 0.07 },   # ride
  { note = 44, on_beat = -0.07 },                    # pédale
  { note = 38, on_beat = -0.02, jitter = 0.03 },     # caisse claire
]
```

Une voix est une **note MIDI** de la piste (note GM du kit). Son décalage (fraction de pas, −0,2..0,2)
s'ajoute aux `offsets` et au `jitter` de la piste ; `on_beat` vaut sur un pas multiple de
`steps_per_pulse`, `off_beat` ailleurs (défaut : `on_beat`) ; `jitter` (0..0,1) remplace celui de la
piste pour cette voix. Une piste qui déclare des voix traite chaque note seule : le ride, la pédale
et la caisse claire d'un même pas ne partagent plus un décalage (GMD : ride +9 ms sur le temps et
+14 ms hors temps, charleston fermé −7 ms, pédale −14 ms, caisse claire et grosse caisse −3 à
−6 ms ; L2 § 6 point 3). Une note par entrée, une seule entrée par note.

### Coup probabiliste (`probability`)

`{ step = 5, voice = "snare", delta_velocity = -10, probability = 0.2 }` : le coup joue avec la
probabilité donnée (0 exclu, 1 inclus), dans un pattern ordinaire ou une relance. Le tirage est **sans
état** : fonction de la graine d'accompagnement de l'air, de la mesure, du pas, de la note et de la
piste (`percussion::hit_plays`), donc stable par graine et identique après un seek. La graine
d'accompagnement est tirée dès qu'un coup est probabiliste (même règle que les phrases). Notes
fantômes du GMD : 5 à 25 % (L2 § 6 point 5).

### Clave (`[clave]`, `clave = "three" | "two"` sur les phrases)

```toml
[clave]
direction = "either"     # "three_two" (défaut), "two_three", ou "either" : tiré une fois par air
[[percussion.phrases]]   # ou [[accompaniment.chords.phrases]]
density = 1
patterns = [2, 3]        # mesure « trois » puis mesure « deux »
clave = "three"          # la phrase commence sur une mesure « trois » du cycle
```

La clave est un cycle de **deux mesures** (« trois » aux trois attaques, « deux » aux deux), continu
d'un chorus à l'autre : la mesure `n` de l'air (0-based, tours compris) est « trois » quand `n` est
pair dans une 3-2, impair dans une 2-3 ; `either` tire le sens de la graine d'accompagnement. Une
phrase qui porte `clave` ne peut **commencer** que sur une mesure de ce côté (y compris la phrase de
fin de passe) : les phrases de comping et de batterie d'A1 restent alignées sur la clave, mesure par
mesure, quel que soit le découpage. `clave` sans `[clave]` est refusé (chemin du champ). La clave
elle-même est écrite en données : patterns de la voix `claves` (son 3-2 : mesure « trois » aux pas
0, 6, 12, mesure « deux » aux pas 4, 8 sur 16 pas).

Montuno et bossa s'expriment avec les mécanismes existants : comping anticipé (`anticipate = true`,
l'attaque joue l'accord en vigueur à la fin de sa durée) et basse anticipée (gabarit avec
`pickup = { when = "bar_end", ... }` : note de la basse de l'accord suivant `lead` pas avant la
barre). Fixture `grille_clave`.

### Rôles de percussion latine (kit GM de GeneralUser GS)

Les rôles sont des `[[voices]]` de `percussion.toml` sur les deux pistes existantes, par note GM du
kit (banque 128, programmes 0, 32 et 40 : vérifié sur `GeneralUser-GS.sf2`, instruments « Accessory
Drums 1 », « Accessory Drums 2 », « Agogo », « Wood Block_1 ») :

| Rôle | Note | Piste de la fixture |
|---|---|---|
| cloche (cowbell) | 56 | `PercussionHigh` |
| bongo aigu / grave | 60 / 61 | `PercussionHigh` |
| conga sourde / ouverte / grave | 62 / 63 / 64 | `PercussionLow` |
| timbale aiguë / grave | 65 / 66 | `PercussionHigh` / `PercussionLow` |
| cabasa, maracas | 69, 70 | au choix |
| claves | 75 | `PercussionHigh` |
| shaker | 82 | `PercussionHigh` |

Fixtures de contrat, sans Odin (`tests/fixtures/profiles/`) : `grille_shuffle_ternaire` (4/4 sur 12
pas, micro-timing de voix, coups probabilistes), `grille_valse_jazz` (3/4, arc et phrases),
`grille_six_huit` (6/8, arc et phrases), `grille_clave` (clave, montuno, tumbao). Tests :
`harmonium_core/tests/swing_metriques_tests.rs`, `harmonium_host/tests/midi_grille_ternaire_tests.rs`.

## Habillage par graine et orchestration par chorus (`[session]`, phase 019 A2)

Une graine ne changeait que le rythme du comping et des notes de passage : tempo, tonalité,
instruments, voicings, kit et arc étaient fixes par profil. `[session]` (ou `session.toml`,
`[files] session = "session.toml"`) déclare des **variantes de session** pondérées, tirées **une
fois par graine** (au premier air, avant sa construction), qui surchargent des réglages globaux
**en liste blanche**. Section absente : aucun tirage, flux, dumps et rendus identiques à ceux
d'avant. Aucun nom de style dans l'algorithme ; la musique est dans les données.

```toml
# session.toml
[[variants]]
group = "tempo"            # un tirage indépendant par groupe ; défaut « main »
name = "lent"              # optionnel, publié avec la variante tirée
weight = 1                 # défaut 1
bpm = [100.0, 116.0]       # une valeur, ou une plage dont la graine tire un BPM entier
swing = 0.70

[[variants]]
group = "comping"
name = "guitare"
instruments = { chords_program = 26 }

[[variants]]
group = "kit"
name = "balais"
instruments = { percussion_bank = 128, percussion_program = 40 }

[[variants]]
group = "arc"
name = "orchestre"
arc = "orchestre"           # arc nommé ci-dessous, à la place de [arc]

[arcs.orchestre]            # même schéma que [arc] ; chemin d'erreur `session.arcs.orchestre…`
[[arcs.orchestre.stages]]
choruses = 1
variants = [{ instruments = { percussion_program = 40 }, bass_feel = "two_feel" }]
[[arcs.orchestre.stages]]
variants = [{ instruments = { chords_program = 16 } }]
```

**Groupes.** Les variantes se rangent en groupes (`group`) ; chaque groupe tire **sa** variante, dans
l'ordre où les groupes apparaissent, par un flux ChaCha8 dérivé de la graine de session. Neuf
groupes de deux à quatre variantes donnent des milliers d'habillages sans les énumérer. **Deux
groupes ne règlent jamais le même champ** (erreur de chargement, `session.variants[i].champ`) :
l'ordre des groupes ne décide de rien. Deux variantes du même groupe le peuvent (une seule est
tirée). Bornes : 64 variantes, 16 groupes, 16 arcs nommés.

| Champ | Bornes | Effet |
|---|---|---|
| `weight` | ≥ 1, somme du groupe dans u32 | Poids dans le groupe |
| `bpm` | valeur, ou `[bas, haut]` ordonné, dans `meter.min_bpm..max_bpm` | Tempo : remplace `default_bpm` comme **base** de la courbe `tempo` de `dynamics` (l'intensité s'y ajoute encore). Une plage tire un BPM entier |
| `tonality` | `{ key = 0..11, mode }` ; `mode` atteignable par le profil (`reachable_modes`) | Tonalité et mode **du premier air** ; les airs suivants se tirent comme avant. Les accords relatifs à la tonique se transposent |
| `instruments` | `melody_program`, `countermelody_program`, `chords_program`, `bass_program`, `percussion_bank`, `percussion_program` (tous optionnels) | Remplace ces champs de `[instruments]` ; une piste qui n'a pas de programme dans le profil ne peut pas en recevoir |
| `odin` | comme `[odin]` (une piste à la fois) | Zones Odin qui remplacent celles de `[odin.<Piste>]` (jeux seulement ; sans `odin` compilé, la piste joue son programme) |
| `voicing` | une famille de `accompaniment.chords.voicing` (`rootless`, `guide_tones`, `drop2`, …) | Famille de voicings du comping ; validée contre le profil (fenêtre `low..high`). Une famille ajoutée plus tard à l'énumération `ChordVoicing` est tirable sans autre changement |
| `swing` | 0,5..=0,75 | Rapport de swing |
| `register_octaves` | `{ Piste = -1..=1 }`, pistes mélodiques | Décale la piste d'octaves entières (les classes de hauteur ne changent pas) ; une note qui sortirait de `0..=127` garde sa hauteur |
| `comping`, `percussion` | 0..=8 `{ tag, weight }` | Tags de repli pour le comping (motifs et phrases) et la batterie (patterns, phrases, relances) : le chorus d'un arc qui déclare les siens passe devant |
| `arc` | nom d'un arc de `[arcs]` | Remplace `[arc]` pour la session |
| `intensity`, `density` | 0..=2 | Multiplicateurs de base de l'intensité et de la densité lues par `dynamics`, comme ceux de l'arc (intensité × densité de session × celles du chorus) ; ni le tempo ni la mélodie |

**Orchestration par chorus.** `instruments = { … }` dans une variante de **l'arc** (étape ou
`fall`) remplace les programmes à partir de la première mesure du chorus, par-dessus ceux de la
session ; un chorus qui n'en déclare pas **les rend** (le changement se défait au chorus
suivant). La clôture du set garde les programmes de son chorus ; l'intro, ceux de la session.
Les mesures publient leurs programmes (`MeasureSnapshot.instruments`, `Measure.instruments`) dès
que le profil orchestre (une variante de session ou une variante d'arc règle `instruments`) ;
sinon rien n'est publié et le contrat est celui d'avant.

**Visible.** Chaque `MeasureSnapshot` d'une session à variantes porte `session` : une entrée par
groupe (`group`, `index` dans `[[session.variants]]`, `name`) ; `instruments` : les programmes qui
sonnent. Le tempo, le swing, la tonalité (`position.key`, `position.mode`) et les accords publiés
sont ceux de la variante. Côté hôte : `TuneGenerator::session()` et `MusicComposer::session_variants()`
(après la première mesure composée), `composition_bpm` des snapshots (tempo nominal de la variante).

**Qui gagne, hôte contre variante.**

| Réglage | Sans commande de l'hôte | Commande de l'hôte, qui l'emporte |
|---|---|---|
| tempo | celui de la variante (base de la courbe `tempo`) | `SetBpm` / `ScoreCommand::Tempo` : exactement la valeur de l'hôte, sur chaque mesure ; la rampe de set et la variante n'y touchent plus. `SetAudibleTempo` passe encore par-dessus au rendu |
| tonalité et mode | ceux de la variante, au premier air | `[fixed_tonality]` du profil de l'hôte (donc une grille utilisateur transposée par lui) : la variante est **ignorée**, même si son mode n'est pas atteignable |
| swing | celui de la variante | `SetSwing` (`MusicalParams::set_swing_ratio`, `swing_explicit`) jusqu'au prochain profil ou `Reset` |
| programmes, voicings, registre, tags, arc, intensité, densité, Odin | ceux de la variante | aucune commande : un hôte qui ne veut pas d'une variante installe un profil sans `[session]` |

**Programmes jusqu'au rendu.** Le playhead n'émet rien de plus : le playback copie
`Measure.instruments` (`Copy`) au chargement de la mesure et annonce `AudioEvent::SetInstruments`
au premier pas dont les programmes diffèrent de ceux déjà annoncés (aucune allocation dans le
callback). Le renderer SoundFont change de programme (et de banque pour le kit) ; l'enregistreur
MIDI écrit un `program_change` par piste qui change, à la barre (kit sur le canal GM 9) ;
`timeline_to_midi` fait de même depuis les mesures. Les attaques anticipées de la mesure suivante
(avant la barre) sonnent encore avec l'instrument précédent. Un `UpdateMusicalParams` ou un
`ProgramChange` de l'hôte efface l'annonce : la mesure sous la tête la redonne au pas suivant.

**Tirage et seek.** Deux entiers de 32 bits du RNG injecté, **seulement si le profil a des
variantes** ; puis un flux ChaCha8 dérivé par groupe (et par groupe pour une plage de BPM). Le
rejeu (`reset_to_initial`, `deterministic_seek`) retire la variante et la retire de nouveau, la
même. Un changement de profil la retire aussi.

**Erreurs** : chemin du champ (`session.variants[3].tonality.mode`, `session.arcs.orchestre.stages[0]
.variants[0].instruments.bass_program`), fichier et ligne quand le profil vient de `session.toml`.
Champ inconnu : refusé.

Fixture de contrat : `tests/fixtures/profiles/grille_habillage/` (blues de douze mesures, quatre
chorus, arc et phrases ; 23 variantes en neuf groupes, 3 456 habillages) ; tests :
`harmonium_core/tests/session_variants_tests.rs`, `harmonium_host/tests/session_orchestration_tests.rs`.
Compte rendu : [`specs/019-bibliotheque-accompagnements/a2-habillage.md`](../specs/019-bibliotheque-accompagnements/a2-habillage.md).

## Difficulté de la mélodie (`[melody.difficulty]`, phase 019 E10)

Section optionnelle de `[melody]`, absente des profils distribués : sans elle le flux est celui
d'avant. Elle borne la mélodie **publiée** (la piste `Melody` du snapshot), pas seulement les
cellules : une borne tient sur chaque paire de notes consécutives, qu'elles soient dans la même
cellule, dans deux cellules voisines, de part et d'autre d'une frontière de phrase ou de partie,
d'une reprise ou d'un tour au suivant.

```toml
[melody.difficulty]
max_leap_semitones = 4            # plus grand saut entre deux attaques (1..24)
max_attacks_per_bar = 4           # attaques par mesure (1..cell_units)
min_note_units = 2                # plus petite durée, en unités de cellule (1..cell_units)
max_cell_ambitus_semitones = 7    # ambitus des attaques d'une mesure (1..36)
out_of_scale = false              # défaut true : faux = aucune note hors de la gamme de l'air
ornaments = false                 # défaut true : faux = aucun ornement sur la mélodie
chord_fit = false                 # défaut true : faux = pas de rapprochement des accords
```

Chaque borne omise est sans effet. `ornaments` et `chord_fit` valent `true` par défaut : les
ornements (`[ornaments]`) et l'ajustement des notes aux accords des modèles `loop` et
`rewrite_functional` s'ajoutent alors à la ligne écrite, **hors des bornes** ; pour que le
snapshot lui-même respecte `max_leap_semitones`, `max_attacks_per_bar` et `min_note_units`, les
mettre à `false`. `out_of_scale = false` retient, dans l'ajustement aux accords, les seules notes
de la gamme de l'air.

Mécanisme (`compose/difficulty.rs`) : le tirage des cellules préfère celles dont le saut interne,
le saut d'entrée et l'ambitus respectent le filtre ; puis la ligne jouée de l'air entier est
réécrite. Les attaques les plus courtes sont fusionnées (la note retirée prolonge la précédente ;
temps faibles et fin de mesure d'abord) jusqu'à `max_attacks_per_bar` et `min_note_units` ; les
hauteurs sont reprises par programmation dynamique sur toute la ligne, au plus petit écart total
aux notes écrites, sur les notes de la gamme dans la tessiture, avec le saut et l'ambitus pour
contraintes (la dernière note d'une cadence résiste, une ligne constante convient toujours). La
dernière mesure se raccorde à la première, et la dernière mesure du premier tour aussi (intro
d'entrée par la fin). Les mesures d'une partie qui tait la mélodie (`tracks_off`) ou muettes par
l'alternance ne comptent pas : le saut se borne d'une mesure publiée à la suivante.

Un profil à filtre compose sa ligne mesure par mesure sur tous les tours (comme les profils à
reprises variées : `material_bars` couvre tous les tours). **La levée d'intro
(`arrangement.intro.pickup_eighths`) est retirée** : ses croches ne tiennent pas dans les bornes.

Limites : les bornes valent dans un air et ses tours ; la jonction entre deux airs d'un même set
n'est pas bornée. Une bascule de mode parallèle (`dynamics.parallel_modes`) déplace des notes
d'au plus un demi-ton après le filtre (un saut peut varier de deux). Le filtre ne peut que simplifier : une mélodie déjà
clairsemée garde sa densité (régler `min_note_units` au-dessus de sa durée native pour la calmer).

Pilotage par l'hôte : en données seulement. Le filtre s'applique au tirage de la ligne d'un air ;
un `SetStyleProfile` en cours d'air laisse l'air courant tel quel (comme tout changement de
cellules ou de tessiture) et change les airs suivants. Une application qui règle un niveau
(`harmonium_training`) installe le profil du niveau avant `SetSeed`/`NewMelody`, comme pour la
tessiture : `profile.melody.difficulty = Some(MelodyDifficulty { .. })` sur sa copie du profil.

Mesure de référence : `cargo run -p harmonium_core --example melody_difficulty --release -- --filter`
(3 graines × 64 mesures, les profils distribués qui ont une mélodie, quatre niveaux).

## Alternance de mesures (question-réponse, `[arrangement.alternation]`, phase 019 E11)

Section optionnelle du fichier d'arrangement (ou de `[arrangement]` inline), à côté de `[entries]`,
même clé par nom de piste. Une piste y joue `play` mesures de chaque cycle de `period` mesures et
se tait les autres :

```toml
[alternation]
Melody = { period = 4 }                        # joue 2 mesures, se tait 2
Countermelody = { period = 8, play = 3, phase = 2 }
```

- `period` : cycle en mesures, `2..=32` ; `play` : mesures jouées, `1..period`, défaut `period / 2` ;
  `phase` : `0..period`, décalage du cycle. La mesure `n` (0-based) de la forme est jouée quand
  `(n + phase) % period < play` ; `phase = play` : la piste se tait d'abord.
- Les mesures se comptent depuis la première mesure de la forme de l'air, hors intro, en continu
  d'un tour à l'autre ; un nouvel air repart de zéro. L'intro et la mesure de clôture ne sont
  pas touchées.
- Une piste muette est composée comme toujours puis retirée du snapshot : les autres pistes
  (vétos de la basse, voix dérivées comme `Countermelody`, qui suit la mélodie) restent celles du
  profil sans alternance. Pour taire aussi la seconde voix, la déclarer avec la même alternance.
- Toute piste de `[entries]` peut alterner ; une piste absente de `[entries]` est refusée. Les
  drapeaux de jeu (`EnableMelody`…) et les entrées progressives restent des plafonds.

Pilotage par l'hôte : `SetStyleProfile` avec `arrangement.alternation` renseigné (ou vidé) agit sur
les mesures encore à composer, sans toucher aux cellules : c'est la lecture du profil courant à
chaque mesure. Il réinitialise tempo et swing, que l'application renvoie.

## Règles de phrase

Le profil décide des répétitions ; le moteur ne suppose ni huit mesures, ni
deux parties, ni un regroupement 4+4. `roles.len()` donne la longueur de chaque
partie ; les références de motifs et les frontières sont 1-based. La première
mesure accepte les mêmes choix pondérés que toutes les autres (sans référence
antérieure locale, puisqu'elle n'en a pas).

Les valeurs ci-dessous sont des **exemples de réglage**, pas des estimations
historiques. Les valeurs omises conservent les reprises/tours identiques,
la résolution `per_tune` et les contraintes de registre antérieures.

| Paramètre | À l'oreille / unité | Mécanique | Libre |
|---|---|---|---|
| `form.sequence` | Ordre et nombre de passes ; indices des parties déclarées | `[0,0,1,1]` | `[0,1]`, ou quatre parties `[0,1,2,3]` |
| `parts.roles` | Une décision par mesure ; première mesure comprise | `"Open"` puis `CopyOf` | `{ choose = [{role="Open",weight=1},{role="Free",weight=3}] }` dès m.1 |
| `form.resolve` | `per_tune` garde le matériel ; `per_pass` retire rôles **et cellules** à chaque passe jouée, tours compris | `"per_tune"` (défaut) | `"per_pass"` |
| `form.repeat_variation` | Probabilité 0..1 de tenter une variation à chaque mesure des passes 2+ | `0.0` (défaut) | `0.35` |
| `form.tour_variation` | Même probabilité aux tours 2+, à partir du premier tour déjà arrangé | `0.0` (défaut) | `0.20` |
| `form.variation_operations` | Menu des opérations autorisées, départ équiprobable ; essaie les autres si l'opération ne convient pas | `["octave"]` | `["varied_end","repeated_notes","octave"]` (défaut) |
| `form.varied_end_keep` | Fraction conservée avant la fin variée, strictement entre 0 et 1, tronquée à l'unité de cellule ; vaut aussi pour `VariedEndOf` | `0.75` | `0.25` ; défaut `0.5` |
| `form.cadence_min_units` | Tenue tonique minimale de `CadTonic`, 1..`cell_units` unités | `4` | `2` (défaut) |
| `form.cadence_search_from` | Fraction 0..1 à partir de laquelle rechercher une tonique déjà tenue ; sinon pose la tenue minimale en fin de mesure | `0.0` | `0.75` ; défaut `0.5` |
| `parts.phrase_ends` | Frontières explicites pour percussion cadentielle et changements de mode émotionnels ; ne remplace pas les rôles harmoniques | `[4,8]` | `[3,8]` (3+5) ; vide/omis : rôles cadentiels |
| `parts.link_previous` | Continuité de hauteur avec la partie précédente ; les `CopyOfPart` restent explicites | `true` (défaut) | `false` : départ indépendant |
| `parts.melody_center_offset` | Centre de la partie en demi-tons relativement au centre mélodique | `0` partout | `0` en A, `2` en B, `-3` en C |
| `form.avoid_adjacent` | Évite de retirer la même cellule libre immédiatement, octave commune ignorée | `false` | `true` (défaut) |
| `form.opening_retries` | Nombre maximal de nouveaux tirages si deux airs commencent pareil, 0..10000 | `0` | `8` (défaut) |
| `form.close_part_loop` | Pénalise le saut de retour de fin de partie vers son début | `true` (défaut) | `false` |
| `form.max_leap_degrees` | Seuil de saut préféré à l'entrée et au retour ; pas diatoniques, 0..28, contrainte souple | `4` (quinte, défaut) | `7` |
| `form.contour_window_semitones` | Écart préféré au centre de partie, 0..127 ; la meilleure candidate sert de repli | `0.0` | `8.0` ; défaut `4.0` |
| `form.register_tolerance_semitones` | Tolérance autour de la frontière médiane des registres, 0..127 | `0.0` | `12.0` ; défaut `6.0` |
| `parts.rhythm` | Distribution de durées aux positions choisies, voir ci-dessous | Un motif unique | Plusieurs motifs aux poids de corpus |
| `parts.second_ending` | Variante de la dernière mesure à la dernière passe de la partie | Omettre, ou `"CadTonic"` | Distribution tonique/quinte/autre |
| `parts.ornament_protected_bars` | Mesures sans aucun ornement (cadences), tous types de `[ornaments]` ; omission : dernière mesure si son rôle est cadentiel ; `[]` : aucune | `[4,8]` | `[]` |
| `parts.tracks_off` | Pistes tues sur la partie, lues par la même porte que l'arrangement et la densité ; les tenues liées qui traversent la frontière se ferment à la barre et reprennent ensuite (repli sûr) ; la basse muette dispense de motif pour la partie | `["PercussionLow", "PercussionHigh"]` en B | `["Bass", "PercussionLow", "PercussionHigh"]` au pont de `nt_desert` |
| `melody.params.max_continuity_step_distance`, `continuity_max_retries` | Distance diatonique préférée entre cellules et nombre d'essais ; contraintes souples, tessiture prioritaire | `2`, `16` | `7`, `4` |

Une variation conserve le mode et la tessiture. `repeated_notes` remplace une
répétition intérieure par son voisin diatonique ; `octave` déplace la cellule
entière seulement si elle tient dans la tessiture ; `varied_end` emprunte une
fin compatible de la banque. Les cibles et tenues terminales des cadences sont
protégées. Une opération impossible est sautée, sans fabriquer de notes hors
banque ; une mesure sans aucune opération applicable reste identique.
`repeat_variation` se tire au premier tour : si `tour_variation = 0`, ce tour
varié reste identique aux suivants. `per_pass` retire indépendamment chaque
passe, même si les rôles sont fixes ; une copie entre parties vise toujours
le matériel de première passe de la partie source.

### Rythme par position

```toml
# Dans une [[form.parts]], durées en unités de cellule, somme = melody.cell_units.
rhythm = [
  { bar = 4, patterns = [
    { durations = [2,1,1,1,1,2], weight = 3 },
    { durations = [1,1,1,1,4], weight = 2 },
  ] },
  { bar = 8, patterns = [
    { durations = [2,2,4], weight = 1 },
    { durations = [3,1,2,2], weight = 1 },
  ] },
]
```

Le rythme agit sur `Open`, `Free`, `CadOpen`, `CadOther`, `CadHalf` et `CadFifth`.
Les attaques sont repositionnées, la dernière hauteur est conservée puis la
cible cadentielle est appliquée. `CadFull`/`CadTonic` gardent leurs tenues
cadentielles ; les copies, séquences et fins variées conservent le rythme du
motif : les préférences de la position ne cassent jamais une copie.
Chaque distribution contient 1..256 motifs, durées entières positives, poids
non négatifs à somme positive dans u32, une seule entrée par position.

Les sept profils utilisent aux m.4/8 les effectifs `real.rhythm.cells` de leurs
JSON `references/structure` du labo. Préférence cadentielle explicite :
conditionner sur les motifs couvrant exactement la mesure, représentables
sur la grille actuelle, à durées **contrastées** (au moins deux durées
distinctes). Les croches continues, pulsations isochrones et tenues uniques
restent disponibles dans les cellules et cadences fermées ; on ne les ajoute
pas une seconde fois aux fins libres. Aucun arrondi des effectifs, aucune
nouvelle cellule ni retouche des poids de rôles. Le labo ne fournit pas encore
de distribution de motifs de durée *par position* : la même distribution
conditionnelle est donc configurée séparément pour A/B:m.4/8, pas présentée
comme une mesure positionnelle directe. Les fractions et silences du corpus
non représentables par les cellules entières ne sont pas quantifiés en secret.

### Trois recettes

À adapter dans un profil complet ; les snippets remplacent les champs
correspondants, pas les banques et instruments.

**Comptine très répétitive** : `resolve = "per_tune"`, `repeat_variation = 0.0`,
`tour_variation = 0.0`, `avoid_adjacent = false`, `opening_retries = 0`,
et un seul set sans changement de couleur (`set.length = [{ tunes = 16, weight = 1 }]`,
`set.key` à 0 sauf `same = 1`). Pour chaque partie :

```toml
roles = ["Open", {CopyOf=1}, {CopyOf=1}, "CadHalf",
         {CopyOf=1}, {CopyOf=2}, {CopyOf=3}, "CadTonic"]
phrase_ends = [4,8]
# Omettre second_ending et rhythm pour garder exactement le matériel.
```

**Marche libre « a b c d »** : `resolve = "per_tune"`, aucune copie imposée,
`repeat_variation = 0.0`, parties de huit mesures (ou toute autre longueur) :

```toml
roles = ["Free", "Free", "Free", "CadOpen", "Free", "Free", "Free", "CadTonic"]
link_previous = false
phrase_ends = [4,8]
# Conserver les préférences rhythm du profil de marche.
```

La recette retire les liens obligatoires ; « a b c d » n'est pas garanti à
chaque tirage, puisque des cellules peuvent fortuitement coïncider.

**Reprise variée de violoneux** : préserver les motifs pondérés du profil,
`resolve = "per_tune"`, `repeat_variation = 0.35`, `tour_variation = 0.15`,
`variation_operations = ["varied_end","repeated_notes","octave"]`,
`varied_end_keep = 0.5`. Conserver `second_ending` pour la vraie conclusion
de A2/B2 ; la variation ne change pas sa cible cadentielle.

**Boucle courte sans cadence** : une partie de 2 ou 4 mesures en `Open`/`Free`,
répétée par `sequence`, mélodie entrant au second tour, banques limitées aux
seuls rôles de poids positif :

```toml
[form]
sequence = [0, 0]
[[form.parts]]
name = "Boucle"
melody_center_offset = 0
roles = ["Open", "Free", "Free", "Free"]

[melody.params]
weight_role_run = 3
weight_role_arp = 0
weight_role_turn = 0

[arrangement.entries]
Melody = { tour = 2, part = 0, pass = 1 }
Chords = { tour = 1, part = 0, pass = 1 }
Bass = { tour = 1, part = 0, pass = 1 }
PercussionLow = { tour = 1, part = 0, pass = 1 }
PercussionHigh = { tour = 1, part = 0, pass = 1 }
```

### Opérations de motif et cadences

Un rôle fixe reste valide sans consommer de tirage structurel. Une alternative
pondérée se tire selon `resolve`, sur le `ChaCha8Rng` du générateur. Le matériel
de toutes les passes/tours nécessaires est construit au début de l'air ;
une invalidation émotionnelle ne retire aucune de ces décisions.

```toml
roles = [
  "Open",
  "Free",
  { SequenceOf = { bar = 1, shift = [{ degrees = -1, weight = 2 }, { degrees = 2, weight = 1 }] } },
  { VariedEndOf = 2 },
  { choose = [{ role = { CopyOf = 1 }, weight = 53 }, { role = "Free", weight = 47 }] },
  { CopyOf = 2 },
  "Free",
  { choose = [{ role = "CadTonic", weight = 22 }, { role = "CadHalf", weight = 8 }, { role = "CadOpen", weight = 70 }] },
]
second_ending = { choose = [{ role = "CadTonic", weight = 70 }, { role = "CadFifth", weight = 10 }, { role = "CadOther", weight = 20 }] }
```

- `CopyOf = k` : copie exacte de la mesure antérieure `k` (1-based).
- `SequenceOf` : mêmes attaques et durées, déplacement uniforme en degrés
  diatoniques, dans le mode publié. `shift` est une distribution de décalages
  `degrees` dans `-14..14`, sans degré zéro dans la notation des notes.
  Une octave commune garde le contour dans la tessiture quand il y tient ;
  le rendu replie par octave les notes qui dépassent, comme les autres cellules.
- `VariedEndOf = k` : préfixe de `k` selon `form.varied_end_keep`, puis fin
  d'une autre cellule de la banque, avec attaque au raccord et fin distincte
  même après réduction des octaves. Deux fins compatibles sont requises.
- `{ CopyOfPart = { part = "A", bar = 7 } }` : copie exacte entre parties.
  La partie référencée doit précéder la partie courante dans la déclaration
  **et** lors de sa première occurrence dans `sequence`.
- `Open`, `Free`, `CadHalf`, `CadFull` : rôles fixes historiques ;
  `CadFull` conserve les cellules d'origine, levées de reprise comprises.
- `CadTonic` : cellule cadentielle, dernier repos tonique tenu au moins
  `cadence_min_units` unités (défaut deux) ; harmonie dominante–tonique.
  Une tenue éligible après `cadence_search_from` est conservée, levée absorbée.
- `CadOpen` : cellule libre, fin non tonique dans le mode, sans accord imposé.
  `CadHalf` conserve sa fin sur 2, 5 ou 7 et sa dominante harmonique.
  Tous ces rôles cadentiels marquent une frontière de phrase pour le changement
  de mode émotionnel, même si la cadence est ouverte.
  Les qualités de I/V proviennent de `harmony.candidates`, y compris aux
  cadences ; elles ne sont pas remplacées par une qualité codée selon le mode.
- `CadFifth` : cellule de demi-cadence, dernière attaque ajustée à la quinte ;
  rythme de la cellule ou de la préférence positionnelle, dernier accord dominant.
- `CadOther` : cellule libre, dernière attaque hors tonique/quinte, toujours
  dans le mode ; aucun accord imposé. `CadFifth` et `CadOther` marquent aussi
  une frontière de phrase.

`[[form.parts]] second_ending` est optionnel et accepte exactement les mêmes
rôles fixes, opérations et distributions que `roles`. Il remplace **uniquement
la dernière mesure à la dernière passe de cette partie dans `sequence`** :
A2/B2 en AABB ; A3 si la partie A joue trois fois. Les autres passes utilisent
`roles` quand cette variante n'est pas sélectionnée. Avec les paramètres par
défaut, toutes les autres mesures restent identiques. La partie doit jouer au
moins deux fois. Les références restent strictement antérieures à la mesure
finale. Mélodie et harmonie sont précalculées ; `resolve` et les variations
peuvent diversifier les autres passes. L'outro `end_after_tour` reste prioritaire
et conclut sur la tonique, quel que soit `second_ending`.

Poids entiers non négatifs, somme positive dans `u32`, au plus 64 choix ou
décalages. Pas de choix imbriqué, de référence en avant ni d'auto-référence.
Toutes les branches, même de poids zéro, sont validées avant génération.

### Source des réglages distribués (P3-T1, contrat 11)

Les README de profil citent les fichiers DS-T9 du labo et leurs champs `real`.
Aux m.2/3/5, la référence locale est m.1 ; à m.6, m.2 ; à m.7, m.3.
Les poids `exact`, `sequence`, `varied_ending` viennent de
`real.transformations.<position>.rates`, arrondis sur 10 000 ; `Free` est le
complément. Les déplacements reprennent les effectifs de `real.sequence_shifts`
à la même position, sans inventer une distribution lorsqu'elle est absente.
À B:m7, la copie de A:m7 utilise son taux `exact` mesuré ; les opérations locales
se partagent le complément proportionnellement.

Aux m.4/8, `CadTonic` utilise `tonic_cadence_rate` dans
`real.phrases.rest_by_position`. Faute de classe harmonique finale dans DS-T9,
`CadHalf` emploie **comme proxy**, et non comme mesure de demi-cadence,
`max(0, long_final_rate - tonic_cadence_rate)` ; `CadOpen` reçoit le complément.
Les tirages par position sont indépendants : les taux conjoints m.5–6 et les
structures de paires ne sont pas imposés et doivent être remesurés au labo.
P3-T3 ajoute les préférences rythmiques par position décrites plus haut ;
les banques et les poids structurels restent inchangés.

### Secondes fins distribuées (P3-T2, contrat 12)

Les poids de `second_ending` sont les **effectifs**, sans arrondi, de la
dernière note écrite des fins finales A/B des parties de huit mesures :
tonique → `CadTonic`, quinte → `CadFifth`, autre → `CadOther`.
La commande de mesure et les quatorze relevés tradition × type se trouvent
dans [le rapport de phase](../specs/011-phase3-phrase/plan.md#p3-t2--fins-de-reprise).
Le parseur du labo complète les mesures partielles par la levée de reprise :
cette note ajoutée est exclue de la fin écrite mesurée. Les premières fins
conservent leurs réglages DS-T9 de **repos tonique**, qui ne sont pas une mesure
de dernière note écrite ; les deux indicateurs ne sont pas interchangeables.
Les taux réels finaux ne justifient pas une tonique obligatoire dans tous les
styles, et les petits effectifs sont signalés dans le rapport.

### Reproduire le contrat historique à rôles fixes

Omettre `second_ending`, `rhythm` et les variations, garder les autres valeurs
par défaut, puis remplacer `roles` dans chaque partie du reel par :

```toml
roles = ["Open", "Free", "Free", "CadHalf", { CopyOf = 1 }, { CopyOf = 2 }, "Free", "CadFull"]
```

Aucun drapeau de compatibilité ni second générateur : ces rôles ne consomment
aucun tirage structurel. Le test `profile_d2_main_note_reference` compare ce
profil à `profile_d2_fixed.json` et le profil distribué à `profile_d2_main.json`, tous
deux à la `TUNING_VERSION` courante (la référence « ancien main » a été régénérée en V7b :
le bloc de tirages de set avant l'air 1 décale le flux de toute la session). Régénération
explicite des deux références :
`HARMONIUM_UPDATE_GOLDENS=1 cargo test -p harmonium-cli profile_d2_main_note_reference`.

## Modèle fonctionnel par réécriture

Les règles se lisent dans le profil, sans état partagé ni RNG caché :

```toml
[harmony]
model = "rewrite_functional"
steps_per_chord = 8
max_recursion = 2
scale_guidance = true

[[harmony.rules]]
lhs = "V"
rhs = ["II", "V"]
weight = 0.85
min_valence = -1.0
max_valence = 1.0
category = "Preparation"
max_recursion = 0
```

Chaque règle remplace son symbole gauche par la séquence droite ; les symboles
en attente sont consommés aux pas harmoniques suivants. Poids finis positifs
ou nuls (au moins un positif), 1..64 symboles droits, valences ordonnées dans
`-1..1`. `max_recursion > 0` soumet une règle à la limite globale de
back-cycling. Catégories Steedman et multiplicateurs `Extended` : `Cadential` 0,8,
`Preparation` 1,5, `BackCycle` 1,3, `TritoneSubstitution` 1,2, `Deceptive` et
`ModalInterchange` 1,0. Sans règle applicable : mouvement fonctionnel par
quintes porté de Steedman.

Symboles : `I`, `II`, `III`, `IV`, `V`, `VI`, `VII`, `VofV`, `VofII`,
`VofIV`, `VofVI`, `VofIII`, `FlatII`, `FlatVI`, `FlatVII`. La réalisation
produit des accords absolus à partir de la tonique publiée. Chaque accord est
publié dans `MeasureSnapshot.chord_changes` avec son `start_step` et son
contexte ; les champs historiques de mesure décrivent le premier accord.
`scale_guidance = true` publie aussi les gammes Lydian Chromatic ; les
cellules restent la source du rythme et du contour mélodique, puis les notes
sont rapprochées des accords/gammes publiés et les tenues sont coupées aux
changements d'accord.

Le dossier `../tests/fixtures/profiles/jazz_test/` illustre un profil complet
**de test**, en 4/4 swing et AABA 32 mesures. Il n'est pas embarqué avec les
profils distribués ; un style jazz destiné à l'écoute demandera un corpus
et une phase de validation musicale.

L'accompagnement de ce modèle (basse et accords) passe par les mêmes voix que les autres
modèles : `hits`, `motifs`, `passing`, `walking` côté basse ; `voicing`, `comping`,
`spread` côté accords. Aucun second chemin pour les accords absolus : le moteur les lit
relatifs à la tonique.
