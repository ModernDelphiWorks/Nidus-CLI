//! Template Manager - Sistema de gerenciamento de templates customizáveis
//! 
//! Este módulo implementa:
//! - Carregamento de templates customizados
//! - Cache inteligente para templates
//! - Sistema de variáveis configuráveis
//! - Suporte a templates externos

use crate::error::CliError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Configuração de um template customizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Nome do template
    pub name: String,
    /// Descrição do template
    pub description: String,
    /// Versão do template
    pub version: String,
    /// Autor do template
    pub author: Option<String>,
    /// Variáveis disponíveis no template
    pub variables: HashMap<String, TemplateVariable>,
    /// Arquivos do template
    pub files: Vec<TemplateFile>,
    /// Dependências necessárias
    pub dependencies: Vec<String>,
}

/// Definição de uma variável de template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Nome da variável
    pub name: String,
    /// Descrição da variável
    pub description: String,
    /// Valor padrão
    pub default_value: Option<String>,
    /// Se é obrigatória
    pub required: bool,
    /// Tipo da variável (string, boolean, number)
    pub var_type: String,
}

/// Definição de um arquivo de template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    /// Nome do arquivo
    pub name: String,
    /// Caminho relativo no template
    pub path: String,
    /// Conteúdo do template
    pub content: String,
    /// Se deve ser processado (substituir variáveis)
    pub process: bool,
}

/// Cache de templates para melhor performance
#[derive(Debug)]
struct TemplateCache {
    /// Templates em cache
    templates: HashMap<String, (TemplateConfig, SystemTime)>,
    /// Tempo de vida do cache (em segundos)
    ttl: u64,
}

/// Gerenciador principal de templates
pub struct TemplateManager {
    /// Cache de templates
    cache: TemplateCache,
    /// Diretório base de templates
    templates_dir: PathBuf,
    /// Templates built-in
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
            timestamp.elapsed().map_or(false, |elapsed| elapsed.as_secs() < self.ttl)
        });
    }
}

impl TemplateManager {
    /// Cria um novo gerenciador de templates
    pub fn new(templates_dir: PathBuf) -> Result<Self, CliError> {
        let mut manager = Self {
            cache: TemplateCache::new(3600), // 1 hora de cache
            templates_dir,
            builtin_templates: HashMap::new(),
        };
        
        manager.load_builtin_templates()?;
        Ok(manager)
    }

    /// Carrega templates built-in do sistema
    fn load_builtin_templates(&mut self) -> Result<(), CliError> {
        // Template padrão para módulos
        let module_template = TemplateConfig {
            name: "default-module".to_string(),
            description: "Template padrão para módulos Nest4D".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Nest4D Team".to_string()),
            variables: self.create_default_variables(),
            files: self.create_default_module_files(),
            dependencies: vec![
                "nest4d".to_string(),
                "evolution4d".to_string(),
            ],
        };

        self.builtin_templates.insert("default-module".to_string(), module_template);
        Ok(())
    }

    /// Cria variáveis padrão para templates
    fn create_default_variables(&self) -> HashMap<String, TemplateVariable> {
        let mut variables = HashMap::new();
        
        variables.insert("mod".to_string(), TemplateVariable {
            name: "mod".to_string(),
            description: "Nome do módulo em CamelCase".to_string(),
            default_value: None,
            required: true,
            var_type: "string".to_string(),
        });

        variables.insert("author".to_string(), TemplateVariable {
            name: "author".to_string(),
            description: "Autor do módulo".to_string(),
            default_value: Some("Nest4D Developer".to_string()),
            required: false,
            var_type: "string".to_string(),
        });

        variables.insert("namespace".to_string(), TemplateVariable {
            name: "namespace".to_string(),
            description: "Namespace do projeto".to_string(),
            default_value: None,
            required: false,
            var_type: "string".to_string(),
        });

        variables
    }

    /// Cria arquivos padrão para template de módulo
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

    /// Obtém um template (cache primeiro, depois disco, depois built-in)
    pub fn get_template(&mut self, name: &str) -> Result<TemplateConfig, CliError> {
        // 1. Verifica cache
        if let Some(template) = self.cache.get(name) {
            return Ok(template.clone());
        }

        // 2. Tenta carregar do disco
        if let Ok(template) = self.load_template_from_disk(name) {
            self.cache.insert(name.to_string(), template.clone());
            return Ok(template);
        }

        // 3. Verifica built-in
        if let Some(template) = self.builtin_templates.get(name) {
            return Ok(template.clone());
        }

        Err(CliError::ValidationError(format!("Template '{}' not found", name)))
    }

    /// Carrega template do disco
    fn load_template_from_disk(&self, name: &str) -> Result<TemplateConfig, CliError> {
        let template_path = self.templates_dir.join(name).join("template.json");
        
        if !template_path.exists() {
            return Err(CliError::ValidationError(format!("Template file not found: {:?}", template_path)));
        }

        let content = fs::read_to_string(&template_path)
            .map_err(|e| CliError::IoError(e))?;
        
        let mut template: TemplateConfig = serde_json::from_str(&content)
            .map_err(|e| CliError::JsonError(e))?;

        // Carrega conteúdo dos arquivos
        let template_dir = template_path.parent().unwrap();
        for file in &mut template.files {
            let file_path = template_dir.join(&file.path);
            if file_path.exists() {
                file.content = fs::read_to_string(&file_path)
                    .map_err(|e| CliError::IoError(e))?;
            }
        }

        Ok(template)
    }

    /// Lista todos os templates disponíveis
    pub fn list_templates(&self) -> Vec<String> {
        let mut templates = Vec::new();
        
        // Built-in templates
        templates.extend(self.builtin_templates.keys().cloned());
        
        // Templates do disco
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

    /// Processa variáveis em um template
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
                // Processa variáveis no conteúdo e caminho usando sintaxe {{var}}
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

    /// Limpa cache expirado
    pub fn cleanup_cache(&mut self) {
        self.cache.clear_expired();
    }

    /// Instala um template externo
    pub fn install_template(&mut self, source: &str, name: &str) -> Result<(), CliError> {
        // TODO: Implementar download e instalação de templates externos
        // Por enquanto, apenas um placeholder
        println!("Installing template '{}' from '{}'...", name, source);
        Ok(())
    }
}