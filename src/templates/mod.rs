//! Módulo de templates do Nidus CLI
//! 
//! Este módulo gerencia todos os templates disponíveis no sistema,
//! incluindo templates built-in e customizados.

pub mod config;
pub mod processor;
pub mod template_manager;

pub use config::*;
pub use processor::*;
pub use template_manager::*;