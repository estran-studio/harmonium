---
spec_type: per_repo_feature
meta_epic: 006-v2-ep4-improvisation-trainer
meta_repo_path: harmonium_specs/specs/006-v2-ep4-improvisation-trainer/
status: draft
priority: urgent
workstream: W2
strategy: STRATEGY_2026.md — The app listens (harmonic scoring)
---

# Position de transport infra-temps

**Créé** : 2026-08-13 — découvert en recette de la branche `feat/w2-midi-position`

**Statut** : Draft — bloque la valeur rythmique de W2

## Le problème

Le moteur n'expose sa position musicale qu'**une fois par temps**.

```rust
// harmonium_host/src/playback.rs:296-298
let current_step = self.playhead.position.step_in_bar(4);
if current_step.is_multiple_of(4) {
    self.send_report();
}
```

Le rapport n'est envoyé que lorsque le step est un multiple de 4, c'est-à-dire
sur les temps entiers. Tout consommateur lisant `current_step` obtient donc
toujours un multiple de 4, et le temps fractionnaire calculé en aval
(`current_step / 4.0 + 1.0`, `harmonium_practice/src/engine.rs:1339`) vaut
toujours 1.0, 2.0, 3.0 ou 4.0 — jamais autre chose.

La grille interne du playhead est pourtant quatre fois plus fine
(`ticks_per_beat = 4`, soit la double-croche). L'information existe, elle
n'est simplement pas publiée.

## Pourquoi ça compte maintenant

Le coach d'impro (W2) horodate chaque note MIDI jouée avec la position du
transport, pour répondre à sa question centrale : **la note est-elle tombée
sur le changement d'accord ?**

Avec une position quantifiée au temps :

- une note jouée n'importe où dans le temps 2 — de 2.0 à 2.99 — est
  horodatée « temps 2.0 ». À 120 bpm, c'est une incertitude d'une
  demi-seconde sur un phénomène qui se joue à la dizaine de millisecondes ;
- **l'anticipation ne peut jamais être détectée.** Le « et du 4 » poussé
  juste avant la barre — le geste le plus caractéristique du phrasé jazz —
  est horodaté temps 4.0 de la mesure courante. Il est donc lu comme un
  atterrissage sur temps faible contre l'accord sortant, exactement le
  contresens que la règle d'anticipation du modèle de données existe pour
  éviter ;
- les métriques de densité, de silence et de respiration sont mesurées en
  temps, avec une résolution d'un temps : le plus long silence et le nombre
  de respirations sont arrondis au temps entier ;
- le code de fenêtres d'atterrissage `[B, B+0.5)` contre `[B+0.5, B+1)`,
  écrit et testé côté app, est structurellement mort : aucune note ne tombe
  jamais dans la seconde moitié d'un temps.

Autrement dit, W2 mesure aujourd'hui la hauteur avec exactitude et le rythme
à un temps près. La moitié harmonique du produit fonctionne ; la moitié
rythmique est bornée par cette ligne.

## User Scenarios & Testing

### User Story 1 — La position publiée suit la grille interne (Priority: P1)

En tant que consommateur du moteur, je veux lire une position de transport à
la résolution de la grille interne, pour horodater un événement extérieur là
où il est réellement tombé.

**Test indépendant** : faire avancer le transport d'une mesure en 4/4 et
collecter toutes les positions publiées. On doit voir 16 positions distinctes
(une par double-croche), pas 4.

**Acceptance Scenarios**

1. **Given** le transport joue, **When** un consommateur lit la position à un
   instant arbitraire, **Then** il obtient la position à la résolution de la
   grille, pas le dernier temps entier écoulé.
2. **Given** une note arrive juste avant une barre de mesure, **When** sa
   position est lue, **Then** elle est distinguable d'une note arrivée sur le
   temps précédent.
3. **Given** deux lectures au même instant de transport, **When** on les
   compare, **Then** elles sont identiques (déterminisme).

### Edge Cases

- **Lecture en pause** — la position ne bouge pas ; deux lectures successives
  rendent la même valeur.
- **Changement de tempo** — la position reste musicale, jamais dérivée d'une
  horloge murale.
- **Lecture depuis le thread MIDI** — le chemin de lecture ne doit pas prendre
  le mutex du moteur : un thread temps réel ne peut pas se permettre d'y
  bloquer. Voir la note de conception ci-dessous.

## Requirements

- **FR-001**: Le moteur DOIT exposer la position de transport à la résolution
  de sa grille interne (`ticks_per_beat`), et non une fois par temps.
- **FR-002**: La lecture de la position DOIT être possible sans acquérir le
  mutex du moteur, pour être appelable depuis un callback MIDI temps réel.
- **FR-003**: La position exposée DOIT rester musicale — issue du playhead,
  jamais interpolée depuis une horloge murale.
- **FR-004**: Le changement NE DOIT PAS altérer le flux de `MeasureSnapshot`
  à graine et entrées identiques — c'est une addition de lecture, pas une
  modification de génération.
- **FR-005**: La cadence d'émission des rapports existants NE DOIT PAS être
  augmentée si cela charge le thread audio ; le mécanisme de publication est
  à choisir en conséquence.

## Success Criteria

- **SC-001**: Sur une mesure en 4/4, la position publiée prend 16 valeurs
  distinctes au lieu de 4.
- **SC-002**: Une note MIDI jouée sur le « et du 4 » est horodatée avec un
  temps fractionnaire ≥ 4.5, et non 4.0.
- **SC-003**: Les tests existants du moteur passent inchangés, et le flux de
  snapshots est identique à graine identique (FR-004).
- **SC-004**: Le chemin de lecture ne contient aucun verrou bloquant.

## Note de conception (piste, à trancher en plan)

`playback.rs:293` maintient déjà un `playhead_bar: AtomicUsize` mis à jour
**à chaque buffer audio**, pas une fois par temps, et lisible sans verrou. Un
atomique frère portant le step dans la mesure suivrait exactement le même
motif : résolution de grille, aucun verrou, aucune charge supplémentaire sur
le thread audio, et aucun changement à la cadence des rapports.

Ce serait aussi une réponse à FR-002, qui règle au passage un problème
constaté côté app : l'horodatage MIDI prend aujourd'hui le mutex du moteur
depuis le callback midir.

## Assumptions

- La grille interne (`ticks_per_beat = 4`, double-croche) est une résolution
  suffisante pour le scoring d'impro. Si l'usage montre le contraire, c'est un
  autre changement.
- Ce travail n'est pas transverse au sens de la constitution locale : il
  n'altère pas le contrat de snapshot, il ajoute une lecture. Il reste donc
  une spec par-repo, rattachée au méta-epic W2.

---

## Footer — pointeurs

- `harmonium_host/src/playback.rs:293-298` — l'atomique existant et la
  condition d'émission qui quantifie au temps
- `harmonium_host/src/playback.rs:458-480` — construction et envoi du rapport
- `harmonium_core/src/timeline/pointers.rs` — `Playhead`, `MusicalPosition`,
  `step_in_bar`
- Consommateur : `harmonium_training/harmonium_practice/src/engine.rs:1339`
  puis `harmonium_training/src-tauri/src/midi_input.rs`
- Méta-epic : `harmonium_specs/specs/006-v2-ep4-improvisation-trainer/`
