# Quickstart — vérification de W1

## Tests moteur

```bash
cd harmonium
cargo test --workspace          # la porte : CI GitHub réelle sur ce repo
cargo test -p harmonium_core comping
```

## L'invariant de non-régression, à lancer en premier

Le test qui compte plus que les autres : à graine identique, activer le
comping ne doit rien changer aux notes de basse, lead, caisse et charley.
S'il tombe, le RNG du comping tire dans le flux principal (R5) et toute
session sauvegardée se rejoue différemment.

```bash
cargo test -p harmonium_core comping_does_not_perturb_existing_tracks
```

## Écoute — le vrai critère

Les statistiques prouvent que le générateur obéit. Seule l'oreille prouve
qu'on lui a demandé la bonne chose.

```bash
cd harmonium && cargo run -p harmonium_cli
```

1. Charger un blues en Si♭, tempo 120, style swing medium.
2. `set enable_voicing true` — le piano doit apparaître.
3. Écouter 12 mesures **sans regarder l'écran** : peut-on suivre les
   changements d'accord à l'oreille ? C'est le critère d'arrêt du
   workstream, hérité de `STRATEGY_2026.md`.
4. Vérifier ce qui doit s'entendre : des frappes irrégulières et non
   systématiquement alignées sur la mélodie, des mesures entièrement
   muettes de temps en temps, des anticipations sur le « et du 4 » qui se
   prolongent sur le temps 1 suivant.
5. `set voicing_density 0.2` puis `0.9` — l'épaisseur des accords change.
6. Couper le canal 4 au mixer — le piano se tait, le reste continue.

## Exports

```bash
cargo test -p harmonium_core --test music_generation_tests
```

Le MIDI exporté doit porter une cinquième piste sur le canal 4, et la
partition MusicXML une portée d'accords. Une liaison Charleston doit y
apparaître comme une note tenue par-dessus la barre, pas comme deux notes
réattaquées.

## Ce qui ne se vérifie pas ici

L'authenticité du comping mesurée par `harmonium_lab` : la métrique
n'existe pas (R7). C'est une spec à écrire dans ce repo-là, pas un critère
de W1.
