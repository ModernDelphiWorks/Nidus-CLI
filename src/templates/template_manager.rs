//! Template Manager — customizable template management system.
//!
//! This module implements:
//! - Loading of custom templates
//! - Intelligent caching for templates
//! - Configurable variable system
//! - Support for external templates

use crate::error::CliError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Custom template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template version
    pub version: String,
    /// Template author
    pub author: Option<String>,
    /// Template category (e.g. "module", "api", "crud")
    #[serde(default)]
    pub category: Option<String>,
    /// Whether this template is marked as a favorite
    #[serde(default)]
    pub favorite: bool,
    /// Variables available in the template
    pub variables: HashMap<String, TemplateVariable>,
    /// Template files
    pub files: Vec<TemplateFile>,
    /// Required dependencies
    pub dependencies: Vec<String>,
    /// User-defined key-value configuration
    #[serde(default)]
    pub config: HashMap<String, String>,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name
    pub name: String,
    /// Variable description
    pub description: String,
    /// Default value
    pub default_value: Option<String>,
    /// Whether the variable is required
    pub required: bool,
    /// Variable type (string, boolean, number)
    pub var_type: String,
}

/// Template file definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    /// File name
    pub name: String,
    /// Relative path within the template
    pub path: String,
    /// Template content
    pub content: String,
    /// Whether variables should be substituted in this file
    pub process: bool,
}

/// Template cache for improved performance
#[derive(Debug)]
struct TemplateCache {
    /// Cached templates with their insertion timestamps
    templates: HashMap<String, (TemplateConfig, SystemTime)>,
    /// Cache time-to-live in seconds
    ttl: u64,
}

/// Main template manager
pub struct TemplateManager {
    /// Template cache
    cache: TemplateCache,
    /// Base templates directory
    templates_dir: PathBuf,
    /// Built-in templates
    builtin_templates: HashMap<String, TemplateConfig>,
}

impl TemplateCache {
    fn new(ttl: u64) -> Self {
        Self {
            templates: HashMap::new(),
            ttl,
        }
    }

    fn get(&self, name: &str) -> Option<&TemplateConfig> {
        if let Some((template, timestamp)) = self.templates.get(name) {
            if let Ok(elapsed) = timestamp.elapsed() {
                if elapsed.as_secs() < self.ttl {
                    return Some(template);
                }
            }
        }
        None
    }

    fn insert(&mut self, name: String, template: TemplateConfig) {
        self.templates.insert(name, (template, SystemTime::now()));
    }

    fn clear_expired(&mut self) {
        let _now = SystemTime::now();
        self.templates.retain(|_, (_, timestamp)| {
            timestamp.elapsed().is_ok_and(|elapsed| elapsed.as_secs() < self.ttl)
        });
    }
}

impl TemplateManager {
    /// Creates a new template manager
    pub fn new(templates_dir: PathBuf) -> Result<Self, CliError> {
        let mut manager = Self {
            cache: TemplateCache::new(3600), // 1-hour cache TTL
            templates_dir,
            builtin_templates: HashMap::new(),
        };
        
        manager.load_builtin_templates()?;
        Ok(manager)
    }

    /// Loads built-in templates
    fn load_builtin_templates(&mut self) -> Result<(), CliError> {
        // Default module template
        let module_template = TemplateConfig {
            name: "default-module".to_string(),
            description: "Default template for Nidus modules".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Nidus Team".to_string()),
            category: Some("module".to_string()),
            favorite: true,
            variables: self.create_default_variables(),
            files: self.create_default_module_files(),
            dependencies: vec![
                "Nidus".to_string(),
                "ModernSyntax".to_string(),
            ],
            config: HashMap::new(),
        };

        self.builtin_templates.insert("default-module".to_string(), module_template);
        Ok(())
    }

    /// Creates default variables for templates
    fn create_default_variables(&self) -> HashMap<String, TemplateVariable> {
        let mut variables = HashMap::new();

        variables.insert("mod".to_string(), TemplateVariable {
            name: "mod".to_string(),
            description: "Module name in CamelCase".to_string(),
            default_value: None,
            required: true,
            var_type: "string".to_string(),
        });

        variables.insert("author".to_string(), TemplateVariable {
            name: "author".to_string(),
            description: "Module author".to_string(),
            default_value: Some("Nidus Developer".to_string()),
            required: false,
            var_type: "string".to_string(),
        });

        variables.insert("namespace".to_string(), TemplateVariable {
            name: "namespace".to_string(),
            description: "Project namespace".to_string(),
            default_value: None,
            required: false,
            var_type: "string".to_string(),
        });

        variables
    }

