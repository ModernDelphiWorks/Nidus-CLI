# Nidus CLI

🚀 **Nidus Framework for Delphi** - A modern command-line tool for rapid Delphi application development inspired by NestJS.

## 📋 Table of Contents

- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Available Commands](#-available-commands)
- [Project Structure](#️-project-structure)
- [Examples](#-examples)
- [Configuration](#️-configuration)
- [Contributing](#-contributing)
- [License](#-license)

## 🔧 Installation

### Prerequisites

- Delphi 10.3 or higher
- Git installed
- Windows (support for other platforms in development)

### Installation via Cargo

```bash
cargo install Nidus-cli
```

### Installation via Binary

Download the latest binary from the [releases page](https://github.com/ModernDelphiWorks/Nidus-cli/releases) and add it to your PATH.

## 🚀 Quick Start

### 1. Create a new project

```bash
Nidus new MyApp --path ./
```

### 2. Install dependencies

```bash
cd MyApp
Nidus install
```

### 3. Generate a module

```bash
Nidus gen module User
```

### 4. Generate specific components

```bash
Nidus gen controller User
Nidus gen service User
Nidus gen repository User
```

## 📚 Available Commands

### `new` - Create new project

```bash
Nidus new <name> [--path <path>] [--with-tests]
```

**Arguments:**
- `<name>`: Project name (required)

**Options:**
- `--path`: Path where to create the project (must start with `./`)
- `--with-tests`: Include test structure

### `gen` - Generate components

```bash
Nidus gen <type> <name> [options]
```

**Available types:**
- `module`: Generates a complete module
- `controller`: Generates only the controller
- `service`: Generates only the service
- `repository`: Generates only the repository
- `interface`: Generates only the interface
- `infra`: Generates only the infrastructure
- `handler`: Generates only the handler
- `scaffold`: Generates complete structure
- `all`: Generates all components

**Options:**
- `--flat`: Don't create subfolder for the module
- `--path <path>`: Specific path for generation
- `--overwrite`: Overwrite existing files

### `install` - Install framework dependencies

```bash
Nidus install
```

Clones all dependency repositories listed in `nidus.json` into `./dependencies`.

> **Note:** To install custom code-generation templates, use `Nidus template install <source>`.

### `update` - Update framework dependencies

```bash
Nidus update
```

Fetches updates for all dependency repositories in `./dependencies`.

> **Note:** To update custom code-generation templates, use `Nidus template update`.

### `sync` - Sync dependency paths to Delphi project

```bash
Nidus sync
```

Adds `src`/`Source` paths from `./dependencies` to the `.dproj` search path. Previously named `add-paths` (still works as alias).

### `info` - Show CLI environment details

```bash
Nidus info
```

### `templates` - List built-in template types

```bash
Nidus templates
```

### `pattern` - Show architectural pattern reference

```bash
Nidus pattern
```

### Other commands

```bash
Nidus --version    # Shows version
Nidus --help       # Shows help
```

## 🏗️ Project Structure

A typical Nidus project has the following structure:

```
MyApp/
├── MyApp.dpr              # Main project file
├── Nidus.json            # Project configuration
├── src/
│   ├── AppModule.pas       # Main application module
│   └── modules/
│       └── user/
│           ├── UserModule.pas
│           ├── UserController.pas
│           ├── UserService.pas
│           ├── UserRepository.pas
│           ├── UserInterface.pas
│           └── UserInfra.pas
└── test/                   # Tests (if --with-tests was used)
    └── ...
```

## 💡 Examples

### Creating a complete user module

```bash
# Generates module, controller, service, repository, interface and infra
Nidus gen module User
```

### Generating individual components

```bash
# Only the controller
Nidus gen controller Product

# Service in a specific path
Nidus gen service Order --path ./src/modules/orders

# Overwrite existing file
Nidus gen repository Customer --overwrite
```

### Flat structure (without subfolders)

```bash
# Generates files directly in current folder
Nidus gen module Auth --flat
```

## ⚙️ Configuration

The `Nidus.json` file contains the project configurations:

```json
{
  "name": "Nidus",
  "description": "Nidus Framework for Delphi",
  "version": "1.0.0",
  "homepage": "https://www.nest4f.com.br",
  "srcmain": "src",
  "projects": [],
  "download": "https://github.com/ModernDelphiWorks/Nidus.git",
  "dependencies": [
    "https://github.com/HashLoad/horse.git",

## 🎨 Template System

Nidus CLI uses a standardized template system with the following features:

### Template Variables

All templates use the `{{variable}}` syntax for variable substitution:

- `{{project}}`: Project name (used in project templates)
- `{{mod}}`: Module name (used in module templates)

### Example Template Usage

```pascal
unit {{mod}}Module;

interface

uses
  {{mod}}Controller,
  {{mod}}Service;

type
  T{{mod}}Module = class
  end;

end.
```

### Supported Templates

- **Project Templates**: `project.dpr`, `AppModule.pas`
- **Module Templates**: `module.pas`, `controller.pas`, `service.pas`, `repository.pas`, `interface.pas`, `infra.pas`, `handler.pas`
```

### Adding dependencies

Edit the `dependencies` array in `Nidus.json` and run:

```bash
Nidus install
```

## 🧪 Running Tests

```bash
# Run all tests
cargo test

# Run specific tests
cargo test validation
cargo test integration

# Run with detailed output
cargo test -- --nocapture
```

## 🆕 Recent Improvements

### Version 0.1.0

- ✅ **Simplified Command Syntax**: Clean and intuitive `new` command
  - Usage: `Nidus new MyApp`

- ✅ **Standardized Template System**: All templates now use `{{variable}}` syntax
  - Consistent variable substitution across all templates
  - Better maintainability and extensibility

- ✅ **Dynamic Version Management**: Version is now read from `Cargo.toml`
  - No more hardcoded versions in the code
  - Automatic synchronization between package and CLI version

- ✅ **Enhanced Documentation**: Complete README with examples and usage

- ✅ **Comprehensive Testing**: Integration and validation tests included

## 🤝 Contributing

1. Fork the project
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Local Development

```bash
# Clone the repository
git clone https://github.com/your-repo/Nidus-cli.git
cd Nidus-cli

# Install dependencies
cargo build

# Run tests
cargo test

# Run CLI locally
cargo run -- --help
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- 📖 [Documentation](https://Nidus.dev)
- 🐛 [Report Bug](https://github.com/your-repo/Nidus-cli/issues)
- 💡 [Request Feature](https://github.com/your-repo/Nidus-cli/issues)
- 💬 [Discussions](https://github.com/your-repo/Nidus-cli/discussions)

## 🎯 Roadmap

- [ ] Support for Linux and macOS
- [ ] Customizable templates
- [ ] IDE integration
- [ ] Automatic documentation generation
- [ ] Docker support
- [ ] Plugin system

---

**Made with ❤️ for the Delphi community**
