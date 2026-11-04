# T1 — Style iReal → profil distribué, et usage du moteur par training

Date : 2026-11-04. Branche `wquintal/v2t1-preuve-livraison`, moteur de `main` à `22c3afb` (profils identiques à
`3fb01da`). **Document pour la session `harmonium_training`** : rien de training dans ce dépôt (constitution 4.5.0).
Aucun code ni profil n'est modifié par cette tâche. Les chiffres de variété citent
[`v2-variance.md`](v2-variance.md) (mesure V2, mêmes profils, même binaire).

Tout ce qui est écrit « mesuré » vient d'une exécution de cette tâche ; ce qui est lu dans le code ou les
comptes rendus est dit tel quel ; `[INFERENCE]` marque ce qui n'a pas été vérifié par exécution.

## 1. Le champ `style` d'une chanson iReal

`harmonium_practice/src/ireal.rs` (copie de lecture : `~/orca/harmonium-019/training-ireal.rs`, l. 622-650) lit le
champ **Style** d'une chanson iReal comme une **chaîne libre** (`ChordChart.style: Option<String>`, `None` si le champ
est vide) : formats `irealbook://titre=compositeur=style=tonalité=n=corps` et `irealb://titre=compositeur=…=style=
tonalité=corps`. Le code ne tire rien du style (ni tempo, ni feel) : il ne lit ni le tempo ni le nombre de
répétitions du fichier, et la mesure vient de la balise `T..` du corps (4/4 par défaut). La tonalité est lue avec son
mode (`-` final : éolien, sinon ionien), ce que `[fixed_tonality]` reprend tel quel (§ 4).

### Styles d'iReal Pro (consultés le 2026-11-04)

