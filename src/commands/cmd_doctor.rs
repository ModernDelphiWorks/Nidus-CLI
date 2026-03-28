use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::cmd_install::clone_repository_quiet;
use super::command_trait::cmd_trait::CliCommand;
use crate::core::core_add_paths_dproj::dproj;
use crate::core::core_utils::utils;
use crate::validation::validate_git_url;
use clap::{Arg, ArgAction, Command};
use colored::*;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct CommandDoctor;

// ── JSON-serializable report types ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum DoctorLevel {
    Ok,
    Warning,
    Error,
    Info,
}

#[derive(Debug, Clone, Serialize)]
struct DoctorFinding {
    level: DoctorLevel,
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    details: Vec<String>,
}

impl DoctorFinding {
    fn ok(code: &str, msg: impl Into<String>) -> Self {
        Self { level: DoctorLevel::Ok, code: code.into(), message: msg.into(), details: vec![] }
    }
    fn warn(code: &str, msg: impl Into<String>) -> Self {
        Self { level: DoctorLevel::Warning, code: code.into(), message: msg.into(), details: vec![] }
    }
    fn error(code: &str, msg: impl Into<String>) -> Self {
        Self { level: DoctorLevel::Error, code: code.into(), message: msg.into(), details: vec![] }
    }
    fn info(code: &str, msg: impl Into<String>) -> Self {
        Self { level: DoctorLevel::Info, code: code.into(), message: msg.into(), details: vec![] }
    }
    fn with_details(mut self, d: Vec<String>) -> Self {
        self.details = d;
        self
    }
}

#[derive(Debug, Serialize)]
struct DoctorSection {
    name: String,
    findings: Vec<DoctorFinding>,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    version: String,
    healthy: bool,
    issues: usize,
    warnings: usize,
    sections: Vec<DoctorSection>,
}

// ── CliCommand impl ────────────────────────────────────────────────────────────

impl CliCommand for CommandDoctor {
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("doctor")
            .about("🩺 Check project health and report inconsistencies")
            .arg(
                Arg::new("json")
                    .long("json")
                    .action(ArgAction::SetTrue)
                    .help("Output report as JSON (useful for CI/CD pipelines)"),
            )
            .arg(
                Arg::new("fix")
                    .long("fix")
                    .action(ArgAction::SetTrue)
                    .help("Automatically fix detected issues where possible"),
            )
    }

    fn execute(global_dto: &mut ConfigGlobalDTO, matches: &clap::ArgMatches) {
        let json_mode = matches.get_flag("json");
        let fix_mode  = matches.get_flag("fix");
        let report = run_doctor_checks(global_dto);

        if json_mode {
            match serde_json::to_string_pretty(&report) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("{} Could not serialize report: {}", "❌".red(), e),
            }
        } else {
            print_report(&report);
        }

        if fix_mode {
            fix_issues(&report, global_dto);
        }
    }
}

// ── Core check logic (returns structured report) ──────────────────────────────

