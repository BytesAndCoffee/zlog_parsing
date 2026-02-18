# Design Document: Python to Rust IRC Log Parser Translation

## Overview

This design translates a Python-based IRC log parsing system to idiomatic Rust while preserving all functional behavior. The system consists of two main processes: a Queue Manager that moves logs from the source table to a processing queue, and a Log Parser that evaluates logs against user-defined rules and routes matches to recipients.

The Rust implementation leverages:
- **sqlx** for async MySQL operations with compile-time query checking
- **tokio** for the async runtime and task management
- **serde/serde_json** for type-safe JSON handling
- **tracing** for structured logging
- **thiserror** for ergonomic error handling
- **dotenvy** for environment configuration

The design emphasizes type safety, zero-cost abstractions, and idiomatic Rust patterns while maintaining functional equivalence with the Python implementation.

## Architecture

### High-Level Structure

```
┌─────────────────────────────────────────────────────────────┐
│                         Main Process                         │
│  ┌────────────────┐              ┌─────────────────────┐   │
│  │ Queue Manager  │              │    Log Parser       │   │
│  │   (Task 1)     │              │     (Task 2)        │   │
│  └────────┬───────┘              └──────────┬──────────┘   │
│           │                                  │               │
│           └──────────┬───────────────────────┘               │
│                      │                                       │
│              ┌───────▼────────┐                             │
│              │ Database Layer │                             │
│              │ (Connection    │                             │
│              │  Pool)         │                             │
│              └───────┬────────┘                             │
└──────────────────────┼──────────────────────────────────────┘
                       │
                ┌──────▼───────┐
                │ MySQL Server │
                └──────────────┘
```

### Process Model

The system runs two independent async tasks:

1. **Queue Manager Task**: Continuously polls the `logs` table for new entries and copies them to `logs_queue`
2. **Log Parser Task**: Continuously polls `logs_queue`, evaluates logs against rules, and routes matches

Both tasks share a connection pool but operate independently. This separation ensures that log ingestion never blocks log processing.

### Module Structure

```
src/
├── main.rs                 # Entry point, task spawning, signal handling
├── config.rs               # Configuration loading from environment
├── db/
│   ├── mod.rs             # Database layer public interface
│   ├── connection.rs      # Connection pool setup
│   ├── schema.rs          # Table schemas and validation
│   ├── operations.rs      # CRUD operations (insert, select, delete, replace)
│   └── models.rs          # Database row types (Log, User, etc.)
├── queue_manager.rs       # Queue management logic
├── log_parser.rs          # Log parsing and routing logic
├── rules/
│   ├── mod.rs            # Rule engine public interface
│   ├── types.rs          # Rule types and enums
│   ├── validation.rs     # Rule validation logic
│   └── matching.rs       # Rule matching logic
└── error.rs              # Error types and conversions
```

## Components and Interfaces

### Configuration Module (`config.rs`)

**Purpose**: Load and validate configuration from environment variables.

**Types**:
```rust
pub struct Config {
    pub database: DatabaseConfig,
    pub queue_manager: QueueManagerConfig,
    pub log_parser: LogParserConfig,
    pub logging: LoggingConfig,
}

pub struct DatabaseConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
    pub ssl_ca_path: Option<String>,
}

pub struct QueueManagerConfig {
    pub poll_interval_ms: u64,
    pub starting_id: i32,
}

pub struct LogParserConfig {
    pub poll_interval_ms: u64,
    pub rule_refresh_interval: usize,  // Refresh rules every N logs
}

pub struct LoggingConfig {
    pub error_log_path: String,
    pub debug_log_path: String,
    pub max_log_size_bytes: u64,
    pub max_log_backups: usize,
}
```

**Interface**:
```rust
impl Config {
    /// Load configuration from environment variables
    /// Returns error if required variables are missing or invalid
    pub fn from_env() -> Result<Self, ConfigError>;
}
```

### Database Layer (`db/`)

#### Connection Module (`db/connection.rs`)

**Purpose**: Manage MySQL connection pool.

**Interface**:
```rust
use sqlx::MySqlPool;

/// Create a connection pool from configuration
pub async fn create_pool(config: &DatabaseConfig) -> Result<MySqlPool, DbError>;
```

#### Models Module (`db/models.rs`)

**Purpose**: Define strongly-typed database row structures.

