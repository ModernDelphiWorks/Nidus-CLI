pub mod module {
    use crate::core::{
        core_add_unit_module::module_unit, core_add_unit_project::project_unit,
        core_generate_project::project, core_utils::utils,
    };
    use crate::templates::template_manager::TemplateManager;
    use crate::error::CliError;
    use std::{fs, path::PathBuf};

    /// Obtém o conteúdo de um template de forma inteligente (disco primeiro, depois embutido)
    fn get_template_content(template_manager: &mut TemplateManager, component: &str) -> Result<String, CliError> {
        // Tenta obter template customizado primeiro
        if let Ok(template) = template_manager.get_template(&format!("module-{}", component)) {
            if let Some(file) = template.files.iter().find(|f| f.name.contains(component)) {
                return Ok(file.content.clone());
            }
        }
        
        // Fallback para templates embutidos
        let builtin_template = match component {
            "controller" => include_str!("../templates/module/controller.pas"),
            "service" => include_str!("../templates/module/service.pas"),
            "repository" => include_str!("../templates/module/repository.pas"),
            "interface" => include_str!("../templates/module/interface.pas"),
            "infra" => include_str!("../templates/module/infra.pas"),
            "module" => include_str!("../templates/module/module.pas"),
            "handler" => include_str!("../templates/module/handler.pas"),
            _ => return Err(CliError::ValidationError(format!("Unknown component: {}", component))),
        };
        
        Ok(builtin_template.to_string())
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

    /// Gera os arquivos solicitados para um módulo Nest4d dentro de `src/modules/<module_name>/`
    ///
    /// # Parâmetros
    /// - `src_path`: caminho base da pasta `src/` do projeto
    /// - `module_name`: nome do módulo (ex: "config")
    /// - `components`: lista de componentes a gerar ("controller", "service", etc) ou "all"
    pub fn generate_module_structure(
        src_path: PathBuf,
        module_name: &str,
        components: &[&str],
    ) -> std::io::Result<()> {
        use colored::*;

        let module_dir: PathBuf = src_path.join("modules").join(module_name.to_lowercase());
        fs::create_dir_all(&module_dir)?;

        let all_components = [
            "module",
            "handler",
            "controller",
            "service",
            "repository",
            "interface",
            "infra",
        ];

        let to_generate: Vec<&str> = if components.contains(&"all") {
            all_components.to_vec()
        } else {
            components.to_vec()
        };

        let mut created_files: Vec<(PathBuf, u64, String)> = Vec::new();
        let mod_camel_case = utils::camel_case(module_name);

        // Inicializa o TemplateManager
        let templates_dir = get_templates_directory().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Template directory error: {}", e))
        })?;
        let mut template_manager = TemplateManager::new(templates_dir).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("TemplateManager error: {}", e))
        })?;

        for comp in to_generate.iter().copied() {
            let filename = format!(
                "{}{}.pas",
                utils::camel_case(module_name),
                utils::camel_case(comp)
            );
            let filepath = module_dir.join(&filename);
            
            // Usa a nova lógica inteligente de templates
            let template_content = get_template_content(&mut template_manager, comp).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Template error: {}", e))
            })?;

            let content = template_content.replace("{{mod}}", &mod_camel_case);

            created_files.push(utils::write_file_with_stats(&filepath, &content)?);
        }

        // 🔍 Filtra só module e handler para o AppModule
        let generated_for_appmodule: Vec<&str> = to_generate
            .iter()
            .filter(|c| **c == "module" || **c == "handler")
            .copied()
            .collect();

        // 📌 Monta unidades para adicionar no .dpr (todas as geradas)
        let units_with_paths: Vec<(String, PathBuf)> = created_files
            .iter()
            .map(|(path, _, _)| {
                let unit_name: String = path.file_stem().unwrap().to_string_lossy().to_string();
                (unit_name, path.clone())
            })
            .collect();

        // 📄 Busca o .dpr e adiciona as units
        let dpr_path: PathBuf = project::ensure_project_dpr_exists();

        // ✍️ Adiciona as units ao .dpr
        project_unit::add_units_to_dpr(&dpr_path, &units_with_paths)?;

        // 📝 Atualiza o AppModule
        module_unit::update_app_module(module_name, &generated_for_appmodule)?;

        // 📊 Resumo
        println!("\n{}", "🧱 Module generation summary".bold().cyan());
        println!("{} {}", "📦 Module:".bold(), module_name);
        println!(
            "{} {}",
            "📁 Dir:".bold(),
            utils::path_to_unix_style(&module_dir)
        );

        let total_bytes: u64 = created_files.iter().map(|(_, b, _)| b).sum();
        for (path, _, size_str) in &created_files {
            println!(
                "  • {} {}",
                utils::path_to_unix_style(path),
                size_str.yellow()
            );
        }
        println!(
            "{} {} ({} {})",
            "✅ Files created:".bold(),
            created_files.len(),
            "total".dimmed(),
            format!("{} bytes", total_bytes).green()
        );

        Ok(())
    }
}
