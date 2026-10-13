//! The providers an author configured: where to call, which model, and whether a key is held.
//! The key itself is in the credential store and is never read back out to the interface.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::secrets::SecretStore;
use crate::workspace::CommandError;

const FILE_NAME: &str = "ai-providers.json";

/// Which wire protocol a provider speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// Anthropic's Messages API.
    Anthropic,
    /// The chat completions shape `OpenAI` and most local servers (Ollama, LM Studio) share.
    OpenAiCompatible,
}

/// What is stored about a provider, without its key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub kind: ProviderKind,
    /// Where to call, without a trailing slash: `https://api.anthropic.com`,
    /// `http://localhost:11434/v1`.
    pub base_url: String,
    pub model: String,
}

impl Provider {
    /// Whether the provider runs on this machine, so nothing leaves it.
    #[must_use]
    pub fn is_local(&self) -> bool {
        host_of(&self.base_url).is_some_and(|host| {
            matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]" | "::1")
        })
    }
}

/// The host part of an `http(s)` URL, lowercase.
fn host_of(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let without_user = authority.rsplit('@').next()?;
    let host = if without_user.starts_with('[') {
        without_user.split(']').next().map(|h| format!("{h}]"))?
    } else {
        without_user.split(':').next()?.to_owned()
    };
    (!host.is_empty()).then(|| host.to_lowercase())
}

/// Checks where a provider may be called: HTTPS, or HTTP only on this machine.
pub fn validate_url(url: &str) -> Result<(), CommandError> {
    let Some(host) = host_of(url) else {
        return Err(CommandError::new(
            "rejected",
            "The address must start with https:// (or http:// for a server on this computer).",
        ));
    };
    let local = matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]");
    if url.starts_with("http://") && !local {
        return Err(CommandError::new(
            "rejected",
            "A provider outside this computer must be called over https://.",
        ));
    }
    Ok(())
}

/// A provider as the interface sees it: never with its key.
///
/// Mirrored by `src/features/assistant/api/assistant-api.ts`.
#[derive(Debug, Serialize)]
pub struct ProviderView {
    #[serde(flatten)]
    pub provider: Provider,
    pub has_key: bool,
    pub is_local: bool,
}

/// The providers file, in the application's own configuration folder.
pub struct ProviderStore {
    path: PathBuf,
}

impl ProviderStore {
    #[must_use]
    pub fn in_directory(directory: &Path) -> Self {
        Self {
            path: directory.join(FILE_NAME),
        }
    }

    pub fn list(&self) -> Result<Vec<Provider>, CommandError> {
        match fs::read_to_string(&self.path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|error| CommandError::new("storage", format!("providers file: {error}"))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(CommandError::new("storage", error.to_string())),
        }
    }

    pub fn save_all(&self, providers: &[Provider]) -> Result<(), CommandError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| CommandError::new("storage", error.to_string()))?;
        }
        let text = serde_json::to_string_pretty(providers)
            .map_err(|error| CommandError::new("storage", error.to_string()))?;
        fs::write(&self.path, text).map_err(|error| CommandError::new("storage", error.to_string()))
    }

    pub fn view(&self, secrets: &dyn SecretStore) -> Result<Vec<ProviderView>, CommandError> {
        self.list()?
            .into_iter()
            .map(|provider| {
                let has_key = secrets
                    .get(&provider.id)
                    .map_err(|e| CommandError::new("storage", e))?
                    .is_some();
                let is_local = provider.is_local();
                Ok(ProviderView {
                    provider,
                    has_key,
                    is_local,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(url: &str) -> Provider {
        Provider {
            id: "p".into(),
            name: "P".into(),
            kind: ProviderKind::OpenAiCompatible,
            base_url: url.into(),
            model: "m".into(),
        }
    }

    #[test]
    fn only_this_machine_counts_as_local() {
        assert!(provider("http://localhost:11434/v1").is_local());
        assert!(provider("http://127.0.0.1:1234/v1").is_local());
        assert!(provider("http://[::1]:8080").is_local());
        assert!(!provider("https://api.anthropic.com").is_local());
        // A host that merely starts like a local one is not local.
        assert!(!provider("https://localhost.evil.example/v1").is_local());
        assert!(!provider("https://user@localhost.evil.example/").is_local());
        assert!(!provider("not a url").is_local());
    }

    #[test]
    fn plain_http_is_only_accepted_on_this_machine() {
        assert!(validate_url("https://api.openai.com/v1").is_ok());
        assert!(validate_url("http://localhost:11434/v1").is_ok());
        assert!(validate_url("http://api.openai.com/v1").is_err());
        assert!(validate_url("ftp://localhost").is_err());
        assert!(validate_url("https://").is_err());
    }
}
