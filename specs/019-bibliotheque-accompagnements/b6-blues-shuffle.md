# B6 — Blues shuffle, 12/8 ternaire (`gmd_blues_shuffle`)

Date : 2026-11-04. Compilateur : nightly-2026-10-03 (`0abfedbc7`, override rustup de l'orchestrateur) ; tout ce qui suit est compilé et mesuré avec lui. Branche `wquintal/b6-shuffle`, partie de `main` à `3b90e67` (`TUNING_VERSION` 29, **inchangé** : profil neuf, aucun
profil existant touché, aucun fichier de `src/` d'un crate moteur modifié). Des **données** seulement : l'A6 (grille de douze pas,
micro-timing par voix, coups probabilistes), l'A3 (arc), l'A1 (phrases, relances) et l'A2 (session) suffisent.

## Ce qui est livré

Un profil distribué sans Odin, `profiles/gmd_blues_shuffle/` (`profile.toml`, `percussion.toml`, `arc.toml`, `session.toml`,
`dynamics.toml`, `arrangement.toml`, `set.toml`, `cells.toml`, `README.md`). Le moteur le charge seul (`build.rs` embarque `profiles/*`).

| Brief | Livré |
|---|---|
| 1. Profil nommé par sa source, grille ternaire (12 pas, A6) | `gmd_blues_shuffle` : 4/4 sur 12 pas (3 par temps), swing 0,664 (médiane des cinq fichiers ternaires du GMD), tempo 72–134, défaut 125 |
| Batterie GMD shuffle (et rock shuffle si la source le justifie) | **Oui, justifiée** : `blues/shuffle` n'a que 2 fichiers ternaires sur 4 (« trop peu »). Lus aussi : `rock/shuffle` (2 fichiers ternaires) et `funk/purdieshuffle` (1 fichier, 26 relances ternaires) : 5 fichiers `beat`, 4 batteurs, 340 mesures ; 29 fichiers `fill`. 18 patterns, 24 phrases, 12 relances, 52 coups probabilistes, micro-timing de 7 voix |
| Basse en shuffle, motifs « boogie » fondamentale-quinte-sixte en croches ternaires (règles de métier) | 9 riffs d'une ou deux mesures (boogie, montée, pompe, marche, quinte, deux mesures, triolets de blues lent, tenue) ; registre et proportions d'approche de WJazzD blues (87 solos) ; passage au temps 4 (`before = [3]`) |
| Comping piano ou guitare, shuffle, accords de 7ᵉ | 12 rythmes (`sparse` / `medium` / `dense`, dont les triolets de piano) et 17 phrases ; piano 0 par défaut ; voicings `guide_tones`, `shell`, `drop2`, `rootless` |
| Blues de 12 mesures par défaut (quick change, turnaround) | 5 boucles : forme commune, changement rapide (quick change), retournement I-VI-ii-V (turnaround), mineur, fin posée |
| 2. `[session]` complet, arc et relances (A1) | 9 groupes (tempo, tonalité, comping, basse, kit, voicing, registre, énergie, arc), 414 720 habillages ; arc de cinq chorus + trois arcs nommés ; relances `hat`/`ride` |
| 3. `harmonium_determinism`, README, tests de contrat, licences | Les profils distribués y entrent seuls (`StyleProfile::distributed()`), natif = wasm vérifié ; `README.md` du profil et de `profiles/` ; 11 tests de contrat ; mentions CC BY 4.0 (GMD) et ODbL / DbCL (WJazzD) dans `[meta] corpus` et le README |

Outils (dans `specs/019-bibliotheque-accompagnements/b6/`) : `batterie.py` (percussion + micro-timing depuis le GMD), `emit_profile.py` +
`profile.toml.tmpl` (grille, basse, comping), `emit_session.py` (session et arc), `habillage_mesure.py`, `distance_gmd.py`, `preuves/*.json`.
Seul fichier existant modifié : `b8/gmd_batterie.py` (champs `feel`, `steps`, `start_cuts` de `DrumSpec`, `H019_GMD_CACHE`, **valeurs par défaut
= comportement de B8**, jamais lu par un profil existant au runtime).

## Choix documentés (sans question au propriétaire)

- **Sources** : voir ci-dessus ; le « rock 12/8 » (19 fichiers ternaires de `rock`) n'est pas lu (c'est le feel de `gmd_rock`).
- **Seuils d'éligibilité** : un fichier suffit (`min_files = 1`), 8 mesures au moins, pas de plafond de part de fichier : les seuils de B8 ne
  retiennent aucun groupe ici. Conséquence écrite dans le README : chaque groupe décrit surtout un batteur.
