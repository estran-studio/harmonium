//! Contrat du chargement : ce que le chargeur garde au cache, supprime, contrôle, et ce que l'écran
//! en dit.

use harmonium_studio::{
    loading::{
        CacheDecision, CacheState, LoadState, Phase, ResourceStatus, Source, cache_key, check_size,
        decide, megabytes, parse_buffer_frames, stale_entries,
    },
    manifest::{
        FEET_BANK_ID, LoadedResources, MAIN_BANK_ID, Manifest, Resource, hashed_file_name,
        sha256_hex,
    },
};

fn resource(id: &str, original: &str, content: &[u8], bytes: u64) -> Resource {
    let sha256 = sha256_hex(content);
    Resource { id: id.to_owned(), file: hashed_file_name(original, &sha256), bytes, sha256 }
}

fn manifest() -> Manifest {
    Manifest {
        resources: vec![
            resource(MAIN_BANK_ID, "bank.sf2", b"bank", 32_319_396),
            resource(FEET_BANK_ID, "feet.sf2", b"feet", 154_186),
        ],
    }
}

#[test]
fn the_cache_key_is_the_hashed_file_so_new_content_is_a_new_key() {
    let one = resource(MAIN_BANK_ID, "bank.sf2", b"one", 3);
    let two = resource(MAIN_BANK_ID, "bank.sf2", b"two", 3);
    assert!(cache_key(&one).ends_with(&one.file));
    assert_ne!(cache_key(&one), cache_key(&two));
}

#[test]
fn only_an_entry_of_the_manifest_size_is_used() {
    let bank = resource(MAIN_BANK_ID, "bank.sf2", b"bank", 1000);
    assert_eq!(decide(&bank, Some(1000)), CacheDecision::UseCached);
    assert_eq!(decide(&bank, Some(999)), CacheDecision::Download, "truncated entry");
    assert_eq!(decide(&bank, Some(1001)), CacheDecision::Download, "oversized entry");
    assert_eq!(decide(&bank, Some(0)), CacheDecision::Download);
    assert_eq!(decide(&bank, None), CacheDecision::Download, "no entry");
}

#[test]
fn a_download_of_the_wrong_size_never_passes() {
    let bank = resource(MAIN_BANK_ID, "bank.sf2", b"bank", 1000);
    assert!(check_size(&bank, 1000).is_ok());
    assert!(check_size(&bank, 600).is_err(), "cut short");
    assert!(check_size(&bank, 1400).is_err(), "longer than announced");
    assert!(check_size(&bank, 0).is_err());
}

#[test]
fn after_a_load_only_entries_outside_the_manifest_are_deleted() {
    let manifest = manifest();
    let current: Vec<String> = manifest.resources.iter().map(cache_key).collect();
    let old_bank = "https://example.org/studio/resources/bank.deadbeef0000.sf2".to_owned();
    let mut cached = current.clone();
    cached.push(old_bank.clone());
    // Une adresse absolue, une requête de version : la clé reste le nom de fichier.
    cached.push(format!("https://example.org/studio/{}?v=2", current[1]));

    assert_eq!(stale_entries(&cached, &manifest), [old_bank.as_str()]);
    assert!(stale_entries(&current, &manifest).is_empty(), "nothing to prune");
    assert_eq!(stale_entries(&[], &manifest), Vec::<&str>::new());
}

#[test]
fn an_entry_the_manifest_does_not_name_is_stale() {
    let manifest = Manifest { resources: vec![resource(MAIN_BANK_ID, "bank.sf2", b"b", 1)] };
    let cached = vec!["resources/feet.000000000000.sf2".to_owned()];
    assert_eq!(stale_entries(&cached, &manifest).len(), 1);
}

#[test]
fn a_cached_address_with_encoded_characters_still_names_its_file() {
    // Le navigateur garde l'adresse encodée ; le manifeste, le nom tel quel (presets Odin).
    let preset = resource("odin:Bass Crusher [RM]", "Bass Crusher [RM].odin", b"odin", 4);
    let manifest = Manifest { resources: vec![preset.clone()] };
    let encoded = preset.file.replace(' ', "%20").replace('[', "%5B").replace(']', "%5D");
    let cached = vec![
        format!("https://example.org/studio/resources/{encoded}"),
        format!("https://example.org/studio/resources/{}", preset.file.replace(' ', "%20")),
    ];
    assert!(stale_entries(&cached, &manifest).is_empty(), "the cached preset is kept");
}

