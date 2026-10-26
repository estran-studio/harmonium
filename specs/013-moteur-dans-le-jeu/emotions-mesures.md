# Mesures des 4 émotions (généré par `measure_emotions.py`)

7 profils × 20 graines × (neutre + 4 émotions × 5 niveaux + combinaisons), 66 mesures, 2 tours, `--automation` constante. Moyennes sur les graines ; l’écart type entre graines est dans `emotions-mesures.json` (régénéré par `measure_emotions.py`, non versionné). Une émotion bouge, les trois autres restent au neutre du profil :

| Profil | arousal | valence | densité | tension |
|---|---:|---:|---:|---:|
| reel_quebecois | 0.5 | 0.3 | 0.5 | 0.3 |
| reel_acadien | 0.5 | 0.3 | 0.5 | 0.3 |
| gigue_quebecoise | 0.5 | 0.3 | 0.5 | 0.3 |
| valse_quebecoise | 0.5 | 0.3 | 0.5 | 0.3 |
| marche_britannique_coloniale | 0.5 | 0.5 | 0.5 | 0.5 |
| gigue_britannique_coloniale | 0.5 | 0.3 | 0.5 | 0.3 |
| hornpipe_britannique_coloniale | 0.5 | 0.3 | 0.5 | 0.3 |

Colonnes = niveau de l'émotion (valence : −1, −0,5, 0, 0,5, 1) ; **neutre** = les quatre au neutre.

## Résumé : niveau 0 → niveau 1

Cellule = valeur au niveau 0 → valeur au niveau 1 (valence : −1 → +1) ; les trois autres axes au neutre du profil. Crête : maximum sur les graines.

### arousal

| Mesure | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| Tempo (BPM) | 220.0 → 245.0 | 218.0 → 243.0 | 162.5 → 187.5 | 135.0 → 160.0 | 96.0 → 123.0 | 148.0 → 166.0 | 86.0 → 100.0 |
| Vélocité moyenne, mélodie | 65.0 → 118.7 | 64.6 → 118.1 | 64.5 → 118.0 | 67.3 → 120.5 | 74.8 → 123.6 | 67.1 → 113.3 | 70.0 → 117.5 |
| Notes par mesure, mélodie | 7.38 → 7.42 | 7.77 → 7.99 | 5.14 → 4.95 | 4.57 → 3.97 | 4.22 → 4.14 | 5.08 → 5.02 | 6.60 → 6.51 |
| Notes par mesure, accords | 3.99 → 15.93 | 3.99 → 15.93 | 3.99 → 15.93 | 4.00 → 16.00 | 0.00 → 0.00 | 7.97 → 21.90 | 7.97 → 12.28 |
| Notes par mesure, basse | 2.37 → 2.84 | 2.42 → 2.95 | 2.52 → 2.97 | 1.00 → 1.01 | 0.00 → 0.00 | 2.44 → 2.90 | 2.22 → 2.55 |
| Densité de pieds (notes talon + orteil par mesure) | 2.00 → 11.95 | 2.00 → 11.95 | 2.00 → 7.97 | 1.00 → 5.00 | 9.50 → 14.00 | 0.00 → 0.00 | 0.00 → 0.00 |
| Part de mesures en mode sombre (%) | 0.0 → 0.0 | 25.0 → 25.0 | 0.0 → 0.0 | 0.0 → 0.0 | 15.0 → 15.0 | 15.0 → 15.0 | 15.0 → 15.0 |
| Hauteur moyenne de la mélodie (MIDI) | 71.8 → 72.0 | 72.2 → 72.2 | 71.9 → 72.2 | 71.1 → 70.6 | 85.1 → 85.1 | 71.7 → 71.9 | 72.2 → 72.1 |
| Notes Detached (% de la mélodie) | 0.0 → 0.1 | 0.0 → 0.0 | 0.0 → 0.0 | 0.5 → 0.9 | 0.0 → 0.0 | 0.0 → 0.0 | 0.0 → 0.0 |
| Ornements : notes ajoutées par mesure | 0.566 → 0.853 | 1.191 → 1.394 | 0.109 → 0.094 | 0.272 → 0.266 | 0.234 → 0.156 | 0.175 → 0.109 | 0.094 → 0.159 |
| Notes de passage de basse par mesure | 0.377 → 0.845 | 0.425 → 0.958 | 0.520 → 0.978 | 0.003 → 0.009 | 0.000 → 0.000 | 0.445 → 0.908 | 0.227 → 0.558 |
| Variations de la 2ᵉ voix (% des notes de la mélodie) | 8.4 → 18.6 | 9.2 → 17.6 | 5.3 → 10.2 | 4.1 → 5.9 | 2.8 → 3.1 | — → — | — → — |
| Crête du rendu (dBFS) | -7.30 → -0.74 | -5.05 → -0.70 | -7.55 → -1.63 | -8.98 → 0.02 | -5.83 → -1.00 | -4.71 → 0.87 | -8.05 → -0.76 |

### valence

| Mesure | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| Tempo (BPM) | 230.0 → 230.0 | 228.0 → 228.0 | 172.5 → 172.5 | 145.0 → 145.0 | 108.0 → 108.0 | 156.0 → 156.0 | 92.0 → 92.0 |
| Vélocité moyenne, mélodie | 89.4 → 95.2 | 89.2 → 94.7 | 89.2 → 96.3 | 92.4 → 103.3 | 96.4 → 103.2 | 88.6 → 94.3 | 92.2 → 100.8 |
| Notes par mesure, mélodie | 7.51 → 7.36 | 7.94 → 7.94 | 5.08 → 5.12 | 4.13 → 4.39 | 4.22 → 4.02 | 5.11 → 5.00 | 6.55 → 6.52 |
| Notes par mesure, accords | 7.97 → 7.97 | 7.97 → 7.97 | 7.97 → 7.97 | 8.00 → 8.00 | 0.00 → 0.00 | 7.97 → 7.97 | 7.97 → 7.97 |
| Notes par mesure, basse | 2.60 → 2.63 | 2.74 → 2.74 | 2.78 → 2.81 | 1.01 → 1.00 | 0.00 → 0.00 | 2.80 → 2.79 | 2.47 → 2.42 |
| Densité de pieds (notes talon + orteil par mesure) | 7.97 → 7.97 | 7.97 → 7.97 | 5.98 → 5.98 | 3.00 → 3.00 | 9.50 → 9.50 | 0.00 → 0.00 | 0.00 → 0.00 |
| Part de mesures en mode sombre (%) | 100.0 → 0.0 | 100.0 → 0.0 | 100.0 → 0.0 | 100.0 → 0.0 | 100.0 → 0.0 | 100.0 → 0.0 | 100.0 → 0.0 |
| Hauteur moyenne de la mélodie (MIDI) | 66.6 → 76.7 | 67.0 → 76.1 | 67.5 → 76.9 | 66.4 → 75.5 | 85.1 → 84.6 | 67.9 → 75.5 | 69.0 → 76.4 |
| Notes Detached (% de la mélodie) | 0.0 → 0.1 | 0.0 → 0.0 | 0.0 → 0.0 | 0.3 → 0.5 | 0.0 → 0.0 | 0.0 → 0.0 | 0.0 → 0.0 |
| Ornements : notes ajoutées par mesure | 0.819 → 0.613 | 1.197 → 1.447 | 0.097 → 0.081 | 0.234 → 0.200 | 0.219 → 0.156 | 0.025 → 0.025 | 0.150 → 0.131 |
| Notes de passage de basse par mesure | 0.609 → 0.639 | 0.748 → 0.742 | 0.783 → 0.814 | 0.013 → 0.003 | 0.000 → 0.000 | 0.809 → 0.794 | 0.472 → 0.423 |
| Variations de la 2ᵉ voix (% des notes de la mélodie) | 14.2 → 11.9 | 13.0 → 13.0 | 8.8 → 6.6 | 4.1 → 4.8 | 2.0 → 1.8 | — → — | — → — |
| Crête du rendu (dBFS) | -2.29 → -1.46 | -1.31 → -0.86 | -2.40 → -1.20 | -1.46 → -1.45 | -3.92 → -2.95 | -3.72 → -2.08 | -4.81 → -2.82 |

### density

| Mesure | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| Tempo (BPM) | 230.0 → 230.0 | 228.0 → 228.0 | 172.5 → 172.5 | 145.0 → 145.0 | 108.0 → 108.0 | 156.0 → 156.0 | 92.0 → 92.0 |
| Vélocité moyenne, mélodie | 92.3 → 93.7 | 91.7 → 93.6 | 92.6 → 94.0 | 97.6 → 99.4 | 101.1 → 101.1 | 91.2 → 92.4 | 95.9 → 96.9 |
| Notes par mesure, mélodie | 7.29 → 7.66 | 7.89 → 7.97 | 4.96 → 5.50 | 4.28 → 5.27 | 4.10 → 4.10 | 4.97 → 5.61 | 6.42 → 7.40 |
| Notes par mesure, accords | 0.00 → 15.93 | 0.00 → 15.93 | 0.00 → 15.93 | 0.00 → 16.00 | 0.00 → 0.00 | 0.00 → 21.90 | 0.00 → 12.20 |
| Notes par mesure, basse | 0.00 → 2.84 | 0.00 → 3.00 | 0.00 → 2.94 | 0.00 → 1.02 | 0.00 → 0.00 | 0.00 → 3.02 | 0.00 → 2.55 |
| Densité de pieds (notes talon + orteil par mesure) | 0.00 → 11.95 | 0.00 → 11.95 | 0.00 → 7.97 | 0.00 → 5.00 | 5.00 → 14.00 | 0.00 → 0.00 | 0.00 → 0.00 |
| Part de mesures en mode sombre (%) | 0.0 → 0.0 | 25.0 → 25.0 | 0.0 → 0.0 | 0.0 → 0.0 | 15.0 → 15.0 | 15.0 → 15.0 | 15.0 → 15.0 |
| Hauteur moyenne de la mélodie (MIDI) | 71.5 → 71.7 | 72.2 → 72.2 | 72.0 → 72.1 | 71.1 → 71.5 | 84.7 → 84.7 | 71.9 → 72.1 | 72.0 → 72.3 |
| Notes Detached (% de la mélodie) | 0.0 → 0.0 | 0.0 → 0.0 | 0.0 → 0.0 | 0.5 → 0.4 | 0.0 → 0.0 | 0.0 → 0.0 | 0.0 → 0.0 |
| Ornements : notes ajoutées par mesure | 0.588 → 1.613 | 1.284 → 2.022 | 0.113 → 1.222 | 0.237 → 2.291 | 0.172 → 0.172 | 0.031 → 1.300 | 0.106 → 2.094 |
| Notes de passage de basse par mesure | 0.000 → 0.845 | 0.000 → 1.000 | 0.000 → 0.942 | 0.000 → 0.017 | 0.000 → 0.000 | 0.000 → 1.025 | 0.000 → 0.558 |
| Variations de la 2ᵉ voix (% des notes de la mélodie) | 10.7 → 13.6 | 10.6 → 12.9 | 6.2 → 10.5 | 3.8 → 8.3 | 1.7 → 2.2 | — → — | — → — |
| Crête du rendu (dBFS) | -1.97 → -2.34 | -3.65 → -1.77 | -5.32 → -2.35 | -2.23 → -3.81 | -3.73 → -3.38 | -3.03 → -1.48 | -4.03 → -3.92 |

