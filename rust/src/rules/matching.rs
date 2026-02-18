// Rule matching logic

use crate::db::models::Log;
use std::collections::HashMap;

/// Evaluate conditional logic (only_if, not_if)
/// 
/// Returns true if all conditions are satisfied, false otherwise.
/// 
/// Conditions can be:
/// - "contains": Check if the value appears in the message field
/// - Other keys: Check if the log's field equals the condition value
/// 
/// Requirements: 7.3, 7.4, 7.8
pub fn evaluate_conditions(
    conditions: &HashMap<String, String>,
    log: &Log,
    case_sensitive: bool,
) -> bool {
    for (key, value) in conditions {
        if key == "contains" {
            // Check if value appears in the message field
            let Some(message) = &log.message else {
                return false;
            };
            
            let contains = if case_sensitive {
                message.contains(value)
            } else {
                message.to_lowercase().contains(&value.to_lowercase())
            };
            
            if !contains {
                return false;
            }
        } else {
            // Field equality check
            let field_value = match key.as_str() {
                "window" => Some(&log.window),
                "type" => Some(&log.r#type),
                "user" => log.user.as_ref(),
                "network" => log.network.as_ref(),
                "nick" => log.nick.as_ref(),
                "message" => log.message.as_ref(),
                _ => None,
            };
            
            let Some(field_value) = field_value else {
                return false;
            };
            
            let matches = if case_sensitive {
                field_value == value
            } else {
                field_value.to_lowercase() == value.to_lowercase()
            };
            
            if !matches {
                return false;
            }
        }
    }
    
    true
}

/// Check if a log matches a rule
/// 
/// Evaluates a rule against a log entry, handling:
/// - PM rule matching using is_pm()
/// - Substring rule matching with case sensitivity
/// - Checking that match string doesn't appear solely in nick
/// - Evaluating only_if conditions (all must pass)
/// - Evaluating not_if conditions (suppress if all pass)
/// 
/// Requirements: 5.1, 5.2, 5.3, 5.4, 7.2, 7.6
pub fn match_rule(rule: &crate::rules::types::Rule, log: &Log) -> bool {
    use crate::rules::types::Rule;
    
    match rule {
        Rule::Pm { only_if, not_if } => {
            // Check if it's a PM
            if !is_pm(log) {
                return false;
            }
            
            // Evaluate only_if conditions (all must pass)
            if let Some(conditions) = only_if {
                if !evaluate_conditions(conditions, log, false) {
                    return false;
                }
            }
            
            // Evaluate not_if conditions (suppress if all pass)
            if let Some(conditions) = not_if {
                if evaluate_conditions(conditions, log, false) {
                    return false;
                }
            }
            
            true
        }
        Rule::Substring {
            match_str,
            case_sensitive,
            only_if,
            not_if,
        } => {
            // Check if message exists
            let Some(message) = &log.message else {
                return false;
            };
            
            // Check if match string appears in message (with case sensitivity)
            let message_contains = if *case_sensitive {
                message.contains(match_str)
            } else {
                message.to_lowercase().contains(&match_str.to_lowercase())
            };
            
            if !message_contains {
                return false;
            }
            
            // Check that match string doesn't appear SOLELY in nick
            // The requirement: "THE Rule_Engine SHALL NOT match when the match string appears only in the sender's nick field"
            // This means: if the message contains the match string, but that match string is only there because
            // the nick is mentioned in the message, we should not match.
            
            if let Some(nick) = &log.nick {
                let nick_contains = if *case_sensitive {
                    nick.contains(match_str)
                } else {
                    nick.to_lowercase().contains(&match_str.to_lowercase())
                };
                
                // If the match string is in the nick, we need to check if it appears in the message
                // outside of the nick context
                if nick_contains {
                    // Check if the message is exactly the nick (or just the nick with whitespace)
                    let message_trimmed = message.trim();
                    let matches_exactly = if *case_sensitive {
                        message_trimmed == nick
                    } else {
                        message_trimmed.to_lowercase() == nick.to_lowercase()
                    };
                    
                    if matches_exactly {
                        // The message is just the nick, so the match is solely in the nick
                        return false;
                    }
                    
                    // Otherwise, the match string appears in both the message and nick,
                    // but the message has additional content, so it's not SOLELY in the nick
                }
            }
            
            // Evaluate only_if conditions (all must pass)
            if let Some(conditions) = only_if {
                if !evaluate_conditions(conditions, log, *case_sensitive) {
                    return false;
                }
            }
            
            // Evaluate not_if conditions (suppress if all pass)
            if let Some(conditions) = not_if {
                if evaluate_conditions(conditions, log, *case_sensitive) {
                    return false;
                }
            }
            
            true
        }
    }
}

/// Check if a log is a private message
/// 
/// A log is considered a PM if:
/// - The window field equals the nick field
/// - The window field does not start with "#"
/// 
/// Requirements: 6.1, 6.2
pub fn is_pm(log: &Log) -> bool {
    // If nick is None, it can't be a PM
    let Some(nick) = &log.nick else {
        return false;
    };
    
    // Check if window equals nick and doesn't start with "#"
    log.window == *nick && !log.window.starts_with('#')
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

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

    fn create_log_with_message(message: &str) -> Log {
        Log {
            id: 1,
            created_at: Utc::now(),
            user: Some("testuser".to_string()),
            network: Some("testnet".to_string()),
            window: "#channel".to_string(),
            r#type: "msg".to_string(),
            nick: Some("alice".to_string()),
            message: Some(message.to_string()),
        }
    }

    #[test]
    fn test_is_pm_when_window_equals_nick() {
        let log = create_test_log("alice", Some("alice"));
        assert!(is_pm(&log), "Should be PM when window equals nick");
    }

    #[test]
    fn test_is_not_pm_when_window_starts_with_hash() {
        let log = create_test_log("#channel", Some("#channel"));
        assert!(!is_pm(&log), "Should not be PM when window starts with #");
    }

    #[test]
    fn test_is_not_pm_when_window_differs_from_nick() {
        let log = create_test_log("#channel", Some("alice"));
        assert!(!is_pm(&log), "Should not be PM when window differs from nick");
    }

    #[test]
    fn test_is_not_pm_when_nick_is_none() {
        let log = create_test_log("alice", None);
        assert!(!is_pm(&log), "Should not be PM when nick is None");
    }

    #[test]
    fn test_is_pm_case_sensitive() {
        let log = create_test_log("Alice", Some("alice"));
        assert!(!is_pm(&log), "Should be case-sensitive: Alice != alice");
    }

    // Tests for evaluate_conditions()

    #[test]
    fn test_evaluate_conditions_contains_case_insensitive() {
        let log = create_log_with_message("Hello World");
        let mut conditions = HashMap::new();
        conditions.insert("contains".to_string(), "hello".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, false),
            "Should match 'hello' in 'Hello World' case-insensitively"
        );
    }

    #[test]
    fn test_evaluate_conditions_contains_case_sensitive() {
        let log = create_log_with_message("Hello World");
        let mut conditions = HashMap::new();
        conditions.insert("contains".to_string(), "hello".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, true),
            "Should not match 'hello' in 'Hello World' case-sensitively"
        );
        
        conditions.insert("contains".to_string(), "Hello".to_string());
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should match 'Hello' in 'Hello World' case-sensitively"
        );
    }

    #[test]
    fn test_evaluate_conditions_contains_not_found() {
        let log = create_log_with_message("Hello World");
        let mut conditions = HashMap::new();
        conditions.insert("contains".to_string(), "goodbye".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, false),
            "Should not match when substring not found"
        );
    }

    #[test]
    fn test_evaluate_conditions_contains_with_none_message() {
        let mut log = create_log_with_message("test");
        log.message = None;
        
        let mut conditions = HashMap::new();
        conditions.insert("contains".to_string(), "test".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, false),
            "Should return false when message is None"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_window() {
        let log = create_test_log("#engineering", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("window".to_string(), "#engineering".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should match window field"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_case_insensitive() {
        let log = create_test_log("#Engineering", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("window".to_string(), "#engineering".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, false),
            "Should match window field case-insensitively"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_case_sensitive() {
        let log = create_test_log("#Engineering", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("window".to_string(), "#engineering".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, true),
            "Should not match window field case-sensitively when case differs"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_type() {
        let log = create_test_log("#channel", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("type".to_string(), "msg".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should match type field"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_nick() {
        let log = create_test_log("#channel", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("nick".to_string(), "alice".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should match nick field"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_user() {
        let log = create_test_log("#channel", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("user".to_string(), "testuser".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should match user field"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_network() {
        let log = create_test_log("#channel", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("network".to_string(), "testnet".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should match network field"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_equality_message() {
        let log = create_log_with_message("test message");
        let mut conditions = HashMap::new();
        conditions.insert("message".to_string(), "test message".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should match message field"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_not_matching() {
        let log = create_test_log("#channel", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("window".to_string(), "#other".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, true),
            "Should not match when field value differs"
        );
    }

    #[test]
    fn test_evaluate_conditions_field_none_value() {
        let mut log = create_test_log("#channel", Some("alice"));
        log.user = None;
        
        let mut conditions = HashMap::new();
        conditions.insert("user".to_string(), "testuser".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, true),
            "Should return false when field is None"
        );
    }

    #[test]
    fn test_evaluate_conditions_unknown_field() {
        let log = create_test_log("#channel", Some("alice"));
        let mut conditions = HashMap::new();
        conditions.insert("unknown_field".to_string(), "value".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, true),
            "Should return false for unknown field"
        );
    }

    #[test]
    fn test_evaluate_conditions_multiple_conditions_all_pass() {
        let log = create_log_with_message("important update");
        let mut conditions = HashMap::new();
        conditions.insert("contains".to_string(), "important".to_string());
        conditions.insert("window".to_string(), "#channel".to_string());
        conditions.insert("nick".to_string(), "alice".to_string());
        
        assert!(
            evaluate_conditions(&conditions, &log, false),
            "Should match when all conditions pass"
        );
    }

    #[test]
    fn test_evaluate_conditions_multiple_conditions_one_fails() {
        let log = create_log_with_message("important update");
        let mut conditions = HashMap::new();
        conditions.insert("contains".to_string(), "important".to_string());
        conditions.insert("window".to_string(), "#other".to_string());
        conditions.insert("nick".to_string(), "alice".to_string());
        
        assert!(
            !evaluate_conditions(&conditions, &log, false),
            "Should not match when any condition fails"
        );
    }

    #[test]
    fn test_evaluate_conditions_empty_conditions() {
        let log = create_test_log("#channel", Some("alice"));
        let conditions = HashMap::new();
        
        assert!(
            evaluate_conditions(&conditions, &log, true),
            "Should return true for empty conditions"
        );
    }

    // Tests for match_rule()

    #[test]
    fn test_match_rule_pm_basic() {
        use crate::rules::types::Rule;
        
        let log = create_test_log("alice", Some("alice"));
        let rule = Rule::Pm {
            only_if: None,
            not_if: None,
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should match PM rule when window equals nick"
        );
    }

    #[test]
    fn test_match_rule_pm_not_matching() {
        use crate::rules::types::Rule;
        
        let log = create_test_log("#channel", Some("alice"));
        let rule = Rule::Pm {
            only_if: None,
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match PM rule for channel message"
        );
    }

    #[test]
    fn test_match_rule_pm_with_only_if_pass() {
        use crate::rules::types::Rule;
        
        let mut log = create_test_log("alice", Some("alice"));
        log.network = Some("freenode".to_string());
        
        let mut only_if = HashMap::new();
        only_if.insert("network".to_string(), "freenode".to_string());
        
        let rule = Rule::Pm {
            only_if: Some(only_if),
            not_if: None,
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should match PM rule when only_if conditions pass"
        );
    }

    #[test]
    fn test_match_rule_pm_with_only_if_fail() {
        use crate::rules::types::Rule;
        
        let mut log = create_test_log("alice", Some("alice"));
        log.network = Some("freenode".to_string());
        
        let mut only_if = HashMap::new();
        only_if.insert("network".to_string(), "other".to_string());
        
        let rule = Rule::Pm {
            only_if: Some(only_if),
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match PM rule when only_if conditions fail"
        );
    }

    #[test]
    fn test_match_rule_pm_with_not_if_suppress() {
        use crate::rules::types::Rule;
        
        let mut log = create_test_log("alice", Some("alice"));
        log.network = Some("freenode".to_string());
        
        let mut not_if = HashMap::new();
        not_if.insert("network".to_string(), "freenode".to_string());
        
        let rule = Rule::Pm {
            only_if: None,
            not_if: Some(not_if),
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should suppress PM rule when not_if conditions all pass"
        );
    }

    #[test]
    fn test_match_rule_pm_with_not_if_no_suppress() {
        use crate::rules::types::Rule;
        
        let mut log = create_test_log("alice", Some("alice"));
        log.network = Some("freenode".to_string());
        
        let mut not_if = HashMap::new();
        not_if.insert("network".to_string(), "other".to_string());
        
        let rule = Rule::Pm {
            only_if: None,
            not_if: Some(not_if),
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should not suppress PM rule when not_if conditions fail"
        );
    }

    #[test]
    fn test_match_rule_substring_basic_case_insensitive() {
        use crate::rules::types::Rule;
        
        let log = create_log_with_message("Hello World");
        let rule = Rule::Substring {
            match_str: "hello".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should match substring case-insensitively"
        );
    }

    #[test]
    fn test_match_rule_substring_basic_case_sensitive() {
        use crate::rules::types::Rule;
        
        let log = create_log_with_message("Hello World");
        let rule = Rule::Substring {
            match_str: "hello".to_string(),
            case_sensitive: true,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match 'hello' in 'Hello World' case-sensitively"
        );
        
        let rule2 = Rule::Substring {
            match_str: "Hello".to_string(),
            case_sensitive: true,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            match_rule(&rule2, &log),
            "Should match 'Hello' in 'Hello World' case-sensitively"
        );
    }

    #[test]
    fn test_match_rule_substring_not_in_message() {
        use crate::rules::types::Rule;
        
        let log = create_log_with_message("Hello World");
        let rule = Rule::Substring {
            match_str: "goodbye".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match when substring not in message"
        );
    }

    #[test]
    fn test_match_rule_substring_none_message() {
        use crate::rules::types::Rule;
        
        let mut log = create_log_with_message("test");
        log.message = None;
        
        let rule = Rule::Substring {
            match_str: "test".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match when message is None"
        );
    }

    #[test]
    fn test_match_rule_substring_only_in_nick() {
        use crate::rules::types::Rule;
        
        let mut log = create_log_with_message("alice");
        log.nick = Some("alice".to_string());
        
        let rule = Rule::Substring {
            match_str: "alice".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match when match string appears only in nick"
        );
    }

    #[test]
    fn test_match_rule_substring_in_message_and_nick() {
        use crate::rules::types::Rule;
        
        let mut log = create_log_with_message("alice said hello");
        log.nick = Some("alice".to_string());
        
        let rule = Rule::Substring {
            match_str: "alice".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should match when match string appears in message beyond just nick"
        );
    }

    #[test]
    fn test_match_rule_substring_not_in_nick() {
        use crate::rules::types::Rule;
        
        let mut log = create_log_with_message("important update");
        log.nick = Some("bob".to_string());
        
        let rule = Rule::Substring {
            match_str: "important".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should match when match string is in message but not in nick"
        );
    }

    #[test]
    fn test_match_rule_substring_with_only_if_pass() {
        use crate::rules::types::Rule;
        
        let log = create_log_with_message("important update");
        
        let mut only_if = HashMap::new();
        only_if.insert("window".to_string(), "#channel".to_string());
        
        let rule = Rule::Substring {
            match_str: "important".to_string(),
            case_sensitive: false,
            only_if: Some(only_if),
            not_if: None,
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should match when substring matches and only_if conditions pass"
        );
    }

    #[test]
    fn test_match_rule_substring_with_only_if_fail() {
        use crate::rules::types::Rule;
        
        let log = create_log_with_message("important update");
        
        let mut only_if = HashMap::new();
        only_if.insert("window".to_string(), "#other".to_string());
        
        let rule = Rule::Substring {
            match_str: "important".to_string(),
            case_sensitive: false,
            only_if: Some(only_if),
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match when only_if conditions fail"
        );
    }

    #[test]
    fn test_match_rule_substring_with_not_if_suppress() {
        use crate::rules::types::Rule;
        
        let log = create_log_with_message("important update");
        
        let mut not_if = HashMap::new();
        not_if.insert("window".to_string(), "#channel".to_string());
        
        let rule = Rule::Substring {
            match_str: "important".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: Some(not_if),
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should suppress when not_if conditions all pass"
        );
    }

    #[test]
    fn test_match_rule_substring_with_not_if_no_suppress() {
        use crate::rules::types::Rule;
        
        let log = create_log_with_message("important update");
        
        let mut not_if = HashMap::new();
        not_if.insert("window".to_string(), "#other".to_string());
        
        let rule = Rule::Substring {
            match_str: "important".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: Some(not_if),
        };
        
        assert!(
            match_rule(&rule, &log),
            "Should not suppress when not_if conditions fail"
        );
    }

    #[test]
    fn test_match_rule_substring_case_sensitive_in_conditions() {
        use crate::rules::types::Rule;
        
        let mut log = create_log_with_message("Important update");
        log.window = "#Engineering".to_string();
        
        let mut only_if = HashMap::new();
        only_if.insert("window".to_string(), "#engineering".to_string());
        
        // Case-sensitive rule should not match due to window case mismatch
        let rule = Rule::Substring {
            match_str: "Important".to_string(),
            case_sensitive: true,
            only_if: Some(only_if.clone()),
            not_if: None,
        };
        
        assert!(
            !match_rule(&rule, &log),
            "Should not match when case_sensitive and only_if window case differs"
        );
        
        // Case-insensitive rule should match
        let rule2 = Rule::Substring {
            match_str: "important".to_string(),
            case_sensitive: false,
            only_if: Some(only_if),
            not_if: None,
        };
        
        assert!(
            match_rule(&rule2, &log),
            "Should match when case_insensitive and only_if conditions pass"
        );
    }
}
