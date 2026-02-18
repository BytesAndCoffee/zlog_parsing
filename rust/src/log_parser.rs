// Log parsing and routing logic

use sqlx::MySqlPool;
use std::collections::{HashMap, HashSet};
use crate::config::LogParserConfig;
use crate::rules::types::Rule;
use crate::rules::validation::validate_rules;
use crate::db::operations::{fetch_users, fetch_user, fetch_pm_table};
use crate::error::LogParserError;

/// Log parser that evaluates logs against user rules and routes matches
pub struct LogParser {
    pool: MySqlPool,
    config: LogParserConfig,
    user_rules: HashMap<String, Vec<Rule>>,
    pm_cache: HashSet<(String, String)>,
    logs_processed: usize,
}

impl LogParser {
    /// Create a new log parser
    /// 
    /// Initializes the log parser by loading users, their rules, and the PM cache
    /// from the database.
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `config` - Log parser configuration
    /// 
    /// # Returns
    /// * `Ok(LogParser)` if initialization succeeds
    /// * `Err(LogParserError)` if loading users or PM cache fails
    /// 
    /// # Requirements
    /// - 8.1: Load all user nicknames from the users table at startup
    /// - 8.2: Fetch the hotwords JSON field for each user
    /// - 9.1: Load all existing PM records from the pm_table at startup into an in-memory cache
    pub async fn new(
        pool: MySqlPool,
        config: LogParserConfig,
    ) -> Result<Self, LogParserError> {
        let mut parser = Self {
            pool,
            config,
            user_rules: HashMap::new(),
            pm_cache: HashSet::new(),
            logs_processed: 0,
        };
        
        // Load users and their rules
        parser.load_users_and_rules().await?;
        
        // Load PM cache
        parser.load_pm_cache().await?;
        
        Ok(parser)
    }
    
