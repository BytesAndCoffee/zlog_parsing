// Rule validation logic

use crate::error::ValidationError;
use super::types::Rule;

/// Validate a single rule
/// 
/// This function validates that a rule is well-formed and contains all required fields.
/// Most validation is handled by serde during deserialization, but this function
/// provides additional semantic validation.
/// 
/// # Arguments
/// * `rule` - The rule to validate
/// 
/// # Returns
/// * `Ok(())` if the rule is valid
/// * `Err(ValidationError)` if the rule is invalid
/// 
/// # Requirements
/// - 4.1: Validates that each rule contains a "type" field (handled by serde)
/// - 4.2: For substring rules, validates "match" field exists (handled by serde)
/// - 4.3: For PM rules, accepts without "match" field (handled by serde)
/// - 4.4: Validates case_sensitive is bool (handled by serde)
/// - 4.5: Validates only_if and not_if are objects (handled by serde)
/// - 4.6: Rejects unsupported rule types (handled by serde)
pub fn validate_rule(rule: &Rule) -> Result<(), ValidationError> {
    match rule {
        Rule::Substring { match_str, .. } => {
            // Validate that match string is not empty
            if match_str.is_empty() {
                return Err(ValidationError::InvalidRule(
                    "Substring rule 'match' field cannot be empty".to_string()
                ));
            }
            Ok(())
        }
        Rule::Pm { .. } => {
            // PM rules don't require additional validation
            Ok(())
        }
    }
}

/// Validate a list of rules
/// 
/// This function validates all rules in a list and returns an error if any rule is invalid.
/// 
/// # Arguments
/// * `rules` - The list of rules to validate
/// 
/// # Returns
/// * `Ok(())` if all rules are valid
/// * `Err(ValidationError)` if any rule is invalid
pub fn validate_rules(rules: &[Rule]) -> Result<(), ValidationError> {
    for (index, rule) in rules.iter().enumerate() {
        validate_rule(rule).map_err(|e| {
            ValidationError::InvalidRule(format!("Rule at index {}: {}", index, e))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validate_substring_rule_valid() {
        let rule = Rule::Substring {
            match_str: "test".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        assert!(validate_rule(&rule).is_ok());
    }

    #[test]
    fn test_validate_substring_rule_empty_match() {
        let rule = Rule::Substring {
            match_str: "".to_string(),
            case_sensitive: false,
            only_if: None,
            not_if: None,
        };
        
        let result = validate_rule(&rule);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_pm_rule_valid() {
        let rule = Rule::Pm {
            only_if: None,
            not_if: None,
        };
        
        assert!(validate_rule(&rule).is_ok());
    }

    #[test]
    fn test_validate_substring_rule_with_conditions() {
        let mut only_if = HashMap::new();
        only_if.insert("window".to_string(), "#engineering".to_string());
        
        let rule = Rule::Substring {
            match_str: "important".to_string(),
            case_sensitive: true,
            only_if: Some(only_if),
            not_if: None,
        };
        
        assert!(validate_rule(&rule).is_ok());
    }

    #[test]
    fn test_validate_rules_all_valid() {
        let rules = vec![
            Rule::Substring {
                match_str: "test1".to_string(),
                case_sensitive: false,
                only_if: None,
                not_if: None,
            },
            Rule::Pm {
                only_if: None,
                not_if: None,
            },
            Rule::Substring {
                match_str: "test2".to_string(),
                case_sensitive: true,
                only_if: None,
                not_if: None,
            },
        ];
        
        assert!(validate_rules(&rules).is_ok());
    }

    #[test]
    fn test_validate_rules_with_invalid_rule() {
        let rules = vec![
            Rule::Substring {
                match_str: "test1".to_string(),
                case_sensitive: false,
                only_if: None,
                not_if: None,
            },
            Rule::Substring {
                match_str: "".to_string(),  // Invalid: empty match string
                case_sensitive: false,
                only_if: None,
                not_if: None,
            },
            Rule::Pm {
                only_if: None,
                not_if: None,
            },
        ];
        
        let result = validate_rules(&rules);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("index 1"));
        assert!(err_msg.contains("cannot be empty"));
    }
}
