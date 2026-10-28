# S1 — Sonde web (phase 015) : rapport de mesures

Sonde jetable (jamais fusionnée), branche `wquintal/s1-sonde-web`, base `6b6dbef`. Code : `probes/s1-web/`
(projet cargo autonome), modifications du moteur dans les crates de la branche (liste en § 1.4).
Questions : voir `specs/015-studio/plan.md`, § S1.

## 0. Résumé des décisions

| Décision | Réponse | Statut |
|---|---|---|
| (a) Moteur + egui sur le fil principal, sans AudioWorklet | **Oui, mesuré** : 0 tampon en retard sur 5 min sans Odin et 3 min avec 8/16/24 voix Odin, à 1024 et 4096 images (Chromium 150, macOS). Tampon par défaut recommandé : 2048 (non mesuré, entre deux tailles mesurées), à valider par le propriétaire sur Firefox/Safari. Onglet masqué : **bloqué** (§ 3.4), une observation fortuite montre que la génération pilotée par la boucle UI s'arrête sans rendu. | mesuré / bloqué en partie |
| (b) SF2 vs SF3 | SF3 : 7,62 Mo contre 32,3 Mo (−76 %), mais décodage Vorbis ×20 à l'analyse (natif 533,9 ms contre 26,6 ms). **Recommandation : servir le SF3** ; l'analyse wasm du SF3 n'a pas été mesurée dans le navigateur (estimation calculée ≈ 0,83 s). | mesuré natif, wasm calculé |
| (c) Service worker pour le bundle | **Non mesuré** (§ 4.3). Recommandation conditionnelle : pas de service worker au départ ; Cache Storage pour les banques seulement. | bloqué |
| (d) Seuils de chargement | Propositions § 5, fondées sur les mesures locales et des temps de téléchargement **calculés**. | partiel |
| (e) Déterminisme natif/wasm | **Cassé avant correction (0/66), identique après (66/66)** ; sortie native inchangée. | mesuré |

## 1. Installation

