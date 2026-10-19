.PHONY: run run/debug play test test/sheet test/lab-export fmt lint audit quality

# ════════════════════════════════════════════════════════════════════
# RUN
# ════════════════════════════════════════════════════════════════════

## Interactive REPL (harmonium_cli)
run:
	HARMONIUM_CLI=1 cargo run -p harmonium-cli --release -- $(ARGS)

run/debug:
	HARMONIUM_CLI=1 cargo run -p harmonium-cli -- $(ARGS)

## Standalone player with OSC control (binary `harmonium`)
play:
	cargo run -p harmonium --release --bin harmonium -- $(ARGS)

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
