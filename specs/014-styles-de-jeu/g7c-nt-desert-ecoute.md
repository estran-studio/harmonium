# G7c — Désert `nt_desert` après écoute (données seulement)

Retour du propriétaire sur les rendus G7 : « gros bruit de buzz, batterie
super en avant qui ne va pas avec le son des autres instruments, beat rigide
sans fluidité, parfois des solos bizarres au milieu, beaucoup de
sifflements ». Diagnostic mesuré (graine 1, 32 mesures, automation neutre) :
deux scies 81 empilées (basse D1–G#1 ET accords, 62 notes d'accords sur 80
sous MIDI 48) ; charleston sur les 12 pas pairs (12 coups/mesure), vélocités
plates 50–58 sans accent ; mélodie sifflée (programme 78, 77 notes / 32
mesures) tirée de cellules génériques. Dans *Fläshyn* il n'y a pas de lead
sifflé (il appartient à *A Way*), et la transcription du motif de *A Way*
est trop incertaine (voiced_ratio ~7 %) pour justifier un motif rare.

Recette appliquée : les essais du propriétaire (mêmes notes retouchées en
MIDI, rendues GeneralUser — `ecoute_g7/essai_v1..v3`) : sifflement retiré,
accords en nappe chaude 89 (vél. ×0,8), charleston sur les pulsations,
batterie −4 dB avec accents (temps fort plein, autres ×0,8) et ±2 % de
décalage, kit GS Power / TR-808. Aucun code moteur touché.

## Ce qui a changé (données seulement)

- `profiles/nt_desert/arrangement.toml` : entrée `Melody` supprimée — aucun
  sifflement ne sonne (mécanisme G4). La section `[melody]` et `cells.toml`
  restent pour le plan interne du vote harmonique.
- `profiles/nt_desert/profile.toml` : `chords_program` 81 → 89 (nappe
  chaude), `center` 44 → 60 (tessiture médium), vélocité du coup lié 66 → 53
  (×0,8) ; `[performance.tracks.Heel/Toe]` : `jitter = 0.02` (micro-timing et
  dispersion) + règle `Accent` (+4 vélocité) au pas 0 ; mix batterie −9/−10 dB
  (−4 dB vs G7) avec envois de salle 0,08 → 0,15. Une seule voix grave : la
  scie 81 ne reste que sur `Bass`.
- `profiles/nt_desert/feet.toml` : charleston (touche 42) sur les pulsations
  (pas 0/6/12/18) au lieu des 12 pas pairs ; accents par temps dans les
  motifs (temps fort plein, autres ×0,8 : pieds 92 → 74, 84 → 67) ; relances
  de densité 2 inchangées (pas 16/22).
- `profiles/nt_desert/dynamics.toml` : seuil `Melody` retiré de
  `density_tracks` (donnée morte) ; à density 0,2 ne reste que la nappe.
- `tests/fixtures/profiles/gigue_bourdon/profile.toml` (principe VIII) :
  `jitter = 0.02` sur `Heel`/`Toe`, sans règle d'accent — ni hauteurs, ni
  vélocités, ni durées ne changent.
- Tests : `nt_desert_profile_tests` réécrit au nouveau contrat (6 tests) ;
  `part_tracks_off_tests` gagne la preuve VIII (micro-timing publié sur les
  pieds de la gigue).
