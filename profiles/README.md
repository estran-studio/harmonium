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

| Profil | Tradition et corpus | Mètre, tempo | Instruments | Pieds | 2ᵉ voix |
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
| `isaac_burning_psyche` (essai) | variante d'`isaac_burning`, mêmes notes | 4/4, 140 | Odin Bass Low + Moonshade (« plus psychédélique ») ; SoundFont identique | oui | non |
| `isaac_downpour` (essai) | The Binding of Isaac, Downpour (Ridiculon) ; River of Despair | 4/4, 144 | chœur 52 (dès B), nappe chaude 89, basse doigts 33 tenue | oui (clairsemé, caisse au temps 3) | non |

Les « sept profils » cités plus bas désignent les sept premiers, réglés pendant les phases 009–012. Les trois
derniers (phase 013, R1) sont construits par le même chemin — références du labo, `mine-cells`, formules de
[Source des réglages distribués](#source-des-réglages-distribués-p3-t1-contrat-11) — par
`specs/013-moteur-dans-le-jeu/r1_profils.py` (`selftest` rejoue les formules sur quatre profils existants) ;
mesures et limites dans le plan 013, section R1.
Le onzième, `nt_desert` (phase 014, G7 à G7h), est le premier profil non trad : désert de *Nuclear Throne*, voir `profiles/nt_desert/README.md` et `specs/014-styles-de-jeu/g7f-nt-desert-riche.md` (écriture composée sans sifflement, comparaisons avant/après) ; G7e et G7h (legato, registre, longueurs et timbre lisibles) sont résumés dans le README du profil et le tableau d'état du plan 014.
Les profils `isaac_*` sont des essais d'écoute de fin de phase 014 : mêmes mécanismes que `nt_desert`, aucun ajout moteur, voir `profiles/isaac_basement/README.md` (les trois autres suivent le même schéma ; leur en-tête de `profile.toml` résume l'écriture).