### tension

| Mesure | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| Tempo (BPM) | 229.4 → 233.0 | 227.4 → 231.0 | 171.9 → 175.5 | 144.4 → 148.0 | 108.0 → 112.5 | 155.5 → 158.0 | 91.6 → 93.6 |
| Vélocité moyenne, mélodie | 89.4 → 109.4 | 89.0 → 108.2 | 89.4 → 113.8 | 94.9 → 114.0 | 100.4 → 113.0 | 88.6 → 105.4 | 93.5 → 107.5 |
| Notes par mesure, mélodie | 7.30 → 7.41 | 7.90 → 8.03 | 4.96 → 5.04 | 4.28 → 4.23 | 4.10 → 4.15 | 4.97 → 4.99 | 6.42 → 6.42 |
| Notes par mesure, accords | 7.97 → 7.97 | 7.97 → 7.97 | 7.97 → 7.97 | 8.00 → 8.00 | 0.00 → 0.00 | 7.97 → 7.97 | 7.97 → 7.97 |
| Notes par mesure, basse | 2.53 → 2.65 | 2.67 → 2.71 | 2.64 → 2.76 | 1.01 → 1.00 | 0.00 → 0.00 | 2.73 → 2.84 | 2.35 → 2.41 |
| Densité de pieds (notes talon + orteil par mesure) | 7.97 → 7.97 | 7.97 → 7.97 | 5.98 → 5.98 | 3.00 → 3.00 | 9.50 → 9.50 | 0.00 → 0.00 | 0.00 → 0.00 |
| Part de mesures en mode sombre (%) | 0.0 → 0.0 | 25.0 → 25.0 | 0.0 → 0.0 | 0.0 → 0.0 | 15.0 → 15.0 | 15.0 → 15.0 | 15.0 → 15.0 |
| Hauteur moyenne de la mélodie (MIDI) | 71.5 → 72.0 | 72.2 → 72.0 | 72.0 → 71.8 | 71.1 → 71.0 | 84.7 → 84.8 | 71.9 → 71.9 | 72.0 → 72.2 |
| Notes Detached (% de la mélodie) | 0.0 → 0.0 | 0.0 → 0.0 | 0.0 → 0.0 | 0.5 → 0.6 | 0.0 → 0.0 | 0.0 → 0.0 | 0.0 → 0.0 |
| Ornements : notes ajoutées par mesure | 0.600 → 0.634 | 1.297 → 1.306 | 0.113 → 0.128 | 0.237 → 0.203 | 0.172 → 0.094 | 0.031 → 0.047 | 0.106 → 0.178 |
| Notes de passage de basse par mesure | 0.537 → 0.655 | 0.678 → 0.719 | 0.644 → 0.766 | 0.011 → 0.000 | 0.000 → 0.000 | 0.731 → 0.844 | 0.356 → 0.414 |
| Variations de la 2ᵉ voix (% des notes de la mélodie) | 15.8 → 15.0 | 16.2 → 14.9 | 11.1 → 7.6 | 9.3 → 3.7 | 4.9 → 2.0 | — → — | — → — |
| Crête du rendu (dBFS) | -1.57 → -4.11 | -1.82 → -2.56 | -2.64 → -3.48 | -2.55 → -3.00 | -2.45 → -2.22 | -3.36 → -2.39 | -4.39 → -3.43 |

## Qui fait bouger quoi

Variation moyenne (niveau 1 − niveau 0) sur les profils où la mesure existe.

| Mesure | arousal | valence | density | tension |
|---|---:|---:|---:|---:|
| Tempo (BPM) | +22.7 | +0.0 | +0.0 | +3.3 |
| Vélocité moyenne, mélodie | +50.9 | +7.2 | +1.2 | +18.0 |
| Vélocité moyenne, toutes pistes | +22.2 | +3.0 | -21.1 | +8.7 |
| Vélocité moyenne, accords | +7.7 | -0.0 | -44.2 | -1.2 |
| Vélocité moyenne, pieds | +27.1 | +0.0 | -36.4 | +5.5 |
| Notes par mesure, mélodie | -0.11 | -0.02 | +0.51 | +0.05 |
| Notes par mesure, accords | +9.44 | +0.00 | +13.98 | +0.00 |
| Notes par mesure, basse | +0.32 | -0.00 | +2.19 | +0.06 |
| Notes par mesure, talon | +0.57 | +0.00 | +1.56 | +0.00 |
| Notes par mesure, orteil | +4.34 | +0.00 | +4.99 | +0.00 |
| Notes par mesure, 2ᵉ voix | -0.21 | +0.00 | +0.13 | +0.15 |
| Hauteur moyenne de la mélodie (MIDI) | +0.0 | +7.5 | +0.2 | +0.1 |
| Part de mesures en mode sombre (%) | +0.0 | -100.0 | +0.0 | +0.0 |
| Notes Detached (% de la mélodie) | +0.1 | +0.0 | -0.0 | +0.0 |
| Notes Linked (% de la mélodie) | +0.1 | -0.0 | -0.1 | +0.0 |
| Notes Accent (% de la mélodie) | +0.0 | +0.0 | -0.1 | -0.1 |
| Gate moyen de la mélodie (‰) | -6 | +1 | +14 | -101 |
| Durée moyenne d'une note de mélodie (pas) | -0.26 | -0.00 | -0.25 | -0.80 |
| Ornements : notes ajoutées par mesure | +0.056 | -0.012 | +1.169 | +0.005 |
| Notes de passage de basse par mesure | +0.323 | -0.003 | +0.627 | +0.063 |
| Variations de la 2ᵉ voix (% des notes de la mélodie) | +5.2 | -0.8 | +2.9 | -2.8 |
| Densité de pieds (notes talon + orteil par mesure) | +4.91 | +0.00 | +6.55 | +0.00 |
| Crête du rendu (dBFS) | +6.22 | +1.01 | +0.70 | -0.34 |
| Échantillons écrêtés (>= 1) | +0.3 | +0.0 | +0.0 | +0.0 |

## Combinaisons

| Scénario | arousal | valence | densité | tension |
|---|---:|---:|---:|---:|
| calme | 0.2 | 0.5 | 0.3 | 0.1 |
| combat | 0.8 | 0 | 0.8 | 0.6 |
| danger | 0.3 | -0.6 | 0.3 | 0.9 |
| boss | 1 | -0.5 | 1 | 0.8 |
| tout_au_maximum | 1 | 1 | 1 | 1 |
| tout_au_maximum_sombre | 1 | -1 | 1 | 1 |

Crête et écrêtés : maximum sur les graines ; les autres mesures : moyennes.

### Tempo (BPM)

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 230.0 | 228.0 | 172.5 | 145.0 | 108.0 | 156.0 | 92.0 |
| calme | 223.6 | 221.6 | 166.1 | 138.6 | 100.8 | 150.9 | 88.2 |
| combat | 240.3 | 238.3 | 182.8 | 155.3 | 117.9 | 162.9 | 97.5 |
| danger | 227.7 | 225.7 | 170.2 | 142.7 | 106.1 | 154.2 | 90.6 |
| boss | 245.0 | 243.0 | 187.5 | 160.0 | 123.0 | 166.0 | 100.0 |
| tout_au_maximum | 245.0 | 243.0 | 187.5 | 160.0 | 123.0 | 166.0 | 100.0 |
| tout_au_maximum_sombre | 245.0 | 243.0 | 187.5 | 160.0 | 123.0 | 166.0 | 100.0 |

### Vélocité moyenne, mélodie

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 92.3 | 91.7 | 92.6 | 97.6 | 101.1 | 91.2 | 95.9 |
| calme | 74.6 | 74.4 | 74.7 | 78.8 | 84.7 | 75.7 | 80.1 |
| combat | 118.2 | 117.9 | 119.6 | 122.2 | 119.3 | 113.1 | 117.8 |
| danger | 93.4 | 92.2 | 96.3 | 94.5 | 94.6 | 92.0 | 93.0 |
| boss | 126.1 | 125.7 | 126.1 | 125.8 | 124.5 | 120.9 | 122.8 |
| tout_au_maximum | 124.7 | 123.1 | 126.0 | 125.3 | 124.4 | 123.4 | 124.6 |
| tout_au_maximum_sombre | 124.6 | 123.9 | 126.2 | 125.5 | 124.5 | 123.5 | 124.5 |

### Notes par mesure, accords

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 7.97 | 7.97 | 7.97 | 8.00 | 0.00 | 7.97 | 7.97 |
| calme | 3.99 | 3.99 | 3.99 | 4.00 | 0.00 | 3.99 | 3.99 |
| combat | 15.93 | 15.93 | 15.93 | 16.00 | 0.00 | 21.90 | 12.32 |
| danger | 3.99 | 3.99 | 3.99 | 4.00 | 0.00 | 7.97 | 7.97 |
| boss | 15.93 | 15.93 | 15.93 | 16.00 | 0.00 | 21.90 | 12.28 |
| tout_au_maximum | 15.93 | 15.93 | 15.93 | 16.00 | 0.00 | 21.90 | 12.27 |
| tout_au_maximum_sombre | 15.93 | 15.93 | 15.93 | 16.00 | 0.00 | 21.90 | 12.22 |

### Densité de pieds (notes talon + orteil par mesure)

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 7.97 | 7.97 | 5.98 | 3.00 | 9.50 | 0.00 | 0.00 |
| calme | 2.00 | 2.00 | 2.00 | 1.00 | 5.00 | 0.00 | 0.00 |
| combat | 11.95 | 11.95 | 7.97 | 5.00 | 14.00 | 0.00 | 0.00 |
| danger | 2.00 | 2.00 | 2.00 | 1.00 | 9.50 | 0.00 | 0.00 |
| boss | 11.95 | 11.95 | 7.97 | 5.00 | 14.00 | 0.00 | 0.00 |
| tout_au_maximum | 11.95 | 11.95 | 7.97 | 5.00 | 14.00 | 0.00 | 0.00 |
| tout_au_maximum_sombre | 11.95 | 11.95 | 7.97 | 5.00 | 14.00 | 0.00 | 0.00 |

### Part de mesures en mode sombre (%)

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 0.0 | 25.0 | 0.0 | 0.0 | 15.0 | 15.0 | 15.0 |
| calme | 0.0 | 10.0 | 0.0 | 0.0 | 15.0 | 0.0 | 0.0 |
| combat | 0.0 | 15.0 | 0.0 | 0.0 | 15.0 | 0.0 | 0.0 |
| danger | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 |
| boss | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 |
| tout_au_maximum | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| tout_au_maximum_sombre | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 | 100.0 |

