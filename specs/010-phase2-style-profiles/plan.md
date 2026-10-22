# Phase 2 — Profils de style : un style = des données

**Statut** : T1–T5 mergées (2026-10-22) ; reste le critère 4 (écoute du propriétaire : kits
`harmonium_lab/target/blindtest/{gigue-1,valse-1}`, démos `demos/{gigue,valse}_seed1.wav`) |
**Dépend de** : phase 1 (reel validé pour la démo)

Bilan :
- P2-T1 : schéma de profil v2, `profiles/reel_quebecois/`, générateur piloté par le profil ; D2 prouvé
  (22 558 notes identiques). `--check-cells` valide la longueur de cellule du profil choisi.
- P2-T2 (labo) : formes, cellules de 6 croches, harmonie et tempos de gigue et valse.
- P2-T3/T4 : `gigue_quebecoise` (6/8) et `valse_quebecoise` (3/4, oom-pah-pah) en données seulement.
- P2-T5a/b/c : inventaire Legacy ; portage de la théorie (`theory`), de Bjorklund (`algo::euclidean`,
  motifs de percussion euclidiens) et de la grammaire de Steedman (`models/jazz`,
  `harmony.model = "jazz_functional"`, fixture `tests/fixtures/profiles/jazz_test/`) ; suppression du
  chemin Legacy (−22 643 lignes, tag `pre-legacy-removal`). Le moteur ne tourne plus que sur profils.
- Contrôle des kits d'écoute : réels restreints aux airs avec accords (sinon l'absence de basse
  trahissait les vrais) ; aucune mesure simple ne sépare réels et générés.
- Labo (écart connu, à corriger) : `trad-compare` compte ~1,5 changement d'accord par mesure pour le
  moteur alors que ses snapshots en ont ~0,25 ; la mesure infère les accords depuis les voicings MIDI.

## But

Configurer le moteur pour un style en écrivant un **profil** (TOML), sans toucher au code. Le reel
québécois devient un profil parmi d'autres ; gigue 6/8 et valse 3/4 le prouvent. Plus tard, d'autres
traditions (acadienne, britannique coloniale, autochtones) et d'autres genres (jazz, comme
l'ancien moteur) s'ajoutent de la même façon (constitution v3, principe V).

## Problème

Ce qui définit le reel est aujourd'hui à moitié en données (cellules, pieds, arrangement, réaction
aux émotions : `harmonium_core/src/trad/reel_*.toml`) et à moitié en code :

- `TuneKind::Reel` (seule variante) : signature 2/2, 16 pas par mesure, croche = 2 pas, pulse à la
  blanche, codés en `const fn` ;
- forme AABB fixe, parties de 8 mesures (`BARS_PER_STRAIN`, `BARS_PER_TOUR`, `TUNE_HARMONY_BARS`) ;
- rôles par position dans la partie codés en dur (`TunePlan::bar_role` : m.1 `Open`, m.4 `CadHalf`,
  m.5–6 copies de 1–2, m.8 `CadFull`) ;
- cellules de 8 croches (`NOTES_PER_CELL`), feet `REEL_STEPS`, harmonie à la demi-mesure ;
- l'ancien `StyleProfile` (`style_profile.rs`) n'est qu'une surcouche de `TuningParams` de
  l'époque Legacy.

## Décisions

### D1 — Un profil de style = un dossier de données

`profiles/<id>/profile.toml` (+ fichiers référencés : cellules, pieds, arrangement, dynamique).
Sections :

| Section | Contenu |
|---|---|
| `[meta]` | nom, tradition précise (région, époque), tags, corpus de référence (manifeste lab) |
| `[meter]` | signature, pas par mesure, pas par unité de cellule, pulse dansée (`steps_per_pulse`, `pulses_per_bar`), swing par défaut, tempos (défaut, min, max) |
| `[form]` | parties (A, B, C…), mesures par partie, reprises, schéma joué (AABB, AABBCC…), rôles par position (ouverture, demi-cadence, cadence, copie de mesure) |
| `[melody]` | banque de cellules (rôles, poids), longueur de cellule en unités, registre, tessiture, tonalités et modes permis |
| `[harmony]` | modèle d'harmonie (`trad_functional` aujourd'hui ; `modal`, `drone`, `none`, `jazz_functional` plus tard), rythme harmonique (par mesure / demi-mesure), table de degrés |
| `[accompaniment]` | motifs par piste en pas (chop, basse, oom-pah-pah de valse…), vélocités, durées |
| `[percussion]` | table de pieds/tambour par densité, notes et banque SF2 |
| `[instruments]` | piste → programme/banque SoundFont |
| `[arrangement]`, `[intro]` | tours, entrées, intros, fin (format P1-T8/T11) |
| `[dynamics]` | réaction aux 4 émotions (format `reel_intensity.toml`, P1-T12/T13) |

