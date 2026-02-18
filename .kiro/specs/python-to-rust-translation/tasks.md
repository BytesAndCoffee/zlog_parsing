# Implementation Plan: Python to Rust IRC Log Parser Translation

## Overview

This implementation plan translates a Python IRC log parsing system to idiomatic Rust. The approach is incremental: first establish the foundation (configuration, database layer, data models), then implement the rule engine, followed by the two main processes (queue manager and log parser), and finally wire everything together with proper error handling and logging.

## Tasks

- [x] 1. Set up project structure and dependencies
  - Create new Rust project with `cargo init`
  - Add dependencies to Cargo.toml: sqlx (with mysql and runtime-tokio features), tokio, serde, serde_json, tracing, tracing-subscriber, dotenvy, thiserror, chrono, proptest (dev dependency)
  - Create module structure: config.rs, error.rs, db/ (mod.rs, connection.rs, models.rs, schema.rs, operations.rs), rules/ (mod.rs, types.rs, validation.rs, matching.rs), queue_manager.rs, log_parser.rs
  - Set up .env.example with required environment variables
  - _Requirements: 12.1, 12.2, 12.3_

- [ ] 2. Implement configuration module
  - [x] 2.1 Create Config structs for all configuration sections
    - Define Config, DatabaseConfig, QueueManagerConfig, LogParserConfig, LoggingConfig structs
    - Implement Default trait for optional configuration values
    - _Requirements: 12.3, 12.4_
  
  - [x] 2.2 Implement Config::from_env() method
    - Load environment variables using dotenvy
    - Parse and validate required variables (DB_HOST, DB_USERNAME, DB_PASSWORD, DB_NAME)
    - Parse optional variables with defaults (pool size, intervals, log sizes)
    - Return ConfigError for missing or invalid values
    - _Requirements: 12.1, 12.3, 12.5, 12.6_
  
  - [ ]* 2.3 Write unit tests for configuration loading
    - Test missing required environment variables
    - Test invalid configuration values
    - Test default values for optional configuration
    - _Requirements: 12.5_

- [ ] 3. Implement error types
  - [x] 3.1 Define error enums in error.rs
    - Create AppError, ConfigError, DbError, ValidationError, QueueManagerError, LogParserError enums
    - Use thiserror derive macro for error implementations
    - Implement From traits for error conversions
    - _Requirements: 1.5, 11.4_
  
  - [ ]* 3.2 Write unit tests for error conversions
    - Test error type conversions (e.g., DbError -> AppError)
    - Test error message formatting
    - _Requirements: 1.5_

- [ ] 4. Implement database models
  - [x] 4.1 Create database row structs in db/models.rs
    - Define Log, LogWithRecipient, User, PmRecord, IdTrack structs
    - Derive sqlx::FromRow, Debug, Clone for all structs
    - Use Option<T> for nullable fields
    - Use chrono::DateTime<Utc> for datetime fields
    - Use sqlx::types::Json<Vec<Rule>> for hotwords field
    - _Requirements: 13.1, 13.4, 13.6_
  
  - [ ]* 4.2 Write unit tests for model serialization
    - Test serde serialization/deserialization of models
    - Test handling of None values in optional fields
    - _Requirements: 13.2, 13.4_

- [ ] 5. Implement rule types and validation
  - [x] 5.1 Define Rule enum in rules/types.rs
    - Create Rule enum with Substring and Pm variants
    - Add serde attributes for JSON serialization
    - Use HashMap<String, String> for only_if and not_if conditions
    - _Requirements: 13.2, 13.5_
  
  - [x] 5.2 Implement rule validation in rules/validation.rs
    - Implement validate_rule() function
    - Check for required fields based on rule type
    - Validate field types (case_sensitive is bool, conditionals are objects)
    - Return ValidationError for invalid rules
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_
  
  - [ ]* 5.3 Write property test for rule type validation
    - **Property 7: Rule Type Validation**
    - **Validates: Requirements 4.1, 4.6**
  
  - [ ]* 5.4 Write property test for substring rule match field validation
    - **Property 8: Substring Rule Match Field Validation**
    - **Validates: Requirements 4.2**
  
  - [ ]* 5.5 Write property test for PM rule validation
    - **Property 9: PM Rule Validation**
    - **Validates: Requirements 4.3**
  
  - [ ]* 5.6 Write property test for case sensitive field validation
    - **Property 10: Case Sensitive Field Validation**
    - **Validates: Requirements 4.4**
  
  - [ ]* 5.7 Write property test for conditional field validation
    - **Property 11: Conditional Field Validation**
    - **Validates: Requirements 4.5**

