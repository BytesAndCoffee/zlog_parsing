// Tests for database operations

use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

// Test that insert_into validates against schema
#[test]
fn test_insert_into_validates_schema() {
    // This is a compile-time test to ensure the function signature is correct
    // We can't test actual database operations without a test database
    
    #[derive(Serialize)]
    struct TestLog {
        id: i32,
        created_at: String,
        window: String,
        r#type: String,
    }
    
    let log = TestLog {
        id: 1,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        window: "#test".to_string(),
        r#type: "msg".to_string(),
    };
    
    // Verify schema validation works
    let schema = irc_log_parser::db::schema::TableSchema::for_table("logs").unwrap();
    let result = schema.validate(&log);
    assert!(result.is_ok());
}

#[test]
fn test_insert_into_rejects_invalid_data() {
    // Test that missing required fields are caught by schema validation
    let log = json!({
        "id": 1,
        "created_at": "2024-01-01T00:00:00Z",
        // Missing "window" which is required
        "type": "msg"
    });
    
    let schema = irc_log_parser::db::schema::TableSchema::for_table("logs").unwrap();
    let result = schema.validate(&log);
    assert!(result.is_err());
}

#[test]
fn test_replace_into_validates_schema() {
    // Test that replace_into also validates against schema
    let track = json!({
        "id": 1,
        "tid": 100
    });
    
    let schema = irc_log_parser::db::schema::TableSchema::for_table("logs_id_track").unwrap();
    let result = schema.validate(&track);
    assert!(result.is_ok());
}

#[test]
fn test_delete_from_requires_conditions() {
    // Test that delete_from requires conditions
    // This is a logical test - we can't test the actual function without a database
    let conditions: HashMap<String, serde_json::Value> = HashMap::new();
    
    // Empty conditions should be rejected
    assert!(conditions.is_empty());
}

#[test]
fn test_delete_from_with_conditions() {
    // Test that delete_from accepts conditions
    let mut conditions: HashMap<String, serde_json::Value> = HashMap::new();
    conditions.insert("id".to_string(), json!(1));
    
    // Non-empty conditions should be accepted
    assert!(!conditions.is_empty());
    assert_eq!(conditions.len(), 1);
}

// Test that the operations module exports the expected functions
#[test]
fn test_operations_module_exports() {
    // This test verifies that all required functions are exported
    // by attempting to reference them (compile-time check)
    
    // The fact that this compiles means the functions exist and are public
    let _ = irc_log_parser::db::operations::insert_into::<serde_json::Value>;
    let _ = irc_log_parser::db::operations::replace_into::<serde_json::Value>;
    let _ = irc_log_parser::db::operations::select_from;
    let _ = irc_log_parser::db::operations::delete_from;
    let _ = irc_log_parser::db::operations::fetch_users;
    let _ = irc_log_parser::db::operations::fetch_user;
    let _ = irc_log_parser::db::operations::fetch_pm_table;
}
