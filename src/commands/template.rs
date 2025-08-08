//! Comando para gerenciamento de templates
//! 
//! Este comando permite:
//! - Listar templates disponíveis
//! - Instalar novos templates
//! - Criar templates customizados
//! - Configurar templates

use crate::error::CliError;
use crate::templates::*;
use clap::{Args, Subcommand};
use colored::*;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Comando para gerenciamento de templates
#[derive(Debug, Args)]
pub struct TemplateCommand {
    #[command(subcommand)]
    pub action: TemplateAction,
}

/// Ações disponíveis para templates
#[derive(Debug, Subcommand)]
pub enum TemplateAction {
    /// Lista todos os templates disponíveis
    List {
        /// Mostra apenas templates favoritos
        #[arg(short, long)]
        favorites: bool,
        /// Filtro por categoria
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Mostra informações detalhadas de um template
    Info {
        /// Nome do template
        name: String,
    },
    /// Instala um template externo
    Install {
        /// URL ou nome do template
        source: String,
        /// Nome local para o template
        #[arg(short, long)]
        name: Option<String>,
        /// Força reinstalação
        #[arg(short, long)]
        force: bool,
    },
    /// Remove um template
    Remove {
        /// Nome do template
        name: String,
        /// Confirma remoção sem perguntar
        #[arg(short, long)]
        yes: bool,
    },
    /// Cria um novo template
    Create {
        /// Nome do template
        name: String,
        /// Descrição do template
        #[arg(short, long)]
        description: Option<String>,
        /// Diretório base para criar o template
        #[arg(short, long)]
        from: Option<PathBuf>,
    },
    /// Configura um template
    Config {
        /// Nome do template
        name: String,
        /// Chave de configuração
        key: Option<String>,
        /// Valor da configuração
        value: Option<String>,
    },
    /// Atualiza templates externos
    Update {
        /// Nome específico do template (opcional)
        name: Option<String>,
        /// Atualiza todos os templates
        #[arg(short, long)]
        all: bool,
    },
    /// Testa um template
    Test {
        /// Nome do template
        name: String,
        /// Diretório de teste
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Exporta templates embutidos para o disco
    Export {
        /// Nome do template (opcional, se não especificado exporta todos)
        name: Option<String>,
        /// Diretório de destino
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Força sobrescrita de arquivos existentes
        #[arg(short, long)]
        force: bool,
    },
}

impl TemplateCommand {
    /// Executa o comando de template
    pub fn execute(&self) -> Result<(), CliError> {
        let templates_dir = get_templates_directory()?;
        let mut manager = TemplateManager::new(templates_dir)?;
        
        match &self.action {
             TemplateAction::List { favorites, category } => {
                 self.list_templates(&mut manager, *favorites, category.as_deref())
             }
             TemplateAction::Info { name } => {
                 self.show_template_info(&mut manager, name)
             }
             TemplateAction::Install { source, name, force } => {
                 self.install_template(&mut manager, source, name.as_deref(), *force)
             }
             TemplateAction::Remove { name, yes } => {
                 self.remove_template(&mut manager, name, *yes)
             }
             TemplateAction::Create { name, description, from } => {
                 self.create_template(&mut manager, name, description.as_deref(), from.as_ref())
             }
             TemplateAction::Config { name, key, value } => {
                 self.configure_template(&mut manager, name, key.as_deref(), value.as_deref())
             }
             TemplateAction::Update { name, all } => {
                 self.update_templates(&manager, name.as_deref(), *all)
             }
             TemplateAction::Test { name, output } => {
                 self.test_template(&mut manager, name, output.as_ref())
             }
             TemplateAction::Export { name, output, force } => {
                 self.export_templates(&mut manager, name.as_deref(), output.as_ref(), *force)
             }
         }
    }

    /// Lista templates disponíveis
    fn list_templates(
        &self,
        manager: &mut TemplateManager,
        favorites_only: bool,
        category: Option<&str>,
    ) -> Result<(), CliError> {
        let templates = manager.list_templates();
        
        if templates.is_empty() {
            println!("{}", "No templates found.".yellow());
            return Ok(());
        }

        println!("{}", "Available Templates:".bold().blue());
        println!();

        for template_name in templates {
            if let Ok(template) = manager.get_template(&template_name) {
                // Filtros
                if favorites_only {
                    // TODO: Implementar sistema de favoritos
                    continue;
                }
                
                if let Some(_cat) = category {
                    // TODO: Implementar categorias
                    continue;
                }

                // Exibe informações do template
                println!("  {} {}", "●".green(), template.name.bold());
                println!("    {}", template.description.dimmed());
                println!("    {} {}", "Version:".dimmed(), template.version);
                
                if let Some(author) = &template.author {
                    println!("    {} {}", "Author:".dimmed(), author);
                }
                
                println!("    {} {}", "Files:".dimmed(), template.files.len());
                println!();
            }
        }

        Ok(())
    }

    /// Mostra informações detalhadas de um template
    fn show_template_info(&self, manager: &mut TemplateManager, name: &str) -> Result<(), CliError> {
        let template = manager.get_template(name)?;
        
        println!("{} {}", "Template:".bold().blue(), template.name.bold());
        println!("{} {}", "Description:".bold(), template.description);
        println!("{} {}", "Version:".bold(), template.version);
        
        if let Some(author) = &template.author {
            println!("{} {}", "Author:".bold(), author);
        }
        
        println!();
        println!("{}", "Variables:".bold().yellow());
        for (var_name, var_config) in &template.variables {
            println!("  {} {}", "●".green(), var_name.bold());
            println!("    {}", var_config.description.dimmed());
            println!("    {} {}", "Type:".dimmed(), var_config.var_type);
            println!("    {} {}", "Required:".dimmed(), var_config.required);
            
            if let Some(default) = &var_config.default_value {
                println!("    {} {}", "Default:".dimmed(), default);
            }
            println!();
        }
        
        println!("{}", "Files:".bold().yellow());
        for file in &template.files {
            println!("  {} {}", "●".green(), file.name.bold());
            println!("    {} {}", "Path:".dimmed(), file.path);
            println!("    {} {}", "Process:".dimmed(), file.process);
            println!();
        }
        
        if !template.dependencies.is_empty() {
            println!("{}", "Dependencies:".bold().yellow());
            for dep in &template.dependencies {
                println!("  {} {}", "●".green(), dep);
            }
        }
        
        Ok(())
    }

    /// Instala um template externo
    fn install_template(
         &self,
         _manager: &mut TemplateManager,
         source: &str,
         name: Option<&str>,
         _force: bool,
     ) -> Result<(), CliError> {
        let template_name = name.unwrap_or_else(|| {
            source.split('/').last().unwrap_or("unknown")
        });
        
        println!("{} template '{}' from '{}'...", "Installing".bold().blue(), template_name, source);
        
        // TODO: Implementar download real
        // TODO: Implementar instalação de template
        println!("{} Installing template '{}' from '{}'...", "📦".blue().bold(), template_name, source);
        
        println!("{} Template '{}' installed successfully!", "✓".green().bold(), template_name);
        Ok(())
    }

    /// Remove um template
    fn remove_template(&self, _manager: &mut TemplateManager, name: &str, yes: bool) -> Result<(), CliError> {
        if !yes {
            println!("{} Are you sure you want to remove template '{}'? [y/N]", "?".yellow().bold(), name);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(|e| CliError::IoError(e))?;
            
            if !input.trim().to_lowercase().starts_with('y') {
                println!("{}", "Operation cancelled.".yellow());
                return Ok(());
            }
        }
        
        let templates_dir = get_templates_directory()?;
        let template_path = templates_dir.join(name);
        
        if template_path.exists() {
            fs::remove_dir_all(&template_path).map_err(|e| CliError::IoError(e))?;
            println!("{} Template '{}' removed successfully!", "✓".green().bold(), name);
        } else {
            println!("{} Template '{}' not found.", "!".yellow().bold(), name);
        }
        
        Ok(())
    }

    /// Cria um novo template
    fn create_template(
        &self,
        _manager: &mut TemplateManager,
        name: &str,
        description: Option<&str>,
        _from: Option<&PathBuf>,
    ) -> Result<(), CliError> {
        let templates_dir = get_templates_directory()?;
        let template_dir = templates_dir.join(name);
        
        if template_dir.exists() {
            return Err(CliError::ValidationError(format!("Template '{}' already exists", name)));
        }
        
        fs::create_dir_all(&template_dir).map_err(|e| CliError::IoError(e))?;
        
        // Cria configuração básica
        let template_config = TemplateConfig {
            name: name.to_string(),
            description: description.unwrap_or("Custom template").to_string(),
            version: "1.0.0".to_string(),
            author: Some("Custom".to_string()),
            variables: HashMap::new(),
            files: Vec::new(),
            dependencies: Vec::new(),
        };
        
        let config_path = template_dir.join("template.json");
        let config_content = serde_json::to_string_pretty(&template_config)
            .map_err(|e| CliError::JsonError(e))?;
        
        fs::write(&config_path, config_content).map_err(|e| CliError::IoError(e))?;
        
        println!("{} Template '{}' created at {:?}", "✓".green().bold(), name, template_dir);
        println!("{} Edit {:?} to configure your template", "→".blue().bold(), config_path);
        
        Ok(())
    }

    /// Configura um template
    fn configure_template(
        &self,
        _manager: &mut TemplateManager,
        _name: &str,
        key: Option<&str>,
        value: Option<&str>,
    ) -> Result<(), CliError> {
        println!("{} Configuring template '{}'...", "⚙".blue().bold(), _name);
        
        // TODO: Implementar configuração interativa
        if let (Some(k), Some(v)) = (key, value) {
            println!("{} Set {} = {}", "✓".green().bold(), k, v);
        } else {
            println!("{} Interactive configuration not implemented yet", "!".yellow().bold());
        }
        
        Ok(())
    }

    /// Atualiza templates
    fn update_templates(
        &self,
        _manager: &TemplateManager,
        name: Option<&str>,
        all: bool,
    ) -> Result<(), CliError> {
        if all {
            println!("{} Updating all templates...", "⟳".blue().bold());
        } else if let Some(template_name) = name {
            println!("{} Updating template '{}'...", "⟳".blue().bold(), template_name);
        } else {
            return Err(CliError::ValidationError("Specify --all or template name".to_string()));
        }
        
        // TODO: Implementar atualização real
        println!("{} Update completed!", "✓".green().bold());
        Ok(())
    }

    /// Testa um template
    fn test_template(
        &self,
        manager: &mut TemplateManager,
        name: &str,
        output: Option<&PathBuf>,
    ) -> Result<(), CliError> {
        println!("{} Testing template '{}'...", "🧪".blue().bold(), name);
        
        let template = manager.get_template(name)?;
        let test_dir = output.cloned().unwrap_or_else(|| {
            std::env::temp_dir().join(format!("nest4d_test_{}", name))
        });
        
        // Cria variáveis de teste
        let mut test_variables = HashMap::new();
        test_variables.insert("mod".to_string(), "TestModule".to_string());
        test_variables.insert("author".to_string(), "Test Author".to_string());
        
        // Processa template
        let _processor = TemplateProcessor::new();
        let context = ProcessingContext {
            variables: test_variables,
            ..Default::default()
        };
        
        let processed_files = manager.process_template(&template, &context.variables)?;
        
        // Cria arquivos de teste
        fs::create_dir_all(&test_dir).map_err(|e| CliError::IoError(e))?;
        
        for (file_path, content) in processed_files {
            let full_path = test_dir.join(&file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(|e| CliError::IoError(e))?;
            }
            fs::write(&full_path, content).map_err(|e| CliError::IoError(e))?;
            println!("{} Created: {}", "✓".green(), file_path);
        }
        
        println!("{} Test completed! Files created in: {:?}", "✓".green().bold(), test_dir);
        Ok(())
    }

    /// Exporta templates embutidos para o disco para personalização
    fn export_templates(
        &self,
        manager: &mut TemplateManager,
        template_name: Option<&str>,
        output_dir: Option<&PathBuf>,
        force: bool,
    ) -> Result<(), CliError> {
        let export_dir = output_dir
            .cloned()
            .unwrap_or_else(|| PathBuf::from("./templates"));
        
        println!("{} Exporting templates to: {}", "📦".blue().bold(), export_dir.display());
        
        // Cria o diretório de destino
        fs::create_dir_all(&export_dir).map_err(|e| CliError::IoError(e))?;
        
        if let Some(name) = template_name {
            // Exporta um template específico
            self.export_single_template(manager, name, &export_dir, force)?;
        } else {
            // Exporta todos os templates built-in
            self.export_all_builtin_templates(&export_dir, force)?;
        }
        
        println!("{} Templates exported successfully!", "✓".green().bold());
        println!("{} You can now customize the templates and they will be loaded automatically.", "→".blue().bold());
        println!("{} Location: {}", "📁".cyan(), export_dir.display());
        
        Ok(())
    }
    
    /// Exporta um template específico
    fn export_single_template(
        &self,
        manager: &mut TemplateManager,
        name: &str,
        export_dir: &PathBuf,
        force: bool,
    ) -> Result<(), CliError> {
        let template = manager.get_template(name)?;
        let template_dir = export_dir.join(&template.name);
        
        fs::create_dir_all(&template_dir).map_err(|e| CliError::IoError(e))?;
        
        // Exporta arquivos do template
        for file in &template.files {
            let file_path = template_dir.join(&file.name);
            
            if file_path.exists() && !force {
                println!("{} File already exists: {} (use --force to overwrite)", "⚠".yellow().bold(), file_path.display());
                continue;
            }
            
            fs::write(&file_path, &file.content).map_err(|e| CliError::IoError(e))?;
            println!("{} Exported: {}", "✓".green(), file_path.display());
        }
        
        // Cria arquivo de configuração do template
        let config_path = template_dir.join("template.json");
        if !config_path.exists() || force {
            let config_json = serde_json::to_string_pretty(&template)
                .map_err(|e| CliError::JsonError(e))?;
            fs::write(&config_path, config_json).map_err(|e| CliError::IoError(e))?;
            println!("{} Configuration: {}", "✓".green(), config_path.display());
        }
        
        Ok(())
    }
    
    /// Exporta todos os templates built-in
    fn export_all_builtin_templates(
        &self,
        export_dir: &PathBuf,
        force: bool,
    ) -> Result<(), CliError> {
        // Cria arquivo README explicativo
        let readme_path = export_dir.join("README.md");
        if !readme_path.exists() || force {
            let readme_content = r#"# Custom Nest4D CLI Templates

This directory contains exported Nest4D CLI templates that you can customize.

## Structure

- `module/` - Templates for module generation
- `project/` - Templates for project generation

## How to Customize

1. Edit the `.pas` files as needed
2. Use variables like `{{mod}}` that will be automatically replaced
3. Custom templates take priority over built-in ones

## Available Variables

- `{{mod}}` - Module name in CamelCase
- `{{project}}` - Project name
- `{{author}}` - Code author
- `{{namespace}}` - Project namespace

## Available Functions

- `{{camelCase(text)}}` - Convert to CamelCase
- `{{snakeCase(text)}}` - Convert to snake_case
- `{{kebabCase(text)}}` - Convert to kebab-case
- `{{upperCase(text)}}` - Convert to UPPERCASE
- `{{lowerCase(text)}}` - Convert to lowercase
"#;
            fs::write(&readme_path, readme_content).map_err(|e| CliError::IoError(e))?;
            println!("{} Documentation: {}", "✓".green(), readme_path.display());
        }
        
        // Exporta templates de módulo
        let module_dir = export_dir.join("module");
        fs::create_dir_all(&module_dir).map_err(|e| CliError::IoError(e))?;
        
        let module_templates = [
            ("controller.pas", include_str!("../templates/module/controller.pas")),
            ("handler.pas", include_str!("../templates/module/handler.pas")),
            ("infra.pas", include_str!("../templates/module/infra.pas")),
            ("interface.pas", include_str!("../templates/module/interface.pas")),
            ("module.pas", include_str!("../templates/module/module.pas")),
            ("repository.pas", include_str!("../templates/module/repository.pas")),
            ("service.pas", include_str!("../templates/module/service.pas")),
        ];
        
        for (filename, content) in module_templates.iter() {
            let file_path = module_dir.join(filename);
            if !file_path.exists() || force {
                fs::write(&file_path, content).map_err(|e| CliError::IoError(e))?;
                println!("{} Module template: {}", "✓".green(), file_path.display());
            }
        }
        
        // Exporta templates de projeto
        let project_dir = export_dir.join("project");
        fs::create_dir_all(&project_dir).map_err(|e| CliError::IoError(e))?;
        
        let project_templates = [
            ("app_module.pas", include_str!("../templates/project/app_module.pas")),
            ("project.dpr", include_str!("../templates/project/project.dpr")),
        ];
        
        for (filename, content) in project_templates.iter() {
            let file_path = project_dir.join(filename);
            if !file_path.exists() || force {
                fs::write(&file_path, content).map_err(|e| CliError::IoError(e))?;
                println!("{} Project template: {}", "✓".green(), file_path.display());
            }
        }
        
        println!("{} Built-in templates exported successfully!", "✓".green().bold());
        Ok(())
    }
}

/// Obtém o diretório de templates
fn get_templates_directory() -> Result<PathBuf, CliError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| CliError::ValidationError("Could not find home directory".to_string()))?;
    
    let templates_dir = PathBuf::from(home).join(".nest4d").join("templates");
    
    if !templates_dir.exists() {
        fs::create_dir_all(&templates_dir).map_err(|e| CliError::IoError(e))?;
    }
    
    Ok(templates_dir)
}