- [ ] 6. Implement rule matching logic
  - [x] 6.1 Implement is_pm() helper in rules/matching.rs
    - Check if window equals nick and doesn't start with "#"
    - _Requirements: 6.1, 6.2_
  
  - [x] 6.2 Implement evaluate_conditions() helper
    - Handle "contains" key for message substring checks
    - Handle field equality checks for other keys
    - Respect case_sensitive parameter
    - _Requirements: 7.3, 7.4, 7.8_
  
  - [x] 6.3 Implement match_rule() function
    - Handle PM rule matching using is_pm()
    - Handle substring rule matching with case sensitivity
    - Check that match string doesn't appear solely in nick
    - Evaluate only_if conditions (all must pass)
    - Evaluate not_if conditions (suppress if all pass)
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 7.2, 7.6_
  
  - [ ]* 6.4 Write property test for substring matching with case sensitivity
    - **Property 12: Substring Matching with Case Sensitivity**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4**
  
  - [ ]* 6.5 Write property test for PM detection logic
    - **Property 13: PM Detection Logic**
    - **Validates: Requirements 6.1, 6.2**
  
  - [ ]* 6.6 Write property test for only_if conditional logic
    - **Property 14: Only_If Conditional Logic**
    - **Validates: Requirements 7.2, 7.3, 7.4, 7.8**
  
  - [ ]* 6.7 Write property test for not_if suppression logic
    - **Property 15: Not_If Suppression Logic**
    - **Validates: Requirements 7.6, 7.8**

- [x] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Implement database schema validation
  - [x] 8.1 Define table schemas in db/schema.rs
    - Create TableSchema and ColumnSpec structs
    - Define ColumnType enum (Int, String, DateTime, Json)
    - Implement for_table() to return static schemas for all tables
    - _Requirements: 10.1_
  
  - [x] 8.2 Implement schema validation logic
    - Implement validate() method on TableSchema
    - Check for missing non-nullable columns
    - Check for null values in non-nullable columns
    - Validate column types match expected types
    - Allow nullable columns to be absent
    - _Requirements: 10.2, 10.4, 10.6_
  
  - [ ]* 8.3 Write property test for non-nullable column validation
    - **Property 17: Non-Nullable Column Validation**
    - **Validates: Requirements 10.2**
  
  - [ ]* 8.4 Write property test for column type validation
    - **Property 18: Column Type Validation**
    - **Validates: Requirements 10.4**
  
  - [ ]* 8.5 Write property test for nullable column flexibility
    - **Property 19: Nullable Column Flexibility**
    - **Validates: Requirements 10.6**
  
  - [ ]* 8.6 Write unit test for validation error messages
    - Test that error messages include column name and expected type
    - _Requirements: 10.5_

- [ ] 9. Implement database connection and operations
  - [x] 9.1 Implement connection pool creation in db/connection.rs
    - Create create_pool() async function
    - Configure MySqlPoolOptions with pool size from config
    - Set up SSL/TLS with certificate path if provided
    - Return DbError on connection failure
    - _Requirements: 1.1, 1.2, 1.4, 1.5_
  
  - [x] 9.2 Implement CRUD operations in db/operations.rs
    - Implement insert_into() with schema validation
    - Implement replace_into() with schema validation
    - Implement select_from() with base_id filtering and ordering
    - Implement delete_from() with conditions
    - Implement fetch_users(), fetch_user(), fetch_pm_table()
    - Use sqlx query macros for compile-time checking where possible
    - _Requirements: 1.3, 1.6, 2.1, 2.4, 3.1_
  
  - [ ]* 9.3 Write property test for log ordering consistency
    - **Property 1: Log Ordering Consistency**
    - **Validates: Requirements 2.1, 2.7, 3.1**
  
  - [ ]* 9.4 Write unit test for database connection errors
    - Test connection failure with invalid credentials
    - Test error message contains context
    - _Requirements: 1.5_