- Page officielle [Backing track styles](https://www.irealpro.com/learn/backing-track-styles) (mise à jour juillet
  2026) : **50 styles intégrés** en trois groupes, **Jazz (27), Latin (8), Pop (15)**, et trois packs achetés
  séparément ([Style Packs](https://www.irealpro.com/in-apps)) : **Blues (12), Brazilian (11), Salsa (17)**, soit 90.
  Les quatre styles de swing « simples » sont un seul style à quatre jeux de patterns choisis **par le tempo**
  (< 100 : Slow Swing ; 100-139 : Medium Swing ; 140-219 : Medium Up Swing ; ≥ 220 : Up Tempo Swing ; Up Tempo Swing
  choisi à la main garde ses patterns dès 210). Trois styles jazz changent de feel **par section** (Latin/Swing,
  Bossa/Swing, Swing Two/Four), un par chorus (Ballad Double Time Feel).
- Noms des 50 styles intégrés et liste « Styles (for display) » du champ de chanson :
  [ireal-musicxml, `doc/irealpro.md`](https://github.com/infojunkie/ireal-musicxml/blob/main/doc/irealpro.md)
  (document tiers, relu contre les comptes 27 / 8 / 15 de la page officielle : ils concordent). Noms du champ
  « display » : Afoxe, Afro, Baião, Ballad, Bossa Nova, Chacarera, Even 8ths, Funk, Latin, Medium Swing, Medium Up
  Swing, Pop, Pop Ballad, Reggae, RnB, Rock, Rock Pop, Samba, Samba Funk, Shuffle, Slow Bossa, Slow Swing, Up Tempo
  Swing, Waltz.
- Les noms des packs viennent de la page officielle des packs. Le [format de fichier
  officiel](https://www.irealpro.com/ireal-pro-file-format) décrit le champ comme « a short text description of the
  style used for sorting in the app. Medium Swing, Ballad, Pop, Rock… » : **n'importe quelle chaîne est possible**
  (les chansons du forum portent des variantes, des préfixes de groupe comme `Jazz-Medium Swing`, ou rien).

## 2. Table style iReal → profil distribué

Légende : **✓** le profil est la source du style ; **~** proche, une partie du feel manque (dit en remarque) ;
**✗** aucun profil, repli indiqué. Colonne « Plage » : `[min_bpm, max_bpm]` du profil (le tempo de l'utilisateur
passe par `SetBpm`, borné 40-300 par le moteur, pas par la plage du profil : hors plage le style sonne moins juste,
voir § 6). Profils : [`profiles/README.md`](../../profiles/README.md), tableau « Profils distribués ».

| Style iReal | Profil | Couv. | Plage (BPM) | Remarque |
|---|---|:-:|---|---|
| Medium Swing | `wjazzd_swing` | ✓ | 90–285 | swing selon le tempo (JTD), contrebasse en marche |
| Medium Up Swing, Medium Up Swing 2 | `wjazzd_swing` | ✓ | 90–285 | idem ; pas de « 2 » distinct |
| Up Tempo Swing, Up Tempo Swing 2, Double Time Swing | `wjazzd_swing` | ✓ | 90–285 | plage jusqu'à 285 ; `rapide` est l'arc nommé des tempos hauts |
| Slow Swing | `wjazzd_swing` (≥ 90) ou `wjazzd_ballad` (< 88) | ~ | 90–285 / 44–84 | iReal change de patterns à 100 ; ici le seuil est celui des plages de profil |
| Guitar Trio | `wjazzd_swing` | ~ | 90–285 | le comping à la guitare (programme 26) est une variante de session tirée, **pas imposable** (§ 6) |
| Swing Two/Four | `wjazzd_swing` | ~ | 90–285 | deux temps seulement dans l'arc (étape d'exposition) ; pas d'alternance par section |
| Blue Note, Doo Doo Cats | `wjazzd_swing` | ~ | 90–285 | `[INFERENCE]` : feel non documenté publiquement par iReal ; repli swing |
| Ballad, Ballad Swing, Ballad Melodic | `wjazzd_ballad` | ✓ | 44–84 | balais ou baguettes tirés par graine, deux temps puis marche |
| Ballad Even | `wjazzd_ballad` + `SetSwing(0.5)` | ~ | 44–84 | croches droites : `SetSwing` désactive les courbes par piste (§ 4.3) |
| Ballad Double Time Feel | `wjazzd_ballad` | ~ | 44–84 | pas de doublement de feel au chorus du milieu |
| Long Notes | `wjazzd_ballad` | ~ | 44–84 | pas de notes tenues d'orchestre ; balais |
| Pop Ballad | `wjazzd_ballad` (jazz) ou `gmd_pop` à ≤ 90 | ~ | 44–84 / 83–142 | le style pop lent n'a pas de source dédiée |
| Waltz, Jazz Waltz, tout 3/4 | `gmd_jazz_waltz` | ✓ | 100–230 | seul profil en 3/4 (12 pas par mesure) |
| Bossa Nova, Bossa Acoustic, Bossa Electric, Slow Bossa | `gmd_latin_brazilian_bossa` | ✓ | 104–180 | Electric : le programme est tiré par graine, pas imposable ; Slow Bossa < 104 : hors plage |
| Bossa Ballad, Bossa Cha, Bossa/Swing | `gmd_latin_brazilian_bossa` | ~ | 104–180 | feel de section non reproduit |
| Samba, Samba Fast, Samba Funk | `gmd_latin_brazilian_samba` | ✓ / ~ | 85–120 | Fast (> 120) et Funk : hors plage ou sans source dédiée |
| Samba Brush, Samba Percussion, Partido Alto, Maracatu, Afoxé | `gmd_latin_brazilian_samba` | ~ | 85–120 | percussions de samba seulement (surdo, tamborim, agogô, pandeiro) |
| Baião | `gmd_latin_brazilian_baiao` | ✓ | 95–110 | zabumba, triangle, agogô, accordéon |
| Latin, Son Montuno 2-3, Son Montuno 3-2, Mambo, Guaracha, Timba, Songo | `gmd_afrocuban` | ~ | 105–128 | clave son 3-2 / 2-3 et montuno en règles de métier ; **la clave n'est pas choisie par l'hôte** ; un mambo rapide dépasse la plage |
| Rumba Guaguancó, Cha-Cha-Chá, Bolero, Danzón, Guajira, Comparsa, Mozambique, Merengue, Bomba, Plena, Tango | `gmd_afrocuban` (repli) | ✗ | 105–128 | pas de source ni de feel propre (GMD : 1 fichier `afrocuban` droit, 1 `rhumba`) ; repli le moins faux : 4/4 latin droit |
| Latin/Swing | `gmd_afrocuban` | ✗ | 105–128 | alternance latin/swing par section non reproduite ; option : `SetStyleProfile` à la frontière de section (§ 6) |
| Even 8ths, Even 8ths Open, Even 16ths | `gmd_funk` | ~ | 80–130 | croches droites ; GMD `funk` ; pas de variante « Open » |
| Funk, Glam Funk, Virtual Funk, Funky | `gmd_funk` | ✓ / ~ | 80–130 | Glam et Virtual : sans source dédiée |
| Second Line, Nola | `gmd_neworleans_funk_secondline` | ✓ | 82–124 | piano, tuba, caisse claire de second line |
| Shuffle, Chicago Shuffle, Texas Rock, Rock 12/8, Slo-Mo, Mo-Slo, tout 12/8 | `gmd_blues_shuffle` | ✓ / ~ | 72–134 | ternaire sur 12 pas ; un 12/8 écrit en 4 temps de 3 croches tombe sur 12 pas × 3 |
| Rock, Rock Pop, Slow Rock | `gmd_rock` | ✓ / ~ | 90–130 | guitare saturée, accords en triades ; pas d'accord de puissance |
| Pop, Smooth, Country, Bluegrass | `gmd_pop` | ~ | 83–142 | pas de profil country (GMD `country` non transformé en profil), la valse country passe par `gmd_jazz_waltz` |
| Soul, RnB, Stax, Muddy | `gmd_soul` | ✓ / ~ | 80–105 | GMD soul/motown ; les styles de blues n'ont pas de source propre |
| Reggae | `gmd_reggae` | ✓ | 83–121 | GMD reggae : 2 fichiers (« juste ») |
| Disco, House | `gmd_dance_disco` | ✓ / ~ | 120–137 | House : même quatre-temps droit, sans source dédiée |
| Afro, Afro 12/8, Afro 6/8 | `gmd_blues_shuffle` (repli ternaire) | ✗ | 72–134 | le profil afro (`gmd_afrobeat`) est un afrobeat en 4/4 droit, pas un 12/8 ; à garder pour « Afrobeat » libre |
| Chacarera | aucun | ✗ | — | GMD `latin/chacarera` (1 fichier) non transformé en profil ; repli : mètre |
| Gypsy Jazz, Trad Jazz, New Orleans Swing | `wjazzd_swing` (repli) | ✗ | 90–285 | pas de source (pompe manouche, batterie trad) ; sonne swing moderne |
| Gospel, Lucille, Shout, Flat Tire, Hip-hop… | selon le mot (§ 3) | ✗ | — | styles de packs sans profil ; `gmd_hiphop` (70–100) est la seule source hip-hop, absente d'iReal |

`gmd_hiphop` et `gmd_afrobeat` n'ont **aucun style iReal** (hip-hop et afrobeat n'y figurent pas) : training peut les
proposer comme styles libres, pas comme traduction d'un champ `style`. Les profils `reel_*`, `gigue_*`, `hornpipe_*`, `marche_*`, `valse_*` ne sont **pas** des profils de
grille : leur harmonie est `melody_functional` (ou `none`), le moteur n'y pose pas de grille utilisateur (mesure V2 :
« sans objet »). Les profils de jeu (`nt_desert`, `isaac_*`) sont `loop` mais portent des motifs de basse par partie
nommée : la copie sous grille est refusée au chargement (`accompaniment.bass.motifs[0].part: partie inconnue dans
form.parts`, mesuré), ils ne sont pas candidats.

## 3. Choisir le profil (règle proposée)

1. **Normaliser** la chaîne : minuscules, sans accent, séparateurs et ponctuation → espace, préfixe de groupe retiré
   (`jazz-`, `latin-`, `pop-`, `blues-`, `brazilian-`, `salsa-`, et `latin-brazil:` des noms de groupes), suffixe
   ` 2` conservé s'il est dans la table.
2. **Nom exact** dans la table du § 2.
3. **Mot-clé** (ordre) : `swing`, `jazz` → `wjazzd_swing` ; `ballad` → `wjazzd_ballad` ; `bossa` ; `samba` ; `baiao` ;
   `waltz` → `gmd_jazz_waltz` ; `funk` ; `shuffle`, `blues` → `gmd_blues_shuffle` ; `rock` ; `soul`, `rnb` ; `reggae` ;
   `disco`, `house` ; `latin`, `salsa`, `mambo`, `son` → `gmd_afrocuban` ; `pop` → `gmd_pop`.
4. **Repli par mètre**, le champ `style` étant vide ou inconnu : **3/4 → `gmd_jazz_waltz`** ; **12/8 (ou 4 temps de
   3 croches) → `gmd_blues_shuffle`** ; **4/4 → `wjazzd_swing`** (le grille-type d'iReal est une chanson de jazz).
   Autre mètre (2/4, 5/4, 6/8, 7/4…) : **aucun profil de grille** (seuls 4/4, 3/4 et le 12 pas ternaire existent) ;
   training refuse le style ou joue la grille en 4/4 sans l'avouer, à décider côté produit.
5. **Le mètre l'emporte sur le style** : une grille en 3/4 ne peut aller que sur `gmd_jazz_waltz`, une grille en 4/4
   jamais sur lui. Une grille de 12/8 sur un profil 4/4 à 16 pas n'a pas de sens.

## 4. Construire le profil de l'hôte et le piloter

### 4.1 Le profil, à partir d'un profil distribué

Le profil distribué est un **modèle** : training le copie et remplace **deux sections**, la forme et l'harmonie, par la
grille de l'utilisateur ; tout le reste (accompagnement, percussions, instruments, `[session]`, `[arc]`, swing,
dynamique) est gardé tel quel.

```rust
let mut p = StyleProfile::distributed().iter().find(|p| p.meta.id == "gmd_funk").unwrap().clone();
p.form.parts = vec![/* une partie « Grille » : roles = [Open, Free × (N-1)], phrase_ends = [N], melody_center_offset = 0 */];
p.form.sequence = vec![0];                                   // un chorus = une passe de la grille
p.harmony.model = "loop".into();
p.harmony.steps_per_chord = /* inchangé : lu dans le profil, voir ci-dessous */;
p.harmony.parts = BTreeMap::from([("Grille".into(), HarmonyPart::ChordLoop { loops: vec![ChordLoop { weight: 1, chords }], generated: vec![] })]);
p.fixed_tonality = Some(FixedTonality { key: tonic_pc, mode });   // §4.2
p.arrangement.tours_per_tune = n_chorus;                          // §5
p.validate()?;                                               // mêmes contrôles que le chargement
engine.send(EngineCommand::SetStyleProfile(Box::new(p)));
```

Équivalent en TOML, rejouable hors ligne avec `harmonium-cli --export --profile <dossier>` : voir
`harmonium_lab/src/harmonium_lab/usergrid.py` (la transformation que V2 a mesurée, sur 17 profils, **tous**
chargés et exportés : [`v2-variance.md`](v2-variance.md)). Détails qui comptent, **mesurés** :

- **Accords** : `LoopChord { semitones: 0..=11 (ou degree), quality, bass: Option<u8>, steps }`, tous **relatifs à la
  tonique** ; `quality` est le texte de `Chord::from_name` (`7`, `m7`, `maj7`, `6`, `m7b5`, `7b9`, `7alt`…) ;
  `bass` pour les accords slash. Chaque `steps` est un **multiple de `harmony.steps_per_chord`** et la somme vaut
  **`N × meter.steps_per_bar`** (sinon refus au chargement).
- **Pas par mesure** (lus dans le profil, pas dans la grille) : 16 (profils 4/4), **12** (`gmd_jazz_waltz`,
  `gmd_blues_shuffle`), `steps_per_chord` 4 (3 pour `gmd_blues_shuffle`). Un accord sur un temps de la grille iReal
  tombe sur un multiple de `steps_per_chord` pour les 17 profils de grille ; **un profil dont `steps_per_chord`
  est une mesure entière** (`isaac_*`, `nt_desert`) ne tolère pas deux accords par mesure : c'est une raison de plus
  de les exclure.
- **Partie unique et nommée** : le moteur exige une entrée `harmony.parts` par partie de `form.parts` ; les profils
  de grille acceptent la partie « Grille » (aucun motif de basse ne la référence).
- **Longueur de grille minimale** : les breaks de l'arc (`[arc]` ou arcs nommés) portent un `start` en mesures du
  chorus, **refusé au chargement s'il dépasse la longueur de la grille**. Seuil lu dans l'arc ; refus constaté à 8 mesures
  pour les sept profils des deux premières lignes ci-dessous, acceptation à 16 (24 pour la deuxième ligne), jamais essayé à la longueur
  exacte :

| Profil | Grille minimale (mesures) | Cause |
|---|---:|---|
| `gmd_afrocuban`, `gmd_funk`, `gmd_latin_brazilian_{baiao,bossa,samba}` | **9** | `breaks = [{ start = 9, … }]` |
| `gmd_jazz_waltz`, `wjazzd_ballad` | **24** | `breaks = [{ start = 24, … }]` (AABA de 32) |
| `gmd_blues_shuffle`, `gmd_neworleans_funk_secondline` | 7 | un blues de 12 passe |
| les autres profils de grille | 4 (`wjazzd_swing`, `start = -4`) ou 2 | départs négatifs depuis la fin |

  Une grille plus courte que ce seuil **oblige training à retirer ou borner les `breaks`** de la copie (parcours de
  `arc.stages[].variants[].breaks` et des arcs de `session.arcs`) ; sinon `SetStyleProfile` échoue. Manque de schéma
  signalé à A7 (un break hors chorus devrait être ignoré, pas refusé).
- **Tonalité** : `fixed_tonality` ne demande aucune mélodie. Les accords relatifs se transposent.

### 4.2 Ce que l'hôte impose, ce que la graine tire

| Réglage | Imposé par l'hôte | Sinon, tiré par la graine |
|---|---|---|
| **Tonalité** | `p.fixed_tonality = Some({ key: pc de la tonique d'iReal, mode: Ionian ou Aeolian })` ; la variante de session `tonalite` est **ignorée** (même hors des modes atteignables) ; pour transposer : reconstruire le profil et `SetStyleProfile` (sans seek ni reset de graine) | tonalité de la variante (un seul air) |
| **Tempo** | `EngineCommand::SetBpm(bpm)` : exactement la valeur de l'hôte sur chaque mesure, la courbe de swing la lit ; `ResetBpm` rend le tempo à la graine | tempo de la variante : plage du profil, 1 à 4 groupes `tempo` |
| **Swing** | **ne pas envoyer `SetSwing`** pour un profil à courbes (`wjazzd_*`, `gmd_jazz_waltz`) : le swing suit le tempo (JTD) ; `SetSwing(r)` impose `r` à toutes les pistes et **ignore les courbes** jusqu'au prochain profil | rapport de la variante / courbe |
| **Habillage** | rien : programmes de comping, de basse et de kit, **famille de voicings**, registre, énergie, **arc nommé**, tags de comping et de batterie | oui, **un tirage par graine** (A2) |
| **Nombre de chorus** | `p.arrangement.tours_per_tune` (§ 5) | valeur du profil (3 à 5) |
| **Mélodie** | `EnableMelody(bool)` (§ 4.4) | — |

Une variante de session **ne s'impose pas** : un hôte qui ne veut pas d'un habillage (« toujours des balais »)
installe une copie du profil dont `[session]` ne garde que la variante voulue. Une commande d'imposition est un
manque relevé pour A7 (voir § 6).

**Cas de `wjazzd_swing` (swing, plage du profil 90 à 285 BPM, session 100 à 265)** : le groupe `rythme` tire **ensemble** le tempo, le kit (balais au tempo lent,
jamais au-delà de 142 BPM) et l'arc (lent 100-118, medium 122-142, medium-up 148-186, up 192-225, très rapide 232-265).
`SetBpm` n'impose **pas** la variante : une graine lente jouée à 230 BPM garde ses balais et son exposition à deux temps
(B1, `b1-swing.md`, « Swing rapide : décision pour T1 »). Usage : lire `MeasureSnapshot.session` (groupe `rythme`) de la
première mesure et **choisir la graine** dont la variante est à l'échelle du tempo voulu (`medium_*`, `medium_up_*`,
`up_rapide`, `tres_rapide`) ; ne pas imposer `SetSwing`. Sous cette règle, le style iReal (Medium / Medium Up / Up Tempo
Swing) se traduit en une **plage de tempo** et en un **choix de graine**, pas en un profil de plus.

