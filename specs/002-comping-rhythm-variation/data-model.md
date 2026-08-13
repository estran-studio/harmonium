# Phase 1 — Modèle de données : la voix d'accords

Normatif pour les cas limites. Quand ce document et l'intuition divergent,
ce document gagne.

## `TrackId::Chord`

Cinquième variante de `TrackId` (`harmonium_core/src/timeline/mod.rs:45`),
canal MIDI 4.

```
Bass  → 0     Lead  → 1     Snare → 2     Hat → 3     Chord → 4
TrackId::ALL : [Bass, Lead, Snare, Hat, Chord]   // longueur 5
```

L'ordre canonique place `Chord` en dernier : `ALL` est itéré par
`MeasureSnapshot::from_measure` et par `Playhead::tick`, et l'ajout en queue
laisse les index existants stables.

## `CompingParams`

Dixième sous-structure de `TuningParams` (`harmonium_core/src/tuning.rs:434`),
avec `serde(default)` pour que les profils de style existants se chargent
sans modification.

| Champ | Type | Bornes | Sens |
|---|---|---|---|
| `hits_per_bar` | f32 | 0.0 ..= steps de la mesure (clampé) | frappes moyennes par mesure |
| `anticipation_probability` | f32 | 0.0 ..= 1.0 | probabilité d'une frappe sur le « et » du 4 |
| `layoff_probability` | f32 | 0.0 ..= 1.0 | probabilité d'une mesure entièrement muette |
| `charleston_probability` | f32 | 0.0 ..= 1.0 | probabilité qu'une anticipation soit liée au temps 1 suivant |
| `responsive_to_melody` | bool | — | la densité s'inverse avec celle du lead |

`validate()` vérifie les quatre f32 ; `hits_per_bar` est clampé à la
génération, pas rejeté à la validation (R4).

**Valeurs par défaut** — swing medium, le style de référence du produit :
`hits_per_bar: 2.5`, `anticipation: 0.4`, `layoff: 0.15`,
`charleston: 0.3`, `responsive_to_melody: true`. Elles doivent produire un
résultat écoutable sans aucune configuration (SC-003).

## `CompingTrigger`

Produit par le générateur, consommé par le choix des hauteurs.

```
{ step: usize, velocity: u8, duration_steps: usize, tied: bool }
```

- `step` — index 0-based dans la mesure
- `duration_steps` — **peut dépasser** le nombre de steps de la mesure : c'est
  ainsi que se représente une liaison par-dessus la barre (R3)
- `tied` — marque une anticipation Charleston. Informatif pour les exports et
  le débogage ; l'audio n'a besoin que de `duration_steps`

## Placement des frappes

Grille : `ticks_per_beat = 4`, donc une mesure 4/4 fait 16 steps.

1. **Mesure muette** — tirage unique par mesure contre `layoff_probability`.
   Si muette, zéro déclencheur, et aucun autre tirage n'est consommé pour
   cette mesure.
2. **Anticipation** — tirage contre `anticipation_probability`. Si retenue,
   une frappe est posée sur le « et du 4 » : dernière croche de la mesure,
   soit `steps - 2` sur une grille de 16 (temps 4 = step 12, son « et » =
   step 14).
3. **Charleston** — seulement si une anticipation a été posée : tirage contre
   `charleston_probability`. Si retenue, `duration_steps` porte la note
   jusqu'au temps 1 de la mesure suivante et `tied = true`.
4. **Frappes restantes** — `hits_per_bar` (moins l'anticipation déjà posée)
   réparties sur la mesure. Le placement évite la coïncidence systématique
   avec les déclencheurs de lead : c'est l'exigence FR-001, et elle se vérifie
   statistiquement (SC-001), pas par une interdiction stricte — une
   coïncidence occasionnelle est musicalement normale.

**Réponse à la mélodie** — quand `responsive_to_melody` est vrai, la densité
de la mesure est réduite proportionnellement au nombre de notes de lead de
cette mesure. Un lead vide **ne fait pas** tomber le comping à zéro : il
revient à la densité de référence (cas limite de la spec).

## Fin de la liaison en dernière mesure

Une anticipation Charleston sur la dernière mesure générée n'a pas de temps 1
où se résoudre. Décision : le déclencheur est **raccourci** à la fin de la
mesure — ni supprimé (on perdrait la frappe, audible), ni laissé pendant (le
NoteOff n'arriverait jamais et la note resterait bloquée).

## NoteOff sur la piste d'accords

Les pistes existantes coupent par remplacement — la note suivante tue la
précédente. La piste d'accords **ne peut pas** fonctionner ainsi : une
liaison par-dessus la barre doit survivre au chargement de la mesure
suivante.

Le playhead tient donc, pour la piste d'accords seule, des échéances de
NoteOff exprimées en **steps absolus depuis le début de la session**, et non
relatives à la mesure courante. Elles sont conservées à travers
`load_measure()`. À l'échéance, un NoteOff est émis pour chaque hauteur
concernée.

Les pistes Bass, Lead, Snare et Hat gardent exactement leur comportement
actuel : ce changement est additif, il ne doit modifier le son d'aucune
piste existante.

## Choix des hauteurs

Le déclencheur porte le *quand*. Le *quoi* vient du contexte d'accord de la
mesure (`Measure::chord_context`) et des paramètres existants
`voicing_density` / `voicing_tension`, portés depuis
`harmonium_audio/src/voicing/` vers `harmonium_core/src/voicing/` (R1).

Le voicing par défaut est **shell** — fondamentale omise, tierce et septième
présentes : c'est le comping jazz idiomatique, et ça laisse la fondamentale à
la basse au lieu de doubler.

## Déterminisme

Le comping tire dans un RNG enfant dérivé de `(session_seed, index de
mesure)`, jamais dans le flux principal (R5).

Invariant vérifiable, et il est la garantie de non-régression du workstream :
à graine identique, les notes de Bass, Lead, Snare et Hat sont **strictement
identiques** que le comping soit actif ou non.
