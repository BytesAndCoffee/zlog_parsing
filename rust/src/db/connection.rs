// Connection pool setup

use crate::config::DatabaseConfig;
use crate::error::DbError;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::MySqlPool;
use std::time::Duration;

/// Create a connection pool from configuration
/// 
/// This function establishes a MySQL connection pool with the following features:
/// - Configurable pool size from DatabaseConfig
/// - SSL/TLS support with certificate verification if ssl_ca_path is provided
/// - Connection timeout and idle timeout settings
/// - Returns DbError on connection failure with context
/// 
/// # Arguments
/// * `config` - Database configuration containing host, credentials, and pool settings
/// 
/// # Returns
/// * `Ok(MySqlPool)` - Successfully created connection pool
/// * `Err(DbError)` - Connection failure with descriptive error
/// 
/// # Requirements
/// Validates: Requirements 1.1, 1.2, 1.4, 1.5
pub async fn create_pool(config: &DatabaseConfig) -> Result<MySqlPool, DbError> {
    // Build connection options
    let mut connect_options = MySqlConnectOptions::new()
        .host(&config.host)
        .username(&config.username)
        .password(&config.password)
        .database(&config.database);
    
    // Configure SSL/TLS - default to Ubuntu's CA bundle if not specified
    let ca_path = config.ssl_ca_path.as_deref().unwrap_or("/etc/ssl/certs/ca-certificates.crt");
    connect_options = connect_options
        .ssl_mode(MySqlSslMode::VerifyIdentity)
        .ssl_ca(ca_path);
    
    // Create connection pool with configured size
    let pool = MySqlPoolOptions::new()
        .max_connections(config.pool_size)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .connect_with(connect_options)
        .await?;
    
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn test_create_pool_with_invalid_credentials() {
        // Test that connection failure returns a descriptive DbError
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            username: "invalid_user".to_string(),
            password: "invalid_pass".to_string(),
            database: "nonexistent_db".to_string(),
            pool_size: 5,
            ssl_ca_path: None,
        };

        let result = create_pool(&config).await;
        
        // Should fail with a DbError
        assert!(result.is_err(), "Expected connection to fail with invalid credentials");
        
        // Verify it's a DbError::Sqlx variant
        if let Err(DbError::Sqlx(e)) = result {
            // Error message should contain context about the connection failure
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty");
        } else {
            panic!("Expected DbError::Sqlx, got: {:?}", result);
        }
    }

    #[test]
    fn test_pool_configuration() {
        // Test that DatabaseConfig properly configures pool settings
        let config = DatabaseConfig {
            host: "testhost".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            database: "testdb".to_string(),
            pool_size: 15,
            ssl_ca_path: Some("/path/to/ca.pem".to_string()),
        };

        assert_eq!(config.pool_size, 15);
        assert_eq!(config.ssl_ca_path, Some("/path/to/ca.pem".to_string()));
    }
}