### Durée moyenne d'une note de mélodie (pas)

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 2.25 | 2.12 | 2.44 | 3.00 | 3.96 | 2.43 | 2.50 |
| calme | 2.21 | 2.23 | 2.52 | 2.98 | 4.07 | 2.46 | 2.44 |
| combat | 1.99 | 1.90 | 2.05 | 2.21 | 3.19 | 1.99 | 2.00 |
| danger | 2.08 | 1.99 | 2.26 | 2.57 | 3.17 | 2.26 | 2.28 |
| boss | 1.20 | 1.15 | 1.24 | 1.52 | 2.78 | 1.25 | 1.28 |
| tout_au_maximum | 1.18 | 1.13 | 1.21 | 1.57 | 2.55 | 1.22 | 1.26 |
| tout_au_maximum_sombre | 1.17 | 1.12 | 1.24 | 1.54 | 2.67 | 1.24 | 1.25 |

### Ornements : notes ajoutées par mesure

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 0.588 | 1.284 | 0.113 | 0.237 | 0.172 | 0.031 | 0.106 |
| calme | 0.872 | 1.269 | 0.188 | 0.263 | 0.172 | 0.075 | 0.175 |
| combat | 1.519 | 2.178 | 1.247 | 1.844 | 0.109 | 1.494 | 1.834 |
| danger | 0.509 | 1.269 | 0.053 | 0.266 | 0.188 | 0.044 | 0.066 |
| boss | 1.766 | 1.753 | 1.212 | 2.581 | 0.172 | 1.528 | 2.281 |
| tout_au_maximum | 0.944 | 1.619 | 0.441 | 0.887 | 0.219 | 1.312 | 2.281 |
| tout_au_maximum_sombre | 0.916 | 1.216 | 0.531 | 0.900 | 0.188 | 1.472 | 2.188 |

### Crête du rendu (dBFS)

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | -1.84 | -1.72 | -2.25 | -2.21 | -3.31 | -3.03 | -4.03 |
| calme | -4.54 | -2.76 | -4.58 | -6.16 | -3.58 | -4.22 | -6.25 |
| combat | -0.53 | -0.75 | -1.74 | -3.32 | -0.83 | -1.01 | -1.80 |
| danger | -5.03 | -3.58 | -4.81 | -5.10 | -4.06 | -4.09 | -5.86 |
| boss | -2.70 | -1.81 | -2.44 | -3.64 | -0.92 | -1.86 | -2.19 |
| tout_au_maximum | -1.83 | -1.95 | -2.56 | -3.76 | -0.99 | -1.33 | -2.07 |
| tout_au_maximum_sombre | -2.22 | -1.89 | -2.90 | -3.54 | -0.93 | -2.33 | -2.64 |

### Échantillons écrêtés (>= 1)

| Scénario | reel_quebecois | reel_acadien | gigue_quebecoise | valse_quebecoise | marche_britannique_coloniale | gigue_britannique_coloniale | hornpipe_britannique_coloniale |
|---|---:|---:|---:|---:|---:|---:|---:|
| neutre | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| calme | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| combat | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| danger | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| boss | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| tout_au_maximum | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| tout_au_maximum_sombre | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

La **crête** est le maximum sur les graines ; toutes les autres colonnes sont des moyennes.

## arousal

### arousal — Tempo (BPM)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 230.0 | 220.0 | 225.0 | 230.0 | 237.5 | 245.0 |
| reel_acadien | 228.0 | 218.0 | 223.0 | 228.0 | 235.5 | 243.0 |
| gigue_quebecoise | 172.5 | 162.5 | 167.5 | 172.5 | 180.0 | 187.5 |
| valse_quebecoise | 145.0 | 135.0 | 140.0 | 145.0 | 152.5 | 160.0 |
| marche_britannique_coloniale | 108.0 | 96.0 | 102.0 | 108.0 | 115.5 | 123.0 |
| gigue_britannique_coloniale | 156.0 | 148.0 | 152.0 | 156.0 | 161.0 | 166.0 |
| hornpipe_britannique_coloniale | 92.0 | 86.0 | 89.0 | 92.0 | 96.0 | 100.0 |

### arousal — Vélocité moyenne, mélodie

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 92.3 | 65.0 | 78.7 | 92.3 | 108.6 | 118.7 |
| reel_acadien | 91.7 | 64.6 | 78.0 | 91.7 | 108.4 | 118.1 |
| gigue_quebecoise | 92.6 | 64.5 | 78.5 | 92.6 | 109.5 | 118.0 |
| valse_quebecoise | 97.6 | 67.3 | 82.1 | 97.6 | 115.4 | 120.5 |
| marche_britannique_coloniale | 101.1 | 74.8 | 87.8 | 101.1 | 116.5 | 123.6 |
| gigue_britannique_coloniale | 91.2 | 67.1 | 79.6 | 91.2 | 104.3 | 113.3 |
| hornpipe_britannique_coloniale | 95.9 | 70.0 | 83.3 | 95.9 | 109.7 | 117.5 |

### arousal — Vélocité moyenne, toutes pistes

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 75.3 | 58.9 | 63.3 | 75.3 | 78.3 | 79.0 |
| reel_acadien | 75.6 | 58.8 | 63.8 | 75.6 | 79.1 | 79.9 |
| gigue_quebecoise | 74.4 | 57.0 | 62.8 | 74.4 | 78.0 | 76.1 |
| valse_quebecoise | 79.6 | 59.4 | 66.7 | 79.6 | 81.2 | 74.7 |
| marche_britannique_coloniale | 90.1 | 63.0 | 76.4 | 90.1 | 106.8 | 117.4 |
| gigue_britannique_coloniale | 72.1 | 52.9 | 62.5 | 72.1 | 73.6 | 60.5 |
| hornpipe_britannique_coloniale | 75.9 | 55.4 | 65.9 | 75.9 | 79.0 | 73.1 |

### arousal — Vélocité moyenne, accords

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 74.0 | 48.1 | 61.1 | 74.0 | 66.7 | 56.8 |
| reel_acadien | 74.0 | 48.1 | 61.2 | 74.0 | 66.6 | 56.8 |
| gigue_quebecoise | 74.0 | 48.1 | 61.1 | 74.0 | 66.7 | 56.8 |
| valse_quebecoise | 74.0 | 48.0 | 61.1 | 74.0 | 66.7 | 56.8 |
| marche_britannique_coloniale | — | — | — | — | — | — |
| gigue_britannique_coloniale | 62.1 | 43.5 | 52.7 | 62.1 | 57.3 | 47.6 |
| hornpipe_britannique_coloniale | 62.1 | 43.5 | 52.7 | 62.1 | 57.4 | 50.8 |

### arousal — Vélocité moyenne, pieds

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 45.7 | 30.3 | 36.4 | 45.7 | 50.5 | 62.9 |
| reel_acadien | 45.6 | 30.0 | 36.5 | 45.6 | 50.4 | 62.8 |
| gigue_quebecoise | 44.1 | 30.1 | 35.5 | 44.1 | 51.3 | 64.3 |
| valse_quebecoise | 43.8 | 30.1 | 35.0 | 43.8 | 47.3 | 58.9 |
| marche_britannique_coloniale | 80.3 | 52.3 | 66.3 | 80.3 | 98.0 | 113.7 |
| gigue_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |
| hornpipe_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |

### arousal — Notes par mesure, mélodie

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.29 | 7.38 | 7.17 | 7.29 | 7.40 | 7.42 |
| reel_acadien | 7.89 | 7.77 | 7.99 | 7.89 | 7.91 | 7.99 |
| gigue_quebecoise | 4.96 | 5.14 | 5.14 | 4.96 | 5.07 | 4.95 |
| valse_quebecoise | 4.28 | 4.57 | 4.45 | 4.28 | 4.25 | 3.97 |
| marche_britannique_coloniale | 4.10 | 4.22 | 4.10 | 4.10 | 4.15 | 4.14 |
| gigue_britannique_coloniale | 4.97 | 5.08 | 4.92 | 4.97 | 4.92 | 5.02 |
| hornpipe_britannique_coloniale | 6.42 | 6.60 | 6.61 | 6.42 | 6.56 | 6.51 |

### arousal — Notes par mesure, accords

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 3.99 | 7.97 | 7.97 | 7.97 | 15.93 |
| reel_acadien | 7.97 | 3.99 | 7.97 | 7.97 | 7.97 | 15.93 |
| gigue_quebecoise | 7.97 | 3.99 | 7.97 | 7.97 | 7.97 | 15.93 |
| valse_quebecoise | 8.00 | 4.00 | 8.00 | 8.00 | 8.00 | 16.00 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 21.90 |
| hornpipe_britannique_coloniale | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 12.28 |

### arousal — Notes par mesure, basse

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.62 | 2.37 | 2.54 | 2.62 | 2.75 | 2.84 |
| reel_acadien | 2.73 | 2.42 | 2.59 | 2.73 | 2.82 | 2.95 |
| gigue_quebecoise | 2.67 | 2.52 | 2.68 | 2.67 | 2.93 | 2.97 |
| valse_quebecoise | 1.01 | 1.00 | 1.01 | 1.01 | 1.01 | 1.01 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 2.76 | 2.44 | 2.62 | 2.76 | 2.86 | 2.90 |
| hornpipe_britannique_coloniale | 2.38 | 2.22 | 2.33 | 2.38 | 2.49 | 2.55 |

### arousal — Notes par mesure, talon

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.99 | 2.00 | 3.99 | 3.99 | 3.99 | 3.99 |
| reel_acadien | 3.99 | 2.00 | 3.99 | 3.99 | 3.99 | 3.99 |
| gigue_quebecoise | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| valse_quebecoise | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| marche_britannique_coloniale | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### arousal — Notes par mesure, orteil

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.98 | 0.00 | 3.98 | 3.98 | 7.96 | 7.96 |
| reel_acadien | 3.98 | 0.00 | 3.98 | 3.98 | 7.96 | 7.96 |
| gigue_quebecoise | 3.98 | 0.00 | 3.98 | 3.98 | 5.97 | 5.97 |
| valse_quebecoise | 2.00 | 0.00 | 2.00 | 2.00 | 4.00 | 4.00 |
| marche_britannique_coloniale | 7.50 | 7.50 | 7.50 | 7.50 | 7.50 | 12.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### arousal — Notes par mesure, 2ᵉ voix

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 6.71 | 6.99 | 6.71 | 6.71 | 6.60 | 6.54 |
| reel_acadien | 7.28 | 7.43 | 7.61 | 7.28 | 7.31 | 7.30 |
| gigue_quebecoise | 4.69 | 4.96 | 4.93 | 4.69 | 4.73 | 4.65 |
| valse_quebecoise | 4.19 | 4.44 | 4.35 | 4.19 | 4.16 | 3.88 |
| marche_britannique_coloniale | 4.06 | 4.14 | 4.05 | 4.06 | 4.09 | 4.09 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### arousal — Hauteur moyenne de la mélodie (MIDI)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 71.5 | 71.8 | 71.5 | 71.5 | 72.1 | 72.0 |
| reel_acadien | 72.2 | 72.2 | 72.3 | 72.2 | 72.1 | 72.2 |
| gigue_quebecoise | 72.0 | 71.9 | 72.0 | 72.0 | 72.2 | 72.2 |
| valse_quebecoise | 71.1 | 71.1 | 70.7 | 71.1 | 71.0 | 70.6 |
| marche_britannique_coloniale | 84.7 | 85.1 | 84.7 | 84.7 | 84.8 | 85.1 |
| gigue_britannique_coloniale | 71.9 | 71.7 | 72.1 | 71.9 | 71.9 | 71.9 |
| hornpipe_britannique_coloniale | 72.0 | 72.2 | 72.4 | 72.0 | 72.1 | 72.1 |

