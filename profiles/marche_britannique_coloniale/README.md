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
- **Harmonie** : Modèle `none` (aucune piste d'accords ni de basse). La mélodie reste tonale et s'articule selon les cadences de forme (demi-cadence m.4 et cadence complète m.8).

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