**Tout `SetStyleProfile` réinitialise tempo et swing** : renvoyer `SetBpm` (et `SetSwing` si l'utilisateur l'a
choisi) après chaque installation de profil, avec les mutes et gains (`install_profile` de training le fait déjà).

### 4.3 Habillage et variété sous grille (mesures V2)

Sous grille utilisateur, tonalité et tempo sont imposés : la graine ne tire plus que les programmes, les voicings,
le swing (si non imposé), le kit, l'arc et la densité. **Mesuré** sur 16 graines × 4 chorus, grille de la colonne « Grille »
(détail et seuils : [`v2-variance.md`](v2-variance.md)) :

| Profil | Grille (mesures) | Habillages distincts hors tempo et tonalité | Programmes comping / basse / kit | Comping changé d'un chorus au suivant | Basse | Batterie | Paires de chorus identiques (accompagnement) |
|---|---:|---:|---|---:|---:|---:|---:|
| `gmd_afrobeat` | 8 | 15 / 16 | 3 / 3 / 3 | 86 % | 5 % | 100 % | 0 % |
| `gmd_afrocuban` | 16 | 16 / 16 | 3 / 3 / 5 | 87 % | 46 % | 100 % | 0 % |
| `gmd_blues_shuffle` | 8 | 13 / 16 | 6 / 4 / 5 | 85 % | 10 % | 97 % | 0 % |
| `gmd_dance_disco` | 8 | 15 / 16 | 3 / 4 / 4 | 82 % | 5 % | 71 % | 0 % |
| `gmd_funk` | 16 | 15 / 16 | 4 / 4 / 4 | 91 % | 9 % | 77 % | 0 % |
| `gmd_hiphop` | 8 | 13 / 16 | 3 / 3 / 4 | 83 % | 4 % | 88 % | 0 % |
| `gmd_jazz_waltz` | 24 | 13 / 16 | 4 / 3 / 2 | 89 % | 99 % | 88 % | 0 % |
| `gmd_latin_brazilian_baiao` | 16 | 16 / 16 | 3 / 2 / 4 | 85 % | 47 % | 83 % | 0 % |
| `gmd_latin_brazilian_bossa` | 16 | 16 / 16 | 4 / 3 / 4 | 85 % | 74 % | 100 % | 0 % |
| `gmd_latin_brazilian_samba` | 16 | 12 / 16 | 4 / 3 / 3 | 84 % | 20 % | 100 % | 0 % |
| `gmd_neworleans_funk_secondline` | 8 | 14 / 16 | 3 / 3 / 3 | 84 % | 39 % | 75 % | 0 % |
| `gmd_pop` | 8 | 15 / 16 | 4 / 4 / 4 | 82 % | 5 % | 90 % | 0 % |
| `gmd_reggae` | 8 | 12 / 16 | 4 / 3 / 3 | 85 % | 5 % | 91 % | 0 % |
| `gmd_rock` | 8 | 8 / 16 | 3 / 2 / 3 | 82 % | 5 % | 92 % | 0 % |
| `gmd_soul` | 8 | 13 / 16 | 4 / 3 / 3 | 85 % | 5 % | 75 % | 0 % |
| `wjazzd_ballad` | 24 | 11 / 16 | 3 / 3 / 2 | 89 % | 89 % | 80 % | 0 % |
| `wjazzd_swing` | 8 | 14 / 16 | 3 / 2 / 2 | 83 % | 99 % | 83 % | 0 % |