|---|---|
| `[meta]` | `id` ASCII minuscule, `name`, `tradition` précise (région, époque), `tags`, `corpus` (référence au manifeste du labo et aux droits), `aliases` optionnels : noms courts pour `--style` (ex. `["gigue", "jig"]`), uniques parmi les profils distribués. Les tags ne servent pas à choisir un style. |
| `[files]` | `cells`, `feet`, `arrangement`, `dynamics`, `set` : chemins relatifs au dossier, sans `..`. Omettre pour écrire les sections inline. |
| `[meter]` | `time_signature = {numerator, denominator}`, `steps_per_bar`, `steps_per_unit` (unité des cellules), `steps_per_pulse`, `pulses_per_bar`, `default_swing_ratio`, `default_bpm`, `min_bpm`, `max_bpm`. Tempo à la noire ; pulse dansée indépendante. |
| `[form]` | `sequence` : indices 0-based des parties jouées (AABB = `[0,0,1,1]`). Chaque `[[form.parts]]` donne `name`, `melody_center_offset` en demi-tons et `roles`, un rôle fixe ou pondéré par mesure (voir « Règles de phrase »), plus `tracks_off` optionnel : pistes tues sur cette partie (`["Bass", "Heel", "Toe"]`, défaut : aucune), lues par la même porte que l'arrangement et la densité. |
| `[melody]` | `cell_units` unités par cellule (= une mesure), `strong_units`, `weak_units` (accents 0-based), `key_modes = [{key, mode, weight}]` (classe MIDI 0..11). `params` règle tessiture, registre, continuité, vélocités et poids de rôles. `cells` contient une liste `[[cells]]` avec `role`, `notes`, `weight` ; seuls les rôles effectivement employés par la forme (et les rôles libres de poids positif) sont exigés. Notes : degrés `1..14`, octave grave `1,`..`7,`, `-` pour prolonger. |
| `[harmony]` | `model = "trad_functional"`, `"jazz_functional"`, `"loop"` ou `"none"`, `steps_per_chord` diviseur de la mesure. Trad : `[[harmony.candidates]]` par mode, `chords = [{degree, quality}]` ; chaque mode atteignable (`ionian`, `mixolydian`, `dorian`, `aeolian`, `phrygian`, `harmonic_minor`) est requis avec I et V. V majeur en ionien, phrygien et mineur harmonique (sensible pour les deux derniers), mineur en mixolydien, dorien et éolien. Le II suit la seconde du mode (bII phrygien). Jazz : `[[harmony.rules]]`, `max_recursion` (0..16, défaut 0), `scale_guidance` (défaut false). Boucle : `parts` par nom de partie, `default_bars_per_chord` (défaut 1), voir ci-dessous. |
| `[accompaniment]` | Vote harmonique (`strong_steps`, poids, `tie_priority`), voix `[accompaniment.bass]` et `[accompaniment.chords]` décrites ci-dessous. |
| `[percussion]` | Notes MIDI, vélocités, gain, humanisation et `[[patterns]]` par densité 0, 1, 2 (`feet.toml` externe) : `weight` entier strictement positif (défaut 1), `hits = [{step, track="Heel"\|"Toe", delta_velocity}]` **ou** `euclid = {pulses, steps, rotation}` avec `track = "Heel"` ou `"Toe"` (défaut Heel). Plusieurs patterns d'une densité forment les alternatives pondérées ; voir « Batterie par graine ». Rotation entière, positive vers la droite ; `steps` égale la grille, `0 <= pulses <= steps`. Durée zéro = déclenchement de sample. |
| `[instruments]` | Programmes SoundFont 0-based : `melody_program`, `countermelody_program` optionnel, `chords_program`, `bass_program`, `percussion_program` et `percussion_bank`. |
| `[[odin.<Piste>.zones]]` | Optionnel. Zones d'émotion ordonnées d'une piste (`Melody`, `Chords`, `Bass`, `Countermelody`) : `when`, `preset`, `level_db` et `morph`. Dernière zone sans `when` obligatoire. Voir « Instrument Odin par piste ». |
| `[arrangement]` | `tours_per_tune`, `entries` par piste (`tour`, `part`, `pass`). `Heel` et `Toe` sont requis ; toute autre piste (`Melody` comprise) peut entrer plus tard ou être omise. Une piste entrée continue à jouer. Les entrées ne s'appliquent qu'au premier air de chaque set (voir « Forme longue »). |
| `[arrangement.intro]` / `[intro]` du fichier d'arrangement | Formes pondérées d'intro : rôle `percussion`, `tail` ou `vamp`, longueur en mesures, levée en unités. Intro complète au premier air de chaque set ; raccords des suivants dans `[set.transitions]`. |
| `[dynamics]` | Courbes par axe émotionnel : intensité, tempo, vélocités, densité, longueur de note et substitutions harmoniques ; seuils et frontière de changement de mode. Chaque mode atteignable (`key_modes`, `color`, `parallel_modes`) doit avoir sa bascule claire ↔ sombre : phrygien ↔ mixolydien, mineur harmonique ↔ ionien (majeur le plus proche, degrés altérés minimaux). Chaque courbe est une liste de points `[entrée, sortie]`, ordonnée ; extrémités constantes. Voir `dynamics.toml` pour les noms complets. |
| `[mix]` | Gain de bus, salle, niveaux/placement/envois par piste et EQ stéréo ; schéma et bornes ci-dessous. |
| `[set]` (`set.toml`) | Longueur, tonalités, raccords, clôture, rampe par air et mémoire signature ; section « Forme longue (set) » ci-dessous. |

Le chargeur vérifie les références, bornes, copies antérieures, métrique et banques
avant l'installation. Les erreurs nomment le chemin du champ fautif. Un modèle
d'harmonie non implémenté est refusé : aucune substitution silencieuse.

## Batterie par graine

Dans `feet.toml`, `[[patterns]]` correspond à `percussion.patterns` du profil
chargé. Chaque densité 0, 1, 2 doit être couverte ; plusieurs entrées d'une
même densité constituent une banque, non une superposition :

