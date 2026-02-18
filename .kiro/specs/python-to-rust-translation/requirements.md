# Requirements Document

## Introduction

This document specifies the requirements for translating a Python-based IRC log parsing and routing system to idiomatic Rust. The system is a rule-driven log analysis engine that polls a MySQL database for new IRC logs, evaluates them against user-defined filtering rules, and routes matched logs to users. The translation must preserve all functional behavior while leveraging Rust's type safety, performance characteristics, and modern async patterns.

## Glossary

- **Log_Parser**: The main component that evaluates IRC logs against user rules and routes matches
- **Queue_Manager**: The component that moves new logs from the source table to the processing queue
- **Rule_Engine**: The subsystem that validates and evaluates filtering rules against log entries
- **Database_Layer**: The abstraction layer for MySQL connectivity and operations
- **IRC_Log**: A record from the logs table containing IRC message data
- **Hotword_Rule**: A user-defined filtering rule stored as JSON that specifies matching criteria
- **PM**: Private Message - an IRC message where the window equals the sender's nick
- **Connection_Pool**: A managed pool of database connections for concurrent access
- **Schema_Validator**: Component that validates data against table schemas before database operations

## Requirements

### Requirement 1: Database Connectivity

**User Story:** As a system operator, I want the Rust system to connect to MySQL databases with connection pooling, so that database operations are efficient and reliable.

#### Acceptance Criteria

1. THE Database_Layer SHALL establish connections to MySQL using environment variables for configuration (DB_HOST, DB_USERNAME, DB_PASSWORD, DB_NAME)
2. THE Database_Layer SHALL maintain a connection pool with configurable size limits
3. WHEN a database operation is requested, THE Database_Layer SHALL acquire a connection from the pool
4. THE Database_Layer SHALL support SSL/TLS connections with certificate verification
5. WHEN a connection fails, THE Database_Layer SHALL return a descriptive error with context
6. THE Database_Layer SHALL use async/await patterns for all database operations

### Requirement 2: Queue Management Process

**User Story:** As a system operator, I want new IRC logs to be automatically moved from the source table to a processing queue, so that logs can be processed without blocking new log ingestion.

#### Acceptance Criteria

1. THE Queue_Manager SHALL poll the logs table for new entries with id greater than the last processed id
2. WHEN new logs are found, THE Queue_Manager SHALL copy them to the logs_queue table
3. WHEN a log is successfully copied, THE Queue_Manager SHALL update the logs_id_track table with the processed id
4. THE Queue_Manager SHALL use REPLACE INTO semantics for updating the tracking table
5. WHEN no new logs are found, THE Queue_Manager SHALL sleep for a configurable interval before polling again
6. WHEN a database error occurs during copying, THE Queue_Manager SHALL log the error and continue operation
7. THE Queue_Manager SHALL process logs in ascending id order

### Requirement 3: Log Parsing and Routing

**User Story:** As a system operator, I want IRC logs to be evaluated against user rules and routed to matching users, so that users receive notifications for relevant messages.

#### Acceptance Criteria

1. THE Log_Parser SHALL fetch logs from the logs_queue table in ascending id order
2. WHEN a log has type "msg" or "action", THE Log_Parser SHALL evaluate it against all user rules
3. WHEN a log has a type other than "msg" or "action", THE Log_Parser SHALL skip processing and log the skip
4. WHEN a rule matches a log, THE Log_Parser SHALL insert the log into both the push table and event_log table with the recipient field set
5. WHEN inserting into push or event_log fails with a duplicate entry error, THE Log_Parser SHALL log the duplicate and continue
6. WHEN a log is processed, THE Log_Parser SHALL delete it from the logs_queue table
7. THE Log_Parser SHALL track the last processed id to resume from the correct position
8. WHEN no logs are available in the queue, THE Log_Parser SHALL sleep for a configurable interval before polling again

### Requirement 4: Rule Engine - Validation

**User Story:** As a developer, I want rules to be validated before use, so that invalid rules are rejected and don't cause runtime errors.

#### Acceptance Criteria

