---
spec_type: per_repo_feature
meta_epic: 006-v2-ep4-improvisation-trainer
meta_repo_path: harmonium_specs/specs/006-v2-ep4-improvisation-trainer/
status: draft
priority: high
workstream: W2
strategy: STRATEGY_2026.md — The app listens (improv coach)
---

# S'entendre jouer sans lancer la grille

**Créé** : 2026-08-15 — découvert au premier essai au clavier

**Statut** : Draft

## Le problème, tel qu'il se vit

Tu ouvres l'app, tu branches ton clavier, tu poses les doigts sur les
touches pour te chauffer — **rien**. Aucun son. Il faut appuyer sur lecture,
donc lancer la grille, la batterie et la basse, pour entendre ne serait-ce
qu'une note de soi.

Pour un coach d'improvisation, c'est un défaut d'usage sérieux. On veut
tâtonner un voicing, chercher une note, vérifier qu'on est bien branché —
tout ça sans démarrer un morceau.

## Pourquoi ça se produit

Le monitoring MIDI passe par `process_live_midi()`, appelé en tête de
`process_buffer()` (`harmonium_host/src/playback.rs:238`) — c'est-à-dire
**dans le callback audio**.

Or « mettre en pause » signifie littéralement arrêter le périphérique :

```rust
// harmonium_host/src/native_handle.rs:136
pub fn pause(&self) -> Result<(), String> {
    self.stream.0.pause()   // arrête le flux cpal
}
```

Flux arrêté → callback arrêté → `process_live_midi()` n'est jamais appelé.
Et l'app démarre en `start_paused()` : avant le premier appui sur lecture,
il n'existe aucun chemin par lequel un son puisse sortir.

**Il n'y a aucune notion d'« arrêté » dans le moteur audio.** Le
`PlaybackEngine` n'a pas de drapeau de transport : le seul mécanisme d'arrêt
est la coupure du périphérique. C'est la racine, et c'est ce que cette spec
change.

## User Scenarios & Testing

### User Story 1 — S'entendre à l'arrêt (Priority: P1)

En tant qu'improvisateur, je veux entendre les notes que je joue dès que mon
clavier est branché, sans avoir à lancer la grille, pour me chauffer,
chercher un voicing ou simplement vérifier que je suis connecté.

**Test indépendant** : ouvrir l'app, activer l'entrée MIDI, ne PAS appuyer
sur lecture, jouer — on s'entend.

**Acceptance Scenarios**

1. **Given** l'app vient de démarrer et l'entrée MIDI est active, **When**
   je joue une note sans avoir lancé la lecture, **Then** je l'entends.
2. **Given** je joue à l'arrêt, **When** j'écoute, **Then** j'entends
   **uniquement** mes notes — aucune basse, aucune batterie, aucun accord :
   la grille n'avance pas.
3. **Given** je joue à l'arrêt puis j'appuie sur lecture, **When** la grille
   démarre, **Then** elle démarre à la position attendue, sans saut ni
   décalage dû au temps passé à l'arrêt.
4. **Given** je mets en pause en cours de lecture, **When** je joue,
   **Then** je m'entends toujours, et la grille reste immobile.

### User Story 2 — Ne pas payer l'inactivité (Priority: P2)

En tant qu'utilisateur, je veux que l'app ne consomme pas inutilement quand
elle est ouverte et silencieuse depuis longtemps.

**Acceptance Scenarios**

1. **Given** l'app est à l'arrêt sans entrée MIDI active, **When** elle
   reste ouverte, **Then** elle ne maintient pas un périphérique audio actif
   pour rien.

### Edge Cases

- **Note tenue au moment où on lance la lecture** — elle ne doit pas être
  coupée par le démarrage du transport.
- **Note tenue au moment où on met en pause** — elle doit pouvoir être
  relâchée normalement ; pas de note bloquée.
- **Entrée MIDI désactivée alors qu'on est à l'arrêt** — plus rien à
  monitorer, l'état revient à celui d'avant.
- **Périphérique audio qui disparaît** (casque débranché) à l'arrêt — même
  comportement qu'en lecture, pas de chemin particulier.

## Requirements

- **FR-001**: Les notes jouées sur le clavier MIDI DOIVENT être audibles
  quand le transport est à l'arrêt, y compris avant toute lecture.
- **FR-002**: À l'arrêt, la grille NE DOIT PAS avancer : ni note de basse,
  de batterie ou d'accord, ni progression de la position musicale.
- **FR-003**: Le moteur DOIT porter un état de transport explicite, distinct
  de l'état du périphérique audio. Arrêter le transport et arrêter le
  périphérique deviennent deux choses différentes.
- **FR-004**: Reprendre la lecture après un temps d'arrêt DOIT repartir de
  la position où le transport s'était arrêté.
- **FR-005**: Le changement NE DOIT PAS altérer le flux de `MeasureSnapshot`
  à graine et entrées identiques.
- **FR-006**: Le périphérique audio NE DOIT PAS rester actif indéfiniment
  quand il n'y a rien à monitorer ni à jouer.

## Success Criteria

- **SC-001**: App fraîchement ouverte, entrée MIDI active, aucune lecture
  lancée : on entend les notes jouées.
- **SC-002**: À l'arrêt, l'oreille n'entend que les notes du joueur — la
  position musicale affichée ne bouge pas.
- **SC-003**: Lecture → pause → 30 secondes de tâtonnement au clavier →
  reprise : la grille repart exactement où elle s'était arrêtée.
- **SC-004**: Le flux de snapshots est identique à graine identique, avant
  et après le changement (filet de non-régression existant).

## Assumptions

- Le monitoring reste sur son canal dédié : cette spec ne touche ni au choix
  des sons ni au mixage, seulement au fait que le chemin soit vivant.
- Laisser le callback audio tourner à l'arrêt est le comportement normal
  d'un logiciel musical ; le coût CPU d'un callback qui ne rend que du
  silence et d'éventuelles notes monitorées est acceptable. FR-006 borne le
  cas dégénéré — app ouverte et oubliée.
- La décision d'arrêter réellement le périphérique (et quand) est un
  arbitrage à faire au plan, pas ici.

---

## Footer — pointeurs

- `harmonium_host/src/native_handle.rs:131-139` — `resume()` / `pause()`,
  qui pilotent directement le flux cpal
- `harmonium_host/src/playback.rs:234-238` — `process_buffer()` et
  `process_live_midi()` en tête
- `harmonium_host/src/playback.rs:74` — `PlaybackCommand`, où un état de
  transport n'existe pas encore
- `harmonium_training/harmonium_practice/src/state.rs:33` — `is_playing`
  côté app, aujourd'hui le seul endroit qui sait
- Filet de non-régression : `harmonium_host/tests/snapshot_stream_tests.rs`
