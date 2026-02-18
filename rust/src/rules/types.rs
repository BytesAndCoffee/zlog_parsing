// Rule types and enums

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Rule {
    Substring {
        #[serde(rename = "match")]
        match_str: String,
        #[serde(default)]
        case_sensitive: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        only_if: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        not_if: Option<HashMap<String, String>>,
    },
    Pm {
        #[serde(skip_serializing_if = "Option::is_none")]
        only_if: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        not_if: Option<HashMap<String, String>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_substring_rule_deserialization() {
        let json = r#"{"type":"substring","match":"test","case_sensitive":false}"#;
        let rule: Rule = serde_json::from_str(json).unwrap();
        
        match rule {
            Rule::Substring { match_str, case_sensitive, .. } => {
                assert_eq!(match_str, "test");
                assert_eq!(case_sensitive, false);
            }
            _ => panic!("Expected Substring variant"),
        }
    }

    #[test]
    fn test_pm_rule_deserialization() {
        let json = r#"{"type":"pm"}"#;
        let rule: Rule = serde_json::from_str(json).unwrap();
        
        match rule {
            Rule::Pm { .. } => {
                // Success
            }
            _ => panic!("Expected Pm variant"),
        }
    }

    #[test]
    fn test_substring_rule_with_conditions() {
        let json = r##"{
            "type":"substring",
            "match":"important",
            "case_sensitive":true,
            "only_if":{"window":"#engineering"}
        }"##;
        let rule: Rule = serde_json::from_str(json).unwrap();
        
        match rule {
            Rule::Substring { match_str, case_sensitive, only_if, .. } => {
                assert_eq!(match_str, "important");
                assert_eq!(case_sensitive, true);
                assert!(only_if.is_some());
                let conditions = only_if.unwrap();
                assert_eq!(conditions.get("window"), Some(&"#engineering".to_string()));
            }
            _ => panic!("Expected Substring variant"),
        }
    }

    #[test]
    fn test_rule_serialization() {
        let rule = Rule::Substring {
            match_str: "test".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains(r#""type":"substring"#));
        assert!(json.contains(r#""match":"test"#));
    }

    #[test]
    fn test_case_sensitive_default() {
        // When case_sensitive is not provided, it should default to false
        let json = r#"{"type":"substring","match":"test"}"#;
        let rule: Rule = serde_json::from_str(json).unwrap();
        
        match rule {
            Rule::Substring { case_sensitive, .. } => {
                assert_eq!(case_sensitive, false);
            }
            _ => panic!("Expected Substring variant"),
        }
    }
}
