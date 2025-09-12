#![allow(dead_code)] // Will be used in Epic 2

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Credential storage trait for different backends
pub trait CredentialStore {
    fn get(&self, key: &str) -> Result<Option<String>>;
    fn set(&self, key: &str, value: &str) -> Result<()>;
    fn delete(&self, key: &str) -> Result<()>;
    fn list(&self) -> Result<Vec<String>>;
}

/// Credentials manager
pub struct CredentialsManager {
    store: Box<dyn CredentialStore>,
    cache: HashMap<String, String>,
}

impl CredentialsManager {
    pub fn new() -> Result<Self> {
        let store = Self::create_store()?;
        Ok(Self {
            store,
            cache: HashMap::new(),
        })
    }

    fn create_store() -> Result<Box<dyn CredentialStore>> {
        let store_type = env::var("CREDENTIAL_STORE").unwrap_or_else(|_| "env".to_string());

        match store_type.as_str() {
            "env" => Ok(Box::new(EnvironmentStore::new())),
            "windows_credential_manager" => Ok(Box::new(WindowsCredentialStore::new())),
            _ => Ok(Box::new(EnvironmentStore::new())),
        }
    }

    pub fn get_credential(&mut self, key: &str) -> Result<Option<String>> {
        // Check cache first
        if let Some(value) = self.cache.get(key) {
            return Ok(Some(value.clone()));
        }

        // Get from store
        if let Some(value) = self.store.get(key)? {
            self.cache.insert(key.to_string(), value.clone());
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn set_credential(&mut self, key: &str, value: &str) -> Result<()> {
        self.validate_credential(key, value)?;
        self.store.set(key, value)?;
        self.cache.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn delete_credential(&mut self, key: &str) -> Result<()> {
        self.store.delete(key)?;
        self.cache.remove(key);
        Ok(())
    }

    fn validate_credential(&self, key: &str, value: &str) -> Result<()> {
        if key.is_empty() {
            anyhow::bail!("Credential key cannot be empty");
        }
        if value.is_empty() {
            anyhow::bail!("Credential value cannot be empty");
        }
        if key.len() > 256 {
            anyhow::bail!("Credential key too long (max 256 characters)");
        }
        Ok(())
    }
}

/// Environment variable credential store
pub struct EnvironmentStore;

impl EnvironmentStore {
    pub fn new() -> Self {
        Self
    }
}

impl CredentialStore for EnvironmentStore {
    fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(env::var(key).ok())
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        env::set_var(key, value);
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<()> {
        env::remove_var(key);
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>> {
        Ok(env::vars().map(|(k, _)| k).collect())
    }
}

/// Windows Credential Manager store (stub implementation)
pub struct WindowsCredentialStore;

impl WindowsCredentialStore {
    pub fn new() -> Self {
        Self
    }
}

impl CredentialStore for WindowsCredentialStore {
    fn get(&self, key: &str) -> Result<Option<String>> {
        // TODO: Implement Windows Credential Manager integration
        // For now, fall back to environment variables
        Ok(env::var(key).ok())
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        // TODO: Implement Windows Credential Manager integration
        // For now, fall back to environment variables
        env::set_var(key, value);
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<()> {
        // TODO: Implement Windows Credential Manager integration
        // For now, fall back to environment variables
        env::remove_var(key);
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>> {
        // TODO: Implement Windows Credential Manager integration
        Ok(vec![])
    }
}

/// Credential types used by the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProviderCredentials {
    pub api_key: String,
    pub endpoint: String,
}

/// Helper functions for common credentials
impl CredentialsManager {
    pub fn get_broker_credentials(&mut self) -> Result<Option<BrokerCredentials>> {
        let api_key = self.get_credential("BROKER_API_KEY")?;
        let api_secret = self.get_credential("BROKER_API_SECRET")?;
        let account_id = self.get_credential("BROKER_ACCOUNT_ID")?;

        match (api_key, api_secret) {
            (Some(key), Some(secret)) => Ok(Some(BrokerCredentials {
                api_key: key,
                api_secret: secret,
                account_id,
            })),
            _ => Ok(None),
        }
    }

    pub fn get_data_provider_credentials(&mut self) -> Result<Option<DataProviderCredentials>> {
        let api_key = self.get_credential("DATA_PROVIDER_API_KEY")?;
        let endpoint = self.get_credential("DATA_PROVIDER_ENDPOINT")?;

        match (api_key, endpoint) {
            (Some(key), Some(ep)) => Ok(Some(DataProviderCredentials {
                api_key: key,
                endpoint: ep,
            })),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_store() {
        let store = EnvironmentStore::new();

        // Test set and get
        store.set("TEST_KEY", "test_value").unwrap();
        assert_eq!(
            store.get("TEST_KEY").unwrap(),
            Some("test_value".to_string())
        );

        // Test delete
        store.delete("TEST_KEY").unwrap();
        assert_eq!(store.get("TEST_KEY").unwrap(), None);
    }

    #[test]
    fn test_credentials_manager() {
        let mut manager = CredentialsManager::new().unwrap();

        // Test credential validation
        assert!(manager.set_credential("", "value").is_err());
        assert!(manager.set_credential("key", "").is_err());

        // Test normal operations
        manager.set_credential("TEST_CRED", "secret").unwrap();
        assert_eq!(
            manager.get_credential("TEST_CRED").unwrap(),
            Some("secret".to_string())
        );

        // Test caching
        let cached = manager.get_credential("TEST_CRED").unwrap();
        assert_eq!(cached, Some("secret".to_string()));

        // Cleanup
        manager.delete_credential("TEST_CRED").unwrap();
    }
}
