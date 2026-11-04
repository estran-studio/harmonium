# V2 — Variance prouvée par style et par profil existant

Date : 2026-11-04. Branche `wquintal/v2t1-preuve-livraison`, moteur de `main` à `22c3afb` (profils identiques à `3fb01da`,
`TUNING_VERSION` 32, binaire `harmonium-cli --release` compilé du worktree). **Documents seulement : aucun code, aucun profil
du moteur n'est modifié.** L'outil et les mesures sont au labo (`harmonium_lab`, branche `wquintal/v2t1-preuve-livraison`, commit `dc4b406`,
`references/v2-variance/` : un JSON par profil, `summary.json`, `README.md` avec les seuils, les valeurs mesurées et les distances) ;
`make v2-variance ENGINE_DIR=… ENGINE_BIN=…` les reproduit. Usage par training : [`t1-usage-training.md`](t1-usage-training.md).

## Ce qui est mesuré

**32 profils distribués × 16 graines × 4 chorus** (`seed-variety --chorus 4`, `--tours 4`), puis les **17 profils `loop`
d'accompagnement sous une grille utilisateur de 8 mesures** (accords imposés, `[fixed_tonality]` imposée comme le fait l'hôte,
tempo imposé dans la lecture « hôte » de H1). Trois groupes de critères :

- **H : « New seed s'entend »** (D6) : habillages distincts sur 16 graines (grappe de tempo ± 8 BPM, tonalité, programmes de
  mélodie, d'accords, de basse, kit, famille de voicings, swing, lus dans `MeasureSnapshot.session` / `instruments`), grappes de
  tempo, part des places (chorus, mesure) où comping et batterie diffèrent d'une graine à l'autre ;
- **1 à 9 : « ce n'est pas une boucle »** : paires de chorus identiques, accords stables, comping / voicings / basse / batterie
  changés d'un chorus au suivant, suites distinctes, orchestration ;
- **10 et 11 : l'arc** (densité et vélocité dès le 2ᵉ chorus).

Seuils : ceux de V0 (`harmonium_lab/references/v0-variance/README.md`), **ajustés au vu des chiffres et écrits** dans le README du
labo (tableau V0 → V2 avec la raison de chaque changement : basse jugée à sa classe, marche ≥ 80 % et riff ≥ 10 % ; arc lu dès
le 2ᵉ chorus ; `intensity` abandonnée car publiée constante ; H1 à H3 ajoutés). Rien n'a été abaissé pour qu'un profil passe.

