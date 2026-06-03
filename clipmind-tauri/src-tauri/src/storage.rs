use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use keyring::Entry;

const SERVICE: &str = "com.buffbrain.app";
const API_KEY_USER: &str = "openrouter-api-key";

pub struct SecureStore {
    service: String,
}

impl SecureStore {
    pub fn new() -> Self {
        Self {
            service: SERVICE.to_string(),
        }
    }

    pub fn set_api_key(&self, key: &str) -> Result<()> {
        let entry = Entry::new(&self.service, API_KEY_USER)
            .context("Failed to access keychain")?;
        let encoded = general_purpose::STANDARD.encode(key.as_bytes());
        entry
            .set_password(&encoded)
            .context("Failed to write to keychain")?;
        Ok(())
    }

    pub fn get_api_key(&self) -> Result<Option<String>> {
        let entry = Entry::new(&self.service, API_KEY_USER)
            .context("Failed to access keychain")?;
        match entry.get_password() {
            Ok(encoded) => {
                let bytes = general_purpose::STANDARD
                    .decode(&encoded)
                    .context("Stored key is corrupt")?;
                let s = String::from_utf8(bytes).context("Stored key is not valid UTF-8")?;
                Ok(Some(s))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Keychain read failed: {e}")),
        }
    }

    pub fn delete_api_key(&self) -> Result<()> {
        let entry = Entry::new(&self.service, API_KEY_USER)
            .context("Failed to access keychain")?;
        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Keychain delete failed: {e}")),
        }
    }
}