Lecture : **aucun profil ne boucle** (0 % de paires de chorus identiques), le comping et la batterie changent à chaque
chorus (75 à 100 %), les graines donnent **8 à 16 habillages distincts** même quand l'hôte impose tempo et tonalité
(`gmd_rock` 8 : trois comping, deux basses, trois kits ; `wjazzd_ballad` 11). **La basse des profils à riff change peu d'un
chorus à l'autre** (4 à 10 % pour 9 profils GMD) : entre graines elle diffère, d'un chorus à l'autre non. Les profils à
basse en marche (`wjazzd_swing`, `gmd_jazz_waltz`, `wjazzd_ballad`) changent 89 à 99 % de leurs mesures.

### 4.4 Mélodie, difficulté, question-réponse

- **Mélodie on/off : `EngineCommand::EnableMelody(bool)`**. Seul `wjazzd_swing` **a** une piste `Melody` parmi les
  profils de grille (E12 : saxophone ténor, 213 cellules synthétisées depuis WJazzD, six rôles, phrases de quatre
  mesures, variation de chorus). Les 16 autres profils de grille n'ont pas de `Melody` dans `[entries]` : la
  commande est sans effet, et la copie ne doit pas en réclamer. Pour la lecture à vue, `EnableMelody(true)` sur
  `wjazzd_swing` ; pour l'impro, `EnableMelody(false)` (la mélodie composée disparaît du snapshot).
