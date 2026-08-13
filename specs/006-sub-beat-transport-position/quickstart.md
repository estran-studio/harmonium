# Quickstart — vérification de la position infra-temps

## Tests moteur

```bash
cd harmonium
cargo test --workspace          # LA porte : CI GitHub réelle sur ce repo
cargo test -p harmonium_host transport_position
```

## Le test qui compte le plus

Avant tout le reste : prouver que le contrat audio n'a pas bougé. Ce
changement ajoute une lecture ; s'il modifie le flux de snapshots, c'est
qu'il a débordé de son périmètre.

```bash
cargo test -p harmonium_host snapshot_stream_unchanged
```

## Compilation wasm

Le Handle navigateur partage la même architecture : le changement doit rester
compilable sans `cfg` conditionnel (constitution locale III).

```bash
cargo check -p harmonium_core --target wasm32-unknown-unknown
cargo check -p harmonium_host --target wasm32-unknown-unknown
```

## Vérification à la main — la résolution est-elle réellement là

Le piège de cette tâche : tout peut compiler, tous les tests unitaires
passer, et la position rester quantifiée au temps parce qu'un point
d'écriture a été oublié. La seule preuve est d'observer les valeurs.

```bash
cd harmonium && cargo run -p harmonium_cli
```

1. Lancer la lecture.
2. Échantillonner la position bien plus vite qu'un temps et collecter les
   valeurs distinctes sur une mesure.
3. En 4/4, on doit voir **16 positions distinctes**, pas 4. Si on en voit 4,
   la position vient encore des rapports.
4. Faire un seek en cours de lecture : la position doit repartir au step 0 de
   la mesure visée, sans reculer ni se figer.
5. Mettre en pause : deux lectures successives rendent la même valeur.

## Consommateur réel (autre repo, après merge)

Une fois le moteur mergé, la preuve de bout en bout se fait côté app : jouer
une note sur le « et du 4 » et vérifier que l'événement MIDI porte un temps
≥ 4.5, et non 4.0. C'est SC-002, et c'est la raison d'être de tout ce
travail.
