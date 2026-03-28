pub mod module {
    use crate::core::{
        core_add_unit_module::module_unit, core_add_unit_project::project_unit,
        core_generate_project::project, core_utils::utils,
    };
    use crate::templates::template_manager::TemplateManager;
    use crate::error::CliError;
    use std::{fs, path::PathBuf};

    /// Resolves template content intelligently (named template → disk convention → built-in)
    fn get_template_content(
        template_manager: &mut TemplateManager,
        component: &str,
        template_name: Option<&str>,
    ) -> Result<String, CliError> {
        // 1. If --template was given, try that template first
        if let Some(name) = template_name {
            if let Some(content) = utils::resolve_custom_template(template_manager, name, component) {
                return Ok(content);
            }
            eprintln!(
                "⚠️  Component '{}' not found in template '{}' — using built-in.",
                component, name
            );
        } else {
            // 2. Disk convention: template named "module-{component}"
            if let Some(content) = utils::resolve_custom_template(
                template_manager,
                &format!("module-{}", component),
                component,
            ) {
                return Ok(content);
            }
        }
        // 3. Built-in fallback
        let builtin = match component {
            "controller" => include_str!("../templates/module/controller.pas"),
            "service"    => include_str!("../templates/module/service.pas"),
            "repository" => include_str!("../templates/module/repository.pas"),
            "interface"  => include_str!("../templates/module/interface.pas"),
            "infra"      => include_str!("../templates/module/infra.pas"),
            "module"     => include_str!("../templates/module/module.pas"),
            "handler"    => include_str!("../templates/module/handler.pas"),
            _ => return Err(CliError::ValidationError(format!("Unknown component: {}", component))),
        };
        Ok(builtin.to_string())
    }

    /// Generates the requested files for a Nidus module under `src/modules/<module_name>/`
    ///
    /// # Parameters
    /// - `src_path`: base path of the project `src/` directory
    /// - `module_name`: module name (e.g. "config")
    /// - `components`: list of components to generate ("controller", "service", etc.) or "all"
    /// - `dry_run`: if true, only previews what would be generated without writing to disk
    pub fn generate_module_structure(
        src_path: PathBuf,
        module_name: &str,
        components: &[&str],
        overwrite: bool,
        template_name: Option<&str>,
        dry_run: bool,
    ) -> std::io::Result<()> {
        use colored::*;

        let module_dir: PathBuf = src_path.join("modules").join(module_name.to_lowercase());
        if !dry_run {
            fs::create_dir_all(&module_dir)?;
        }

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

        // Initialize the TemplateManager
        let templates_dir = utils::get_templates_directory().map_err(|e| {
            std::io::Error::other(format!("Template directory error: {}", e))
        })?;
        let mut template_manager = TemplateManager::new(templates_dir).map_err(|e| {
            std::io::Error::other(format!("TemplateManager error: {}", e))
        })?;

        let mut skipped_files: Vec<PathBuf> = Vec::new();

        for comp in to_generate.iter().copied() {
            let filename = format!(
                "{}{}.pas",
                utils::camel_case(module_name),
                utils::camel_case(comp)
            );
            let filepath = module_dir.join(&filename);

            if dry_run {
                // In dry-run mode, just show what would be created
                println!("  → Would create: {}", filepath.display());
                created_files.push((filepath, 0, String::new()));
                continue;
            }

            // Skip file if it already exists and --overwrite was not passed
            if filepath.exists() && !overwrite {
                println!(
                    "  {} {} (use --overwrite to replace)",
                    "⏭  Skipping existing:".yellow(),
                    utils::path_to_unix_style(&filepath)
                );
                skipped_files.push(filepath);
                continue;
            }

            // Resolve template content (named template → disk convention → built-in)
            let template_content = get_template_content(&mut template_manager, comp, template_name).map_err(|e| {
                std::io::Error::other(format!("Template error: {}", e))
            })?;

            let content = template_content.replace("{{mod}}", &mod_camel_case);

            created_files.push(utils::write_file_with_stats(&filepath, &content)?);
        }

        if dry_run {
            // Dry-run summary — nothing was written
            println!(
                "\n🔍 Dry run — {} file(s) would be created (no changes made)",
                created_files.len()
            );
            return Ok(());
        }

        // 🔍 Only module and handler are registered in AppModule — intentional behavior.
        // Controller/Service/Repository/Infra are registered INSIDE the Module itself
        // via Binds(), following the Nidus IoC pattern. AppModule must not know the
        // internal details of each module.
        let generated_for_appmodule: Vec<&str> = to_generate
            .iter()
            .filter(|c| **c == "module" || **c == "handler")
            .copied()
            .collect();

        // 📌 Build the units list to add to .dpr (all generated files)
        let units_with_paths: Vec<(String, PathBuf)> = created_files
            .iter()
            .map(|(path, _, _)| {
                let unit_name: String = path.file_stem().unwrap().to_string_lossy().to_string();
                (unit_name, path.clone())
            })
            .collect();

        // 📄 Locate the .dpr and add the generated units
        let dpr_path: PathBuf = project::ensure_project_dpr_exists()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        // ✍️ Add units to .dpr
        project_unit::add_units_to_dpr(&dpr_path, &units_with_paths)?;

        // 📝 Update AppModule
        module_unit::update_app_module(module_name, &generated_for_appmodule)?;

        // 📊 Summary
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
        if !skipped_files.is_empty() {
            println!(
                "{} {}",
                "⏭  Files skipped (already exist):".bold(),
                skipped_files.len()
            );
        }

        Ok(())
    }
}