fn run_doctor_checks(global_dto: &mut ConfigGlobalDTO) -> DoctorReport {
    let mut sections: Vec<DoctorSection> = Vec::new();
    let mut issues = 0usize;
    let mut warnings = 0usize;

    // ── A. Configuration ──────────────────────────────────────────────────
    let mut sec_a: Vec<DoctorFinding> = Vec::new();

    let config_loaded = global_dto.get_command_install().is_some();

    if config_loaded {
        sec_a.push(DoctorFinding::ok("A1", "nidus.json found and valid"));
    } else {
        sec_a.push(DoctorFinding::error("A1", "nidus.json not found or invalid"));
        issues += 1;
    }

    if let Some(install) = global_dto.get_command_install() {
        if install.mainsrc.trim().is_empty() {
            sec_a.push(DoctorFinding::error("A2", "nidus.json.mainsrc is empty — add: \"mainsrc\": \"./src\""));
            issues += 1;
        } else {
            sec_a.push(DoctorFinding::ok("A2", format!("nidus.json.mainsrc = \"{}\"", install.mainsrc)));
        }

        if install.download.trim().is_empty() {
            sec_a.push(DoctorFinding::error("A3", "nidus.json.download is empty"));
            issues += 1;
        } else {
            match validate_git_url(&install.download) {
                Ok(_) => sec_a.push(DoctorFinding::ok("A3", "nidus.json.download is a valid git URL")),
                Err(_) => {
                    sec_a.push(DoctorFinding::warn(
                        "A3",
                        format!("nidus.json.download \"{}\" is not a recognised git URL", install.download),
                    ));
                    warnings += 1;
                }
            }
        }

        let dep_count = install.dependencies.keys().filter(|k| *k != &install.download).count();
        sec_a.push(DoctorFinding::info("A4", format!("{} extra dependenc(ies) listed in nidus.json", dep_count)));
    }

    sections.push(DoctorSection { name: "configuration".into(), findings: sec_a });

    // ── B. Project Structure ──────────────────────────────────────────────
    let mut sec_b: Vec<DoctorFinding> = Vec::new();

    let mainsrc = global_dto
        .get_command_install()
        .map(|c| c.mainsrc.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "src".to_string());

    // B1: .dpr exists
    let dpr_path: Option<PathBuf> = match fs::read_dir(".") {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().map(|x| x == "dpr").unwrap_or(false))
            .map(|e| e.path()),
        Err(_) => None,
    };

    match &dpr_path {
        Some(p) => sec_b.push(DoctorFinding::ok(
            "B1",
            format!(".dpr found: {}", p.file_name().unwrap_or_default().to_string_lossy()),
        )),
        None => {
            sec_b.push(DoctorFinding::error("B1", ".dpr not found — run `Nidus new <project>`"));
            issues += 1;
        }
    }

    // B2: .dproj exists
    let dproj_names: Vec<String> = fs::read_dir(".")
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "dproj").unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if !dproj_names.is_empty() {
        sec_b.push(DoctorFinding::ok("B2", format!(".dproj found: {}", dproj_names.join(", "))));
    } else {
        sec_b.push(DoctorFinding::warn("B2", ".dproj not found — open project in Delphi IDE to generate"));
        warnings += 1;
    }
    let dproj_exists = !dproj_names.is_empty();

    // B3: mainsrc directory
    if Path::new(&mainsrc).is_dir() {
        sec_b.push(DoctorFinding::ok("B3", format!("sources directory \"{}\" exists", mainsrc)));
    } else {
        sec_b.push(DoctorFinding::error("B3", format!("sources directory \"{}\" not found", mainsrc)));
        issues += 1;
    }

    // B4: AppModule.pas
    let appmodule_path = Path::new(&mainsrc).join("AppModule.pas");
    if appmodule_path.exists() {
        sec_b.push(DoctorFinding::ok("B4", format!("AppModule.pas found at {}", appmodule_path.display())));
    } else {
        sec_b.push(DoctorFinding::warn(
            "B4",
            format!("AppModule.pas not found at {} — run `Nidus new`", appmodule_path.display()),
        ));
        warnings += 1;
    }

    // B5: modules/ directory
    let modules_dir = Path::new(&mainsrc).join("modules");
    if modules_dir.is_dir() {
        let module_count = fs::read_dir(&modules_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count())
            .unwrap_or(0);
        sec_b.push(DoctorFinding::ok("B5", format!("{}/modules/ exists — {} module(s)", mainsrc, module_count)));
    } else {
        sec_b.push(DoctorFinding::warn("B5", format!("{}/modules/ directory not found", mainsrc)));
        warnings += 1;
    }

    sections.push(DoctorSection { name: "project_structure".into(), findings: sec_b });

    // ── C. Dependencies ───────────────────────────────────────────────────
    let mut sec_c: Vec<DoctorFinding> = Vec::new();

    if let Some(install) = global_dto.get_command_install() {
        let mut not_cloned: Vec<String> = Vec::new();
        let mut not_git: Vec<String> = Vec::new();
        let mut total_deps = 0usize;

        for url in install.dependencies.keys() {
            if url == &install.download { continue; }
            total_deps += 1;
            if let Some(name) = utils::extract_repo_name(url) {
                let dest = format!("{}/{}", mainsrc, name);
                let dest_path = Path::new(&dest);
                if dest_path.exists() {
                    if !dest_path.join(".git").is_dir() {
                        not_git.push(name);
                    }
                } else {
                    not_cloned.push(url.clone());
                }
            }
        }

        if not_cloned.is_empty() {
            sec_c.push(DoctorFinding::ok("C1", format!("all {} extra repo(s) cloned", total_deps)));
        } else {
            sec_c.push(
                DoctorFinding::error(
                    "C1",
                    format!("{}/{} repo(s) not cloned — run `Nidus install`", not_cloned.len(), total_deps),
                )
                .with_details(not_cloned),
            );
            issues += 1;
        }

        let framework_name = utils::extract_repo_name(&install.download);
        let framework_ok = framework_name.as_deref()
            .map(|n| Path::new(&format!("{}/{}", mainsrc, n)).exists())
            .unwrap_or(false);

        if framework_ok {
            sec_c.push(DoctorFinding::ok(
                "C2",
                format!("framework \"{}\" cloned", framework_name.as_deref().unwrap_or("?")),
            ));
        } else {
            sec_c.push(DoctorFinding::warn(
                "C2",
                format!("framework \"{}\" not cloned — run `Nidus install`", framework_name.as_deref().unwrap_or("?")),
            ));
            warnings += 1;
        }

        if not_git.is_empty() {
            sec_c.push(DoctorFinding::ok("C3", "all cloned repos have .git/"));
        } else {
            sec_c.push(
                DoctorFinding::warn("C3", format!("{} dir(s) missing .git/ (manual copy?)", not_git.len()))
                    .with_details(not_git),
            );
            warnings += 1;
        }

        if dproj_exists {
            match dproj::find_dproj_and_collect_paths(&mainsrc) {
                Ok((dproj_paths, expected_paths)) => {
                    let dproj_content = dproj_paths
                        .iter()
                        .filter_map(|p| fs::read_to_string(p).ok())
                        .collect::<Vec<_>>()
                        .join("");
                    let missing: Vec<String> = expected_paths
                        .iter()
                        .filter(|p| !dproj_content.contains(p.as_str()))
                        .cloned()
                        .collect();
                    if missing.is_empty() {
                        sec_c.push(DoctorFinding::ok(
                            "C4",
                            format!("DCC_UnitSearchPath — all {} path(s) synced", expected_paths.len()),
                        ));
                    } else {
                        sec_c.push(
                            DoctorFinding::warn(
                                "C4",
                                format!("DCC_UnitSearchPath — {} path(s) missing (run `Nidus sync`)", missing.len()),
                            )
                            .with_details(missing),
                        );
                        warnings += 1;
                    }
                }
                Err(_) => {
                    sec_c.push(DoctorFinding::warn("C4", "DCC_UnitSearchPath — could not scan dependency paths"));
                    warnings += 1;
                }
            }
        } else {
            sec_c.push(DoctorFinding::info("C4", "DCC_UnitSearchPath — skipped (no .dproj found)"));
        }
    } else {
        sec_c.push(DoctorFinding::info("C0", "dependencies — skipped (nidus.json not loaded)"));
    }

    sections.push(DoctorSection { name: "dependencies".into(), findings: sec_c });

    // ── D. Module Consistency ─────────────────────────────────────────────
    let mut sec_d: Vec<DoctorFinding> = Vec::new();

    if let Some(ref dpr) = dpr_path {
        match check_dpr_units_exist(dpr) {
            Ok((ok, missing)) => {
                if missing.is_empty() {
                    sec_d.push(DoctorFinding::ok(
                        "D1",
                        format!("all {} .dpr unit path(s) exist on disk", ok),
                    ));
                } else {
                    sec_d.push(
                        DoctorFinding::error(
                            "D1",
                            format!("{} .dpr unit path(s) registered but file not found", missing.len()),
                        )
                        .with_details(missing),
                    );
                    issues += 1;
                }
            }
            Err(e) => {
                sec_d.push(DoctorFinding::warn("D1", format!("could not parse .dpr: {}", e)));
                warnings += 1;
            }
        }

        if modules_dir.is_dir() {
            match check_modules_registered_in_dpr(dpr, &modules_dir) {
                Ok((registered, orphans)) => {
                    if orphans.is_empty() {
                        sec_d.push(DoctorFinding::ok(
                            "D2",
                            format!("all {} module dir(s) referenced in .dpr", registered),
                        ));
                    } else {
                        sec_d.push(
                            DoctorFinding::warn(
                                "D2",
                                format!("{} module dir(s) not referenced in .dpr", orphans.len()),
                            )
                            .with_details(orphans),
                        );
                        warnings += 1;
                    }
                }
                Err(e) => {
                    sec_d.push(DoctorFinding::warn("D2", format!("could not verify module registration: {}", e)));
                    warnings += 1;
                }
            }

            if appmodule_path.exists() {
                match check_appmodule_consistency(&appmodule_path, &modules_dir) {
                    Ok((ok, orphans)) => {
                        if orphans.is_empty() {
                            sec_d.push(DoctorFinding::ok(
                                "D3",
                                format!("all {} AppModule.pas module reference(s) have matching dirs", ok),
                            ));
                        } else {
                            sec_d.push(
                                DoctorFinding::warn(
                                    "D3",
                                    format!("{} AppModule.pas module reference(s) without matching dir", orphans.len()),
                                )
                                .with_details(orphans),
                            );
                            warnings += 1;
                        }
                    }
                    Err(e) => {
                        sec_d.push(DoctorFinding::warn("D3", format!("could not verify AppModule.pas: {}", e)));
                        warnings += 1;
                    }
                }
            } else {
                sec_d.push(DoctorFinding::info("D3", "AppModule.pas consistency — skipped (file not found)"));
            }
        } else {
            sec_d.push(DoctorFinding::info("D2", "module registration — skipped (no modules/ directory)"));
            sec_d.push(DoctorFinding::info("D3", "AppModule.pas — skipped (no modules/ directory)"));
        }
    } else {
        sec_d.push(DoctorFinding::info("D1", ".dpr checks — skipped (no .dpr found)"));
        sec_d.push(DoctorFinding::info("D2", "module registration — skipped (no .dpr found)"));
        sec_d.push(DoctorFinding::info("D3", "AppModule.pas — skipped (no .dpr found)"));
    }

    sections.push(DoctorSection { name: "module_consistency".into(), findings: sec_d });

    // ── E. Environment ────────────────────────────────────────────────────
    let mut sec_e: Vec<DoctorFinding> = Vec::new();

    match utils::get_templates_directory() {
        Ok(templates_dir) => {
            let count = fs::read_dir(&templates_dir)
                .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count())
                .unwrap_or(0);
            sec_e.push(DoctorFinding::info(
                "E1",
                format!("{} custom template(s) at {}", count, templates_dir.display()),
            ));
        }
        Err(_) => {
            sec_e.push(DoctorFinding::warn("E1", "could not determine templates directory"));
            warnings += 1;
        }
    }

    sec_e.push(DoctorFinding::info("E2", format!("Nidus CLI version: {}", utils::version_str())));

    sections.push(DoctorSection { name: "environment".into(), findings: sec_e });

    DoctorReport {
        version: utils::version_str().to_string(),
        healthy: issues == 0 && warnings == 0,
        issues,
        warnings,
        sections,
    }
}