### arousal — Part de mesures en mode sombre (%)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 25.0 | 25.0 | 25.0 | 25.0 | 25.0 | 25.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| marche_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |
| gigue_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |
| hornpipe_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |

### arousal — Notes Detached (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.1 | 0.1 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.5 | 0.5 | 0.4 | 0.5 | 0.7 | 0.9 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.1 | 0.0 | 0.1 | 0.0 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### arousal — Notes Linked (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.3 | 0.3 | 0.3 | 0.3 | 0.5 | 0.3 |
| reel_acadien | 0.3 | 0.3 | 0.4 | 0.3 | 0.4 | 0.5 |
| gigue_quebecoise | 0.6 | 0.6 | 0.7 | 0.6 | 0.6 | 0.6 |
| valse_quebecoise | 1.5 | 1.4 | 1.5 | 1.5 | 1.6 | 1.9 |
| marche_britannique_coloniale | 2.2 | 2.1 | 2.2 | 2.2 | 2.2 | 2.2 |
| gigue_britannique_coloniale | 0.2 | 0.2 | 0.3 | 0.2 | 0.3 | 0.3 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### arousal — Notes Accent (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 1.0 | 1.2 | 1.2 | 1.0 | 1.0 | 1.1 |
| reel_acadien | 1.2 | 1.3 | 0.9 | 1.2 | 1.1 | 1.0 |
| gigue_quebecoise | 2.1 | 1.2 | 1.7 | 2.1 | 1.9 | 1.4 |
| valse_quebecoise | 1.1 | 0.7 | 0.7 | 1.1 | 0.9 | 1.2 |
| marche_britannique_coloniale | 1.8 | 1.8 | 1.8 | 1.8 | 1.7 | 1.8 |
| gigue_britannique_coloniale | 1.7 | 1.7 | 1.5 | 1.7 | 1.6 | 1.5 |
| hornpipe_britannique_coloniale | 2.0 | 2.2 | 2.3 | 2.0 | 2.4 | 2.0 |

### arousal — Gate moyen de la mélodie (‰)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 937 | 949 | 949 | 937 | 928 | 923 |
| reel_acadien | 906 | 922 | 908 | 906 | 904 | 895 |
| gigue_quebecoise | 963 | 962 | 967 | 963 | 962 | 965 |
| valse_quebecoise | 967 | 967 | 963 | 967 | 965 | 966 |
| marche_britannique_coloniale | 937 | 933 | 937 | 937 | 944 | 934 |
| gigue_britannique_coloniale | 970 | 955 | 968 | 970 | 968 | 962 |
| hornpipe_britannique_coloniale | 964 | 965 | 965 | 964 | 963 | 967 |

### arousal — Durée moyenne d'une note de mélodie (pas)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.25 | 2.30 | 2.28 | 2.25 | 2.22 | 2.12 |
| reel_acadien | 2.12 | 2.24 | 2.09 | 2.12 | 2.12 | 1.98 |
| gigue_quebecoise | 2.44 | 2.45 | 2.35 | 2.44 | 2.38 | 2.30 |
| valse_quebecoise | 3.00 | 3.00 | 2.96 | 3.00 | 2.98 | 2.87 |
| marche_britannique_coloniale | 3.96 | 3.97 | 3.97 | 3.96 | 3.79 | 3.24 |
| gigue_britannique_coloniale | 2.43 | 2.44 | 2.46 | 2.43 | 2.47 | 2.26 |
| hornpipe_britannique_coloniale | 2.50 | 2.44 | 2.43 | 2.50 | 2.43 | 2.26 |

### arousal — Ornements : notes ajoutées par mesure

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.588 | 0.566 | 0.441 | 0.588 | 0.859 | 0.853 |
| reel_acadien | 1.284 | 1.191 | 1.369 | 1.284 | 1.337 | 1.394 |
| gigue_quebecoise | 0.113 | 0.109 | 0.066 | 0.113 | 0.122 | 0.094 |
| valse_quebecoise | 0.237 | 0.272 | 0.287 | 0.237 | 0.269 | 0.266 |
| marche_britannique_coloniale | 0.172 | 0.234 | 0.172 | 0.172 | 0.094 | 0.156 |
| gigue_britannique_coloniale | 0.031 | 0.175 | 0.034 | 0.031 | 0.050 | 0.109 |
| hornpipe_britannique_coloniale | 0.106 | 0.094 | 0.094 | 0.106 | 0.113 | 0.159 |

### arousal — Notes de passage de basse par mesure

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.627 | 0.377 | 0.541 | 0.627 | 0.759 | 0.845 |
| reel_acadien | 0.739 | 0.425 | 0.594 | 0.739 | 0.828 | 0.958 |
| gigue_quebecoise | 0.678 | 0.520 | 0.688 | 0.678 | 0.934 | 0.978 |
| valse_quebecoise | 0.011 | 0.003 | 0.006 | 0.011 | 0.009 | 0.009 |
| marche_britannique_coloniale | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| gigue_britannique_coloniale | 0.769 | 0.445 | 0.623 | 0.769 | 0.869 | 0.908 |
| hornpipe_britannique_coloniale | 0.381 | 0.227 | 0.333 | 0.381 | 0.492 | 0.558 |

### arousal — Variations de la 2ᵉ voix (% des notes de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 12.6 | 8.4 | 10.6 | 12.6 | 16.2 | 18.6 |
| reel_acadien | 13.6 | 9.2 | 10.4 | 13.6 | 15.2 | 17.6 |
| gigue_quebecoise | 8.8 | 5.3 | 5.8 | 8.8 | 9.9 | 10.2 |
| valse_quebecoise | 5.0 | 4.1 | 4.7 | 5.0 | 5.9 | 5.9 |
| marche_britannique_coloniale | 1.9 | 2.8 | 1.7 | 1.9 | 2.0 | 3.1 |
| gigue_britannique_coloniale | — | — | — | — | — | — |
| hornpipe_britannique_coloniale | — | — | — | — | — | — |

### arousal — Densité de pieds (notes talon + orteil par mesure)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 2.00 | 7.97 | 7.97 | 11.95 | 11.95 |
| reel_acadien | 7.97 | 2.00 | 7.97 | 7.97 | 11.95 | 11.95 |
| gigue_quebecoise | 5.98 | 2.00 | 5.98 | 5.98 | 7.97 | 7.97 |
| valse_quebecoise | 3.00 | 1.00 | 3.00 | 3.00 | 5.00 | 5.00 |
| marche_britannique_coloniale | 9.50 | 9.50 | 9.50 | 9.50 | 9.50 | 14.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### arousal — Crête du rendu (dBFS)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | -3.37 | -7.30 | -4.13 | -1.84 | -0.05 | -0.74 |
| reel_acadien | -3.12 | -5.05 | -4.05 | -1.72 | 0.11 | -0.70 |
| gigue_quebecoise | -3.20 | -7.55 | -5.05 | -2.25 | -0.87 | -1.63 |
| valse_quebecoise | -4.51 | -8.98 | -5.56 | -2.21 | -1.51 | 0.02 |
| marche_britannique_coloniale | -3.88 | -5.83 | -5.35 | -3.31 | -1.34 | -1.00 |
| gigue_britannique_coloniale | -3.81 | -4.71 | -4.70 | -3.03 | -0.60 | 0.87 |
| hornpipe_britannique_coloniale | -5.12 | -8.05 | -6.27 | -4.03 | -2.00 | -0.76 |

### arousal — Échantillons écrêtés (>= 1)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.1 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.1 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 1.8 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

## valence

### valence — Tempo (BPM)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 230.0 | 230.0 | 230.0 | 230.0 | 230.0 | 230.0 |
| reel_acadien | 228.0 | 228.0 | 228.0 | 228.0 | 228.0 | 228.0 |
| gigue_quebecoise | 172.5 | 172.5 | 172.5 | 172.5 | 172.5 | 172.5 |
| valse_quebecoise | 145.0 | 145.0 | 145.0 | 145.0 | 145.0 | 145.0 |
| marche_britannique_coloniale | 108.0 | 108.0 | 108.0 | 108.0 | 108.0 | 108.0 |
| gigue_britannique_coloniale | 156.0 | 156.0 | 156.0 | 156.0 | 156.0 | 156.0 |
| hornpipe_britannique_coloniale | 92.0 | 92.0 | 92.0 | 92.0 | 92.0 | 92.0 |

### valence — Vélocité moyenne, mélodie

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 92.3 | 89.4 | 90.7 | 91.8 | 93.0 | 95.2 |
| reel_acadien | 91.7 | 89.2 | 89.9 | 90.7 | 92.2 | 94.7 |
| gigue_quebecoise | 92.6 | 89.2 | 90.5 | 91.8 | 93.7 | 96.3 |
| valse_quebecoise | 97.6 | 92.4 | 94.2 | 96.0 | 99.5 | 103.3 |
| marche_britannique_coloniale | 101.1 | 96.4 | 98.0 | 99.0 | 101.1 | 103.2 |
| gigue_britannique_coloniale | 91.2 | 88.6 | 89.5 | 90.8 | 91.8 | 94.3 |
| hornpipe_britannique_coloniale | 95.9 | 92.2 | 93.5 | 95.2 | 96.9 | 100.8 |

### valence — Vélocité moyenne, toutes pistes

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 75.3 | 74.2 | 74.7 | 75.1 | 75.8 | 76.6 |
| reel_acadien | 75.6 | 74.5 | 74.9 | 75.4 | 76.2 | 77.1 |
| gigue_quebecoise | 74.4 | 73.2 | 73.7 | 74.0 | 74.8 | 76.0 |
| valse_quebecoise | 79.6 | 77.2 | 78.3 | 79.1 | 80.4 | 82.3 |
| marche_britannique_coloniale | 90.1 | 88.0 | 88.8 | 89.2 | 90.1 | 91.0 |
| gigue_britannique_coloniale | 72.1 | 71.4 | 71.5 | 71.9 | 72.4 | 73.1 |
| hornpipe_britannique_coloniale | 75.9 | 74.5 | 75.0 | 75.6 | 76.5 | 77.8 |

