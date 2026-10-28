//! Contrat du manifeste et de son générateur : ce que le chargeur lit est ce que le build écrit, et
//! une source différente (autre format de banque, presets Odin) ne change que des fichiers.

use std::{collections::BTreeMap, fs};

use harmonium_studio::{
    manifest::{
        FEET_BANK_ID, HASH_IN_NAME, LoadedResources, MAIN_BANK_ID, MANIFEST_FILE, Manifest,
        Resource, Role, hashed_file_name, sha256_hex,
    },
    packager::{PackSpec, pack},
};
use tempfile::tempdir;

fn resource(id: &str, file: &str, bytes: u64, content: &[u8]) -> Resource {
    Resource { id: id.to_owned(), file: file.to_owned(), bytes, sha256: sha256_hex(content) }
}

fn hashed(original: &str, content: &[u8]) -> String {
    hashed_file_name(original, &sha256_hex(content))
}

fn manifest_json(resources: &[Resource]) -> String {
    Manifest { resources: resources.to_vec() }.to_json().unwrap()
}

fn main_bank(content: &[u8]) -> Resource {
    resource(MAIN_BANK_ID, &hashed("bank.sf2", content), content.len() as u64, content)
}

#[test]
fn sha256_is_the_standard_digest() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn file_name_carries_the_content_hash_and_keeps_the_extension() {
    let sha = sha256_hex(b"abc");
    let name = hashed_file_name("GeneralUser-GS.sf2", &sha);
    assert_eq!(name, format!("GeneralUser-GS.{}.sf2", &sha[..HASH_IN_NAME]));
    assert_ne!(name, hashed_file_name("GeneralUser-GS.sf2", &sha256_hex(b"abd")));
}

#[test]
fn a_manifest_round_trips_and_roles_are_known() {
    let preset = resource("odin:lead", &hashed("lead.odin", b"p"), 1, b"p");
    let json = manifest_json(&[main_bank(b"bank"), preset]);
    let manifest = Manifest::parse(&json).unwrap();
    assert_eq!(manifest.resources.len(), 2);
    assert_eq!(manifest.resources[0].role(), Some(Role::MainBank));
    assert_eq!(manifest.resources[1].role(), Some(Role::OdinPreset("lead")));
    assert_eq!(manifest.total_bytes(), 5);
}

#[test]
fn a_manifest_with_no_odin_preset_is_normal() {
    let json = manifest_json(&[main_bank(b"bank")]);
    let manifest = Manifest::parse(&json).unwrap();
    let mut contents = BTreeMap::new();
    contents.insert(MAIN_BANK_ID.to_owned(), b"bank".to_vec());
    let loaded = LoadedResources::assemble(&manifest, contents).unwrap();
    assert_eq!(loaded.main_bank, b"bank");
    assert_eq!(loaded.feet_bank, None);
    assert!(loaded.odin_presets.is_empty());
}