1. THE Rule_Engine SHALL validate that each rule contains a "type" field
2. WHEN a rule has type "substring", THE Rule_Engine SHALL validate that it contains a "match" field with a string value
3. WHEN a rule has type "pm", THE Rule_Engine SHALL accept the rule without requiring a "match" field
4. WHEN a rule contains a "case_sensitive" field, THE Rule_Engine SHALL validate that it is a boolean value
5. WHEN a rule contains "only_if" or "not_if" fields, THE Rule_Engine SHALL validate that they are objects
6. WHEN a rule has an unsupported type, THE Rule_Engine SHALL reject the rule and log the rejection
7. THE Rule_Engine SHALL validate all rules for a user before loading them into memory

### Requirement 5: Rule Engine - Substring Matching

**User Story:** As a user, I want to define substring matching rules with case sensitivity options, so that I can filter messages containing specific text.

#### Acceptance Criteria

1. WHEN a substring rule is evaluated, THE Rule_Engine SHALL check if the match string appears in the message field
2. WHEN case_sensitive is false or absent, THE Rule_Engine SHALL perform case-insensitive matching
3. WHEN case_sensitive is true, THE Rule_Engine SHALL perform case-sensitive matching
4. THE Rule_Engine SHALL NOT match when the match string appears only in the sender's nick field
5. WHEN the match string is found in the message and not solely in the nick, THE Rule_Engine SHALL return true

### Requirement 6: Rule Engine - PM Detection

**User Story:** As a user, I want to define rules that match private messages, so that I can be notified of all direct messages.

#### Acceptance Criteria

1. WHEN a PM rule is evaluated, THE Rule_Engine SHALL check if the window field equals the nick field
2. WHEN the window field starts with "#", THE Rule_Engine SHALL determine the message is not a PM
3. WHEN the window equals the nick and does not start with "#", THE Rule_Engine SHALL return true for PM rules

### Requirement 7: Rule Engine - Conditional Logic

**User Story:** As a user, I want to define conditional logic in rules using only_if and not_if clauses, so that I can create complex filtering criteria.

#### Acceptance Criteria

1. WHEN a rule contains an "only_if" object, THE Rule_Engine SHALL evaluate all conditions in the object
2. WHEN any "only_if" condition fails, THE Rule_Engine SHALL return false for the rule
3. WHEN an "only_if" condition has key "contains", THE Rule_Engine SHALL check if the value appears in the message field
4. WHEN an "only_if" condition has a key matching a log field, THE Rule_Engine SHALL check if the log field equals the condition value
5. WHEN a rule contains a "not_if" object, THE Rule_Engine SHALL evaluate all conditions in the object
6. WHEN all "not_if" conditions are satisfied, THE Rule_Engine SHALL return false for the rule (suppression)
7. WHEN any "not_if" condition fails, THE Rule_Engine SHALL continue evaluating the rule normally
8. THE Rule_Engine SHALL respect case_sensitive settings when evaluating conditional logic

### Requirement 8: Rule Loading and Refresh

**User Story:** As a system operator, I want user rules to be loaded from the database and periodically refreshed, so that rule changes take effect without restarting the system.

#### Acceptance Criteria

1. THE Log_Parser SHALL load all user nicknames from the users table at startup
2. THE Log_Parser SHALL fetch the hotwords JSON field for each user
3. WHEN the hotwords field is a JSON string, THE Log_Parser SHALL parse it into a list of rule objects
4. WHEN the hotwords field cannot be parsed, THE Log_Parser SHALL log an error and skip that user's rules
5. THE Log_Parser SHALL validate all rules before storing them in memory
6. WHEN 100 logs have been processed, THE Log_Parser SHALL reload all users and rules from the database
7. THE Log_Parser SHALL continue using the previous rules if reloading fails

### Requirement 9: Private Message Tracking

**User Story:** As a system operator, I want the system to track unique private message conversations, so that PM metadata is available for analysis.

#### Acceptance Criteria

1. THE Log_Parser SHALL load all existing PM records from the pm_table at startup into an in-memory cache
2. WHEN processing a log where window equals nick and does not start with "#", THE Log_Parser SHALL check the PM cache
3. WHEN a PM combination (window, nick) is not in the cache, THE Log_Parser SHALL insert it into the pm_table
4. WHEN a PM is inserted into pm_table, THE Log_Parser SHALL add it to the in-memory cache
5. WHEN inserting into pm_table fails, THE Log_Parser SHALL log the error and continue processing

### Requirement 10: Schema Validation

**User Story:** As a developer, I want all database operations to validate data against table schemas, so that invalid data is rejected before reaching the database.