- **Difficulté (E10)** : en données, sur la copie : `p.melody.difficulty = Some(MelodyDifficulty { max_leap_semitones,
  max_attacks_per_bar, min_note_units, max_cell_ambitus_semitones, out_of_scale, ornaments, chord_fit })`. Installer le
  profil **avant** `SetSeed`/`NewMelody` : `SetStyleProfile` en cours d'air laisse l'air courant tel quel. Pour que le
  snapshot respecte les bornes, mettre `ornaments = false` et `chord_fit = false`. La levée d'intro est retirée.
  Aucun profil distribué n'a de filtre : c'est à training de fixer les quatre niveaux. Effet seulement là où il y a
  une mélodie (`wjazzd_swing`).
- **Question-réponse (E11)** : `p.arrangement.alternation.insert(TrackId::Melody, Alternation { period, play, phase })` (`period` 2..=32).
  Agit sur les mesures encore à composer (`SetStyleProfile` suffit) ; les mesures muettes n'ont aucune note `Melody`
  dans le snapshot, les autres pistes ne changent pas. Une piste absente de `[entries]` est refusée.
- `EnableMelody`, les entrées progressives et l'alternance sont des **plafonds** qui se cumulent.

## 5. Chorus, arc et retombée

- Un chorus = une passe de la grille. Le nombre de chorus est `arrangement.tours_per_tune` (1..=16 ; défaut du profil :
  `wjazzd_ballad` 3, `wjazzd_swing` 4, `gmd_blues_shuffle`/`gmd_funk`/latins 5, autres 4). Le `[set]` des profils de
  grille est **un air** (`length = [{ tunes = 1 }]`) : l'arc est donc de **fin connue**.
