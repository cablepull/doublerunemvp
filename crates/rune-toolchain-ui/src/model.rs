use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema: String,
    pub name: String,
    pub initial: Vec<Expectation>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub tap: String,
    pub expect: Vec<Expectation>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expectation {
    Text { key: String, value: String },
    Present { key: String },
    Absent { key: String },
}

impl Scenario {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "double-rune.scenario/1" {
            return Err("unknown scenario schema".into());
        }
        if self.name.is_empty()
            || self.name.len() > 80
            || !self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid scenario name".into());
        }
        if self.steps.is_empty() || self.steps.len() > 16 {
            return Err("expected 1..=16 actions".into());
        }
        if self.initial.is_empty()
            || self
                .steps
                .iter()
                .any(|s| s.expect.is_empty() || s.tap.is_empty())
        {
            return Err("every observation needs explicit postconditions".into());
        }
        Ok(())
    }
}

fn matching<'a>(tree: &'a Value, key: &str) -> Result<Vec<&'a Value>, String> {
    Ok(tree["elements"]
        .as_array()
        .ok_or("missing element tree")?
        .iter()
        .filter(|e| e["key"] == key && e["exiting"] != true)
        .collect())
}

pub(crate) fn action_target(tree: &Value, key: &str) -> Result<(), String> {
    let elements = matching(tree, key)?;
    if elements.len() != 1 {
        return Err(format!("action target {key} missing or ambiguous"));
    }
    let e = elements[0];
    if e["role"] != "button"
        || e["inert"] == true
        || e["opacity"].as_f64().unwrap_or(0.0) <= 0.0
        || e["rect"][2].as_f64().unwrap_or(0.0) <= 0.0
        || e["rect"][3].as_f64().unwrap_or(0.0) <= 0.0
    {
        return Err(format!(
            "action target {key} is not an active visible button"
        ));
    }
    Ok(())
}

pub(crate) fn assert_tree(tree: &Value, expectations: &[Expectation]) -> Result<(), String> {
    for expected in expectations {
        let (key, absent) = match expected {
            Expectation::Text { key, .. } | Expectation::Present { key } => (key, false),
            Expectation::Absent { key } => (key, true),
        };
        let found = matching(tree, key)?;
        if absent {
            if !found.is_empty() {
                return Err(format!("expected {key} to be absent"));
            }
        } else {
            if found.len() != 1 {
                return Err(format!("expected one element {key}, found {}", found.len()));
            }
            if let Expectation::Text { value, .. } = expected {
                if found[0]["text"] != *value {
                    return Err(format!(
                        "{key}: expected {value:?}, observed {}",
                        found[0]["text"]
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
