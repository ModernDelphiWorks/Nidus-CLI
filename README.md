# Nidus CLI

🚀 **Nidus Framework for Delphi** — A modern command-line tool for rapid Delphi application development inspired by NestJS.

[![CI](https://github.com/ModernDelphiWorks/Nidus-CLI/actions/workflows/ci.yml/badge.svg)](https://github.com/ModernDelphiWorks/Nidus-CLI/actions/workflows/ci.yml)
[![Release](https://github.com/ModernDelphiWorks/Nidus-CLI/actions/workflows/release.yml/badge.svg)](https://github.com/ModernDelphiWorks/Nidus-CLI/releases)
![Version](https://img.shields.io/badge/version-2.0.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
  - [new](#new)
  - [init](#init)
  - [install](#install)
  - [update](#update)
  - [gen](#gen)
  - [remove](#remove)
  - [sync](#sync)
  - [doctor](#doctor)
  - [deps](#deps)
  - [outdated](#outdated)
  - [clean](#clean)
  - [template](#template)
  - [info](#info)
  - [completions](#completions)
- [Project Structure](#project-structure)
- [nidus.json Configuration](#nidusjson-configuration)
- [nidus.lock](#niduslock)
- [Template System](#template-system)
- [Contributing](#contributing)
- [License](#license)

---

## Installation

### Prerequisites

- Git installed and available in PATH
- Delphi 10.3 or higher (for the generated projects)

### Pre-built binary

Download the latest binary for your platform from the [releases page](https://github.com/ModernDelphiWorks/Nidus-CLI/releases) and add it to your PATH.

| Platform | File                 |
| -------- | -------------------- |
| Linux    | `Nidus-linux`        |
| Windows  | `Nidus-windows.exe`  |
| macOS    | `Nidus-macos`        |

### Build from source

```bash
cargo install --git https://github.com/ModernDelphiWorks/Nidus-CLI
```

### Shell tab-completion

```bash
# Bash
Nidus completions bash >> ~/.bashrc

# Zsh
Nidus completions zsh >> ~/.zshrc

# Fish
Nidus completions fish > ~/.config/fish/completions/Nidus.fish
```

---

## Quick Start

```bash
# 1. Scaffold a new project
Nidus new MyApp

# 2. Install framework dependencies
cd MyApp
Nidus install

# 3. Generate a module
Nidus gen module User

# 4. Check project health
Nidus doctor
```

---

## Commands

### `new`

Scaffold a new Delphi/Nidus project.

```bash
Nidus new <name> [--path <path>] [--with-tests]
```

Creates the full project structure: `.dpr`, `AppModule.pas`, `src/modules/`, `.gitignore`.

| Option          | Description                                                    |
| --------------- | -------------------------------------------------------------- |
| `--path <path>` | Directory where the project will be created (default: `.`)     |
| `--with-tests`  | Also creates a `test/` directory                               |

---

### `init`

Initialize `nidus.json` in an existing Delphi project.

```bash
Nidus init [--download <url>] [--mainsrc <dir>] [--force]
```

Creates `nidus.json` without touching any source files. Use this when adopting Nidus in an existing project.

| Option              | Description                                              |
| ------------------- | -------------------------------------------------------- |
| `--download <url>`  | Main framework git URL (default: official Nidus repo)    |
| `--mainsrc <dir>`   | Sources directory (default: `src/`)                      |
| `--force`           | Overwrite if `nidus.json` already exists                 |

---

### `install`

Clone all dependencies listed in `nidus.json`.

```bash
Nidus install
Nidus install --add <url> [--branch <branch>]
Nidus install --remove <url>
Nidus install --frozen
```

Automatically syncs `.dproj` search paths and writes `nidus.lock` after cloning.

| Option              | Description                                                                                         |
| ------------------- | --------------------------------------------------------------------------------------------------- |
| `--add <url>`       | Register a new dependency and clone it immediately. Rolls back `nidus.json` if the clone fails.     |
| `--branch <branch>` | Branch to use with `--add`                                                                          |
| `--remove <url>`    | Remove a dependency from `nidus.json`                                                               |
| `--frozen`          | Fail if `nidus.lock` is missing or any repo diverges from the locked commit                         |

---

### `update`

Fast-forward dependency repositories to the latest remote commit.

```bash
Nidus update
Nidus update --dep <url-or-name>
```

Updates `nidus.lock` after completion.

| Option                 | Description                                              |
| ---------------------- | -------------------------------------------------------- |
| `--dep <url-or-name>`  | Update only the matching dependency (URL or repo name)   |

---

### `gen`

Generate Delphi components for a Nidus module.

```bash
Nidus gen <type> <name> [options]
```

**Types:**

| Type         | Files generated                                                     |
| ------------ | ------------------------------------------------------------------- |
| `module`     | `XxxModule.pas`, `XxxHandler.pas`                                   |
| `handler`    | `XxxHandler.pas`                                                    |
| `controller` | `XxxController.pas`                                                 |
| `service`    | `XxxService.pas`                                                    |
| `repository` | `XxxRepository.pas`                                                 |
| `interface`  | `XxxInterface.pas`                                                  |
| `infra`      | `XxxInfra.pas`                                                      |
| `scaffold`   | All of the above                                                    |
| `all`        | All of the above                                                    |

**Options:**

| Option                | Description                                                                        |
| --------------------- | ---------------------------------------------------------------------------------- |
| `--path <path>`       | Custom path to `src/` folder (default: `./src`)                                    |
| `--overwrite`         | Overwrite existing files                                                           |
| `--template <name>`   | Use a specific custom template from `~/.Nidus/templates/<name>`                    |
| `--dry-run`           | Preview files that would be generated without writing them                         |
| `--interactive` / `-i`| Select components via an interactive multi-select menu (requires TTY)              |

**Examples:**

```bash
# Generate a module with handler
Nidus gen module User

# Preview without writing
Nidus gen scaffold Order --dry-run

# Choose components interactively
Nidus gen scaffold Payment --interactive

# Use a custom template
Nidus gen module Auth --template jwt-template

# Overwrite existing files
Nidus gen controller Product --overwrite
```

---

### `remove`

Remove a module and clean up the `.dpr` file.

```bash
Nidus remove <name>
# alias:
Nidus rm <name>
```

Deletes the module directory and removes its units from the `.dpr` file.

---

### `sync`

Sync `.dproj` unit search paths.

```bash
Nidus sync
# alias:
Nidus add-paths
```

Adds `src`/`Source` sub-paths from the dependencies directory to the `.dproj` `DCC_UnitSearchPath`, using Windows-style separators as required by Delphi.

---

### `doctor`

Run a project health check.

```bash
Nidus doctor
Nidus doctor --fix
Nidus doctor --json
```

Runs a 5-section health check and reports issues and warnings.

| Section                   | Checks                                                               |
| ------------------------- | -------------------------------------------------------------------- |
| **A. Configuration**      | `nidus.json` validity, `mainsrc`, `download` URL                     |
| **B. Project Structure**  | `.dpr`, `.dproj`, `src/`, `AppModule.pas`, `modules/`                |
| **C. Dependencies**       | Clone status, `.git/` integrity, `DCC_UnitSearchPath` sync           |
| **D. Module Consistency** | `.dpr` unit paths, module registration, `AppModule.pas` references   |
| **E. Environment**        | Custom templates count, CLI version                                  |

| Option   | Description                                                                          |
| -------- | ------------------------------------------------------------------------------------ |
| `--fix`  | Auto-fix detected issues: clones missing deps (C1/C2) and syncs `.dproj` paths (C4)  |
| `--json` | Output a structured JSON report — ideal for CI/CD pipelines                          |

---

### `deps`

List all dependencies with clone and git status.

```bash
Nidus deps
Nidus deps --json
```

Shows clone status, branch, and last commit SHA/date per dependency.

---

### `outdated`

Check if dependencies have new commits without updating.

```bash
Nidus outdated
```

Fetches each dependency's remote and compares with the local HEAD. Reports which repos have new commits — without modifying anything.

---

### `clean`

Remove Delphi build artifacts.

```bash
Nidus clean
Nidus clean --execute
Nidus clean --execute --yes
Nidus clean --path ./MyApp
```

Scans for Delphi build artifacts. Git-tracked files are **never** deleted.

| Removed      | Items                                                                          |
| ------------ | ------------------------------------------------------------------------------ |
| Files        | `*.dcu`, `*.dcp`, `*.bpl`, `*.bpi`, `*.drc`, `*.map`                           |
| Directories  | `Win32/`, `Win64/`, `OSX32/`, `OSX64/`, `__history/`, `__recovery/`            |

| Option          | Description                                     |
| --------------- | ----------------------------------------------- |
| `--execute`/`-x`| Actually delete (default is dry-run)            |
| `--yes`/`-y`    | Skip confirmation prompt                        |
| `--path <dir>`  | Directory to scan (default: current dir)        |

---

### `template`

Manage custom code-generation templates.

```bash
Nidus template list
Nidus template install <source>
Nidus template update [<name>]
Nidus template create <name> [--from <dir>]
Nidus template config <name> [<key> <value>]
Nidus template publish <name> <git-url>
```

| Subcommand             | Description                                                                  |
| ---------------------- | ---------------------------------------------------------------------------- |
| `list`                 | List installed custom templates                                              |
| `install <source>`     | Install a template from a git URL or local path                              |
| `update [name]`        | Update one or all installed templates                                        |
| `create <name>`        | Create a new template scaffold                                               |
| `config <name>`        | Get or set template configuration (persisted to `template.json`)             |
| `publish <name> <url>` | Push a local template to a git remote                                        |

---

### `info`

Show project and CLI information.

```bash
Nidus info
```

Displays the CLI banner, version, and — when `nidus.json` is present — a project summary with framework URL, dependencies, cloned count, and module count.

---

### `completions`

Generate shell tab-completion scripts.

```bash
Nidus completions <shell>
```

Supported shells: `bash`, `zsh`, `fish`, `elvish`, `powershell`.

---

## Project Structure

```text
MyApp/
├── MyApp.dpr               # Main Delphi project file
├── MyApp.dproj             # Delphi IDE project file (generated by IDE)
├── nidus.json              # Nidus configuration
├── nidus.lock              # Dependency lockfile (commit SHAs)
├── .gitignore              # Delphi-aware gitignore
├── src/
│   ├── AppModule.pas       # Application root module
│   └── modules/
│       └── user/
│           ├── UserModule.pas
│           ├── UserHandler.pas
│           ├── UserController.pas
│           ├── UserService.pas
│           ├── UserRepository.pas
│           ├── UserInterface.pas
│           └── UserInfra.pas
└── test/                   # Optional test directory (--with-tests)
```

---

## nidus.json Configuration

```json
{
  "name": "MyApp",
  "description": "My Nidus application",
  "version": "master",
  "homepage": "https://www.isaquepinheiro.com.br/nidus",
  "mainsrc": "src/",
  "projects": [],
  "download": "https://github.com/ModernDelphiWorks/Nidus.git",
  "dependencies": {
    "https://github.com/HashLoad/Horse.git": "",
    "https://github.com/ModernDelphiWorks/ModernSyntax.git": "",
    "https://github.com/ModernDelphiWorks/InjectContainer.git": ""
  }
}
```

| Field          | Description                                                          |
| -------------- | -------------------------------------------------------------------- |
| `mainsrc`      | Relative path to the sources directory                               |
| `download`     | Main framework repository URL                                        |
| `dependencies` | Map of `"url": "branch"` (empty string = default branch)             |

---

## nidus.lock

`nidus.lock` is generated automatically after `install` and `update`. It records the exact commit SHA of each cloned dependency, enabling reproducible builds.

```json
{
  "version": "1",
  "generated_at": "2026-03-28T12:00:00Z",
  "dependencies": {
    "https://github.com/HashLoad/Horse.git": {
      "branch": "master",
      "commit": "abc123def456...",
      "locked_at": "2026-03-28T12:00:00Z"
    }
  }
}
```

Use `Nidus install --frozen` to enforce exact commit matching in CI/CD.

---

## Template System

### Built-in templates

All built-in templates use `{{variable}}` placeholders:

- `{{project}}` — project name (`.dpr` template)
- `{{mod}}` — module name (module templates)

### Custom templates

Install a template from a git repository:

```bash
Nidus template install https://github.com/user/my-template.git
```

Templates are stored in `~/.Nidus/templates/<name>/` and resolved automatically during `gen` if a `module-<component>` convention is found, or explicitly with `--template <name>`.

### Template structure

```text
~/.Nidus/templates/my-template/
├── template.json       # Metadata and configuration
└── *.pas               # Template files with {{mod}} placeholders
```

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m 'feat: add my feature'`
4. Push to the branch: `git push origin feature/my-feature`
5. Open a Pull Request

### Local development

```bash
git clone https://github.com/ModernDelphiWorks/Nidus-CLI.git
cd Nidus-CLI
cargo build
cargo test
cargo run -- --help
```

### Running tests

```bash
# All tests
cargo test

# Integration tests only
cargo test --test integration_tests

# Validation tests only
cargo test --test validation_tests

# With output
cargo test -- --nocapture
```

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Support

- 📖 [Documentation](https://www.isaquepinheiro.com.br/docs/nidus)
- 🐛 [Report a bug](https://github.com/ModernDelphiWorks/Nidus-CLI/issues)
- 💡 [Request a feature](https://github.com/ModernDelphiWorks/Nidus-CLI/issues)
- 💬 [Discussions](https://github.com/ModernDelphiWorks/Nidus-CLI/discussions)

---

Made with ❤️ for the Delphi community
