// Tests for database schema validation

use irc_log_parser::db::schema::{TableSchema, ColumnType};

#[test]
fn test_for_table_returns_logs_schema() {
    let schema = TableSchema::for_table("logs");
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert!(schema.columns.contains_key("id"));
    assert!(schema.columns.contains_key("created_at"));
    assert!(schema.columns.contains_key("window"));
    assert!(schema.columns.contains_key("type"));
}

#[test]
fn test_for_table_returns_logs_queue_schema() {
    let schema = TableSchema::for_table("logs_queue");
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert!(schema.columns.contains_key("id"));
    assert!(schema.columns.contains_key("message"));
}

#[test]
fn test_for_table_returns_logs_id_track_schema() {
    let schema = TableSchema::for_table("logs_id_track");
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert!(schema.columns.contains_key("id"));
    assert!(schema.columns.contains_key("tid"));
    assert_eq!(schema.columns.len(), 2);
}

#[test]
fn test_for_table_returns_push_schema() {
    let schema = TableSchema::for_table("push");
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert!(schema.columns.contains_key("recipient"));
}

#[test]
fn test_for_table_returns_event_log_schema() {
    let schema = TableSchema::for_table("event_log");
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert!(schema.columns.contains_key("recipient"));
}

#[test]
fn test_for_table_returns_users_schema() {
    let schema = TableSchema::for_table("users");
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert!(schema.columns.contains_key("nickname"));
    assert!(schema.columns.contains_key("telegram_chat_id"));
    assert!(schema.columns.contains_key("hotwords"));
}

#[test]
fn test_for_table_returns_pm_table_schema() {
    let schema = TableSchema::for_table("pm_table");
    assert!(schema.is_some());
    
    let schema = schema.unwrap();
    assert!(schema.columns.contains_key("window"));
    assert!(schema.columns.contains_key("nick"));
    assert_eq!(schema.columns.len(), 2);
}

#[test]
fn test_for_table_returns_none_for_unknown_table() {
    let schema = TableSchema::for_table("unknown_table");
    assert!(schema.is_none());
}

#[test]
fn test_column_spec_nullable_field() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    // Check non-nullable columns
    let id_col = schema.columns.get("id").unwrap();
    assert!(!id_col.nullable);
    assert_eq!(id_col.column_type, ColumnType::Int);
    
    let window_col = schema.columns.get("window").unwrap();
    assert!(!window_col.nullable);
    assert_eq!(window_col.column_type, ColumnType::String);
    
    // Check nullable columns
    let user_col = schema.columns.get("user").unwrap();
    assert!(user_col.nullable);
    assert_eq!(user_col.column_type, ColumnType::String);
    
    let message_col = schema.columns.get("message").unwrap();
    assert!(message_col.nullable);
    assert_eq!(message_col.column_type, ColumnType::String);
}

#[test]
fn test_column_types() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    // Int type
    let id_col = schema.columns.get("id").unwrap();
    assert_eq!(id_col.column_type, ColumnType::Int);
    
    // String type
    let window_col = schema.columns.get("window").unwrap();
    assert_eq!(window_col.column_type, ColumnType::String);
    
    // DateTime type
    let created_at_col = schema.columns.get("created_at").unwrap();
    assert_eq!(created_at_col.column_type, ColumnType::DateTime);
}

#[test]
fn test_users_table_json_column() {
    let schema = TableSchema::for_table("users").unwrap();
    
    let hotwords_col = schema.columns.get("hotwords").unwrap();
    assert_eq!(hotwords_col.column_type, ColumnType::Json);
    assert!(hotwords_col.nullable);
}

// Tests for schema validation

use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct ValidLog {
    id: i32,
    created_at: String,
    user: Option<String>,
    network: Option<String>,
    window: String,
    r#type: String,
    nick: Option<String>,
    message: Option<String>,
}

#[test]
fn test_validate_valid_log_with_all_fields() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    let log = ValidLog {
        id: 1,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        user: Some("testuser".to_string()),
        network: Some("freenode".to_string()),
        window: "#test".to_string(),
        r#type: "msg".to_string(),
        nick: Some("testnick".to_string()),
        message: Some("test message".to_string()),
    };
    
    let result = schema.validate(&log);
    assert!(result.is_ok());
}

#[test]
fn test_validate_valid_log_with_nullable_fields_absent() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    let log = ValidLog {
        id: 1,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        user: None,
        network: None,
        window: "#test".to_string(),
        r#type: "msg".to_string(),
        nick: None,
        message: None,
    };
    
    let result = schema.validate(&log);
    assert!(result.is_ok());
}

