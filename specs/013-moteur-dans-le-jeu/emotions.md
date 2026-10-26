# Les 4 émotions de la musique — guide du concepteur de jeu

Le jeu pilote la musique avec **quatre boutons**. Ce document dit, pour chacun, ce qu'on entend,
dans quelle plage il sert, ce qu'il fait selon le style, et comment les combiner. Il est écrit pour
quelqu'un qui n'est pas musicien : les mots techniques sont expliqués à la fin. Le détail du code est
dans [`emotions-inventaire.md`](emotions-inventaire.md) ; les tableaux complets (7 styles × 20 graines ×
5 niveaux) dans [`emotions-mesures.md`](emotions-mesures.md). Le branchement sur les signaux du jeu
(ennemis, vie, jauges, zone) **appartient au jeu** : les situations ci-dessous sont des suggestions.

## En bref

| Bouton | Plage | Ce qu'on entend | Quand ça s'entend | Sans effet sur |
|---|---|---|---|---|
| **arousal** — énergie | 0 à 1 (neutre 0,5) | plus vite, plus fort, accompagnement plus occupé | tempo et force : mesure suivante ; paliers de pieds et de guitare à 0,2 / 0,75 / 1 | mode, tonalité, notes de l'air |
| **valence** — clair ↔ sombre | −1 à +1 (neutre 0,3) | mode majeur ou mineur ; registre plus aigu ou plus grave | mode : fin de phrase (~4 mesures) ; tonalité et registre : **air suivant** | tempo, accompagnement |
| **density** — densité | 0 à 1 (neutre 0,5) | les instruments entrent un à un (0 = violon seul), puis plus de notes et d'ornements | pistes, pieds, guitare : mesure suivante | tempo, force du violon |
| **tension** — nervosité | 0 à 1 (genou à 0,3) | notes courtes et sèches, contretemps, harmonie qui « tire » | mesure suivante | mode, registre, pistes présentes |

Le jeu appelle `set_emotions(arousal, valence, density, tension)` puis demande au moteur de
régénérer l'avenir (`invalidate_future`) : les changements s'entendent à la mesure suivante, sauf la valence (voir
plus bas). Un air en cours garde ses notes : les émotions changent **comment** il est joué, pas ce qui est écrit
(sauf le mode et quelques notes ajoutées).

**Styles mesurés** (colonnes des tableaux) : `reel QC` reel québécois · `reel AC` reel acadien ·
`gigue QC` gigue québécoise · `valse QC` valse québécoise · `marche GB` marche britannique coloniale ·
`gigue GB` gigue britannique coloniale · `hornpipe GB` hornpipe britannique coloniale. Chaque case
des tableaux de ce document est **bas · neutre · haut** (niveau 0 · point neutre du style · niveau 1 ; pour la
valence −1 · neutre · +1), les trois autres boutons étant au neutre, moyenne sur 20 graines. Certains styles n'ont
pas certaines pistes : la marche n'a ni guitare ni basse ; les gigue et hornpipe britanniques n'ont pas de pieds
ni de 2ᵉ voix.

## Arousal — l'énergie

**Ce que ça fait entendre.** À 0, une veillée : la musique ralentit, le violon joue doucement, la
guitare ne frappe que la moitié de ses coups et les pieds se réduisent à deux frappes par mesure. À 1, un
élan : plus rapide, plus fort, les notes plus vives, la guitare frappe deux fois plus, les pieds
s'ajoutent, la basse « marche » davantage et la 2ᵉ voix s'écarte plus du violon.