### valence — Vélocité moyenne, accords

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 74.0 | 74.1 | 74.1 | 74.0 | 74.1 | 74.0 |
| reel_acadien | 74.0 | 74.0 | 73.9 | 74.0 | 74.0 | 74.0 |
| gigue_quebecoise | 74.0 | 74.0 | 74.1 | 74.0 | 74.0 | 73.9 |
| valse_quebecoise | 74.0 | 73.9 | 74.0 | 74.0 | 74.0 | 74.0 |
| marche_britannique_coloniale | — | — | — | — | — | — |
| gigue_britannique_coloniale | 62.1 | 62.0 | 62.0 | 62.0 | 62.1 | 62.1 |
| hornpipe_britannique_coloniale | 62.1 | 62.0 | 62.0 | 62.0 | 62.1 | 62.0 |

### valence — Vélocité moyenne, pieds

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 45.7 | 45.7 | 45.6 | 45.7 | 45.6 | 45.3 |
| reel_acadien | 45.6 | 45.5 | 45.5 | 45.5 | 45.6 | 45.8 |
| gigue_quebecoise | 44.1 | 44.0 | 44.2 | 44.0 | 44.1 | 44.1 |
| valse_quebecoise | 43.8 | 44.2 | 44.1 | 43.9 | 43.9 | 44.4 |
| marche_britannique_coloniale | 80.3 | 80.3 | 80.4 | 80.4 | 80.3 | 80.4 |
| gigue_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |
| hornpipe_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |

### valence — Notes par mesure, mélodie

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.29 | 7.51 | 7.35 | 7.28 | 7.45 | 7.36 |
| reel_acadien | 7.89 | 7.94 | 8.10 | 8.24 | 8.23 | 7.94 |
| gigue_quebecoise | 4.96 | 5.08 | 5.03 | 4.98 | 5.01 | 5.12 |
| valse_quebecoise | 4.28 | 4.13 | 4.30 | 4.33 | 4.30 | 4.39 |
| marche_britannique_coloniale | 4.10 | 4.22 | 4.18 | 4.13 | 4.10 | 4.02 |
| gigue_britannique_coloniale | 4.97 | 5.11 | 4.94 | 4.94 | 5.07 | 5.00 |
| hornpipe_britannique_coloniale | 6.42 | 6.55 | 6.52 | 6.48 | 6.58 | 6.52 |

### valence — Notes par mesure, accords

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| reel_acadien | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| gigue_quebecoise | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| valse_quebecoise | 8.00 | 8.00 | 8.00 | 8.00 | 8.00 | 8.00 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| hornpipe_britannique_coloniale | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |

### valence — Notes par mesure, basse

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.62 | 2.60 | 2.55 | 2.61 | 2.66 | 2.63 |
| reel_acadien | 2.73 | 2.74 | 2.66 | 2.76 | 2.73 | 2.74 |
| gigue_quebecoise | 2.67 | 2.78 | 2.75 | 2.78 | 2.75 | 2.81 |
| valse_quebecoise | 1.01 | 1.01 | 1.01 | 1.01 | 1.01 | 1.00 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 2.76 | 2.80 | 2.78 | 2.82 | 2.78 | 2.79 |
| hornpipe_britannique_coloniale | 2.38 | 2.47 | 2.37 | 2.38 | 2.36 | 2.42 |

### valence — Notes par mesure, talon

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 |
| reel_acadien | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 |
| gigue_quebecoise | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| valse_quebecoise | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| marche_britannique_coloniale | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### valence — Notes par mesure, orteil

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 |
| reel_acadien | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 |
| gigue_quebecoise | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 |
| valse_quebecoise | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| marche_britannique_coloniale | 7.50 | 7.50 | 7.50 | 7.50 | 7.50 | 7.50 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### valence — Notes par mesure, 2ᵉ voix

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 6.71 | 6.81 | 6.67 | 6.69 | 6.70 | 6.75 |
| reel_acadien | 7.28 | 7.45 | 7.48 | 7.46 | 7.61 | 7.40 |
| gigue_quebecoise | 4.69 | 4.78 | 4.83 | 4.69 | 4.74 | 4.87 |
| valse_quebecoise | 4.19 | 4.05 | 4.19 | 4.24 | 4.21 | 4.29 |
| marche_britannique_coloniale | 4.06 | 4.18 | 4.15 | 4.10 | 4.06 | 3.97 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### valence — Hauteur moyenne de la mélodie (MIDI)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 71.5 | 66.6 | 68.8 | 70.6 | 72.6 | 76.7 |
| reel_acadien | 72.2 | 67.0 | 69.3 | 71.8 | 73.4 | 76.1 |
| gigue_quebecoise | 72.0 | 67.5 | 69.5 | 71.3 | 73.2 | 76.9 |
| valse_quebecoise | 71.1 | 66.4 | 68.5 | 70.1 | 72.5 | 75.5 |
| marche_britannique_coloniale | 84.7 | 85.1 | 84.9 | 84.9 | 84.7 | 84.6 |
| gigue_britannique_coloniale | 71.9 | 67.9 | 69.4 | 70.9 | 72.6 | 75.5 |
| hornpipe_britannique_coloniale | 72.0 | 69.0 | 70.6 | 71.7 | 73.2 | 76.4 |

### valence — Part de mesures en mode sombre (%)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 100.0 | 100.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 25.0 | 100.0 | 100.0 | 15.0 | 10.0 | 0.0 |
| gigue_quebecoise | 0.0 | 100.0 | 100.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 100.0 | 100.0 | 0.0 | 0.0 | 0.0 |
| marche_britannique_coloniale | 15.0 | 100.0 | 100.0 | 15.0 | 15.0 | 0.0 |
| gigue_britannique_coloniale | 15.0 | 100.0 | 100.0 | 0.0 | 0.0 | 0.0 |
| hornpipe_britannique_coloniale | 15.0 | 100.0 | 100.0 | 0.0 | 0.0 | 0.0 |

### valence — Notes Detached (% de la mélodie)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.1 | 0.1 | 0.0 | 0.1 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.5 | 0.3 | 0.5 | 0.4 | 0.6 | 0.5 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.1 | 0.0 | 0.0 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### valence — Notes Linked (% de la mélodie)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.3 | 0.2 | 0.4 | 0.4 | 0.2 | 0.5 |
| reel_acadien | 0.3 | 0.4 | 0.3 | 0.3 | 0.4 | 0.2 |
| gigue_quebecoise | 0.6 | 0.7 | 0.9 | 0.8 | 0.4 | 0.8 |
| valse_quebecoise | 1.5 | 1.5 | 1.5 | 1.4 | 1.7 | 1.8 |
| marche_britannique_coloniale | 2.2 | 2.4 | 2.1 | 2.0 | 2.2 | 2.0 |
| gigue_britannique_coloniale | 0.2 | 0.3 | 0.3 | 0.3 | 0.2 | 0.2 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### valence — Notes Accent (% de la mélodie)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 1.0 | 1.1 | 1.2 | 1.3 | 1.0 | 1.1 |
| reel_acadien | 1.2 | 0.9 | 1.1 | 0.9 | 0.9 | 0.9 |
| gigue_quebecoise | 2.1 | 2.0 | 1.3 | 1.7 | 1.2 | 1.5 |
| valse_quebecoise | 1.1 | 1.1 | 1.2 | 0.7 | 0.9 | 1.2 |
| marche_britannique_coloniale | 1.8 | 1.8 | 2.0 | 1.6 | 1.8 | 1.6 |
| gigue_britannique_coloniale | 1.7 | 1.6 | 1.8 | 1.8 | 2.0 | 1.7 |
| hornpipe_britannique_coloniale | 2.0 | 2.2 | 2.4 | 2.0 | 2.1 | 2.8 |

### valence — Gate moyen de la mélodie (‰)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 937 | 930 | 942 | 944 | 932 | 938 |
| reel_acadien | 906 | 915 | 898 | 886 | 891 | 905 |
| gigue_quebecoise | 963 | 960 | 964 | 964 | 964 | 965 |
| valse_quebecoise | 967 | 965 | 966 | 966 | 966 | 967 |
| marche_britannique_coloniale | 937 | 934 | 938 | 938 | 937 | 939 |
| gigue_britannique_coloniale | 970 | 969 | 967 | 969 | 967 | 969 |
| hornpipe_britannique_coloniale | 964 | 962 | 962 | 965 | 961 | 962 |

### valence — Durée moyenne d'une note de mélodie (pas)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.25 | 2.21 | 2.25 | 2.24 | 2.23 | 2.23 |
| reel_acadien | 2.12 | 2.09 | 2.08 | 2.05 | 2.04 | 2.12 |
| gigue_quebecoise | 2.44 | 2.39 | 2.41 | 2.43 | 2.42 | 2.36 |
| valse_quebecoise | 3.00 | 3.16 | 2.97 | 2.98 | 2.97 | 2.90 |
| marche_britannique_coloniale | 3.96 | 3.87 | 3.88 | 3.94 | 3.96 | 4.03 |
| gigue_britannique_coloniale | 2.43 | 2.36 | 2.47 | 2.45 | 2.38 | 2.41 |
| hornpipe_britannique_coloniale | 2.50 | 2.46 | 2.47 | 2.48 | 2.45 | 2.46 |

### valence — Ornements : notes ajoutées par mesure

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.588 | 0.819 | 0.656 | 0.581 | 0.781 | 0.613 |
| reel_acadien | 1.284 | 1.197 | 1.484 | 1.703 | 1.762 | 1.447 |
| gigue_quebecoise | 0.113 | 0.097 | 0.059 | 0.078 | 0.109 | 0.081 |
| valse_quebecoise | 0.237 | 0.234 | 0.263 | 0.291 | 0.200 | 0.200 |
| marche_britannique_coloniale | 0.172 | 0.219 | 0.172 | 0.188 | 0.172 | 0.156 |
| gigue_britannique_coloniale | 0.031 | 0.025 | 0.078 | 0.050 | 0.037 | 0.025 |
| hornpipe_britannique_coloniale | 0.106 | 0.150 | 0.144 | 0.094 | 0.159 | 0.131 |

### valence — Notes de passage de basse par mesure

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.627 | 0.609 | 0.550 | 0.616 | 0.664 | 0.639 |
| reel_acadien | 0.739 | 0.748 | 0.667 | 0.764 | 0.738 | 0.742 |
| gigue_quebecoise | 0.678 | 0.783 | 0.752 | 0.789 | 0.755 | 0.814 |
| valse_quebecoise | 0.011 | 0.013 | 0.011 | 0.006 | 0.011 | 0.003 |
| marche_britannique_coloniale | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| gigue_britannique_coloniale | 0.769 | 0.809 | 0.784 | 0.820 | 0.784 | 0.794 |
| hornpipe_britannique_coloniale | 0.381 | 0.472 | 0.377 | 0.388 | 0.364 | 0.423 |