// ── Colored console output ────────────────────────────────────────────────────

fn print_report(report: &DoctorReport) {
    println!("{}", "\n🩺 Nidus Doctor — Project Health Check\n".bold().cyan());

    let section_labels = [
        "── A. Configuration ──────────────────────────────────────────────",
        "── B. Project Structure ──────────────────────────────────────────",
        "── C. Dependencies ───────────────────────────────────────────────",
        "── D. Module Consistency ─────────────────────────────────────────",
        "── E. Environment ────────────────────────────────────────────────",
    ];

    for (i, section) in report.sections.iter().enumerate() {
        if let Some(label) = section_labels.get(i) {
            println!("{}", label.dimmed());
        } else {
            println!("{}", format!("── {}. {} ──", (b'A' + i as u8) as char, section.name).dimmed());
        }

        for f in &section.findings {
            let icon = match f.level {
                DoctorLevel::Ok      => "✅ ",
                DoctorLevel::Warning => "⚠️  ",
                DoctorLevel::Error   => "❌ ",
                DoctorLevel::Info    => "ℹ️  ",
            };
            let line = format!("  {}  {:<6} {}", icon, f.code, f.message);
            match f.level {
                DoctorLevel::Error   => println!("{}", line.red()),
                DoctorLevel::Warning => println!("{}", line.yellow()),
                DoctorLevel::Info    => println!("{}", line.dimmed()),
                DoctorLevel::Ok      => println!("{}", line),
            }
            for detail in &f.details {
                println!("        • {}", detail);
            }
        }
        println!();
    }

    println!("{}", "──────────────────────────────────────────────────────────────────".dimmed());
    if report.healthy {
        println!("{}", "  ✅  Project is healthy — no issues found!".bold().green());
    } else {
        if report.issues > 0 {
            println!("{}", format!("  ❌  {} error(s) detected", report.issues).bold().red());
        }
        if report.warnings > 0 {
            println!("{}", format!("  ⚠️   {} warning(s) detected", report.warnings).bold().yellow());
        }
    }
    println!();
}

