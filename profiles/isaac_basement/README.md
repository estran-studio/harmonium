# `isaac_basement` — essai Basement de *The Binding of Isaac*

Essai d'écoute de fin de phase 014 : peut-on approcher le Basement de
Ridiculon (*Diptera Sonata*, *Periculum*) avec les seuls mécanismes déjà en
place pour `nt_desert` ? Oui en données, sans ajout moteur. Références :
labo `references/isaac-basement/README.md` (G9-G0), dont les propositions
§7 sont reprises ici avec les corrections ci-dessous.

## Écriture

| Partie | Mesures | Harmonie (D mineur harmonique) | Basse | Chœur | Batterie |
|---|---:|---|---|---|---|
| A1 | 8 | pédale D | D1 réattaqué chaque mesure, relance C#2→D2 liée toutes les 2 mesures | tu | stomp |
| A2 | 4 | Dm Gm Dm Dm | montée F2 G2 A2 Bb2, une note par mesure, liée | tu | stomp |
| D | 4 | Bb A Gm A (V majeur, sensible C#) | descente Bb2 A2 G2 F2→E2, liée | plainte lente | stomp |
| Pont | 8 | G#m Gm Fm C#m, 2 mesures chacun (pont de *Periculum*) | fondamentale tenue + voisine chromatique | oui | tue |

Forme : A1 A2 D A2 D Pont A1' (40 mesures), deux tours. Les cellules du
chœur (`cells.toml`) sont composées, non transcrites : notes longues, plainte
Bb→A et sensible C#→D, aucun solo.

Écarts aux propositions du labo :

- Descente en accords **majeurs** Bb et A (VI et V du mineur harmonique) au
  lieu des Bbm/Am du détecteur, qui ne connaît que majeur/mineur ; à juger à
  l'oreille.
- Descente ramenée à 4 mesures (un motif de basse couvre 1 à 4 mesures) et
  reprise A2+D comme dans *Diptera Sonata*.
- Montée et descente à l'octave D2–Bb2 (leçon G7h) ; seule la pédale reste
  en D1.
- Valence haute : bascule vers D mixolydien (le validateur exige un mode
  clair parallèle à chaque mode sombre atteignable).

## Variantes par graine (phase 016, V1)

Les premières alternatives d'accords conservent les notes de l'essai
initial ; les secondes sont des propositions originales, pas des
transcriptions supplémentaires. Les deux ont le même poids (1) et
couvrent exactement la partie :

- **A1** reste une pédale D, sans alternative harmonique.
- **A2** : `Dm Dm Gm A` en réponse à `Dm Gm Dm Dm` (4 mesures).
- **D** : `Bb Gm Dm A` en réponse à `Bb A Gm A` (4 mesures) ; le V
  majeur conserve la sensible C#.
- **Pont** : `G#m Gm C#m Dm`, deux mesures par accord, en réponse au
  pont initial `G#m Gm Fm C#m` (8 mesures).

Chaque partie possède désormais deux motifs de basse pondérés (initial :
1, réponse : 2), au même registre et sur les mêmes contours : relance
C#→D retardée en A1, réattaque discrète dans la montée, F→E retardé dans
la descente, voisine inférieure au pont. Le stomp garde ses caisses
claires aux temps 2/3/4 ; aux densités 1 et 2, une réponse de poids 2
déplace les contretemps de grosse caisse face à l'initial de poids 1.
La densité 0 et les fills initiaux sont conservés ; le pont reste sans
batterie. Trois cellules de chœur originales ajoutent tierce tenue,
retombée vers Bb et soupir 2→1, toujours sans solo.

Les mêmes principes enrichissent `isaac_cellar` (ostinato chromatique,
A2 et pont), `isaac_burning` (C/C2 de huit mesures, riff F# phrygien,
demi-temps ; sa variante psychédélique est une zone de timbre Odin),
`isaac_downpour` (A/B/C de seize mesures, paliers chromatiques et basse
tenue/réarticulée).
Les instruments, tempos, formes et durées ne changent pas.

Le choix est pondéré et déterministe par graine, stable aux reprises de
chaque partie. Les banques d'accords, de batterie et de basse restent
indépendantes ; une nouvelle graine n'impose pas une nouvelle variante à
chaque couche. Ces propositions s'appuient sur les longueurs et grilles
des §3 et §6 des références du labo : pédale mineure harmonique,
voisines phrygiennes locales, stomp ou demi-temps, charleston parcimonieux.

## Alternatives du labo (R1, phase 019)

Propositions P2 (`harmonium_lab/references/p2-variete.md`) intégrées après
écoute du propriétaire (2026-11-02), `TUNING_VERSION` 25 :

- **`isaac_basement`** : deux boucles d'A2 extraites de *Diptera Sonata* et de
  *Periculum* (poids 1 chacune, à côté des deux écrites) ; un motif de basse
  d'A1 (poids 3 = total des écrits). Batterie inchangée : les patterns de P2
  coûtaient −17,9 contre *Periculum* (aucune portée par partie dans P2).
- **`isaac_burning`** : accords et basse de P1 (boucles générées de C et C2
  dans la gamme du mode, gabarits de basse avec relance vers l'accord suivant)
  et de P2 (boucle de C, trois motifs de basse) ; batterie : le groove de
  densité 1 de P2 seulement, sans les grooves par portée de partie de P1.
- **`isaac_downpour`** : une boucle de C et deux motifs de basse d'A de P2 ;
  batterie : le groove de densité 1 seulement.
- **`isaac_cellar`** : inchangé (rien d'exploitable dans les propositions).

Chaque ajout porte sa provenance en commentaire TOML (piste, minutage,
confiance). `isaac_burning` et `isaac_downpour` n'ont pas de README propre :
leur en-tête de `profile.toml` et ce paragraphe tiennent lieu. Mesures
avant/après : `specs/019-bibliotheque-accompagnements/r1-integration-p1-p2.md`.


## Habillage, arc, phrases (R2-jeux, phase 019)

- `session.toml` : groupes tempo (nominal 136-144, poids 8 ; 120-127 et 153-160, poids 1, règle
  d'interprétation dans les bornes 120..160, non transcrite), basse (33/34/38, avec permutations
  Odin des zones existantes, preset et `level_db` ensemble), nappe (91/89/19, idem), chœur (52/53),
  kit (128/0 et 128/16), voicing (close_triad/drop2/open), énergie (0,9-1,1), tags de patterns, arc.
  Aucun groupe de tonalité : D mineur harmonique est la seule tonalité attestée par les deux références
  du niveau (pédale D1, sensible C#) ; C# de Caves et F# de Fundamentum sont d'autres niveaux.
- `arc.toml` : trois étapes d'un chorus, `repeat_from = 1`, aucun `fall` ; arc nommé `orchestre`
  (kit par chorus) dans `session.toml`.
- `percussion.toml` : phrases de 4 et 8 mesures et relances remplaçant les trois cadences ; mêmes
  patterns, aucune nouvelle frappe de grosse caisse ni de caisse claire. Comping de nappe tenue
  (tags `tenue`, `souffle`) sans marche ni accord de jazz.

## Timbres

- SoundFont (GeneralUser GS) : basse doigts 33, nappe chœur 91, chœur
  Aahs 52, kit 128/0.
- Odin (`--features odin`, presets d'usine fournis par l'hôte) : Bass
  « Bass Crusher [RM] », Chords « Haunt [RM] ».

## Rendus

Hors dépôt, `harmonium_lab/corpus/isaac/ecoute/` :

```bash
./target/release/harmonium-cli --export --bars 80 --profile profiles/isaac_basement \
  --seed 1 --soundfont assets/GeneralUser-GS.sf2 --record-wav isaac_basement_generaluser.wav
# Odin : build --features odin, --odin-presets <dossier plat de .odin d'usine>
```