    /// Load users and their rules from database
    /// 
    /// Fetches all users from the database, parses their hotwords JSON field,
    /// validates the rules, and stores them in the user_rules HashMap.
    /// 
    /// # Returns
    /// * `Ok(())` if loading succeeds
    /// * `Err(LogParserError)` if database operations fail
    /// 
    /// # Requirements
    /// - 8.1: Load all user nicknames from the users table at startup
    /// - 8.2: Fetch the hotwords JSON field for each user
    /// - 8.3: Parse hotwords field into a list of rule objects
    /// - 8.4: Log error and skip user if hotwords cannot be parsed
    /// - 8.5: Validate all rules before storing them in memory
    async fn load_users_and_rules(&mut self) -> Result<(), LogParserError> {
        // Clear existing rules
        self.user_rules.clear();
        
        // Fetch all user nicknames
        let nicknames = fetch_users(&self.pool).await?;
        
        tracing::info!("Loading rules for {} users", nicknames.len());
        
        // For each user, fetch their hotwords and parse rules
        for nickname in nicknames {
            match fetch_user(&self.pool, &nickname).await {
                Ok(Some(user)) => {
                    // Check if user has hotwords
                    if let Some(hotwords_json) = user.hotwords {
                        let rules = hotwords_json.0; // Extract Vec<Rule> from Json wrapper
                        
                        // Validate rules
                        match validate_rules(&rules) {
                            Ok(()) => {
                                tracing::debug!(
                                    "Loaded {} rules for user {}",
                                    rules.len(),
                                    nickname
                                );
                                self.user_rules.insert(nickname, rules);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Invalid rules for user {}: {}. Skipping user.",
                                    nickname,
                                    e
                                );
                            }
                        }
                    } else {
                        tracing::debug!("User {} has no hotwords configured", nickname);
                    }
                }
                Ok(None) => {
                    tracing::warn!("User {} not found in database", nickname);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to fetch user {}: {}. Skipping user.",
                        nickname,
                        e
                    );
                }
            }
        }
        
        tracing::info!(
            "Successfully loaded rules for {} users",
            self.user_rules.len()
        );
        
        Ok(())
    }
    
    /// Load PM cache from database
    /// 
    /// Fetches all PM records from the pm_table and stores them in the
    /// in-memory pm_cache HashSet for fast lookup.
    /// 
    /// # Returns
    /// * `Ok(())` if loading succeeds
    /// * `Err(LogParserError)` if database operations fail
    /// 
    /// # Requirements
    /// - 9.1: Load all existing PM records from the pm_table at startup into an in-memory cache
    async fn load_pm_cache(&mut self) -> Result<(), LogParserError> {
        // Clear existing cache
        self.pm_cache.clear();
        
        // Fetch all PM records
        let pm_records = fetch_pm_table(&self.pool).await?;
        
        tracing::info!("Loading {} PM records into cache", pm_records.len());
        
        // Insert into cache
        for record in pm_records {
            self.pm_cache.insert((record.window, record.nick));
        }
        
        tracing::info!(
            "Successfully loaded {} PM records into cache",
            self.pm_cache.len()
        );
        
        Ok(())
    }
    
    /// Track a PM if it's new
    /// 
    /// Checks if the log is a private message (window == nick && !window.starts_with("#")).
    /// If it is a PM and the (window, nick) combination is not in the cache, inserts it
    /// into the pm_table and adds it to the cache.
    /// 
    /// # Arguments
    /// * `log` - The log entry to check
    /// 
    /// # Returns
    /// * `Ok(())` if tracking succeeds or if the PM is already cached
    /// * `Err(DbError)` if database operations fail (errors are logged but not propagated)
    /// 
    /// # Requirements
    /// - 9.2: Check if log is a PM (window equals nick and does not start with "#")
    /// - 9.3: Check if (window, nick) combination is not in the cache, insert into pm_table
    /// - 9.4: Add PM to in-memory cache after insertion
    /// - 9.5: Log errors but continue processing
    async fn maybe_track_pm(&mut self, log: &crate::db::models::Log) -> Result<(), crate::error::DbError> {
        use crate::rules::matching::is_pm;
        use crate::db::operations::insert_into;
        
        // Check if log is a PM
        if !is_pm(log) {
            return Ok(());
        }
        
        // Get the nick (we know it exists because is_pm checks for it)
        let nick = log.nick.as_ref().unwrap();
        
        // Check if (window, nick) is in cache
        let key = (log.window.clone(), nick.clone());
        if self.pm_cache.contains(&key) {
            return Ok(());
        }
        
        // Not in cache, insert into pm_table
        tracing::debug!(
            "Tracking new PM: window={}, nick={}",
            log.window,
            nick
        );
        
        // Create a PmRecord for insertion
        let pm_record = crate::db::models::PmRecord {
            window: log.window.clone(),
            nick: nick.clone(),
        };
        
        // Insert into pm_table
        match insert_into(&self.pool, &pm_record, "pm_table").await {
            Ok(()) => {
                // Add to cache
                self.pm_cache.insert(key);
                tracing::debug!(
                    "Successfully tracked PM: window={}, nick={}",
                    log.window,
                    nick
                );
                Ok(())
            }
            Err(e) => {
                // Log error but continue processing
                tracing::error!(
                    "Failed to insert PM into pm_table (window={}, nick={}): {}. Continuing processing.",
                    log.window,
                    nick,
                    e
                );
                // Return Ok to continue processing despite the error
                Ok(())
            }
        }
    }
    
    /// Check if rules should be refreshed
    /// 
    /// Rules are refreshed every N logs (configured in rule_refresh_interval).
    /// 
    /// # Returns
    /// * `true` if rules should be refreshed
    /// * `false` otherwise
    /// 
    /// # Requirements
    /// - 8.6: Refresh rules every 100 logs
    fn should_refresh_rules(&self) -> bool {
        self.logs_processed > 0 && self.logs_processed % self.config.rule_refresh_interval == 0
    }
    
    /// Run the log parser task
    /// 
    /// Continuously polls logs_queue, evaluates logs against rules, routes matches,
    /// and periodically refreshes rules from the database.
    /// 
    /// # Returns
    /// * `Ok(())` if the task completes (never returns in normal operation)
    /// * `Err(LogParserError)` if a fatal error occurs
    /// 
    /// # Requirements
    /// - 3.1: Fetch logs from the logs_queue table in ascending id order
    /// - 3.6: Delete log from logs_queue table after processing
    /// - 3.7: Track the last processed id to resume from the correct position
    /// - 3.8: Sleep for a configurable interval when no logs are available
    /// - 8.6: Reload all users and rules from the database every 100 logs
    /// - 8.7: Continue using previous rules if reloading fails
    pub async fn run(mut self) -> Result<(), LogParserError> {
        use crate::db::operations::{select_from, delete_from};
        use std::collections::HashMap;
        
        // Initialize last_processed_id to 0 (will fetch all logs with id > 0)
        let mut last_processed_id = 0;
        
        tracing::info!("Log parser started");
        
        loop {
            // Fetch logs from logs_queue with id > last_processed_id
            let logs = match select_from(&self.pool, "logs_queue", last_processed_id, false).await {
                Ok(logs) => logs,
                Err(e) => {
                    tracing::error!("Failed to fetch logs from logs_queue: {}. Retrying...", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(self.config.poll_interval_ms)).await;
                    continue;
                }
            };
            
            if logs.is_empty() {
                // No logs available, sleep before polling again
                tracing::trace!("No logs in queue, sleeping for {}ms", self.config.poll_interval_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(self.config.poll_interval_ms)).await;
                continue;
            }
            
            tracing::debug!("Processing {} logs from queue", logs.len());
            
            // Process logs concurrently using tokio tasks
            let mut tasks = Vec::new();
            
            for log in logs {
                // Clone necessary data for the task
                let pool = self.pool.clone();
                let user_rules = self.user_rules.clone();
                let log_clone = log.clone();
                
                // Spawn a task to process this log
                let task = tokio::spawn(async move {
                    // Process the log
                    let result = Self::process_log_static(&pool, &user_rules, &log_clone).await;
                    (log_clone.id, result)
                });
                
                tasks.push(task);
            }
            
            // Wait for all tasks to complete and collect results
            let mut processed_ids = Vec::new();
            let mut max_id = last_processed_id;
            
            for task in tasks {
                match task.await {
                    Ok((log_id, result)) => {
                        match result {
                            Ok(()) => {
                                tracing::trace!("Successfully processed log id={}", log_id);
                                processed_ids.push(log_id);
                                if log_id > max_id {
                                    max_id = log_id;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to process log id={}: {}. Continuing with next log.",
                                    log_id,
                                    e
                                );
                                // Still add to processed_ids to remove from queue
                                processed_ids.push(log_id);
                                if log_id > max_id {
                                    max_id = log_id;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Task panicked: {}", e);
                    }
                }
                
                // Increment logs_processed counter
                self.logs_processed += 1;
                
                // Check if rules should be refreshed
                if self.should_refresh_rules() {
                    tracing::info!(
                        "Refreshing rules after processing {} logs",
                        self.logs_processed
                    );
                    
                    match self.load_users_and_rules().await {
                        Ok(()) => {
                            tracing::info!("Successfully refreshed rules");
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to refresh rules: {}. Continuing with existing rules.",
                                e
                            );
                            // Continue using previous rules
                        }
                    }
                }
            }
            
            // Update last_processed_id to the highest id we processed
            last_processed_id = max_id;
            
            // Batch delete all processed logs from logs_queue
            if !processed_ids.is_empty() {
                let ids_str = processed_ids.iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                
                let delete_query = format!("DELETE FROM logs_queue WHERE id IN ({})", ids_str);
                
                match sqlx::query(&delete_query).execute(&self.pool).await {
                    Ok(_) => {
                        tracing::debug!("Batch deleted {} logs from logs_queue", processed_ids.len());
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to batch delete logs from logs_queue: {}. Continuing.",
                            e
                        );
                    }
                }
            }
        }
    }
    
    /// Process a single log entry
    /// 
    /// Evaluates the log against all user rules and routes matches to the push
    /// and event_log tables. Also tracks PMs.
    /// 
    /// # Arguments
    /// * `log` - The log entry to process
    /// 
    /// # Returns
    /// * `Ok(())` if processing succeeds
    /// * `Err(LogParserError)` if database operations fail
    /// 
    /// # Requirements
    /// - 3.2: Evaluate logs with type "msg" or "action" against all user rules
    /// - 3.3: Skip logs with types other than "msg" or "action"
    /// - 3.4: Insert matched logs into both push and event_log tables with recipient field
    /// - 3.5: Handle duplicate entry errors gracefully
    /// - 9.2: Track PMs for all logs
    async fn parse_log(&mut self, log: &crate::db::models::Log) -> Result<(), LogParserError> {
        Self::process_log_static(&self.pool, &self.user_rules, log).await?;
        
        // Track PM if applicable
        self.maybe_track_pm(log).await?;
        
        Ok(())
    }
    
    /// Static method to process a log without needing mutable self
    /// This allows concurrent processing via tokio::spawn
    async fn process_log_static(
        pool: &MySqlPool,
        user_rules: &HashMap<String, Vec<Rule>>,
        log: &crate::db::models::Log,
    ) -> Result<(), LogParserError> {
        use crate::rules::matching::match_rule;
        use crate::db::operations::insert_into;
        use crate::db::models::LogWithRecipient;
        
        // Check if log type is "msg" or "action"
        if log.r#type != "msg" && log.r#type != "action" {
            tracing::debug!(
                "Skipping log id={} with type '{}' (not 'msg' or 'action')",
                log.id,
                log.r#type
            );
            return Ok(());
        }
        
        // Evaluate log against all user rules
        for (user, rules) in user_rules {
            for rule in rules {
                if match_rule(rule, log) {
                    tracing::debug!(
                        "Log id={} matched rule for user {}",
                        log.id,
                        user
                    );
                    
                    // Create LogWithRecipient
                    let log_with_recipient = LogWithRecipient {
                        id: log.id,
                        user: log.user.clone(),
                        network: log.network.clone(),
                        window: log.window.clone(),
                        r#type: log.r#type.clone(),
                        nick: log.nick.clone(),
                        message: log.message.clone(),
                        recipient: user.clone(),
                    };
                    
                    // Insert into push table
                    match insert_into(pool, &log_with_recipient, "push").await {
                        Ok(()) => {
                            tracing::debug!(
                                "Inserted log id={} into push table for user {}",
                                log.id,
                                user
                            );
                        }
                        Err(e) => {
                            // Check if it's a duplicate entry error
                            if is_duplicate_entry_error(&e) {
                                tracing::debug!(
                                    "Duplicate entry for log id={} in push table for user {}",
                                    log.id,
                                    user
                                );
                            } else {
                                // Log other errors but continue processing
                                tracing::error!(
                                    "Failed to insert log id={} into push table for user {}: {}",
                                    log.id,
                                    user,
                                    e
                                );
                            }
                        }
                    }
                    
                    // Insert into event_log table
                    match insert_into(pool, &log_with_recipient, "event_log").await {
                        Ok(()) => {
                            tracing::debug!(
                                "Inserted log id={} into event_log table for user {}",
                                log.id,
                                user
                            );
                        }
                        Err(e) => {
                            // Check if it's a duplicate entry error
                            if is_duplicate_entry_error(&e) {
                                tracing::debug!(
                                    "Duplicate entry for log id={} in event_log table for user {}",
                                    log.id,
                                    user
                                );
                            } else {
                                // Log other errors but continue processing
                                tracing::error!(
                                    "Failed to insert log id={} into event_log table for user {}: {}",
                                    log.id,
                                    user,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Helper function to check if an error is a duplicate entry error
fn is_duplicate_entry_error(error: &crate::error::DbError) -> bool {
    match error {
        crate::error::DbError::Sqlx(sqlx_error) => {
            if let Some(db_error) = sqlx_error.as_database_error() {
                // MySQL duplicate entry error code is 1062
                db_error.code().map_or(false, |code| code == "1062")
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use crate::db::models::Log;

    /// Helper function to create a test log
    fn create_test_log(window: &str, nick: Option<&str>) -> Log {
        Log {
            id: 1,
            created_at: Utc::now(),
            user: Some("testuser".to_string()),
            network: Some("testnet".to_string()),
            window: window.to_string(),
            r#type: "msg".to_string(),
            nick: nick.map(|s| s.to_string()),
            message: Some("test message".to_string()),
        }
    }

    #[test]
    fn test_pm_detection_window_equals_nick() {
        use crate::rules::matching::is_pm;
        
        // PM: window equals nick and doesn't start with #
        let log = create_test_log("alice", Some("alice"));
        assert!(is_pm(&log), "Should detect PM when window equals nick");
    }

    #[test]
    fn test_pm_detection_channel_not_pm() {
        use crate::rules::matching::is_pm;
        
        // Not a PM: window starts with #
        let log = create_test_log("#channel", Some("alice"));
        assert!(!is_pm(&log), "Should not detect PM for channel messages");
    }

    #[test]
    fn test_pm_detection_different_window_and_nick() {
        use crate::rules::matching::is_pm;
        
        // Not a PM: window doesn't equal nick
        let log = create_test_log("bob", Some("alice"));
        assert!(!is_pm(&log), "Should not detect PM when window differs from nick");
    }

    #[test]
    fn test_pm_detection_none_nick() {
        use crate::rules::matching::is_pm;
        
        // Not a PM: nick is None
        let log = create_test_log("alice", None);
        assert!(!is_pm(&log), "Should not detect PM when nick is None");
    }
}