```toml
[[patterns]]
density = 1
weight = 2
hits = [
  { step = 0, track = "Heel" },
  { step = 8, track = "Toe" },
]

[[patterns]]
density = 1
weight = 1
hits = [
  { step = 0, track = "Heel" },
  { step = 6, track = "Heel", delta_velocity = -18 },
  { step = 8, track = "Toe" },
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
default_bars_per_chord = 1

[harmony.parts.A]
kind = "pedal"
degree = 1
# neighbor_semitones = 1

[harmony.parts.B]
kind = "chord_loop"
loops = [
  { weight = 2, chords = [
    { degree = 1, quality = "Minor", bars = 2 },
    { semitones = 1, quality = "Major", bars = 2 },
  ] },
  { weight = 1, chords = [
    { degree = 1, quality = "Minor", bars = 2 },
    { degree = 5, quality = "Major", bars = 2 },
  ] },
]
```

Chaque racine indique **exactement un** champ : `degree` numérique modal
`1..7` (1 = tonique), ou `semitones` chromatique `0..11` au-dessus de la
tonique. Les degrés suivent les intervalles du mode actif : le degré 2 vaut
un demi-ton en phrygien, deux en ionien ; les offsets chromatiques restent
fixes. Lors d'une bascule de couleur par `dynamics.color` ou `parallel_modes`,
les degrés sont recalculés dans le nouveau mode. Chaque accord exige une
qualité explicite `"Major"` ou `"Minor"`, qui ne change pas avec le mode.

`loops` est une liste non vide d'alternatives. Chaque `weight` est un entier
strictement positif (défaut 1) ; chaque `chords` est non vide. L'ancien
`chords` directement dans `harmony.parts.<nom>` est refusé, sans alias.
Une boucle est tirée par partie et par air, selon la graine et les poids ;
les reprises de la partie gardent ce choix. Une seule alternative ne
consomme aucun tirage.

`bars` est une durée positive en **mesures entières** ; omission :
`default_bars_per_chord` (défaut 1). La longueur totale de **chaque alternative**
doit être exactement celle de la partie ; développer explicitement les
répétitions d'une ancienne boucle courte. `steps_per_chord` reste requis et
divise la mesure : tous les créneaux d'une mesure portent le même accord.
Les reprises, secondes fins et variations de forme utilisent la même
position dans la boucle.
Pas de changement intra-mesure, d'accord slash ni de turnaround implicite :
une section distincte porte un éventuel raccord explicite.

`kind = "pedal"` n'accepte ni `quality` ni liste d'accords : c'est une vraie
fondamentale seule, sans tierce ni quinte, jouée sur `Chords` par les coups
habituels. `neighbor_semitones = 1..11` ajoute facultativement une seule note,
à cet intervalle chromatique au-dessus de la fondamentale. Pour une pédale
tenue, employer un coup `stroke = "block"`, `step = 0`, une `duration` égale
à `meter.steps_per_bar` et `tie = true` dans chaque motif sélectionnable de
`accompaniment.chords` (voir « Accords grattés »). Les notes contiguës de même
hauteur sont liées sans réattaque ; un changement de fondamentale ferme la
précédente. Éviter gate raccourci, respiration ou décalage sur `Chords` pour
préserver cette coïncidence. Le registre vient de `accompaniment.chords.center`
et le timbre de `instruments.chords_program` ; par exemple Saw Lead GM 82
s'écrit `chords_program = 81` (0-based), avec un registre grave.

Les triades gardent la polyphonie `chords.strings` et la conduite de voix
existantes ; une pédale seule reste une note, son voisin une dyade, sans
doublure automatique en octaves. `Bass` suit la fondamentale (ou la quinte
selon ses coups), sans lui imposer une tenue : la scie continue est sur
`Chords`, pas sur une basse percussive qui décroît.
Le voisin reste au-dessus de la pédale, sans renversement de la dyade.
Avec `voicing = "fixed_triad"`, la fenêtre `low..high` doit couvrir les
douze fondamentales et ce voisin (`high - low >= 11 + neighbor_semitones`).

