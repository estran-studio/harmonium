# Contrat — position de transport exposée par le moteur

Ce que les consommateurs du moteur voient changer. Tout ce qui n'est pas ici
reste inchangé.

## Ce qui s'ajoute

### `TransportPosition`

```rust
pub struct TransportPosition {
    pub bar: usize,   // 1-indexée
    pub step: usize,  // 0-indexé dans la mesure, grille ticks_per_beat
    pub beat: f32,    // dérivé : step / ticks_per_beat + 1.0
}
```

### Accès sans verrou

Deux formes, selon que l'appelant peut ou non se permettre un verrou :

```rust
// Sur NativeHandle et sur le Handle wasm — pratique, prend le verrou du
// composer comme le fait déjà playhead_bar().
pub fn transport_position(&self) -> TransportPosition;

// Le partage brut, pour les appelants temps réel : à cloner UNE FOIS au
// démarrage, puis load(Relaxed) sans verrou dans le chemin critique.
pub fn transport_position_handle(&self) -> Arc<AtomicU64>;
```

Le second est ce dont le callback MIDI a besoin (FR-002). L'empaquetage est
documenté dans `data-model.md` ; une fonction pure de dépaquetage est
exposée avec, pour que l'appelant n'ait pas à reproduire le décalage de bits.

## Ce qui change sans changer de signature

`playhead_bar()` continue de rendre la mesure courante, désormais dérivée du
champ empaqueté. Aucun appelant à modifier — `composer.rs:214`, `:267`,
`:289`, `native_handle.rs:217` compilent tels quels.

## Ce qui disparaît

`playhead_bar: Arc<AtomicUsize>` comme champ partagé. Remplacé, pas doublé :
deux sources de vérité pour la mesure pourraient diverger (constitution II).

Impact : les signatures de construction qui le prennent en paramètre —
`MusicComposer::new`, `new_with_seed`, `PlaybackEngine::new`, et les deux
chemins de `audio.rs` — passent l'`Arc<AtomicU64>`. Le test
`harmonium_host/tests/deterministic_seek_tests.rs:90` le construit aussi et
suit.

## Ce qui ne change PAS

- **`MeasureSnapshot`** — aucun champ ajouté, retiré ni modifié. Le contrat
  audio du moteur (constitution locale, principe I) est intact, donc **aucun
  bump de version de `TuningParams`** n'est requis. À prouver par un test de
  non-régression, pas à affirmer.
- **`EngineReport` et sa cadence d'émission** — inchangés. L'UI continue de
  lire l'état par les rapports sondés à 100 ms, où la résolution au temps
  suffit (research R2).
- **La génération** — writehead, timeline, harmonie, mélodie : rien.

## Consommateurs en aval (autre repo, pour information)

`harmonium_training` pourra ensuite :

- lire la position dans le callback MIDI sans verrou, ce qui remplace le
  couple `update_state()` + `get_state()` actuellement pris sous mutex ;
- supprimer sa propre dérivation `current_step / 4.0 + 1.0`
  (`harmonium_practice/src/engine.rs:1339`) au profit du `beat` fourni.

Ce sont des tâches de ce repo-là, pas de celui-ci.
