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
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
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
