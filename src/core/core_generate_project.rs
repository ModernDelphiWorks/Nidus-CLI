pub mod project {
    use std::{fs, io, path::PathBuf};

    use crate::core::core_utils::utils;

    /// Gera a estrutura inicial de um projeto Nest4d.
    ///
    /// # Parâmetros
    /// - `project_path`: caminho base onde será criada a pasta do projeto
    /// - `project_name`: nome do projeto (e da pasta)
    /// - `include_tests`: se verdadeiro, cria também a pasta `test/`
    pub fn generate_project_structure(
        project_path: PathBuf,
        project_name: &str,
        include_tests: bool,
    ) -> io::Result<()> {
        use colored::*;

        let root = project_path.join(project_name);
        let src_path = root.join("src");
        let modules_path = src_path.join("modules");

        fs::create_dir_all(&modules_path)?;

        let mut created_files: Vec<(PathBuf, u64, String)> = Vec::new();
        let mut created_dirs: Vec<PathBuf> = Vec::new();

        // DPR
        let dpr_content =
            include_str!("../templates/project/project.dpr").replace("<project>", project_name);
        created_files.push(utils::write_file_with_stats(
            &root.join(format!("{}.dpr", project_name)),
            &dpr_content,
        )?);

        // AppModule
        created_files.push(utils::write_file_with_stats(
            &src_path.join("AppModule.pas"),
            include_str!("../templates/project/app_module.pas"),
        )?);

        // Test folder opcional
        if include_tests {
            let test_dir = root.join("test");
            fs::create_dir_all(&test_dir)?;
            created_dirs.push(test_dir);
        }

        // Resumo
        println!("\n{}", "🎯 Project scaffold summary".bold().cyan());
        println!("📁 Root: {}", root.display());

        let mut total_bytes: u64 = 0;
        for (path, bytes, size_str) in &created_files {
            println!(
                "  • {} {}",
                utils::path_to_unix_style(path),
                size_str.yellow()
            );

            total_bytes += *bytes;
        }
        if !created_dirs.is_empty() {
            println!("{}", "📂 Directories created:".bold());
            for dir in &created_dirs {
                println!("  • {}", utils::path_to_unix_style(dir));
            }
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

    pub fn ensure_project_dpr_exists() -> PathBuf {
        // Procura .dpr na pasta atual
        let mut dpr_files: Vec<PathBuf> = std::fs::read_dir(".")
            .unwrap()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension().map(|ext| ext == "dpr").unwrap_or(false) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        if dpr_files.is_empty() {
            eprintln!(
                "❌ Nenhum arquivo .dpr encontrado. Execute primeiro `nest4d new <projeto>`."
            );
            std::process::exit(1);
        }

        // Retorna o primeiro encontrado
        dpr_files.remove(0)
    }
}
