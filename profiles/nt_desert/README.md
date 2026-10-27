# `nt_desert` — Désert de *Nuclear Throne*

Profil de la zone désert (*Fläshyn*, Desert A ; bourdon de *A Way*, Desert B),
composés par Jukio Kallio (*Nuclear Throne OST*, Vlambeer, 2015). Premier profil
non trad du moteur, et premier profil de jeu : il n'ajoute aucun algorithme,
il compose ceux des tâches G1 à G5.

Corpus : `harmonium_lab`, `corpus/nuclear_throne/manifest.jsonl` (pistes 02 et
19 ; enregistrements, stems et transcriptions hors dépôt, droits réservés —
analyse et inspiration seulement). Dérivés employés : grilles §6 du compte
rendu `references/nt-desert/README.md` (batterie §6.2, basse §6.1, accords
§6.3), banque `assets/nt_desert.sf2` (`references/nt-desert/banque.md`) et
essais du propriétaire (`corpus/nuclear_throne/ecoute_g7/essai_v1..v3`, G7c).

## Ce que joue le profil

Arche de *Fläshyn* en 48 mesures par air (`sequence = [0, 1, 2, 3, 0]`) :

| Partie | Mesures | Harmonie (`loop`, G2) | Basse (motifs G3) | Dessus |
|---|---|---|---|---|
| A1 | 1–8 | pédale G# | G#1 tenu 3 mes. + relance D–E–F#–G–G# | nappe 89 liée (G#3, une attaque) |
| A2 | 9–16 | pédale de triton (D mineur) | riff D/D# (b2 phrygien) | batterie en groove, nappe liée |
| Pont | 17–32 | 16 accords tenus (C C E E F B C C / F C E E F B C# C#) | tue (`tracks_off`, G7b) | nappe 89 seule, sans batterie ni basse |
| B | 33–40 | G (2 mes.) / G# (2 mes.) ×2 | quasi-absente | nappe liée, batterie en groove |
| A1' | 41–48 | rejouée d'A1 | rejouée d'A1 | reprise littérale |

Pas de sifflement (G7c) : dans *Fläshyn* il n'y a pas de lead sifflé (il
appartient à *A Way*), et la transcription du motif de *A Way* est trop
incertaine (voiced_ratio ~7 %) pour justifier un motif rare. La mélodie n'a
aucune entrée d'arrangement (mécanisme G4) ; la banque de cellules reste pour
le plan interne.

Mètre 12/8, 24 pas par mesure, 210 noires/min (= 140 noires pointées/min).
Batterie en groove (G7c) : grosse caisse aux pas 0/8/20, caisse claire aux pas
2/10/14, charleston aux pulsations (pas 0/6/12/18) ; temps fort plein, autres
temps ×0,8 dans les motifs, accent du temps fort et micro-timing ±0,02 pas en
`[performance.tracks.Heel/Toe]`. Instruments : nappe chaude 89 (vélocité ×0,8,
tessiture médium, jamais sous MIDI 48), scie grave 81 sur la basse seule
(deux octaves plus bas, seule voix grave), kit 128/0 (touches 36, 38, 42).
Mix : batterie à −9/−10 dB (−4 dB vs G7) avec un peu plus de salle (envois
0,15). Seuls ces presets sont employés.

## Pilotage par les émotions

- **Arousal** : tempo (150–240 noires) et batterie (vélocité, densité 0→2).
- **Density** : couches (basse ≥ 0,25, pieds ≥ 0,3/0,35 ; à 0,2 ne reste que
  la nappe, façon bourdon de *A Way*).
- **Tension** : riff (contretemps, notes plus sèches), sans dominante
  substituée (boucle déclarée, G2).
- **Valence** : G# phrygien au neutre ; bourdon A phrygien en bas (calme
  façon *A Way*) ; G# mixolydien plus clair en haut (bascule G1).

## Écarts connus (voir les comptes rendus G7 et G7c)

1. A2 tient un D mineur (la superposition D + D# + A de l'original n'est pas
   une triade exprimable) ; chute F–C#–D de fin d'A1 omise (pédale pure).
2. Une seule nappe 89 pour la pédale et les triades (pas de pad par partie :
   programme unique par piste).
3. Pont de 16 mesures en un seul air de 48 (l'original fait 16 + reprise de
   10) ; seconde boucle C# de fin de pont incluse, coda omise.
4. G7c : mix et micro-timing réglés d'après les essais du propriétaire sur
   GeneralUser, à valider à l'oreille contre les deux banques.
