// Integration tests for rule validation with serde

use irc_log_parser::rules::types::Rule;
use irc_log_parser::rules::validation::{validate_rule, validate_rules};

#[test]
fn test_serde_rejects_missing_type_field() {
    let json = r#"{"match":"test"}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_serde_rejects_unsupported_type() {
    let json = r#"{"type":"unsupported","match":"test"}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_serde_rejects_substring_without_match() {
    let json = r#"{"type":"substring"}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_serde_rejects_invalid_case_sensitive_type() {
    let json = r#"{"type":"substring","match":"test","case_sensitive":"not_a_bool"}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_serde_rejects_invalid_only_if_type() {
    let json = r#"{"type":"substring","match":"test","only_if":"not_an_object"}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_serde_rejects_invalid_not_if_type() {
    let json = r#"{"type":"substring","match":"test","not_if":["array","not","object"]}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_serde_accepts_pm_without_match() {
    let json = r#"{"type":"pm"}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    
    if let Ok(rule) = result {
        assert!(validate_rule(&rule).is_ok());
    }
}

#[test]
fn test_validation_rejects_empty_match_string() {
    // Serde will accept empty string, but our validation should reject it
    let json = r#"{"type":"substring","match":""}"#;
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    
    if let Ok(rule) = result {
        let validation_result = validate_rule(&rule);
        assert!(validation_result.is_err());
        assert!(validation_result.unwrap_err().to_string().contains("cannot be empty"));
    }
}

#[test]
fn test_validate_rules_with_mixed_valid_and_invalid() {
    let json_rules = vec![
        r#"{"type":"substring","match":"valid"}"#,
        r#"{"type":"pm"}"#,
        r#"{"type":"substring","match":""}"#,  // Invalid: empty match
    ];
    
    let mut rules = Vec::new();
    for json in json_rules {
        if let Ok(rule) = serde_json::from_str(json) {
            rules.push(rule);
        }
    }
    
    let result = validate_rules(&rules);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("index 2"));
}

#[test]
fn test_validate_complex_rule_with_conditions() {
    let json = r##"{
        "type":"substring",
        "match":"important",
        "case_sensitive":true,
        "only_if":{"window":"#engineering","network":"freenode"},
        "not_if":{"nick":"bot"}
    }"##;
    
    let result: Result<Rule, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    
    if let Ok(rule) = result {
        assert!(validate_rule(&rule).is_ok());
    }
}