// ── Auto-fix logic ────────────────────────────────────────────────────────────

fn fix_issues(report: &DoctorReport, global_dto: &mut ConfigGlobalDTO) {
    // Collect fixable findings across all sections
    let fixable: Vec<&DoctorFinding> = report
        .sections
        .iter()
        .flat_map(|s| s.findings.iter())
        .filter(|f| {
            matches!(f.level, DoctorLevel::Error | DoctorLevel::Warning)
                && matches!(f.code.as_str(), "C1" | "C2" | "C4")
        })
        .collect();

    if fixable.is_empty() {
        println!("{}", "🔧 doctor --fix: nothing fixable found.\n".dimmed());
        return;
    }

    println!("{}", "\n🔧 Auto-fixing detected issues...\n".bold().cyan());

    let install = match global_dto.get_command_install() {
        Some(c) => c,
        None => {
            eprintln!("{} Cannot fix: nidus.json not loaded.", "❌".red());
            return;
        }
    };

    let mainsrc = install.mainsrc.trim_end_matches('/').to_string();
    let framework_url  = install.download.clone();
    let framework_branch = install.dependencies.get(&framework_url).cloned().unwrap_or_default();
    let extra_deps: Vec<(String, String)> = install
        .dependencies
        .iter()
        .filter(|(k, _)| **k != framework_url)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let mut fixed = 0usize;
    let mut failed = 0usize;

    for finding in fixable {
        match finding.code.as_str() {
            "C1" => {
                // Clone each missing extra dependency listed in finding.details
                for url in &finding.details {
                    let branch = extra_deps
                        .iter()
                        .find(|(u, _)| u == url)
                        .map(|(_, b)| b.as_str())
                        .unwrap_or("");
                    let name = utils::extract_repo_name(url).unwrap_or_else(|| url.clone());
                    let dest = format!("{}/{}", mainsrc, name);
                    print!("  {} Cloning {}... ", "→".cyan(), name.bold());
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                    if clone_repository_quiet(url, branch, &dest) {
                        println!("{}", "✅".green());
                        fixed += 1;
                    } else {
                        println!("{}", "❌ failed".red());
                        failed += 1;
                    }
                }
            }
            "C2" => {
                // Clone the main framework
                let name = utils::extract_repo_name(&framework_url)
                    .unwrap_or_else(|| "Nidus".to_string());
                let dest = format!("{}/{}", mainsrc, name);
                print!("  {} Cloning framework {}... ", "→".cyan(), name.bold());
                let _ = std::io::Write::flush(&mut std::io::stdout());
                if clone_repository_quiet(&framework_url, &framework_branch, &dest) {
                    println!("{}", "✅".green());
                    fixed += 1;
                } else {
                    println!("{}", "❌ failed".red());
                    failed += 1;
                }
            }
            "C4" => {
                // Sync DCC_UnitSearchPath in all .dproj files
                print!("  {} Syncing .dproj search paths... ", "→".cyan());
                let _ = std::io::Write::flush(&mut std::io::stdout());
                match dproj::update_all_dprojs_in_cwd(&mainsrc) {
                    Ok(_) => {
                        println!("{}", "✅".green());
                        fixed += 1;
                    }
                    Err(e) => {
                        println!("{} {}", "❌".red(), e);
                        failed += 1;
                    }
                }
            }
            _ => {}
        }
    }

    println!();
    println!("{}", "🎯 Fix summary".bold().cyan());
    println!("  {} {}", "Fixed: ".bold(), fixed.to_string().green());
    if failed > 0 {
        println!("  {} {}", "Failed:".bold(), failed.to_string().red());
        println!(
            "  {}",
            "💡 Check network connectivity or run `Nidus install` manually.".dimmed()
        );
    }
    println!();
}

