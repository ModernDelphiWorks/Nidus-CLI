pub mod commands;
pub mod core;
pub mod dto;
pub mod error;
pub mod templates;
pub mod validation;

// Re-export principais tipos
pub use error::{CliError, Result};

/// Inicializa o sistema de logging
pub fn init_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}
