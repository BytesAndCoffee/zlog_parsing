# IRC Log Parser

A high-performance Rust implementation of an IRC log parsing and routing system. This system polls a MySQL database for new IRC logs, evaluates them against user-defined filtering rules, and routes matched logs to users for notification.

## Architecture

The system consists of two independent async tasks that run concurrently:

1. **Queue Manager**: Continuously polls the `logs` table for new entries and copies them to `logs_queue` for processing
2. **Log Parser**: Evaluates logs from `logs_queue` against user-defined rules and routes matches to the `push` and `event_log` tables

Both tasks share a connection pool and operate independently to ensure log ingestion never blocks log processing.

### Key Components

- **Database Layer**: Async MySQL operations with connection pooling (sqlx)
- **Rule Engine**: Type-safe rule validation and matching with support for substring and PM detection
- **Configuration**: Environment-based configuration with .env file support
- **Logging**: Structured logging with rotating file handlers (tracing)
- **Error Handling**: Comprehensive error types with context (thiserror)

## Prerequisites

- Rust 1.70 or later
- MySQL 5.7 or later
- Access to an IRC logs database with the required schema

## Database Schema

The system requires the following tables:

- `logs`: Source table for IRC messages (auto-increment primary key)
- `logs_queue`: Processing queue for logs
- `logs_id_track`: Tracks the last processed log ID
- `users`: User accounts with hotword rules (JSON)
- `push`: Matched logs ready for delivery
- `event_log`: Audit trail of all matched logs
- `pm_table`: Tracks unique private message conversations

See `python/zlog_schema.sql` for the complete schema definition.

## Setup

### 1. Clone and Build

```bash
cd rust
cargo build --release
```

### 2. Configure Environment Variables

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```bash
# Database Configuration (Required)
DB_HOST=localhost
DB_USERNAME=your_username
DB_PASSWORD=your_password
DB_NAME=irc_logs

# Database Connection Pool (Optional, defaults shown)
DB_POOL_SIZE=10

# SSL/TLS Configuration (Optional)
# DB_SSL_CA_PATH=/path/to/ca-cert.pem

# Queue Manager Configuration (Optional, defaults shown)
QUEUE_MANAGER_POLL_INTERVAL_MS=1000
QUEUE_MANAGER_STARTING_ID=0

# Log Parser Configuration (Optional, defaults shown)
LOG_PARSER_POLL_INTERVAL_MS=1000
LOG_PARSER_RULE_REFRESH_INTERVAL=100

# Logging Configuration (Optional, defaults shown)
ERROR_LOG_PATH=logs/error.log
DEBUG_LOG_PATH=logs/debug.log
MAX_LOG_SIZE_BYTES=10485760
MAX_LOG_BACKUPS=5
```

### 3. Initialize Database

Ensure your MySQL database has the required schema. Run the schema creation script:

```bash
mysql -u your_username -p your_database < ../python/zlog_schema.sql
```

Create the additional tracking tables:

```sql
CREATE TABLE IF NOT EXISTS logs_id_track (
    id INT PRIMARY KEY,
    tid INT NOT NULL
);

CREATE TABLE IF NOT EXISTS pm_table (
    window VARCHAR(255) NOT NULL,
    nick VARCHAR(128) NOT NULL,
    PRIMARY KEY (window, nick)
);
```

## Running the System

### Development Mode

```bash
cargo run
```

### Production Mode

```bash
cargo run --release
```

The system will:
1. Load configuration from environment variables
2. Establish database connection pool
3. Spawn Queue Manager and Log Parser tasks
4. Begin processing logs continuously

Press `Ctrl+C` to gracefully shutdown.

## User Rules

Users define filtering rules in the `users.hotwords` JSON column. Rules are evaluated against each IRC message.

### Rule Types

#### Substring Rule

Matches messages containing a specific string:

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

- `match`: String to search for in the message
- `case_sensitive`: Boolean (default: false)
- `only_if`: Optional conditions that must all be satisfied
- `not_if`: Optional conditions that suppress the match if all are satisfied

#### PM Rule

Matches all private messages (where window equals nick):

```json
{
  "type": "pm"
}
```

### Conditional Logic

Rules support `only_if` and `not_if` conditions:

- **only_if**: All conditions must be satisfied for the rule to match
  - `contains`: Check if value appears in message
  - Field names: Check if log field equals the condition value
  