### valence — Variations de la 2ᵉ voix (% des notes de la mélodie)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 12.6 | 14.2 | 13.6 | 13.5 | 14.6 | 11.9 |
| reel_acadien | 13.6 | 13.0 | 14.9 | 15.2 | 13.9 | 13.0 |
| gigue_quebecoise | 8.8 | 8.8 | 6.9 | 8.7 | 8.1 | 6.6 |
| valse_quebecoise | 5.0 | 4.1 | 5.2 | 4.9 | 4.9 | 4.8 |
| marche_britannique_coloniale | 1.9 | 2.0 | 1.7 | 1.7 | 1.9 | 1.8 |
| gigue_britannique_coloniale | — | — | — | — | — | — |
| hornpipe_britannique_coloniale | — | — | — | — | — | — |

### valence — Densité de pieds (notes talon + orteil par mesure)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| reel_acadien | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| gigue_quebecoise | 5.98 | 5.98 | 5.98 | 5.98 | 5.98 | 5.98 |
| valse_quebecoise | 3.00 | 3.00 | 3.00 | 3.00 | 3.00 | 3.00 |
| marche_britannique_coloniale | 9.50 | 9.50 | 9.50 | 9.50 | 9.50 | 9.50 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### valence — Crête du rendu (dBFS)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | -3.37 | -2.29 | -1.78 | -1.48 | -0.84 | -1.46 |
| reel_acadien | -3.12 | -1.31 | -1.51 | -1.27 | -0.63 | -0.86 |
| gigue_quebecoise | -3.20 | -2.40 | -3.23 | -2.31 | -1.90 | -1.20 |
| valse_quebecoise | -4.51 | -1.46 | -3.54 | -3.27 | -3.20 | -1.45 |
| marche_britannique_coloniale | -3.88 | -3.92 | -3.64 | -3.72 | -3.31 | -2.95 |
| gigue_britannique_coloniale | -3.81 | -3.72 | -2.06 | -3.42 | -3.18 | -2.08 |
| hornpipe_britannique_coloniale | -5.12 | -4.81 | -4.91 | -4.31 | -3.53 | -2.82 |

### valence — Échantillons écrêtés (>= 1)

| Profil | neutre | -1 | -0.5 | +0 | +0.5 | +1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

## density

### density — Tempo (BPM)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 230.0 | 230.0 | 230.0 | 230.0 | 230.0 | 230.0 |
| reel_acadien | 228.0 | 228.0 | 228.0 | 228.0 | 228.0 | 228.0 |
| gigue_quebecoise | 172.5 | 172.5 | 172.5 | 172.5 | 172.5 | 172.5 |
| valse_quebecoise | 145.0 | 145.0 | 145.0 | 145.0 | 145.0 | 145.0 |
| marche_britannique_coloniale | 108.0 | 108.0 | 108.0 | 108.0 | 108.0 | 108.0 |
| gigue_britannique_coloniale | 156.0 | 156.0 | 156.0 | 156.0 | 156.0 | 156.0 |
| hornpipe_britannique_coloniale | 92.0 | 92.0 | 92.0 | 92.0 | 92.0 | 92.0 |

### density — Vélocité moyenne, mélodie

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 92.3 | 92.3 | 92.3 | 92.3 | 93.1 | 93.7 |
| reel_acadien | 91.7 | 91.7 | 91.7 | 91.7 | 92.8 | 93.6 |
| gigue_quebecoise | 92.6 | 92.6 | 92.6 | 92.6 | 93.5 | 94.0 |
| valse_quebecoise | 97.6 | 97.6 | 97.6 | 97.6 | 99.1 | 99.4 |
| marche_britannique_coloniale | 101.1 | 101.1 | 101.1 | 101.1 | 101.1 | 101.1 |
| gigue_britannique_coloniale | 91.2 | 91.2 | 91.2 | 91.2 | 91.9 | 92.4 |
| hornpipe_britannique_coloniale | 95.9 | 95.9 | 95.9 | 95.9 | 96.4 | 96.9 |

### density — Vélocité moyenne, toutes pistes

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 75.3 | 93.0 | 80.2 | 75.3 | 70.0 | 70.0 |
| reel_acadien | 75.6 | 92.4 | 80.4 | 75.6 | 70.7 | 70.4 |
| gigue_quebecoise | 74.4 | 93.3 | 80.5 | 74.4 | 71.6 | 71.1 |
| valse_quebecoise | 79.6 | 97.9 | 83.8 | 79.6 | 76.4 | 74.8 |
| marche_britannique_coloniale | 90.1 | 94.5 | 90.1 | 90.1 | 87.6 | 87.6 |
| gigue_britannique_coloniale | 72.1 | 91.3 | 72.4 | 72.1 | 72.6 | 63.7 |
| hornpipe_britannique_coloniale | 75.9 | 95.9 | 76.0 | 75.9 | 76.7 | 73.1 |

### density — Vélocité moyenne, accords

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 74.0 | 110.0 | 74.0 | 74.0 | 74.0 | 71.0 |
| reel_acadien | 74.0 | 110.0 | 74.0 | 74.0 | 74.0 | 71.0 |
| gigue_quebecoise | 74.0 | 110.0 | 74.0 | 74.0 | 74.0 | 71.0 |
| valse_quebecoise | 74.0 | — | 74.0 | 74.0 | 74.0 | 71.0 |
| marche_britannique_coloniale | — | — | — | — | — | — |
| gigue_britannique_coloniale | 62.1 | 110.0 | 62.1 | 62.1 | 62.1 | 56.1 |
| hornpipe_britannique_coloniale | 62.1 | 110.0 | 62.1 | 62.1 | 62.1 | 59.9 |

### density — Vélocité moyenne, pieds

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 45.7 | 110.0 | 49.8 | 45.7 | 38.1 | 38.1 |
| reel_acadien | 45.6 | 110.0 | 50.0 | 45.6 | 38.1 | 38.1 |
| gigue_quebecoise | 44.1 | 110.0 | 49.9 | 44.1 | 38.9 | 38.9 |
| valse_quebecoise | 43.8 | — | 49.9 | 43.8 | 35.5 | 35.5 |
| marche_britannique_coloniale | 80.3 | 83.1 | 80.3 | 80.3 | 79.5 | 79.5 |
| gigue_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |
| hornpipe_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |

### density — Notes par mesure, mélodie

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.29 | 7.29 | 7.29 | 7.29 | 7.45 | 7.66 |
| reel_acadien | 7.89 | 7.89 | 7.89 | 7.89 | 8.08 | 7.97 |
| gigue_quebecoise | 4.96 | 4.96 | 4.96 | 4.96 | 5.34 | 5.50 |
| valse_quebecoise | 4.28 | 4.28 | 4.28 | 4.28 | 5.01 | 5.27 |
| marche_britannique_coloniale | 4.10 | 4.10 | 4.10 | 4.10 | 4.10 | 4.10 |
| gigue_britannique_coloniale | 4.97 | 4.97 | 4.97 | 4.97 | 5.34 | 5.61 |
| hornpipe_britannique_coloniale | 6.42 | 6.42 | 6.42 | 6.42 | 7.00 | 7.40 |

### density — Notes par mesure, accords

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 0.00 | 7.97 | 7.97 | 7.97 | 15.93 |
| reel_acadien | 7.97 | 0.00 | 7.97 | 7.97 | 7.97 | 15.93 |
| gigue_quebecoise | 7.97 | 0.00 | 7.97 | 7.97 | 7.97 | 15.93 |
| valse_quebecoise | 8.00 | 0.00 | 8.00 | 8.00 | 8.00 | 16.00 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 7.97 | 0.00 | 7.97 | 7.97 | 7.97 | 21.90 |
| hornpipe_britannique_coloniale | 7.97 | 0.00 | 7.97 | 7.97 | 7.97 | 12.20 |

### density — Notes par mesure, basse

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.62 | 0.00 | 2.38 | 2.62 | 2.74 | 2.84 |
| reel_acadien | 2.73 | 0.00 | 2.48 | 2.73 | 2.88 | 3.00 |
| gigue_quebecoise | 2.67 | 0.00 | 2.40 | 2.67 | 2.82 | 2.94 |
| valse_quebecoise | 1.01 | 0.00 | 1.00 | 1.01 | 1.01 | 1.02 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 2.76 | 0.00 | 2.48 | 2.76 | 2.91 | 3.02 |
| hornpipe_britannique_coloniale | 2.38 | 0.00 | 2.22 | 2.38 | 2.46 | 2.55 |

### density — Notes par mesure, talon

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.99 | 0.00 | 3.99 | 3.99 | 3.99 | 3.99 |
| reel_acadien | 3.99 | 0.00 | 3.99 | 3.99 | 3.99 | 3.99 |
| gigue_quebecoise | 2.00 | 0.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| valse_quebecoise | 1.00 | 0.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| marche_britannique_coloniale | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### density — Notes par mesure, orteil

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.98 | 0.00 | 0.00 | 3.98 | 7.96 | 7.96 |
| reel_acadien | 3.98 | 0.00 | 0.00 | 3.98 | 7.96 | 7.96 |
| gigue_quebecoise | 3.98 | 0.00 | 0.00 | 3.98 | 5.97 | 5.97 |
| valse_quebecoise | 2.00 | 0.00 | 0.00 | 2.00 | 4.00 | 4.00 |
| marche_britannique_coloniale | 7.50 | 3.00 | 7.50 | 7.50 | 12.00 | 12.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### density — Notes par mesure, 2ᵉ voix

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 6.71 | 6.80 | 6.78 | 6.71 | 6.76 | 6.86 |
| reel_acadien | 7.28 | 7.47 | 7.38 | 7.28 | 7.36 | 7.30 |
| gigue_quebecoise | 4.69 | 4.77 | 4.73 | 4.69 | 4.97 | 5.07 |
| valse_quebecoise | 4.19 | 4.23 | 4.20 | 4.19 | 4.75 | 4.95 |
| marche_britannique_coloniale | 4.06 | 4.05 | 4.06 | 4.06 | 4.05 | 4.05 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### density — Hauteur moyenne de la mélodie (MIDI)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 71.5 | 71.5 | 71.5 | 71.5 | 71.6 | 71.7 |
| reel_acadien | 72.2 | 72.2 | 72.2 | 72.2 | 72.2 | 72.2 |
| gigue_quebecoise | 72.0 | 72.0 | 72.0 | 72.0 | 72.1 | 72.1 |
| valse_quebecoise | 71.1 | 71.1 | 71.1 | 71.1 | 71.4 | 71.5 |
| marche_britannique_coloniale | 84.7 | 84.7 | 84.7 | 84.7 | 84.7 | 84.7 |
| gigue_britannique_coloniale | 71.9 | 71.9 | 71.9 | 71.9 | 72.0 | 72.1 |
| hornpipe_britannique_coloniale | 72.0 | 72.0 | 72.0 | 72.0 | 72.2 | 72.3 |

### density — Part de mesures en mode sombre (%)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 25.0 | 25.0 | 25.0 | 25.0 | 25.0 | 25.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| marche_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |
| gigue_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |
| hornpipe_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |

