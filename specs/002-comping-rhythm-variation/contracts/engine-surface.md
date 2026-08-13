# Contrat — surface publique du moteur touchée par W1

Ce que les consommateurs du moteur (`harmonium_training` via
`harmonium_host`, la CLI, le VST) voient changer. Tout ce qui n'est pas ici
reste inchangé.

## Ce qui s'ajoute

### `TrackId::Chord` — canal MIDI 4

```rust
pub enum TrackId { Bass, Lead, Snare, Hat, Chord }
pub const ALL: [TrackId; 5] = [Bass, Lead, Snare, Hat, Chord];
```

Impact aval, automatique : `MeasureSnapshot` porte les notes d'accords
(elles arrivent avec `track: 4`), `muted_channels[4]` coupe la piste, les
exports MIDI et MusicXML gagnent une piste.

**Rupture de compilation attendue** chez tout appelant qui fait un `match`
exhaustif sur `TrackId` ou qui suppose `ALL.len() == 4`. C'est voulu : le
compilateur sert de liste de tâches.

### `CompingParams` dans `TuningParams`

```rust
pub struct TuningParams {
    // … les neuf existantes, inchangées
    #[serde(default)]
    pub comping: CompingParams,
}
```

Les profils de style existants se chargent sans modification grâce au
`serde(default)`.

### Commandes de contrôle

`enable_voicing`, `set_voicing_density`, `set_voicing_tension` existent déjà
sur `Controller` et sur `NativeHandle` — **et ne font rien aujourd'hui**
(R0). W1 ne crée pas ces commandes : il les rend effectives. Aucune nouvelle
signature côté appelant.

C'est la meilleure nouvelle du plan : l'app, la CLI et le VST pilotent déjà
la fonctionnalité, il n'y a pas de nouvelle plomberie d'interface à écrire.

## Ce qui se supprime

`harmonium_audio::voicing` disparaît entièrement — trait `Voicer`,
`ShellVoicer`, `BlockChordVoicer`, `CompingPattern`, `VoicedNote`,
`apply_drop_two`, `get_guide_tones`. La logique utile est portée dans
`harmonium_core::voicing`.

Le réexport `harmonium_host/src/lib.rs:20` (`pub use harmonium_audio::{…,
voicing}`) disparaît avec. Aucun appelant n'existe (vérifié par grep sur
l'ensemble des crates), donc aucune migration à faire chez les consommateurs.

Constitution II : pas de chemin parallèle, la suppression se fait dans le
même changement que le portage.

## Ce qui change de comportement sans changer de signature

- `enable_voicing(true)` produit désormais du son. Il passe par défaut à
  `true` (SC-003) : `params.rs:390` et `params.rs:712` doivent suivre.
- Le playhead honore `duration_steps` **pour la piste d'accords uniquement**.
  Bass, Lead, Snare, Hat conservent leur sémantique de coupure par
  remplacement — cette réserve est ce qui garantit qu'aucun son existant ne
  bouge.

## Ce qui doit rester strictement identique

À graine de session identique, les notes des pistes Bass, Lead, Snare et Hat
sont inchangées, que le comping soit actif ou non. C'est l'invariant central
du workstream (R5) et il est testable directement.

## Frontière inter-repo

Le moteur livre `CompingParams` et ses valeurs par défaut. Le peuplement des
15 profils de style avec le tableau de valeurs par style vit dans
`harmonium_training/static/profiles/` — tâche de l'autre repo (constitution
V), à ouvrir quand W1 est mergé côté moteur.