- [ ] 10. Implement queue manager
  - [x] 10.1 Implement helper functions in queue_manager.rs
    - Implement get_last_processed_id() to query logs_id_track
    - Implement mark_as_processed() to update logs_id_track using replace_into
    - _Requirements: 2.3, 2.4_
  
  - [x] 10.2 Implement copy_new_logs() function
    - Select logs with id > last_processed_id in ascending order
    - For each log, insert into logs_queue
    - Update logs_id_track with the log's id
    - Log errors but continue on failure
    - Return the highest id processed
    - _Requirements: 2.1, 2.2, 2.3, 2.6, 2.7_
  
  - [x] 10.3 Implement run() async function
    - Initialize last_processed_id from database or use default
    - Loop: call copy_new_logs(), sleep if no new logs
    - Use tokio::time::sleep for polling interval
    - Handle errors gracefully and continue operation
    - _Requirements: 2.5, 2.6, 14.3_
  
  - [ ]* 10.4 Write property test for queue copy preserves data
    - **Property 2: Queue Copy Preserves Data**
    - **Validates: Requirements 2.2**
  
  - [ ]* 10.5 Write property test for tracking table reflects progress
    - **Property 3: Tracking Table Reflects Progress**
    - **Validates: Requirements 2.3**
  
  - [ ]* 10.6 Write unit test for error handling during copy
    - Test that database errors are logged and don't crash the process
    - _Requirements: 2.6_

- [x] 11. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 12. Implement log parser
  - [x] 12.1 Implement LogParser struct and new() method in log_parser.rs
    - Create LogParser struct with pool, config, user_rules, pm_cache, logs_processed fields
    - Implement new() to initialize the struct
    - Call load_users_and_rules() to populate user_rules
    - Call load_pm_cache() to populate pm_cache
    - _Requirements: 8.1, 8.2, 9.1_
  
  - [x] 12.2 Implement load_users_and_rules() method
    - Fetch all user nicknames using fetch_users()
    - For each user, fetch their hotwords using fetch_user()
    - Parse hotwords JSON into Vec<Rule>
    - Validate rules using validate_rules()
    - Store valid rules in user_rules HashMap
    - Log errors for invalid JSON or failed validation
    - _Requirements: 8.2, 8.3, 8.4, 8.5_
  
  - [x] 12.3 Implement load_pm_cache() method
    - Fetch all PM records using fetch_pm_table()
    - Insert (window, nick) tuples into pm_cache HashSet
    - _Requirements: 9.1_
  
  - [x] 12.4 Implement maybe_track_pm() method
    - Check if log is a PM (window == nick && !window.starts_with("#"))
    - Check if (window, nick) is in pm_cache
    - If not in cache, insert into pm_table and add to cache
    - Log errors but continue on failure
    - _Requirements: 9.2, 9.3, 9.4, 9.5_
  
  - [x] 12.5 Implement parse_log() method
    - Check if log type is "msg" or "action", skip otherwise
    - For each user and their rules, call match_rule()
    - If rule matches, create LogWithRecipient and insert into push and event_log
    - Handle duplicate entry errors gracefully
    - Call maybe_track_pm() for the log
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 9.2_
  
  - [x] 12.6 Implement run() method
    - Initialize last_processed_id
    - Loop: fetch logs from logs_queue, process each log, delete from queue
    - Increment logs_processed counter
    - Check if rules should be refreshed (every 100 logs)
    - If refresh needed, call load_users_and_rules()
    - Sleep if no logs available
    - Handle errors gracefully and continue operation
    - _Requirements: 3.1, 3.6, 3.7, 3.8, 8.6, 8.7_
  
  - [ ]* 12.7 Write property test for log type filtering
    - **Property 4: Log Type Filtering**
    - **Validates: Requirements 3.2, 3.3**
  
  - [ ]* 12.8 Write property test for matched logs appear in both tables
    - **Property 5: Matched Logs Appear in Both Tables**
    - **Validates: Requirements 3.4**
  
  - [ ]* 12.9 Write property test for processed logs are removed
    - **Property 6: Processed Logs Are Removed**
    - **Validates: Requirements 3.6**
  
  - [ ]* 12.10 Write property test for PM cache insertion
    - **Property 16: PM Cache Insertion**
    - **Validates: Requirements 9.3, 9.4**
  
  - [ ]* 12.11 Write unit test for duplicate entry error handling
    - Test that duplicate entry errors are logged and don't crash
    - _Requirements: 3.5_
  
  - [ ]* 12.12 Write unit test for invalid JSON in hotwords
    - Test that invalid JSON is logged and user is skipped
    - _Requirements: 8.4_
  
  - [ ]* 12.13 Write unit test for PM tracking error handling
    - Test that PM tracking errors are logged and don't crash
    - _Requirements: 9.5_