### density — Notes Detached (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.5 | 0.5 | 0.5 | 0.5 | 0.4 | 0.4 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### density — Notes Linked (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 |
| reel_acadien | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 |
| gigue_quebecoise | 0.6 | 0.6 | 0.6 | 0.6 | 0.6 | 0.5 |
| valse_quebecoise | 1.5 | 1.5 | 1.5 | 1.5 | 1.3 | 1.2 |
| marche_britannique_coloniale | 2.2 | 2.2 | 2.2 | 2.2 | 2.2 | 2.2 |
| gigue_britannique_coloniale | 0.2 | 0.2 | 0.2 | 0.2 | 0.1 | 0.1 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### density — Notes Accent (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 | 0.9 |
| reel_acadien | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 |
| gigue_quebecoise | 2.1 | 2.1 | 2.1 | 2.1 | 2.0 | 1.9 |
| valse_quebecoise | 1.1 | 1.1 | 1.1 | 1.1 | 0.9 | 0.9 |
| marche_britannique_coloniale | 1.8 | 1.8 | 1.8 | 1.8 | 1.8 | 1.8 |
| gigue_britannique_coloniale | 1.7 | 1.7 | 1.7 | 1.7 | 1.6 | 1.5 |
| hornpipe_britannique_coloniale | 2.0 | 2.0 | 2.0 | 2.0 | 1.8 | 1.7 |

### density — Gate moyen de la mélodie (‰)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 937 | 937 | 937 | 937 | 953 | 962 |
| reel_acadien | 906 | 906 | 906 | 906 | 926 | 952 |
| gigue_quebecoise | 963 | 963 | 963 | 963 | 966 | 968 |
| valse_quebecoise | 967 | 967 | 967 | 967 | 973 | 975 |
| marche_britannique_coloniale | 937 | 937 | 937 | 937 | 937 | 937 |
| gigue_britannique_coloniale | 970 | 970 | 970 | 970 | 972 | 973 |
| hornpipe_britannique_coloniale | 964 | 964 | 964 | 964 | 969 | 974 |

### density — Durée moyenne d'une note de mélodie (pas)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.25 | 2.25 | 2.25 | 2.25 | 2.18 | 2.10 |
| reel_acadien | 2.12 | 2.12 | 2.12 | 2.12 | 2.05 | 2.03 |
| gigue_quebecoise | 2.44 | 2.44 | 2.44 | 2.44 | 2.26 | 2.19 |
| valse_quebecoise | 3.00 | 3.00 | 3.00 | 3.00 | 2.48 | 2.34 |
| marche_britannique_coloniale | 3.96 | 3.96 | 3.96 | 3.96 | 3.96 | 3.96 |
| gigue_britannique_coloniale | 2.43 | 2.43 | 2.43 | 2.43 | 2.26 | 2.15 |
| hornpipe_britannique_coloniale | 2.50 | 2.50 | 2.50 | 2.50 | 2.29 | 2.16 |

### density — Ornements : notes ajoutées par mesure

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.588 | 0.588 | 0.588 | 0.588 | 1.072 | 1.613 |
| reel_acadien | 1.284 | 1.284 | 1.284 | 1.284 | 1.894 | 2.022 |
| gigue_quebecoise | 0.113 | 0.113 | 0.113 | 0.113 | 0.875 | 1.222 |
| valse_quebecoise | 0.237 | 0.237 | 0.237 | 0.237 | 1.756 | 2.291 |
| marche_britannique_coloniale | 0.172 | 0.172 | 0.172 | 0.172 | 0.172 | 0.172 |
| gigue_britannique_coloniale | 0.031 | 0.031 | 0.031 | 0.031 | 0.756 | 1.300 |
| hornpipe_britannique_coloniale | 0.106 | 0.106 | 0.106 | 0.106 | 1.262 | 2.094 |

### density — Notes de passage de basse par mesure

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.627 | 0.000 | 0.380 | 0.627 | 0.748 | 0.845 |
| reel_acadien | 0.739 | 0.000 | 0.483 | 0.739 | 0.884 | 1.000 |
| gigue_quebecoise | 0.678 | 0.000 | 0.406 | 0.678 | 0.828 | 0.942 |
| valse_quebecoise | 0.011 | 0.000 | 0.002 | 0.011 | 0.014 | 0.017 |
| marche_britannique_coloniale | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| gigue_britannique_coloniale | 0.769 | 0.000 | 0.481 | 0.769 | 0.919 | 1.025 |
| hornpipe_britannique_coloniale | 0.381 | 0.000 | 0.228 | 0.381 | 0.461 | 0.558 |

### density — Variations de la 2ᵉ voix (% des notes de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 12.6 | 10.7 | 11.9 | 12.6 | 13.1 | 13.6 |
| reel_acadien | 13.6 | 10.6 | 11.9 | 13.6 | 13.6 | 12.9 |
| gigue_quebecoise | 8.8 | 6.2 | 7.6 | 8.8 | 9.9 | 10.5 |
| valse_quebecoise | 5.0 | 3.8 | 4.6 | 5.0 | 7.4 | 8.3 |
| marche_britannique_coloniale | 1.9 | 1.7 | 1.9 | 1.9 | 2.2 | 2.2 |
| gigue_britannique_coloniale | — | — | — | — | — | — |
| hornpipe_britannique_coloniale | — | — | — | — | — | — |

### density — Densité de pieds (notes talon + orteil par mesure)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 0.00 | 3.99 | 7.97 | 11.95 | 11.95 |
| reel_acadien | 7.97 | 0.00 | 3.99 | 7.97 | 11.95 | 11.95 |
| gigue_quebecoise | 5.98 | 0.00 | 2.00 | 5.98 | 7.97 | 7.97 |
| valse_quebecoise | 3.00 | 0.00 | 1.00 | 3.00 | 5.00 | 5.00 |
| marche_britannique_coloniale | 9.50 | 5.00 | 9.50 | 9.50 | 14.00 | 14.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### density — Crête du rendu (dBFS)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | -3.37 | -1.97 | -1.84 | -1.84 | -1.83 | -2.34 |
| reel_acadien | -3.12 | -3.65 | -1.72 | -1.72 | -1.70 | -1.77 |
| gigue_quebecoise | -3.20 | -5.32 | -2.22 | -2.25 | -2.24 | -2.35 |
| valse_quebecoise | -4.51 | -2.23 | -2.19 | -2.21 | -4.14 | -3.81 |
| marche_britannique_coloniale | -3.88 | -3.73 | -3.31 | -3.31 | -3.38 | -3.38 |
| gigue_britannique_coloniale | -3.81 | -3.03 | -3.03 | -3.03 | -3.03 | -1.48 |
| hornpipe_britannique_coloniale | -5.12 | -4.03 | -4.03 | -4.03 | -4.03 | -3.92 |

### density — Échantillons écrêtés (>= 1)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

## tension

### tension — Tempo (BPM)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 230.0 | 229.4 | 229.9 | 230.9 | 231.9 | 233.0 |
| reel_acadien | 228.0 | 227.4 | 227.9 | 228.9 | 229.9 | 231.0 |
| gigue_quebecoise | 172.5 | 171.9 | 172.4 | 173.4 | 174.4 | 175.5 |
| valse_quebecoise | 145.0 | 144.4 | 144.9 | 145.9 | 146.9 | 148.0 |
| marche_britannique_coloniale | 108.0 | 108.0 | 108.0 | 108.0 | 110.2 | 112.5 |
| gigue_britannique_coloniale | 156.0 | 155.5 | 155.9 | 156.6 | 157.3 | 158.0 |
| hornpipe_britannique_coloniale | 92.0 | 91.6 | 91.9 | 92.5 | 93.0 | 93.6 |

### tension — Vélocité moyenne, mélodie

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 92.3 | 89.4 | 91.9 | 97.5 | 103.0 | 109.4 |
| reel_acadien | 91.7 | 89.0 | 91.4 | 96.8 | 102.1 | 108.2 |
| gigue_quebecoise | 92.6 | 89.4 | 92.0 | 99.1 | 105.9 | 113.8 |
| valse_quebecoise | 97.6 | 94.9 | 97.2 | 102.7 | 107.9 | 114.0 |
| marche_britannique_coloniale | 101.1 | 100.4 | 100.8 | 101.1 | 107.0 | 113.0 |
| gigue_britannique_coloniale | 91.2 | 88.6 | 90.6 | 95.6 | 100.4 | 105.4 |
| hornpipe_britannique_coloniale | 95.9 | 93.5 | 95.5 | 99.5 | 103.5 | 107.5 |

### tension — Vélocité moyenne, toutes pistes

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 75.3 | 73.1 | 75.1 | 77.7 | 80.5 | 83.4 |
| reel_acadien | 75.6 | 73.5 | 75.5 | 78.0 | 80.8 | 83.8 |
| gigue_quebecoise | 74.4 | 72.2 | 74.2 | 76.8 | 79.7 | 82.7 |
| valse_quebecoise | 79.6 | 77.4 | 79.5 | 81.5 | 83.7 | 86.0 |
| marche_britannique_coloniale | 90.1 | 89.6 | 89.9 | 90.1 | 95.8 | 101.4 |
| gigue_britannique_coloniale | 72.1 | 70.6 | 71.9 | 73.1 | 74.5 | 75.5 |
| hornpipe_britannique_coloniale | 75.9 | 74.4 | 75.7 | 76.8 | 78.3 | 79.3 |

### tension — Vélocité moyenne, accords

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 74.0 | 72.2 | 74.0 | 73.0 | 72.0 | 71.0 |
| reel_acadien | 74.0 | 72.3 | 74.0 | 73.0 | 72.0 | 71.0 |
| gigue_quebecoise | 74.0 | 72.2 | 74.0 | 73.0 | 72.0 | 71.1 |
| valse_quebecoise | 74.0 | 72.3 | 74.0 | 73.0 | 72.0 | 71.0 |
| marche_britannique_coloniale | — | — | — | — | — | — |
| gigue_britannique_coloniale | 62.1 | 61.1 | 62.1 | 61.2 | 61.0 | 60.0 |
| hornpipe_britannique_coloniale | 62.1 | 61.1 | 62.1 | 61.3 | 61.1 | 60.1 |

### tension — Vélocité moyenne, pieds

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 45.7 | 44.6 | 45.7 | 47.4 | 49.3 | 51.5 |
| reel_acadien | 45.6 | 44.6 | 45.6 | 47.3 | 49.4 | 51.5 |
| gigue_quebecoise | 44.1 | 43.1 | 44.1 | 45.8 | 47.7 | 49.9 |
| valse_quebecoise | 43.8 | 42.8 | 43.8 | 45.4 | 48.0 | 49.7 |
| marche_britannique_coloniale | 80.3 | 80.2 | 80.3 | 80.3 | 85.8 | 91.1 |
| gigue_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |
| hornpipe_britannique_coloniale | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 | 110.0 |

