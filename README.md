# Harmonium

Moteur Rust de musique procédurale de style folklorique québécois /
canadien-français, piloté en temps réel par l'état d'un jeu. Conçu pour
« 1837 » (roguelike Rust/Bevy) : le beat est exposé au gameplay, les jauges
émotionnelles du personnage pilotent la musique.

Direction et principes : [`.specify/memory/constitution.md`](.specify/memory/constitution.md).
Phase en cours : [`specs/008-phase0-clock-playback/plan.md`](specs/008-phase0-clock-playback/plan.md).

## Workspace

| Crate | Rôle |
|---|---|
| `harmonium_core` | Types purs et génération : grille de pas, `Measure`, `TimelineGenerator`, `Writehead`/`Playhead`, harmonie, `TuningParams`/`StyleProfile`, mapping émotion → paramètres. Aucune I/O. |
| `harmonium_audio` | Rendu SoundFont (oxisynth), enregistrement WAV / MIDI / MusicXML, garde d'allocation temps réel. |
| `harmonium_host` | `MusicComposer` (thread principal) + `PlaybackEngine` (callback cpal), transport lock-free, `NativeHandle`, `create_offline_engine`, binaire `harmonium` avec contrôle OSC. |
| `harmonium_cli` | REPL d'écoute et de test (`harmonium-cli`). |
| `harmonium_lab` | Ingestion de corpus MusicXML, métriques de distance, tuning assisté. |

## Démarrer

```bash
make run                 # REPL, charge assets/test.sf2 par défaut
make run ARGS="--soundfont path/to/bank.sf2"
make play ARGS="--osc"   # lecteur autonome, /harmonium/params (arousal valence density tension) sur UDP 8080
make test                # cargo test --workspace
make quality             # fmt + clippy + tests
```

## Architecture d'exécution

```mermaid
flowchart LR
  G[Jeu / CLI / OSC] -->|setters| C[MusicComposer\nthread principal]
  C -->|Measure| P[PlaybackEngine\ncallback cpal]
  P -->|AudioEvent| S[oxisynth SF2]
  P -->|AtomicU64 bar,step| G
  C -->|MeasureSnapshot| G
```

Le composer génère des mesures d'avance ; le callback audio les joue pas à
pas et publie la position sans verrou. Les paramètres structurels prennent
effet à la mesure suivante ; gains, mute, programmes GM et seek sont
immédiats.

## Intégration dans un jeu

`harmonium::audio::create_offline_engine(sf2, sample_rate)` rend un
`MusicComposer` et un `PlaybackEngine` sans ouvrir de périphérique : le jeu
appelle `playback.process_buffer()` depuis son propre callback audio et
`composer.generate_ahead()` depuis sa boucle principale.