Le profil remplace l'ancien `StyleProfile` (un seul concept, pas de chemin parallèle).
`Style { Legacy, Reel }` devient : le générateur Legacy (tant qu'il existe) ou le générateur de
profils + un identifiant de profil.

### D2 — Le reel ne change pas d'une note

Critère de non-régression du refactor : pour les mêmes graines, émotions et automations, les
**notes** (piste, hauteur, début, durée, vélocité) de `reel_quebecois` sont identiques à `main`.
Le format des snapshots peut changer (bump `TUNING_VERSION`, goldens régénérés).

### D3 — Les algorithmes deviennent génériques, une fois

Ce qui suppose 2/2, 8 croches, 8 mesures ou AABB est réécrit pour lire le profil : grille,
forme et rôles, longueur de cellule, rythme harmonique, motifs d'accompagnement et de percussion.
Pas de `match` sur un nom de style.

### D4 — Preuve : gigue 6/8 et valse 3/4 en données seulement

Corpus du labo : 88 gigues, 67 valses. Le labo extrait cellules (6 croches), formes, accords par
position et tempos ; on écrit `profiles/gigue_quebecoise/` et `profiles/valse_quebecoise/`.
Si un algorithme manque (ex. accompagnement oom-pah-pah), il s'ajoute en code générique, puis
le profil l'utilise. Critère : `harmonium-cli --export --style gigue_quebecoise` rend une gigue
reconnaissable à l'écoute ; aucun fichier `.rs` ne mentionne « gigue » ou « valse ».

### D5 — Legacy : trier avant de supprimer (P1-T6 redéfinie)

L'ancien moteur (`harmony/*` : grammaire de Steedman, néo-riemannien, Lydian chromatique,
conduite de voix ; `sequencer.rs` : rythmes euclidiens ; `timeline/generator.rs`) faisait du
jazz. Le propriétaire veut pouvoir refaire du jazz. On inventorie : ce qui sert un futur profil
(ex. grammaire de Steedman → modèle d'harmonie `jazz_functional`) est porté comme modèle
générique sélectionnable par profil ; le reste est supprimé avec le chemin Legacy.
Un profil jazz complet (corpus, mélodie) est une phase ultérieure.

## Découpage

| Tâche | Contenu | Dépend de | Repo |
|---|---|---|---|
| P2-T1 | Schéma de profil v2 + chargeur validé ; `profiles/reel_quebecois/` ; générateur lit grille, forme, rôles, longueur de cellule, rythme harmonique, motifs depuis le profil ; CLI `--style <id>` / `--profile <dossier>` ; D2 prouvé | — | moteur |
| P2-T2 | Labo : formes, cellules de 6 croches, accords par position, tempos pour gigue et valse ; sortie au format de profil ; `make profile-draft TYPE=jig` | — | labo |
| P2-T3 | `profiles/gigue_quebecoise/` (6/8, pulse à la noire pointée, pieds 6/8) + algorithmes génériques manquants | T1, T2 | moteur |
| P2-T4 | `profiles/valse_quebecoise/` (3/4, oom-pah-pah, harmonie à la mesure) | T1, T2 | moteur |
| P2-T5 | Inventaire Legacy : porter (modèles génériques) ou supprimer ; exécute P1-T6 | T1 | moteur |

T1 et T2 en parallèle (dépôts différents). T3 et T4 en parallèle après T1+T2.

## Critères d'acceptation

1. D2 : notes du reel identiques à `main` (même graine, mêmes émotions, même automation).
2. Gigue et valse rendues depuis leurs profils, aucune mention de style dans le code.
3. Les 4 émotions agissent sur chaque profil via sa section `[dynamics]`.
4. Écoute du propriétaire : « c'est une gigue », « c'est une valse ».
5. `cargo test --workspace`, clippy sans nouvel avertissement hors tests, fmt.