Le contexte du premier accord est publié dans les champs habituels du
snapshot et dans `chord_changes` au pas 0. Les cellules mélodiques fournissent
toujours rythme et contour ; comme pour le jazz sans `scale_guidance`, leurs
porteuses sont rapprochées des hauteurs de la sonorité courante avant
interprétation et ornements. Une pédale pure contraint donc les porteuses à
sa seule classe de hauteur ; ses ornements restent ceux du mode actif.
La tension ne substitue pas de dominante aux accords déclarés. Les intros
vamp et l'outro tonique restent les raccords explicites de l'arrangement ;
la boucle décrit les mesures du corps de l'air.


## Instrument Odin par piste (option `odin`)

Chaque piste garde son **programme SoundFont**, toujours requis : c'est le rendu
de tout binaire compilé sans `odin` (1837, par défaut). Une piste peut en plus
déclarer de une à quatre zones Odin, en données (aucun `preset` direct sur la piste) :

```toml
[[odin.Bass.zones]]
when = { tension = [0.6, 1.0] }
preset = "basse_psychedelique"
level_db = -5.0
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
- **Choix à l'exécution.** Sans `odin` compilé, ou si l'hôte ne fournit pas le
  preset de la zone active, la piste joue son programme SoundFont. Avec `odin`
  et le preset présent, elle joue Odin. Aucun nom de style ni canal fixe : la clé est la
  piste (`Melody`, `Chords`, `Bass`, `Countermelody`) ; `Heel` et `Toe` restent
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
  installées à la prochaine frontière de partie (`bar_in_strain == 1`,
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

Heel et Toe suivent `instruments.percussion_bank`, un seul chemin
(`apply_profile_programs`) : banque `128` = kit GM sur le canal MIDI 9 (marche,
`nt_desert`) ; toute autre banque = canaux propres 3 et 4 sur cette banque. La
banque de pieds `harmonium_trad.sf2` est chargée en banque `1` : les profils
trad à podorythmie déclarent `percussion_bank = 1`. Un profil à kit déclare 128 :
charger la banque de pieds ne le détourne plus. Une banque désignée et non
chargée par l'hôte (ni kit GM en 128, ni banque de pieds en 1) n'a aucun repli :
les pieds sont silencieux et le chargement du profil journalise un avertissement
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
Le registre basse est MIDI ordonné ; le modèle jazz exige douze hauteurs
consécutives. `center` et `max_drift` pilotent la conduite de voix des triades.
`default_pattern` est utilisé lorsque les courbes `chop_intensity + density_chop`
valent zéro ; sinon l'index est `round(1 + offset)`, borné à la table.

### Motifs de basse

`bass.motifs` remplace `bass.hits` : les deux sources sont exclusives, puis
passent par la même réalisation de notes et de doublures. Aucun changement
pour un profil sans banque de motifs. Modèles compatibles :
`trad_functional` et `loop` ; les motifs suivent aussi les accords et pédales G2.

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

`reference = "chord_root"` (défaut) suit la fondamentale au pas d'attaque ;
`"tonic"` garde la tonique pour un bourdon. Une tenue franchissant une barre
est découpée en segments publiés avec `Articulation::Tied` sur les suivants :
le lecteur et l'export MIDI G5 prolongent la même hauteur sans nouvelle attaque.
Si la référence change, la nouvelle hauteur attaque normalement. `tie = true`
sur une note demande aussi une liaison avec la fin coïncidente précédente,
y compris à l'intérieur d'une mesure. Un timbre à sustain et aucune
interprétation raccourcissant le gate sont nécessaires pour une vraie pédale ;
la fixture NT utilise **Saw Lead, `bass_program = 81` (0-based)**.
`legato = true` (G7e) lie la note à la suivante par chevauchement au lecteur :
une seule attaque par hauteur, la fin de la note tenue recouvre l'attaque
nouvelle ; la première note d'un motif attaque normalement.
`doubling = ["octave", "tenth"]` suit les mêmes règles que les coups.

Banque : au plus 256 motifs, poids `u32` (défaut 1), somme positive sans
débordement ; poids nul = jamais tiré. `part = "A"` filtre par nom de partie ;
les motifs spécifiques éligibles ont priorité sur les motifs sans `part`.
Chaque partie doit disposer d'au moins un motif de poids positif, sauf celles
qui taisent la basse (`tracks_off`, voir `[form]`) : aucun motif ne peut les
nommer (refusé comme sans emploi), et le plan n'y tire aucune tranche.
Le tirage préfère les motifs entiers qui tiennent avant la fin de partie ;
sinon il tronque un motif à cette frontière, jamais au-delà.

La sélection est fixée une fois par air, dans un domaine ChaCha8 dérivé du
matériel écrit, sans déplacer les tirages des autres pistes. Les passes et tours
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
Les tirages réutilisent la graine de matériel de V3b, dans un domaine basse
distinct par mesure et attaque : ni mélodie, ni pieds, ni air suivant déplacé.
Les valeurs distribuées viennent du §7 de l'étude, **à valider à l'oreille** :
marche sans harmonie ; valse seulement dans le dernier quart de phrase ;
hornpipe plus rare (`none_weight = 10`), reel québécois `5`, autres reels/gigues `4`.

### Accords grattés

`chords.strings` (3..6, défaut 3) complète la triade par octaves en montant.
Chaque coup porte `stroke = "block" | "down" | "up"` (défaut `block`),
`probability` (0..1, défaut 1) et `tie = true | false` (défaut `false`). Zéro/1 :
aucun tirage de présence ; entre les deux : un tirage dérivé par attaque. `down`
va du grave à l'aigu ; `up` ne joue que les `up_strings` les plus aiguës, dans
l'ordre inverse. Dans le snapshot, l'ordre du Vec et des identifiants est
toujours l'ordre d'attaque. Un coup `tie = true` publie `articulation = "Tied"` :
si une hauteur sonne encore au moment exact où l'attaque suivante commence
(même canal, même hauteur, fin coïncidente), le lecteur et l'export MIDI la
prolongent sans nouvelle attaque ; sinon, attaque normale. Une nappe s'écrit
avec un coup `block` d'une mesure entière et `tie = true`.

```toml
[accompaniment.chords.strum]
down_gap_ms = 6.0
up_gap_ms = 4.0
up_strings = 3
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
| `up_strings` | 2..`strings`. |
| `anchor` | 0..1 : première corde / centre de l'étalement / dernière corde sur l'attaque V2. |
| `tempo_exponent` | 0..1 : écart × `(ref_bpm / tempo)^exposant` ; 0 : ms fixes, 1 : geste relatif au tempo. |
| `ref_bpm` | 1..1000, fini. |
| `max_spread_ratio` | 0..0,4 de la distance au coup suivant du motif, retour à la mesure suivante compris. |
| `velocity_slope`, `velocity_jitter` | 0..20 unités MIDI ; bonus décroissant de la première à la dernière corde, puis jitter individuel, **avant V2**. |

