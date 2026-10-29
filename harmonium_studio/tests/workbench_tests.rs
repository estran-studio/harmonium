//! L'atelier sans fenêtre : copie d'un profil distribué, dossier enregistré relu par le décodeur
//! de `harmonium-cli --profile`, archive téléchargée puis téléversée, coloration TOML.

use std::io::Cursor;

use harmonium_core::StyleProfile;
use harmonium_studio::{
    platform::{read_profile_folder, write_profile_folder},
    workbench::{Draft, Origin, PROFILE_FILE, TomlToken, files_from_upload, toml_spans},
};

fn distributed_files(id: &str) -> Result<&'static [(&'static str, &'static str)], String> {
    let index = StyleProfile::distributed()
        .iter()
        .position(|profile| profile.meta.id == id)
        .ok_or(format!("profil {id}"))?;
    Ok(StyleProfile::distributed_files()[index])
}

#[test]
fn a_fork_keeps_every_file_and_comment_under_a_new_identity() -> Result<(), String> {
    let draft = Draft::fork("reel_quebecois")?;
    let original = distributed_files("reel_quebecois")?;
    let names: Vec<&str> = draft.files.iter().map(|file| file.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "profile.toml",
            "cells.toml",
            "feet.toml",
            "arrangement.toml",
            "dynamics.toml",
            "set.toml"
        ],
        "profile.toml, puis les banques dans l'ordre de son [files]"
    );
    assert_eq!(draft.files.len(), original.len());

    for file in &draft.files {
        let text = original
            .iter()
            .find(|(name, _)| *name == file.name)
            .map(|(_, text)| *text)
            .ok_or(format!("{} dans l'original", file.name))?;
        if file.name == PROFILE_FILE {
            // Seules les lignes `id` et `name` de [meta] changent ; commentaires et mise en forme
            // restent.
            let changed: Vec<(&str, &str)> =
                file.text.lines().zip(text.lines()).filter(|(a, b)| a != b).collect();
            assert_eq!(file.text.lines().count(), text.lines().count());
            assert_eq!(
                changed,
                vec![
                    ("id = \"reel_quebecois_fork\"", "id = \"reel_quebecois\""),
                    ("name = \"Reel québécois (fork)\"", "name = \"Reel québécois\""),
                ]
            );
        } else {
            assert_eq!(file.text, text, "{} copié tel quel", file.name);
        }
    }

    // La musique est celle de l'original : seul [meta] diffère.
    let mut fork = draft.decode().map_err(|e| e.to_string())?;
    let original = StyleProfile::named("reel_quebecois").map_err(|e| e.to_string())?;
    assert_eq!(fork.meta.id, "reel_quebecois_fork");
    fork.meta.id.clone_from(&original.meta.id);
    fork.meta.name.clone_from(&original.meta.name);
    assert_eq!(fork, original);
    Ok(())
}

#[test]
fn a_saved_folder_loads_like_harmonium_cli_profile_and_reopens_identically() -> Result<(), String> {
    let mut draft = Draft::fork("isaac_basement")?;
    let edited = draft.files.iter_mut().find(|f| f.name == PROFILE_FILE).ok_or("profile.toml")?;
    edited.text.push_str("\n# Modifié dans l'atelier.\n");
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let folder = dir.path().join("isaac_basement_fork");

    assert_eq!(write_profile_folder(&folder, &draft)?, draft.files.len());

    // `harmonium-cli --profile <dossier>` lit par `StyleProfile::load`.
    let loaded = StyleProfile::load(&folder).map_err(|e| e.to_string())?;
    assert_eq!(loaded, draft.decode().map_err(|e| e.to_string())?);
    let reopened = read_profile_folder(&folder)?;
    assert_eq!(reopened.files, draft.files);
    assert_eq!(reopened.origin, Origin::Folder(folder.display().to_string()));
    Ok(())
}

#[test]
fn a_saved_folder_refuses_names_that_leave_it() -> Result<(), String> {
    let draft = Draft::new(
        Origin::Upload,
        vec![
            (PROFILE_FILE.into(), "[files]\ncells = \"../cells.toml\"\n".into()),
            ("../cells.toml".into(), String::new()),
        ],
    )?;
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let folder = dir.path().join("inside");
    assert!(write_profile_folder(&folder, &draft).is_err());
    assert!(!dir.path().join("cells.toml").exists(), "rien d'écrit hors du dossier");
    Ok(())
}