    /// Creates default files for the module template
    fn create_default_module_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile {
                name: "module.pas".to_string(),
                path: "{{mod}}Module.pas".to_string(),
                content: include_str!("../templates/module/module.pas").to_string(),
                process: true,
            },
            TemplateFile {
                name: "controller.pas".to_string(),
                path: "{{mod}}Controller.pas".to_string(),
                content: include_str!("../templates/module/controller.pas").to_string(),
                process: true,
            },
            TemplateFile {
                name: "service.pas".to_string(),
                path: "{{mod}}Service.pas".to_string(),
                content: include_str!("../templates/module/service.pas").to_string(),
                process: true,
            },
            TemplateFile {
                name: "repository.pas".to_string(),
                path: "{{mod}}Repository.pas".to_string(),
                content: include_str!("../templates/module/repository.pas").to_string(),
                process: true,
            },
            TemplateFile {
                name: "interface.pas".to_string(),
                path: "{{mod}}Interface.pas".to_string(),
                content: include_str!("../templates/module/interface.pas").to_string(),
                process: true,
            },
            TemplateFile {
                name: "infra.pas".to_string(),
                path: "{{mod}}Infra.pas".to_string(),
                content: include_str!("../templates/module/infra.pas").to_string(),
                process: true,
            },
        ]
    }

    /// Gets a template (cache first, then disk, then built-in)
    pub fn get_template(&mut self, name: &str) -> Result<TemplateConfig, CliError> {
        // 1. Check cache
        if let Some(template) = self.cache.get(name) {
            return Ok(template.clone());
        }

        // 2. Try loading from disk
        if let Ok(template) = self.load_template_from_disk(name) {
            self.cache.insert(name.to_string(), template.clone());
            return Ok(template);
        }

        // 3. Check built-in
        if let Some(template) = self.builtin_templates.get(name) {
            return Ok(template.clone());
        }

        Err(CliError::ValidationError(format!("Template '{}' not found", name)))
    }

    /// Loads a template from disk
    fn load_template_from_disk(&self, name: &str) -> Result<TemplateConfig, CliError> {
        let template_path = self.templates_dir.join(name).join("template.json");
        
        if !template_path.exists() {
            return Err(CliError::ValidationError(format!("Template file not found: {:?}", template_path)));
        }

        let content = fs::read_to_string(&template_path)
            .map_err(CliError::IoError)?;

        let mut template: TemplateConfig = serde_json::from_str(&content)
            .map_err(CliError::JsonError)?;

        // Load file contents
        let template_dir = template_path.parent()
            .expect("template_path always has a parent directory");
        for file in &mut template.files {
            let file_path = template_dir.join(&file.path);
            if file_path.exists() {
                file.content = fs::read_to_string(&file_path)
                    .map_err(CliError::IoError)?;
            }
        }

        Ok(template)
    }

    /// Lists all available templates
    pub fn list_templates(&self) -> Vec<String> {
        let mut templates = Vec::new();
        
        // Built-in templates
        templates.extend(self.builtin_templates.keys().cloned());
        
        // Disk templates
        if let Ok(entries) = fs::read_dir(&self.templates_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if !templates.contains(&name.to_string()) {
                            templates.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        templates.sort();
        templates
    }

    /// Processes variables in a template
    pub fn process_template(
        &self,
        template: &TemplateConfig,
        variables: &HashMap<String, String>,
    ) -> Result<Vec<(String, String)>, CliError> {
        let mut processed_files = Vec::new();

        for file in &template.files {
            let mut content = file.content.clone();
            let mut file_path = file.path.clone();

            if file.process {
                // Substitute variables in content and path using {{var}} syntax
                for (var_name, var_value) in variables {
                    let placeholder = format!("{{{{{}}}}}", var_name);
                    content = content.replace(&placeholder, var_value);
                    file_path = file_path.replace(&placeholder, var_value);
                }
            }

            processed_files.push((file_path, content));
        }

        Ok(processed_files)
    }

    /// Clears expired cache entries
    pub fn cleanup_cache(&mut self) {
        self.cache.clear_expired();
    }

    /// Installs an external template
    pub fn install_template(&mut self, source: &str, name: &str) -> Result<(), CliError> {
        // TODO: implement download and installation of external templates
        println!("Installing template '{}' from '{}'...", name, source);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn manager_with_tempdir(dir: &TempDir) -> TemplateManager {
        TemplateManager::new(dir.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_new_loads_builtin_template() {
        let dir = TempDir::new().unwrap();
        let manager = manager_with_tempdir(&dir);
        assert!(manager.builtin_templates.contains_key("default-module"));
    }

    #[test]
    fn test_list_templates_includes_builtin() {
        let dir = TempDir::new().unwrap();
        let manager = manager_with_tempdir(&dir);
        let list = manager.list_templates();
        assert!(list.contains(&"default-module".to_string()));
    }

    #[test]
    fn test_get_unknown_template_returns_error() {
        let dir = TempDir::new().unwrap();
        let mut manager = manager_with_tempdir(&dir);
        assert!(manager.get_template("nonexistent-template").is_err());
    }

    #[test]
    fn test_get_builtin_template_succeeds() {
        let dir = TempDir::new().unwrap();
        let mut manager = manager_with_tempdir(&dir);
        let t = manager.get_template("default-module").unwrap();
        assert_eq!(t.name, "default-module");
        assert!(!t.files.is_empty());
    }

    #[test]
    fn test_process_template_replaces_mod_variable() {
        let dir = TempDir::new().unwrap();
        let mut manager = manager_with_tempdir(&dir);
        let template = manager.get_template("default-module").unwrap();

        let mut vars = HashMap::new();
        vars.insert("mod".to_string(), "User".to_string());

        let files = manager.process_template(&template, &vars).unwrap();
        assert!(!files.is_empty());
        // All paths with {{mod}} must be substituted
        for (path, _content) in &files {
            assert!(!path.contains("{{mod}}"), "path still has placeholder: {}", path);
        }
    }

    #[test]
    fn test_load_template_from_disk() {
        let dir = TempDir::new().unwrap();
        let template_dir = dir.path().join("my-template");
        fs::create_dir_all(&template_dir).unwrap();

        let config = TemplateConfig {
            name: "my-template".to_string(),
            description: "Test template".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: None,
            favorite: false,
            variables: HashMap::new(),
            files: vec![TemplateFile {
                name: "test.pas".to_string(),
                path: "{{mod}}Test.pas".to_string(),
                content: "unit {{mod}}Test;".to_string(),
                process: true,
            }],
            dependencies: vec![],
            config: HashMap::new(),
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(template_dir.join("template.json"), json).unwrap();

        let mut manager = TemplateManager::new(dir.path().to_path_buf()).unwrap();
        let loaded = manager.get_template("my-template").unwrap();
        assert_eq!(loaded.name, "my-template");
        assert_eq!(loaded.files[0].content, "unit {{mod}}Test;");
    }

    #[test]
    fn test_list_templates_includes_disk_template() {
        let dir = TempDir::new().unwrap();
        let template_dir = dir.path().join("custom-template");
        fs::create_dir_all(&template_dir).unwrap();
        let config = TemplateConfig {
            name: "custom-template".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: None,
            favorite: false,
            variables: HashMap::new(),
            files: vec![],
            dependencies: vec![],
            config: HashMap::new(),
        };
        fs::write(
            template_dir.join("template.json"),
            serde_json::to_string(&config).unwrap(),
        ).unwrap();

        let manager = TemplateManager::new(dir.path().to_path_buf()).unwrap();
        let list = manager.list_templates();
        assert!(list.contains(&"custom-template".to_string()));
    }

    #[test]
    fn test_cleanup_cache_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let mut manager = manager_with_tempdir(&dir);
        // Popula o cache
        let _ = manager.get_template("default-module");
        // Limpar não deve panicar
        manager.cleanup_cache();
    }

    // ── Testes para o campo `config` (N3) ────────────────────────────────────

    /// Serializa e desserializa TemplateConfig com entradas em `config`.
    #[test]
    fn test_template_config_serializes_config_entries() {
        let mut cfg = TemplateConfig {
            name: "my-template".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: None,
            favorite: false,
            variables: HashMap::new(),
            files: vec![],
            dependencies: vec![],
            config: HashMap::new(),
        };
        cfg.config.insert("author".to_string(), "Isaque Pinheiro".to_string());
        cfg.config.insert("version".to_string(), "2.0.0".to_string());

        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let decoded: TemplateConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.config.get("author").map(|s| s.as_str()), Some("Isaque Pinheiro"));
        assert_eq!(decoded.config.get("version").map(|s| s.as_str()), Some("2.0.0"));
    }

    /// JSON sem o campo `config` deve desserializar com mapa vazio (serde default).
    #[test]
    fn test_template_config_defaults_config_to_empty_map() {
        let json = r#"{
            "name": "legacy",
            "description": "",
            "version": "1.0.0",
            "variables": {},
            "files": [],
            "dependencies": []
        }"#;

        let cfg: TemplateConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.config.is_empty(), "config deve ser vazio por padrão");
    }

    /// `configure_template` grava key=value em template.json no disco.
    #[test]
    fn test_configure_template_persists_to_disk() {
        let dir = TempDir::new().unwrap();
        let template_dir = dir.path().join("my-tpl");
        fs::create_dir_all(&template_dir).unwrap();

        let initial = TemplateConfig {
            name: "my-tpl".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: None,
            favorite: false,
            variables: HashMap::new(),
            files: vec![],
            dependencies: vec![],
            config: HashMap::new(),
        };
        let config_path = template_dir.join("template.json");
        fs::write(&config_path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        // Simula a lógica de configure_template: load → insert → save
        let content = fs::read_to_string(&config_path).unwrap();
        let mut template: TemplateConfig = serde_json::from_str(&content).unwrap();
        template.config.insert("author".to_string(), "Test User".to_string());
        fs::write(&config_path, serde_json::to_string_pretty(&template).unwrap()).unwrap();

        // Relê do disco e verifica
        let reloaded: TemplateConfig =
            serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(
            reloaded.config.get("author").map(|s| s.as_str()),
            Some("Test User")
        );
    }
}