// ── Helper functions ──────────────────────────────────────────────────────────

fn check_dpr_units_exist(dpr_path: &Path) -> io::Result<(usize, Vec<String>)> {
    let content = fs::read_to_string(dpr_path)?;
    let base_dir = dpr_path.parent().unwrap_or(Path::new("."));
    let mut ok = 0usize;
    let mut missing = Vec::new();

    for line in content.lines() {
        if let Some(start) = line.find("in '") {
            let rest = &line[start + 4..];
            if let Some(end) = rest.find('\'') {
                let raw_path = &rest[..end];
                let normalized = raw_path.replace('\\', "/");
                let full_path = base_dir.join(&normalized);
                if full_path.exists() {
                    ok += 1;
                } else {
                    missing.push(normalized);
                }
            }
        }
    }

    Ok((ok, missing))
}

fn check_modules_registered_in_dpr(
    dpr_path: &Path,
    modules_dir: &Path,
) -> io::Result<(usize, Vec<String>)> {
    let content = fs::read_to_string(dpr_path)?;
    let mut registered = 0usize;
    let mut orphans = Vec::new();

    for entry in fs::read_dir(modules_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() { continue; }
        let module_name = entry.file_name().to_string_lossy().to_string();
        let fragment_fwd = format!("modules/{}/", module_name);
        let fragment_bwd = format!("modules\\{}\\", module_name);
        if content.contains(&fragment_fwd) || content.contains(&fragment_bwd) {
            registered += 1;
        } else {
            orphans.push(module_name);
        }
    }

    Ok((registered, orphans))
}