#[test]
fn test_validate_missing_non_nullable_column() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    // Missing "window" which is non-nullable
    let log = json!({
        "id": 1,
        "created_at": "2024-01-01T00:00:00Z",
        "type": "msg"
    });
    
    let result = schema.validate(&log);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, irc_log_parser::error::ValidationError::MissingColumn(_)));
    assert!(err.to_string().contains("window"));
}

#[test]
fn test_validate_null_value_in_non_nullable_column() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    // "window" is non-nullable but has null value
    let log = json!({
        "id": 1,
        "created_at": "2024-01-01T00:00:00Z",
        "window": null,
        "type": "msg"
    });
    
    let result = schema.validate(&log);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, irc_log_parser::error::ValidationError::NullValue(_)));
    assert!(err.to_string().contains("window"));
}

#[test]
fn test_validate_type_mismatch_int_column() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    // "id" should be int but is string
    let log = json!({
        "id": "not_an_int",
        "created_at": "2024-01-01T00:00:00Z",
        "window": "#test",
        "type": "msg"
    });
    
    let result = schema.validate(&log);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, irc_log_parser::error::ValidationError::TypeMismatch { .. }));
    assert!(err.to_string().contains("id"));
    assert!(err.to_string().contains("Int"));
}

#[test]
fn test_validate_type_mismatch_string_column() {
    let schema = TableSchema::for_table("logs").unwrap();
    
    // "window" should be string but is int
    let log = json!({
        "id": 1,
        "created_at": "2024-01-01T00:00:00Z",
        "window": 123,
        "type": "msg"
    });
    
    let result = schema.validate(&log);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, irc_log_parser::error::ValidationError::TypeMismatch { .. }));
    assert!(err.to_string().contains("window"));
    assert!(err.to_string().contains("String"));
}

#[test]
fn test_validate_logs_id_track_valid() {
    let schema = TableSchema::for_table("logs_id_track").unwrap();
    
    let track = json!({
        "id": 1,
        "tid": 100
    });
    
    let result = schema.validate(&track);
    assert!(result.is_ok());
}

#[test]
fn test_validate_logs_id_track_missing_tid() {
    let schema = TableSchema::for_table("logs_id_track").unwrap();
    
    let track = json!({
        "id": 1
    });
    
    let result = schema.validate(&track);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, irc_log_parser::error::ValidationError::MissingColumn(_)));
    assert!(err.to_string().contains("tid"));
}

#[test]
fn test_validate_push_table_with_recipient() {
    let schema = TableSchema::for_table("push").unwrap();
    
    let push = json!({
        "id": 1,
        "window": "#test",
        "type": "msg",
        "recipient": "user123"
    });
    
    let result = schema.validate(&push);
    assert!(result.is_ok());
}

#[test]
fn test_validate_push_table_missing_recipient() {
    let schema = TableSchema::for_table("push").unwrap();
    
    let push = json!({
        "id": 1,
        "window": "#test",
        "type": "msg"
    });
    
    let result = schema.validate(&push);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, irc_log_parser::error::ValidationError::MissingColumn(_)));
    assert!(err.to_string().contains("recipient"));
}

#[test]
fn test_validate_users_table_with_json_column() {
    let schema = TableSchema::for_table("users").unwrap();
    
    let user = json!({
        "nickname": "testuser",
        "telegram_chat_id": 123456,
        "hotwords": [{"type": "substring", "match": "test"}]
    });
    
    let result = schema.validate(&user);
    assert!(result.is_ok());
}

#[test]
fn test_validate_users_table_nullable_json_absent() {
    let schema = TableSchema::for_table("users").unwrap();
    
    let user = json!({
        "nickname": "testuser"
    });
    
    let result = schema.validate(&user);
    assert!(result.is_ok());
}

#[test]
fn test_validate_pm_table_valid() {
    let schema = TableSchema::for_table("pm_table").unwrap();
    
    let pm = json!({
        "window": "user1",
        "nick": "user2"
    });
    
    let result = schema.validate(&pm);
    assert!(result.is_ok());
}

#[test]
fn test_validate_pm_table_missing_nick() {
    let schema = TableSchema::for_table("pm_table").unwrap();
    
    let pm = json!({
        "window": "user1"
    });
    
    let result = schema.validate(&pm);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, irc_log_parser::error::ValidationError::MissingColumn(_)));
    assert!(err.to_string().contains("nick"));
}
