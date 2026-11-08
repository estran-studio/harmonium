# Q0 — corrections rapides des styles (phase 020, vague 1)

**Statut** : livrée le 2026-11-07 sur la branche `wquintal/q0-corrections-rapides` (Mac, reprise du commit `wip(020/Q0)` laissé par `ovh-inner`). `TUNING_VERSION` **34**.
Rien n'a été écouté par l'agent ; William écoute avant que les styles soient déclarés finis (liens plus bas). Tout ce qui est écrit « mesuré » vient d'une exécution de cette
tâche, sur **16 graines**, avec le binaire de la branche (`harmonium-cli --export --record-truth`, 48 mesures, densité de batterie 1 par défaut, mesure de clôture exclue) ;
les profils « avant » sont ceux de `448f996` (`git archive`), les profils « après » ceux de la branche.

## Ce que le commit de reprise contenait

`TUNING_VERSION` 34 et sa ligne de doc, un test de swing droit pour le rock et le hip-hop, et deux scripts (`appliquer.py`, `batterie.py`) **jamais lancés** : aucun profil
n'était modifié. Gardés : l'idée de partir des fichiers de `448f996` sans toucher aux générateurs de 019, la regex qui retire le groupe `feel`, le calcul des décalages
par pas de la basse et des accords depuis les voix de la batterie. Refaits : la table des vélocités (absolue, elle remontait le niveau des accords de rock de 83 à 112 et
cassait la calibration du mixage : elle devient relative et conserve le niveau moyen), le `jitter` de la batterie, les garde-fous de `batterie.py` (voir point 3), les
tests, les mesures. Les `.pyc` committés par erreur sont retirés.

## Définitions des mesures

- **Statistiques de groove du GMD** (point 3). Signature d'une mesure : l'ensemble des (pas, grosse caisse | caisse claire) de vélocité ≥ 40 (grosse caisse 35/36, caisse claire
  37/38/40). Trois nombres **par fichier** `beat` 4/4 : part de la mesure la plus fréquente, mesures distinctes / mesures, part des mesures identiques à la précédente ;
  **médiane sur les fichiers** du groupe, fichiers d'au moins 8 mesures entières (sans ce plancher, les fichiers de 2 à 4 mesures gonflent la première statistique : la médiane
  du rock passe de 0,23 à 0,29). Cette définition retrouve exactement les médianes de l'orchestrateur pour le funk (0,33 / 0,47 / 0,08), le pop (0,31 / 0,61 / 0,14) et le latin
  (0,14 / 0,64 / 0,05) ; pour le rock elle donne 0,23 / 0,62 / 0,11 (orchestrateur : 0,23 / 0,62 / 0,09). Le groupe de chaque profil est celui de ses sources de 019
  (`mesure.py`, table `REF`) : rock `rock`, funk `funk`, pop `pop`, `pop/soft`, `pop/groove7`, soul les six sous-styles de `gmd_soul`, reggae `reggae`, blues
  `blues/shuffle` + `rock/shuffle` + `funk/purdieshuffle` ternaires, bossa / samba / baião leurs styles `latin/brazilian-*`, afro-cubain `afrocuban` + `afrocuban/rhumba`,
  jazz `jazz` et `jazz/swing` (swing et ternaire, 15 fichiers), ballade `jazz` ≤ 112 BPM (2 fichiers), valse `jazz` 3/4 (2 fichiers).
- **Côté moteur** : mêmes signatures sur la batterie rendue (vélocité ≥ 40 aussi), fenêtres consécutives de la longueur médiane des fichiers du groupe (au plus 47 mesures),
  médiane sur les fenêtres des 16 graines.
- **Mise en place** (point 4) : sur la vérité terrain (`performance.offset` de chaque note, en millièmes de pas, converti en ms au tempo de la mesure, strum compris), aux pas où
  deux voix attaquent ensemble. « 1re corde » = la note la plus en avance de l'accord ; « barycentre » = moyenne des cordes.
- **Vélocités** (point 2) : écart-type de toutes les notes de la piste, **par graine** ; le tableau donne le minimum sur les 16 graines.

## 1. Swing léger retiré

`gmd_rock` et `gmd_hiphop` n'ont plus de groupe `feel` (les variantes `droit` et `swing_leger` sont retirées de `session.toml` ; le feel est celui du profil, 0,5). Mesuré sur
16 graines : **avant, 4 graines de rock (3, 5, 8, 13) et 5 de hip-hop (5, 8, 12, 13, 16) en swing léger**, croches en contretemps retardées jusqu'à 65 et 75 ms (0,59 à 0,60 de
rapport, à 109 BPM : 55 ms, ce que l'orchestrateur entendait) ; **après, 0 graine**. Aucun autre profil droit n'a de variante de swing : `grep` des `session.toml`
(seuls la valse jazz, la ballade, les trads et ces deux profils en ont) et test `q0_straight_feel_tests` (13 profils `gmd_*` droits × 16 graines × 32 mesures : rapport 0,5 partout).
Le générateur de 019 (`r2-styles/habillage.py`) ne produit plus la variante, pour que `b8/generer.py` ne la rétablisse pas. Habillages : le nombre de combinaisons du rock passe
de 27648 à 13824 et celui du hip-hop de 311040 à 155520 (README et en-tête de `session.toml` mis à jour) ; **16 habillages distincts sur 16 graines pour 12 des 13 profils d'E1,
avant et après** (samba : 15, avant et après ; tableau dans « Preuves »).

## 2. Accents et notes fantômes