- Docs : `profiles/nt_desert/README.md` (table des parties, groove, écarts),
  `profiles/README.md` (ligne `nt_desert`, § Interprétation : les pieds de
  `nt_desert` s'ajoutent à la mélodie des trads).

Arche de *Fläshyn*, pont sans batterie ni basse (G7b) et 4 émotions inchangés.

## Avant / après chiffré (graine 1, 32 mesures neutres, snapshots)

| Mesure | Avant (G7) | Après (G7c) |
|---|---:|---:|
| Notes de sifflement (piste 0) / 32 mes. | 77 | **0** |
| Coups de charleston / mesure | 12 (tous les pas pairs) | **4** (pulsations 0/6/12/18) |
| Coups de pieds / 32 mes. (16 avec batterie) | 240 | **112** (3 kick + 3 snare + 4 hat × 16) |
| Notes d'accords sous MIDI 48 | 62 / 80 | **0 / 80** (nappe ≥ 54 : G#3 en pédale, triades C–E–F–B… au pont) |
| Notes de basse | 58 | 58 (riff inchangé, seule voix grave) |
| Vélocités charleston | 50–58 plates | **63–70, 90 au temps fort** (étiquette `Accent` au pas 0) |
| Décalages publiés des pieds | 0 | **±20 millièmes de pas** (jitter ±0,02) |
| Mix batterie (Heel/Toe) | −5/−6 dB, salle 0,08 | **−9/−10 dB, salle 0,15** |

Vérifié au MIDI exporté (`/tmp/g7c-nt-desert-ecoute.mid`, 241 notes, graine 1,
32 mes.) : canal 0 vide, accords ≥ 54, basse ≤ 32.

## Preuves de test

- `nt_desert_profile_tests` : 6 tests verts (nom + contenu 89/sans entrée
  Melody, arche 48 sans sifflement + triades + pont muet + reprise A1',
  pédale liée une attaque — nappe G#3 mes. 1–8 — sans orpheline, groove
  exact + accents + micro-timing borné, presets, calme density 0,2 = nappe
  seule).
- `part_tracks_off_tests` : 4 tests verts (pont muet + reprise, B de gigue
  sans pieds, validation ×5 cas, micro-timing VIII).
- Non-régression : goldens `main_golden_*` des 10 profils trad intacts, sans
  régénération ; `TUNING_VERSION = 21` inchangé (données `nt_desert` et
  fixture hors goldens ; aucun `harmonium_*/src` touché).
- `make quality` vert : fmt, clippy, workspace + doctests.
- Smoke : `--export --bars 32 --profile profiles/nt_desert --seed 1` (MIDI
  2 125 octets, bilan ci-dessus) et fixture `gigue_bourdon` (3 861 octets,
  mélodie intacte : 113 notes canal 0).

## Limites

- Kit inchangé (128/0, rock sec de `nt_desert.sf2`) : les essais Power/808 du
  propriétaire sont des kits GeneralUser ; le choix du kit reste à trancher à
  l'oreille entre les deux banques.
- Micro-timing et accents repris des essais tels quels (±0,02 pas, +4
  vélocité au temps fort) : valeurs d'écoute, pas de mesure de corpus.
- La nappe 89 ne change pas de programme par partie (une seule nappe pour
  pédale et triades, contrainte programme-unique-par-piste).
- `melody_program = 78` reste déclaré (champ requis par le schéma) sans
  jamais sonner ; `[mix.tracks.Melody]` de même.

## Ce qu'il faut écouter (`corpus/nuclear_throne/ecoute_g7/`, hors dépôt)

64 mesures, graine 1, `--feet-soundfont` explicite (kit du profil, pas les
pieds de podorythmie) : `nt_desert_seed1.wav` (banque `nt_desert.sf2`,
écrasé), **`nt_desert_seed1_generaluser.wav`** (GeneralUser GS, nouveau),
plus les 4 variantes `nt_desert_{arousal,valence,density,tension}.wav`
(réécrites, banque `nt_desert.sf2`).

| Minutage (neutre, ≈ 110 s) | Écouter |
|---|---|
| 0–13,7 s (mes. 1–8) | Pédale G# : scie grave seule en bas + nappe deux octaves au-dessus, **sans battement de scies**, sans sifflement |
| 13,7–27,4 s (mes. 9–16) | Riff D/D#, charleston aux 4 pulsations, temps fort marqué |
| 27,4–54,9 s (mes. 17–32) | Pont : nappe 89 seule (triades C–E–F–B…), **sans batterie ni basse ni sifflement** |
| 54,9–68,6 s (mes. 33–40) | Alternance G/G#, batterie en groove (comparer la fluidité avec G7) |
| `density` | Nappe seule façon *A Way* (valider le calme) |
| `seed1_generaluser` vs `seed1` | Même notes, timbres GM vs banque du jeu ; la batterie vient du kit du profil dans les deux |
