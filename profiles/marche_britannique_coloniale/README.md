# Profil de style : Marche britannique coloniale (fifre et tambour)

Style de garnison militaire britannique (vers 1760–1840) joué au **fifre et tambour**, sans guitare ni accords. Utile au jeu « 1837 » (soldats britanniques, garnison de Québec).

## Tradition et sources (Constitution V et VII)

- **Tradition** : Musique militaire britannique, régiments et garnisons d'Amérique du Nord, c. 1760–1840.
- **Corpus de référence** : 25 airs de marche du corpus `bascanada/harmonium_lab` issus de :
  - *Nottingham Music Database* (NMD)
  - *John Chambers AmRev* (AmRev, trillian)
  - *24 Country Dances for the Year 1809* (Cahusac, Londres)
- **Métrique retenue** : 2/2 (Alla Breve / Quick March), 8 croches par mesure (16 pas sur la grille de doubles-croches), pulse dansée à la blanche (2 pas cadencés par mesure : gauche-droite).
- **Tempo** : 108 BPM à la noire (valeur médiane observée), plage 90–126 BPM. Pas de swing (0.50, droit et militaire).

## Instrumentation

- **Fifre** : General MIDI 72 (Piccolo), tessiture 72–102 (D5 à F#7), centre mélodique à 84.
- **Tambour militaire** (General MIDI kit canal 9) :
  - Grosse caisse : note 36 sur piste Heel (temps forts, gauche-droite)
  - Caisse claire : note 38 sur piste Toe (battements d'ordonnance et roulements en 16ᵉ)
  - Caisse claire flam / coup accentué : note 40 sur piste Toe (conclusions cadentielles aux fins de phrase)
- **Harmonie** : Modèle `none` (aucune piste d'accords ni de basse). La mélodie reste tonale ; les cadences en m.4 et m.8 sont désormais pondérées plutôt qu'imposées.

## Banque de cellules mélodiques (`cells.toml`)

- 111 cellules au total, extraites des 16 airs en 2/2 et 4/4 :
  - `open` : 16 cellules
  - `run` : 30 cellules (top 30 parmi 43)
  - `arp` : 30 cellules (top 30 parmi 45)
  - `turn` : 4 cellules (seul rôle à moins de 15 cellules, ce qui reflète fidèlement la tradition militaire où les ornements en gruppetto sont très rares au profit des fanfares, arpèges et traits directs)
  - `cad_half` : 16 cellules
  - `cad_full` : 15 cellules
- Chaque entrée porte son poids d'occurrence observé (`weight`).

## Réaction aux 4 émotions (`dynamics.toml`)

- **Arousal** : accélère le tempo de marche (décalage jusqu'à +15 BPM) et renforce la dynamique du tambour (`feet_velocity`).
- **Density** : active le palier de roulements en doubles croches (16ᵉ) à la caisse claire (`density_feet`).
- **Tension** : renforce l'accentuation des frappes et des flams cadentiels (`strong_accent`, `tension_offbeat`).
- **Valence** : pilote la brillance et fait basculer le mode entre majeur (Ionien) et mineur militaire (Éolien) au prochain air (`color`, `parallel_modes`).

## Phrases probabilistes — DS-T9 / P3-T1

Source : `harmonium_lab/references/structure/britannique_coloniale-march.json`, champ `real`
(statistiques DS-T9, corpus et droits dans le manifeste du labo).
Les taux par mesure de `transformations` pilotent copies, séquences et fins variées ;
les déplacements utilisent les effectifs `sequence_shifts` de la même position.
Les repos toniques viennent de `phrases.rest_by_position` ; les autres longues
fins servent de proxy de demi-cadence. Voir [le schéma et les limites](../README.md#phrases--rôles-pondérés-motifs-et-cadences).

| Partie | Copie m.5←1 | Copie m.6←2 | Cadence tonique m.8 | Copie B:m.7←A:m.7 |
|---|---:|---:|---:|---:|
| A | 52.94 % | 29.41 % | 41.18 % | 0.00 % |
| B | 33.33 % | 16.67 % | 50.00 % | 47.06 % |

Poids arrondis sur 10 000 ; les mesures se tirent indépendamment.
Les taux conjoints m.5–6 et les structures de paires restent des mesures de
sortie, pas des garanties du profil. Banques de cellules inchangées.