### 1.1 Machine et navigateur
- Apple M4 Max, 14 cœurs, 36 Go, macOS 26.7.1 (`sysctl`, `sw_vers`).
- Chromium : Chrome for Testing 150.0.7871.24 (UA « Chrome/150.0.0.0 »), piloté par l'outil `browser`.
- Firefox et Safari : **à mesurer par le propriétaire** (§ 6). La machine était partagée avec d'autres agents
  (charge moyenne 33 relevée à 03:14, après les mesures ; la charge pendant les mesures n'a pas été enregistrée).

### 1.2 Compilation
- `rustc 1.101.0-nightly (0abfedbc7 2026-10-02)`, cible `wasm32-unknown-unknown`, profil `release`
  (`opt-level=3`, `lto="fat"`, `codegen-units=1`, `panic="abort"`), `wasm-bindgen` 0.2.129 (`--target web`),
  `wasm-opt` 124 `-O3` avec bulk-memory, nontrapping-float-to-int, sign-ext, mutable-globals, SIMD.
  Script : `probes/s1-web/build.sh`.
- cpal 0.15.3 (hôte WebAudio, feature `wasm-bindgen`), eframe/egui 0.34 (glow), oxisynth 0.1.0 avec `sf3`,
  `odin2-core` (dépôt `estran-studio/odin-rs`).
- Taille du wasm : 17 801 617 o brut, **16 262 866 o** après `wasm-opt`, **7 133 844 o** transférés (gzip -9, serveur de la sonde).

### 1.3 Méthode
- Serveur `server.mjs` : ETag, `Cache-Control` réglable (`no-cache` pour la revalidation), gzip précalculé,
  débit et latence simulables (`/__ctl`), journal des requêtes, dépôt des résultats (`/__put`). Banques toujours `no-store`
  côté HTTP : elles ne peuvent venir que de Cache Storage.
- Page `www/index.html` + `main.js` : chargement chronométré ; vérité terrain des retards en enveloppant
  `AudioBufferSourceNode.start(when)` : marge = `when − AudioContext.currentTime` (négative = tampon en retard) ;
  côté wasm, durée de `process_buffer`, du rendu Odin et du total par callback, durée de `update` egui, de
  `generate_ahead()`, rAF, tâches longues (`PerformanceObserver`).
- Application : `reel_quebecois`, graine 42, GeneralUser GS + banque de pieds (`harmonium_trad.sf2`), egui repeint en continu
  (rAF ≈ 60 Hz), `generate_ahead()` appelé depuis la boucle UI, 44 100 Hz (valeur par défaut de cpal WebAudio).
  Protocole : 10 s de réchauffement, 300 s sans Odin, puis 60 s avec 8, 16, 24 voix Odin (compteurs remis à zéro entre phases).
- Odin : banque de 24 voix (`src/odin_load.rs`) rendue dans le même callback que oxisynth ; gain 0,03.
- Résolution de `performance.now()` en Chromium : 0,1 ms (les temps sont des multiples de 100 µs).

### 1.4 Modifications du moteur (jetables, dans les crates de la branche)
1. `harmonium_host` : `std::time::Instant` → `web_time::Instant` (`clock.rs`, `audio.rs`, `playback.rs`, `native_handle.rs`) ; `Cargo.toml` : `web-time`,
   `rosc` réservé aux cibles non-wasm, `cpal` (`wasm-bindgen`) et `getrandom` (`js`) pour wasm32.
2. Correctif des tirages (e) : `harmonium_core/src/algo/mod.rs` (`next_range_usize` tire un `u64` ; nouvelle `next_range_u64`),
   `harmonium_core/src/time_plan/score.rs` (`session_rng`), `harmonium_core/src/trad/generator.rs` (`performance_seed`),
   `harmonium_host/src/composer.rs` (choix aléatoire de tonalité/gamme : `gen_range(0..len as u64)`).

## 2. (e) Déterminisme natif / wasm

- Cas : 11 profils de `StyleProfile::distributed()` × graines 1–3 × 2 réglages d'émotions (`mid`, `hi`) = 66 fichiers, 64 mesures chacun,
  par le même chemin de code (`create_offline_engine` + `generate_bars` + `take_snapshots`), `MeasureSnapshot` sérialisés en JSON, comparés octet à octet.
  Natif : `src/bin/det_native.rs` ; wasm : `www/det.html` dans Chromium ; comparaison : `scripts/det_compare.py`.
- **Avant correction : 0 / 66 identiques** (`data/det_compare_before_fix.txt`) ; premier écart toujours dans `$[0].chord_name`
  (ex. `G` natif contre `D` wasm).
- **Cause** : `rand` 0.8 `gen_range` sur `usize` échantillonne par l'entier de la largeur du pointeur ; en wasm32 (32 bits) le flux ChaCha8 n'est pas
  consommé comme en natif 64 bits. Les mesures ne diffèrent donc pas par un calcul flottant mais par la suite de tirages.
- **Correction** (sites en § 1.4.2) : tirage sur `u64` quelle que soit la plateforme.
- **Après correction : 66 / 66 identiques** (`data/det_compare_after_fix.txt`). Natif inchangé : `diff -rq data/det_native data/det_native_postfix` est vide
  (vérifié par le coordinateur).
- Note : S3 reprend le correctif et ajoute un banc natif/wasm permanent ; la présente section reste la preuve d'origine.

## 3. (a) Audio WebAudio sur le fil principal

### 3.1 Résultats — tampon 1024 images (23,2 ms), Chromium 150 sans fenêtre (navigateur partagé)
Fichier : `data/web_buf1024_headless_shared.json`. Durées en ms ; « retards » = nombre de `start(when)` avec marge < 0 (vérité terrain) ;
« retards fin de rendu » = marge − temps de rendu < 0 (estimation côté wasm).

| Phase | Callbacks | Retards | Retards fin de rendu | `process_buffer` moy / p99 / max | Odin moy | Total moy / p99 / max | Marge min / médiane | Intervalles rAF > 20 ms / > 50 ms (max) | Tâches longues |
|---|---|---|---|---|---|---|---|---|---|
| sans Odin (300 s) | 12 932 | **0** | 0 | 0,61 / 1,30 / 1,80 | 0,00 | 0,61 / 1,30 / 1,80 | 19,2 / 19,2 | 25 / 0 (32,3) | 0 |
| 8 voix (60 s) | 2 588 | **0** | 0 | 0,68 / 1,30 / 1,80 | 1,54 | 2,22 / 3,70 / 4,20 | 13,4 / 19,2 | 1 / 0 (26,6) | 0 |
| 16 voix | 2 588 | **0** | 0 | 0,62 / 1,20 / 1,70 | 2,63 | 3,25 / 5,10 / 5,60 | 13,4 / 19,2 | 1 / 0 (20,8) | 0 |
| 24 voix | 2 589 | **0** | 0 | 0,52 / 1,10 / 1,30 | 3,41 | 3,93 / 6,10 / 6,60 | 13,4 / 19,2 | 3 / 0 (23,2) | 0 |

`update` egui : moyenne 0,12–0,14 ms, max 0,6 ms. `generate_ahead()` : max 0,5 ms. Mémoire wasm : 166 002 688 o. Tampons silencieux : 0.

### 3.2 Résultats — tampon 4096 images (92,9 ms), Chromium 150 fenêtré (profil dédié)
Fichiers : `data/web_buf4096/{sf_only,odin8,odin16,odin24}.json`.

| Phase | Callbacks | Retards | Retards fin de rendu | `process_buffer` moy / p99 / max | Odin moy | Total moy / p99 / max | Marge min / médiane | Intervalles rAF > 20 ms / > 50 ms (max) | Tâches longues |
|---|---|---|---|---|---|---|---|---|---|
| sans Odin (300 s) | 3 234 | **0** | 0 | 0,70 / 2,30 / 4,50 | 0,00 | 0,70 / 2,30 / 4,50 | 83,0 / 88,9 | 26 / 0 (25,3) | 0 |
| 8 voix (60 s) | 647 | **0** | 0 | 0,62 / 2,00 / 3,70 | 1,22 | 1,85 / 6,50 / 8,70 | 83,0 / 88,9 | 69 / 0 (25,3) | 0 |
| 16 voix | 648 | **0** | 0 | 0,69 / 2,30 / 3,60 | 2,53 | 3,22 / 9,60 / 11,80 | 77,2 / 88,9 | 57 / 0 (25,4) | 0 |
| 24 voix | 648 | **0** | 0 | 0,64 / 1,60 / 2,60 | 3,57 | 4,21 / 8,50 / 12,00 | 77,2 / 88,9 | 47 / 0 (25,3) | 0 |

`update` egui moyenne 0,034–0,045 ms (max 0,2–0,4) ; `generate_ahead()` max 0,2–0,3 ms.

Remarques d'honnêteté : les deux tailles n'ont pas été mesurées dans le même environnement (voir § 3.4) ; la marge de départ est
d'environ `tampon − 4 ms` : l'hôte cpal programme chaque tampon quasi immédiatement avant la fin du précédent.
On ignore si la sortie du premier jeu (1024) était la carte son ou un puits nul de Chromium.

### 3.3 Lecture
- Charge de rendu (calculé : total max / durée du tampon) : 1024 → 6,6/23,2 = 28 % au pire avec 24 voix ; 4096 → 12,0/92,9 = 13 %.
  Moyenne avec 24 voix : 3,93/23,2 = 17 % (1024).
- Le risque réel sur le fil principal n'est pas le rendu mais un blocage du fil : plus long intervalle rAF observé 32,3 ms contre une marge de 13–19 ms à 1024.
  Aucun retard n'a pourtant été compté ; à 4096 la marge de 77–89 ms absorbe tout ce qui a été observé.
- **Recommandation (a)** : moteur + egui sur le fil principal sans AudioWorklet, tampon 2048 par défaut (≈ 46 ms de marge, non mesuré : valider), 4096 en repli
  si la plateforme montre des retards. AudioWorklet + threads wasm seulement si Firefox/Safari (§ 6) montrent des retards.

### 3.4 Onglet masqué et mesures bloquées
- **Onglet masqué 60 s : bloqué.** Tentatives : mêmes onglets pilotés ; l'outil de navigation gèle les onglets inactifs, les onglets gérés ont été relâchés
  en cours de mesure (« Released managed tab »), l'exécution d'un tour d'outil est limitée à ~200–300 s, et le navigateur fenêtré s'est mis en veille / a figé
  (l'horloge de la page avançait de ~60 s pendant ~1000 s d'horloge murale ; le contexte audio ne progressait plus, `afplay` bloquait aussi : sortie son système morte).
  Un relancement du navigateur a échoué (`Browser open timed out`).
- **Observation fortuite** (navigateur partagé, nouvel onglet dont le rendu `requestAnimationFrame` était suspendu, probablement hors écran) : avec `gen=ui`,
  `gen_calls` est resté à 0, `playhead_bar` et `writehead_bar` bloqués à 8 pendant ~500 s, alors que les callbacks audio continuaient
  (745 callbacks, `ctx` « running », 0 retard). Cela montre que **la génération pilotée par la boucle UI cesse quand rAF cesse** ; l'audio ne manque pas de tampons
  immédiatement mais la musique n'avance plus. Effet musical exact : [INFÉRENCE]. Source : première version de `data/` non conservée (donnée invalide pour les
  autres tableaux) ; reproductible avec `?gen=ui` dans un onglet sans rAF. Cela justifie pour la phase : génération sur minuteur de Worker (`?gen=worker`,
  `www/tick-worker.js`, déjà dans la sonde) avec une avance en mesures suffisante pour l'étranglement à 1 Hz — **non mesuré**.