| Mesure | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| Tempo (BPM) | 220,0 · 230,0 · 245,0 | 218,0 · 228,0 · 243,0 | 162,5 · 172,5 · 187,5 | 135,0 · 145,0 · 160,0 | 96,0 · 108,0 · 123,0 | 148,0 · 156,0 · 166,0 | 86,0 · 92,0 · 100,0 |
| Force du violon (1 à 127) | 65,0 · 92,3 · 118,7 | 64,6 · 91,7 · 118,1 | 64,5 · 92,6 · 118,0 | 67,3 · 97,6 · 120,5 | 74,8 · 101,1 · 123,6 | 67,1 · 91,2 · 113,3 | 70,0 · 95,9 · 117,5 |
| Durée moyenne d'une note (pas) | 2,30 · 2,25 · 2,12 | 2,24 · 2,12 · 1,98 | 2,45 · 2,44 · 2,30 | 3,00 · 3,00 · 2,87 | 3,97 · 3,96 · 3,24 | 2,44 · 2,43 · 2,26 | 2,44 · 2,50 · 2,26 |
| Coups de guitare par mesure | 3,99 · 7,97 · 15,93 | 3,99 · 7,97 · 15,93 | 3,99 · 7,97 · 15,93 | 4,00 · 8,00 · 16,00 | 0,00 · 0,00 · 0,00 | 7,97 · 7,97 · 21,90 | 7,97 · 7,97 · 12,28 |
| Frappes de pieds par mesure | 2,00 · 7,97 · 11,95 | 2,00 · 7,97 · 11,95 | 2,00 · 5,98 · 7,97 | 1,00 · 3,00 · 5,00 | 9,50 · 9,50 · 14,00 | 0,00 · 0,00 · 0,00 | 0,00 · 0,00 · 0,00 |
| Notes de passage de la basse par mesure | 0,377 · 0,627 · 0,845 | 0,425 · 0,739 · 0,958 | 0,520 · 0,678 · 0,978 | 0,003 · 0,011 · 0,009 | 0,000 · 0,000 · 0,000 | 0,445 · 0,769 · 0,908 | 0,227 · 0,381 · 0,558 |
| 2ᵉ voix différente du violon (% des notes) | 8,4 · 12,6 · 18,6 | 9,2 · 13,6 · 17,6 | 5,3 · 8,8 · 10,2 | 4,1 · 5,0 · 5,9 | 2,8 · 1,9 · 3,1 | — · — · — | — · — · — |

**Plage utile.** Le tempo et la force sont continus : tout le segment 0 → 1 sert, et chaque pas de 0,1
s'entend (reel : environ 2,5 BPM et 5 points de force). Les textures procèdent par **paliers** :

| Arousal | Guitare | Pieds (reel) | À savoir |
|---|---|---|---|
| 0 à 0,2 | moitié moins de coups | 2 par mesure | accompagnement allégé |
| 0,25 à 0,7 | normal | 8 par mesure | plage courante : seuls tempo et force bougent |
| 0,75 à 0,95 | normal | 12 par mesure | les pieds s'ajoutent |
| 1 | **double** | 12 par mesure | seul le niveau 1 double la guitare |

Mêmes seuils dans le reel québécois, le reel acadien, la gigue québécoise et la valse. La marche ne bouge qu'à
1 (9,5 → 14 frappes de pieds ; roulements de caisse claire, mais voir *Limites*). **Au-dessus de 0,75, le volume atteint
0 dBFS** : la gigue britannique à 1 (+0,87 dBFS), le reel acadien à 0,75 (+0,11), la valse à 1 (+0,02) écrêtent sur
certaines graines (*Limites*).

**Situations possibles.** Exploration calme (0,2–0,3) ; déplacement normal (0,5) ; poursuite ou combat
(0,75–0,9) ; dernier assaut, boss (1). L'arousal suit bien une « intensité d'action », pas une quantité d'objets.

## Valence — clair ↔ sombre

