# Phase 017 — Plugin Bevy mince

**Statut** : ouverte le 2026-10-29 sous la constitution 4.4.0 (qui fait entrer `harmonium_bevy`
au workspace). Suite des phases 014 et 016 : le son NT et Isaac, la variété par graine et le timbre
par zone d'émotion sont validés à l'écoute ; le goulot est l'intégration en jeu.

## Problème

Aucun jeu ne peut jouer Harmonium aujourd'hui. L'ancien `harmonium_bevy` (API supprimée :
`RhythmMode::Euclidean`, `sequencer`, ancien Odin) n'existe plus ; `alacod` garde une dépendance
morte désactivée (`crates/game/Cargo.toml`, feature `harmonium`). L'hôte actuel suffit pourtant :
`LiveHost` (`harmonium_host/src/live_host.rs`) ouvre la sortie audio, exécute les
`EngineCommand` (`SetStyleProfile`, `SetEmotionParams`, `SetMasterVolume`, `SetSeed`, …), compose
par `pump()` et publie `BeatInfo` et `MeasureSnapshot`.

Décision du propriétaire (2026-10-29) : le plugin vit dans harmonium, avec une application de test
locale ; **pas de travail dans `alacod`** pendant cette phase.

## Tâche

| Tâche | Contenu |
|---|---|
| B1 | Crate `harmonium_bevy` (Bevy 0.19.1, la version d'`alacod`) : plugin mince sur `LiveHost`, ressources par l'`AssetServer`, API jeu (profil, émotions, volume, pause, graine), horloge musicale exposée au gameplay, transition de profil en fondu ; application de test locale (démo) ; tests sans carte son |

## Décisions

- **Mince** : aucune logique musicale dans le plugin ; tout passe par `EngineCommand` et l'API
  publique de `LiveHost` (constitution, périmètre). Ce que le plugin ne sait pas faire par ce
  chemin est un manque de l'hôte, documenté, pas contourné.
- **Hors workspace par défaut**, comme le studio : Bevy est lourd ; `cargo build`/`cargo test`
  nus restent inchangés ; cibles `make` dédiées. Option `odin` du crate, désactivée par défaut
  (un jeu fermé compile sans) ; la démo l'active.
- **Ressources par l'`AssetServer`** (chargeurs `.sf2`/`.sf3`, `.odin`) pour marcher en natif
  comme en web ; le moteur démarre quand les ressources déclarées sont chargées.
- **Transition de profil** : fondu de volume côté plugin (sortie → changement de profil →
  entrée), en attendant les transitions de style J3 (phase 013, reportées).
- **Émotions** : le jeu écrit une cible à chaque frame ; le plugin n'envoie une commande que sur
  changement significatif, cadence bornée.

## Acceptation

1. La démo (`make bevy/demo`) ouvre une fenêtre et joue : profils `nt_desert` et `isaac_*` au
   clavier, émotions au clavier (avec la bascule de son d'`isaac_burning` par la tension), pause,
   nouvelle graine ; HUD : profil, émotions, mesure et temps, un témoin visuel qui pulse sur le
   temps.
2. Tests sans carte son (hôte détaché) : commandes du jeu → état de l'hôte (profil, émotions,
   volume) ; horloge publiée ; fondu de profil sans changement de profil avant le silence.
3. `cargo check --target wasm32-unknown-unknown -p harmonium_bevy`.
4. `cargo test --workspace` inchangé hors du crate ; tests du crate verts ; clippy, fmt.
5. Écoute du propriétaire dans la démo.

## État

| Tâche | État | Compte rendu |
|---|---|---|
| B1 | lancée (2026-10-29) | `b1-plugin-bevy.md` |
