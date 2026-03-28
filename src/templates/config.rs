//! Customizable template configuration.
//!
//! This module defines configuration structures for custom templates and their variables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Global templates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesConfig {
    /// Base directory for custom templates
    pub templates_dir: PathBuf,
    /// Default template for new modules
    pub default_module_template: String,
    /// Default template for new projects
    pub default_project_template: String,
    /// Whether caching is enabled
    pub cache_enabled: bool,
    /// Cache time-to-live in seconds
    pub cache_ttl: u64,
    /// User's favorite templates
    pub favorite_templates: Vec<String>,
    /// External template repositories
    pub template_repositories: Vec<TemplateRepository>,
    /// Per-template specific configuration
    pub template_configs: HashMap<String, TemplateSpecificConfig>,
}

/// External template repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRepository {
    /// Repository name
    pub name: String,
    /// Repository URL
    pub url: String,
    /// Specific branch or tag
    pub branch: Option<String>,
    /// Whether this repository is enabled
    pub enabled: bool,
    /// Last update timestamp
    pub last_update: Option<String>,
}

/// Per-template specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSpecificConfig {
    /// Default variable values for this template
    pub default_variables: HashMap<String, String>,
    /// Whether to always prompt for variables
    pub always_prompt: bool,
    /// Pre-generation hooks
    pub pre_hooks: Vec<String>,
    /// Post-generation hooks
    pub post_hooks: Vec<String>,
}

/// Developer profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperProfile {
    /// Developer name
    pub name: String,
    /// Developer email
    pub email: String,
    /// Organization / company
    pub organization: Option<String>,
    /// Default namespace
    pub default_namespace: Option<String>,
    /// Preferred templates
    pub preferred_templates: Vec<String>,
    /// Custom settings
    pub custom_settings: HashMap<String, String>,
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            templates_dir: PathBuf::from(".Nidus/templates"),
            default_module_template: "default-module".to_string(),
            default_project_template: "default-project".to_string(),
            cache_enabled: true,
            cache_ttl: 3600, // 1 hour
            favorite_templates: vec![
                "default-module".to_string(),
                "api-module".to_string(),
                "crud-module".to_string(),
            ],
            template_repositories: vec![
                TemplateRepository {
                    name: "official".to_string(),
                    url: "https://github.com/Nidus/templates".to_string(),
                    branch: Some("main".to_string()),
                    enabled: true,
                    last_update: None,
                },
            ],
            template_configs: HashMap::new(),
        }
    }
}

impl Default for DeveloperProfile {
    fn default() -> Self {
        Self {
            name: "Nidus Developer".to_string(),
            email: "developer@Nidus.com".to_string(),
            organization: None,
            default_namespace: None,
            preferred_templates: vec!["default-module".to_string()],
            custom_settings: HashMap::new(),
        }
    }
}

/// Template variable configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariableConfig {
    /// Variable name
    pub name: String,
    /// Default value
    pub default_value: Option<String>,
    /// Variable description
    pub description: String,
    /// Variable type
    pub var_type: VariableType,
    /// Whether the variable is required
    pub required: bool,
    /// Optional regex validation
    pub validation: Option<String>,
    /// Choices for `Choice`-typed variables
    pub choices: Option<Vec<String>>,
}

/// Supported variable types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    /// Plain string
    String,
    /// Integer number
    Integer,
    /// Floating-point number
    Float,
    /// Boolean
    Boolean,
    /// One of a fixed set of choices
    Choice,
    /// List of strings
    Array,
    /// Date/time
    DateTime,
    /// Email address
    Email,
    /// URL
    Url,
}

/// Hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Hook name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory
    pub working_dir: Option<PathBuf>,
    /// Whether to fail silently
    pub fail_silently: bool,
    /// Timeout in seconds
    pub timeout: Option<u64>,
}

/// Smart cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Maximum cache size in MB
    pub max_size_mb: u64,
    /// Default TTL in seconds
    pub default_ttl: u64,
    /// Cleanup strategy
    pub cleanup_strategy: CacheCleanupStrategy,
    /// Whether compression is enabled
    pub compression_enabled: bool,
}

/// Cache cleanup strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheCleanupStrategy {
    /// Least recently used
    LRU,
    /// First in, first out
    FIFO,
    /// TTL-based
    TTL,
    /// Manual cleanup
    Manual,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_mb: 100,
            default_ttl: 3600,
            cleanup_strategy: CacheCleanupStrategy::LRU,
            compression_enabled: true,
        }
    }
}
