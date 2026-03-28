use clap::builder::styling::{AnsiColor, Effects};
use clap::builder::Styles;
use clap::{Arg, ArgAction, ArgMatches, Command};
use clap_complete::{generate, Shell};
use colored::Colorize;
use log::info;
use nidus::commands::command_trait::cmd_trait::CliCommand;
use nidus::commands::options::{
    option_info::CommandInfo, option_pattern::CommandPattern, option_template::CommandTemplate,
};
use nidus::dto::config_global_dto::ConfigGlobalDTO;
use nidus::error::Result;
use nidus::{
    commands::{
        cmd_add_paths::CommandAddPaths, cmd_clean::CommandClean, cmd_deps::CommandDeps,
        cmd_doctor::CommandDoctor, cmd_gen::CommandGen, cmd_init::CommandInit,
        cmd_install::CommandInstall, cmd_new::CommandNew, cmd_outdated::CommandOutdated,
        cmd_remove::CommandRemove, cmd_template::CommandTemplate as CmdTemplate,
        cmd_update::CommandUpdate,
    },
    core::core_utils::utils,
    init_logging,
};
use std::io;

fn build_cli() -> Command {
    let styles: Styles = Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default());

    let about_text = format!(
        "{}\n{}",
        "⚡ Nidus CLI".bright_yellow().bold(),
        "Automate Delphi Nidus code creation for faster development"
            .bright_white()
            .italic()
    );

    Command::new("Nidus")
        .version(utils::version_str())
        .disable_version_flag(true)
        .styles(styles)
        .bin_name("Nidus")
        .author("Isaque Pinheiro, isaquesp@gmail.com")
        .about(about_text)
        .subcommand(CommandInfo::command())
        .subcommand(CommandTemplate::command())
        .subcommand(CommandPattern::command())
        .subcommand(CommandAddPaths::command())
        .subcommand(CommandNew::command())
        .subcommand(CommandGen::command())
        .subcommand(CommandInstall::command())
        .subcommand(CmdTemplate::command())
        .subcommand(CommandUpdate::command())
        .subcommand(CommandDoctor::command())
        .subcommand(CommandRemove::command())
        .subcommand(CommandInit::command())
        .subcommand(CommandClean::command())
        .subcommand(CommandDeps::command())
        .subcommand(CommandOutdated::command())
        .subcommand(
            Command::new("completions")
                .about("🐚 Generate shell completion scripts")
                .long_about(
                    "Outputs a shell completion script to stdout.\n\
                     Add the output to your shell profile to enable tab-completion.\n\n\
                     Examples:\n  \
                     Nidus completions bash >> ~/.bashrc\n  \
                     Nidus completions zsh  >> ~/.zshrc\n  \
                     Nidus completions fish > ~/.config/fish/completions/Nidus.fish",
                )
                .arg(
                    Arg::new("shell")
                        .help("Target shell (bash, zsh, fish, elvish, powershell)")
                        .required(true)
                        .value_parser(["bash", "zsh", "fish", "elvish", "powershell"]),
                ),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::Version)
                .help("Print version"),
        )
}

fn main() {
    init_logging();
    info!("🚀 Starting Nidus-cli v{}", utils::version_str());
    if let Err(e) = run() {
        utils::handle_error(e);
    }
}

fn run() -> Result<()> {
    let mut cmd = build_cli();
    let matches: ArgMatches = cmd.clone().get_matches();

    // Shell completion — handled before loading nidus.json (no project context needed)
    if let Some(("completions", sub)) = matches.subcommand() {
        let shell: Shell = sub
            .get_one::<String>("shell")
            .unwrap()
            .parse()
            .expect("clap value_parser already validated the shell name");
        generate(shell, &mut cmd, "Nidus", &mut io::stdout());
        return Ok(());
    }

    // Only `update` strictly requires an existing nidus.json.
    // `install` handles a missing nidus.json itself — it creates a default one first.
    if matches!(matches.subcommand_name(), Some("update")) {
        utils::check_init_json_exist(&matches)?;
    }

    let mut config_global: ConfigGlobalDTO = ConfigGlobalDTO::new()?;

    match matches.subcommand() {
        Some(("info", sub))      => CommandInfo::execute(&mut config_global, sub),
        Some(("templates", sub)) => CommandTemplate::execute(&mut config_global, sub),
        Some(("pattern", sub))   => CommandPattern::execute(&mut config_global, sub),
        Some(("install", sub))   => CommandInstall::execute(&mut config_global, sub),
        Some(("sync", sub)) | Some(("add-paths", sub)) => CommandAddPaths::execute(&mut config_global, sub),
        Some(("new", sub))       => CommandNew::execute(&mut config_global, sub),
        Some(("gen", sub))       => CommandGen::execute(&mut config_global, sub),
        Some(("template", sub))  => CmdTemplate::execute(&mut config_global, sub),
        Some(("update", sub))    => CommandUpdate::execute(&mut config_global, sub),
        Some(("doctor", sub))    => CommandDoctor::execute(&mut config_global, sub),
        Some(("remove", sub)) | Some(("rm", sub)) => CommandRemove::execute(&mut config_global, sub),
        Some(("init", sub))      => CommandInit::execute(&mut config_global, sub),
        Some(("clean", sub))     => CommandClean::execute(&mut config_global, sub),
        Some(("deps", sub))      => CommandDeps::execute(&mut config_global, sub),
        Some(("outdated", sub))  => CommandOutdated::execute(&mut config_global, sub),
        _ => {}
    }
    Ok(())
}