- L'arc (`[arc]`, ou arc nommé tiré par la graine dans `[session]`, groupe `arc`) fait varier d'un chorus à l'autre :
  intensité, densité, feel de basse (deux temps → marche), tags de comping et de batterie, breaks et stop-time. Les
  profils de grille ont **2 à 4 étapes** couvrant 2 à 4 chorus et `repeat_from = 1` : au-delà, l'arc reprend à la
  deuxième étape.
- **Retombée (`[[fall]]`)** : variante du **dernier** chorus, jouée seulement si le nombre de chorus est connu **et
  d'au moins 2**. Avec `tours_per_tune = 1` aucune retombée ni arc. Tous les profils de grille en ont une.
- **Clôture** : la dernière mesure du dernier chorus est remplacée par la tonique et la grosse caisse (V0), puis
  le moteur commence un nouvel air (nouvelle intro) : training arrête ou boucle à la fin de `tours_per_tune`.
- Pour un exercice **sans fin** (nombre de chorus inconnu), retirer `[set]` de la copie : « un set sans fin », l'arc se tient
  sur sa dernière étape ou reprend à `repeat_from`, et **aucune retombée n'est jouée** (`profiles/README.md`, « Forme longue »).

## 6. Limites connues (au 2026-11-04)

| Limite | Détail |
|---|---|
| **`SetLoop` : 16 mesures au plus** | `MEASURE_WINDOW_CAP = 16` (`harmonium_host/src/playback.rs:29`, `check_loop_len` : `end − start < 16`) : impossible de boucler un chorus AABA de 32 mesures, ni même 17 mesures ; la forme se répète de toute façon, la limite ne gêne que « boucler tout le chorus » |
| **`SetStyleProfile` réinitialise tempo et swing** | à renvoyer ; documenté, pas un défaut |
| **Breaks hors grille** | refusés au chargement (§ 4.1) : borner ou retirer côté training |
| **Départ et SoundFont** | `LiveHost::start_paused` ≈ 4,5 s en release avec la SoundFont MuseScore `.sf3` (mesuré par training) ; **non mesuré ici** avec GeneralUser GS (`assets/GeneralUser-GS.sf2`, 32 Mo, `.sf2`) : une banque `.sf2` décompressée est le levier indiqué par training. Training sonne sans Odin (D1) : tous les profils de grille sont **sans Odin** |
| **E14 (tempo audible rampé après un `Seek`)** | aucune trace de correction dans ce dépôt (`git log --grep E14`, `specs/`) : le contournement `SetAudibleTempo { bpm, ramp_pulses: 0 }` reste nécessaire |
| **Imposer une variante** | « les balais quelle que soit la graine » : pas de commande ; contournement : copie du profil à un seul variant par groupe ; relevé pour A7 |
| **Habillage et tonalité** | la tonalité d'une variante ne vaut que pour le premier air ; sous `[fixed_tonality]` elle est ignorée |
| **Meters** | seuls 4/4, 3/4 et le 12 pas ternaire ont un profil de grille |
| **Styles à feel de section** | Latin/Swing, Bossa/Swing, Swing Two/Four, Ballad Double Time Feel : un seul feel par profil. Option : `SetStyleProfile` à la frontière de section (appliqué à la mesure suivante, sans reset de graine), en renvoyant `SetBpm`/`SetSwing` ; **non essayé ici** |

