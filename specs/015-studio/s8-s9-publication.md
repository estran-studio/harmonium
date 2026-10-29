# S8 et S9 — Ressources web, site public, CI

Branche `wquintal/s8-s9-publication` (depuis `1571654`), fusionnée dans `main` (`f6d8896`), suivie de deux
correctifs sur `main` (`d9e382d`, `d864a84`). Décisions du propriétaire (2026-10-29, plan § H) : banque
web en SF3 ; site sur Cloudflare Pages, projet existant `harmonium-website`, page d'accueil et studio en
sous-page ; publication autorisée.

**En ligne** : <https://harmonium-website.pages.dev/> (page d'accueil) et
<https://harmonium-website.pages.dev/studio/> (studio web). Rien n'a été écouté : les essais web ont tourné
dans un Chromium sans tête lancé avec `--mute-audio`.

## S8 — Ressources web

- **Banque** : `GeneralUser-GS.sf3`, 7 620 750 octets, convertie de GeneralUser GS v2.0.3 par
  `harmonium_studio/scripts/sf2_to_sf3.py` (repris de la sonde S1, oggenc 1.4.3 qualité 4). Ajout : le numéro
  de série de chaque flux Ogg est le rang de l'échantillon (`oggenc -s`), donc deux conversions donnent le même
  fichier (même SHA-256 vérifié deux fois). La comparaison d'écoute SF2 / SF3 rendue pour la décision :
  `/tmp/ecoute_sf3/{reel_quebecois,valse_quebecoise}_{sf2,sf3}.wav` (16 mesures, graine 1, hors ligne).
- **Presets Odin** : les 8 presets que nomment les profils (`[[odin.<piste>.zones]] preset`) sont des presets
  d'usine d'Odin 2 (dépôt TheWaveWarden/odin2, commit `265e9e2`, `assets/Soundbanks/Factory Presets/`),
  GPL-3.0 comme le code d'Odin 2 et comme le bundle. C'était le prérequis de S8 (source et licence).
- **Release** `studio-resources-1` du dépôt : banque SF3, banque de pieds, 8 presets (noms d'asset sans espace
  ni crochet, `odin-Bass-Crusher-RM.odin`), licence de GeneralUser GS ; notes avec source et licence de chaque
  fichier.
- **Verrou** `harmonium_studio/web-resources.json` : release, et pour chaque fichier rôle, nom d'origine, nom
  d'asset, taille, SHA-256. `scripts/web_resources.py` :
  - `fetch` télécharge, contrôle taille et SHA-256, range sous le nom d'origine (le nom d'un preset est son nom
    de fichier) ; un écart fait échouer la commande (essayé : un octet ajouté à un preset → « ressources non
    conformes au verrou », sortie 1) ;
  - `check-profiles` échoue si un profil nomme un preset absent du verrou (essayé : preset inconnu → sortie 1),
    ou si aucun preset n'est trouvé (le format `[odin]` a changé pendant la tâche, T1 : premier correctif) ;
  - `stage` prépare une release suivante et réécrit le verrou.