#### Acceptance Criteria

1. THE Database_Layer SHALL define schemas for all tables (logs, logs_queue, logs_id_track, push, event_log, users, pm_table)
2. WHEN inserting or replacing data, THE Database_Layer SHALL validate that all non-nullable columns are present
3. WHEN a non-nullable column is missing or null, THE Database_Layer SHALL return an error
4. THE Database_Layer SHALL validate that column values match the expected types
5. WHEN a column type does not match, THE Database_Layer SHALL return an error with the column name and expected type
6. THE Database_Layer SHALL allow nullable columns to be absent from the data

### Requirement 11: Error Handling and Logging

**User Story:** As a system operator, I want comprehensive error logging with different severity levels, so that I can diagnose issues and monitor system health.

#### Acceptance Criteria

1. THE system SHALL use structured logging with timestamp, level, and message fields
2. THE system SHALL log errors to a rotating error log file with configurable size limits
3. THE system SHALL log debug information to a rotating debug log file with configurable size limits
4. WHEN a database operation fails, THE system SHALL log the error with context about the operation
5. WHEN a rule fails validation, THE system SHALL log the invalid rule and the reason for rejection
6. WHEN a rule matches a log, THE system SHALL log the match at debug level with rule and log details
7. THE system SHALL continue operation after logging errors unless the error is unrecoverable

### Requirement 12: Configuration Management

**User Story:** As a system operator, I want to configure the system using environment variables, so that I can deploy to different environments without code changes.

#### Acceptance Criteria

1. THE system SHALL load configuration from environment variables at startup
2. THE system SHALL support a .env file for local development configuration
3. THE system SHALL require DB_HOST, DB_USERNAME, DB_PASSWORD, and DB_NAME environment variables
4. THE system SHALL support optional configuration for connection pool size, sleep intervals, and log file sizes
5. WHEN a required environment variable is missing, THE system SHALL return an error and exit
6. THE system SHALL validate configuration values at startup before connecting to the database

### Requirement 13: Type Safety and Data Modeling

**User Story:** As a developer, I want strong typing for all data structures, so that type errors are caught at compile time.

#### Acceptance Criteria

1. THE system SHALL define struct types for all database row types (Log, User, Rule, etc.)
2. THE system SHALL use serde for JSON serialization and deserialization of rules
3. THE system SHALL use Result types for all operations that can fail
4. THE system SHALL use Option types for all nullable database fields
5. THE system SHALL define enum types for log types (msg, action) and rule types (substring, pm)
6. THE system SHALL use chrono types for datetime fields

### Requirement 14: Async Runtime and Concurrency

**User Story:** As a developer, I want the system to use async/await patterns, so that database operations don't block the runtime.

#### Acceptance Criteria

1. THE system SHALL use tokio as the async runtime
2. THE system SHALL use async functions for all database operations
3. THE system SHALL use async sleep for polling intervals
4. THE Queue_Manager and Log_Parser SHALL run as separate async tasks
5. THE system SHALL use sqlx for async MySQL operations with compile-time query checking where possible
6. THE system SHALL handle task cancellation gracefully on shutdown signals

### Requirement 15: Transaction Safety

**User Story:** As a system operator, I want database operations to be transactionally safe, so that data integrity is maintained during failures.

#### Acceptance Criteria

1. WHEN copying a log to logs_queue, THE Queue_Manager SHALL update logs_id_track in the same logical transaction
2. WHEN an insert operation fails, THE system SHALL not update tracking tables
3. THE system SHALL use autocommit mode for individual operations
4. WHEN a batch of operations is required, THE system SHALL use explicit transactions
5. WHEN a transaction fails, THE system SHALL roll back all changes and log the failure

### Requirement 16: Performance and Resource Management

**User Story:** As a system operator, I want the system to use resources efficiently, so that it can run continuously without degradation.

#### Acceptance Criteria

1. THE system SHALL reuse database connections from the pool rather than creating new connections
2. THE system SHALL limit memory usage by processing logs in batches rather than loading all logs at once
3. THE system SHALL use efficient string operations for rule matching (avoiding unnecessary allocations)
4. THE system SHALL release database connections back to the pool after operations complete
5. THE system SHALL use bounded channels for inter-task communication if needed
6. THE system SHALL handle backpressure gracefully when the queue grows large
