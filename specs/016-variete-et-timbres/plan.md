# Phase 016 — Variété par graine et timbre selon l'émotion

**Statut** : ouverte le 2026-10-29 sous la constitution 4.3.0. Suite de la phase 014 (styles de
jeu), close sur Nuclear Throne et des essais Isaac validés à l'écoute.

## Problème

Constats du propriétaire dans le studio (015) sur les profils `isaac_*` :

1. **Une nouvelle graine change presque rien.** Les accords et la batterie sont identiques d'une
   graine à l'autre, seule la mélodie change, peu audible. Cause dans le code :
   - `models/chord_loop.rs` : une seule boucle d'accords par partie (`HarmonyPart::ChordLoop {
     chords }`), lue dans l'ordre, sans tirage.
   - `trad/feet.rs` : `pattern(density)` prend le premier pattern de la densité ; un seul par
     densité.
   - `BassMotifPlan` tire déjà un motif pondéré par partie (`generator.rs`, sel `0x424D_4F54`),
     mais les profils `isaac_*` n'en déclarent qu'un par partie.
2. **Le timbre ne suit pas l'émotion.** Un preset Odin fixe par piste (`OdinTrackSpec { preset,
   morph }`) ; le morphing ne bouge que ses paramètres continus. Le propriétaire aime deux sons
   pour Burning Basement (A « trad » : Bass Crusher + Haunt ; B « psychédélique » : Bass Low +
   Moonshade), aujourd'hui deux profils (`isaac_burning`, `isaac_burning_psyche`).
   Échec antérieur (commit `5174899`, janvier 2026) : interpolation bilinéaire de 4 presets codés
   en dur. Interpoler des presets de structure différente (oscillateurs, filtres, routages) donne
   un hybride sans identité ; et `OdinTrackPatch` documente qu'un changement de rang de preset
   **coupe les notes** de la voix. Cette phase ne mélange pas de presets : elle **bascule** de
   l'un à l'autre en fondu.

## Tâches

| Tâche | Contenu | Où |
|---|---|---|
| V1 | Variété par graine : boucles d'accords alternatives pondérées par partie, patterns de batterie alternatifs pondérés par densité, tirés par flux dérivés de la graine ; données `isaac_*` enrichies (accords, batterie, motifs de basse, cellules de chœur) | `harmonium_core` (modèle, validation), `profiles/isaac_*`, fixtures |
| T1 | Timbre par zone d'émotion : table zone → preset par piste Odin ; bascule seulement aux frontières de partie, fondu enchaîné entre deux instances, morphing conservé dans la zone ; `isaac_burning` réunit A et B | `harmonium_core::odin`, `harmonium_host` (compositeur), `harmonium_audio` (`odin_voices`, `odin_bank`), `profiles/` |

V1 et T1 sont indépendantes et partent en parallèle. Points de contact : `style_profile.rs`
(validation), `profiles/README.md`, `profiles/isaac_burning/profile.toml` (V1 : sections
musicales ; T1 : sections `[odin.*]` et `[mix]`). Fusion : V1 puis T1, conflits résolus en
gardant les deux.

## Contrats communs

- Principe IV : un seul chemin. V1 remplace la forme à boucle unique par la forme à alternatives
  (une alternative = l'ancien cas) et migre tous les profils et fixtures ; T1 remplace
  `preset` unique par la table de zones (une zone sans bornes = l'ancien cas) et migre tous les
  profils. Aucun alias de schéma.
- Principe II et constitution : `ChaCha8Rng` seulement, flux dérivés par `ornaments::mix` avec un
  sel propre ; **aucun tirage quand il n'y a qu'un candidat**, pour que les 10 profils trad
  restent identiques (`TUNING_VERSION` et goldens inchangés).
- Principe III : rien n'alloue ni ne verrouille dans le callback ; les instances Odin se
  construisent hors callback (chemin `OdinDispatch` existant).
- Principe V et VIII : aucun nom de style en code ; chaque mécanisme employé par une fixture d'un
  autre style.
- Principe VI : tests sur le contrat (`MeasureSnapshot`, rendu), pas sur les internes.
- Ne pas toucher `harmonium_studio/` ni `specs/015-studio/` (autre session).

## Acceptation

1. V1 : sur `isaac_basement`, `isaac_cellar`, `isaac_burning`, `isaac_downpour`, deux graines
   différentes donnent des suites d'accords **et** des grilles de batterie différentes sur au
   moins une partie ; une même graine reste identique (déterminisme).
2. V1 : les 10 profils trad et `nt_desert` donnent des notes identiques à `main`.
3. T1 : `isaac_burning` bascule de A vers B quand l'émotion entre dans la zone B, à la frontière
   de partie suivante, sans coupure audible (fondu), sans réattaque fantôme ni note orpheline ;
   rendu hors ligne déterministe avec automation d'émotion.
4. T1 : profils à zone unique (tous les autres) rendus identiques à `main` avec `odin`.
5. Écoute du propriétaire sur la page LAN.
6. `cargo test --workspace` (avec et sans `--features odin`), clippy, fmt.

## État

| Tâche | État | Compte rendu |
|---|---|---|
| V1 | fusionnée (`1571654`) : `loops = [{ weight, chords }]` par partie (sel `0x4841_524D`), patterns de pieds pondérés par (partie, densité) (sel `0x4645_4554`), aucun tirage à candidat unique ; 66/66 dumps trad et `nt_desert` identiques ; `isaac_*` : 2 à 3 parties à boucles alternatives, seconds motifs de basse, 2 patterns par densité, 3 cellules de plus ; 679 tests verts | `v1-variete-graine.md` |
| T1 | lancée (2026-10-29) | `t1-timbre-emotion.md` |