- **Pas de `[swing]`** : les cinq fichiers ternaires ne montrent aucune pente avec le tempo (0,627 à 112 BPM, 0,650 à 130, 0,664 à 134,
  0,666 à 72, 0,696 à 125) ; la courbe d'A6 est celle de JTD (jazz de piano). Un rapport fixe `default_swing_ratio = 0,664`. Pas de
  variante de swing en session.
- **Pas de 12/8 lent sur 24 pas** : A6 ne swingue jamais une grille composée ; le blues lent est un shuffle de 12 pas à 72–92 BPM (la bande
  `lent` de la session, appuyée sur un seul fichier de la source à 72 BPM).
- **Basse** : `reference = "chord_root"`, sans tierce (la tierce dépend de la qualité) ; la sixte sur `Im7` du blues mineur est une couleur
  de blues, pas une note de l'accord (le moteur ne l'interdit pas : le contrat ne contraint que fondamentale, quinte, sixte, septième).
- **Arc** : retombée à l'intensité 0,45 (pas 0,6 comme `gmd_funk`) : la densité de batterie est `1 + percussion_intensity(intensité)` arrondie ;
  à 0,6 la retombée gardait la grosse caisse (58 coups en 12 mesures). Mêmes cibles de LUFS que B8 pour le mixage.
- **Tonalités** : mixolydien sur mi, la, sol, do, si♭, ré (poids 3, 3, 2, 2, 2, 1). `dynamics.parallel_modes` : mixolydien clair / dorien sombre,
  comme les autres profils sous grille.

## Preuves chiffrées

### Habillages distincts sur 16 graines (`habillage_mesure.py`, vérité terrain du CLI, `preuves/habillage.json`)

| | Résultat (16 graines) | Seuil (D6) |
|---|---|---|
| Habillages distincts (grappe de tempo ± 8 BPM, tonalité, programmes accords, basse, kit, voicings) | **16 / 16** | ≥ 8 |
| Grappes de tempo | 5 (78 à 133 BPM) | ≥ 2 |
| Tonalités | 6 (mi, la, sol, do, si♭, ré) | ≥ 3 |
| Programmes de comping / basse / kits / voicings | 6 / 4 / 5 / 4 | ≥ 3 / 2 / 3 / 2 |
| Arcs tirés | défaut 7, orchestre 7, montée 2 | — |

Test de contrat équivalent : `seize_graines_donnent_au_moins_huit_habillages_dans_les_bornes_du_style`.

### Variété entre graines et entre chorus (`harmonium-lab seed-variety --chorus 4`, `preuves/variete.json`)

16 graines × 4 chorus de 12 mesures, `--engine-bin` de la branche, lancé depuis `~/workspace/harmonium`. Cellule : **mesures changées d'un
chorus au suivant / suites distinctes sur 16 graines** (chorus égal, place égale). Pas d'« état d'avant » (profil neuf) : `gmd_funk` (B4,
même moteur, même habillage d'A2, mesuré dans la même séance) sert de repère.

| Profil | comping rythme | comping voicings | basse | batterie | accompagnement | accords | orchestration |
|---|---|---|---|---|---|---|---|
| `gmd_blues_shuffle` | 89 % / 16 | 97 % / 16 | 6 % / 16 | 95 % / 16 | 99 % / 16 | 0 % / 4 | 21 % / 15 |
| `gmd_funk` (repère) | 93 % / 16 | 98 % / 16 | 6 % / 16 | 78 % / 16 | 100 % / 16 | 0 % / 5 | 19 % / 13 |

Toutes les couches actives atteignent 16 / 16 suites distinctes (comping, basse, batterie, accompagnement) ; la batterie change de 95 % de ses
mesures d'un chorus au suivant, 0 % de paires de chorus identiques. **Ce qui ne varie pas** : la basse (6 % de mesures changées : un plan de
motifs par air, limite d'A4/S0, voir manques) et les accords (une boucle par air, 4 boucles distinctes vues sur 16 graines : la cinquième,
« fin posée », a le poids le plus faible).