### Ce qu'E13 permet de retirer du contournement de training

E13 est fusionnée (2026-11-03, [`e13-mesures-perimees.md`](e13-mesures-perimees.md)) : `DropFrom { bar, epoch }`, rejet
des boîtes d'un epoch antérieur à l'adoption, bit « non chargé » de l'atomique de transport, réserve de file, re-staging
immédiat après `Seek`/`SetSeed`/`NewMelody`/`Reset`. Test de contrat côté moteur : 13/27 échecs sur `main` avant,
0/28 après (`harmonium_host/tests/stale_measures_tests.rs`). Dans `harmonium_practice/src/engine.rs` :

1. **retirer `drain_playback_window`** entier : `SetOutputMute(true)`, `ClearLoop`, les `SeekPlayhead` de stationnement
   (`u32::MAX - …`) et leurs attentes de callback, le `SeekPlayhead(return_bar)` qui sert de barrière, puis
   `SetOutputMute(false)` ;
2. **dans `prepare_playback`** : le cycle `resume` → attente de `transport_position().bar` (2 s) → `pause`, la
   ré-application de `SetLoop` après `ClearLoop` (la boucle n'est plus touchée), le drapeau `seek_bar` ; **il reste le
   positionnement de la tête** (`SeekPlayhead(return_bar)` ou `Seek(return_bar)`) ;
3. **`rebuild_current_bar`** (`SetSeed` + `Seek(bar)`) pour `SetBpm`/`SetSwing` à l'arrêt : inutile quand la tête est sur
   une mesure non chargée (départ de piste, après `Seek`) ; **à garder** pour qu'un changement de tempo à l'arrêt
   **au milieu d'une mesure** s'applique au reste de cette mesure (la mesure chargée est audible, toujours gardée) ;
4. **`session_start` / `new_piece`** : plus de purge avant de rejouer, `SetSeed` + `Seek(1)` suffisent ; **à garder** :
   `install_profile` qui renvoie `SetBpm`, `SetSwing`, mutes et gains après `SetStyleProfile` ;
5. ne plus compter sur `drain_playback_window` pour qu'« aucune composition ne tourne » ; un `SeekPlayhead` très loin
   reste à éviter pendant un `pump` (le compositeur composerait jusqu'à la tête).

Ce qu'E13 **ne retire pas** : le contournement d'E14 (`SetAudibleTempo`), la limite de `SetLoop`, le délai de
démarrage.