#[test]
fn the_downloaded_archive_unzips_into_a_loadable_folder_and_uploads_back() -> Result<(), String> {
    let draft = Draft::fork("nt_desert")?;
    let bytes = draft.to_zip()?;

    // Décompressée, l'archive donne le dossier `<identifiant>/`, que `--profile` lit.
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    zip::ZipArchive::new(Cursor::new(&bytes))
        .and_then(|mut archive| archive.extract(dir.path()))
        .map_err(|e| e.to_string())?;
    let loaded =
        StyleProfile::load(dir.path().join("nt_desert_fork")).map_err(|e| e.to_string())?;
    assert_eq!(loaded, draft.decode().map_err(|e| e.to_string())?);

    // Téléversée, elle redonne le même brouillon.
    let files = files_from_upload(vec![("nt_desert_fork.zip".into(), bytes)])?;
    let uploaded = Draft::new(Origin::Upload, files)?;
    assert_eq!(uploaded.files, draft.files);
    Ok(())
}

#[test]
fn an_upload_keeps_the_profile_files_only() -> Result<(), String> {
    let draft = Draft::fork("valse_quebecoise")?;
    let mut picked: Vec<(String, Vec<u8>)> =
        draft.files.iter().map(|f| (f.name.clone(), f.text.clone().into_bytes())).collect();
    picked.push(("README.md".into(), b"# Notes\n".to_vec()));
    picked.push(("cover.png".into(), vec![0x89, b'P', b'N', b'G', 0xff]));
    let uploaded = Draft::new(Origin::Upload, files_from_upload(picked)?)?;
    let names: Vec<&str> = uploaded.files.iter().map(|f| f.name.as_str()).collect();
    let mut expected: Vec<&str> = draft.files.iter().map(|f| f.name.as_str()).collect();
    expected.push("README.md");
    assert_eq!(names, expected, "profile.toml, ses banques dans l'ordre de [files], puis le reste");

    // Sans profile.toml, rien à éditer.
    let banks_only: Vec<(String, Vec<u8>)> = draft
        .files
        .iter()
        .filter(|f| f.name != PROFILE_FILE)
        .map(|f| (f.name.clone(), f.text.clone().into_bytes()))
        .collect();
    assert!(files_from_upload(banks_only).is_err());
    Ok(())
}

#[test]
fn an_archive_of_a_nested_folder_drops_what_is_outside_the_profile() -> Result<(), String> {
    use std::io::Write;

    let draft = Draft::fork("gigue_acadienne")?;
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    let mut add = |name: &str, text: &str| -> Result<(), String> {
        writer.start_file(name, options).map_err(|e| e.to_string())?;
        writer.write_all(text.as_bytes()).map_err(|e| e.to_string())
    };
    // Un dossier compressé par le Finder : un parent, ses métadonnées `__MACOSX`, un voisin.
    for file in &draft.files {
        add(&format!("work/gigue/{}", file.name), &file.text)?;
        add(&format!("__MACOSX/work/gigue/._{}", file.name), "")?;
    }
    add("work/other.toml", "x = 1\n")?;
    let bytes = writer.finish().map_err(|e| e.to_string())?.into_inner();

    let files = files_from_upload(vec![("Archive.zip".into(), bytes)])?;
    let uploaded = Draft::new(Origin::Upload, files)?;
    assert_eq!(uploaded.files, draft.files);
    Ok(())
}

#[test]
fn toml_spans_cover_the_text_and_tell_keys_from_values() {
    let text = "[meta]\nid = \"a#b\" # vrai commentaire\n[[form.parts]]\nrhythm = [\n  [1, 2],\n  { bar = 4, weight = 0.5 },\n]\nflag = true\n";
    let spans = toml_spans(text);
    let mut covered = 0;
    for (range, _) in &spans {
        assert_eq!(range.start, covered, "contigus");
        covered = range.end;
    }
    assert_eq!(covered, text.len(), "tout le texte");

    let kind_of = |needle: &str| {
        let at = text.find(needle).unwrap_or(usize::MAX);
        spans.iter().find(|(range, _)| range.contains(&at)).map(|(_, token)| *token)
    };
    assert_eq!(kind_of("[meta]"), Some(TomlToken::Header));
    assert_eq!(kind_of("[[form.parts]]"), Some(TomlToken::Header));
    assert_eq!(kind_of("id ="), Some(TomlToken::Key));
    assert_eq!(kind_of("\"a#b\""), Some(TomlToken::String), "un # dans une chaîne");
    assert_eq!(kind_of("# vrai"), Some(TomlToken::Comment));
    // Un tableau en début de ligne, dans une valeur, n'est pas un en-tête.
    assert_eq!(kind_of("[1, 2]"), Some(TomlToken::Plain));
    assert_eq!(kind_of("1, 2]"), Some(TomlToken::Literal));
    assert_eq!(kind_of("bar ="), Some(TomlToken::Key), "clé d'une table en ligne");
    assert_eq!(kind_of("weight"), Some(TomlToken::Key));
    assert_eq!(kind_of("0.5"), Some(TomlToken::Literal));
    assert_eq!(kind_of("flag"), Some(TomlToken::Key), "nouvelle clé après le tableau fermé");
    assert_eq!(kind_of("true"), Some(TomlToken::Literal));
}
