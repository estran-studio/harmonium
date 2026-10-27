# G5 — Nappes sur la piste `Chords`

## Constat initial

Les données seules ne suffisaient pas : un coup `ChordHit { step: 0, duration: steps_per_bar, stroke: Block }` place sa fin à `time = length` dans `Playhead::load_measure`, ce qui pousse un `Spill` à `step = 0` de la mesure suivante. Au pas 0, le tri historique (`(time, on, …)`) plaçait la fin avant l'attaque : même quand l'accord ne changeait pas, chaque barre émettait `NoteOff` puis `NoteOn`.

## Choix et pourquoi

Plutôt que d'étendre `duration` au-delà de la barre dans le générateur (ce qui aurait forcé à deviner l'accord futur et aurait laissé déborder une ancienne tenue sur un accord régénéré après `invalidate_future`), chaque mesure reste autonome :

1. **En données** : `ChordHit.tie: bool` (`#[serde(default)]`, `false`). Un coup lié publie `Articulation::Tied` dans le snapshot (`MeasureSnapshot` / `TimelineNote`).
2. **Au lecteur (`Playhead::tick`)** : les attaques liées sont triées juste avant les fins au même instant (`tied_rank`). Si une attaque liée trouve la même hauteur déjà active sur le canal *et* que la fin de cette note active tombe exactement au même instant dans le lot du pas, l'identifiant actif bascule vers la nouvelle note sans émettre de `NoteOn` ; la fin coïncidente de l'ancienne note ne correspond plus à `active` et est ignorée ; la nouvelle fin en vol (`pending`) fermera la tenue plus tard.
3. **Repli sûr (invalidation, changement d'accord, gate < 1000)** : si la hauteur ne sonne plus ou que l'accord a changé, la fusion n'a pas lieu : la fin en vol ferme l'ancienne hauteur à la barre et les nouvelles hauteurs attaquent normalement. Aucune note orpheline, aucune allocation ni verrou dans le callback.
4. **Cohérence d'export** : `timeline::midi_export` fusionne les plages contiguës de même hauteur quand la seconde porte `Articulation::Tied`.

Les 10 profils distribués ont `tie = false` partout : leurs snapshots et les goldens sont inchangés, sans bump de `TUNING_VERSION`.

## Fichiers touchés

- `harmonium_core/src/trad/accomp_profile.rs` — `ChordHit.tie` (défaut `false`).
- `harmonium_core/src/trad/accomp.rs` — `hit.tie` → `Articulation::Tied`.
- `harmonium_core/src/timeline/mod.rs` — variante `Articulation::Tied`.
- `harmonium_core/src/timeline/pointers.rs` — `PreparedEvent.tied`, `tied_rank`, fusion sans réattaque dans `Playhead::tick`.
- `harmonium_core/src/timeline/midi_export.rs` — fusion des plages liées à l'export SMF.
- `harmonium_core/src/performance.rs` — refus de `Tied` dans les règles d'interprétation (la liaison vient des coups).
- `harmonium_core/src/report.rs` — sérialisation de `Tied` dans `NoteSnapshot`.
- `profiles/README.md` — documentation de `tie`.
- `tests/fixtures/profiles/nappe_pad/` — fixture nappe (GM 89, `chords_program = 88`, 4/4, triade serrée).
- `tests/fixtures/profiles/valse_nappe/` — fixture d'un second style (valse québécoise 3/4 avec chœur GM 52, `chords_program = 51`, principe VIII).
- `harmonium_core/tests/nappe_contract_tests.rs` — tests de contrat sur le flux de snapshots et le lecteur.

## Preuves

- `cargo test -p harmonium_core --test nappe_contract_tests` : 5 tests (tenue sans réattaque puis réattaque au changement, bilan `NoteOn == NoteOff` après régénération par changement de valence, valse 3/4 employant le même mécanisme, opt-in + refus dans `performance.rules`, garde-fou sur les 10 profils distribués).
- Smoke CLI (`--export --bars 32 --profile tests/fixtures/profiles/nappe_pad --seed 1 --record-midi /tmp/g5-nappes.mid`) : 40 événements sur le canal 1 (20 `NoteOn`, 20 `NoteOff`, bilan 0 par hauteur) ; la quinte commune `sol4` (MIDI 67) entre `C` et `G` n'est attaquée qu'une fois au tick 0 et tenue jusqu'au tick 59520 (mesure 31). Même bilan exact (96 événements, bilan 0 sur les 9 hauteurs) sur `valse_nappe`.

## Limites

- La fusion exige la coïncidence exacte entre la fin de la note précédente et l'attaque liée : si une règle `performance.tracks.Chords` raccourcit le `gate` (< 1000), ajoute une respiration (`breath > 0`) ou décale l'attaque (`offset != 0` ou `strum` sur un coup non `block`), les deux instants divergent et le lecteur retombe sur une attaque normale (jamais de note bloquée, mais plus de liaison). Pour une nappe, utiliser `stroke = "block"` sans détachement sur `Chords`.
- Le changement de dynamique à l'intérieur d'une tenue liée n'émet pas d'aftertouch ni de CC11 : la vélocité de l'attaque initiale reste en vigueur jusqu'au prochain changement de hauteur.

## Ce qu'il faut écouter

```sh
cargo run -q -p harmonium-cli --release -- --profile tests/fixtures/profiles/nappe_pad --seed 1
cargo run -q -p harmonium-cli --release -- --profile tests/fixtures/profiles/valse_nappe --seed 1
```

Avec une banque GM (`assets/GeneralUser-GS.sf2`) : vérifier que le pad (ou le chœur sur la valse) tient sans pompage aux barres où l'accord reste sur `I`, que les voix communes restent tenues lors d'un changement `I → V`, et qu'un changement d'émotion (`valence` ou `tension` dans le REPL) renouvelle l'accord à la mesure suivante sans note coincée.