**Correction d'une mesure de V0** : `chorusvariety.midi_programs` triait les `program_change` d'un même tick par numéro de
programme ; quand le profil et la session règlent le même canal (profil d'abord), **le plus grand numéro gagnait au lieu du
dernier écrit**. Corrigé au labo avec un test. Les chiffres d'orchestration de V0 n'en dépendent pas ; ceux de R2 et B
(« Orchestration : suites distinctes ») étaient **sous-estimés** quand la graine tirait un programme plus petit que celui du profil.

## Résultats

Tenu ✓, non tenu ✗, sans objet · (couche muette ou critère propre à l'hôte) ; « Tenus » = tenus / applicables. Critères :


- **H1** : habillages distincts sur 16 graines ≥ 12 (sous grille, tempo et tonalité imposés : ≥ 8)
- **H2** : grappes de tempo (± 8 BPM) ≥ 2 (sans objet sous grille)
- **H3** : places où les graines diffèrent : comping rythme ≥ 75 % / batterie ≥ 50 %
- **1** : paires de chorus identiques, accompagnement entier : 0 %
- **2** : paires de chorus identiques par couche (comping rythme / voicings / batterie) ≤ 5 %
- **3** : accords changés d'un chorus au suivant ≤ 2 %
- **4** : comping rythme changé d'un chorus au suivant ≥ 75 %
- **5** : comping voicings changés ≥ 65 %
- **6** : basse changée ≥ 80 % (marche) ou ≥ 10 % (riff, ostinato)
- **7** : batterie changée ≥ 30 % ; distance graduée ≥ 10 %
- **8** : suites de 4 chorus distinctes sur 16 graines (comping rythme / basse / batterie) ≥ 12
- **9** : orchestration : programmes de comping / basse / kit ≥ 3 / 2 / 2 ; suites distinctes ≥ 4
- **10** : arc, densité (notes par mesure) : étendue de la trajectoire dès le 2e chorus ≥ 25 %
- **11** : arc, vélocité moyenne : étendue dès le 2e chorus ≥ 8 %

### Profils distribués (grille propre du profil)

| Profil | H1 | H2 | H3 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | Tenus |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---:|
| `gigue_acadienne` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | 5 / 14 |
| `gigue_britannique_coloniale` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | · | ✗ | ✓ | ✗ | ✗ | 6 / 13 |
| `gigue_quebecoise` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | 5 / 14 |
| `gmd_afrobeat` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_afrocuban` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | 13 / 14 |
| `gmd_blues_shuffle` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_dance_disco` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 14 |
| `gmd_funk` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_hiphop` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_jazz_waltz` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | 12 / 14 |
| `gmd_latin_brazilian_baiao` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_latin_brazilian_bossa` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | 13 / 14 |
| `gmd_latin_brazilian_samba` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_neworleans_funk_secondline` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 14 / 14 |
| `gmd_pop` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_reggae` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_rock` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `gmd_soul` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `hornpipe_britannique_coloniale` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | · | ✗ | ✓ | ✗ | ✗ | 6 / 13 |
| `isaac_basement` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | 7 / 14 |
| `isaac_burning` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | 8 / 14 |
| `isaac_cellar` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | 8 / 14 |
| `isaac_downpour` | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | 6 / 14 |
| `marche_britannique_coloniale` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | · | · | · | ✗ | ✗ | ✗ | ✗ | ✗ | 4 / 11 |
| `nt_desert` | ✗ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | 5 / 14 |
| `reel_acadien` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | 5 / 14 |
| `reel_britannique_coloniale` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | · | ✗ | ✓ | ✗ | ✗ | 6 / 13 |
| `reel_quebecois` | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | 5 / 14 |
| `valse_britannique_coloniale` | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | · | ✗ | ✓ | ✗ | ✗ | 4 / 13 |
| `valse_quebecoise` | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 2 / 14 |
| `wjazzd_ballad` | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 14 |
| `wjazzd_swing` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | 13 / 14 |

### Sous grille utilisateur (17 profils `loop` d'accompagnement)

Les 15 profils trad et de jeu qui n'y figurent pas sont **sans objet** : trad (`melody_functional` ou `none`) sans grille
utilisateur, jeux (`loop`) **refusés au chargement** (`accompaniment.bass.motifs[0].part: partie inconnue dans form.parts`).
Longueur de la grille : 8 mesures ; **16** pour `gmd_afrocuban`, `gmd_funk`, `gmd_latin_brazilian_{baiao,bossa,samba}` ; **24** pour
`gmd_jazz_waltz` et `wjazzd_ballad` (le moteur refuse un `breaks.start` plus grand que la grille, voir « Manques »).

| Profil | H1 | H2 | H3 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | Tenus |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---:|
| `gmd_afrobeat.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✗ | 11 / 13 |
| `gmd_afrocuban.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | 12 / 13 |
| `gmd_blues_shuffle.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_dance_disco.grille` | ✓ | · | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 11 / 13 |
| `gmd_funk.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_hiphop.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_jazz_waltz.grille` | ✓ | · | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | 11 / 13 |
| `gmd_latin_brazilian_baiao.grille` | ✓ | · | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_latin_brazilian_bossa.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 13 / 13 |
| `gmd_latin_brazilian_samba.grille` | ✓ | · | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_neworleans_funk_secondline.grille` | ✓ | · | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_pop.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_reggae.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_rock.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `gmd_soul.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `isaac_basement.grille` | sans objet |  |  |  |  |  |  |  |  |  |  |  |  |  | |
| `isaac_burning.grille` | sans objet |  |  |  |  |  |  |  |  |  |  |  |  |  | |
| `isaac_cellar.grille` | sans objet |  |  |  |  |  |  |  |  |  |  |  |  |  | |
| `isaac_downpour.grille` | sans objet |  |  |  |  |  |  |  |  |  |  |  |  |  | |
| `nt_desert.grille` | sans objet |  |  |  |  |  |  |  |  |  |  |  |  |  | |
| `wjazzd_ballad.grille` | ✓ | · | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 12 / 13 |
| `wjazzd_swing.grille` | ✓ | · | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | 12 / 13 |

## Habillages : ce qu'une graine change

| Profil | Habillages distincts (hôte : sans tempo ni tonalité) | Tempo (BPM) | Grappes | Tonalités | Programmes de comping / basse / kit |
|---|---:|---|---:|---:|---|
| `gigue_acadienne` | 16 / 16 (13) | 168–191 | 3 | 5 | 4 / 3 / 1 |
| `gigue_britannique_coloniale` | 16 / 16 (13) | 144–167 | 3 | 6 | 3 / 2 / 3 |
| `gigue_quebecoise` | 16 / 16 (14) | 160–184 | 3 | 4 | 4 / 2 / 1 |
| `gmd_afrobeat` | 15 / 16 (15) | 86–109 | 3 | 4 | 3 / 3 / 3 |
| `gmd_afrocuban` | 16 / 16 (16) | 105–128 | 2 | 6 | 3 / 3 / 5 |
| `gmd_blues_shuffle` | 16 / 16 (13) | 78–133 | 5 | 6 | 6 / 4 / 5 |
| `gmd_dance_disco` | 16 / 16 (15) | 120–136 | 2 | 5 | 3 / 4 / 4 |
| `gmd_funk` | 16 / 16 (15) | 80–126 | 4 | 4 | 4 / 4 / 4 |
| `gmd_hiphop` | 16 / 16 (13) | 70–99 | 3 | 6 | 3 / 3 / 4 |
| `gmd_jazz_waltz` | 15 / 16 (13) | 118–194 | 5 | 5 | 4 / 3 / 2 |
| `gmd_latin_brazilian_baiao` | 16 / 16 (16) | 95–110 | 2 | 6 | 3 / 2 / 4 |
| `gmd_latin_brazilian_bossa` | 16 / 16 (16) | 104–176 | 4 | 6 | 4 / 3 / 4 |
| `gmd_latin_brazilian_samba` | 14 / 16 (12) | 89–116 | 3 | 6 | 4 / 3 / 3 |
| `gmd_neworleans_funk_secondline` | 15 / 16 (14) | 82–118 | 3 | 4 | 3 / 3 / 3 |
| `gmd_pop` | 16 / 16 (15) | 83–142 | 2 | 3 | 4 / 4 / 4 |
| `gmd_reggae` | 16 / 16 (12) | 83–121 | 3 | 5 | 4 / 3 / 3 |
| `gmd_rock` | 16 / 16 (8) | 90–129 | 4 | 4 | 3 / 2 / 3 |
| `gmd_soul` | 15 / 16 (13) | 80–105 | 2 | 4 | 4 / 3 / 3 |
| `hornpipe_britannique_coloniale` | 16 / 16 (15) | 80–103 | 3 | 6 | 3 / 2 / 3 |
| `isaac_basement` | 12 / 16 (11) | 136–156 | 2 | 1 | 3 / 3 / 2 |
| `isaac_burning` | 14 / 16 (13) | 126–153 | 3 | 1 | 3 / 3 / 3 |
| `isaac_cellar` | 14 / 16 (14) | 130–152 | 3 | 1 | 3 / 3 / 4 |
| `isaac_downpour` | 11 / 16 (8) | 126–162 | 3 | 1 | 2 / 2 / 2 |
| `marche_britannique_coloniale` | 16 / 16 (10) | 90–125 | 4 | 5 | 0 / 0 / 3 |
| `nt_desert` | 8 / 16 (4) | 204–230 | 3 | 1 | 2 / 2 / 1 |
| `reel_acadien` | 15 / 16 (13) | 215–237 | 3 | 5 | 4 / 2 / 1 |
| `reel_britannique_coloniale` | 16 / 16 (13) | 192–223 | 4 | 5 | 3 / 2 / 3 |
| `reel_quebecois` | 15 / 16 (14) | 220–239 | 2 | 6 | 4 / 2 / 1 |
| `valse_britannique_coloniale` | 16 / 16 (14) | 126–149 | 3 | 5 | 3 / 2 / 3 |
| `valse_quebecoise` | 16 / 16 (14) | 140–150 | 1 | 7 | 4 / 2 / 1 |
| `wjazzd_ballad` | 16 / 16 (11) | 48–77 | 3 | 6 | 3 / 3 / 2 |
| `wjazzd_swing` | 16 / 16 (14) | 106–259 | 9 | 7 | 3 / 2 / 2 |

## Profils sous le seuil, et pourquoi

Les 17 profils d'accompagnement (`gmd_*`, `wjazzd_*`) tiennent **12 à 14 critères sur 14** (un seul, `gmd_neworleans_funk_secondline`,
les tient tous ; sous grille, `gmd_latin_brazilian_bossa` 13 / 13) : aucune boucle (critère 1 : 0 % de paires de chorus
identiques), accords stables, comping, voicings et batterie qui changent d'un chorus au suivant, 16 suites distinctes sur 16
graines, arc visible dans la densité. Trois familles de causes expliquent les écarts ; les deux autres familles de profils
(trad, jeux) échouent par construction.

**1. Basse en riff identique d'un chorus au suivant (critère 6) : 9 profils GMD, aussi sous grille.**
`gmd_afrobeat`, `gmd_blues_shuffle`, `gmd_dance_disco`, `gmd_funk`, `gmd_hiphop`, `gmd_pop`, `gmd_reggae`, `gmd_rock`,
`gmd_soul` : 2 à 6 % des mesures de basse changent d'un chorus au suivant (seuil 10 %), 21 à 53 % des paires de chorus ont une basse
exactement identique (`gmd_hiphop` 53 %, `gmd_rock` 50 %, cinq profils 46 %, `gmd_blues_shuffle` 37 %, `gmd_funk` 21 %). Les riffs sont des motifs écrits, choisis à la partie ;
l'arc ne restreint pas les motifs de basse (il ne règle que `bass_feel`), et les options d'A4 ne s'appliquent pas aux riffs :
c'est le manque cité par R2-styles et repris par A7 (« 3 % de mesures de basse changent d'un chorus au suivant »).
La graine, elle, change la basse (16 suites distinctes sur 16 graines, programmes de basse ≥ 2) : **entre graines oui,
d'un chorus à l'autre non**. Les profils à basse en marche (`wjazzd_*`, `gmd_jazz_waltz`) changent 90 à 100 % de leurs
mesures ; les latins et New Orleans (basse en riff avec plusieurs motifs) 20 à 76 %.

