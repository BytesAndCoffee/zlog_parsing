// Database row types (Log, User, etc.)

use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::rules::types::Rule;

/// Represents a log entry from the logs or logs_queue table
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Log {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub user: Option<String>,
    pub network: Option<String>,
    pub window: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub r#type: String,  // "type" is a keyword, use raw identifier
    pub nick: Option<String>,
    pub message: Option<String>,
}

/// Represents a log entry with a recipient field for push and event_log tables
#[derive(Debug, Clone, Serialize)]
pub struct LogWithRecipient {
    pub id: i32,
    pub user: Option<String>,
    pub network: Option<String>,
    pub window: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub nick: Option<String>,
    pub message: Option<String>,
    pub recipient: String,
}

/// Represents a user from the users table
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub nickname: String,
    pub telegram_chat_id: Option<i64>,
    pub hotwords: Option<sqlx::types::Json<Vec<Rule>>>,
}

/// Represents a PM record from the pm_table
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PmRecord {
    pub window: String,
    pub nick: String,
}

/// Represents an ID tracking record from the logs_id_track table
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IdTrack {
    pub id: i32,
    pub tid: i32,
}