**Types**:
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Log {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub user: Option<String>,
    pub network: Option<String>,
    pub window: String,
    pub r#type: String,  // "type" is a keyword, use raw identifier
    pub nick: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LogWithRecipient {
    pub id: i32,
    pub user: Option<String>,
    pub network: Option<String>,
    pub window: String,
    pub r#type: String,
    pub nick: Option<String>,
    pub message: Option<String>,
    pub recipient: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub nickname: String,
    pub telegram_chat_id: Option<i64>,
    pub hotwords: Option<sqlx::types::Json<Vec<Rule>>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PmRecord {
    pub window: String,
    pub nick: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IdTrack {
    pub id: i32,
    pub tid: i32,
}
```

#### Schema Module (`db/schema.rs`)

**Purpose**: Define table schemas and validation logic.

**Types**:
```rust
use std::collections::HashMap;

pub struct TableSchema {
    pub columns: HashMap<String, ColumnSpec>,
}

pub struct ColumnSpec {
    pub nullable: bool,
    pub column_type: ColumnType,
}

pub enum ColumnType {
    Int,
    String,
    DateTime,
    Json,
}
```

**Interface**:
```rust
impl TableSchema {
    /// Get schema for a specific table
    pub fn for_table(table: &str) -> Option<&'static TableSchema>;
    
    /// Validate that a value matches the schema
    pub fn validate<T: Serialize>(&self, value: &T) -> Result<(), ValidationError>;
}
```

#### Operations Module (`db/operations.rs`)

**Purpose**: Provide CRUD operations with schema validation.

**Interface**:
```rust
use sqlx::MySqlPool;

/// Insert a row into a table with schema validation
pub async fn insert_into<T: Serialize>(
    pool: &MySqlPool,
    row: &T,
    table: &str,
) -> Result<(), DbError>;

/// Replace a row in a table (INSERT ... ON DUPLICATE KEY UPDATE)
pub async fn replace_into<T: Serialize>(
    pool: &MySqlPool,
    row: &T,
    table: &str,
) -> Result<(), DbError>;

/// Select rows from a table where id > base_id
pub async fn select_from(
    pool: &MySqlPool,
    table: &str,
    base_id: i32,
    desc: bool,
) -> Result<Vec<Log>, DbError>;

/// Delete rows from a table matching conditions
pub async fn delete_from(
    pool: &MySqlPool,
    table: &str,
    conditions: &HashMap<String, serde_json::Value>,
) -> Result<(), DbError>;

/// Fetch all user nicknames
pub async fn fetch_users(pool: &MySqlPool) -> Result<Vec<String>, DbError>;

/// Fetch a single user by nickname
pub async fn fetch_user(pool: &MySqlPool, nickname: &str) -> Result<Option<User>, DbError>;

/// Fetch all PM records
pub async fn fetch_pm_table(pool: &MySqlPool) -> Result<Vec<PmRecord>, DbError>;
```

### Rules Module (`rules/`)

#### Types Module (`rules/types.rs`)

**Purpose**: Define rule types and structures.

**Types**:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Rule {
    Substring {
        #[serde(rename = "match")]
        match_str: String,
        #[serde(default)]
        case_sensitive: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        only_if: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        not_if: Option<HashMap<String, String>>,
    },
    Pm {
        #[serde(skip_serializing_if = "Option::is_none")]
        only_if: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        not_if: Option<HashMap<String, String>>,
    },
}
```

#### Validation Module (`rules/validation.rs`)

**Purpose**: Validate rule structures.

**Interface**:
```rust
/// Validate a single rule
pub fn validate_rule(rule: &Rule) -> Result<(), ValidationError>;

/// Validate a list of rules
pub fn validate_rules(rules: &[Rule]) -> Result<(), ValidationError>;
```

#### Matching Module (`rules/matching.rs`)

**Purpose**: Evaluate rules against log entries.

**Interface**:
```rust
use crate::db::models::Log;

/// Check if a log matches a rule
pub fn match_rule(rule: &Rule, log: &Log) -> bool;

/// Helper: Check if a log is a private message
pub fn is_pm(log: &Log) -> bool;

/// Helper: Evaluate conditional logic (only_if, not_if)
fn evaluate_conditions(
    conditions: &HashMap<String, String>,
    log: &Log,
    case_sensitive: bool,
) -> bool;
```

### Queue Manager Module (`queue_manager.rs`)

**Purpose**: Move logs from `logs` table to `logs_queue` table.

**Interface**:
```rust
use sqlx::MySqlPool;
use crate::config::QueueManagerConfig;

/// Run the queue manager task
/// Continuously polls logs table and copies new entries to logs_queue
pub async fn run(pool: MySqlPool, config: QueueManagerConfig) -> Result<(), QueueManagerError>;

/// Get the last processed ID from logs_id_track
async fn get_last_processed_id(pool: &MySqlPool) -> Result<Option<i32>, DbError>;

/// Mark an ID as processed in logs_id_track
async fn mark_as_processed(pool: &MySqlPool, id: i32) -> Result<(), DbError>;

/// Copy new logs from logs to logs_queue
async fn copy_new_logs(
    pool: &MySqlPool,
    last_id: i32,
) -> Result<Option<i32>, DbError>;
```

### Log Parser Module (`log_parser.rs`)

**Purpose**: Parse logs from queue, evaluate rules, and route matches.

**Types**:
```rust
use std::collections::{HashMap, HashSet};

pub struct LogParser {
    pool: MySqlPool,
    config: LogParserConfig,
    user_rules: HashMap<String, Vec<Rule>>,
    pm_cache: HashSet<(String, String)>,
    logs_processed: usize,
}
```

**Interface**:
```rust
impl LogParser {
    /// Create a new log parser
    pub async fn new(
        pool: MySqlPool,
        config: LogParserConfig,
    ) -> Result<Self, LogParserError>;
    
    /// Run the log parser task
    pub async fn run(mut self) -> Result<(), LogParserError>;
    
    /// Load users and their rules from database
    async fn load_users_and_rules(&mut self) -> Result<(), DbError>;
    
    /// Load PM cache from database
    async fn load_pm_cache(&mut self) -> Result<(), DbError>;
    
    /// Process a single log entry
    async fn parse_log(&mut self, log: &Log) -> Result<(), LogParserError>;
    
    /// Track a PM if it's new
    async fn maybe_track_pm(&mut self, log: &Log) -> Result<(), DbError>;
    
    /// Check if rules should be refreshed
    fn should_refresh_rules(&self) -> bool;
}
```

### Error Module (`error.rs`)

**Purpose**: Define error types for the application.

**Types**:
```rust
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    
    #[error("Invalid configuration value for {key}: {reason}")]
    InvalidValue { key: String, reason: String },
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQL error: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("Schema validation failed: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

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

#[derive(Error, Debug)]
pub enum QueueManagerError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),
}

#[derive(Error, Debug)]
pub enum LogParserError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    
    #[error("Rule validation error: {0}")]
    RuleValidation(#[from] ValidationError),
}
```

## Data Models

### Log Entry Flow

```
logs (source)
  ↓ (Queue Manager copies)
logs_queue (processing queue)
  ↓ (Log Parser evaluates)
push (matched logs for delivery)
event_log (audit trail)
```

### Rule Structure

Rules are stored as JSON in the `users.hotwords` column. The JSON structure maps to the `Rule` enum:

**Substring Rule Example**:
```json
{
  "type": "substring",
  "match": "important",
  "case_sensitive": false,
  "only_if": {
    "window": "#engineering"
  }
}
```

**PM Rule Example**:
```json
{
  "type": "pm"
}
```

### State Management

The Log Parser maintains in-memory state:

1. **User Rules Map**: `HashMap<String, Vec<Rule>>` - Maps usernames to their rules
2. **PM Cache**: `HashSet<(String, String)>` - Tracks seen (window, nick) combinations
3. **Logs Processed Counter**: `usize` - Tracks when to refresh rules

This state is refreshed periodically (every 100 logs) to pick up rule changes without restarting.

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property 1: Log Ordering Consistency

*For any* batch of logs fetched from the database with a base id, all returned logs should have id > base_id and should be ordered in ascending order by id.

**Validates: Requirements 2.1, 2.7, 3.1**

### Property 2: Queue Copy Preserves Data

*For any* log copied from the logs table to logs_queue, all fields (id, created_at, user, network, window, type, nick, message) should have identical values in both tables.

**Validates: Requirements 2.2**

### Property 3: Tracking Table Reflects Progress

*For any* log successfully copied to logs_queue, the logs_id_track table should contain a tid value greater than or equal to that log's id.

**Validates: Requirements 2.3**

### Property 4: Log Type Filtering

*For any* log with type "msg" or "action", it should be evaluated against rules. For any log with a different type, it should be skipped without evaluation.

**Validates: Requirements 3.2, 3.3**

### Property 5: Matched Logs Appear in Both Tables

*For any* log that matches a user's rule, that log should appear in both the push table and event_log table with the recipient field set to the matching user's nickname.

**Validates: Requirements 3.4**

### Property 6: Processed Logs Are Removed

*For any* log that has been processed (evaluated against all rules), it should no longer exist in the logs_queue table.

**Validates: Requirements 3.6**

### Property 7: Rule Type Validation

*For any* rule object, if it does not contain a "type" field, validation should fail. If it contains a "type" field with an unsupported value (not "substring" or "pm"), validation should fail.

**Validates: Requirements 4.1, 4.6**

### Property 8: Substring Rule Match Field Validation

*For any* rule with type "substring", if it does not contain a "match" field or the "match" field is not a string, validation should fail.

**Validates: Requirements 4.2**

### Property 9: PM Rule Validation

*For any* rule with type "pm", validation should succeed even without a "match" field.

**Validates: Requirements 4.3**

### Property 10: Case Sensitive Field Validation

*For any* rule containing a "case_sensitive" field, if the value is not a boolean, validation should fail.

**Validates: Requirements 4.4**

### Property 11: Conditional Field Validation

*For any* rule containing "only_if" or "not_if" fields, if either value is not an object/map, validation should fail.

**Validates: Requirements 4.5**

### Property 12: Substring Matching with Case Sensitivity

*For any* substring rule and log, if the match string appears in the log's message field (respecting case_sensitive setting), and does not appear solely in the nick field, the rule should match. Case-insensitive matching should treat "ABC", "abc", and "AbC" as equivalent. Case-sensitive matching should treat them as distinct.

**Validates: Requirements 5.1, 5.2, 5.3, 5.4**

### Property 13: PM Detection Logic

*For any* log and PM rule, the rule should match if and only if the log's window field equals the nick field and the window field does not start with "#".

**Validates: Requirements 6.1, 6.2**

### Property 14: Only_If Conditional Logic

*For any* rule with an "only_if" object and any log, if any condition in the only_if object fails (either a "contains" check on the message or a field equality check), the rule should not match. The "contains" key should check if the value appears in the message field. Other keys should check if the log's field equals the condition value. Case sensitivity should be respected.

**Validates: Requirements 7.2, 7.3, 7.4, 7.8**

### Property 15: Not_If Suppression Logic

*For any* rule with a "not_if" object and any log, if all conditions in the not_if object are satisfied (both "contains" checks and field equality checks), the rule should not match (suppression). If any condition fails, the rule evaluation should continue normally. Case sensitivity should be respected.

**Validates: Requirements 7.6, 7.8**

### Property 16: PM Cache Insertion

*For any* log where window equals nick and does not start with "#", if the (window, nick) combination is not in the PM cache, it should be inserted into the pm_table and added to the cache.

**Validates: Requirements 9.3, 9.4**

### Property 17: Non-Nullable Column Validation

*For any* data being inserted or replaced, if any non-nullable column is missing or has a null value, schema validation should fail with an error indicating the missing column.

**Validates: Requirements 10.2**

### Property 18: Column Type Validation

*For any* data being inserted or replaced, if any column value does not match the expected type for that column (e.g., string where int is expected), schema validation should fail.

**Validates: Requirements 10.4**

### Property 19: Nullable Column Flexibility

*For any* data being inserted or replaced, if a nullable column is absent from the data, schema validation should succeed.

**Validates: Requirements 10.6**

## Error Handling

### Error Propagation Strategy

The system uses Rust's `Result` type for error propagation with custom error types defined in `error.rs`. Errors are categorized by domain:

- **ConfigError**: Configuration loading and validation failures
- **DbError**: Database connectivity and operation failures
- **ValidationError**: Schema and rule validation failures
- **QueueManagerError**: Queue management specific errors
- **LogParserError**: Log parsing specific errors

### Error Recovery Patterns

1. **Transient Database Errors**: Log the error and continue processing. The system should not crash on individual operation failures.

2. **Rule Validation Errors**: Skip the invalid rule and log a warning. Continue processing other rules.

3. **Duplicate Entry Errors**: Log at debug level and continue. Duplicates are expected in some scenarios (e.g., retries).

4. **Connection Pool Exhaustion**: The sqlx pool will handle waiting for available connections. Configure pool size appropriately.

5. **Fatal Errors**: Configuration errors and initial connection failures should cause the application to exit with a descriptive error message.

### Logging Strategy

Use the `tracing` crate with the following levels:

- **ERROR**: Database connection failures, unrecoverable errors, configuration errors
- **WARN**: Rule validation failures, duplicate entries, recoverable errors
- **INFO**: Startup messages, rule refresh events, periodic status updates
- **DEBUG**: Individual log processing, rule matching details, database operations
- **TRACE**: Detailed execution flow (disabled in production)

Configure rotating file handlers:
- `error.log`: ERROR and WARN levels, 10KB max size, 5 backups
- `debug.log`: All levels, 1MB max size, 10 backups

## Testing Strategy

### Dual Testing Approach

The system requires both unit tests and property-based tests for comprehensive coverage:

**Unit Tests**: Focus on specific examples, edge cases, and error conditions
- Configuration loading with missing/invalid environment variables
- Database connection failures and error messages
- Duplicate entry error handling
- Invalid JSON parsing in hotwords field
- Specific rule matching examples (e.g., PM detection for "#channel" vs "user")

**Property-Based Tests**: Verify universal properties across all inputs
- Use the `proptest` crate for property-based testing in Rust
- Each property test should run a minimum of 100 iterations
- Each test should reference its design document property using a comment tag

### Property Test Configuration

Each property-based test must:
1. Run at least 100 iterations (configure with `proptest! { #![proptest_config(ProptestConfig::with_cases(100))] }`)
2. Include a comment tag: `// Feature: python-to-rust-translation, Property N: [property text]`
3. Generate random inputs appropriate for the property being tested
4. Use proptest strategies for generating valid and invalid data

### Test Organization

```
tests/
├── unit/
│   ├── config_tests.rs
│   ├── db_operations_tests.rs
│   ├── rule_validation_tests.rs
│   └── rule_matching_tests.rs
├── property/
│   ├── log_ordering_tests.rs
│   ├── queue_copy_tests.rs
│   ├── rule_validation_tests.rs
│   ├── rule_matching_tests.rs
│   └── schema_validation_tests.rs
└── integration/
    ├── queue_manager_tests.rs
    └── log_parser_tests.rs
```

### Example Property Test Structure

```rust
use proptest::prelude::*;

// Feature: python-to-rust-translation, Property 12: Substring Matching with Case Sensitivity
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn test_substring_matching_case_insensitive(
        match_str in "[a-zA-Z]{3,10}",
        message in "[a-zA-Z ]{10,50}",
    ) {
        // Generate a log with the match string in the message
        let log = Log {
            message: Some(format!("{} {}", message, match_str)),
            nick: Some("testnick".to_string()),
            // ... other fields
        };
        
        let rule = Rule::Substring {
            match_str: match_str.clone(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        // Property: case-insensitive matching should succeed
        assert!(match_rule(&rule, &log));
        
        // Property: different case should also match
        let rule_upper = Rule::Substring {
            match_str: match_str.to_uppercase(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        assert!(match_rule(&rule_upper, &log));
    }
}
```

### Integration Testing

Integration tests should:
- Use a test MySQL database (configure via TEST_DB_* environment variables)
- Test the full flow: queue manager → logs_queue → log parser → push/event_log
- Verify rule refresh behavior after 100 logs
- Test PM tracking across multiple logs
- Verify error recovery and continuation after failures

### Test Data Generation

For property tests, use proptest strategies:

```rust
// Strategy for generating valid logs
fn arb_log() -> impl Strategy<Value = Log> {
    (
        any::<i32>(),
        any::<DateTime<Utc>>(),
        prop::option::of("[a-z]{3,10}"),
        prop::option::of("[a-z]{3,10}"),
        "[a-z#]{3,10}",
        prop_oneof!["msg", "action", "join", "part"],
        prop::option::of("[a-z]{3,10}"),
        prop::option::of("[a-zA-Z ]{10,100}"),
    ).prop_map(|(id, created_at, user, network, window, type_, nick, message)| {
        Log { id, created_at, user, network, window, r#type: type_.to_string(), nick, message }
    })
}

// Strategy for generating valid substring rules
fn arb_substring_rule() -> impl Strategy<Value = Rule> {
    (
        "[a-z]{3,10}",
        any::<bool>(),
        prop::option::of(prop::collection::hash_map("[a-z]{3,10}", "[a-z]{3,10}", 0..3)),
        prop::option::of(prop::collection::hash_map("[a-z]{3,10}", "[a-z]{3,10}", 0..3)),
    ).prop_map(|(match_str, case_sensitive, only_if, not_if)| {
        Rule::Substring { match_str, case_sensitive, only_if, not_if }
    })
}
```

### Continuous Integration

The test suite should run on every commit:
1. Unit tests (fast, no external dependencies)
2. Property tests (moderate speed, 100 iterations per property)
3. Integration tests (slower, requires test database)

Use `cargo test --all-features` to run all tests. Use `cargo test --test property_tests` to run only property tests.
