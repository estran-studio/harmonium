# oxisynth 0.1.0 — copie locale pour le callback audio

## Provenance et licence

Sources de la publication [oxisynth 0.1.0 sur crates.io](https://crates.io/crates/oxisynth/0.1.0),
projet amont [PolyMeilex/oxisynth](https://github.com/PolyMeilex/oxisynth),
auteur déclaré par le paquet : Poly <marynczak.bartlomiej@gmail.com>.
Les métadonnées de la publication indiquent le commit
`87c08c1a0165c7ea3577d86571f7688622387674`, sous-répertoire `oxisynth`.
Somme de contrôle crates.io du paquet original :
`68d089fc07e074f57ac6698292ce3da05eac1d282b821eb4156d4c7d8e36c6ec`.

La licence déclarée par le manifeste amont est **LGPL-2.1**. Le fichier
[`LICENSE`](LICENSE) est conservé intégralement ; les mentions présentes dans
les sources sont inchangées. Cette copie ne relève pas de la licence
`MIT OR Apache-2.0` du code Harmonium. La distribution d'un binaire liant cette
bibliothèque doit respecter les obligations de la LGPL, notamment celles
concernant les sources et la possibilité de remplacement ou de réédition de
liens avec une version modifiée de la bibliothèque.

La copie contient les sources de la bibliothèque, son manifeste et sa licence.
Les exemples, dépendances de développement et fichiers de publication ne sont
pas nécessaires à l'intégration et ne sont pas vendus. Les dépendances de
production, dont `oxisynth-chorus` et `oxisynth-reverb`, restent résolues par Cargo.
Le paquet est exclu des membres du workspace Harmonium pour ne pas exécuter ses
tests amont, qui dépendent de données externes absentes de la publication.

## Modifications locales — 2026-10-30

- `src/core/midi.rs` : le tableau de tri de 64 modulateurs est initialisé par
  `[None; 64]`, sans le `Vec` temporaire précédemment alloué puis libéré à
  chaque `NoteOn`. Le type, l'ordre et les valeurs restent identiques.
- `src/core/voice_pool/mod.rs` : `VoicePool::new` réserve la capacité maximale
  des voix. `set_polyphony_limit` réserve aussi l'éventuelle croissance avant
  les notes suivantes. Cette méthode de configuration reste une opération
  hors callback ; la création d'une note, le réemploi et le vol d'une voix
  n'augmentent plus la capacité. Les modulateurs d'une voix sont déjà stockés
  dans un tableau fixe de 64 éléments ; `Voice::new` n'a pas d'autre conteneur
  dynamique à créer.
- `Cargo.toml` : retrait des déclarations d'exemples et dépendances de
  développement non utilisés. `harmonium_audio` sélectionne cette copie par
  une dépendance `path` directe, sans patch global du workspace.
- `rustfmt.toml` : conserve le format de la publication amont ; le formatage
  Harmonium ne réécrit pas les sources tierces.

Le patch ne change ni les algorithmes DSP, ni les enveloppes, ni les programmes
et banques, ni la politique de vol de voix. Les fontes sont toujours chargées
avant le callback. La limite de polyphonie n'est pas modifiée pendant le rendu.
Les garanties de temps réel supposent aussi qu'aucun journaliseur configuré
par l'hôte ne bloque ou n'alloue sur les messages de diagnostic amont.