- **not_if**: If all conditions are satisfied, the rule is suppressed

Example with conditions:

```json
{
  "type": "substring",
  "match": "deploy",
  "only_if": {
    "window": "#ops",
    "contains": "production"
  },
  "not_if": {
    "nick": "bot"
  }
}
```

This matches messages containing "deploy" in #ops that also contain "production", but not from the user "bot".

## Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test Suites

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Property-based tests
cargo test --test '*' -- --ignored
```

### Test Database Setup

Integration tests require a test database. Configure it in `.env.test`:

```bash
DB_HOST=localhost
DB_USERNAME=test_user
DB_PASSWORD=test_password
DB_NAME=irc_logs_test
```

## Logging

The system uses structured logging with two log files:

- **error.log**: ERROR and WARN level messages
- **debug.log**: All log levels including DEBUG and TRACE

Logs rotate automatically based on size limits configured in environment variables.

### Log Levels

- **ERROR**: Database connection failures, unrecoverable errors
- **WARN**: Rule validation failures, duplicate entries, recoverable errors
- **INFO**: Startup messages, rule refresh events, periodic status
- **DEBUG**: Individual log processing, rule matching details
- **TRACE**: Detailed execution flow (disabled in production)

## Performance Tuning

### Connection Pool Size

Adjust `DB_POOL_SIZE` based on your workload:
- Higher values: Better concurrency, more database connections
- Lower values: Fewer resources, potential bottleneck

### Poll Intervals

- `QUEUE_MANAGER_POLL_INTERVAL_MS`: How often to check for new logs
- `LOG_PARSER_POLL_INTERVAL_MS`: How often to check the processing queue

Lower values increase responsiveness but use more CPU.

### Rule Refresh Interval

`LOG_PARSER_RULE_REFRESH_INTERVAL`: Number of logs to process before reloading rules from the database (default: 100). Lower values pick up rule changes faster but increase database load.

## Deployment

See [README_BUILD.md](README_BUILD.md) for cross-compilation and deployment instructions, including:
- Building for x86_64 Linux from Apple Silicon macOS
- Deploying to remote hosts via Tailscale
- Setting up systemd services

## Project Structure

```
rust/
├── src/
│   ├── main.rs              # Entry point, task spawning
│   ├── config.rs            # Configuration loading
│   ├── error.rs             # Error types
│   ├── queue_manager.rs     # Queue management logic
│   ├── log_parser.rs        # Log parsing and routing
│   ├── db/
│   │   ├── mod.rs          # Database layer interface
│   │   ├── connection.rs   # Connection pool setup
│   │   ├── models.rs       # Database row types
│   │   ├── operations.rs   # CRUD operations
│   │   └── schema.rs       # Schema validation
│   └── rules/
│       ├── mod.rs          # Rule engine interface
│       ├── types.rs        # Rule types and enums
│       ├── validation.rs   # Rule validation
│       └── matching.rs     # Rule matching logic
├── tests/                   # Integration and property tests
├── Cargo.toml              # Dependencies and build config
└── README.md               # This file
```

## Dependencies

- **sqlx**: Async MySQL driver with compile-time query checking
- **tokio**: Async runtime
- **serde/serde_json**: JSON serialization
- **tracing**: Structured logging
- **dotenvy**: Environment variable loading
- **thiserror**: Error handling
- **chrono**: Date/time handling
- **proptest**: Property-based testing (dev)

## Troubleshooting

### Connection Errors

If you see database connection errors:
1. Verify MySQL is running: `mysql -u your_username -p`
2. Check credentials in `.env`
3. Ensure database exists: `SHOW DATABASES;`
4. Check firewall rules if connecting remotely

### Missing Tables

If you see "table doesn't exist" errors:
1. Run the schema creation script
2. Verify all tables exist: `SHOW TABLES;`
3. Check table structure: `DESCRIBE table_name;`

### Rule Validation Errors

If rules aren't matching:
1. Check logs for validation errors
2. Verify JSON syntax in `users.hotwords`
3. Test rules with simple examples first
4. Enable DEBUG logging to see matching details

### Performance Issues

If the system is slow:
1. Increase `DB_POOL_SIZE`
2. Add database indexes on frequently queried columns
3. Reduce `LOG_PARSER_RULE_REFRESH_INTERVAL`
4. Check database query performance with `EXPLAIN`

## License

See the main project LICENSE file.