fn check_appmodule_consistency(
    appmodule_path: &Path,
    modules_dir: &Path,
) -> io::Result<(usize, Vec<String>)> {
    let content = fs::read_to_string(appmodule_path)?;
    let mut ok = 0usize;
    let mut orphans = Vec::new();

    let uses_start = content.to_lowercase().find("uses").map(|pos| pos + 4).unwrap_or(0);
    let uses_end = content[uses_start..].find(';').map(|p| uses_start + p).unwrap_or(content.len());
    let uses_block = &content[uses_start..uses_end];

    for token in uses_block.split(',') {
        let raw = token.find('{').map(|i| &token[..i]).unwrap_or(token).trim();
        let unit_name = raw.find(" in '").map(|i| &raw[..i]).unwrap_or(raw).trim();

        if !unit_name.ends_with("Module") { continue; }
        // Skip qualified names like Nidus.App.Module
        if unit_name.contains('.') { continue; }
        if !unit_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) { continue; }

        let prefix = unit_name.trim_end_matches("Module");
        if prefix.is_empty() { continue; }

        let dir_name = prefix.to_lowercase();
        if modules_dir.join(&dir_name).is_dir() {
            ok += 1;
        } else {
            orphans.push(format!("{} → modules/{}/", unit_name, dir_name));
        }
    }

    Ok((ok, orphans))
}
