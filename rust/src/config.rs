// Configuration module
// Load and validate configuration from environment variables

use crate::error::ConfigError;

/// Main configuration struct containing all subsystem configurations
#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub queue_manager: QueueManagerConfig,
    pub log_parser: LogParserConfig,
    pub logging: LoggingConfig,
}

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
    pub ssl_ca_path: Option<String>,
}

/// Queue manager configuration
#[derive(Debug, Clone)]
pub struct QueueManagerConfig {
    pub poll_interval_ms: u64,
    pub starting_id: i32,
}

/// Log parser configuration
#[derive(Debug, Clone)]
pub struct LogParserConfig {
    pub poll_interval_ms: u64,
    pub rule_refresh_interval: usize, // Refresh rules every N logs
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub error_log_path: String,
    pub debug_log_path: String,
    pub max_log_size_bytes: u64,
    pub max_log_backups: usize,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            username: String::new(),
            password: String::new(),
            database: String::new(),
            pool_size: 10,
            ssl_ca_path: None,
        }
    }
}

impl Default for QueueManagerConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            starting_id: 0,
        }
    }
}

impl Default for LogParserConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            rule_refresh_interval: 100,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            error_log_path: String::from("logs/error.log"),
            debug_log_path: String::from("logs/debug.log"),
            max_log_size_bytes: 10_485_760, // 10MB
            max_log_backups: 5,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    /// Returns error if required variables are missing or invalid
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present (for local development)
        // Skip loading .env during tests to allow test isolation
        #[cfg(not(test))]
        let _ = dotenvy::dotenv();
        
        // Load required database configuration
        let database = DatabaseConfig {
            host: get_required_env("DB_HOST")?,
            username: get_required_env("DB_USERNAME")?,
            password: get_required_env("DB_PASSWORD")?,
            database: get_required_env("DB_NAME")?,
            pool_size: get_optional_env("DB_POOL_SIZE")?
                .unwrap_or(DatabaseConfig::default().pool_size),
            ssl_ca_path: std::env::var("DB_SSL_CA_PATH").ok(),
        };
        
        // Load optional queue manager configuration
        let queue_manager = QueueManagerConfig {
            poll_interval_ms: get_optional_env("QUEUE_MANAGER_POLL_INTERVAL_MS")?
                .unwrap_or(QueueManagerConfig::default().poll_interval_ms),
            starting_id: get_optional_env("QUEUE_MANAGER_STARTING_ID")?
                .unwrap_or(QueueManagerConfig::default().starting_id),
        };
        
        // Load optional log parser configuration
        let log_parser = LogParserConfig {
            poll_interval_ms: get_optional_env("LOG_PARSER_POLL_INTERVAL_MS")?
                .unwrap_or(LogParserConfig::default().poll_interval_ms),
            rule_refresh_interval: get_optional_env("LOG_PARSER_RULE_REFRESH_INTERVAL")?
                .unwrap_or(LogParserConfig::default().rule_refresh_interval),
        };
        
        // Load optional logging configuration
        let logging = LoggingConfig {
            error_log_path: std::env::var("ERROR_LOG_PATH")
                .unwrap_or_else(|_| LoggingConfig::default().error_log_path),
            debug_log_path: std::env::var("DEBUG_LOG_PATH")
                .unwrap_or_else(|_| LoggingConfig::default().debug_log_path),
            max_log_size_bytes: get_optional_env("MAX_LOG_SIZE_BYTES")?
                .unwrap_or(LoggingConfig::default().max_log_size_bytes),
            max_log_backups: get_optional_env("MAX_LOG_BACKUPS")?
                .unwrap_or(LoggingConfig::default().max_log_backups),
        };
        
        Ok(Config {
            database,
            queue_manager,
            log_parser,
            logging,
        })
    }
}

/// Helper function to get a required environment variable
fn get_required_env(key: &str) -> Result<String, ConfigError> {
    std::env::var(key).map_err(|_| ConfigError::MissingEnvVar(key.to_string()))
}