- Deux séries complètes supplémentaires ont été perdues (onglet détaché, audio système figé) : `web_buf4096` (tampon 4096) a abouti, la reprise à 1024 n'a pas abouti.
- Action système temporaire (`defaults write com.google.chrome.for.testing NSAppSleepDisabled`) : annulée (`defaults delete`), hors du worktree.

## 4. (b) et (c) Chargement

### 4.1 Premier chargement local (SF2), `data/web_buf1024_headless_shared.json`
Serveur local, sans limitation de débit, gzip pour wasm/js (banques non compressées).

| Étape | Valeur |
|---|---|
| Wasm : ttfb / téléchargement + instanciation | 1,5 ms / 44,5 ms (7 133 844 o transférés, 16 262 866 o décodés) |
| GeneralUser GS SF2 : ttfb / corps | 6,7 ms / 38,6 ms (32 319 396 o) |
| Mise en Cache Storage de la banque (`cache.put`) | 36,9 ms (ouverture 18,2 ms) |
| Banque de pieds : corps / put | 3,8 ms / 1,2 ms (154 186 o) |
| Analyse SF2 seule dans wasm (`bench_parse`) | 41,4 ms |
| `prepare` : création du moteur (analyse comprise) / première génération / total | 68 / 6,6 / 74,6 ms |
| Init Odin (24 voix) | < 0,1 ms (sous la résolution du chronomètre) |
| Fin de l'init JS → première image egui, « Start » prêt | 557 ms depuis le début de la page (dont 233 ms pour la première image egui) |

