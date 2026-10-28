#!/usr/bin/env bash
# Compare octet par octet les snapshots natifs et wasm (voir `harmonium_determinism/src/lib.rs`).
#
# Prérequis : cible `wasm32-unknown-unknown`, Node, et un CLI `wasm-bindgen` de la version du crate
# verrouillé dans Cargo.lock (`cargo install --locked wasm-bindgen-cli --version <x.y.z>`).
# Variable : WASM_BINDGEN = chemin du CLI (défaut : `wasm-bindgen` du PATH).
# `run.sh --locked-version` affiche la version du CLI à installer, sans rien exécuter.
set -euo pipefail

cd "$(dirname "$0")/../.."
OUT=target/determinism
BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"

locked=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/[^0-9.]/, "", $0); print; exit }' Cargo.lock)
if [ "${1:-}" = "--locked-version" ]; then
  echo "$locked"
  exit 0
fi
installed=$("$BINDGEN" --version | awk '{ print $2 }')
if [ "$locked" != "$installed" ]; then
  echo "wasm-bindgen CLI $installed, Cargo.lock verrouille $locked :" >&2
  echo "  cargo install --locked wasm-bindgen-cli --version $locked" >&2
  exit 1
fi

rm -rf "$OUT"
cargo run --release -p harmonium_determinism --bin determinism_dump -- "$OUT/native"
cargo build --release --target wasm32-unknown-unknown -p harmonium_determinism --lib
"$BINDGEN" --target nodejs --out-dir "$OUT/pkg" \
  target/wasm32-unknown-unknown/release/harmonium_determinism.wasm
node harmonium_determinism/scripts/dump_wasm.cjs "$OUT/pkg" "$OUT/wasm"

same=0
different=0
for native in "$OUT"/native/*; do
  name=$(basename "$native")
  if cmp -s "$native" "$OUT/wasm/$name"; then
    same=$((same + 1))
  else
    different=$((different + 1))
    echo "DIFFÈRE : $name" >&2
  fi
done
total=$(ls "$OUT/native" | wc -l | tr -d ' ')
extra=$(ls "$OUT/wasm" | wc -l | tr -d ' ')
echo "natif = wasm : $same / $total fichiers identiques (différents : $different, côté wasm : $extra)"
[ "$different" -eq 0 ] && [ "$total" -eq "$extra" ]