**2. Batterie qui rejoue un chorus à l'identique (critère 2) : `wjazzd_ballad` 29 %, `gmd_jazz_waltz` 21 %,
`gmd_latin_brazilian_samba` 17 %, `gmd_dance_disco` 15 %, `gmd_latin_brazilian_baiao` 5,2 % (0,2 point au-dessus du seuil),
`gmd_neworleans_funk_secondline` 8 % sous grille de 8 mesures.** Cause probable **[INFERENCE : non isolée par un export de contrôle]** : les chorus 2 à 4 partagent la dernière
étape de l'arc (`repeat_from = 1`) et la batterie y est tirée par étiquette. Le critère est plus dur que la musique (deux
chorus de solo qui partagent une même batterie sont normaux) ; il est tenu par les 12 autres profils.

**3. Arc de vélocité ou de densité sous le seuil (critères 10 et 11) :**
`wjazzd_swing` 3,3 %, `gmd_jazz_waltz` 1,1 %, `gmd_afrocuban` 7,3 % (seuil 8 %) : les arcs de jazz et d'afro-cubain
agissent sur la densité (étendue 49 à 82 %) et le feel, pas sur la vélocité. `gmd_latin_brazilian_bossa` : densité 24,85 %
(seuil 25 %), au point près ; sous grille de 16 mesures le même profil tient tout (13 / 13). `gmd_afrobeat` sous grille : vélocité
6,7 % (le profil seul la tient à 11 %).

