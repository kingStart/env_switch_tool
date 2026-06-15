use envtools_domain::error::DomainError;
use envtools_domain::model::env_group::EnvGroup;
use envtools_domain::model::env_variable::{EnvVariable, PathMode};
use envtools_domain::model::priority::Priority;
use envtools_domain::repository::GroupRepository;

/// Portable exchange format for import/export (JSON-based).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportData {
    pub version: u32,
    pub groups: Vec<ExportGroup>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportGroup {
    pub name: String,
    pub description: String,
    pub active: bool,
    pub priority: u32,
    pub variables: Vec<ExportVariable>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportVariable {
    pub key: String,
    pub value: String,
    pub path_mode: String,
}

pub struct ExportImportUseCase<'a> {
    repo: &'a dyn GroupRepository,
}

impl<'a> ExportImportUseCase<'a> {
    pub fn new(repo: &'a dyn GroupRepository) -> Self {
        Self { repo }
    }

    /// Export all groups (or specific ones) to portable JSON format.
    pub fn export(&self, filter: Option<&[String]>) -> Result<ExportData, DomainError> {
        let all_groups = self.repo.find_all()?;
        let groups: Vec<EnvGroup> = match filter {
            Some(names) => all_groups
                .into_iter()
                .filter(|g| names.iter().any(|n| n == g.name()))
                .collect(),
            None => all_groups,
        };

        let export_groups = groups
            .iter()
            .map(|g| ExportGroup {
                name: g.name().to_string(),
                description: g.description().to_string(),
                active: g.is_active(),
                priority: g.priority().value(),
                variables: g
                    .variables()
                    .iter()
                    .map(|v| ExportVariable {
                        key: v.key().to_string(),
                        value: v.value().to_string(),
                        path_mode: match v.path_mode() {
                            PathMode::Override => "override".to_string(),
                            PathMode::Prepend => "prepend".to_string(),
                            PathMode::Append => "append".to_string(),
                        },
                    })
                    .collect(),
            })
            .collect();

        Ok(ExportData {
            version: 1,
            groups: export_groups,
        })
    }

    /// Import groups from JSON data. Strategy: merge (skip existing) or overwrite.
    pub fn import(&self, data: &ExportData, overwrite: bool) -> Result<ImportResult, DomainError> {
        let mut imported = 0u32;
        let mut skipped = 0u32;
        let mut overwritten = 0u32;

        for eg in &data.groups {
            let exists = self.repo.exists(&eg.name)?;

            if exists && !overwrite {
                skipped += 1;
                continue;
            }

            let variables: Vec<EnvVariable> = eg
                .variables
                .iter()
                .filter_map(|v| {
                    let mode = match v.path_mode.as_str() {
                        "prepend" => PathMode::Prepend,
                        "append" => PathMode::Append,
                        _ => PathMode::Override,
                    };
                    EnvVariable::with_path_mode(&v.key, &v.value, mode).ok()
                })
                .collect();

            let group = EnvGroup::from_state(
                eg.name.clone(),
                eg.description.clone(),
                variables,
                eg.active,
                Priority::new(eg.priority),
            );

            self.repo.save(&group)?;

            if exists {
                overwritten += 1;
            } else {
                imported += 1;
            }
        }

        Ok(ImportResult {
            imported,
            skipped,
            overwritten,
        })
    }
}

#[derive(Debug)]
pub struct ImportResult {
    pub imported: u32,
    pub skipped: u32,
    pub overwritten: u32,
}
