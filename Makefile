.PHONY: soundfont run run/debug studio studio/assets studio/web/build studio/web/serve test test/sheet test/lab-export test/determinism wasm/check fmt lint audit quality

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
## Dossier de presets Odin (`*.odin`) ; vide : aucun.
STUDIO_ODIN ?=
STUDIO_RESOURCES := harmonium_studio/resources

## Copie les banques en fichiers à empreinte dans harmonium_studio/resources, avec assets.json
studio/assets:
	cargo run -q -p harmonium_studio --no-default-features --bin studio_assets -- \
		--main-bank $(STUDIO_BANKS)/GeneralUser-GS.sf2 \
		--feet-bank $(STUDIO_BANKS)/harmonium_trad.sf2 \
		$(if $(STUDIO_ODIN),--odin-dir $(STUDIO_ODIN)) \
		--out $(STUDIO_RESOURCES)

## Studio natif (fenêtre)
studio: studio/assets
	cargo run -p harmonium_studio --release -- $(ARGS)

## Studio web : build Trunk dans harmonium_studio/dist (demande `trunk` et `rustup target add wasm32-unknown-unknown`)
studio/web/build: studio/assets
	cd harmonium_studio && trunk build --release

## Studio web servi en local (http://127.0.0.1:8787)
studio/web/serve: studio/assets
	cd harmonium_studio && trunk serve --release

# ════════════════════════════════════════════════════════════════════
# TESTS
# ════════════════════════════════════════════════════════════════════

## Sans `odin` (configuration livrée par 1837), puis le studio (toujours avec `odin`, donc à part :
## les options s'unifient entre paquets compilés ensemble), puis le contrat Odin du host.
test:
	cargo +stable test --workspace --exclude harmonium_studio
	cargo +stable test -p harmonium_studio
	cargo +stable test -p harmonium --features odin --test odin_contract_tests

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
	cargo +stable clippy --workspace --exclude harmonium_studio --all-targets -- -A clippy::all -W clippy::unwrap_used -W clippy::panic -W clippy::todo -W clippy::expect_used
	cargo +stable clippy -p harmonium_studio --all-targets -- -A clippy::all -W clippy::unwrap_used -W clippy::panic -W clippy::todo -W clippy::expect_used

audit:
	cargo audit

quality: fmt lint test
	@echo "✨ All quality checks passed!"