**Profils trad (10 : `reel_*`, `gigue_*`, `hornpipe_*`, `marche_*`, `valse_*`) : 2 à 6 critères tenus sur 11 à 14.** L'habillage
est là (H1 : 16 / 16 pour 8 d'entre eux, 15 / 16 pour `reel_acadien` et `reel_quebecois`, tempo sur 1 à 4 grappes ; kit unique
pour les profils québécois et acadiens : une seule banque de percussion), la variance par chorus **non** : R2-trad a posé `[session]` (tempo, tonalité, instrument de mélodie, comping,
basse, voicings, énergie, arcs sur les tours qui règlent l'intensité et la vélocité de la mélodie) sans comping de phrases,
sans phrases ni relances de batterie (A1) et sans `[arc]`. Mesuré : comping rythme et voicings changent 4 à 11 % des mesures
d'un chorus au suivant (51 à 78 % de paires de chorus identiques pour le comping), batterie de pieds 15 à 25 % (absente de
`gigue_britannique_coloniale`, `hornpipe`, `reel_britannique_coloniale`, `valse_britannique_coloniale`), densité plate dès le 2ᵉ chorus (étendue 0 à 8 %), vélocité
2 à 7 %. **`marche_britannique_coloniale`** (`harmony.model = "none"`, caisse seule, pas de comping ni de basse) et les deux
valses boucle **l'accompagnement entier** (critère 1) : 83 % de paires identiques pour la marche, 54 % pour
`valse_britannique_coloniale`, 31 % pour `valse_quebecoise` (basse 7 à 8 % de mesures changées). `valse_quebecoise` est le
profil le plus bas (2 / 14) : plage de tempo de 10 BPM (140-150 : une seule grappe, critère H2), kit unique, comping à 57 % et batterie à 65 % de paires
identiques. Les trad sont des **airs de danse** : répéter l'accompagnement d'un tour au suivant
est la tradition même, et les profils n'ont aucune grille utilisateur ; la décision est celle du propriétaire (D2 : « tous les styles
d'iReal », donc pas ces profils pour training ; leur variance entre graines, elle, est prouvée).

