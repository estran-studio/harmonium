// Écrit, depuis le module wasm chargé par Node, les mêmes fichiers que `determinism_dump` :
// node dump_wasm.cjs <dossier du paquet wasm-bindgen --target nodejs> <dossier de sortie>
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const [pkgDir, outDir] = process.argv.slice(2);
if (!pkgDir || !outDir) {
  console.error("usage : node dump_wasm.cjs <paquet wasm> <dossier de sortie>");
  process.exit(2);
}

const engine = require(path.resolve(pkgDir, "harmonium_determinism.js"));
fs.mkdirSync(outDir, { recursive: true });

const names = engine.case_names();
names.forEach((name, index) => {
  fs.writeFileSync(path.join(outDir, name), engine.run_case(index));
});
const probes = engine.probe_names();
probes.forEach((name, index) => {
  fs.writeFileSync(path.join(outDir, name), engine.run_probe(index));
});
console.log(`${names.length + probes.length} fichiers écrits dans ${outDir}`);
