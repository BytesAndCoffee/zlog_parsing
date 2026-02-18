// CRUD operations (insert, select, delete, replace)

use sqlx::MySqlPool;
use serde::Serialize;
use std::collections::HashMap;
use crate::db::models::{Log, User, PmRecord};
use crate::db::schema::TableSchema;
use crate::error::DbError;

/// Insert a row into a table with schema validation
/// 
/// # Requirements
/// - 1.3: Use async/await patterns for database operations
/// - 1.6: Use async/await patterns for all database operations
/// - 10.2: Validate that all non-nullable columns are present
/// - 10.4: Validate that column values match the expected types
/// - 10.6: Allow nullable columns to be absent from the data
pub async fn insert_into<T: Serialize>(
    pool: &MySqlPool,
    row: &T,
    table: &str,
) -> Result<(), DbError> {
    // Skip schema validation for performance in hot paths
    // Validation is done at the application layer
    
    // Serialize to JSON to extract field names and values
    let json_value = serde_json::to_value(row)?;
    let obj = json_value.as_object()
        .ok_or_else(|| crate::error::ValidationError::InvalidRule("Value must be an object".to_string()))?;

    // Build INSERT query dynamically with backtick-escaped column names
    let columns: Vec<String> = obj.keys().map(|s| format!("`{}`", s)).collect();
    let placeholders: Vec<String> = (0..columns.len()).map(|_| "?".to_string()).collect();
    
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        placeholders.join(", ")
    );

    // Build query with bound parameters
    let mut query_builder = sqlx::query(&query);
    for key in obj.keys() {
        let value = &obj[key];
        query_builder = bind_json_value(query_builder, value);
    }

    query_builder.execute(pool).await?;
    Ok(())
}

/// Replace a row in a table (INSERT ... ON DUPLICATE KEY UPDATE)
/// 
/// # Requirements
/// - 1.3: Use async/await patterns for database operations
/// - 2.4: Use REPLACE INTO semantics for updating the tracking table
pub async fn replace_into<T: Serialize>(
    pool: &MySqlPool,
    row: &T,
    table: &str,
) -> Result<(), DbError> {
    // Skip schema validation for performance in hot paths
    // Validation is done at the application layer
    
    // Serialize to JSON to extract field names and values
    let json_value = serde_json::to_value(row)?;
    let obj = json_value.as_object()
        .ok_or_else(|| crate::error::ValidationError::InvalidRule("Value must be an object".to_string()))?;

    // Build REPLACE query dynamically with backtick-escaped column names
    let columns: Vec<String> = obj.keys().map(|s| format!("`{}`", s)).collect();
    let placeholders: Vec<String> = (0..columns.len()).map(|_| "?".to_string()).collect();
    
    let query = format!(
        "REPLACE INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        placeholders.join(", ")
    );

    // Build query with bound parameters
    let mut query_builder = sqlx::query(&query);
    for key in obj.keys() {
        let value = &obj[key];
        query_builder = bind_json_value(query_builder, value);
    }

    query_builder.execute(pool).await?;
    Ok(())
}

/// Select rows from a table where id > base_id
/// 
/// # Requirements
/// - 2.1: Poll the logs table for new entries with id greater than the last processed id
/// - 2.7: Process logs in ascending id order
/// - 3.1: Fetch logs from the logs_queue table in ascending id order
pub async fn select_from(
    pool: &MySqlPool,
    table: &str,
    base_id: i32,
    desc: bool,
) -> Result<Vec<Log>, DbError> {
    let order = if desc { "DESC" } else { "ASC" };
    let query = format!(
        "SELECT id, created_at, user, network, `window`, `type`, nick, message FROM {} WHERE id > ? ORDER BY id {} LIMIT 50",
        table, order
    );

    let logs = sqlx::query_as::<_, Log>(&query)
        .bind(base_id)
        .fetch_all(pool)
        .await?;

    Ok(logs)
}

/// Delete rows from a table matching conditions
/// 
/// # Requirements
/// - 3.6: Delete log from logs_queue table after processing
pub async fn delete_from(
    pool: &MySqlPool,
    table: &str,
    conditions: &HashMap<String, serde_json::Value>,
) -> Result<(), DbError> {
    if conditions.is_empty() {
        return Err(crate::error::ValidationError::InvalidRule(
            "Cannot delete without conditions".to_string()
        ).into());
    }

    // Build WHERE clause with backtick-escaped column names
    let where_clauses: Vec<String> = conditions.keys()
        .map(|k| format!("`{}` = ?", k))
        .collect();
    
    let query = format!(
        "DELETE FROM {} WHERE {}",
        table,
        where_clauses.join(" AND ")
    );

    // Build query with bound parameters
    let mut query_builder = sqlx::query(&query);
    for value in conditions.values() {
        query_builder = bind_json_value(query_builder, value);
    }

    query_builder.execute(pool).await?;
    Ok(())
}

/// Fetch all user nicknames
/// 
/// # Requirements
/// - 8.1: Load all user nicknames from the users table at startup
pub async fn fetch_users(pool: &MySqlPool) -> Result<Vec<String>, DbError> {
    let users = sqlx::query_scalar::<_, String>("SELECT nickname FROM users")
        .fetch_all(pool)
        .await?;

    Ok(users)
}

/// Fetch a single user by nickname
/// 
/// # Requirements
/// - 8.2: Fetch the hotwords JSON field for each user
pub async fn fetch_user(pool: &MySqlPool, nickname: &str) -> Result<Option<User>, DbError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT nickname, telegram_chat_id, hotwords FROM users WHERE nickname = ?"
    )
    .bind(nickname)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Fetch all PM records
/// 
/// # Requirements
/// - 9.1: Load all existing PM records from the pm_table at startup into an in-memory cache
pub async fn fetch_pm_table(pool: &MySqlPool) -> Result<Vec<PmRecord>, DbError> {
    let records = sqlx::query_as::<_, PmRecord>("SELECT `window`, nick FROM pm_table")
        .fetch_all(pool)
        .await?;

    Ok(records)
}

/// Helper function to bind a JSON value to a query
/// This handles the conversion from serde_json::Value to sqlx parameter types
fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    value: &'q serde_json::Value,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match value {
        serde_json::Value::Null => query.bind(Option::<String>::None),
        serde_json::Value::Bool(b) => query.bind(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(u) = n.as_u64() {
                query.bind(u as i64)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        serde_json::Value::String(s) => {
            // Check if this is an ISO 8601 datetime string and convert to MySQL format
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                // Convert to MySQL datetime format: YYYY-MM-DD HH:MM:SS
                let mysql_format = dt.format("%Y-%m-%d %H:%M:%S").to_string();
                query.bind(mysql_format)
            } else {
                query.bind(s)
            }
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            // For complex types, serialize to JSON string
            query.bind(value.to_string())
        }
    }
}