### tension — Notes par mesure, mélodie

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.29 | 7.30 | 7.29 | 7.29 | 7.50 | 7.41 |
| reel_acadien | 7.89 | 7.90 | 7.89 | 7.85 | 8.03 | 8.03 |
| gigue_quebecoise | 4.96 | 4.96 | 4.96 | 4.96 | 5.13 | 5.04 |
| valse_quebecoise | 4.28 | 4.28 | 4.28 | 4.28 | 4.27 | 4.23 |
| marche_britannique_coloniale | 4.10 | 4.10 | 4.10 | 4.10 | 4.10 | 4.15 |
| gigue_britannique_coloniale | 4.97 | 4.97 | 4.97 | 4.97 | 4.99 | 4.99 |
| hornpipe_britannique_coloniale | 6.42 | 6.42 | 6.42 | 6.42 | 6.42 | 6.42 |

### tension — Notes par mesure, accords

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| reel_acadien | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| gigue_quebecoise | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| valse_quebecoise | 8.00 | 8.00 | 8.00 | 8.00 | 8.00 | 8.00 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| hornpipe_britannique_coloniale | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |

### tension — Notes par mesure, basse

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.62 | 2.53 | 2.62 | 2.63 | 2.63 | 2.65 |
| reel_acadien | 2.73 | 2.67 | 2.73 | 2.75 | 2.72 | 2.71 |
| gigue_quebecoise | 2.67 | 2.64 | 2.67 | 2.69 | 2.80 | 2.76 |
| valse_quebecoise | 1.01 | 1.01 | 1.01 | 1.01 | 1.02 | 1.00 |
| marche_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| gigue_britannique_coloniale | 2.76 | 2.73 | 2.76 | 2.75 | 2.82 | 2.84 |
| hornpipe_britannique_coloniale | 2.38 | 2.35 | 2.38 | 2.40 | 2.38 | 2.41 |

### tension — Notes par mesure, talon

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 |
| reel_acadien | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 | 3.99 |
| gigue_quebecoise | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| valse_quebecoise | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| marche_britannique_coloniale | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### tension — Notes par mesure, orteil

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 |
| reel_acadien | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 |
| gigue_quebecoise | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 | 3.98 |
| valse_quebecoise | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 | 2.00 |
| marche_britannique_coloniale | 7.50 | 7.50 | 7.50 | 7.50 | 7.50 | 7.50 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### tension — Notes par mesure, 2ᵉ voix

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 6.71 | 6.48 | 6.71 | 6.71 | 6.82 | 6.65 |
| reel_acadien | 7.28 | 7.08 | 7.28 | 7.27 | 7.39 | 7.39 |
| gigue_quebecoise | 4.69 | 4.58 | 4.69 | 4.69 | 4.83 | 4.78 |
| valse_quebecoise | 4.19 | 4.00 | 4.19 | 4.19 | 4.20 | 4.17 |
| marche_britannique_coloniale | 4.06 | 3.93 | 4.06 | 4.06 | 4.06 | 4.10 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### tension — Hauteur moyenne de la mélodie (MIDI)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 71.5 | 71.5 | 71.5 | 71.5 | 72.0 | 72.0 |
| reel_acadien | 72.2 | 72.2 | 72.2 | 72.2 | 72.0 | 72.0 |
| gigue_quebecoise | 72.0 | 72.0 | 72.0 | 72.0 | 71.9 | 71.8 |
| valse_quebecoise | 71.1 | 71.1 | 71.1 | 71.1 | 70.6 | 71.0 |
| marche_britannique_coloniale | 84.7 | 84.7 | 84.7 | 84.7 | 84.7 | 84.8 |
| gigue_britannique_coloniale | 71.9 | 71.9 | 71.9 | 71.9 | 71.9 | 71.9 |
| hornpipe_britannique_coloniale | 72.0 | 72.0 | 72.0 | 72.0 | 72.2 | 72.2 |

### tension — Part de mesures en mode sombre (%)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 25.0 | 25.0 | 25.0 | 25.0 | 25.0 | 25.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| marche_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |
| gigue_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |
| hornpipe_britannique_coloniale | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 | 15.0 |

### tension — Notes Detached (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.5 | 0.5 | 0.5 | 0.5 | 0.8 | 0.6 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### tension — Notes Linked (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.3 | 0.3 | 0.3 | 0.3 | 0.4 | 0.6 |
| reel_acadien | 0.3 | 0.3 | 0.3 | 0.3 | 0.2 | 0.2 |
| gigue_quebecoise | 0.6 | 0.6 | 0.6 | 0.6 | 0.5 | 0.8 |
| valse_quebecoise | 1.5 | 1.5 | 1.5 | 1.5 | 1.4 | 1.2 |
| marche_britannique_coloniale | 2.2 | 2.2 | 2.2 | 2.2 | 2.2 | 2.2 |
| gigue_britannique_coloniale | 0.2 | 0.2 | 0.2 | 0.2 | 0.2 | 0.2 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

### tension — Notes Accent (% de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 1.0 | 1.0 | 1.0 | 1.0 | 1.1 | 1.1 |
| reel_acadien | 1.2 | 1.2 | 1.2 | 1.2 | 0.9 | 0.9 |
| gigue_quebecoise | 2.1 | 2.1 | 2.1 | 2.1 | 1.4 | 1.6 |
| valse_quebecoise | 1.1 | 1.1 | 1.1 | 1.1 | 1.0 | 0.9 |
| marche_britannique_coloniale | 1.8 | 1.8 | 1.8 | 1.8 | 1.8 | 1.7 |
| gigue_britannique_coloniale | 1.7 | 1.7 | 1.7 | 1.7 | 1.5 | 1.5 |
| hornpipe_britannique_coloniale | 2.0 | 2.0 | 2.0 | 2.0 | 2.3 | 2.3 |

### tension — Gate moyen de la mélodie (‰)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 937 | 964 | 942 | 918 | 893 | 860 |
| reel_acadien | 906 | 932 | 910 | 889 | 859 | 822 |
| gigue_quebecoise | 963 | 993 | 968 | 942 | 916 | 891 |
| valse_quebecoise | 967 | 996 | 972 | 946 | 919 | 898 |
| marche_britannique_coloniale | 937 | 987 | 962 | 937 | 912 | 893 |
| gigue_britannique_coloniale | 970 | 1000 | 975 | 950 | 923 | 898 |
| hornpipe_britannique_coloniale | 964 | 994 | 969 | 944 | 923 | 898 |

### tension — Durée moyenne d'une note de mélodie (pas)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 2.25 | 2.26 | 2.25 | 2.23 | 2.06 | 1.26 |
| reel_acadien | 2.12 | 2.13 | 2.12 | 2.11 | 1.95 | 1.21 |
| gigue_quebecoise | 2.44 | 2.50 | 2.44 | 2.37 | 2.23 | 1.36 |
| valse_quebecoise | 3.00 | 3.07 | 3.00 | 2.91 | 2.59 | 1.86 |
| marche_britannique_coloniale | 3.96 | 3.97 | 3.96 | 3.96 | 3.80 | 3.10 |
| gigue_britannique_coloniale | 2.43 | 2.43 | 2.43 | 2.43 | 2.26 | 2.26 |
| hornpipe_britannique_coloniale | 2.50 | 2.50 | 2.50 | 2.48 | 2.27 | 2.25 |

### tension — Ornements : notes ajoutées par mesure

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.588 | 0.600 | 0.588 | 0.588 | 0.753 | 0.634 |
| reel_acadien | 1.284 | 1.297 | 1.284 | 1.247 | 1.438 | 1.306 |
| gigue_quebecoise | 0.113 | 0.113 | 0.113 | 0.113 | 0.188 | 0.128 |
| valse_quebecoise | 0.237 | 0.237 | 0.237 | 0.237 | 0.247 | 0.203 |
| marche_britannique_coloniale | 0.172 | 0.172 | 0.172 | 0.172 | 0.172 | 0.094 |
| gigue_britannique_coloniale | 0.031 | 0.031 | 0.031 | 0.031 | 0.047 | 0.047 |
| hornpipe_britannique_coloniale | 0.106 | 0.106 | 0.106 | 0.106 | 0.178 | 0.178 |

### tension — Notes de passage de basse par mesure

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.627 | 0.537 | 0.627 | 0.634 | 0.636 | 0.655 |
| reel_acadien | 0.739 | 0.678 | 0.739 | 0.759 | 0.720 | 0.719 |
| gigue_quebecoise | 0.678 | 0.644 | 0.678 | 0.697 | 0.802 | 0.766 |
| valse_quebecoise | 0.011 | 0.011 | 0.011 | 0.011 | 0.020 | 0.000 |
| marche_britannique_coloniale | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| gigue_britannique_coloniale | 0.769 | 0.731 | 0.769 | 0.752 | 0.822 | 0.844 |
| hornpipe_britannique_coloniale | 0.381 | 0.356 | 0.381 | 0.409 | 0.380 | 0.414 |

### tension — Variations de la 2ᵉ voix (% des notes de la mélodie)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 12.6 | 15.8 | 12.6 | 12.8 | 14.8 | 15.0 |
| reel_acadien | 13.6 | 16.2 | 13.6 | 13.6 | 15.6 | 14.9 |
| gigue_quebecoise | 8.8 | 11.1 | 8.8 | 8.8 | 8.4 | 7.6 |
| valse_quebecoise | 5.0 | 9.3 | 5.0 | 5.0 | 4.9 | 3.7 |
| marche_britannique_coloniale | 1.9 | 4.9 | 1.8 | 1.9 | 2.8 | 2.0 |
| gigue_britannique_coloniale | — | — | — | — | — | — |
| hornpipe_britannique_coloniale | — | — | — | — | — | — |

### tension — Densité de pieds (notes talon + orteil par mesure)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| reel_acadien | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 | 7.97 |
| gigue_quebecoise | 5.98 | 5.98 | 5.98 | 5.98 | 5.98 | 5.98 |
| valse_quebecoise | 3.00 | 3.00 | 3.00 | 3.00 | 3.00 | 3.00 |
| marche_britannique_coloniale | 9.50 | 9.50 | 9.50 | 9.50 | 9.50 | 9.50 |
| gigue_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| hornpipe_britannique_coloniale | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

### tension — Crête du rendu (dBFS)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | -3.37 | -1.57 | -1.60 | -2.35 | -2.07 | -4.11 |
| reel_acadien | -3.12 | -1.82 | -1.49 | -1.50 | -1.65 | -2.56 |
| gigue_quebecoise | -3.20 | -2.64 | -2.20 | -2.64 | -2.49 | -3.48 |
| valse_quebecoise | -4.51 | -2.55 | -2.25 | -1.34 | -2.49 | -3.00 |
| marche_britannique_coloniale | -3.88 | -2.45 | -3.34 | -3.31 | -2.73 | -2.22 |
| gigue_britannique_coloniale | -3.81 | -3.36 | -3.09 | -2.71 | -2.64 | -2.39 |
| hornpipe_britannique_coloniale | -5.12 | -4.39 | -4.06 | -3.77 | -3.26 | -3.43 |

### tension — Échantillons écrêtés (>= 1)

| Profil | neutre | 0 | 0.25 | 0.5 | 0.75 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| reel_quebecois | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| reel_acadien | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| valse_quebecoise | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| marche_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| gigue_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| hornpipe_britannique_coloniale | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