**Ce que ça fait entendre.** Sous −0,3, la musique passe en **mode sombre** (mineur) ; au-dessus de 0,1, en
mode clair (majeur) ; entre les deux, elle garde le mode qui joue (zone de maintien : elle évite de basculer
sans cesse). Ce n'est pas un simple filtre : l'air suivant démarre dans une autre **tonalité** et un autre
**registre** (de −1 à +1, le violon monte de dix demi-tons en moyenne, plus qu'une sixte). La
force du violon gagne 7 points sur les temps forts : un peu plus « brillant » quand c'est positif.

| Mesure | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mesures en mode sombre (%) | 100,0 · 0,0 · 0,0 | 100,0 · 25,0 · 0,0 | 100,0 · 0,0 · 0,0 | 100,0 · 0,0 · 0,0 | 100,0 · 15,0 · 0,0 | 100,0 · 15,0 · 0,0 | 100,0 · 15,0 · 0,0 |
| Hauteur moyenne du violon (note MIDI ; 60 = do central) | 66,6 · 71,5 · 76,7 | 67,0 · 72,2 · 76,1 | 67,5 · 72,0 · 76,9 | 66,4 · 71,1 · 75,5 | 85,1 · 84,7 · 84,6 | 67,9 · 71,9 · 75,5 | 69,0 · 72,0 · 76,4 |
| Force du violon (1 à 127) | 89,4 · 92,3 · 95,2 | 89,2 · 91,7 · 94,7 | 89,2 · 92,6 · 96,3 | 92,4 · 97,6 · 103,3 | 96,4 · 101,1 · 103,2 | 88,6 · 91,2 · 94,3 | 92,2 · 95,9 · 100,8 |

**Plage utile.** Seuils : **sombre ≤ −0,3**, **clair ≥ 0,1** (marche : sombre < 0, clair > 0,2, changement à la partie de 8
mesures). Entre −0,3 et 0,1 (zone de maintien) rien ne bascule : la proportion de sombre reflète le mode de
l'air tiré. Plus on s'éloigne des seuils, plus le registre et la brillance suivent (continus). Effet **lent** :
le mode attend la fin de phrase (reel ≈ 4 s), la tonalité et le registre le début de l'air suivant. Pour une
transition audible immédiate, changer la valence **avant** le changement d'air.

**Situations possibles.** Village, auberge, victoire (+0,5 à +0,8) ; forêt hostile, ruines (−0,4 à −0,6) ;
défaite, deuil (−0,8). À lier au **lieu** ou à la **phase de l'histoire**, pas à des événements rapides.

## Density — la densité

**Ce que ça fait entendre.** À 0, le violon joue **seul**. Les instruments entrent un à un en montant : les
talons à 0,10, la basse à 0,15, la guitare à 0,25, les orteils à 0,35 (reel ; la marche garde ses pieds à toute
densité, de 5 à 14 frappes). Au neutre (0,5), l'arrangement normal. Au-dessus : les pieds passent de 8 à 12 frappes à 0,75, la guitare
double à 1, la basse « marche » plus, et le violon reçoit de **nouvelles notes de passage** (des notes
ajoutées entre deux notes écrites : de 0,6 à 1,6 par mesure en reel). La 2ᵉ voix varie un peu plus.

| Mesure | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| Coups de guitare par mesure | 0,00 · 7,97 · 15,93 | 0,00 · 7,97 · 15,93 | 0,00 · 7,97 · 15,93 | 0,00 · 8,00 · 16,00 | 0,00 · 0,00 · 0,00 | 0,00 · 7,97 · 21,90 | 0,00 · 7,97 · 12,20 |
| Notes de basse par mesure | 0,00 · 2,62 · 2,84 | 0,00 · 2,73 · 3,00 | 0,00 · 2,67 · 2,94 | 0,00 · 1,01 · 1,02 | 0,00 · 0,00 · 0,00 | 0,00 · 2,76 · 3,02 | 0,00 · 2,38 · 2,55 |
| Frappes de pieds par mesure | 0,00 · 7,97 · 11,95 | 0,00 · 7,97 · 11,95 | 0,00 · 5,98 · 7,97 | 0,00 · 3,00 · 5,00 | 5,00 · 9,50 · 14,00 | 0,00 · 0,00 · 0,00 | 0,00 · 0,00 · 0,00 |
| Notes du violon par mesure | 7,29 · 7,29 · 7,66 | 7,89 · 7,89 · 7,97 | 4,96 · 4,96 · 5,50 | 4,28 · 4,28 · 5,27 | 4,10 · 4,10 · 4,10 | 4,97 · 4,97 · 5,61 | 6,42 · 6,42 · 7,40 |
| Notes ajoutées par les ornements (par mesure) | 0,588 · 0,588 · 1,613 | 1,284 · 1,284 · 2,022 | 0,113 · 0,113 · 1,222 | 0,237 · 0,237 · 2,291 | 0,172 · 0,172 · 0,172 | 0,031 · 0,031 · 1,300 | 0,106 · 0,106 · 2,094 |
| Notes de passage de la basse par mesure | 0,000 · 0,627 · 0,845 | 0,000 · 0,739 · 1,000 | 0,000 · 0,678 · 0,942 | 0,000 · 0,011 · 0,017 | 0,000 · 0,000 · 0,000 | 0,000 · 0,769 · 1,025 | 0,000 · 0,381 · 0,558 |
| 2ᵉ voix différente du violon (% des notes) | 10,7 · 12,6 · 13,6 | 10,6 · 13,6 · 12,9 | 6,2 · 8,8 · 10,5 | 3,8 · 5,0 · 8,3 | 1,7 · 1,9 · 2,2 | — · — · — | — · — · — |

**Plage utile.** 0 à 0,35 : les entrées d'instruments (sensible au 0,05 près autour des seuils) ; 0,35 à 0,5 :
presque rien ; 0,5 à 1 : lent enrichissement (notes de passage), palier à 0,75 (pieds) et à 1 (guitare). Aucun effet
sur le tempo ni sur la force du violon.

**Situations possibles.** Nombre d'ennemis ou d'alliés à l'écran ; scène vide (0–0,1) contre bataille (0,8–1) ;
taille du groupe du joueur ; foule dans une zone.

## Tension — la nervosité

**Ce que ça fait entendre.** Sous son *genou* (0,3 ; marche 0,5), rien de notable ; au-dessus, les notes du violon
deviennent **courtes et sèches**, les **contretemps** (temps faibles) sont accentués, la guitare tient ses accords
plus longtemps (jusqu'à ×1,5) et l'harmonie « tire » vers l'accord de dominante au début des parties. La tension
pousse aussi un peu le tempo (+3 BPM) et la force (+18 à 1) : elle est donc partiellement de l'arousal
(*Recouvrements*). Sa vraie signature est la **durée des notes** : en reel, la note moyenne passe de 2,25 à 1,26
pas (deux fois plus courte), avec un **saut à environ 0,85** où les croches deviennent des doubles-croches.

| Mesure | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| Tempo (BPM) | 229,4 · 230,0 · 233,0 | 227,4 · 228,0 · 231,0 | 171,9 · 172,5 · 175,5 | 144,4 · 145,0 · 148,0 | 108,0 · 108,0 · 112,5 | 155,5 · 156,0 · 158,0 | 91,6 · 92,0 · 93,6 |
| Force du violon (1 à 127) | 89,4 · 92,3 · 109,4 | 89,0 · 91,7 · 108,2 | 89,4 · 92,6 · 113,8 | 94,9 · 97,6 · 114,0 | 100,4 · 101,1 · 113,0 | 88,6 · 91,2 · 105,4 | 93,5 · 95,9 · 107,5 |
| Durée moyenne d'une note (pas) | 2,26 · 2,25 · 1,26 | 2,13 · 2,12 · 1,21 | 2,50 · 2,44 · 1,36 | 3,07 · 3,00 · 1,86 | 3,97 · 3,96 · 3,10 | 2,43 · 2,43 · 2,26 | 2,50 · 2,50 · 2,25 |
| Part jouée de chaque note (‰) | 964 · 937 · 860 | 932 · 906 · 822 | 993 · 963 · 891 | 996 · 967 · 898 | 987 · 937 · 893 | 1000 · 970 · 898 | 994 · 964 · 898 |
| Notes de basse par mesure | 2,53 · 2,62 · 2,65 | 2,67 · 2,73 · 2,71 | 2,64 · 2,67 · 2,76 | 1,01 · 1,01 · 1,00 | 0,00 · 0,00 · 0,00 | 2,73 · 2,76 · 2,84 | 2,35 · 2,38 · 2,41 |

**Plage utile.** Du genou à 1 ; sous le genou, la musique est un peu plus ample (tension 0 : notes ×1,1 plus
longues, −0,6 BPM). Le saut de durée (≈ 0,85) est audible : prévoir de monter la tension par paliers ou
de passer vite de 0,8 à 0,9.

**Situations possibles.** Embuscade, danger imminent, approche d'un ennemi puissant, minuterie, vie basse.

## Recouvrements

Mesuré, variation moyenne de 0 à 1 sur les 7 styles :

| Ce qui change | arousal | valence | density | tension |
|---|---:|---:|---:|---:|
| Tempo (BPM) | **+22,7** | 0 | 0 | +3,3 |
| Force du violon | **+50,9** | +7,2 | +1,2 | +18,0 |
| Coups de guitare par mesure | +9,4 | 0 | **+14,0** | 0 |
| Frappes de pieds par mesure | +4,9 | 0 | **+6,6** | 0 |
| Notes de passage de basse | +0,32 | 0 | **+0,63** | +0,06 |
| Notes ajoutées par ornements (par mesure) | +0,06 | −0,01 | **+1,17** | 0 |
| Durée moyenne d'une note (pas) | −0,26 | 0 | −0,25 | **−0,80** |
| Hauteur moyenne du violon (MIDI) | 0 | **+7,5** | +0,2 | 0 |
| Part de mesures en mode sombre | 0 | **−100 %** | 0 | 0 |

- **arousal ≈ density en haut de l'échelle** : à 1, chacun double la guitare et ajoute 4 frappes de pied. Ils
  se séparent par le tempo et la force (arousal seul) et par les entrées d'instruments et les ornements
  (densité seule). En bas ils se distinguent nettement : arousal 0, c'est lent, doux et allégé ; densité 0, c'est le
  violon seul au même tempo et à la même force.
- **tension ≈ arousal sur la force** : la tension fait environ le tiers de ce que fait l'arousal pour le violon
  (+18 contre +51). Elle se reconnaît à la **brièveté des notes** et aux contretemps, pas au volume.
- **valence** ne ressemble à aucun autre : elle seule change mode, tonalité et registre.

## Conseils de combinaison

- **Ne pas brancher arousal et densité sur le même signal** : on n'entendrait qu'un seul bouton. Une
  association naturelle : arousal ← gravité de l'action (combat ou non) ; densité ← nombre d'acteurs
  présents.
- **Pour un danger silencieux** : tension haute, arousal bas. Le tempo reste calme, les notes deviennent courtes
  et nerveuses (scénario *danger*).
- **Pour un combat** : arousal et densité hauts (0,8), tension moyenne (0,5–0,6). C'est le scénario le plus
  proche de l'écrêtage (reel acadien −0,75 dBFS, marche −0,83 : voir le tableau, ligne *combat*).
- **La force du violon sature près de 127** (plafond MIDI) : arousal 1 seul donne déjà 118 en moyenne, *boss* et
  *tout au maximum* 121 à 126. Cumuler arousal très haut et tension haute n'ajoute plus rien d'audible sur la
  force ; ce qui les distingue alors, c'est le tempo (arousal) et la brièveté des notes (tension).
- **La valence se change lentement** (changement de lieu, de phase). La modifier pendant un combat ne s'entend
  qu'à la prochaine phrase pour le mode, au prochain air pour le reste.
- Les trois axes dynamiques (arousal, density, tension) s'entendent à la mesure suivante : on peut les piloter en
  continu, mais l'oreille ne distingue pas les écarts de moins de 0,1 sauf au passage d'un seuil.

Combinaisons mesurées (valeurs absolues ; la valence vit dans −1..1) :

| Scénario | arousal | valence | densité | tension |
|---|---:|---:|---:|---:|
| calme | 0,2 | 0,5 | 0,3 | 0,1 |
| combat | 0,8 | 0 | 0,8 | 0,6 |
| danger | 0,3 | -0,6 | 0,3 | 0,9 |
| boss | 1 | -0,5 | 1 | 0,8 |
| tout_au_maximum | 1 | 1 | 1 | 1 |
| tout_au_maximum_sombre | 1 | -1 | 1 | 1 |

Tempo (BPM) · force du violon (1 à 127) :

| Scénario | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 230 · 92 | 228 · 92 | 172 · 93 | 145 · 98 | 108 · 101 | 156 · 91 | 92 · 96 |
| calme | 224 · 75 | 222 · 74 | 166 · 75 | 139 · 79 | 101 · 85 | 151 · 76 | 88 · 80 |
| combat | 240 · 118 | 238 · 118 | 183 · 120 | 155 · 122 | 118 · 119 | 163 · 113 | 97 · 118 |
| danger | 228 · 93 | 226 · 92 | 170 · 96 | 143 · 95 | 106 · 95 | 154 · 92 | 91 · 93 |
| boss | 245 · 126 | 243 · 126 | 188 · 126 | 160 · 126 | 123 · 124 | 166 · 121 | 100 · 123 |
| tout_au_maximum | 245 · 125 | 243 · 123 | 188 · 126 | 160 · 125 | 123 · 124 | 166 · 123 | 100 · 125 |
| tout_au_maximum_sombre | 245 · 125 | 243 · 124 | 188 · 126 | 160 · 126 | 123 · 124 | 166 · 124 | 100 · 125 |

Coups de guitare · frappes de pieds, par mesure :

| Scénario | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 8,0 · 8,0 | 8,0 · 8,0 | 8,0 · 6,0 | 8,0 · 3,0 | 0,0 · 9,5 | 8,0 · 0,0 | 8,0 · 0,0 |
| calme | 4,0 · 2,0 | 4,0 · 2,0 | 4,0 · 2,0 | 4,0 · 1,0 | 0,0 · 5,0 | 4,0 · 0,0 | 4,0 · 0,0 |
| combat | 15,9 · 11,9 | 15,9 · 11,9 | 15,9 · 8,0 | 16,0 · 5,0 | 0,0 · 14,0 | 21,9 · 0,0 | 12,3 · 0,0 |
| danger | 4,0 · 2,0 | 4,0 · 2,0 | 4,0 · 2,0 | 4,0 · 1,0 | 0,0 · 9,5 | 8,0 · 0,0 | 8,0 · 0,0 |
| boss | 15,9 · 11,9 | 15,9 · 11,9 | 15,9 · 8,0 | 16,0 · 5,0 | 0,0 · 14,0 | 21,9 · 0,0 | 12,3 · 0,0 |
| tout_au_maximum | 15,9 · 11,9 | 15,9 · 11,9 | 15,9 · 8,0 | 16,0 · 5,0 | 0,0 · 14,0 | 21,9 · 0,0 | 12,3 · 0,0 |
| tout_au_maximum_sombre | 15,9 · 11,9 | 15,9 · 11,9 | 15,9 · 8,0 | 16,0 · 5,0 | 0,0 · 14,0 | 21,9 · 0,0 | 12,2 · 0,0 |

Mesures en mode sombre (%) · durée moyenne d'une note (pas) :

| Scénario | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 0 · 2,2 | 25 · 2,1 | 0 · 2,4 | 0 · 3,0 | 15 · 4,0 | 15 · 2,4 | 15 · 2,5 |
| calme | 0 · 2,2 | 10 · 2,2 | 0 · 2,5 | 0 · 3,0 | 15 · 4,1 | 0 · 2,5 | 0 · 2,4 |
| combat | 0 · 2,0 | 15 · 1,9 | 0 · 2,1 | 0 · 2,2 | 15 · 3,2 | 0 · 2,0 | 0 · 2,0 |
| danger | 100 · 2,1 | 100 · 2,0 | 100 · 2,3 | 100 · 2,6 | 100 · 3,2 | 100 · 2,3 | 100 · 2,3 |
| boss | 100 · 1,2 | 100 · 1,1 | 100 · 1,2 | 100 · 1,5 | 100 · 2,8 | 100 · 1,3 | 100 · 1,3 |
| tout_au_maximum | 0 · 1,2 | 0 · 1,1 | 0 · 1,2 | 0 · 1,6 | 0 · 2,6 | 0 · 1,2 | 0 · 1,3 |
| tout_au_maximum_sombre | 100 · 1,2 | 100 · 1,1 | 100 · 1,2 | 100 · 1,5 | 100 · 2,7 | 100 · 1,2 | 100 · 1,3 |

Crête maximale (dBFS) · échantillons écrêtés (maximum sur 20 graines) :

| Scénario | reel QC | reel AC | gigue QC | valse QC | marche GB | gigue GB | hornpipe GB |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | -1,8 · 0 | -1,7 · 0 | -2,2 · 0 | -2,2 · 0 | -3,3 · 0 | -3,0 · 0 | -4,0 · 0 |
| calme | -4,5 · 0 | -2,8 · 0 | -4,6 · 0 | -6,2 · 0 | -3,6 · 0 | -4,2 · 0 | -6,2 · 0 |
| combat | -0,5 · 0 | -0,8 · 0 | -1,7 · 0 | -3,3 · 0 | -0,8 · 0 | -1,0 · 0 | -1,8 · 0 |
| danger | -5,0 · 0 | -3,6 · 0 | -4,8 · 0 | -5,1 · 0 | -4,1 · 0 | -4,1 · 0 | -5,9 · 0 |
| boss | -2,7 · 0 | -1,8 · 0 | -2,4 · 0 | -3,6 · 0 | -0,9 · 0 | -1,9 · 0 | -2,2 · 0 |
| tout_au_maximum | -1,8 · 0 | -2,0 · 0 | -2,6 · 0 | -3,8 · 0 | -1,0 · 0 | -1,3 · 0 | -2,1 · 0 |
| tout_au_maximum_sombre | -2,2 · 0 | -1,9 · 0 | -2,9 · 0 | -3,5 · 0 | -0,9 · 0 | -2,3 · 0 | -2,6 · 0 |

## Limites connues (à traiter avant le jeu)

1. **Écrêtage** à arousal élevé : la gigue britannique à 1 (+0,87 dBFS, 35 échantillons au-dessus de 1 sur
   20 graines), le reel acadien à 0,75 (+0,11) et la valse à 1 (+0,02) dépassent 0 dBFS ; le reel québécois à
   0,75 est à −0,05 dBFS. `master_db` ne s'adapte pas à l'intensité. Les combinaisons mesurées restent sous 0 dBFS.
   Proposition P3 (ci-dessous).
2. **La caisse claire de la marche est muette** dans `harmonium_trad.sf2` : densité et arousal ne s'entendent
   donc pas aux pieds de la marche avec cette banque (ils sont dans le flux et le MIDI). Le comparateur d'écoute rend
   les pieds de la marche avec la banque GM.
3. **`smoothness` et `rhythm_tension`** ne sont pas des boutons : le jeu ne peut pas s'en servir.
4. **Pas de valeur de passage lisse** pour les textures : guitare et pieds changent par paliers (tableau de
   l'arousal). Pour une montée progressive, jouer sur le tempo et la force.

## Proposition pour mieux séparer les émotions (non appliquée)

Trois changements de **données** (aucun code), produits par `proposition_emotions.py` sur une copie
des profils ; le dépôt n'est pas modifié. La tâche J2 les applique après écoute.

1. **P1 — densité = qui joue, arousal = comment ça joue.** `density_feet` et `density_chop` à zéro : les pieds
   et le grattage suivent l'arousal seul. La densité garde les entrées d'instruments (0 → 0,35), les notes de
   passage et d'ornement, et la 2ᵉ voix. Contrepartie : de 0,5 à 1, la densité devient discrète (ornements et
   notes de passage, plus de guitare ni de pieds en plus) ; si c'est trop peu, ajouter des seuils de pistes
   au-dessus du neutre est un changement de données de même nature. La marche, sans seuils de pistes, n'aurait
   plus d'effet de densité sur les pieds : lui donner des seuils (`density_tracks` : Heel ≥ 0,10, Toe ≥ 0,35,
   comme le reel) fait partie de P1.
2. **P2 — tension moins forte.** `tension_offbeat` × 0,6 et `tension_intensity` × 0,5 : la tension fait moins
   de force et de tempo, et garde les notes courtes, les contretemps et les dominantes.
3. **P3 — plus d'écrêtage.** `master_db` : gigue britannique 16 → 14, valse 12 → 11, reel acadien 15,5 → 14,5 (seuls
   les trois styles qui écrêtent ; marge mesurée de 0,9 à 1,1 dBFS ; le neutre devient 1 à 2 dB plus bas).

