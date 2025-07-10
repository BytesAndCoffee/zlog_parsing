# Modules Documentation

## parse_logs.py

### setup_logging
Sets up logging for the application, creating handlers for different log levels and adding them to the logger.

### parse_log
Parses a log entry and inserts it into the `push` and `event_log` tables if certain conditions are met.

### fetch_pm_table
Fetches all rows from the `pm_table`.

### pm_update
Updates the `pm_table` with a new log entry if certain conditions are met.

### main
Main function that sets up logging, fetches logs from the `logs_queue`, processes them, and updates the `pm_table`.

## psconnect.py

### get_db_connection
Establishes a connection to the database using environment variables.

### validate_schema
Validates a row against the schema of a specified table.

### insert_into
Inserts a row into a specified table after validating the schema.

### replace_into
Replaces a row in a specified table after validating the schema.

### select_from
Selects rows from a specified table based on conditions.

### delete_from
Deletes rows from a specified table based on conditions.

## zlog_queue.py

### setup_logging
Sets up logging for the application, creating handlers for different log levels and adding them to the logger.

### get_last_processed_id
Fetches the last processed ID from the `logs_id_track` table.

### mark_as_processed
Marks a log entry as processed by updating the `logs_id_track` table.

### copy_new_logs
Copies new log entries from the `logs` table to the `logs_queue` table and marks them as processed.

### main
Main function that sets up logging, copies new logs, and marks them as processed in a loop.

## rules.py

### validate_rule
Checks that a rule dictionary has the proper structure.

### validate_rules
Applies `validate_rule` to a list of rules.

### match_rule
Determines whether a log entry matches a given rule.

### fetch_rules
Retrieves a user's hotword rules from the database.
