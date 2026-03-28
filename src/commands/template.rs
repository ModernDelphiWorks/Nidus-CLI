//! Template management command.
//!
//! Supports: listing, installing, creating, configuring, updating, testing, and exporting templates.

use crate::commands::cmd_update::{update_repo, UpdateStatus};
use crate::core::core_utils::utils;
use crate::error::CliError;
use crate::templates::*;
use crate::validation::validate_git_url;
use clap::{Args, Subcommand};
use colored::*;
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Template management command
#[derive(Debug, Args)]
pub struct TemplateCommand {
    #[command(subcommand)]
    pub action: TemplateAction,
}

/// Available template actions
#[derive(Debug, Subcommand)]
pub enum TemplateAction {
    /// List all available templates
    List {
        /// Show only favorite templates
        #[arg(short, long)]
        favorites: bool,
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Show detailed information about a template
    Info {
        /// Template name
        name: String,
    },
    /// Install an external template
    Install {
        /// Template source URL or name
        source: String,
        /// Local name for the template
        #[arg(short, long)]
        name: Option<String>,
        /// Force reinstallation
        #[arg(short, long)]
        force: bool,
    },
    /// Remove a template
    Remove {
        /// Template name
        name: String,
        /// Confirm removal without prompting
        #[arg(short, long)]
        yes: bool,
    },
    /// Create a new template
    Create {
        /// Template name
        name: String,
        /// Template description
        #[arg(short, long)]
        description: Option<String>,
        /// Base directory to create the template from
        #[arg(short, long)]
        from: Option<PathBuf>,
    },
    /// Configure a template
    Config {
        /// Template name
        name: String,
        /// Configuration key
        key: Option<String>,
        /// Configuration value
        value: Option<String>,
    },
    /// Update external templates
    Update {
        /// Specific template name (optional)
        name: Option<String>,
        /// Update all installed templates
        #[arg(short, long)]
        all: bool,
    },
    /// Test a template by rendering it to a temporary directory
    Test {
        /// Template name
        name: String,
        /// Output directory for rendered files
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Export built-in templates to disk for customization
    Export {
        /// Template name (optional; exports all if omitted)
        name: Option<String>,
        /// Destination directory
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,
    },
}

impl TemplateCommand {
    /// Executes the template command
    pub fn execute(&self) -> Result<(), CliError> {
        let templates_dir = utils::get_templates_directory()?;
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

    /// Lists available templates
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
                // Apply filters
                if favorites_only && !template.favorite {
                    continue;
                }

                if let Some(cat) = category {
                    if template.category.as_deref() != Some(cat) {
                        continue;
                    }
                }

                // Display template details
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

    /// Shows detailed information about a template
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

    /// Installs an external template via git clone
    fn install_template(
        &self,
        _manager: &mut TemplateManager,
        source: &str,
        name: Option<&str>,
        force: bool,
    ) -> Result<(), CliError> {
        validate_git_url(source)?;

        let repo_name = name
            .map(|n| n.to_string())
            .or_else(|| utils::extract_repo_name(source))
            .ok_or_else(|| CliError::validation_error("Could not determine template name from URL — use --name"))?;

        let templates_dir = utils::get_templates_directory()?;
        let dest = templates_dir.join(&repo_name);

        if dest.exists() {
            if force {
                fs::remove_dir_all(&dest).map_err(CliError::IoError)?;
            } else {
                return Err(CliError::validation_error(format!(
                    "Template '{}' is already installed. Use --force to reinstall or `Nidus template update`.",
                    repo_name
                )));
            }
        }

        println!("{} Cloning template '{}'...", "📦".blue(), repo_name.bold());

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username, allowed| {
            if allowed.is_ssh_key() {
                git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
            } else {
                git2::Cred::default()
            }
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(source, &dest)
            .map_err(|e| CliError::validation_error(format!("Failed to clone template: {}", e)))?;

        println!("{} Template '{}' installed at {:?}", "✅".green(), repo_name.bold(), dest);
        Ok(())
    }

    /// Removes a template
    fn remove_template(&self, _manager: &mut TemplateManager, name: &str, yes: bool) -> Result<(), CliError> {
        if !yes {
            println!("{} Are you sure you want to remove template '{}'? [y/N]", "?".yellow().bold(), name);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(CliError::IoError)?;
            
            if !input.trim().to_lowercase().starts_with('y') {
                println!("{}", "Operation cancelled.".yellow());
                return Ok(());
            }
        }
        
        let templates_dir = utils::get_templates_directory()?;
        let template_path = templates_dir.join(name);
        
        if template_path.exists() {
            fs::remove_dir_all(&template_path).map_err(CliError::IoError)?;
            println!("{} Template '{}' removed successfully!", "✓".green().bold(), name);
        } else {
            println!("{} Template '{}' not found.", "!".yellow().bold(), name);
        }
        
        Ok(())
    }

    /// Creates a new template, optionally scanning `.pas` files from `--from <dir>`
    fn create_template(
        &self,
        _manager: &mut TemplateManager,
        name: &str,
        description: Option<&str>,
        from: Option<&PathBuf>,
    ) -> Result<(), CliError> {
        let templates_dir = utils::get_templates_directory()?;
        let template_dir = templates_dir.join(name);

        if template_dir.exists() {
            return Err(CliError::ValidationError(format!("Template '{}' already exists", name)));
        }

        fs::create_dir_all(&template_dir).map_err(CliError::IoError)?;

        // Scan .pas files if --from was provided
        let files = if let Some(src_dir) = from {
            if !src_dir.exists() {
                return Err(CliError::validation_error(format!(
                    "Source directory not found: {}",
                    src_dir.display()
                )));
            }
            self.scan_pas_files_as_template(src_dir)?
        } else {
            Vec::new()
        };

        let file_count = files.len();
        let template_config = TemplateConfig {
            name: name.to_string(),
            description: description.unwrap_or("Custom template").to_string(),
            version: "1.0.0".to_string(),
            author: None,
            category: None,
            favorite: false,
            variables: HashMap::new(),
            files,
            dependencies: Vec::new(),
            config: HashMap::new(),
        };

        let config_path = template_dir.join("template.json");
        let config_content = serde_json::to_string_pretty(&template_config)
            .map_err(CliError::JsonError)?;
        fs::write(&config_path, config_content).map_err(CliError::IoError)?;

        if let Some(src) = from {
            println!(
                "{} Template '{}' created from {} with {} file(s)",
                "✅".green(),
                name.bold(),
                src.display(),
                file_count
            );
            println!(
                "{}  '{{{{mod}}}}' placeholders substituted where the module name was detected.",
                "ℹ️ ".blue()
            );
        } else {
            println!("{} Template '{}' created at {}", "✅".green(), name.bold(), template_dir.display());
        }
        println!("{} Edit {} to configure your template", "→".blue().bold(), config_path.display());

        Ok(())
    }

    /// Scans `.pas` files in `src_dir` and builds `TemplateFile` entries,
    /// replacing the CamelCase module name (derived from the dir name) with `{{mod}}`.
    fn scan_pas_files_as_template(&self, src_dir: &PathBuf) -> Result<Vec<TemplateFile>, CliError> {
        let dir_name = src_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let camel_name = utils::camel_case(&dir_name);

        let mut files = Vec::new();
        for entry in fs::read_dir(src_dir).map_err(CliError::IoError)? {
            let entry = entry.map_err(CliError::IoError)?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());
            if ext.as_deref() != Some("pas") {
                continue;
            }

            let raw_filename = path.file_name().unwrap().to_string_lossy().to_string();
            let raw_content = fs::read_to_string(&path).map_err(CliError::IoError)?;

            let (template_filename, template_content) = if !camel_name.is_empty() {
                (
                    raw_filename.replace(&camel_name, "{{mod}}"),
                    raw_content.replace(&camel_name, "{{mod}}"),
                )
            } else {
                (raw_filename.clone(), raw_content)
            };

            files.push(TemplateFile {
                name: raw_filename,
                path: template_filename,
                content: template_content,
                process: true,
            });
        }

        Ok(files)
    }

    /// Configures a template — persists key=value into template.json
    fn configure_template(
        &self,
        _manager: &mut TemplateManager,
        name: &str,
        key: Option<&str>,
        value: Option<&str>,
    ) -> Result<(), CliError> {
        let templates_dir = utils::get_templates_directory()?;
        let config_path = templates_dir.join(name).join("template.json");

        if !config_path.exists() {
            return Err(CliError::validation_error(format!(
                "Template '{}' not found (expected {:?})",
                name, config_path
            )));
        }

        let content = fs::read_to_string(&config_path).map_err(CliError::IoError)?;
        let mut template: TemplateConfig =
            serde_json::from_str(&content).map_err(CliError::JsonError)?;

        match (key, value) {
            (Some(k), Some(v)) => {
                template.config.insert(k.to_string(), v.to_string());
                let updated =
                    serde_json::to_string_pretty(&template).map_err(CliError::JsonError)?;
                fs::write(&config_path, updated).map_err(CliError::IoError)?;
                println!(
                    "{} Set {} = {} in template '{}'",
                    "✅".green(),
                    k.bold(),
                    v,
                    name
                );
            }
            _ => {
                println!("{} Configuration for '{}':", "⚙".blue().bold(), name);
                if template.config.is_empty() {
                    println!("  (no custom configuration set)");
                } else {
                    for (k, v) in &template.config {
                        println!("  {} = {}", k.bold(), v);
                    }
                }
            }
        }

        Ok(())
    }

    /// Updates installed templates via git pull (fast-forward)
    fn update_templates(
        &self,
        _manager: &TemplateManager,
        name: Option<&str>,
        _all: bool,
    ) -> Result<(), CliError> {
        let templates_dir = utils::get_templates_directory()?;

        // Collect candidate directories
        let candidates: Vec<_> = if let Some(n) = name {
            vec![templates_dir.join(n)]
        } else {
            match fs::read_dir(&templates_dir) {
                Ok(rd) => rd
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .map(|e| e.path())
                    .collect(),
                Err(_) => vec![],
            }
        };

        if candidates.is_empty() {
            println!("{}", "No installed templates found.".yellow());
            return Ok(());
        }

        println!("{}", "\n🔄 Updating templates...\n".cyan());

        let mut updated = 0usize;
        let mut up_to_date = 0usize;
        let mut skipped = 0usize;

        for dir in &candidates {
            let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();

            // Skip directories that are not git repos
            if !dir.join(".git").exists() {
                println!("{} {} — not a git repository, skipping", "⏭".dimmed(), dir_name.dimmed());
                skipped += 1;
                continue;
            }

            match update_repo(&dir.to_string_lossy(), "") {
                Ok(UpdateStatus::FastForwarded) => {
                    println!("{} {}", "✅ Updated:".green(), dir_name.green());
                    updated += 1;
                }
                Ok(UpdateStatus::UpToDate) => {
                    println!("{} {}", "🔁 Already up to date:".blue(), dir_name.blue());
                    up_to_date += 1;
                }
                Err(e) => {
                    println!("{} {}: {}", "⚠️ ".yellow(), dir_name.yellow(), e);
                    skipped += 1;
                }
            }
        }

        println!("\n{}: {}  {}: {}  {}: {}",
            "✅ Updated".bold(), updated,
            "🔁 Up to date".bold(), up_to_date,
            "⏭ Skipped".bold(), skipped,
        );
        Ok(())
    }

    /// Tests a template by rendering it to a temporary directory
    fn test_template(
        &self,
        manager: &mut TemplateManager,
        name: &str,
        output: Option<&PathBuf>,
    ) -> Result<(), CliError> {
        println!("{} Testing template '{}'...", "🧪".blue().bold(), name);
        
        let template = manager.get_template(name)?;
        let test_dir = output.cloned().unwrap_or_else(|| {
            std::env::temp_dir().join(format!("Nidus_test_{}", name))
        });
        
        // Build test variables
        let mut test_variables = HashMap::new();
        test_variables.insert("mod".to_string(), "TestModule".to_string());
        test_variables.insert("author".to_string(), "Test Author".to_string());

        // Process template
        let _processor = TemplateProcessor::new();
        let context = ProcessingContext {
            variables: test_variables,
            ..Default::default()
        };

        let processed_files = manager.process_template(&template, &context.variables)?;

        // Write test output files
        fs::create_dir_all(&test_dir).map_err(CliError::IoError)?;
        
        for (file_path, content) in processed_files {
            let full_path = test_dir.join(&file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(CliError::IoError)?;
            }
            fs::write(&full_path, content).map_err(CliError::IoError)?;
            println!("{} Created: {}", "✓".green(), file_path);
        }
        
        println!("{} Test completed! Files created in: {:?}", "✓".green().bold(), test_dir);
        Ok(())
    }

    /// Exports built-in templates to disk for customization
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
        
        // Create destination directory
        fs::create_dir_all(&export_dir).map_err(CliError::IoError)?;

        if let Some(name) = template_name {
            // Export a single named template
            self.export_single_template(manager, name, &export_dir, force)?;
        } else {
            // Export all built-in templates
            self.export_all_builtin_templates(&export_dir, force)?;
        }
        
        println!("{} Templates exported successfully!", "✓".green().bold());
        println!("{} You can now customize the templates and they will be loaded automatically.", "→".blue().bold());
        println!("{} Location: {}", "📁".cyan(), export_dir.display());
        
        Ok(())
    }
    
    /// Exports a single named template to the given directory
    fn export_single_template(
        &self,
        manager: &mut TemplateManager,
        name: &str,
        export_dir: &Path,
        force: bool,
    ) -> Result<(), CliError> {
        let template = manager.get_template(name)?;
        let template_dir = export_dir.join(&template.name);
        
        fs::create_dir_all(&template_dir).map_err(CliError::IoError)?;
        
        // Export template files
        for file in &template.files {
            let file_path = template_dir.join(&file.name);
            
            if file_path.exists() && !force {
                println!("{} File already exists: {} (use --force to overwrite)", "⚠".yellow().bold(), file_path.display());
                continue;
            }
            
            fs::write(&file_path, &file.content).map_err(CliError::IoError)?;
            println!("{} Exported: {}", "✓".green(), file_path.display());
        }
        
        // Write template configuration file
        let config_path = template_dir.join("template.json");
        if !config_path.exists() || force {
            let config_json = serde_json::to_string_pretty(&template)
                .map_err(CliError::JsonError)?;
            fs::write(&config_path, config_json).map_err(CliError::IoError)?;
            println!("{} Configuration: {}", "✓".green(), config_path.display());
        }
        
        Ok(())
    }
    
    /// Exports all built-in templates to the given directory
    fn export_all_builtin_templates(
        &self,
        export_dir: &Path,
        force: bool,
    ) -> Result<(), CliError> {
        // Create README documentation file
        let readme_path = export_dir.join("README.md");
        if !readme_path.exists() || force {
            let readme_content = r#"# Custom Nidus CLI Templates

This directory contains exported Nidus CLI templates that you can customize.

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
            fs::write(&readme_path, readme_content).map_err(CliError::IoError)?;
            println!("{} Documentation: {}", "✓".green(), readme_path.display());
        }
        
        // Export module templates
        let module_dir = export_dir.join("module");
        fs::create_dir_all(&module_dir).map_err(CliError::IoError)?;
        
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
                fs::write(&file_path, content).map_err(CliError::IoError)?;
                println!("{} Module template: {}", "✓".green(), file_path.display());
            }
        }
        
        // Export project templates
        let project_dir = export_dir.join("project");
        fs::create_dir_all(&project_dir).map_err(CliError::IoError)?;
        
        let project_templates = [
            ("app_module.pas", include_str!("../templates/project/app_module.pas")),
            ("project.dpr", include_str!("../templates/project/project.dpr")),
        ];
        
        for (filename, content) in project_templates.iter() {
            let file_path = project_dir.join(filename);
            if !file_path.exists() || force {
                fs::write(&file_path, content).map_err(CliError::IoError)?;
                println!("{} Project template: {}", "✓".green(), file_path.display());
            }
        }
        
        println!("{} Built-in templates exported successfully!", "✓".green().bold());
        Ok(())
    }
}