Effet mesuré (mêmes 7 styles, 20 graines ; avant → **après**) :

| Variation du neutre à 1, moyenne sur les 7 styles | arousal | density | tension |
|---|---:|---:|---:|
| Tempo (BPM) | +13,3 → **+13,3** | +0,0 → **+0,0** | +2,9 → **+1,4** |
| Force du violon | +23,9 → **+23,9** | +1,2 → **+1,2** | +15,6 → **+8,6** |
| Coups de guitare par mesure | +7,2 → **+7,2** | +7,2 → **+0,0** | +0,0 → **+0,0** |
| Frappes de pieds par mesure | +2,4 → **+2,4** | +2,4 → **+0,0** | +0,0 → **+0,0** |
| Notes de passage de basse par mesure | +0,15 → **+0,15** | +0,17 → **+0,17** | +0,03 → **+0,04** |
| Notes ajoutées par ornements (par mesure) | +0,07 → **+0,07** | +1,17 → **+1,17** | +0,01 → **+0,02** |
| Durée moyenne d'une note (pas) | -0,24 → **-0,24** | -0,25 → **-0,25** | -0,77 → **-0,76** |

Crête, avant → après :

| Crête maximale sur 20 graines (dBFS) | arousal 1 | tout au maximum | neutre |
|---|---:|---:|---:|
| reel QC | -0,7 → **-0,7** | -1,8 → **-1,8** | -1,8 → **-1,8** |
| reel AC | -0,7 → **-1,7** | -2,0 → **-3,0** | -1,7 → **-2,7** |
| gigue QC | -1,6 → **-1,6** | -2,6 → **-2,7** | -2,2 → **-2,2** |
| valse QC | +0,0 → **-1,0** (1 → 0 écrêtés) | -3,8 → **-4,8** | -2,2 → **-3,2** |
| marche GB | -1,0 → **-1,0** | -1,0 → **-1,0** | -3,3 → **-3,3** |
| gigue GB | +0,9 → **-1,1** (35 → 0 écrêtés) | -1,3 → **-3,3** | -3,0 → **-5,0** |
| hornpipe GB | -0,8 → **-0,8** | -2,1 → **-2,1** | -4,0 → **-4,0** |

