use irc_log_parser::config::{Config, LoggingConfig};
use std::env;
use std::fs;
use std::path::Path;

#[test]
fn test_logging_setup() {
    // Set up test environment variables
    env::set_var("DB_HOST", "localhost");
    env::set_var("DB_USERNAME", "testuser");
    env::set_var("DB_PASSWORD", "testpass");
    env::set_var("DB_NAME", "testdb");
    env::set_var("ERROR_LOG_PATH", "test_logs/error.log");
    env::set_var("DEBUG_LOG_PATH", "test_logs/debug.log");
    
    // Load config
    let config = Config::from_env().expect("Failed to load config");
    
    // Verify logging config
    assert_eq!(config.logging.error_log_path, "test_logs/error.log");
    assert_eq!(config.logging.debug_log_path, "test_logs/debug.log");
    
    // Clean up test environment
    env::remove_var("DB_HOST");
    env::remove_var("DB_USERNAME");
    env::remove_var("DB_PASSWORD");
    env::remove_var("DB_NAME");
    env::remove_var("ERROR_LOG_PATH");
    env::remove_var("DEBUG_LOG_PATH");
    
    // Clean up test logs directory if it exists
    let _ = fs::remove_dir_all("test_logs");
}

#[test]
fn test_logging_config_defaults() {
    let logging_config = LoggingConfig::default();
    
    assert_eq!(logging_config.error_log_path, "logs/error.log");
    assert_eq!(logging_config.debug_log_path, "logs/debug.log");
    assert_eq!(logging_config.max_log_size_bytes, 10_485_760); // 10MB
    assert_eq!(logging_config.max_log_backups, 5);
}
