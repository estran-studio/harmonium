.PHONY: soundfont run run/debug studio studio/assets studio/open studio/install-handler studio/uninstall-handler studio/web/build studio/web/serve studio/web/resources studio/site studio/site/deploy bevy/demo bevy/test bevy/check test test/sheet test/lab-export test/determinism wasm/check fmt lint audit quality

# ════════════════════════════════════════════════════════════════════
# ════════════════════════════════════════════════════════════════════
# ASSETS
# ════════════════════════════════════════════════════════════════════

## Télécharge la banque GM complète GeneralUser-GS si absente (> 30 Mo)
soundfont:
	@if [ -f assets/GeneralUser-GS.sf2 ]; then \
		size=$$(wc -c < assets/GeneralUser-GS.sf2 | tr -d ' '); \
		if [ "$$size" -gt 30000000 ]; then \
			echo "assets/GeneralUser-GS.sf2 déjà présent ($$size octets)."; \
			exit 0; \
		fi; \
		echo "Fichier assets/GeneralUser-GS.sf2 incomplet ($$size octets), retéléchargement..."; \
	fi; \
	mkdir -p assets && \
	echo "Téléchargement de GeneralUser-GS.sf2..." && \
	curl -fsSL -o assets/GeneralUser-GS.sf2.tmp https://github.com/mrbumpy409/GeneralUser-GS/raw/main/GeneralUser-GS.sf2 && \
	size=$$(wc -c < assets/GeneralUser-GS.sf2.tmp | tr -d ' ') && \
	if [ "$$size" -le 30000000 ]; then \
		echo "Erreur: fichier téléchargé trop petit ($$size octets <= 30 Mo)" >&2; \
		rm -f assets/GeneralUser-GS.sf2.tmp; \
		exit 1; \
	fi && \
	mv assets/GeneralUser-GS.sf2.tmp assets/GeneralUser-GS.sf2 && \
	echo "assets/GeneralUser-GS.sf2 installé avec succès ($$size octets)."

# RUN
# ════════════════════════════════════════════════════════════════════

## Interactive REPL (harmonium_cli)
run:
	HARMONIUM_CLI=1 cargo run -p harmonium-cli --release -- $(ARGS)

run/debug:
	HARMONIUM_CLI=1 cargo run -p harmonium-cli -- $(ARGS)

# ════════════════════════════════════════════════════════════════════
# STUDIO (harmonium_studio : natif et web)
# ════════════════════════════════════════════════════════════════════

## Dossier des banques (hors git) : par défaut `assets/` du checkout principal, aussi depuis un worktree.
STUDIO_BANKS ?= $(shell dirname "$$(git rev-parse --path-format=absolute --git-common-dir)")/assets
## Dossier de presets Odin (`*.odin`) : `assets/odin` du checkout principal s'il existe (local, non versionné),
## pour que les profils de jeu ouverts par un lien sonnent avec leurs presets ; vide : aucun.
STUDIO_ODIN ?= $(wildcard $(STUDIO_BANKS)/odin)
STUDIO_RESOURCES := harmonium_studio/resources

## Copie les banques en fichiers à empreinte dans harmonium_studio/resources, avec assets.json
studio/assets:
	cargo run -q -p harmonium_studio --no-default-features --bin studio_assets -- \
		--main-bank $(STUDIO_BANKS)/GeneralUser-GS.sf2 \
		--percussion-bank $(STUDIO_BANKS)/harmonium_trad.sf2 \
		$(if $(STUDIO_ODIN),--odin-dir $(STUDIO_ODIN)) \
		--out $(STUDIO_RESOURCES)

## Studio natif (fenêtre)
studio: studio/assets
	cargo run -p harmonium_studio --release -- $(ARGS)

## Ouvre un lien dans le studio natif : le transmet au studio qui tourne, sinon le lance (en pause : aucun son
## tant que l'on n'appuie pas sur lecture, sauf PLAY=1). Soit URL='harmonium://open?…', soit les paramètres :
##   make studio/open PROFILE=<id ou dossier> SEED=3 BAR=17 LOOP=17-24 BPM=180 SWING=0.6 SOLO=Bass MUTE=Melody,Chords
## Un id de profil distribué ouvre le dossier profiles/<id> de ce dépôt (rechargé à chaud quand ses fichiers changent).
studio/open: studio/assets
	cargo run -q -p harmonium_studio --release -- open \
		$(if $(URL),'$(URL)') \
		$(if $(PROFILE),--profile '$(PROFILE)') $(if $(SEED),--seed '$(SEED)') $(if $(BAR),--bar '$(BAR)') \
		$(if $(LOOP),--loop '$(LOOP)') $(if $(BPM),--bpm '$(BPM)') $(if $(SWING),--swing '$(SWING)') \
		$(if $(SOLO),--solo '$(SOLO)') $(if $(MUTE),--mute '$(MUTE)') $(if $(PLAY),--play)

## Binaire du studio que le gestionnaire de liens appelle (celui de CE dépôt ou worktree ; réinstaller pour en changer).
STUDIO_BIN ?= $(CURDIR)/target/release/harmonium_studio

## Enregistre le schéma harmonium:// (macOS : appli de lancement dans ~/Applications ; Linux : .desktop + xdg-mime) :
## un clic sur un lien harmonium://open?… l'ouvre dans le studio.
studio/install-handler:
	cargo build -q -p harmonium_studio --release
	sh harmonium_studio/scripts/install_link_handler.sh '$(STUDIO_BIN)'

## Retire le gestionnaire de liens.
studio/uninstall-handler:
	sh harmonium_studio/scripts/install_link_handler.sh --uninstall