## Les écouter

`harmonium_lab/target/ecoute/emotions/comparateur.html` (kit `emotions` de `kits_ecoute.py --kits emotions` ;
à ouvrir par `python3 -m http.server 8765 --directory harmonium_lab/target/ecoute`, puis
`http://127.0.0.1:8765/emotions/comparateur.html`) : un air en
boucle, une émotion et un niveau (bas / neutre / haut) à choisir, quatre styles (reel, gigue, valse, marche).
Les versions ont le **même air et le même nombre de mesures** ; seul le tempo les rend inégales en durée, donc
une seule joue à la fois et la suivante reprend à la même mesure et à la même fraction de mesure (soit
au début de la mesure suivante, soit tout de suite). Le volume n'est pas égalisé (la force fait partie
de l'effet) ; une case permet de l'égaliser pour écouter le reste.

## Mots utilisés

- **Mesure** : un « tour » de la musique (en reel, ≈ 1 s ; en valse, ≈ 1,2 s). **Phrase** : 4 mesures. **Air** : la mélodie en
  cours (32 mesures par tour, plusieurs tours) ; un **set** enchaîne plusieurs airs.
- **BPM** : battements par minute, la vitesse.
- **Force / vélocité** : à quel point on appuie sur la note (1 à 127) ; plus c'est haut, plus c'est fort.
- **Pas** : la plus petite division du rythme ; en reel et gigue, une double-croche (2 pas = une croche).
- **Mode sombre / clair** : mineur (éolien, dorien) contre majeur (ionien, mixolydien).
- **Registre** : la hauteur moyenne de la mélodie ; **demi-ton** : la plus petite distance entre deux notes (12 par octave).
- **Notes de passage / ornements** : petites notes ajoutées entre ou autour des notes écrites.
- **dBFS** : niveau sonore ; 0 est le maximum du fichier, au-delà le son est écrêté (déformé).
