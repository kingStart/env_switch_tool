use std::collections::HashMap;

use crate::model::env_group::EnvGroup;
use crate::model::env_variable::PathMode;

/// Resolved environment: the final set of key-value pairs after merging all active groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnvironment {
    pub variables: HashMap<String, String>,
    /// Keys managed by envtools (for cleanup/unset tracking).
    pub managed_keys: Vec<String>,
}

/// Domain Service: resolves conflicts across multiple active groups
/// and computes the final merged environment snapshot.
pub struct GroupPolicy;

impl GroupPolicy {
    /// Merge all active groups into a single resolved environment.
    /// Conflict resolution: higher priority wins.
    /// PATH-like variables are merged with prepend/append semantics.
    pub fn resolve(groups: &[&EnvGroup], path_separator: &str) -> ResolvedEnvironment {
        let mut sorted: Vec<&&EnvGroup> = groups.iter().collect();
        // Sort by priority ascending so higher priority overwrites later
        sorted.sort_by_key(|g| g.priority());

        let mut result: HashMap<String, String> = HashMap::new();
        let mut prepends: HashMap<String, Vec<String>> = HashMap::new();
        let mut appends: HashMap<String, Vec<String>> = HashMap::new();

        for group in &sorted {
            for var in group.variables() {
                match var.path_mode() {
                    PathMode::Override => {
                        result.insert(var.key().to_string(), var.value().to_string());
                    }
                    PathMode::Prepend => {
                        prepends
                            .entry(var.key().to_string())
                            .or_default()
                            .push(var.value().to_string());
                    }
                    PathMode::Append => {
                        appends
                            .entry(var.key().to_string())
                            .or_default()
                            .push(var.value().to_string());
                    }
                }
            }
        }

        // Merge prepend/append into result
        for (key, parts) in &prepends {
            let prepend_str = parts.join(path_separator);
            let existing = result
                .get(key)
                .cloned()
                .unwrap_or_else(|| format!("${{{key}}}"));
            result.insert(
                key.clone(),
                format!("{prepend_str}{path_separator}{existing}"),
            );
        }
        for (key, parts) in &appends {
            let append_str = parts.join(path_separator);
            let existing = result
                .get(key)
                .cloned()
                .unwrap_or_else(|| format!("${{{key}}}"));
            result.insert(
                key.clone(),
                format!("{existing}{path_separator}{append_str}"),
            );
        }

        let managed_keys: Vec<String> = result.keys().cloned().collect();

        ResolvedEnvironment {
            variables: result,
            managed_keys,
        }
    }

    /// Determine which keys need to be unset when transitioning from old to new resolved state.
    pub fn diff_unset_keys(
        old: &ResolvedEnvironment,
        new: &ResolvedEnvironment,
    ) -> Vec<String> {
        old.managed_keys
            .iter()
            .filter(|k| !new.variables.contains_key(*k))
            .cloned()
            .collect()
    }
}