/// Helper function to get an optional environment variable and parse it
fn get_optional_env<T: std::str::FromStr>(key: &str) -> Result<Option<T>, ConfigError> {
    match std::env::var(key) {
        Ok(val) => val.parse::<T>().map(Some).map_err(|_| ConfigError::InvalidValue {
            key: key.to_string(),
            reason: format!("Failed to parse value: {}", val),
        }),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use serial_test::serial;

    // Helper to clear all environment variables used by config
    fn clear_env_vars() {
        env::remove_var("DB_HOST");
        env::remove_var("DB_USERNAME");
        env::remove_var("DB_PASSWORD");
        env::remove_var("DB_NAME");
        env::remove_var("DB_POOL_SIZE");
        env::remove_var("DB_SSL_CA_PATH");
        env::remove_var("QUEUE_MANAGER_POLL_INTERVAL_MS");
        env::remove_var("QUEUE_MANAGER_STARTING_ID");
        env::remove_var("LOG_PARSER_POLL_INTERVAL_MS");
        env::remove_var("LOG_PARSER_RULE_REFRESH_INTERVAL");
        env::remove_var("ERROR_LOG_PATH");
        env::remove_var("DEBUG_LOG_PATH");
        env::remove_var("MAX_LOG_SIZE_BYTES");
        env::remove_var("MAX_LOG_BACKUPS");
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_required_vars() {
        clear_env_vars();
        
        // Set required environment variables
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_USERNAME", "testuser");
        env::set_var("DB_PASSWORD", "testpass");
        env::set_var("DB_NAME", "testdb");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.username, "testuser");
        assert_eq!(config.database.password, "testpass");
        assert_eq!(config.database.database, "testdb");
        assert_eq!(config.database.pool_size, 10); // Default value

        clear_env_vars();
    }

    #[test]
    #[serial]
    fn test_config_from_env_missing_required_var() {
        clear_env_vars();

        let result = Config::from_env();
        assert!(result.is_err(), "Expected error for missing DB_HOST");
        
        if let Err(ConfigError::MissingEnvVar(var)) = result {
            assert_eq!(var, "DB_HOST");
        } else {
            panic!("Expected MissingEnvVar error, got: {:?}", result);
        }
        
        clear_env_vars();
    }

    #[test]
    #[serial]
    fn test_config_from_env_with_optional_vars() {
        clear_env_vars();
        
        // Set required environment variables
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_USERNAME", "testuser");
        env::set_var("DB_PASSWORD", "testpass");
        env::set_var("DB_NAME", "testdb");
        
        // Set optional variables
        env::set_var("DB_POOL_SIZE", "20");
        env::set_var("QUEUE_MANAGER_POLL_INTERVAL_MS", "2000");
        env::set_var("LOG_PARSER_RULE_REFRESH_INTERVAL", "200");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.database.pool_size, 20);
        assert_eq!(config.queue_manager.poll_interval_ms, 2000);
        assert_eq!(config.log_parser.rule_refresh_interval, 200);

        clear_env_vars();
    }

    #[test]
    #[serial]
    fn test_config_from_env_invalid_optional_value() {
        clear_env_vars();
        
        // Set required environment variables
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_USERNAME", "testuser");
        env::set_var("DB_PASSWORD", "testpass");
        env::set_var("DB_NAME", "testdb");
        
        // Set invalid optional variable
        env::set_var("DB_POOL_SIZE", "not_a_number");

        let result = Config::from_env();
        assert!(result.is_err(), "Expected error for invalid DB_POOL_SIZE");
        
        if let Err(ConfigError::InvalidValue { key, .. }) = result {
            assert_eq!(key, "DB_POOL_SIZE");
        } else {
            panic!("Expected InvalidValue error, got: {:?}", result);
        }

        clear_env_vars();
    }
}