#[test]
fn buffer_sizes_are_validated() {
    assert_eq!(parse_buffer_frames("2048"), Ok(2048));
    assert_eq!(parse_buffer_frames(" 4096 "), Ok(4096));
    assert!(parse_buffer_frames("20").is_err(), "too small");
    assert!(parse_buffer_frames("2048000").is_err(), "too large");
    assert!(parse_buffer_frames("abc").is_err());
    assert!(parse_buffer_frames("-1").is_err());
}

fn state_with(manifest: &Manifest) -> LoadState {
    let mut state = LoadState::new();
    assert_eq!(state.phase, Phase::Manifest);
    state.manifest_loaded(manifest);
    assert_eq!(state.phase, Phase::Resources);
    state
}

#[test]
fn each_line_says_where_the_resource_is() {
    let mut state = state_with(&manifest());
    let text = |state: &LoadState, index: usize| state.view().lines[index].status_text();

    state.set_status(MAIN_BANK_ID, ResourceStatus::Downloading(12_345_678));
    assert_eq!(text(&state, 0), "downloading 12.3 / 32.3 MB");
    state.set_status(MAIN_BANK_ID, ResourceStatus::Loaded(Source::Device));
    assert_eq!(text(&state, 0), "on this device");
    state.set_status(FEET_BANK_ID, ResourceStatus::Failed("HTTP 404".into()));
    assert!(text(&state, 1).contains("HTTP 404"));
    assert_eq!(megabytes(154_186), "0.2");
}

#[test]
fn progress_is_weighted_by_size_and_needs_no_content_length() {
    let mut state = state_with(&manifest());
    assert_eq!(state.fraction(), 0.0);
    state.set_status(MAIN_BANK_ID, ResourceStatus::Downloading(16_159_698));
    let half_of_the_bank = 16_159_698.0 / 32_473_582.0;
    assert!((state.fraction() - half_of_the_bank).abs() < 1e-4);
    state.set_status(MAIN_BANK_ID, ResourceStatus::Loaded(Source::Network));
    state.set_status(FEET_BANK_ID, ResourceStatus::Loaded(Source::Network));
    assert!((state.fraction() - 1.0).abs() < 1e-6);
}

#[test]
fn the_first_visit_says_that_the_next_ones_load_from_the_device() {
    let mut state = state_with(&manifest());
    state.cache_working();
    assert!(state.notices().is_empty(), "nothing downloaded yet");
    state.set_status(MAIN_BANK_ID, ResourceStatus::Downloading(10));
    let notices = state.notices();
    assert_eq!(notices.len(), 1);
    assert!(notices[0].contains("next visits load from this device"));
}

#[test]
fn a_return_visit_from_the_cache_has_no_notice() {
    let mut state = state_with(&manifest());
    state.cache_working();
    state.set_status(MAIN_BANK_ID, ResourceStatus::Loaded(Source::Device));
    state.set_status(FEET_BANK_ID, ResourceStatus::Loaded(Source::Device));
    assert!(state.notices().is_empty());
}

#[test]
fn a_refused_cache_warns_that_the_next_visit_downloads_again() {
    let mut state = state_with(&manifest());
    state.cache_working();
    state.set_status(MAIN_BANK_ID, ResourceStatus::Loaded(Source::Network));
    state.cache_unavailable("QuotaExceededError");
    assert_eq!(state.cache, CacheState::Unavailable("QuotaExceededError".into()));
    let notices = state.notices();
    assert_eq!(notices.len(), 1);
    assert!(notices[0].contains("next visit downloads them again"));
    // Un cache qui marchait puis a échoué ne redevient pas « working ».
    state.cache_working();
    assert!(matches!(state.cache, CacheState::Unavailable(_)));
}

#[test]
fn the_resources_leave_the_state_once_and_a_failure_is_kept() {
    let manifest = manifest();
    let mut state = state_with(&manifest);
    let loaded = LoadedResources { main_bank: b"bank".to_vec(), ..LoadedResources::default() };
    state.finish(loaded.clone());
    assert_eq!(state.phase, Phase::Done);
    assert_eq!(state.take_resources(), Some(loaded));
    assert_eq!(state.take_resources(), None);

    let mut failed = state_with(&manifest);
    failed.fail("HTTP 500");
    assert_eq!(failed.phase, Phase::Failed("HTTP 500".into()));
}
