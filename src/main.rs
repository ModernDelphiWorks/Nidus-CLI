use clap::builder::styling::{AnsiColor, Effects};
use clap::builder::Styles;
use clap::{ArgMatches, Command};
use colored::Colorize;
use nest4d::commands::command_trait::cmd_trait::ICommand;
use nest4d::commands::options::{
    option_info::CommandInfo, option_pattern::CommandPattern, option_template::CommandTemplate,
};
use nest4d::dto::config_global_dto::ConfigGlobalDTO;
use nest4d::{
    commands::{
        cmd_add_paths::CommandAddPaths, cmd_gen::CommandGen, cmd_install::CommandInstall,
        cmd_new::CommandNew, cmd_update::CommandUpdate,
    },
    core::core_utils::utils,
};
use std::env;

fn main() {
    // Debug detail
    env::set_var("RUST_BACKTRACE", "1");

    let styles: Styles = Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default());

    let about_text = format!(
        "{}\n{}",
        "⚡ Nest4D CLI".bright_yellow().bold(),
        "Automate Delphi Nest4D code creation for faster development"
            .bright_white()
            .italic()
    );

    let cmd: Command = Command::new("nest4d")
        .version(utils::version_str())
        .styles(styles)
        .bin_name("nest4d")
        .author("Isaque Pinheiro, isaquesp@gmail.com")
        .about(about_text)
        .arg(CommandTemplate::arg())
        .arg(CommandInfo::arg())
        .arg(CommandPattern::arg())
        .subcommand(CommandAddPaths::command())
        .subcommand(CommandNew::command())
        .subcommand(CommandGen::command())
        .subcommand(CommandInstall::command())
        .subcommand(CommandUpdate::command());

    // Match
    let matches: ArgMatches = cmd.get_matches();
    // Create JSON if not found
    utils::check_init_json_exist(&matches);
    // Struct store data
    let mut config_global: ConfigGlobalDTO = ConfigGlobalDTO::new();
    // If options (tags), not executed match
    let is_options: bool = _check_option_tag(&matches, &mut config_global);
    // If not options (tags)
    if !is_options {
        match matches.subcommand() {
            Some(("install", matches)) => CommandInstall::execute(&mut config_global, matches),
            Some(("add-paths", matches)) => CommandAddPaths::execute(&mut config_global, matches),
            Some(("new", matches)) => CommandNew::execute(&mut config_global, matches),
            Some(("gen", matches)) => CommandGen::execute(&mut config_global, matches),
            Some(("update", matches)) => CommandUpdate::execute(&mut config_global, matches),
            _ => unreachable!("clap should ensure we don't get here"),
        };
        // println!("{}", config_global);
    }
}

fn _check_option_tag(matches: &ArgMatches, config_global: &mut ConfigGlobalDTO) -> bool {
    let mut is_options: bool = false;

    match (
        matches.get_flag("info"),
        matches.get_flag("templates"),
        matches.get_flag("pattern"),
    ) {
        (true, _, _) => {
            CommandInfo::execute(config_global, matches);
            is_options = true;
        }
        (_, true, _) => {
            CommandTemplate::execute(config_global, matches);
            is_options = true;
        }
        (_, _, true) => {
            CommandPattern::execute(config_global, matches);
            is_options = true;
        }
        _ => {}
    }

    is_options
}
