use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub environment: Environment,
    pub database: DatabaseConfig,
    pub engine: EngineConfig,
    pub ipc: IpcConfig,
    pub api: ApiConfig,
    pub features: FeaturesConfig,
    pub paths: PathsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    CI,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub max_memory: String,
    pub threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub tick_buffer_size: usize,
    pub max_parallel_algorithms: usize,
    pub python_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfig {
    pub port: u16,
    pub max_message_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub update_server_url: String,
    pub telemetry_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub hot_reload: bool,
    pub debug_mode: bool,
    pub profiling_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub algorithm_path: PathBuf,
    pub data_path: PathBuf,
    pub cache_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load .env file based on NODE_ENV
        let env_file = match env::var("NODE_ENV").as_deref() {
            Ok("production") => ".env.production",
            Ok("ci") => ".env.ci",
            _ => ".env.development",
        };

        // Load environment variables from file
        dotenv::from_filename(env_file).ok();

        // Parse configuration
        let config = Config {
            environment: Self::parse_environment()?,
            database: Self::parse_database_config()?,
            engine: Self::parse_engine_config()?,
            ipc: Self::parse_ipc_config()?,
            api: Self::parse_api_config()?,
            features: Self::parse_features_config()?,
            paths: Self::parse_paths_config()?,
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn parse_environment() -> Result<Environment> {
        match env::var("NODE_ENV").as_deref() {
            Ok("production") => Ok(Environment::Production),
            Ok("ci") => Ok(Environment::CI),
            _ => Ok(Environment::Development),
        }
    }

    fn parse_database_config() -> Result<DatabaseConfig> {
        Ok(DatabaseConfig {
            path: PathBuf::from(
                env::var("DB_PATH").unwrap_or_else(|_| "./data/dev.duckdb".to_string()),
            ),
            max_memory: env::var("DB_MAX_MEMORY").unwrap_or_else(|_| "4GB".to_string()),
            threads: env::var("DB_THREADS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .context("Invalid DB_THREADS")?,
        })
    }

    fn parse_engine_config() -> Result<EngineConfig> {
        Ok(EngineConfig {
            tick_buffer_size: env::var("ENGINE_TICK_BUFFER_SIZE")
                .unwrap_or_else(|_| "100000".to_string())
                .parse()
                .context("Invalid ENGINE_TICK_BUFFER_SIZE")?,
            max_parallel_algorithms: env::var("ENGINE_MAX_PARALLEL_ALGORITHMS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .context("Invalid ENGINE_MAX_PARALLEL_ALGORITHMS")?,
            python_threads: env::var("ENGINE_PYTHON_THREADS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .context("Invalid ENGINE_PYTHON_THREADS")?,
        })
    }

    fn parse_ipc_config() -> Result<IpcConfig> {
        Ok(IpcConfig {
            port: env::var("IPC_PORT")
                .unwrap_or_else(|_| "7878".to_string())
                .parse()
                .context("Invalid IPC_PORT")?,
            max_message_size: env::var("IPC_MAX_MESSAGE_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .context("Invalid IPC_MAX_MESSAGE_SIZE")?,
        })
    }

    fn parse_api_config() -> Result<ApiConfig> {
        Ok(ApiConfig {
            update_server_url: env::var("UPDATE_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            telemetry_enabled: env::var("TELEMETRY_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }

    fn parse_features_config() -> Result<FeaturesConfig> {
        Ok(FeaturesConfig {
            hot_reload: env::var("HOT_RELOAD")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            debug_mode: env::var("DEBUG_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            profiling_enabled: env::var("PROFILING_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }

    fn parse_paths_config() -> Result<PathsConfig> {
        Ok(PathsConfig {
            algorithm_path: PathBuf::from(
                env::var("ALGORITHM_PATH").unwrap_or_else(|_| "./algorithms".to_string()),
            ),
            data_path: PathBuf::from(
                env::var("DATA_PATH").unwrap_or_else(|_| "./data".to_string()),
            ),
            cache_path: PathBuf::from(
                env::var("CACHE_PATH").unwrap_or_else(|_| "./data/cache".to_string()),
            ),
        })
    }

    fn validate(&self) -> Result<()> {
        // Validate port range
        if self.ipc.port < 1024 {
            anyhow::bail!("IPC port must be >= 1024");
        }

        // Validate buffer sizes
        if self.engine.tick_buffer_size == 0 {
            anyhow::bail!("Tick buffer size must be > 0");
        }

        // Validate thread counts
        if self.engine.max_parallel_algorithms == 0 {
            anyhow::bail!("Max parallel algorithms must be > 0");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        // Test configuration loading and validation
        std::env::set_var("NODE_ENV", "development");
        std::env::set_var("IPC_PORT", "8080");
        std::env::set_var("ENGINE_TICK_BUFFER_SIZE", "1000");

        let config = Config::load();
        assert!(config.is_ok());
    }
}
