//! Configuração de templates customizáveis
//! 
//! Este módulo define as estruturas de configuração para
//! templates personalizados e suas variáveis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuração global de templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesConfig {
    /// Diretório base para templates customizados
    pub templates_dir: PathBuf,
    /// Template padrão para novos módulos
    pub default_module_template: String,
    /// Template padrão para novos projetos
    pub default_project_template: String,
    /// Cache habilitado
    pub cache_enabled: bool,
    /// Tempo de vida do cache em segundos
    pub cache_ttl: u64,
    /// Templates favoritos do usuário
    pub favorite_templates: Vec<String>,
    /// Repositórios de templates externos
    pub template_repositories: Vec<TemplateRepository>,
    /// Configurações específicas por template
    pub template_configs: HashMap<String, TemplateSpecificConfig>,
}

/// Repositório de templates externos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRepository {
    /// Nome do repositório
    pub name: String,
    /// URL do repositório
    pub url: String,
    /// Branch ou tag específica
    pub branch: Option<String>,
    /// Se está habilitado
    pub enabled: bool,
    /// Última atualização
    pub last_update: Option<String>,
}

/// Configuração específica de um template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSpecificConfig {
    /// Variáveis padrão para este template
    pub default_variables: HashMap<String, String>,
    /// Se deve sempre perguntar pelas variáveis
    pub always_prompt: bool,
    /// Hooks pré-geração
    pub pre_hooks: Vec<String>,
    /// Hooks pós-geração
    pub post_hooks: Vec<String>,
}

/// Perfil de desenvolvimento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperProfile {
    /// Nome do desenvolvedor
    pub name: String,
    /// Email do desenvolvedor
    pub email: String,
    /// Organização/empresa
    pub organization: Option<String>,
    /// Namespace padrão
    pub default_namespace: Option<String>,
    /// Templates preferidos
    pub preferred_templates: Vec<String>,
    /// Configurações personalizadas
    pub custom_settings: HashMap<String, String>,
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            templates_dir: PathBuf::from(".nest4d/templates"),
            default_module_template: "default-module".to_string(),
            default_project_template: "default-project".to_string(),
            cache_enabled: true,
            cache_ttl: 3600, // 1 hora
            favorite_templates: vec![
                "default-module".to_string(),
                "api-module".to_string(),
                "crud-module".to_string(),
            ],
            template_repositories: vec![
                TemplateRepository {
                    name: "official".to_string(),
                    url: "https://github.com/nest4d/templates".to_string(),
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
            name: "Nest4D Developer".to_string(),
            email: "developer@nest4d.com".to_string(),
            organization: None,
            default_namespace: None,
            preferred_templates: vec!["default-module".to_string()],
            custom_settings: HashMap::new(),
        }
    }
}

/// Configuração de variáveis de template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariableConfig {
    /// Nome da variável
    pub name: String,
    /// Valor padrão
    pub default_value: Option<String>,
    /// Descrição da variável
    pub description: String,
    /// Tipo da variável
    pub var_type: VariableType,
    /// Se é obrigatória
    pub required: bool,
    /// Validação regex (opcional)
    pub validation: Option<String>,
    /// Opções para variáveis do tipo choice
    pub choices: Option<Vec<String>>,
}

/// Tipos de variáveis suportadas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    /// String simples
    String,
    /// Número inteiro
    Integer,
    /// Número decimal
    Float,
    /// Booleano
    Boolean,
    /// Escolha entre opções
    Choice,
    /// Lista de strings
    Array,
    /// Data/hora
    DateTime,
    /// Email
    Email,
    /// URL
    Url,
}

/// Configuração de hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Nome do hook
    pub name: String,
    /// Comando a ser executado
    pub command: String,
    /// Argumentos do comando
    pub args: Vec<String>,
    /// Diretório de trabalho
    pub working_dir: Option<PathBuf>,
    /// Se deve falhar silenciosamente
    pub fail_silently: bool,
    /// Timeout em segundos
    pub timeout: Option<u64>,
}

/// Configuração de cache inteligente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Se o cache está habilitado
    pub enabled: bool,
    /// Tamanho máximo do cache em MB
    pub max_size_mb: u64,
    /// TTL padrão em segundos
    pub default_ttl: u64,
    /// Estratégia de limpeza
    pub cleanup_strategy: CacheCleanupStrategy,
    /// Compressão habilitada
    pub compression_enabled: bool,
}

/// Estratégias de limpeza de cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheCleanupStrategy {
    /// Menos recentemente usado
    LRU,
    /// Primeiro a entrar, primeiro a sair
    FIFO,
    /// Baseado em TTL
    TTL,
    /// Limpeza manual
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