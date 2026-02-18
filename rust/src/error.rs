// Error types and conversions

use thiserror::Error;

/// Top-level application error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    
    #[error("Queue manager error: {0}")]
    QueueManager(#[from] QueueManagerError),
    
    #[error("Log parser error: {0}")]
    LogParser(#[from] LogParserError),
}

/// Configuration loading and validation errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    
    #[error("Invalid configuration value for {key}: {reason}")]
    InvalidValue { key: String, reason: String },
}

/// Database connectivity and operation errors
#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQL error: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("Schema validation failed: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Schema and rule validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required column: {0}")]
    MissingColumn(String),
    
    #[error("Type mismatch for column {column}: expected {expected}")]
    TypeMismatch { column: String, expected: String },
    
    #[error("Null value for non-nullable column: {0}")]
    NullValue(String),
    
    #[error("Invalid rule: {0}")]
    InvalidRule(String),
}

/// Queue management specific errors
#[derive(Error, Debug)]
pub enum QueueManagerError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),
}

/// Log parsing specific errors
#[derive(Error, Debug)]
pub enum LogParserError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    
    #[error("Rule validation error: {0}")]
    RuleValidation(#[from] ValidationError),
}
