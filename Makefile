.PHONY: soundfont run run/debug test test/sheet test/lab-export fmt lint audit quality

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
# TESTS
# ════════════════════════════════════════════════════════════════════

test:
	cargo +stable test --workspace

test/sheet:
	cargo test -p harmonium_core --test music_generation_tests generate_all_music_tests -- --ignored --nocapture

test/lab-export:
	cargo test -p harmonium_core --test music_generation_tests generate_all_lab_exports -- --ignored --nocapture

# ════════════════════════════════════════════════════════════════════
# QUALITY & CI
# ════════════════════════════════════════════════════════════════════

fmt:
	cargo +nightly fmt --all -- --check

lint:
	cargo +stable clippy --workspace --all-targets -- -A clippy::all -W clippy::unwrap_used -W clippy::panic -W clippy::todo -W clippy::expect_used

audit:
	cargo audit

quality: fmt lint test
	@echo "✨ All quality checks passed!"