**Règle.** Chaque coup du comping (et de la basse quand elle était plate : écart-type < 8) garde la vélocité que 019 lui a donnée ; Q0 y ajoute un écart selon sa place dans la
mesure : `amplitude × coefficient(classe)` plus un décalage de niveau qui **conserve le niveau moyen de 019** (le mixage calibré reste valable). Classes : temps 1, temps 3,
backbeat (2 et 4), croche de contretemps, doubles croches, attaque anticipée, plus des pas précis (syncopes, clave) ; les coups probabilistes (notes fantômes) reçoivent un écart
supplémentaire négatif. Les formes sont les règles de métier de chaque style (`SHAPES` dans `appliquer.py`, une phrase par style dans le README du profil, section « Corrections Q0 ») :
charleston du jazz, syncopes de la bossa, surdo de la samba, clave de l'afro-cubain, « un » et scratchs du funk, skank du reggae, backbeat du rock et du shuffle. Aucun corpus
ne les mesure : ce sont des règles de métier, dites telles. Les amplitudes viennent de `tuner_velocites.py` (elles montent jusqu'à ce que le plus mauvais des 16 graines passe
12,3 pour les accords, 10,3 pour la basse, niveau moyen recalé à ±1,5). La basse en marche (jazz) garde son mécanisme (`accent` d'arrivée porté à 20, courbe de phrase ±18 d'A6).
**Plage** : `velocity_min = 32`, `velocity_max = 118` (A6) sur les accords et la basse de ces 13 profils : le modulateur de vélocité par défaut du SoundFont 2.01 est
−40·log10(v/127) dB, soit −24 dB à 32 et −1,3 dB à 118 ; la vérification hors ligne est ci-dessous (5 profils).

Réglages retenus :

| Profil | Comping : forme, amplitude / décalage de niveau | Basse | Batterie : `fixed` ; notes fantômes (nombre à chance) ; exposant ; tenue ; relances |
|---|---|---|---|
| `wjazzd_swing` | `jazz_comp` 28 / -10 | marche 78 → 72, arrivées 6 → +20, phrase ±18 | 0,8 ; 3 à 0,35 ; 1 ; ×1 ; ×1 |
| `wjazzd_ballad` | `ballade_comp` 24 / -1 | marche 66 → 61, arrivées 20 → +20, phrase ±18 | 0,7 ; 4 à 0,2 ; 1 ; ×6 ; ×2 |
| `gmd_jazz_waltz` | `valse_comp` 20 / -2 | marche 72 → 66, arrivées 20 → +20, phrase ±18 | 0,9 ; 5 à 0,3 ; 0,75 ; ×6 ; ×4 |
| `gmd_latin_brazilian_bossa` | `bossa_comp` 28 / -7 | `bossa_bass` 17 / -10 | 1 ; 0 à 0,5 ; 1 ; ×16 ; ×1 |
| `gmd_latin_brazilian_samba` | `samba_comp` 36 / -11 | `samba_bass` 28 / -2 | 1 ; 0 à 0,45 ; 1 ; ×1 ; ×1 |
| `gmd_latin_brazilian_baiao` | `baiao_comp` 28 / -14 | `baiao_bass` 14 / -6 | 0,5 ; 8 à 0,45 ; 2 ; ×24 ; ×1 |
| `gmd_afrocuban` | `afrocubain_comp` 28 / -8 | `afrocubain_bass` 14 / -6 | 0,5 ; 5 à 0,1 ; 0,5 ; ×16 ; ×0,25 |
| `gmd_funk` | `funk_comp` 20 / -3 | inchangée (motifs de 019 déjà accentués) | 0,95 ; 0 à 0,45 ; 1 ; ×1 ; ×2 |
| `gmd_soul` | `soul_comp` 26 / -8 | `soul_bass` 14 / -3 | 0,95 ; 0 à 0,45 ; 3 ; ×16 ; ×8 |
| `gmd_reggae` | `reggae_comp` 22 / -15 | `reggae_bass` 18 / -6 | 1 ; 3 à 0,05 ; 1 ; ×64 ; ×1 |
| `gmd_rock` | `rock_comp` 24 / -11 | `rock_bass` 15 / -3 | 1 ; 1 à 0,05 ; 2 ; ×2 ; ×4 |
| `gmd_pop` | `pop_comp` 30 / -10 | `pop_bass` 19 / -6 | 0,8 ; 2 à 0,2 ; 1 ; ×2 ; ×16 |
| `gmd_blues_shuffle` | `shuffle_comp` 20 / -8 | inchangée (motifs de 019 déjà accentués) | 0,95 ; 3 à 0,2 ; 2 ; ×4 ; ×1 |

Résultats :

