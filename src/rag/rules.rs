//! Rule-based system support for RAG pipelines.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rule for rule-based systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Rule ID
    pub id: String,

    /// Rule name
    pub name: String,

    /// Rule conditions (key-value pairs)
    pub conditions: HashMap<String, RuleCondition>,

    /// Rule actions
    pub actions: Vec<RuleAction>,

    /// Rule priority (higher = more priority)
    pub priority: i32,

    /// Is rule enabled
    pub enabled: bool,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Exact match
    Equals(String),

    /// Contains substring
    Contains(String),

    /// Matches regex pattern
    Matches(String),

    /// Numeric comparison
    GreaterThan(f64),
    LessThan(f64),

    /// In list
    In(Vec<String>),

    /// Custom condition (serialized as string)
    Custom(String),
}

/// Rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Set a value
    SetValue { key: String, value: String },

    /// Add to list
    AddToList { key: String, value: String },

    /// Log message
    Log { level: String, message: String },

    /// Call function (by name)
    CallFunction { function: String, args: Vec<String> },

    /// Stop rule processing
    Stop,
}

/// Rule engine for rule-based systems
pub struct RuleEngine {
    rules: Vec<Rule>,
    context: HashMap<String, String>,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            context: HashMap::new(),
        }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
    }

    /// Set context value
    pub fn set_context(&mut self, key: String, value: String) {
        self.context.insert(key, value);
    }

    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }

    /// Evaluate a rule condition
    fn evaluate_condition(&self, key: &str, condition: &RuleCondition) -> bool {
        let value = match self.context.get(key) {
            Some(v) => v,
            None => return false,
        };

        match condition {
            RuleCondition::Equals(expected) => value == expected,
            RuleCondition::Contains(substring) => value.contains(substring),
            RuleCondition::Matches(pattern) => {
                // Simple pattern matching (could use regex crate for more complex patterns)
                value.contains(pattern)
            }
            RuleCondition::GreaterThan(threshold) => value
                .parse::<f64>()
                .map(|v| v > *threshold)
                .unwrap_or(false),
            RuleCondition::LessThan(threshold) => value
                .parse::<f64>()
                .map(|v| v < *threshold)
                .unwrap_or(false),
            RuleCondition::In(list) => list.contains(value),
            RuleCondition::Custom(_) => {
                // Custom conditions would need specialized handling
                false
            }
        }
    }

    /// Execute rule actions
    fn execute_actions(&mut self, actions: &[RuleAction]) -> Result<bool> {
        for action in actions {
            match action {
                RuleAction::SetValue { key, value } => {
                    self.context.insert(key.clone(), value.clone());
                }
                RuleAction::AddToList { key, value } => {
                    let current = self.context.get(key).cloned().unwrap_or_default();
                    let new_value = if current.is_empty() {
                        value.clone()
                    } else {
                        format!("{},{}", current, value)
                    };
                    self.context.insert(key.clone(), new_value);
                }
                RuleAction::Log {
                    level: _,
                    message: _,
                } => {
                    // Logging would be handled by tracing/logging framework
                }
                RuleAction::CallFunction {
                    function: _,
                    args: _,
                } => {
                    // Function calls would need to be handled by caller
                }
                RuleAction::Stop => {
                    return Ok(true); // Stop processing
                }
            }
        }
        Ok(false) // Continue processing
    }

    /// Execute all matching rules
    pub fn execute(&mut self) -> Result<Vec<String>> {
        let mut executed_rules = Vec::new();

        for rule in &self.rules.clone() {
            if !rule.enabled {
                continue;
            }

            // Check all conditions
            let all_conditions_met = rule
                .conditions
                .iter()
                .all(|(key, condition)| self.evaluate_condition(key, condition));

            if all_conditions_met {
                executed_rules.push(rule.id.clone());
                let should_stop = self.execute_actions(&rule.actions)?;
                if should_stop {
                    break;
                }
            }
        }

        Ok(executed_rules)
    }

    /// Clear all rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[Rule] {
        &self.rules
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(id: &str, key: &str, cond: RuleCondition, actions: Vec<RuleAction>) -> Rule {
        let mut conditions = HashMap::new();
        conditions.insert(key.to_string(), cond);
        Rule {
            id: id.to_string(),
            name: format!("rule_{}", id),
            conditions,
            actions,
            priority: 0,
            enabled: true,
        }
    }

    #[test]
    fn test_rule_engine_default() {
        let engine = RuleEngine::default();
        assert!(engine.get_rules().is_empty());
    }

    #[test]
    fn test_rule_engine_set_get_context() {
        let mut engine = RuleEngine::new();
        engine.set_context("key".to_string(), "value".to_string());
        assert_eq!(engine.get_context("key"), Some(&"value".to_string()));
        assert_eq!(engine.get_context("missing"), None);
    }

    #[test]
    fn test_evaluate_condition_equals() {
        let mut engine = RuleEngine::new();
        engine.set_context("status".to_string(), "active".to_string());

        let rule = make_rule(
            "r1",
            "status",
            RuleCondition::Equals("active".to_string()),
            vec![RuleAction::SetValue {
                key: "result".to_string(),
                value: "matched".to_string(),
            }],
        );
        engine.add_rule(rule);
        let executed = engine.execute().unwrap();
        assert_eq!(executed, vec!["r1"]);
        assert_eq!(engine.get_context("result"), Some(&"matched".to_string()));
    }

    #[test]
    fn test_evaluate_condition_contains() {
        let mut engine = RuleEngine::new();
        engine.set_context("text".to_string(), "hello world".to_string());

        let rule = make_rule(
            "r1",
            "text",
            RuleCondition::Contains("world".to_string()),
            vec![],
        );
        engine.add_rule(rule);
        let executed = engine.execute().unwrap();
        assert_eq!(executed, vec!["r1"]);
    }

    #[test]
    fn test_evaluate_condition_matches() {
        let mut engine = RuleEngine::new();
        engine.set_context("name".to_string(), "test-model-v2".to_string());

        let rule = make_rule(
            "r1",
            "name",
            RuleCondition::Matches("model".to_string()),
            vec![],
        );
        engine.add_rule(rule);
        assert_eq!(engine.execute().unwrap(), vec!["r1"]);
    }

    #[test]
    fn test_evaluate_condition_greater_than() {
        let mut engine = RuleEngine::new();
        engine.set_context("score".to_string(), "85.5".to_string());

        let rule = make_rule("r1", "score", RuleCondition::GreaterThan(80.0), vec![]);
        engine.add_rule(rule);
        assert_eq!(engine.execute().unwrap(), vec!["r1"]);
    }

    #[test]
    fn test_evaluate_condition_less_than() {
        let mut engine = RuleEngine::new();
        engine.set_context("latency".to_string(), "50".to_string());

        let rule = make_rule("r1", "latency", RuleCondition::LessThan(100.0), vec![]);
        engine.add_rule(rule);
        assert_eq!(engine.execute().unwrap(), vec!["r1"]);
    }

    #[test]
    fn test_evaluate_condition_in_list() {
        let mut engine = RuleEngine::new();
        engine.set_context("format".to_string(), "onnx".to_string());

        let rule = make_rule(
            "r1",
            "format",
            RuleCondition::In(vec![
                "safetensors".to_string(),
                "onnx".to_string(),
                "gguf".to_string(),
            ]),
            vec![],
        );
        engine.add_rule(rule);
        assert_eq!(engine.execute().unwrap(), vec!["r1"]);
    }

    #[test]
    fn test_evaluate_condition_custom() {
        let mut engine = RuleEngine::new();
        engine.set_context("x".to_string(), "y".to_string());

        let rule = make_rule(
            "r1",
            "x",
            RuleCondition::Custom("custom_fn".to_string()),
            vec![],
        );
        engine.add_rule(rule);
        // Custom always returns false
        assert!(engine.execute().unwrap().is_empty());
    }

    #[test]
    fn test_action_add_to_list() {
        let mut engine = RuleEngine::new();
        engine.set_context("status".to_string(), "go".to_string());

        let rule = make_rule(
            "r1",
            "status",
            RuleCondition::Equals("go".to_string()),
            vec![
                RuleAction::AddToList {
                    key: "log".to_string(),
                    value: "first".to_string(),
                },
                RuleAction::AddToList {
                    key: "log".to_string(),
                    value: "second".to_string(),
                },
            ],
        );
        engine.add_rule(rule);
        engine.execute().unwrap();
        assert_eq!(engine.get_context("log"), Some(&"first,second".to_string()));
    }

    #[test]
    fn test_action_log_and_call_function() {
        let mut engine = RuleEngine::new();
        engine.set_context("x".to_string(), "y".to_string());

        let rule = make_rule(
            "r1",
            "x",
            RuleCondition::Equals("y".to_string()),
            vec![
                RuleAction::Log {
                    level: "info".to_string(),
                    message: "matched".to_string(),
                },
                RuleAction::CallFunction {
                    function: "noop".to_string(),
                    args: vec![],
                },
            ],
        );
        engine.add_rule(rule);
        assert_eq!(engine.execute().unwrap(), vec!["r1"]);
    }

    #[test]
    fn test_action_stop() {
        let mut engine = RuleEngine::new();
        engine.set_context("a".to_string(), "1".to_string());

        let mut rule1 = make_rule(
            "r1",
            "a",
            RuleCondition::Equals("1".to_string()),
            vec![RuleAction::Stop],
        );
        rule1.priority = 10;

        let mut rule2 = make_rule("r2", "a", RuleCondition::Equals("1".to_string()), vec![]);
        rule2.priority = 5;

        engine.add_rule(rule1);
        engine.add_rule(rule2);

        // Only r1 should execute since it has Stop
        let executed = engine.execute().unwrap();
        assert_eq!(executed, vec!["r1"]);
    }

    #[test]
    fn test_disabled_rule_skipped() {
        let mut engine = RuleEngine::new();
        engine.set_context("x".to_string(), "y".to_string());

        let mut rule = make_rule("r1", "x", RuleCondition::Equals("y".to_string()), vec![]);
        rule.enabled = false;
        engine.add_rule(rule);

        assert!(engine.execute().unwrap().is_empty());
    }

    #[test]
    fn test_clear_rules() {
        let mut engine = RuleEngine::new();
        engine.add_rule(make_rule(
            "r1",
            "x",
            RuleCondition::Equals("y".to_string()),
            vec![],
        ));
        engine.clear_rules();
        assert!(engine.get_rules().is_empty());
    }

    #[test]
    fn test_missing_context_key() {
        let mut engine = RuleEngine::new();
        // No context set — condition on "missing_key" should fail
        let rule = make_rule(
            "r1",
            "missing_key",
            RuleCondition::Equals("val".to_string()),
            vec![],
        );
        engine.add_rule(rule);
        assert!(engine.execute().unwrap().is_empty());
    }

    #[test]
    fn test_priority_ordering() {
        let mut engine = RuleEngine::new();
        engine.set_context("x".to_string(), "1".to_string());

        let mut r1 = make_rule("low", "x", RuleCondition::Equals("1".to_string()), vec![]);
        r1.priority = 1;
        let mut r2 = make_rule("high", "x", RuleCondition::Equals("1".to_string()), vec![]);
        r2.priority = 10;

        engine.add_rule(r1);
        engine.add_rule(r2);

        let rules = engine.get_rules();
        assert_eq!(rules[0].id, "high");
        assert_eq!(rules[1].id, "low");
    }
}