- **Make** : `make studio/web/resources` (verrou → `harmonium_studio/resources` par `studio_assets`),
  `make studio/site` (page d'accueil et studio sous `/studio/` dans `target/site`), `make studio/site/deploy`.
- **About** : la banque est « compressed to SF3 for the web » ; les presets publiés sont des presets d'usine
  d'Odin 2 (GPL-3.0), lien vers Odin 2.
- **Correctif du cache web** (`d864a84`) : l'élagage de Cache Storage comparait l'adresse gardée par le
  navigateur, encodée (`Bass%20Crusher%20[RM].….odin`), au nom du manifeste (`Bass Crusher [RM].….odin`) et
  supprimait à chaque visite les 7 presets dont le nom a une espace ; ils étaient retéléchargés à la visite
  suivante. Le nom est maintenant décodé ; test `a_cached_address_with_encoded_characters_still_names_its_file`
  (échoue avant, passe après). Constaté sur le site en ligne, pas dans S5 (aucun preset n'avait d'espace).
- **Page de démarrage** : `initializer.js` écrivait dans `#boot` après que l'application l'avait retiré
  (`TypeError` dans la console à chaque chargement) ; chaque mise à jour vérifie maintenant que l'élément est
  encore là.

### Mesures (site en ligne, Chromium 150 sans tête, son coupé)

| Mesure | Résultat | Seuil S1 |
|---|---|---|
| Visite suivante : requêtes de ressources | `assets.json` seul (revalidé) ; ni banque, ni pieds, ni preset | aucune requête de banque : tenu |
| Visite suivante : du rechargement au bouton Start | 1,2 s | ≤ 2 s : tenu |
| Premier chargement, débit émulé 10 Mbit/s (latence 40 ms) | wasm reçu à 7,8 s ; dernière ressource à 15,3 s ; Start ensuite (analyse SF3 et Odin, ≈ 1 s) | ≤ 10 s : **non tenu** |
| Transfert | wasm 18,7 Mo brut, ≈ 8,5 Mo compressé par Cloudflare ; ressources 7,9 Mo | — |

Le premier chargement ne peut pas tenir 10 s à 10 Mbit/s avec ces octets : 16,4 Mo à 1,25 Mo/s font 13 s de
transfert à eux seuls. `wasm-opt -Oz` ne gagne que 37 Ko (18,69 → 18,66 Mo) : le wasm est surtout fait de
données (les tables d'ondes d'Odin, compilées dans le bundle), pas de code. Pistes, à décider : tables d'ondes
téléchargées comme une ressource (en cache comme la banque) ou calculées au démarrage ; SF3 en qualité Vorbis
plus basse (à écouter) ; téléchargement des ressources en parallèle du wasm (ne réduit pas les octets).

## S9 — CI et déploiement

- **`.github/workflows/site.yml`** (PR et push sur `main`) : Rust stable + `wasm32-unknown-unknown`, trunk
  0.22.1 (binaire de la release de trunk), `make studio/site` (ressources de la release vérifiées, build web
  du studio avec `odin`, page d'accueil). Sur `main`, `wrangler pages deploy` vers `harmonium-website` si le
  dépôt a le secret `CLOUDFLARE_API_TOKEN` et la variable `CLOUDFLARE_ACCOUNT_ID` ; sinon un avertissement du
  job, sans échec. Premier passage réel : `40911897707`, succès en 3 min 17 s, avertissement de déploiement
  (pas encore de jeton).
- Le build natif du studio et ses tests restent dans `quality.yml` (`make test`, `make lint`, dépendances
  système d'eframe) ; le build wasm du studio est maintenant dans `site.yml`.
- **Audit** : `cargo audit` échouait sur RUSTSEC-2026-0385 (`ctutils` 0.4.3) ; `cargo update -p ctutils` →
  0.4.7, plus aucune vulnérabilité.
- `main` poussé (95 commits), variable `CLOUDFLARE_ACCOUNT_ID` posée sur le dépôt.
- **Déploiements** : à la main par `wrangler` (connexion OAuth du propriétaire), `b4c56cc9` puis `308efaea`
  (production, branche `main`).

## À faire par le propriétaire

1. **Jeton Cloudflare pour la CI** : tableau de bord Cloudflare → *My Profile → API Tokens → Create Token*,
   modèle « Edit Cloudflare Workers » ou permission *Account → Cloudflare Pages → Edit* sur le compte ; puis
   `gh secret set CLOUDFLARE_API_TOKEN`. Ensuite chaque push sur `main` met le site à jour.
2. **Domaine** (facultatif) : `harmonium.estran.studio` répond 522 (enregistrement DNS vers une origine morte) ;
   le rattacher au projet `harmonium-website` (*Custom domains*) donnerait `https://harmonium.estran.studio/studio/`.
3. **Écouter** sur le site en ligne : `reel_quebecois` (banque SF3), `nt_desert` et les styles d'Isaac (Odin).
4. **Firefox et Safari** : chargement, cache (visite suivante sans requête de banque), lecture 5 min.
5. **Seuil du premier chargement** : choisir une des pistes ci-dessus, ou relever le seuil.
6. L'ancien site du projet `harmonium-website` (dépôt privé `harmonium_website`, « Harmonium Training ») est
   remplacé par cette page d'accueil ; ses déploiements restent dans l'historique du projet Cloudflare.