| Profil | Accords : écart-type min (avant → après) | Accords : médiane des écarts-types | Accords : niveau moyen | Basse : écart-type min | Basse : niveau moyen | Plage accords + basse après |
|---|---|---|---|---|---|---|
| `wjazzd_swing` | 2,5 → **12,9** | 2,9 → 16,1 | 55,4 → 56,2 | 4,9 → 13,7 | 73,3 → 73,5 | 32 à 108 |
| `wjazzd_ballad` | 3,9 → **12,9** | 4,4 → 14,0 | 54,4 → 54,5 | 2,1 → 14,2 | 69,9 → 70,4 | 32 à 101 |
| `gmd_jazz_waltz` | 2,4 → **15,3** | 3,6 → 16,5 | 55,5 → 55,9 | 0,0 → 10,8 | 76,5 → 77,0 | 32 à 106 |
| `gmd_latin_brazilian_bossa` | 4,6 → **12,8** | 6,4 → 14,8 | 68,0 → 68,6 | 7,1 → 10,4 | 82,9 → 83,3 | 32 à 108 |
| `gmd_latin_brazilian_samba` | 7,3 → **12,6** | 8,7 → 15,7 | 67,3 → 67,1 | 6,7 → 9,5 | 86,4 → 86,2 | 32 à 118 |
| `gmd_latin_brazilian_baiao` | 5,2 → **12,6** | 6,3 → 15,1 | 67,6 → 68,6 | 7,2 → 10,6 | 82,6 → 83,0 | 32 à 108 |
| `gmd_afrocuban` | 5,4 → **12,6** | 7,2 → 14,8 | 66,8 → 66,6 | 6,7 → 10,5 | 87,5 → 87,8 | 32 à 107 |
| `gmd_funk` | 8,1 → **13,5** | 10,3 → 17,1 | 68,3 → 68,9 | 9,8 → 9,8 | 85,2 → 85,2 | 32 à 116 |
| `gmd_soul` | 4,8 → **12,4** | 5,5 → 13,3 | 69,1 → 69,6 | 5,2 → 10,6 | 89,0 → 89,3 | 46 à 115 |
| `gmd_reggae` | 6,4 → **12,4** | 8,0 → 19,4 | 75,4 → 75,5 | 4,7 → 10,6 | 90,3 → 91,4 | 32 à 116 |
| `gmd_rock` | 5,8 → **13,0** | 6,7 → 16,1 | 82,1 → 82,3 | 4,7 → 10,4 | 90,0 → 90,0 | 44 à 116 |
| `gmd_pop` | 4,7 → **12,9** | 5,3 → 16,0 | 71,1 → 71,6 | 5,1 → 10,4 | 86,8 → 86,8 | 40 à 113 |
| `gmd_blues_shuffle` | 10,0 → **15,1** | 10,9 → 17,5 | 72,5 → 73,2 | 9,2 → 9,2 | 88,2 → 88,2 | 32 à 118 |