**Profils de jeu (5 : `isaac_*`, `nt_desert`) : 5 à 8 critères tenus sur 14.** Habillage correct (H1 : 14 / 16 pour `isaac_burning` et
`isaac_cellar`, 12 pour `isaac_basement`, 11 pour `isaac_downpour` sous le seuil de 12, **8 pour `nt_desert`**), mais **comping, basse et
accords-nappes identiques d'un chorus à l'autre** (basse 0 % de mesures changées ; comping : 100 % de paires de chorus identiques pour
`isaac_burning`, `isaac_cellar` et `isaac_downpour`, 40 % pour `isaac_basement`, 31 % pour `nt_desert`), densité plate (`isaac_cellar` : 16,8 / 16,7 / 16,7 / 16,6 notes par mesure, étendue 1 %) : seules la batterie (40 à
69 % de mesures changées, par phrases et relances) et la mélodie bougent. C'est le constat de R2-jeux (« arc sans choix de motifs ou
gabarits de basse ni de boucles ») : l'arc des jeux règle l'intensité et les tags de batterie, rien qui change les
notes du comping ou de la basse. `isaac_downpour` n'a qu'un tour par air (`tours_per_tune = 1`, 11 habillages) et `nt_desert` n'a ni groupe de kit ni
groupe de comping (2 programmes de comping, 2 de basse, 1 de kit sur 16 graines). Ces profils ne sont pas candidats à training (D1, D2) : la mesure
dit où le moteur ne varie pas, elle ne demande pas de les corriger ici.

## Distance au corpus

Aucun profil ne perd de distance par rapport à `main` : **aucun profil n'est modifié par V2**, et les distances rejouées
ici avec le binaire de ce worktree **retrouvent à l'identique** les chiffres des comptes rendus (R2-trad, R2-styles B8,
B3, B5, B6) ; c'est la preuve que la variance mesurée plus haut et la distance au corpus décrivent le même état du moteur.

**Profils trad** (`make v2-distance-trad`, `harmonium-lab trad-compare`, 16 graines × 32 mesures, écart moyen |z| des écarts signalés, **plus petit = plus proche**, bruit de tirage ± 0,07 ; `hornpipe_britannique_coloniale` : aucun air de hornpipe dans le corpus, non mesuré) :

