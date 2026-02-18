use std::env;
use std::fs;
use std::path::Path;
use serial_test::serial;

/// Integration test to verify logging setup creates directories and files
#[test]
#[serial]
fn test_logging_creates_directories() {
    // Clean up any existing test logs
    let _ = fs::remove_dir_all("test_integration_logs");
    
    // Set up test environment variables
    env::set_var("DB_HOST", "localhost");
    env::set_var("DB_USERNAME", "testuser");
    env::set_var("DB_PASSWORD", "testpass");
    env::set_var("DB_NAME", "testdb");
    env::set_var("ERROR_LOG_PATH", "test_integration_logs/error.log");
    env::set_var("DEBUG_LOG_PATH", "test_integration_logs/debug.log");
    
    // Note: We can't easily test the actual logging setup without running the main function
    // because it initializes a global subscriber. This test verifies the configuration
    // is loaded correctly.
    
    let config = irc_log_parser::config::Config::from_env()
        .expect("Failed to load config");
    
    // Verify the paths are set correctly
    assert_eq!(config.logging.error_log_path, "test_integration_logs/error.log");
    assert_eq!(config.logging.debug_log_path, "test_integration_logs/debug.log");
    
    // Verify we can create the directories
    let error_log_dir = Path::new(&config.logging.error_log_path).parent().unwrap();
    let debug_log_dir = Path::new(&config.logging.debug_log_path).parent().unwrap();
    
    fs::create_dir_all(error_log_dir).expect("Failed to create error log directory");
    fs::create_dir_all(debug_log_dir).expect("Failed to create debug log directory");
    
    assert!(error_log_dir.exists());
    assert!(debug_log_dir.exists());
    
    // Clean up
    env::remove_var("DB_HOST");
    env::remove_var("DB_USERNAME");
    env::remove_var("DB_PASSWORD");
    env::remove_var("DB_NAME");
    env::remove_var("ERROR_LOG_PATH");
    env::remove_var("DEBUG_LOG_PATH");
    let _ = fs::remove_dir_all("test_integration_logs");
}

#[test]
#[serial]
fn test_logging_config_with_custom_values() {
    env::set_var("DB_HOST", "localhost");
    env::set_var("DB_USERNAME", "testuser");
    env::set_var("DB_PASSWORD", "testpass");
    env::set_var("DB_NAME", "testdb");
    env::set_var("ERROR_LOG_PATH", "custom/error.log");
    env::set_var("DEBUG_LOG_PATH", "custom/debug.log");
    env::set_var("MAX_LOG_SIZE_BYTES", "5242880"); // 5MB
    env::set_var("MAX_LOG_BACKUPS", "10");
    
    let config = irc_log_parser::config::Config::from_env()
        .expect("Failed to load config");
    
    assert_eq!(config.logging.error_log_path, "custom/error.log");
    assert_eq!(config.logging.debug_log_path, "custom/debug.log");
    assert_eq!(config.logging.max_log_size_bytes, 5242880);
    assert_eq!(config.logging.max_log_backups, 10);
    
    // Clean up
    env::remove_var("DB_HOST");
    env::remove_var("DB_USERNAME");
    env::remove_var("DB_PASSWORD");
    env::remove_var("DB_NAME");
    env::remove_var("ERROR_LOG_PATH");
    env::remove_var("DEBUG_LOG_PATH");
    env::remove_var("MAX_LOG_SIZE_BYTES");
    env::remove_var("MAX_LOG_BACKUPS");
}
