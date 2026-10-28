# G7d — Mixage de `nt_desert` (données seulement)

Retour du propriétaire sur le rendu Odin : « j'ai l'impression d'écouter une
TV sans signal quand c'est juste un long son ». Hypothèse posée : la basse
Odin trop faible (−31 à −43 dBFS selon le preset, contre −21 dBFS pour la
scie SoundFont 81) laisse la batterie et sa réverbération dominer pendant
les tenues. **Vérifié par mesure, pas à l'oreille** (graine 1, pistes seules,
`--solo-track`, 44,1 kHz) : l'hypothèse de niveau est confirmée, celle du
« souffle » est plus subtile (voir § Cibles et § Limites).

- Le stem Toe (caisse claire + charleston 42) porte à lui seul **toute** la
  bande 5–12 kHz du rendu Odin pendant les tenues (3,2 % de l'énergie totale
  du mix, contre 0,0 % pour basse et nappe) : le charleston est bien le seul
  son de bruit en jeu.
- Mais le rendu Odin n'était **pas** plus plat ni plus riche en aigus que
  *Fläshyn* (part 5–12 kHz du mix : 2,6 % contre 5,5 % pour la référence à
  l'ouverture ; platitude 3,2×10⁻² contre 6,9×10⁻²). Le défaut entendu est un
  défaut de **rapport** (basse −14 dB sous la batterie), pas un excès
  spectral : le mix référencé est lui-même riche en charleston.

## Cibles documentées

1. **Basse ≥ batterie + 3 dB RMS pendant A1** (somme des stems Heel+Toe pour
   la batterie), dans les trois rendus : banque `nt_desert.sf2`, GeneralUser,
   Odin usine. Fenêtre principale : mesures 1–8 ; fenêtre « tenues » :
   mesures 1–3 (pédale G# liée avant la relance chromatique).
2. **Nappe ≥ batterie** quand la basse ne joue pas (B, mesures 33–40).
3. **Part 5–12 kHz du mix pendant les tenues proche de celle de *Fläshyn***
   (5,5 % à l'ouverture, 4,8–7,6 % selon les sections). Mesurée avant/après ;
   la cible 1 (basse +17 dB) la fait mécaniquement chuter — voir § Limites.

## Ce qui a changé (données seulement, aucun code moteur)

- `profiles/nt_desert/profile.toml`
  - `[mix]` : le gain master (14 dB) ne s'applique qu'aux pistes SoundFont,
    pas aux voix Odin (G6b) ; il descend à **9 dB** et basse/nappe passent à
    **0 dB** (−5/−4) : les trois rendus gardent leur équilibre SoundFont
    (−0 dB net) et l'Odin gagne +5 dB de piste. Batterie **−5/−6 dB**
    (−9/−10), envois de salle des pieds **0,06** (0,15) : la queue de
    réverbération du charleston ne tient plus la place pendant les tenues.
  - `[odin.Bass]` : règle `master_volume` (arousal 0,77, density 0,77) — le
    preset d'usine sort à −15 dB de master (socle 0,178) ; au neutre le
    morphing porte le master à ≈ ×2 (borne 2,0), la voie prévue par le
    schéma. Avec le niveau de piste, la basse Odin remonte de 17 dB.
- `profiles/nt_desert/feet.toml` : le charleston (touche 42, seul son de
  bruit employé) est **réduit aux accents de temps** — pas 0 et 12 à la
  densité neutre, pulsations 6/18 seulement à la densité 2 (arousal haut) —
  et **sous la caisse claire** : deltas −20 (64), −30 (54), −32 (52),
  −40 (44) sur une base de 84, contre 67 pour la caisse claire. Aucune
  cymbale (crash 49, ride 51) n'est employée : le profil n'en a jamais joué.
  Grosse caisse et caisse claire inchangées (le kick domine le RMS batterie,
  ce n'est pas lui le problème).
- Principe VIII : `tests/fixtures/profiles/valse_nappe/profile.toml` gagne
  la même règle `master_volume` morphée par arousal/density sur sa nappe
  (autre style, 3/4) — la règle n'était employée par aucune fixture.
- Tests : `nt_desert_profile_tests` au nouveau contrat charleston
  (pas {0, 12} exacts, accents conservés, **moyenne sous la caisse claire**
  — l'accent +4 du temps fort et l'humanisation ±4 interdisent la marge
  coup à coup) ; `profile_files_tests` épinglait `master_db = 14.0`
  (littéral mis à jour).
- Docs : `profiles/nt_desert/README.md` (groove et mix), `profiles/README.md`
  (§ Odin « Niveau » : le master du preset, fixe ou morphé par les émotions).

## Avant / après chiffré (graine 1, A1, RMS en dBFS)

| Rendu | Basse | Batterie | **Basse − batterie** | Nappe | 5–12 kHz mix |
|---|---:|---:|---:|---:|---:|
| sf2 avant (mes. 1–8 / tenues) | −21,4 / −22,1 | −29,2 | **+7,8 / +7,1** | −28,0 | 0,58 % / 0,64 % |
| sf2 après | −21,4 / −22,1 | −30,2 | **+8,8 / +8,1** | −29,0 | 0,23 % / 0,26 % |
| GeneralUser avant | −25,8 / −27,6 | −35,7 | **+9,9 / +8,3** | −34,9 | 0,18 % / 0,21 % |
| GeneralUser après | −25,8 / −27,6 | −36,7 | **+10,9 / +9,2** | −35,9 | 0,14 % / 0,15 % |
| Odin avant | −42,6 / −43,5 | −29,2 | **−13,4 / −14,2** | −33,6 | 3,54 % / 2,60 % |
| Odin après | −25,4 / −26,3 | −30,2 | **+4,8 / +3,9** | −29,6 | 0,43 % / 0,35 % |

Référence *Fläshyn* (stems demucs, même fenêtre) : basse ≈ batterie
(±2 dB selon la section), part 5–12 kHz 5,5 %, platitude 6,9×10⁻².

Autres mesures clés :

- **Platitude du mix Odin pendant les tenues** (0,1–12 kHz) : 3,2×10⁻² avant
  → **7,2×10⁻³ après** : le « son TV » recule de 4×, la basse portante
  remplit le spectre.
- **Attribution 5–12 kHz** (Odin avant) : Toe 3,21 % de l'énergie totale,
  basse 0,00, nappe 0,00, Heel 0,00 — le charleston seul.
- **Part 5–12 kHz de la batterie** : 4,5 % avant → 2,1 % après (sf2 et Odin ;
  1,2 → 0,8 % en GeneralUser) : le kit sonne plus grave, la caisse claire
  et le kick gardent leur place.
- **A2 (mes. 9–16)** : basse−batterie Odin **+4,8 dB** après (−13,3 avant) ;
  sf2 +5,7 ; GeneralUser +12,4 (batterie GM faible en absolu, non retouchée).
- **B (mes. 33–40)**, basse quasi absente par données (comme l'original) :
  nappe ≥ batterie dans les trois rendus (Odin : −22,7 contre −30,4).
- **Extrêmes d'émotions** (Odin, automation) : arousal=density=tension=1,0 →
  crête −2,4 dBFS (pas de clip), basse−batterie +1,3 dB (la batterie suit
  l'arousal, la basse la suit aussi) ; arousal 0,0 → −14,9 dBFS. Le +3 dB
  tient au neutre ; aux extrêmes la marge se resserre sans s'inverser.

## Preuves de test

- `nt_desert_profile_tests` : 6 tests verts au nouveau contrat (charleston
  {0, 12}, accents et micro-timing inchangés, moyenne charleston +3 sous la
  caisse claire sur 16 mesures).
- `profile_files_tests` : 10 verts (littéral `master_db` à jour).
- `part_tracks_off_tests` : 4 verts (pont muet, reprise, fixture gigue).
- `harmonium --features odin --test odin_contract_tests` : 5 verts
  (déterminisme, morphing, Tied, bascule de pistes) — la nouvelle règle
  `master_volume` passe le contrat sans changement de code.
- `style_profile_tests` (goldens `main_golden_*` des 10 profils trad) : 7
  verts, **sans régénération** ; `TUNING_VERSION` inchangé (aucun fichier
  `harmonium_*/src` touché).
- `make quality` vert (fmt + clippy + workspace).
- Smoke : `--export --bars 32 --profile tests/fixtures/profiles/gigue_bourdon
  --seed 1 --record-midi /tmp/g7d-gigue-smoke.mid` : 3 861 octets (taille des
  smokes G7b/G7c, fixture et moteur inchangés), mélodie intacte (153 notes
  canal 0, vérifiées au MIDI) ; `--export --bars 32 --profile
  profiles/nt_desert --seed 1 --record-midi /tmp/g7d-nt-desert-mix.mid` :
  canal 0 vide, accords ≥ 54, basse 26–32, 130 coups de kit sur les 16
  mesures à batterie (3 kick + 3 caisse claire + 2 charleston par mesure).

## Limites

- **Cible 3 non tenue, par construction** : remonter la basse de 17 dB
  dilue la part 5–12 kHz (le dénominateur croît) ; la ramener à ~5,5 %
  exigerait un charleston *plus* fort qu'avant G7d — contradictoire avec la
  consigne de le baisser. Nos kit et nappes restent plus sombres que la
  référence (part 5–12 kHz des stems tonals : ~0 % contre 1,5–1,9 % pour
  *Fläshyn*). Levet restant : charleston de la banque plus riche, ou nappe
  plus brillante (filtre du preset) — à trancher à l'oreille, pas fait ici.
- Le charleston « aux accents » s'entend comme **accents de temps**
  (pas 0/12) et non accents de section : le schéma des `[[patterns]]` est
  par densité, sans dimension de partie ; aucune cymbale n'existe dans le
  profil pour marquer les sections.
- Le preset `basse_saturee` (Sausage Bassener, −15 dB de master, filtre à
  114 Hz) est responsable de l'essentiel de l'écart : la règle
  `master_volume` compense au neutre ; un preset mieux calibré rendrait la
  compensation inutile.
- Le morphing `master_volume` suit arousal et density : à arousal 0,0 la
  basse Odin retombe à −31 dB (niveau de piste seul). C'est cohérent avec
  le pilotage (calme = tout baisse), mais la cible +3 dB n'est garantie
  qu'au neutre et aux arousal/density hauts.
- Stems demucs de la référence fragiles sur une piste chiptune : les
  niveaux absolus par stem de *Fläshyn* sont indicatifs ; seule la part
  5–12 kHz du mix complet (5,5 %) est solide.

## Ce qu'il faut écouter (`corpus/nuclear_throne/ecoute_g7/`, hors dépôt)

64 mesures, graine 1 (mesure = 1,714 s) : **`g7d_sf2.wav`** (banque
`nt_desert.sf2`), **`g7d_generaluser.wav`**, **`g7d_odin_usine.wav`**
(presets d'usine Sausage Bassener + Darken Nature, lus hors dépôt via
`--odin-presets` ; aucun `.odin` commité).

| Minutage | Écouter |
|---|---|
| 0–13,7 s (A1) | La pédale G# **tient le premier plan** dans les trois rendus ; pendant les tenues, plus de « TV sans signal » — le charleston ne passe plus devant |
| 13,7–27,4 s (A2) | Riff D/D# : la basse domine toujours ; le groove garde kick/snare, le charleston ne siffle plus |
| 27,4–54,9 s (pont) | Nappe seule (inchangée) |
| 54,9–68,6 s (B) | Nappe au-dessus de la batterie, basse quasi absente comme l'original |
| `g7d_odin_usine` contre `g7d_sf2` | Même notes : la basse saturée doit sonner **au-dessus** du kit, pas en dessous ; comparer avec `odin_usine_deep.wav`/`odin_nt_desert_seed1.wav` (avant G7d) pour le contraste avant/après |