| Profil | Référence du labo | |z| (corpus de la tradition) | |z| (corpus québécois par défaut, comme R2-trad) | R2-trad « après » |
|---|---|---:|---:|---:|
| `reel_quebecois` | `trad-reel` | 0.430 | 0.430 | 0.430 |
| `reel_acadien` | `trad-acadienne-reel` | 0.764 | 0.672 | 0.672 |
| `reel_britannique_coloniale` | `trad-britannique_coloniale-reel` | 0.446 | 0.845 | 0.845 |
| `gigue_quebecoise` | `trad-jig` | 0.587 | 0.587 | 0.587 |
| `gigue_acadienne` | `trad-acadienne-jig` | 0.782 | 0.700 | 0.700 |
| `gigue_britannique_coloniale` | `trad-britannique_coloniale-jig` | 0.596 | 0.650 | 0.650 |
| `marche_britannique_coloniale` | `trad-britannique_coloniale-march` | 0.908 | 1.012 | 1.012 |
| `valse_quebecoise` | `trad-waltz` | 0.632 | 0.632 | 0.632 |
| `valse_britannique_coloniale` | `trad-britannique_coloniale-waltz` | 1.002 | 0.721 | 0.721 |

La colonne « corpus québécois » retrouve **les neuf valeurs de R2-trad** : R2-trad comparait les profils acadiens et britanniques aux airs réels du corpus québécois (`trad-compare` charge `corpus/trad` par défaut). La colonne « corpus de la tradition » est la comparaison cohérente : acadien +0,09 (reel), +0,08 (gigue), britannique −0,40 (reel), −0,05 (gigue), −0,10 (marche), **+0,28 (valse britannique, 1,002)** ; la valse britannique est le profil trad le plus loin de son corpus (60 airs réels, un accord par mesure là où 32 % des mesures chiffrées en portent deux : R1 § Mesures).

**Batterie des profils d'accompagnement contre le GMD** (`make v2-distance-batterie` : scripts `b8/distance_gmd.py`, `b6/distance_gmd.py`, `b25_distance.py`, `b3/*_distance.py` du moteur, 16 graines × 48 mesures ; Bray-Curtis des fréquences d'attaque par pas, moyenne de tout / grosse caisse / caisse claire / charleston, **0 = identique, plus petit = plus proche**) :

| Profil | Script | Profil distribué | Copie sans arc ni session (ou témoin B2/B5) | Copie non calibrée |
|---|---|---:|---:|---:|
| `gmd_rock` | `b8/distance_gmd.py` | 0.184 | 0.236 | 0.500 |
| `gmd_pop` | `b8/distance_gmd.py` | 0.171 | 0.170 | 0.479 |
| `gmd_hiphop` | `b8/distance_gmd.py` | 0.147 | 0.156 | 0.543 |
| `gmd_soul` | `b8/distance_gmd.py` | 0.252 | 0.247 | 0.525 |
| `gmd_reggae` | `b8/distance_gmd.py` | 0.179 | 0.186 | 0.595 |
| `gmd_dance_disco` | `b8/distance_gmd.py` | 0.163 | 0.111 | 0.643 |
| `gmd_afrobeat` | `b8/distance_gmd.py` | 0.152 | 0.114 | 0.548 |
| `gmd_blues_shuffle` | `b6/distance_gmd.py` | 0.090 | 0.097 | 0.666 |
| `wjazzd_ballad` | `b25_distance.py` | 0.510 | 0.346 (témoin) | — |
| `gmd_jazz_waltz` | `b25_distance.py` | 0.380 | 0.542 (témoin) | — |
| `gmd_latin_brazilian_bossa` | `b3/gmd_latin_brazilian_bossa_distance.py` | 0.203 | 0.194 | 0.490 |
| `gmd_latin_brazilian_samba` | `b3/gmd_latin_brazilian_samba_distance.py` | 0.354 | 0.367 | 0.735 |
| `gmd_latin_brazilian_baiao` | `b3/gmd_latin_brazilian_baiao_distance.py` | 0.157 | 0.074 | 0.661 |
| `gmd_afrocuban` | `b3/gmd_afrocuban_distance.py` | 0.301 | 0.300 | 0.559 |

Les quatorze valeurs « profil distribué » sont **identiques à celles des comptes rendus** (B8 : 0,184 / 0,171 / 0,147 / 0,252 / 0,179 / 0,163 / 0,152 ; B6 0,090 ; B2 + B5 0,510 et 0,380 ; B3 0,203, 0,354, 0,157, 0,301). Seul le **témoin** de `wjazzd_ballad` diffère du compte rendu B2 (0,346 contre 0,651) : c'est le `percussion.toml` de `wjazzd_swing`, que B1 a réécrit (patterns et relances du GMD jazz) après B2, le jour même ; la valeur du témoin n'est pas stable, celle du profil l'est. Le témoin de la valse est celui de la fixture d'exercice (0,542, identique).