- Cible « écart-type du comping ≥ 12 sur 16 graines » : **atteinte sur les 13 profils** (minimum 12,4). Niveau moyen des accords et de la basse conservé à ±1,1 près.
- Basse : écart-type minimum ≥ 10,4 sur les 10 profils retouchés (bossa, baião, afro-cubain, soul, reggae, rock, pop et les trois jazz) ; la samba reste à 9,5 (ses gabarits à deux tenues par mesure
  n'ont que deux coups ; au-delà, les accents de surdo saturent à 118), le funk (9,8) et le shuffle (9,2) gardent les accents de leurs motifs de 019.
- Plage : aucune note des accords ni de la basse hors de 32 à 118 sur 16 graines × 47 mesures (`velocity_min`, `velocity_max` d'A6 les garantissent).
- **Audibilité** (`audible.py`, rendu piste solo hors ligne, GeneralUser GS, crête des 120 ms après une attaque isolée, 3 graines) : la crête reste au-dessus de −26 dBFS
  pour toutes les tranches de vélocité mesurées sur le rock, la bossa, le funk, le reggae (accords 34–60 : de −25 à −16 dBFS ; basse ≥ 54 : de −25 à −12 dBFS) et au-dessus de −37 dBFS
  sur la ballade (accords de vélocité 61–90 : −36,8 dBFS, tranche à n = 35 attaques isolées, plus bruitée que les autres). Limite : la crête mélange la note et la queue de la précédente.

## 3. Batterie : groove tenu par phrase

**Mécanisme.** Les patterns, les phrases et les relances de 019 restent ; `batterie.py` règle, par profil, six nombres (tableau ci-dessus) : `fixed` (les coups de grosse
caisse et de caisse claire hors temps forts, certains jusque-là, deviennent probabilistes), notes fantômes de caisse claire ajoutées à des places libres, exposant des
probabilités mesurées, poids ×N des phrases qui répètent un seul pattern (le groove est **tenu sur la phrase**), poids ×N de « pas de relance » (le groove ne change qu'en fin
de phrase, avec la relance). Les réglages sont cherchés par `tuner_batterie.py` (tirages aléatoires puis coordonnées sur 16 graines, coût = écarts aux trois statistiques du groupe
GMD ; dans la dernière version, plus une pénalité si la distance au GMD recule). Contrats de 019 que la recherche a dû respecter (les tests de profil les vérifient) : la densité 0 reste le timekeeper
seul, la densité 2 garde ses notes fantômes, la grosse caisse du baião (zabumba) reste complète, l'exposition de la samba reste au surdo, les balais n'ajoutent pas de balayage aux
baguettes, la samba garde au moins 8 relances au sommet sur 16 graines. Les modifications ne touchent donc que les patterns de densité 1 (et les notes fantômes de densité 1 et 2).

| Profil | GMD (fichiers ; mesures) | Part de la mesure la plus fréquente | Mesures distinctes / mesures | Identique à la précédente | Écart max au GMD après |
|---|---|---|---|---|---|
| `wjazzd_swing` | 15 ; 46 | 0,20 : 0,90 → **0,87** | 0,59 : 0,04 → **0,09** | 0,18 : 0,83 → **0,80** | 0,67 ✗ |
| `wjazzd_ballad` | 2 ; 55 | 0,25 : 0,37 → **0,17** | 0,56 : 0,14 → **0,53** | 0,12 : 0,41 → **0,12** | 0,08 ✗ |
| `gmd_jazz_waltz` | 2 ; 108 | 0,10 : 0,68 → **0,11** | 0,53 : 0,06 → **0,53** | 0,10 : 0,89 → **0,07** | 0,04 ✓ |
| `gmd_latin_brazilian_bossa` | 2 ; 315 | 0,24 : 0,21 → **0,21** | 0,37 : 0,49 → **0,47** | 0,12 : 0,08 → **0,08** | 0,10 ✗ |
| `gmd_latin_brazilian_samba` | 5 ; 68 | 0,04 : 0,34 → **0,34** | 0,85 : 0,47 → **0,47** | 0,01 : 0,37 → **0,37** | 0,38 ✗ |
| `gmd_latin_brazilian_baiao` | 3 ; 84 | 0,11 : 0,44 → **0,30** | 0,64 : 0,18 → **0,52** | 0,13 : 0,50 → **0,17** | 0,19 ✗ |
| `gmd_afrocuban` | 2 ; 100 | 0,08 : 0,18 → **0,13** | 0,69 : 0,61 → **0,68** | 0,17 : 0,08 → **0,07** | 0,10 ✗ |
| `gmd_funk` | 30 ; 59 | 0,36 : 0,33 → **0,27** | 0,44 : 0,26 → **0,41** | 0,08 : 0,13 → **0,09** | 0,09 ✗ |
| `gmd_soul` | 28 ; 16 | 0,62 : 0,44 → **0,59** | 0,26 : 0,19 → **0,25** | 0,40 : 0,27 → **0,40** | 0,03 ✓ |
| `gmd_reggae` | 2 ; 66 | 0,28 : 0,20 → **0,21** | 0,54 : 0,56 → **0,59** | 0,17 : 0,04 → **0,04** | 0,12 ✗ |
| `gmd_rock` | 87 ; 25 | 0,21 : 0,18 → **0,20** | 0,62 : 0,70 → **0,62** | 0,11 : 0,06 → **0,10** | 0,01 ✓ |
| `gmd_pop` | 13 ; 16 | 0,31 : 0,31 → **0,31** | 0,56 : 0,56 → **0,56** | 0,14 : 0,30 → **0,13** | 0,01 ✓ |
| `gmd_blues_shuffle` | 4 ; 84 | 0,17 : 0,19 → **0,19** | 0,47 : 0,44 → **0,47** | 0,10 : 0,21 → **0,15** | 0,06 ✗ |

- **Dans ± 0,05 du groupe GMD sur les trois statistiques : valse, soul, rock, pop** (0,01 à 0,04). Blues : 0,06 (« identique à la précédente », 0,15 contre 0,10) ; les huit autres profils : voir la raison ci-dessous.
- **Hors tolérance, avec la raison mesurée** :
  - `wjazzd_swing` (0,67) : sa batterie ne joue ni grosse caisse ni caisse claire à la densité 0 (timekeeper seul, contrat des tests) ; 51 % des mesures ont une signature vide
    (66 % avant ; ballade 18 → 9 %, valse 25 → 6 %) et le reste est le même pattern de balai. Le GMD jazz comporte de la grosse caisse ou de la caisse claire dans presque chaque mesure : la statistique n'est pas atteignable sans
    rompre le contrat ; ajouter ces notes à la densité 0 a été essayé (mesures distinctes 0,50) et refusé par les tests du blues et du reggae, qui ont le même contrat. Ballade (0,08, première statistique seule) : même cause.
  - `gmd_latin_brazilian_samba` (0,38), `gmd_latin_brazilian_baiao` (0,19), bossa (0,10), afro-cubain (0,10) : la « grosse caisse » et la « caisse claire » de ces profils sont des instruments
    d'orchestration (surdo, zabumba, caixa) qui définissent le style ; les groupes GMD sont de 2 à 5 fichiers d'un ou deux batteurs et leurs statistiques (samba : 85 % de mesures
    distinctes, 4 % pour la plus fréquente) décrivent un batteur sur un kit, pas cette orchestration. La samba est laissée **inchangée** (le meilleur réglage trouvé,
    distinctes 0,47 → 0,56, éloignait sa distance au GMD de 0,314 à 0,323 et ne rapprochait pas les deux autres statistiques).
  - `gmd_funk` (0,09 : part de la mesure la plus fréquente 0,27 contre 0,36) : aucun des réglages essayés (près de 200 évaluations) ne rapproche cette statistique sans écarter les deux autres.
  - `gmd_reggae` (0,12 : « identique à la précédente » 0,04 contre 0,17) : le meilleur compromis avec une distance au GMD qui ne recule pas (le réglage qui tenait les trois statistiques à 0,04 près éloignait la distance de 0,179 à 0,240).
- **Distance de la batterie au GMD** (méthode de `b8/distance_gmd.py`, Bray-Curtis des fréquences d'attaque par pas, `distance.py`, 16 graines × 48 mesures) : **moyenne sans perte
  sur les 13 profils (écart maximal +0,003, blues)** ; gains notables : valse 0,502 → 0,430, soul 0,252 → 0,205, afro-cubain 0,273 → 0,248, baião 0,157 → 0,139, rock 0,186 → 0,173.

| Profil | toutes voix | grosse caisse | caisse claire | charleston | moyenne (avant → après) |
|---|---|---|---|---|---|
| `wjazzd_swing` | 0,379 → 0,368 | 0,850 → 0,850 | 0,467 → 0,448 | 0,675 → 0,675 | 0,593 → **0,585** |
| `wjazzd_ballad` | 0,573 → 0,549 | 0,900 → 0,921 | 0,730 → 0,670 | 0,931 → 0,948 | 0,783 → **0,772** |
| `gmd_jazz_waltz` | 0,323 → 0,226 | 0,638 → 0,605 | 0,439 → 0,286 | 0,607 → 0,605 | 0,502 → **0,430** |
| `gmd_latin_brazilian_bossa` | 0,105 → 0,104 | 0,076 → 0,077 | 0,471 → 0,468 | 0,162 → 0,163 | 0,203 → **0,203** |
| `gmd_latin_brazilian_samba` | 0,111 → 0,111 | 0,189 → 0,189 | 0,668 → 0,668 | 0,288 → 0,288 | 0,314 → **0,314** |
| `gmd_latin_brazilian_baiao` | 0,035 → 0,026 | 0,058 → 0,055 | 0,337 → 0,268 | 0,199 → 0,206 | 0,157 → **0,139** |
| `gmd_afrocuban` | 0,189 → 0,168 | 0,212 → 0,257 | 0,403 → 0,280 | 0,288 → 0,288 | 0,273 → **0,248** |
| `gmd_funk` | 0,057 → 0,051 | 0,243 → 0,245 | 0,168 → 0,162 | 0,099 → 0,100 | 0,142 → **0,140** |
| `gmd_soul` | 0,086 → 0,084 | 0,314 → 0,297 | 0,302 → 0,193 | 0,304 → 0,245 | 0,252 → **0,205** |
| `gmd_reggae` | 0,151 → 0,150 | 0,161 → 0,160 | 0,326 → 0,329 | 0,079 → 0,081 | 0,179 → **0,180** |
| `gmd_rock` | 0,058 → 0,085 | 0,148 → 0,216 | 0,230 → 0,175 | 0,309 → 0,216 | 0,186 → **0,173** |
| `gmd_pop` | 0,071 → 0,117 | 0,220 → 0,180 | 0,180 → 0,249 | 0,212 → 0,140 | 0,171 → **0,172** |
| `gmd_blues_shuffle` | 0,028 → 0,026 | 0,076 → 0,103 | 0,136 → 0,123 | 0,121 → 0,121 | 0,090 → **0,093** |

  `b8/distance_gmd.py` lui-même (profils de B8, colonne « profil » moyenne) : rock 0,186 → 0,173, pop 0,171 → 0,172, soul 0,252 → 0,205, reggae 0,179 → 0,180, hip-hop 0,147 → 0,146
  (session seule), disco 0,163 → 0,163 et afrobeat 0,152 → 0,152 (profils non touchés : identiques).

## 4. Mise en place

**Constat (avant).** Les voix de la batterie portent les décalages mesurés par voix (A6 : médianes du GMD, de −0,1 à −0,25 pas : 15 à 35 ms **en avance** sur la grille) pendant que la basse
et les accords étaient posés sur la grille (0 pas) : 14 à 23 ms de retard médian de la basse sur la grosse caisse (baião, afro-cubain, soul, pop, reggae, blues), 15 à 20 ms des accords sur
la caisse claire, et une dispersion de 11 à 13 ms (soul, reggae, rock, pop : `jitter` 0,1 pas, le plafond d'A6, **partout, batterie comprise**). Jazz : le piano était réglé 0,2 pas (+20 ms mesuré
par la JTD) derrière une batterie sans décalage : 46 ms de retard sur la ballade, où 0,2 pas de 16 vaut 50 ms à 48 BPM (la valeur de `wjazzd_swing` avait été reprise telle quelle en pas, pas en ms).

**Décomposition et règles retenues.**

| Source | Effet mesuré avant | Réglage Q0 |
|---|---|---|
| Décalage par voix de la batterie (A6 `voices`) | −15 à −35 ms de la grille, par voix | **inchangé** (mesuré) : la basse et les accords s'y alignent |
| Placement de la basse | 0 pas partout ⇒ retard de 4 à 23 ms | `offsets` de la basse = décalage de la grosse caisse, pas par pas (temps / hors temps) |
| Placement des accords | 0 pas ⇒ retard de 2 à 20 ms ; jazz : +0,2 pas | `offsets` des accords = moyenne caisse claire et timekeeper, pas par pas ; jazz : 0 (la batterie n'a aucun décalage par voix) |
| `jitter` de la basse et des accords | 0,1 pas (et 0,03 à 0,05 pour la bossa, la samba, le funk…) | **0,02 pas** (écart-type 1,2 % de pas, 1 à 2 ms) |
| `jitter` de la batterie (`gmd_rock`, `gmd_soul`, `gmd_reggae`, `gmd_pop`) | 0,1 pas (écart-type 0,058 pas ≈ 8 ms) | 0,07 / 0,045 / 0,06 / 0,06 pas : √3 × l'écart-type **intra-fichier** du GMD (0,03 à 0,05 pas par voix : 4 à 6 ms) ; la dispersion de 0,14 pas mesurée entre fichiers est celle des batteurs entre eux, pas d'un batteur |
| Égrenage (strum) des accords (bossa, samba, funk, reggae, rock, blues) | première corde ≈ 3 à 11 ms **avant** la caisse claire (`anchor = 0,5`) | `anchor = 0` : la première corde tombe sur la référence, le barycentre 3 à 7 ms après |

**Valeurs d'ensemble visées, et pourquoi.** Basse sur la grosse caisse : médiane ≤ 1 ms (« à quelques ms près », règle de métier du brief) ; accords (première corde) sur la caisse claire : médiane
dans ± 4 ms ; dispersion de chacune des deux paires limitée par celle de la batterie seule (écart-type intra-fichier GMD 4 à 6 ms) : la basse et les accords n'ajoutent que 1 à 2 ms. Dérive de la grille :
la basse et les accords ne dépassent pas la batterie mesurée (p95 de |décalage|) ; **exceptions** : jazz, où la batterie n'a aucun décalage (0 ms) et où les accords gardent leur `jitter` (p95 6 à 18 ms ;
pour la ballade, des accords portent un décalage de −0,05 à −0,08 pas (15 à 20 ms), cause non établie [INFERENCE : attaques anticipées du comping]) ; afro-cubain, où l'égrenage porte les accords et la basse à 26 ms contre 25 ms pour la batterie.

| Profil | Basse − grosse caisse : médiane ; écart-type (avant → après) | 1re corde − caisse claire : médiane ; écart-type | Barycentre des cordes − caisse claire : médiane | Dérive de la grille p95 : accords, basse, batterie max (après) |
|---|---|---|---|---|
| `wjazzd_swing` | +0,2 ; 9,8 → **-2,0 ; 7,1** | -2,1 ; 18,6 → **-2,5 ; 4,3** | -2,5 | 8,4, 5,8, 0,0 |
| `wjazzd_ballad` | — | +46,2 ; 20,1 → **-0,9 ; 6,6** | -0,9 | 18,0, 5,2, 0,0 |
| `gmd_jazz_waltz` | -0,2 ; 5,7 → **-0,1 ; 1,2** | +18,9 ; 7,5 → **-0,2 ; 1,5** | -0,2 | 6,3, 2,2, 0,0 |
| `gmd_latin_brazilian_bossa` | -0,9 ; 12,1 → **+0,0 ; 1,4** | +0,1 ; 4,3 → **-3,6 ; 1,6** | +3,2 | 14,5, 23,7, 27,3 |
| `gmd_latin_brazilian_samba` | +1,9 ; 4,1 → **+0,0 ; 1,8** | +4,1 ; 5,2 → **+1,5 ; 1,9** | +4,2 | 8,6, 8,7, 8,1 |
| `gmd_latin_brazilian_baiao` | +15,1 ; 3,0 → **+0,1 ; 1,7** | +17,1 ; 2,3 → **+2,1 ; 2,0** | +2,1 | 18,6, 21,3, 30,6 |
| `gmd_afrocuban` | +23,2 ; 3,7 → **+0,0 ; 1,6** | +19,7 ; 5,7 → **-2,1 ; 3,6** | -2,1 | 25,9, 26,4, 25,0 |
| `gmd_funk` | +9,2 ; 4,4 → **+0,0 ; 1,8** | +2,4 ; 5,6 → **-0,1 ; 2,9** | +2,4 | 10,3, 13,9, 22,1 |
| `gmd_soul` | +22,2 ; 13,4 → **+0,3 ; 4,5** | +14,9 ; 13,3 → **+1,8 ; 4,7** | +1,8 | 16,2, 28,0, 33,4 |
| `gmd_reggae` | +15,5 ; 12,3 → **+0,3 ; 5,4** | +20,6 ; 12,9 → **+2,7 ; 6,1** | +6,9 | 24,6, 19,3, 32,7 |
| `gmd_rock` | +3,8 ; 11,5 → **-0,1 ; 6,1** | -6,0 ; 11,8 → **-1,6 ; 6,1** | +7,6 | 17,5, 7,8, 20,4 |
| `gmd_pop` | +18,9 ; 12,0 → **+0,2 ; 5,0** | +15,5 ; 12,0 → **+2,1 ; 5,6** | +2,1 | 22,5, 29,8, 35,5 |
| `gmd_blues_shuffle` | +14,0 ; 5,3 → **+0,0 ; 2,2** | +3,5 ; 6,7 → **+2,0 ; 2,9** | +7,0 | 15,3, 21,5, 23,3 |

Dans la colonne « barycentre » le retard est celui du centre de l'accord (égrenage compris) ; rock, reggae et blues restent à 7 ms (3 intervalles de 5 à 7 ms de strum). Le décalage accords / timekeeper
n'est pas réduit davantage : à l'intérieur de la batterie, charleston, pédale et ride diffèrent de 5 à 20 ms (`soul` : pédale −0,2 pas, charleston −0,07), mesure du GMD conservée.

## Preuves

- **Habillages** (premier tirage de session, 16 graines) : le test `seize_graines_donnent_au_moins_huit_habillages_dans_les_bornes_du_style` passe, le swing de `SWINGS` est désormais `[0,5]`.
| Profil | avant | après |
|---|---:|---:|
| `wjazzd_swing` | 16 | 16 |
| `wjazzd_ballad` | 16 | 16 |
| `gmd_jazz_waltz` | 16 | 16 |
| `gmd_latin_brazilian_bossa` | 16 | 16 |
| `gmd_latin_brazilian_samba` | 15 | 15 |
| `gmd_latin_brazilian_baiao` | 16 | 16 |
| `gmd_afrocuban` | 16 | 16 |
| `gmd_funk` | 16 | 16 |
| `gmd_soul` | 16 | 16 |
| `gmd_reggae` | 16 | 16 |
| `gmd_rock` | 16 | 16 |
| `gmd_pop` | 16 | 16 |
| `gmd_blues_shuffle` | 16 | 16 |

- **`seed-variety --chorus 4`** (16 graines × 4 chorus, `harmonium-lab`, mêmes profils avant / après) : couche « accompagnement », tous les chorus distincts, 16 suites distinctes sur 16 graines
  avant et après pour les 13 profils ; **aucune perte de plus d'un point sur la couche complète sauf le rock : distance graduée entre graines 70,1 % → 69,0 % (−1,1 point)**, conséquence directe
  du retrait du groupe `feel` (point 1) et de la tenue des grooves (point 3, voulue). Par couche, les pertes de plus d'un point sont toutes celles de la batterie, de la basse et du comping du rock
  (−2,6 à −3,8 points de distance graduée entre graines) et la batterie du pop (−2,1) : la batterie tient son groove, par construction.

| Profil | Mesures changées d'un chorus au suivant | Chorus distincts / graine | Suites distinctes / 16 | Distance graduée entre graines |
|---|---|---|---|---|
| `wjazzd_swing` | 100,0% → 100,0% | 4 → 4 | 16 → 16 | 63,4% → 63,6% |
| `wjazzd_ballad` | 99,4% → 99,7% | 4 → 4 | 16 → 16 | 59,8% → 61,7% |
| `gmd_jazz_waltz` | 99,7% → 100,0% | 4 → 4 | 16 → 16 | 63,2% → 66,1% |
| `gmd_latin_brazilian_bossa` | 100,0% → 100,0% | 4 → 4 | 16 → 16 | 65,2% → 65,3% |
| `gmd_latin_brazilian_samba` | 99,9% → 99,9% | 4 → 4 | 16 → 16 | 59,6% → 59,6% |
| `gmd_latin_brazilian_baiao` | 96,1% → 96,6% | 4 → 4 | 16 → 16 | 51,1% → 51,8% |
| `gmd_afrocuban` | 100,0% → 100,0% | 4 → 4 | 16 → 16 | 74,9% → 75,2% |
| `gmd_funk` | 99,9% → 99,9% | 4 → 4 | 16 → 16 | 69,6% → 69,5% |
| `gmd_soul` | 99,7% → 99,9% | 4 → 4 | 16 → 16 | 61,5% → 62,7% |
| `gmd_reggae` | 99,2% → 99,4% | 4 → 4 | 16 → 16 | 56,8% → 57,0% |
| `gmd_rock` | 99,6% → 99,9% | 4 → 4 | 16 → 16 | 70,1% → 69,0% |
| `gmd_pop` | 98,8% → 99,7% | 4 → 4 | 16 → 16 | 61,5% → 61,1% |
| `gmd_blues_shuffle` | 99,5% → 99,8% | 4 → 4 | 16 → 16 | 56,9% → 57,1% |

- **Tests** : `cargo test --workspace -q --no-fail-fast` vert ; `cargo test -p harmonium --features odin` vert ; `cargo test -p harmonium_bevy` vert ; `cargo check -p harmonium_studio` ;
  `cargo +nightly fmt --all --check` ; `profile_d2_main.json` régénéré (`HARMONIUM_UPDATE_GOLDENS=1`) : seul `source` change (`TUNING_VERSION:34`, mêmes notes). Tests de profil modifiés :
  `gmd_session_profile_tests` (swing de la source : droit seul), `gmd_styles_profile_tests` (la vélocité moyenne de la basse de l'arc est mesurée sur le premier temps : les accents de Q0 donnent aux autres
  places un écart propre à chaque motif, et la retombée ne joue que le premier temps), nouveau `q0_straight_feel_tests`.
- **Autres profils identiques à `main`** : `git diff` ne touche que les 13 profils d'E1, `gmd_hiphop` (`session.toml`) et leurs README ; disco, afrobeat, second line, trads, jeux : octet pour octet.

## Liens d'écoute (deux graines contrastées par profil, studio de William)

| Profil | Liens | Ce qui change |
|---|---|---|
| `wjazzd_swing` | `harmonium://open?profile=wjazzd_swing&seed=1`<br>`harmonium://open?profile=wjazzd_swing&seed=2` | comping en charleston accentué (« et » du 2 et anticipation) au lieu de coups égaux ; basse en marche avec arrivées d'accord appuyées ; comping et basse sur la batterie (le piano n'est plus 20 ms derrière) ; notes fantômes de balai ajoutées |
| `wjazzd_ballad` | `harmonium://open?profile=wjazzd_ballad&seed=4`<br>`harmonium://open?profile=wjazzd_ballad&seed=7` | accords tenus avec accent sur le « et » du 2 et l'anticipation ; basse nuancée par phrase ; comping 46 ms en retard avant, sur la batterie maintenant ; plus de variété de caisse claire / balai d'une mesure à l'autre |
| `gmd_jazz_waltz` | `harmonium://open?profile=gmd_jazz_waltz&seed=1`<br>`harmonium://open?profile=gmd_jazz_waltz&seed=4` | temps 2 appuyé, 1 et 3 légers ; basse nuancée par phrase ; comping 19 ms en retard avant, sur la batterie maintenant ; batterie qui tient son groove 2 à 4 mesures au lieu d'un balai identique partout |
| `gmd_latin_brazilian_bossa` | `harmonium://open?profile=gmd_latin_brazilian_bossa&seed=1`<br>`harmonium://open?profile=gmd_latin_brazilian_bossa&seed=2` | syncopes (pas 6 et 14) accentuées au comping, basse appuyée au temps 1 ; basse et accords calés sur la grosse caisse et la cross-stick ; phrases à un seul pattern pesant ×16 à la batterie |
| `gmd_latin_brazilian_samba` | `harmonium://open?profile=gmd_latin_brazilian_samba&seed=1`<br>`harmonium://open?profile=gmd_latin_brazilian_samba&seed=2` | accent sur les contretemps 2 et 4 au comping et à la basse (surdo) ; basse et accords calés sur la batterie ; batterie inchangée (aucun réglage ne rapproche le GMD sans reculer sa distance) |
| `gmd_latin_brazilian_baiao` | `harmonium://open?profile=gmd_latin_brazilian_baiao&seed=4`<br>`harmonium://open?profile=gmd_latin_brazilian_baiao&seed=10` | zabumba / temps 1 appuyé au comping et à la basse ; basse (15 ms de retard avant) et accords (17 ms) calés sur la batterie ; relances moins fréquentes, doubles croches fantômes de la caixa |
| `gmd_afrocuban` | `harmonium://open?profile=gmd_afrocuban&seed=1`<br>`harmonium://open?profile=gmd_afrocuban&seed=9` | clave accentuée au comping, tumbao accentué à la basse ; basse (23 ms de retard avant) et accords (20 ms) calés sur la batterie ; notes fantômes de caisse claire, grooves tenus (poids ×16 des phrases à un seul pattern), relances plus fréquentes |
| `gmd_funk` | `harmonium://open?profile=gmd_funk&seed=1`<br>`harmonium://open?profile=gmd_funk&seed=11` | le « un » très appuyé, scratchs fantômes au comping ; basse et accords calés sur la batterie (basse 9 ms de retard avant) ; relances moins fréquentes |
| `gmd_soul` | `harmonium://open?profile=gmd_soul&seed=1`<br>`harmonium://open?profile=gmd_soul&seed=8` | temps 1 et backbeat appuyés, scratchs fantômes au comping, notes mortes à la basse ; basse et accords sur la batterie (22 et 15 ms de retard avant, dispersion 13 ms avant, 5 ms après) ; groove tenu par phrase |
| `gmd_reggae` | `harmonium://open?profile=gmd_reggae&seed=4`<br>`harmonium://open?profile=gmd_reggae&seed=7` | skank appuyé sur le « et » des pas 6 et 14, basse appuyée sur le temps 3 (one drop) ; basse et accords calés sur la batterie (16 et 21 ms de retard avant) ; phrases à un seul pattern pesant ×64 |
| `gmd_rock` | `harmonium://open?profile=gmd_rock&seed=7`<br>`harmonium://open?profile=gmd_rock&seed=16` | temps 1, backbeat et anticipation appuyés au comping, basse appuyée au temps 1 ; croches droites sur toutes les graines (graines 3 et 5 : croches en contretemps retardées de 55 ms avant) ; basse et accords sur la batterie (dispersion 12 ms avant, 6 ms après) |
| `gmd_pop` | `harmonium://open?profile=gmd_pop&seed=1`<br>`harmonium://open?profile=gmd_pop&seed=11` | temps 1 et backbeat appuyés, croches fantômes ; basse et accords calés sur la batterie (19 et 16 ms de retard avant) ; phrases à un seul pattern pesant ×2, relances plus rares |
| `gmd_blues_shuffle` | `harmonium://open?profile=gmd_blues_shuffle&seed=1`<br>`harmonium://open?profile=gmd_blues_shuffle&seed=11` | temps 1 et backbeat (2 et 4) très appuyés, triolet du milieu fantôme ; basse (14 ms de retard avant) calée sur la grosse caisse ; phrases à un seul pattern pesant ×4 |

Pour le rock et le hip-hop, les graines 3 et 5 (rock) et 5 et 8 (hip-hop) étaient celles du swing léger. Hip-hop (`gmd_hiphop`) : `harmonium://open?profile=gmd_hiphop&seed=5` et
`harmonium://open?profile=gmd_hiphop&seed=8` : croches en contretemps droites (retardées de 75 ms avant), rien d'autre ne change.

## Limites

- Les formes d'accents, les amplitudes et les placements sont des **règles de métier** ; seule l'écoute les valide. Les accords de jazz perdent leur retard de 20 ms mesuré par la JTD (remettre
  `offsets` d'une ligne si l'écoute le préfère).
- Groove de batterie : 9 profils sur 13 sont hors de ± 0,05 sur au moins une statistique (raisons au point 3) ; la mesure du GMD de ces groupes repose sur 2 à 5 fichiers et 1 à 2 batteurs. Le choix a été de
  ne pas reculer la distance au GMD (écart maximal +0,003), de ne pas rompre les contrats de 019 et de laisser la samba inchangée.
- Les statistiques de groove sont mesurées à la densité de batterie 1 ; la densité 2 garde ses notes fantômes de 019 (aux patterns inchangés).
- Les mesures de mise en place utilisent la vérité terrain du moteur, pas un rendu audio ; aucune écoute.
- Reste à faire pour la vague 2 : sons (B1), nouvelles sources (S1) et réécoute par style.

## Reproduction

```sh
python3 specs/020-qualite-des-styles/q0/appliquer.py        # points 1, 2, 4 (profile.toml, session.toml)
python3 specs/020-qualite-des-styles/q0/batterie.py         # point 3 (percussion.toml)
python3 specs/020-qualite-des-styles/q0/readmes.py          # sections « Corrections Q0 » des README
LAB=~/Project/bascanada/harmonium_lab
$LAB/.venv/bin/python specs/020-qualite-des-styles/q0/mesure.py <dossier des profils> <étiquette>     # points 2, 3, 4
$LAB/.venv/bin/python specs/020-qualite-des-styles/q0/distance.py <dossier des profils> <étiquette>   # distance au GMD
python3 specs/020-qualite-des-styles/q0/rapport.py /tmp/h020-q0                                       # tableaux ci-dessus
$LAB/.venv/bin/python specs/020-qualite-des-styles/q0/audible.py profiles gmd_rock                    # audibilité hors ligne
```
Les profils de 019 (`BASE = 448f996`) servent d'entrée à `appliquer.py` et `batterie.py` : relancer les scripts redonne les mêmes fichiers.