- [ ] 13. Implement logging infrastructure
  - [x] 13.1 Set up tracing subscriber in main.rs
    - Configure tracing_subscriber with multiple layers
    - Set up rotating file appender for error.log (ERROR and WARN levels)
    - Set up rotating file appender for debug.log (all levels)
    - Use configuration from LoggingConfig for file sizes and backup counts
    - _Requirements: 11.1, 11.2, 11.3_
  
  - [x] 13.2 Add tracing instrumentation to all modules
    - Add #[instrument] attributes to key functions
    - Use tracing::error!, warn!, info!, debug!, trace! macros
    - Log database errors with context
    - Log rule validation failures with rule details
    - Log rule matches at debug level
    - _Requirements: 11.4, 11.5, 11.6_

- [ ] 14. Implement main entry point and task spawning
  - [x] 14.1 Implement main() function in main.rs
    - Load configuration using Config::from_env()
    - Set up logging using tracing subscriber
    - Create database connection pool
    - Spawn queue_manager::run() as a tokio task
    - Spawn log_parser::run() as a tokio task
    - Set up signal handlers for graceful shutdown (SIGINT, SIGTERM)
    - Await both tasks and handle errors
    - _Requirements: 12.1, 14.1, 14.2, 14.4, 14.6_
  
  - [ ]* 14.2 Write integration test for full system flow
    - Test queue manager copying logs to logs_queue
    - Test log parser processing logs and routing matches
    - Test rule refresh after 100 logs
    - Test PM tracking across multiple logs
    - _Requirements: 2.2, 3.4, 8.6, 9.3_

- [x] 15. Final checkpoint - Ensure all tests pass
  - Run `cargo test --all-features` to verify all tests pass
  - Run `cargo clippy` to check for common mistakes
  - Run `cargo fmt` to ensure consistent formatting
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 16. Documentation and deployment preparation
  - [x] 16.1 Write README.md
    - Document system architecture and components
    - Provide setup instructions (environment variables, database schema)
    - Document how to run the system
    - Document how to run tests
    - _Requirements: 12.2_
  
  - [x] 16.2 Create example configuration files
    - Create .env.example with all required and optional variables
    - Document default values and valid ranges
    - _Requirements: 12.2, 12.3, 12.4_
  
  - [x] 16.3 Add inline documentation
    - Add doc comments to all public functions and types
    - Document error conditions and return values
    - Add examples for complex functions
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6_

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties (minimum 100 iterations each)
- Unit tests validate specific examples and edge cases
- Integration tests verify end-to-end system behavior
- The implementation follows idiomatic Rust patterns: Result types, Option types, async/await, strong typing
- Use `cargo test` to run unit and property tests, use `cargo test --test integration_tests` for integration tests