Lecture depuis Cache Storage : **non mesurée** (la banque n'a été lue que depuis le réseau local ; un chargement « en cache » n'a pas abouti).
Gzip -9 du SF2 : 29 222 769 o (calculé avec `gzip -9 -c | wc -c`) : pas de gain réel, donc pas de compression utile côté serveur.

### 4.2 SF2 contre SF3 (GeneralUser GS)
Conversion : `scripts/sf2_to_sf3.py` (oggenc qualité 4, un flux mono par échantillon, `shdr` réécrit, drapeau Vorbis, `ifil` 3.0) ; aucun convertisseur
n'étant installé, ce script est le notre. Vérification : `src/bin/sf3_check.rs` charge les deux fichiers dans oxisynth natif, rend 30 s de `reel_quebecois`.

| | SF2 | SF3 |
|---|---|---|
| Taille | 32 319 396 o | 7 620 750 o (−76,4 %, calculé) |
| Analyse oxisynth **native** (mesurée) | 26,6 ms | 533,9 ms (×20) |
| Analyse **wasm** | 41,4 ms (mesurée, § 4.1) | ≈ 830 ms (**calculé** : 533,9 × 41,4/26,6 ; non mesuré dans le navigateur) |
| Rendu 30 s (RMS) | 0,08443 | 0,08464 ; écart/sf2 −29,0 dB |
| Téléchargement à 10 Mbit/s (**calculé**, taille × 8 / 10 Mbit/s) | 25,9 s | 6,1 s |

Données : `data/sf3_check.txt`. L'écart de −29 dB vient de la compression Vorbis qualité 4 ; non écouté.
**Recommandation (b)** : servir le SF3. À 10 Mbit/s, 19,8 s économisées au premier chargement (calculé) contre ≈ 0,8 s ajoutée à chaque visite (estimation),
Cache Storage évitant le retéléchargement. Si le temps d'analyse par visite devient gênant, décoder dans un Worker ou à la demande par instrument (hors sonde).

