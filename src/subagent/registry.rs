//! Loads `subagents/*.json` files at startup and provides lookup by name.

use std::collections::HashMap;
use std::path::Path;

use crate::error::MiniMaxError;

use super::types::{SubagentDef, SubagentSummary};

/// In-memory registry of all subagents found in the `subagents/` directory.
#[derive(Debug)]
pub struct SubagentRegistry {
    subagents: HashMap<String, SubagentDef>,
}

impl SubagentRegistry {
    /// Load every `*.json` file in `path` as a subagent definition.
    ///
    /// If the directory does not exist, returns an empty registry with a warning
    /// (so the MCP server can still start in dev setups that haven't created
    /// any subagents yet).
    pub fn load_from_dir(path: &Path) -> Result<Self, MiniMaxError> {
        let mut subagents: HashMap<String, SubagentDef> = HashMap::new();

        if !path.exists() {
            tracing::warn!(
                "subagents directory {:?} does not exist; starting with empty registry",
                path
            );
            return Ok(Self { subagents });
        }

        let entries = std::fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let content = std::fs::read_to_string(&entry_path).map_err(|e| {
                MiniMaxError::Config(format!(
                    "failed to read {}: {}",
                    entry_path.display(),
                    e
                ))
            })?;

            let def: SubagentDef = serde_json::from_str(&content).map_err(|e| {
                MiniMaxError::Config(format!(
                    "failed to parse {}: {}",
                    entry_path.display(),
                    e
                ))
            })?;

            def.validate().map_err(|e| {
                MiniMaxError::Config(format!(
                    "invalid subagent config in {}: {}",
                    entry_path.display(),
                    e
                ))
            })?;

            if let Some(_existing) = subagents.get(&def.name) {
                return Err(MiniMaxError::Config(format!(
                    "duplicate subagent name '{}' (in {} and {})",
                    def.name,
                    existing_source_hint(&subagents, &def.name),
                    entry_path.display()
                )));
            }

            subagents.insert(def.name.clone(), def);
        }

        let count = subagents.len();
        if count == 0 {
            tracing::warn!(
                "no subagents found in {:?}; the run_subagent tool will return errors until at least one is defined",
                path
            );
        } else {
            tracing::info!("loaded {} subagent(s) from {:?}", count, path);
        }

        Ok(Self { subagents })
    }

    /// Empty registry (used in tests).
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            subagents: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&SubagentDef> {
        self.subagents.get(name)
    }

    pub fn list(&self) -> Vec<SubagentSummary> {
        let mut out: Vec<SubagentSummary> = self
            .subagents
            .values()
            .map(|d| SubagentSummary {
                name: d.name.clone(),
                description: d.description.clone(),
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    #[allow(dead_code)]
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.subagents.keys().cloned().collect();
        names.sort();
        names
    }
}

/// Best-effort source hint for the duplicate-name error. We don't store the
/// source path on the def itself; this just reports "(earlier file)".
fn existing_source_hint(
    _subagents: &HashMap<String, SubagentDef>,
    _name: &str,
) -> &'static str {
    "(earlier file)"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_def(dir: &Path, filename: &str, json: &str) {
        let mut f = std::fs::File::create(dir.join(filename)).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    fn minimal_subagent_json(name: &str) -> String {
        format!(
            r#"{{
                "name": "{name}",
                "description": "test subagent",
                "system": "You are a test."
            }}"#
        )
    }

    #[test]
    fn load_empty_dir_returns_empty_registry() {
        let tmp = std::env::temp_dir().join(format!(
            "minimax_subagent_test_empty_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let r = SubagentRegistry::load_from_dir(&tmp).unwrap();
        assert!(r.list().is_empty());
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn load_missing_dir_returns_empty_registry_with_warning() {
        let path = std::path::PathBuf::from("/tmp/minimax_subagent_test_missing_xyz_does_not_exist");
        let _ = std::fs::remove_dir_all(&path);
        let r = SubagentRegistry::load_from_dir(&path).unwrap();
        assert!(r.list().is_empty());
    }

    #[test]
    fn load_valid_defs() {
        let tmp = std::env::temp_dir().join(format!(
            "minimax_subagent_test_valid_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        write_def(&tmp, "a.json", &minimal_subagent_json("alpha"));
        write_def(
            &tmp,
            "b.json",
            r#"{
                "name": "beta",
                "description": "second one",
                "system": "Beta system.",
                "max_iterations": 7,
                "allowed_tools": ["text_to_audio"]
            }"#,
        );
        // Non-JSON files are ignored
        write_def(&tmp, "readme.txt", "not a subagent");

        let r = SubagentRegistry::load_from_dir(&tmp).unwrap();
        assert_eq!(r.list().len(), 2);

        let a = r.get("alpha").expect("alpha");
        assert_eq!(a.system, "You are a test.");
        assert!(a.allowed_tools.is_none());

        let b = r.get("beta").expect("beta");
        assert_eq!(b.max_iterations, Some(7));
        assert_eq!(
            b.allowed_tools.as_ref().unwrap(),
            &vec!["text_to_audio".to_string()]
        );

        assert!(r.get("nope").is_none());
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn duplicate_name_is_rejected() {
        let tmp = std::env::temp_dir().join(format!(
            "minimax_subagent_test_dup_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        write_def(&tmp, "1.json", &minimal_subagent_json("dup"));
        write_def(&tmp, "2.json", &minimal_subagent_json("dup"));

        let err = SubagentRegistry::load_from_dir(&tmp).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("duplicate"),
            "expected duplicate error, got: {msg}"
        );

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let tmp = std::env::temp_dir().join(format!(
            "minimax_subagent_test_invalid_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        // All fields present, but `system` is empty → fails validate()
        write_def(
            &tmp,
            "bad.json",
            r#"{ "name": "x", "description": "y", "system": "" }"#,
        );

        let err = SubagentRegistry::load_from_dir(&tmp).unwrap_err();
        assert!(
            err.to_string().contains("invalid subagent config"),
            "got: {err}"
        );

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn completely_missing_field_fails_at_parse_time() {
        let tmp = std::env::temp_dir().join(format!(
            "minimax_subagent_test_missingsys_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        // Missing `system` entirely → serde fails before validate()
        write_def(
            &tmp,
            "bad.json",
            r#"{ "name": "x", "description": "y" }"#,
        );

        let err = SubagentRegistry::load_from_dir(&tmp).unwrap_err();
        assert!(
            err.to_string().contains("failed to parse"),
            "got: {err}"
        );

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn invalid_json_is_rejected() {
        let tmp = std::env::temp_dir().join(format!(
            "minimax_subagent_test_badjson_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&tmp).unwrap();

        write_def(&tmp, "bad.json", "{ not json");

        let err = SubagentRegistry::load_from_dir(&tmp).unwrap_err();
        assert!(err.to_string().contains("failed to parse"));

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn load_real_subagents_dir() {
        // Smoke test: load the project's own subagents/ dir
        let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let subagents_dir = project_root.join("subagents");
        let r = SubagentRegistry::load_from_dir(&subagents_dir)
            .expect("project subagents/ should load cleanly");
        let names = r.names();
        // At least the 4 example subagents should be present
        for expected in ["echo", "voice-greeter", "orchestrator", "worker"] {
            assert!(
                names.contains(&expected.to_string()),
                "expected subagent '{expected}' missing, loaded: {names:?}"
            );
        }
        // Each loaded subagent must pass validate()
        for def in r.list() {
            let full = r.get(&def.name).unwrap();
            full.validate().expect("validate");
        }
    }
}