### Distance de la batterie au GMD (`distance_gmd.py`, `preuves/distance-gmd.json`)

Bray-Curtis des fréquences d'attaque par pas (0 = identique), sur 12 pas, contre **toutes** les mesures `beat` ternaires des trois styles
(pas seulement les groupes retenus), 16 graines × 47 mesures rendues. « Copie non calibrée » : même profil avec le `percussion.toml` de la
fixture `grille_shuffle_ternaire` (batterie d'exercice écrite à la main), sans arc ni session ; sa grosse caisse et sa caisse claire ne
jouent pas aux pas mesurés (distance 1,0).

| | toutes voix | grosse caisse | caisse claire | charleston | ride | **moyenne** (4 métriques de L2) |
|---|---|---|---|---|---|---|
| profil livré | 0,028 | 0,076 | 0,136 | 0,121 | 0,148 | **0,090** |
| sans arc | 0,029 | 0,129 | 0,135 | 0,096 | 0,175 | 0,097 |
| copie non calibrée | 0,142 | 1,000 | 1,000 | 0,521 | 0,460 | 0,666 |

Profil neuf : pas de « perte » à comparer (aucun état d'avant) ; l'arc n'éloigne pas la batterie de la source (la moyenne passe de 0,097
sans arc à 0,090 avec). La mesure est biaisée vers un bon score (groupes construits sur ces mêmes mesures, un fichier par groupe) : elle
montre que la batterie suit la source, pas qu'elle la dépasse.

### Mixage (`b8/calibrer_mix.py`, hors ligne et en silence)

LUFS intégré en solo, graines 1 et 2, 48 mesures : avant calage accords −29,6, basse −20,8, batterie −25,2 ; après (accords −16,2 dB, basse
−14,8 dB, percussion −16,5 dB) : accords −26,1, basse −25,0, batterie −25,7 (grosse caisse −31,3, reste −27,1), mix complet −21,5.

## Tests

`harmonium_core/tests/gmd_blues_shuffle_profile_tests.rs` (11 tests) : grille de douze mesures (cinq boucles, tierce et septième par mesure,
reprise), mètre ternaire, rapport de swing, tempo ; basse (fenêtre, intervalles, arrivée, paires longue-brève) ; comping dans l'accord publié ;
batterie (densités, caisse claire, relances) ; arc de cinq chorus ; reproductibilité ; variété (comping ≥ 14, basse ≥ 12, batterie ≥ 12 suites
sur 16 graines) ; habillages ; priorité de l'hôte.

## Limites

Voir le README du profil. En bref : cinq fichiers, quatre batteurs, des groupes d'un seul batteur ; relances d'un seul batteur ; basse et
comping sans source ; rapport de swing fixe ; pas de 12/8 sur 24 pas ; mixage au LUFS sans écoute ; aucune écoute par l'agent (D5/D7).

## Manques de schéma (liste pour A7, aucun contourné)

1. **Pas de grille composée swinguée** (12/8 sur 24 pas) : un shuffle lent ne peut s'écrire qu'en 12 pas par mesure.
2. **Basse sans tirage par chorus** (déjà relevé par B4/B7 et B8) : 6 % de mesures changées d'un chorus au suivant ; les motifs de `bars` / `templates` d'S0
   sont fixés par air.
3. **Micro-timing en fraction de pas, pas en ms** : un décalage mesuré en ms dérive avec le tempo tiré (la bande `lent` à 72 BPM, 42 % de pas de moins).
4. **Boucle de grille unique par air** : l'accord ne change pas d'un chorus au suivant (0 % de mesures changées, 4 boucles sur 16 graines).