### 4.3 Revalidation du bundle (c)
Le serveur de la sonde gère `no-cache` + ETag (réponses 304) et `max-age=600`, et le journal `/__log` compte les requêtes, mais la comparaison
chargement à froid / rechargement avec revalidation **n'a pas été exécutée** : les rechargements d'onglet gérés échouaient (§ 3.4). Ce qui est connu :
quatre fichiers de bundle (`index.html`, `main.js`, `s1_web.js`, `s1_web_bg.wasm`) ; en revalidation chacun coûte une requête conditionnelle (304 sans corps) ;
instanciation du wasm de 16 Mo : 44,5 ms en local. Coût de revalidation réel = nombre de requêtes en parallèle × RTT [INFÉRENCE].
**Recommandation (c)** : pas de service worker initialement (un seul aller-retour parallèle après 10 min d'âge) ; à reprendre si la mesure du propriétaire
(§ 6, rechargement avec `cache=etag` et latence simulée `delay=100`) dépasse le seuil de rechargement du § 5.

## 5. (d) Seuils proposés
- Premier chargement : barre de progression obligatoire ; « Start » prêt en ≤ 10 s à 10 Mbit/s avec SF3 (≈ 6,1 s de banque + 5,7 s de wasm = 11,8 s calculés → viser une réduction du wasm, 7,1 Mo gzip) ;
  avec SF2, 31,6 s calculés : refusé.
- Chargement en cache : « Start » prêt ≤ 2 s (mesuré local 0,56 s avec banque réseau locale ; analyse SF2 41 ms ; SF3 ≈ 0,8 s calculé).
- Rechargement revalidé : ≤ 1 s au-delà du chargement en cache (à valider, § 4.3).
Les débits sont des hypothèses (10 Mbit/s), non mesurées.

## 6. Mesure par le propriétaire (Firefox, Safari)
Colonnes : « à mesurer par le propriétaire ».
1. `cd probes/s1-web && ./build.sh` (nécessite `wasm-bindgen-cli` 0.2.129 et `wasm-opt`) ; copier `GeneralUser-GS.sf2`, `harmonium_trad.sf2` dans `www/assets/`,
   `python3 scripts/sf2_to_sf3.py www/assets/GeneralUser-GS.sf2 www/assets/GeneralUser-GS.sf3` (nécessite `oggenc`).
2. `node server.mjs --port 8080`, ouvrir `http://localhost:8080/?sf=sf3&buf=2048&parse_bench=1` (`sf=sf2|sf3`, `buf=1024|2048|4096`, `odin=0|8|16|24`, `gen=ui|timer|worker`).
3. Cliquer **Start** (geste obligatoire sous Safari) puis **Protocole propriétaire** (≈ 8 min) ; **Télécharger les résultats** pour le JSON.
   Onglet masqué : lancer, changer d'onglet 60 s avec `?gen=ui`, revenir, comparer `gen=worker`.
4. Revalidation : `http://localhost:8080/__ctl?cache=etag&delay=100` puis recharger ; froid : `cache=none`.

| Mesure | Firefox | Safari |
|---|---|---|
| Retards 1024 / 2048 / 4096, 0–24 voix Odin | à mesurer par le propriétaire | à mesurer par le propriétaire |
| Onglet masqué 60 s (`ui` / `worker`) | à mesurer par le propriétaire | à mesurer par le propriétaire |
| Analyse SF2 / SF3 dans wasm | à mesurer par le propriétaire | à mesurer par le propriétaire |
| Rechargement revalidé | à mesurer par le propriétaire | à mesurer par le propriétaire |

## 7. Où sont les données
- `probes/s1-web/data/det_compare_before_fix.txt`, `det_compare_after_fix.txt` (versionnés).
- `probes/s1-web/data/sf3_check.txt`, `web_buf1024_headless_shared.json`, `web_buf4096/*.json` (versionnés).
- Non versionnés (reproductibles) : instantanés JSON `data/det_native`, `det_native_postfix`, `det_wasm`, `det_wasm_prefix` (~12 Mo chacun),
  banques `www/assets/*.sf2|sf3`, `www/pkg`, `target`, `.tools`.