## Studio web : build Trunk dans harmonium_studio/dist (demande `trunk` et `rustup target add wasm32-unknown-unknown`)
studio/web/build: studio/assets
	cd harmonium_studio && trunk build --release

## Studio web servi en local (http://127.0.0.1:8787)
studio/web/serve: studio/assets
	cd harmonium_studio && trunk serve --release

## Ressources de la vitrine : la release fixée par harmonium_studio/web-resources.json (banque SF3, percussion,
## presets Odin des profils), taille et SHA-256 contrôlés, puis copiées à empreinte dans harmonium_studio/resources.
STUDIO_WEB_STAGE := target/studio-web-resources
studio/web/resources:
	python3 harmonium_studio/scripts/web_resources.py check-profiles
	python3 harmonium_studio/scripts/web_resources.py fetch --out $(STUDIO_WEB_STAGE)
	cargo run -q -p harmonium_studio --no-default-features --bin studio_assets -- \
		--main-bank "$$(ls $(STUDIO_WEB_STAGE)/main/*)" \
		--percussion-bank "$$(ls $(STUDIO_WEB_STAGE)/percussion/*)" \
		--odin-dir $(STUDIO_WEB_STAGE)/odin \
		--out $(STUDIO_RESOURCES)

## Site public dans target/site : page d'accueil (harmonium_studio/site) et studio web sous /studio/.
SITE_DIR := target/site
studio/site: studio/web/resources
	rm -rf $(SITE_DIR)
	cd harmonium_studio && trunk build --release --public-url /studio/ --dist $(CURDIR)/$(SITE_DIR)/studio
	rm -f $(SITE_DIR)/studio/.trunk-lock
	cp -R harmonium_studio/site/. $(SITE_DIR)/

## Déploie target/site sur Cloudflare Pages (`wrangler login`, ou CLOUDFLARE_API_TOKEN et CLOUDFLARE_ACCOUNT_ID).
CLOUDFLARE_PROJECT ?= harmonium-website
studio/site/deploy: studio/site
	npx --yes wrangler@4 pages deploy $(SITE_DIR) --project-name $(CLOUDFLARE_PROJECT) --branch main

# BEVY : natif, ressources lues directement par AssetServer, aucune copie dans le studio.
# Les banques reprennent le checkout principal ; les presets Odin sont des fichiers plats.
HARMONIUM_BANKS ?= $(STUDIO_BANKS)
HARMONIUM_ODIN ?= $(HOME)/Project/bascanada/odin-presets

bevy/demo:
	HARMONIUM_BANKS="$(HARMONIUM_BANKS)" HARMONIUM_ODIN="$(HARMONIUM_ODIN)" cargo run -p harmonium_bevy --release --example demo --features demo,odin -- $(ARGS)

## Séparer les compilations pour ne pas unifier Odin/GPL avec la configuration sans Odin.
bevy/test:
	cargo +stable test -p harmonium_bevy --no-default-features
	cargo +stable test -p harmonium_bevy --no-default-features --features odin

bevy/check:
	cargo +stable clippy -p harmonium_bevy --no-deps --no-default-features --all-targets -- -D warnings
	cargo +stable clippy -p harmonium_bevy --no-deps --no-default-features --features demo,odin --all-targets -- -D warnings
	cargo check -p harmonium_bevy --no-default-features --target wasm32-unknown-unknown
	cargo check -p harmonium_bevy --no-default-features --features odin --target wasm32-unknown-unknown

# ════════════════════════════════════════════════════════════════════
# TESTS
# ════════════════════════════════════════════════════════════════════

## Sans `odin` (configuration livrée par 1837), puis studio et Bevy séparément :
## les options s'unifient entre paquets compilés ensemble ; Odin implique GPL.
test:
	cargo +stable test --workspace --exclude harmonium_studio --exclude harmonium_bevy
	cargo +stable test -p harmonium_studio
	cargo +stable test -p harmonium --features odin --test odin_contract_tests
	$(MAKE) bevy/test

test/sheet:
	cargo test -p harmonium_core --test music_generation_tests generate_all_music_tests -- --ignored --nocapture

test/lab-export:
	cargo test -p harmonium_core --test music_generation_tests generate_all_lab_exports -- --ignored --nocapture

## Natif contre wasm : mêmes snapshots octet pour octet (profils distribués, graines 1-3,
## deux réglages d'émotions, 64 mesures). Demande `rustup target add wasm32-unknown-unknown`,
## Node et le CLI wasm-bindgen de la version de Cargo.lock (`WASM_BINDGEN` = son chemin).
test/determinism:
	harmonium_determinism/scripts/run.sh

# ════════════════════════════════════════════════════════════════════
# WASM
# ════════════════════════════════════════════════════════════════════

## Le moteur compile pour le navigateur, avec et sans le synthé Odin
wasm/check:
	cargo check --target wasm32-unknown-unknown -p harmonium_core -p harmonium_audio -p harmonium
	cargo check --target wasm32-unknown-unknown -p harmonium_audio -p harmonium --features odin

# ════════════════════════════════════════════════════════════════════
# QUALITY & CI
# ════════════════════════════════════════════════════════════════════

fmt:
	cargo +nightly fmt --all -- --check

lint:
	cargo +stable clippy --workspace --exclude harmonium_studio --exclude harmonium_bevy --all-targets -- -A clippy::all -W clippy::unwrap_used -W clippy::panic -W clippy::todo -W clippy::expect_used
	cargo +stable clippy -p harmonium_studio --all-targets -- -A clippy::all -W clippy::unwrap_used -W clippy::panic -W clippy::todo -W clippy::expect_used
	$(MAKE) bevy/check

audit:
	cargo audit

quality: fmt lint test
	@echo "✨ All quality checks passed!"
