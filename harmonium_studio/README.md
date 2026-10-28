# harmonium_studio

Application egui du moteur Harmonium, native et web (même code). Phase 015, tâche S5 : la coquille — chargement des ressources, cache, bouton Start, écran provisoire qui prouve que le moteur joue. L'écran Live est la tâche S6 ; l'atelier de profils, S7.

Le studio ne lit que le contrat publié du moteur (`BeatInfo`, `MeasureSnapshot`) et ne le pilote que par `LiveHost::apply(EngineCommand)` et `LiveHost::pump()`. Il est toujours compilé avec `odin` : **un binaire ou un bundle du studio est GPL** (constitution 4.2.0).

## Lancer

```sh
make studio                 # natif : prépare les ressources, puis ouvre la fenêtre (release)
make studio ARGS="--buffer 1024"
make studio/web/serve       # web : Trunk, http://127.0.0.1:8787
make studio/web/build       # web : build dans harmonium_studio/dist
```

Le web demande `trunk` et `rustup target add wasm32-unknown-unknown`, et un CLI `wasm-bindgen` de la version de `Cargo.lock` (Trunk télécharge le sien s'il ne trouve pas la bonne version dans le `PATH`) :
`cargo install --locked --root .tools trunk wasm-bindgen-cli --version <version de Cargo.lock>`, puis `PATH=$PWD/.tools/bin:$PATH`.

Réglages :

| | natif | web |
|---|---|---|
| tampon audio (images) | `--buffer <n>` (défaut : celui de la carte son) | `?buffer=<n>` (défaut : 2048) |
| dossier des ressources | `--resources <dossier>` (défaut : `harmonium_studio/resources`) | `resources/` à côté de la page |

Un tampon doit être compris entre 64 et 16 384 images.

## Ressources : le manifeste

`make studio/assets` (appelé par les cibles ci-dessus) lance `studio_assets`, qui copie les sources en fichiers **nommés par leur empreinte** dans `harmonium_studio/resources/` (ignoré par git) et écrit `assets.json` :

```json
{ "resources": [
  { "id": "main-bank", "file": "GeneralUser-GS.9575028c7a1f.sf2", "bytes": 32319396, "sha256": "…" },
  { "id": "feet-bank", "file": "harmonium_trad.4dc8159837b4.sf2", "bytes": 154186, "sha256": "…" }
] }
```

Rôles (`id`) : `main-bank` (obligatoire), `feet-bank`, `odin:<nom court>` (un par preset `.odin`, zéro est normal). Le format de la banque n'est écrit nulle part : SF2 ou SF3, c'est le même rôle.

Sources, par variables du Makefile :

- `STUDIO_BANKS` : dossier de `GeneralUser-GS.sf2` et `harmonium_trad.sf2`. Défaut : `assets/` du checkout principal (aussi depuis un worktree).
- `STUDIO_ODIN` : dossier de presets `*.odin` ; vide par défaut (aucun preset n'existe encore dans le dépôt).

Autres sources (banque SF3, fichiers d'une release) : appeler directement
`cargo run -p harmonium_studio --no-default-features --bin studio_assets -- --main-bank <fichier> [--feet-bank <fichier>] [--odin-dir <dossier>] --out <dossier>` ; le manifeste et le chargeur ne changent pas.

## Chargement

1. **Page HTML** (Trunk, `initializer.js`) : progression du téléchargement du wasm, barre indéterminée sans taille connue, texte d'échec.
2. **Écran egui** : une ligne par ressource (« on this device » ou « downloading x / y MB »), barre globale, avis de première visite.
3. **« Preparing instruments… »** : l'analyse des banques bloque le fil ; le message est peint une image avant.
4. **Start** : le clic démarre la sortie audio (geste exigé par le navigateur) ; puis écran provisoire (profil, mesure et pulsation, sélecteur parmi `StyleProfile::distributed()`).

Web : le manifeste se revalide à chaque visite (`cache: no-cache`) ; chaque ressource est lue dans Cache Storage (cache `harmonium-studio-resources`, clé = `resources/<fichier à empreinte>`) si l'entrée a la taille du manifeste, sinon téléchargée en flux (`cache: no-store`, progression comptée en octets reçus, taille attendue du manifeste) et mise au cache **après** contrôle de sa taille ; après un chargement réussi, les entrées que le manifeste ne cite pas sont supprimées. Pas de `navigator.storage.persist()`. Cache absent ou refusé : les octets restent en mémoire, la musique joue, un avis le dit.

Natif : mêmes ressources lues sur disque par un fil, même écran, pas de cache.

## Génération

`LiveHost::pump()` est appelé par un fil (natif, toutes les 50 ms) ou par un minuteur de Worker (web, toutes les 100 ms, indépendant de `requestAnimationFrame`), avec 16 mesures d'avance. Détail et chiffres : `specs/015-studio/s5-coquille-chargement.md`.

## Organisation

- `src/manifest.rs`, `src/packager.rs`, `src/loading.rs` : logique pure (testée dans `tests/`), sans eframe.
- `src/platform/` : `native.rs` et `web.rs`, les seules différences entre bureau et navigateur.
- `src/app.rs` : l'écran.
- La feature `app` (par défaut) porte eframe et le moteur ; `--no-default-features` ne compile que le manifeste et `studio_assets`.
