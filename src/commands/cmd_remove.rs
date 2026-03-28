use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::CliCommand;
use crate::core::core_utils::utils;
use crate::error::CliError;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::*;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

pub struct CommandRemove;

impl CliCommand for CommandRemove {
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("remove")
            .about("🗑  Remove a generated module")
            .visible_alias("rm")
            .subcommand(
                Command::new("module")
                    .about("Remove a module and all its generated files")
                    .arg(
                        Arg::new("name")
                            .help("Module name to remove")
                            .required(true)
                            .value_name("MODULE_NAME"),
                    )
                    .arg(
                        Arg::new("yes")
                            .long("yes")
                            .short('y')
                            .action(ArgAction::SetTrue)
                            .help("Skip confirmation prompt"),
                    )
                    .arg(
                        Arg::new("path")
                            .long("path")
                            .value_name("PATH")
                            .help("Custom path to src folder (default: ./src)"),
                    )
                    .arg_required_else_help(true),
            )
            .arg_required_else_help(true)
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, matches: &ArgMatches) {
        if let Some(("module", sub)) = matches.subcommand() {
            let name = sub.get_one::<String>("name").expect("name is required");
            let skip_confirm = sub.get_flag("yes");
            let src_path = sub
                .get_one::<String>("path")
                .map(String::to_string)
                .unwrap_or_else(|| "./src".to_string());

            if let Err(e) = remove_module(name, &src_path, skip_confirm) {
                utils::handle_error(e);
            }
        }
    }
}

fn remove_module(name: &str, src_path: &str, skip_confirm: bool) -> Result<(), CliError> {
    let module_dir = PathBuf::from(src_path)
        .join("modules")
        .join(name.to_lowercase());

    if !module_dir.exists() {
        return Err(CliError::validation_error(format!(
            "Module directory not found: {}",
            module_dir.display()
        )));
    }

    // List files to be removed
    println!("{}", "\n🗑  The following will be deleted:".bold().red());
    let files: Vec<_> = fs::read_dir(&module_dir)
        .map_err(CliError::IoError)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    for f in &files {
        println!("  • {}", utils::path_to_unix_style(&f.path()));
    }
    println!("  • {}/", utils::path_to_unix_style(&module_dir));

    // Confirmation
    if !skip_confirm {
        print!(
            "\n{} [y/N] ",
            format!("Remove module '{}'?", name).bold().yellow()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() != "y" {
            println!("{}", "Aborted.".dimmed());
            return Ok(());
        }
    }

    // Remove directory
    fs::remove_dir_all(&module_dir).map_err(CliError::IoError)?;

    // Remove units from .dpr
    let dpr_result = remove_units_from_dpr(name, src_path);

    // Remove module from AppModule.pas
    let appmodule_result = remove_from_appmodule(name, src_path);

    println!(
        "{} Module '{}' removed.",
        "✅".green(),
        name.bold()
    );

    if let Err(e) = dpr_result {
        println!("{} Could not update .dpr: {}", "⚠️ ".yellow(), e);
    }
    if let Err(e) = appmodule_result {
        println!("{} Could not update AppModule.pas: {}", "⚠️ ".yellow(), e);
    }

    Ok(())
}

/// Removes all lines from the `uses` block of `.dpr` that contain the module name prefix.
fn remove_units_from_dpr(module_name: &str, _src_path: &str) -> io::Result<()> {
    // Find .dpr in current directory
    let dpr_path = find_dpr_in_cwd()?;

    let content = fs::read_to_string(&dpr_path)?;
    let prefix = utils::camel_case(module_name);

    // Remove lines that declare a unit belonging to this module.
    // Lines may look like:
    //   "  UserModule in 'src/modules/user/UserModule.pas',"   (with path)
    //   "  UserModule,"                                         (bare)
    let module_path_fragment = format!("modules/{}/", module_name.to_lowercase());
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            // Primary: match by directory path inside the in '...' declaration
            let by_path = trimmed.contains(&module_path_fragment);
            // Fallback: match bare or compound unit name (UserModule, User ,  User; etc.)
            let next = trimmed
                .strip_prefix(prefix.as_str())
                .and_then(|r| r.chars().next());
            let by_name = trimmed.starts_with(prefix.as_str())
                && matches!(next, None | Some(' ') | Some(',') | Some(';') | Some('A'..='Z'));
            !(by_path || by_name)
        })
        .collect();

    let mut result = filtered.join("\n") + "\n";

    // Fix trailing comma: when the removed unit was the last one in the `uses` block,
    // the preceding line may now end with ',' instead of ';'. Find the last ',' that
    // is followed only by whitespace/newlines before the `begin` keyword and fix it.
    if let Some(begin_pos) = result.find("\nbegin") {
        let before_begin = &result[..begin_pos];
        if let Some(comma_pos) = before_begin.rfind(',') {
            let between = &before_begin[comma_pos + 1..];
            if between.trim().is_empty() {
                result.replace_range(comma_pos..comma_pos + 1, ";");
            }
        }
    }

    fs::write(&dpr_path, result)?;
    Ok(())
}

/// Removes the module and handler entries from `AppModule.pas`.
fn remove_from_appmodule(module_name: &str, src_path: &str) -> io::Result<()> {
    let appmodule_path = Path::new(src_path).join("AppModule.pas");
    if !appmodule_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&appmodule_path)?;
    let camel = utils::camel_case(module_name);
    let handler_type = format!("T{}RouteHandler", camel);
    let module_type = format!("T{}Module", camel);

    // Remove lines containing the handler or module type references
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| !line.contains(&handler_type) && !line.contains(&module_type))
        .collect();

    fs::write(&appmodule_path, filtered.join("\n") + "\n")?;
    Ok(())
}

fn find_dpr_in_cwd() -> io::Result<PathBuf> {
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "dpr").unwrap_or(false) {
            return Ok(path);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No .dpr file found in current directory",
    ))
}