Le grattage est composé **après V2** en millièmes de pas. Pour conserver l'ordre,
le geste entier est réduit si nécessaire pour tenir dans ±490 ou le plafond
de voisinage ; aucune corde n'est clampée indépendamment des autres.
`strum` absent ou coup `block` : aucun décalage ni jitter V5, mais `strings`
reste la polyphonie écrite. Mécanique historique : trois sons, pas de `strum`,
probabilités 1, mêmes vélocités/motifs. V2 tire désormais articulation et jitter
**par attaque** sur chaque piste (contexte = hauteur la plus grave de l'attaque
précédente) ; gate/respiration communs, vélocités individuelles conservées.
La validation croisée jitter/écart de V5a est donc inutile : un jitter V2 commun
ne peut pas inverser les cordes. Les approches modales et gestes de cordes
sont refusés avec `jazz_functional`, dont le chemin conserve ses accords absolus.

Les ms sont converties avec le tempo nominal de la mesure ; swing et rampes
du lecteur réutilisent ensuite ce décalage dans la durée gelée de chaque pas,
comme V2 (pas d'horloge murale ni de second ordonnanceur audio).
Les six ensembles harmoniques gardent guitare nylon et basse acoustique :
écarts descendants 6/7/8/9/10 ms selon le profil, valeurs de départ de l'étude.
Reels : montées douces optionnelles à haute densité ; gigues : bas-haut-bas ;
valse et hornpipe : montée rare. La marche reste sans accompagnement tonal.

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
strings = 3
default_pattern = 0
patterns = [{ hits = [
  { step = 4, stroke = "block", duration = 2, velocity = 62 },
  { step = 12, stroke = "block", duration = 2, velocity = 58 },
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
strings = 3
default_pattern = 0
patterns = [{ hits = [
  { step = 4, stroke = "block", duration = 2, velocity = 56 },
  { step = 12, stroke = "block", duration = 2, velocity = 56 },
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
Heel = { level_db = -6.0, pan = -0.06, reverb_send = 0.10 }
Toe = { level_db = -6.0, pan = 0.06, reverb_send = 0.10 }
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
En GM, Heel et Toe partagent le canal 9 : **les réglages de Toe gagnent**,
également dans le MIDI. La banque dédiée les sépare sur 3/4. Les profils ont
les mêmes niveaux/envois aux deux pieds, mais leur léger écart de pan est
réduit à celui de Toe dans le repli GM.

Choix d'espace proposés (pas des mesures historiques) : cuisine amortie pour
le reel québécois, petite salle pour sa gigue, salle communautaire pour le reel
acadien, salle paroissiale plus large pour la valse ; salles de danse courtes
pour gigue/hornpipe britanniques, plein air quasi sec pour fifre-et-tambour.
La guitare est écartée, la basse centrale, les pieds resserrés ; la mélodie
reste près du centre et au-dessus de chaque accompagnement mesuré.
Les README individuels donnent les intentions ; les mesures avant/après et
leurs limites sont dans [Mesures V1](../specs/012-vie-realisme/plan.md#mesures-v1).

## Interprétation

`[performance.tracks.<piste>]` fonctionne pour `Melody`, `Chords`, `Bass`,
`Heel` et `Toe`. Les profils trad distribués n'ajoutent d'interprétation qu'à
la mélodie ; `nt_desert` y ajoute le micro-timing et l'accent des pieds
(jitter et règle `Accent` sur `Heel`/`Toe`, G7c) ; les durées écrites de
basse/guitare et les samples de pieds restent ceux des banques. **Section absente
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
choix de la tonalité d'un air au suivant. Le jeu garde la priorité : la couleur (tonalité, mode)
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

Le moteur réutilise le `TunePlan` résolu et la `FiddleLine` écrite, **pas une graine** :
un changement d'intensité ou de registre émotionnel ne réécrit pas la mélodie.
La couleur parallèle et la dynamique restent pilotées par le jeu ; harmonie,
accompagnement, intro et interprétation sont recalculés pour l'état courant.
La fenêtre de mémoire référence les airs déjà conservés pour la position et le seek :
pas de deuxième copie, et `invalidate_future` retire automatiquement les futurs retours.
Le rejeu depuis la mesure 1 reconstruit la même mémoire.




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
| `parts.tracks_off` | Pistes tues sur la partie, lues par la même porte que l'arrangement et la densité ; les tenues liées qui traversent la frontière se ferment à la barre et reprennent ensuite (repli sûr) ; la basse muette dispense de motif pour la partie | `["Heel", "Toe"]` en B | `["Bass", "Heel", "Toe"]` au pont de `nt_desert` |
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
Heel = { tour = 1, part = 0, pass = 1 }
Toe = { tour = 1, part = 0, pass = 1 }
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

## Modèle fonctionnel jazz

Les règles se lisent dans le profil, sans état partagé ni RNG caché :

```toml
[harmony]
model = "jazz_functional"
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
back-cycling. Catégories Steedman et multiplicateurs jazz : `Cadential` 0,8,
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
