//! Sources multi-fichiers d'un profil : montage des banques et localisation des erreurs.
//!
//! Le décodeur fusionne `profile.toml` et ses banques dans un seul arbre avant de le
//! désérialiser ; ce module garde, pour chaque fichier lu, l'endroit de l'arbre où il est monté.
//! Quand un chemin de champ échoue (désérialisation ou `validate()`), [`locate`] retrouve le
//! fichier qui le porte et la ligne de la valeur fautive.

use std::borrow::Cow;

use toml_edit::ImDocument;

use crate::style_profile::ProfileError;

/// Un fichier source monté dans l'arbre du profil.
pub(crate) struct Mount<'a> {
    /// Chemin de champ où le contenu du fichier est inséré (`""` : racine du profil).
    pub at: &'static str,
    /// Chemin, dans le fichier, qui correspond à `at` (`""` : racine du fichier).
    pub as_path: &'static str,
    pub file: String,
    pub text: Cow<'a, str>,
}

/// Ligne et colonne (base 1, en caractères) d'un décalage en octets.
fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |last| last.chars().count()) + 1;
    (line, column)
}

/// Message de `toml` sur une seule ligne (il en met une par détail).
pub(crate) fn one_line(message: &str) -> String {
    message.trim().replace('\n', ", ")
}

/// Lit un fichier TOML ; une erreur de syntaxe porte le fichier, la ligne et la colonne.
pub(crate) fn parse(mount: &Mount<'_>) -> Result<toml::Value, ProfileError> {
    toml::from_str(&mount.text).map_err(|error| {
        let at = error.span().map(|span| position(&mount.text, span.start));
        ProfileError {
            field: String::new(),
            message: one_line(error.message()),
            file: Some(mount.file.clone()),
            line: at.map(|(line, _)| line),
            column: at.map(|(_, column)| column),
        }
    })
}

enum Segment<'a> {
    Key(&'a str),
    Index(usize),
}

/// Découpe `a.b[2].c` en segments ; les chemins de `validate()` et de `serde_path_to_error`
/// suivent cette forme.
fn segments(path: &str) -> Vec<Segment<'_>> {
    let mut out = Vec::new();
    for part in path.split('.').filter(|part| !part.is_empty()) {
        let (key, mut indexes) = part.split_once('[').unwrap_or((part, ""));
        if !key.is_empty() {
            out.push(Segment::Key(key));
        }
        while !indexes.is_empty() {
            let Some((index, rest)) = indexes.split_once(']') else { break };
            let Ok(index) = index.parse() else { break };
            out.push(Segment::Index(index));
            indexes = rest.strip_prefix('[').unwrap_or("");
        }
    }
    out
}

/// Position (clé si connue, sinon valeur) du segment le plus profond du chemin qui existe
/// dans le fichier. Un chemin qui ne correspond à rien dans le fichier n'a pas de position.
fn find_line(text: &str, path: &str) -> Option<(usize, usize)> {
    let document = ImDocument::parse(text).ok()?;
    let mut item = document.as_item();
    let mut found = None;
    for segment in segments(path) {
        let (key_span, next) = match segment {
            Segment::Key(key) => {
                let Some((key, next)) =
                    item.as_table_like().and_then(|table| table.get_key_value(key))
                else {
                    break;
                };
                (key.span(), next)
            }
            Segment::Index(index) => match item.get(index) {
                Some(next) => (None, next),
                None => break,
            },
        };
        // Les tables implicites (`[a.b]` sans `[a]`) n'ont pas de position : on garde celle du parent.
        if let Some(span) = key_span.or_else(|| next.span()) {
            found = Some(span.start);
        }
        item = next;
    }
    found.map(|offset| position(text, offset))
}

fn under(path: &str, at: &str) -> Option<usize> {
    if at.is_empty() {
        return Some(0);
    }
    let rest = path.strip_prefix(at)?;
    (rest.is_empty() || rest.starts_with(['.', '['])).then_some(at.len())
}

/// Complète une erreur de champ avec le fichier qui porte ce champ et la ligne de sa valeur.
/// Une erreur déjà située (syntaxe) reste inchangée.
pub(crate) fn locate(mounts: &[Mount<'_>], mut error: ProfileError) -> ProfileError {
    if error.file.is_some() {
        return error;
    }
    let Some((mount, skip)) = mounts
        .iter()
        .filter_map(|mount| under(&error.field, mount.at).map(|len| (mount, len)))
        .max_by_key(|(_, len)| *len)
    else {
        return error;
    };
    error.file = Some(mount.file.clone());
    let rest = error.field[skip..].trim_start_matches('.');
    let path = match (mount.as_path.is_empty(), rest.is_empty()) {
        (_, true) => mount.as_path.to_owned(),
        (true, false) => rest.to_owned(),
        (false, false) if rest.starts_with('[') => format!("{}{rest}", mount.as_path),
        (false, false) => format!("{}.{rest}", mount.as_path),
    };
    if let Some((line, column)) = find_line(&mount.text, &path) {
        error.line = Some(line);
        error.column = Some(column);
    }
    error
}