**Reprises des comptes rendus, non rejouées** (profil inchangé depuis, `git log` ; les outils demandent l'audio ou une
copie dédiée du profil) : jeux (`make style-distance`, R2-jeux § 4, 10 graines × 48 mesures, score 0-100 plus grand = plus
proche : `isaac_cellar` audio 50,9 / basse 78,4 / batterie 84,7 / accords 67,3 ; `isaac_basement` 40,4 et 37,6 / 74,9 /
58,6 / 78,4 (*Diptera Sonata*) ; `isaac_burning` 37,4 / 67,8 / 82,8 / 61,1 ; `isaac_downpour` 56,7 / 78,9 / 62,6 / 86,4 ;
`nt_desert` *Fläshyn* 58,5 / 86,1 / 63,0 / 82,2), `gmd_funk` et `gmd_neworleans_funk_secondline` (`make style-distance` sur le corpus
GMD, R2-styles : groupe batterie 76,6 et 72,7), `wjazzd_swing` (basse contre WJazzD 0,158 ; mélodie contre le corpus 0,323 :
B1, E12). Sans corpus de référence : `wjazzd_ballad` et `gmd_jazz_waltz` pour la basse (aucune mesure 3/4 dans WJazzD, aucune
basse de ballade lisible), `gmd_hiphop` et `gmd_afrobeat` pour la basse et le comping, tous les profils pour le comping (pas de
corpus de comping).

## Manques relevés (pour A7 et pour training)

1. **Un `breaks.start` plus grand que la grille est refusé au chargement** (`arc.stages[1].variants[0].breaks[0].start: mesure du
   chorus requise…`). Mesuré : `gmd_afrocuban`, `gmd_funk`, `gmd_latin_brazilian_*` exigent une grille de **9 mesures ou plus**,
   `gmd_jazz_waltz` et `wjazzd_ballad` de **24 ou plus** (AABA de 32). Une chanson iReal de 16 mesures en valse jazz n'est pas
   jouable sans retirer les breaks de la copie. Demande : ignorer (ou borner) un break hors chorus, ou en faire un avertissement.
2. **Imposer une variante de session** (« les balais quelle que soit la graine ») : aucune commande ; l'hôte ne peut que
   remplacer `[session]` dans sa copie du profil. Déjà dans la liste d'A7 ; training en a besoin pour Ballad / Brushes et Guitar Trio.
3. **Basse en riff identique d'un chorus au suivant (9 profils GMD)** : 2 à 6 % de mesures changées, 21 à 53 % de paires de
   chorus identiques ; l'arc ne choisit pas les motifs de basse. Déjà dans la liste d'A7.
4. **`intensity` publiée = celle de l'hôte** (0,50 constante) : une mesure de l'arc par l'intensité n'est pas possible ; elle se lit
   dans la densité et la vélocité. Si l'arc doit être vérifiable de l'extérieur, publier l'intensité effective (`effective_intensity`).
5. **Deux `program_change` au même tick** : le MIDI exporté écrit le programme du profil puis celui de la session sur le même
   canal et le même tick (le dernier sonne) ; inoffensif pour le rendu, trompeur pour un lecteur qui trie les événements (la
   mesure de V0 s'y est trompée). Écrire seulement le programme effectif éviterait l'ambiguïté.
6. **Profils de jeu `loop` sans grille utilisateur** : leurs motifs de basse sont rattachés à des parties nommées ; ils ne
   peuvent pas servir de modèle à training (le brief D1 les écarte de toute façon).

## Reproduire

```bash
. ~/orca/harmonium-019/env.sh && cargo build --release -p harmonium-cli                     # moteur
cd <labo> && make v2-variance ENGINE_DIR=<moteur> ENGINE_BIN=<moteur>/target/release/harmonium-cli
make v2-distance-trad ENGINE_DIR=… ENGINE_BIN=… CORPUS_ROOT=<corpus/>    # écart des profils trad à leur corpus
make v2-distance-batterie ENGINE_WORKTREE=<moteur compilé>             # batterie des profils GMD contre le GMD
```

Aucune lecture audio, aucun lancement du studio. Pas de lien d'écoute : aucun profil n'est touché (l'écoute reste facultative, D5).
