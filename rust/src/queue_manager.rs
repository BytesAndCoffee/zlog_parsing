// Queue management logic

use sqlx::MySqlPool;
use serde::Serialize;
use crate::error::DbError;
use crate::db::operations::replace_into;
use crate::config::QueueManagerConfig;
use tokio::time::{sleep, Duration};

/// Get the last processed ID from logs_id_track
/// 
/// # Requirements
/// - 2.3: Update logs_id_track table with the processed id
/// 
/// Returns None if no tracking record exists yet
async fn get_last_processed_id(pool: &MySqlPool) -> Result<Option<i32>, DbError> {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT tid FROM logs_id_track WHERE id = 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// Mark an ID as processed in logs_id_track
/// 
/// # Requirements
/// - 2.3: Update logs_id_track table with the processed id
/// - 2.4: Use REPLACE INTO semantics for updating the tracking table
async fn mark_as_processed(pool: &MySqlPool, id: i32) -> Result<(), DbError> {
    #[derive(Serialize)]
    struct IdTrackRecord {
        id: i32,
        tid: i32,
    }

    let record = IdTrackRecord {
        id: 1,  // Fixed id for the tracking record
        tid: id,
    };

    replace_into(pool, &record, "logs_id_track").await?;
    Ok(())
}

/// Copy new logs from logs to logs_queue
/// 
/// # Requirements
/// - 2.1: Poll the logs table for new entries with id greater than the last processed id
/// - 2.2: Copy logs to the logs_queue table
/// - 2.3: Update logs_id_track table with the processed id
/// - 2.6: Log errors but continue operation
/// - 2.7: Process logs in ascending id order
/// 
/// Returns the highest id processed, or None if no logs were processed
pub async fn copy_new_logs(
    pool: &MySqlPool,
    last_id: i32,
) -> Result<Option<i32>, DbError> {
    use crate::db::operations::{select_from, insert_into};
    
    // Select logs with id > last_id in ascending order
    let logs = select_from(pool, "logs", last_id, false).await?;
    
    if logs.is_empty() {
        return Ok(None);
    }
    
    let mut highest_id = last_id;
    
    // Process each log
    for log in logs {
        let log_id = log.id;
        
        // Try to insert into logs_queue
        match insert_into(pool, &log, "logs_queue").await {
            Ok(_) => {
                // Try to update tracking table
                match mark_as_processed(pool, log_id).await {
                    Ok(_) => {
                        highest_id = log_id;
                        tracing::debug!("Copied log {} to logs_queue", log_id);
                    }
                    Err(e) => {
                        // Log error but continue
                        tracing::error!("Failed to update logs_id_track for log {}: {}", log_id, e);
                    }
                }
            }
            Err(e) => {
                // Log error but continue
                tracing::error!("Failed to insert log {} into logs_queue: {}", log_id, e);
            }
        }
    }
    
    // Return the highest id that was successfully processed
    if highest_id > last_id {
        Ok(Some(highest_id))
    } else {
        Ok(None)
    }
}

/// Run the queue manager task
/// 
/// Continuously polls the logs table for new entries and copies them to logs_queue.
/// This function runs indefinitely until the task is cancelled.
/// 
/// # Requirements
/// - 2.5: Sleep for a configurable interval when no new logs are found
/// - 2.6: Handle errors gracefully and continue operation
/// - 14.3: Use tokio::time::sleep for polling interval
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `config` - Queue manager configuration
pub async fn run(pool: MySqlPool, config: QueueManagerConfig) -> Result<(), DbError> {
    tracing::info!("Starting queue manager");
    
    // Initialize last_processed_id from database or use default from config
    let mut last_processed_id = match get_last_processed_id(&pool).await {
        Ok(Some(id)) => {
            tracing::info!("Resuming from last processed id: {}", id);
            id
        }
        Ok(None) => {
            tracing::info!("No tracking record found, starting from id: {}", config.starting_id);
            config.starting_id
        }
        Err(e) => {
            tracing::error!("Failed to get last processed id: {}, using starting_id: {}", e, config.starting_id);
            config.starting_id
        }
    };
    
    let poll_interval = Duration::from_millis(config.poll_interval_ms);
    
    // Main processing loop
    loop {
        match copy_new_logs(&pool, last_processed_id).await {
            Ok(Some(new_id)) => {
                // Successfully processed logs, update our tracking
                last_processed_id = new_id;
                tracing::debug!("Updated last_processed_id to {}", new_id);
            }
            Ok(None) => {
                // No new logs found, sleep before polling again
                tracing::trace!("No new logs found, sleeping for {:?}", poll_interval);
                sleep(poll_interval).await;
            }
            Err(e) => {
                // Log error but continue operation
                tracing::error!("Error copying logs: {}, continuing operation", e);
                sleep(poll_interval).await;
            }
        }
    }
}
