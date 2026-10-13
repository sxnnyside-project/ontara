use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use ontara_core::{AssetFile, Entity};
use sha2::{Digest, Sha256};

use crate::error::StorageError;

/// The result of bringing a file into a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedAsset {
    /// The entity that stands for the file in the universe.
    pub entity: Entity,
    pub file: AssetFile,
    /// The same content was already in the project, so nothing was copied.
    pub already_imported: bool,
}

/// How much of a file name is kept; longer names are cut before the extension.
const MAX_STEM_LENGTH: usize = 80;

/// A file name that is safe on every platform and inside a project-relative path.
pub(crate) fn safe_file_name(original: &str) -> String {
    let path = Path::new(original);
    let clean = |text: &str| -> String {
        let mapped: String = text
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        mapped.trim_matches('-').to_owned()
    };
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(clean)
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "asset".to_owned());
    let stem: String = stem.chars().take(MAX_STEM_LENGTH).collect();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| clean(extension).to_lowercase())
        .filter(|extension| !extension.is_empty());
    match extension {
        Some(extension) => format!("{stem}.{extension}"),
        None => stem,
    }
}

/// The media type implied by a file name, for the formats an author typically attaches.
pub(crate) fn media_type_of(file_name: &str) -> Option<&'static str> {
    let extension = Path::new(file_name).extension()?.to_str()?.to_lowercase();
    Some(match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => return None,
    })
}

/// The SHA-256 of a file's content, as lowercase hexadecimal.
pub(crate) fn content_hash(path: &Path) -> Result<String, StorageError> {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";

    let mut file = File::open(path).map_err(|source| StorageError::io(path, source))?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).map_err(|source| StorageError::io(path, source))?;

    let mut hex = String::with_capacity(64);
    for byte in hasher.finalize() {
        hex.push(char::from(DIGITS[usize::from(byte >> 4)]));
        hex.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    Ok(hex)
}

/// Copies `source` into `assets_directory` under a name that is not taken yet, and returns the
/// project-relative path (`assets/<name>`) together with the absolute destination.
///
/// The copy goes to a temporary name first and is renamed, so an interrupted import never leaves
/// a half-written asset under its final name.
pub(crate) fn copy_into(
    source: &Path,
    assets_directory: &Path,
    root: &Path,
) -> Result<(String, PathBuf), StorageError> {
    let original = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    let wanted = safe_file_name(original);
    let (stem, extension) = match wanted.rsplit_once('.') {
        Some((stem, extension)) => (stem.to_owned(), format!(".{extension}")),
        None => (wanted.clone(), String::new()),
    };

    let mut attempt = 1_u32;
    let destination = loop {
        let name = if attempt == 1 {
            wanted.clone()
        } else {
            format!("{stem}-{attempt}{extension}")
        };
        let candidate = assets_directory.join(&name);
        if !candidate.exists() {
            break candidate;
        }
        attempt += 1;
    };

    let partial = assets_directory.join(format!(".import-{}.part", uuid::Uuid::now_v7()));
    fs::copy(source, &partial).map_err(|source_error| StorageError::io(source, source_error))?;
    fs::rename(&partial, &destination).map_err(|error| {
        let _ = fs::remove_file(&partial);
        StorageError::io(&destination, error)
    })?;

    let relative = destination
        .strip_prefix(root)
        .map_err(|_| {
            StorageError::io(
                &destination,
                io::Error::other("the asset was written outside the project"),
            )
        })?
        .to_string_lossy()
        .replace('\\', "/");
    Ok((relative, destination))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_made_safe_and_keep_their_extension() {
        assert_eq!(
            safe_file_name("Ayla Portrait (final).PNG"),
            "Ayla-Portrait--final.png"
        );
        assert_eq!(safe_file_name("../../etc/passwd"), "passwd");
        assert_eq!(safe_file_name("???.jpg"), "asset.jpg");
        assert_eq!(safe_file_name("notes"), "notes");
    }

    #[test]
    fn long_names_are_cut_before_the_extension() {
        let name = safe_file_name(&format!("{}.pdf", "a".repeat(300)));

        assert_eq!(name.len(), MAX_STEM_LENGTH + ".pdf".len());
        assert_eq!(
            Path::new(&name).extension().and_then(|e| e.to_str()),
            Some("pdf")
        );
    }

    #[test]
    fn media_types_are_known_for_common_formats_only() {
        assert_eq!(media_type_of("map.webp"), Some("image/webp"));
        assert_eq!(media_type_of("map.JPEG"), Some("image/jpeg"));
        assert_eq!(media_type_of("map.xyz"), None);
        assert_eq!(media_type_of("map"), None);
    }
}
