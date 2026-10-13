//! Where provider keys live: the operating system's credential store, never a project file, a
//! backup or an export, and never the webview.

#[cfg(test)]
use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::Mutex;

const SERVICE: &str = "com.sxnnyside.ontara";

/// Keeps one secret per provider.
pub trait SecretStore: Send + Sync {
    fn get(&self, provider: &str) -> Result<Option<String>, String>;
    fn set(&self, provider: &str, secret: &str) -> Result<(), String>;
    fn delete(&self, provider: &str) -> Result<(), String>;
}

/// The credential store of the operating system.
pub struct SystemKeychain;

impl SystemKeychain {
    fn entry(provider: &str) -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE, &format!("provider:{provider}")).map_err(|e| e.to_string())
    }
}

impl SecretStore for SystemKeychain {
    fn get(&self, provider: &str) -> Result<Option<String>, String> {
        match Self::entry(provider)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn set(&self, provider: &str, secret: &str) -> Result<(), String> {
        Self::entry(provider)?
            .set_password(secret)
            .map_err(|e| e.to_string())
    }

    fn delete(&self, provider: &str) -> Result<(), String> {
        match Self::entry(provider)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

/// Secrets held in memory, for tests.
#[cfg(test)]
#[derive(Default)]
pub struct MemorySecrets(Mutex<BTreeMap<String, String>>);

#[cfg(test)]
impl SecretStore for MemorySecrets {
    fn get(&self, provider: &str) -> Result<Option<String>, String> {
        Ok(self
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .get(provider)
            .cloned())
    }

    fn set(&self, provider: &str, secret: &str) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .insert(provider.to_owned(), secret.to_owned());
        Ok(())
    }

    fn delete(&self, provider: &str) -> Result<(), String> {
        self.0.lock().map_err(|e| e.to_string())?.remove(provider);
        Ok(())
    }
}
