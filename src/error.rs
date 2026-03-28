use thiserror::Error;

/// Tipo de resultado customizado para a aplicação
pub type Result<T> = std::result::Result<T, CliError>;

/// Enum de erros customizados para o Nidus-cli
#[derive(Error, Debug)]
pub enum CliError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Invalid module name: '{0}'. Use only letters, numbers and underscore.")]
    InvalidModuleName(String),

    #[error("Project not found. Run 'Nidus new <name>' first.")]
    ProjectNotFound,

    #[error("File already exists: {0}. Use --overwrite to overwrite.")]
    FileAlreadyExists(String),

    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),
}

impl CliError {
    pub fn config_error(msg: impl Into<String>) -> Self {
        CliError::ConfigError(msg.into())
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        CliError::ValidationError(msg.into())
    }

    pub fn invalid_module_name(name: impl Into<String>) -> Self {
        CliError::InvalidModuleName(name.into())
    }

    pub fn file_already_exists(path: impl Into<String>) -> Self {
        CliError::FileAlreadyExists(path.into())
    }

    pub fn dependency_not_found(dep: impl Into<String>) -> Self {
        CliError::DependencyNotFound(dep.into())
    }
}

/// Macro para facilitar a criação de erros de configuração
#[macro_export]
macro_rules! config_error {
    ($($arg:tt)*) => {
        $crate::error::CliError::config_error(format!($($arg)*))
    };
}

/// Macro para facilitar a criação de erros de validação
#[macro_export]
macro_rules! validation_error {
    ($($arg:tt)*) => {
        $crate::error::CliError::validation_error(format!($($arg)*))
    };
}