#[test]
fn resources_are_sorted_into_their_roles() {
    let feet = resource(FEET_BANK_ID, &hashed("feet.sf2", b"f"), 1, b"f");
    let b = resource("odin:b", &hashed("b.odin", b"b"), 1, b"b");
    let a = resource("odin:a", &hashed("a.odin", b"a"), 1, b"a");
    let manifest = Manifest { resources: vec![b, main_bank(b"bank"), feet, a] };
    let contents = BTreeMap::from([
        ("odin:a".to_owned(), b"a".to_vec()),
        ("odin:b".to_owned(), b"b".to_vec()),
        (MAIN_BANK_ID.to_owned(), b"bank".to_vec()),
        (FEET_BANK_ID.to_owned(), b"f".to_vec()),
    ]);
    let loaded = LoadedResources::assemble(&manifest, contents).unwrap();
    assert_eq!(loaded.feet_bank.as_deref(), Some(&b"f"[..]));
    let names: Vec<&str> = loaded.odin_presets.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn a_resource_that_was_not_loaded_is_an_error() {
    let manifest = Manifest { resources: vec![main_bank(b"bank")] };
    assert!(LoadedResources::assemble(&manifest, BTreeMap::new()).is_err());
}

#[test]
fn the_manifest_refuses_what_the_loader_could_not_trust() {
    let good = main_bank(b"bank");
    let refused = |resources: &[Resource], why: &str| {
        let result = Manifest::parse(&manifest_json(resources));
        assert!(result.is_err(), "{why}: accepted");
    };
    refused(&[], "no main bank");
    refused(&[resource("feet-bank", &hashed("f.sf2", b"f"), 1, b"f")], "feet bank alone");
    refused(&[good.clone(), good.clone()], "duplicate id");
    refused(&[good.clone(), resource("mystery", &hashed("m.bin", b"m"), 1, b"m")], "unknown id");
    refused(
        &[good.clone(), resource("odin:", &hashed("m.odin", b"m"), 1, b"m")],
        "empty preset name",
    );
    refused(&[Resource { file: "../bank.sf2".into(), ..good.clone() }], "path in file name");
    refused(&[Resource { file: "sub/bank.sf2".into(), ..good.clone() }], "directory in file name");
    refused(&[Resource { file: "bank.sf2".into(), ..good.clone() }], "file name without the hash");
    refused(&[Resource { sha256: "XYZ".into(), ..good.clone() }], "malformed sha256");
    refused(&[Resource { bytes: 0, ..good }], "empty resource");
    assert!(Manifest::parse("not json").is_err());
}

fn spec(dir: &std::path::Path, main: &[u8], feet: Option<&[u8]>) -> PackSpec {
    fs::write(dir.join("main.sf2"), main).unwrap();
    if let Some(feet) = feet {
        fs::write(dir.join("feet.sf2"), feet).unwrap();
    }
    PackSpec {
        main_bank: dir.join("main.sf2"),
        feet_bank: feet.map(|_| dir.join("feet.sf2")),
        odin_dir: None,
    }
}

#[test]
fn pack_writes_hashed_copies_and_a_manifest_the_loader_accepts() {
    let src = tempdir().unwrap();
    let out = tempdir().unwrap();
    let manifest = pack(&spec(src.path(), b"main bank", Some(b"feet")), out.path()).unwrap();

    let json = fs::read_to_string(out.path().join(MANIFEST_FILE)).unwrap();
    assert_eq!(Manifest::parse(&json).unwrap(), manifest);
    for resource in &manifest.resources {
        let bytes = fs::read(out.path().join(&resource.file)).unwrap();
        assert_eq!(bytes.len() as u64, resource.bytes);
        assert_eq!(sha256_hex(&bytes), resource.sha256);
    }
    assert_eq!(manifest.resources[0].id, MAIN_BANK_ID);
    // Les sources ne bougent pas.
    assert_eq!(fs::read(src.path().join("main.sf2")).unwrap(), b"main bank");
}

#[test]
fn a_changed_bank_changes_its_file_name_and_the_old_copy_goes_away() {
    let src = tempdir().unwrap();
    let out = tempdir().unwrap();
    let first = pack(&spec(src.path(), b"version one", Some(b"feet")), out.path()).unwrap();
    let second = pack(&spec(src.path(), b"version two!", Some(b"feet")), out.path()).unwrap();

    let (old, new) = (&first.resources[0], &second.resources[0]);
    assert_eq!(old.id, new.id);
    assert_ne!(old.file, new.file);
    assert!(!out.path().join(&old.file).exists(), "stale copy kept");
    assert!(out.path().join(&new.file).exists());
    // La banque de pieds, inchangée, garde son nom et son fichier.
    assert_eq!(first.resources[1], second.resources[1]);
    assert!(out.path().join(&second.resources[1].file).exists());
}

#[test]
fn another_bank_format_is_only_another_source_file() {
    let src = tempdir().unwrap();
    let out = tempdir().unwrap();
    let sf2 = pack(&spec(src.path(), b"sf2 content", None), out.path()).unwrap();
    fs::write(src.path().join("main.sf3"), b"sf3 content, smaller").unwrap();
    let sf3 = pack(
        &PackSpec { main_bank: src.path().join("main.sf3"), feet_bank: None, odin_dir: None },
        out.path(),
    )
    .unwrap();
    assert_eq!(sf2.resources[0].id, sf3.resources[0].id);
    assert!(sf3.resources[0].file.ends_with(".sf3"));
    assert!(Manifest::parse(&sf3.to_json().unwrap()).is_ok());
}

#[test]
fn odin_presets_are_found_by_extension_and_zero_is_fine() {
    let src = tempdir().unwrap();
    let out = tempdir().unwrap();
    let presets = src.path().join("presets");
    fs::create_dir(&presets).unwrap();
    fs::write(presets.join("pad.odin"), b"pad").unwrap();
    fs::write(presets.join("lead.odin"), b"lead").unwrap();
    fs::write(presets.join("notes.txt"), b"ignored").unwrap();

    let mut with = spec(src.path(), b"main", None);
    with.odin_dir = Some(presets);
    let manifest = pack(&with, out.path()).unwrap();
    let ids: Vec<&str> = manifest.resources.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, [MAIN_BANK_ID, "odin:lead", "odin:pad"]);

    // Dossier absent ou vide : aucun preset, pas d'erreur.
    let mut none = spec(src.path(), b"main", None);
    none.odin_dir = Some(src.path().join("does-not-exist"));
    assert_eq!(pack(&none, out.path()).unwrap().resources.len(), 1);
}

#[test]
fn a_missing_source_is_an_error_not_an_empty_manifest() {
    let src = tempdir().unwrap();
    let out = tempdir().unwrap();
    let missing =
        PackSpec { main_bank: src.path().join("nope.sf2"), feet_bank: None, odin_dir: None };
    assert!(pack(&missing, out.path()).is_err());
}
