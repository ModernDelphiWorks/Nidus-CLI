# Nest4D CLI

🚀 **Nest4D Framework for Delphi** - A modern command-line tool for rapid Delphi application development inspired by NestJS.

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
cargo install nest4d-cli
```

### Installation via Binary

Download the latest binary from the [releases page](https://github.com/your-repo/nest4d-cli/releases) and add it to your PATH.

## 🚀 Quick Start

### 1. Create a new project

```bash
nest4d new --project MyApp --path ./
```

### 2. Install dependencies

```bash
cd MyApp
nest4d install
```

### 3. Generate a module

```bash
nest4d gen module User
```

### 4. Generate specific components

```bash
nest4d gen controller User
nest4d gen service User
nest4d gen repository User
```

## 📚 Available Commands

### `new` - Create new project

```bash
nest4d new --project <name> --path <path> [--with-tests]
```

**Options:**
- `--project, -p`: Project name
- `--path`: Path where to create the project (must start with `./`)
- `--with-tests`: Include test structure

### `gen` - Generate components

```bash
nest4d gen <type> <name> [options]
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

### `install` - Install dependencies

```bash
nest4d install
```

Installs all dependencies defined in `nest4d.json`.

### Other commands

```bash
nest4d --version    # Shows version
nest4d --help       # Shows help
```

## 🏗️ Project Structure

A typical Nest4D project has the following structure:

```
MyApp/
├── MyApp.dpr              # Main project file
├── nest4d.json            # Project configuration
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
nest4d gen module User
```

### Generating individual components

```bash
# Only the controller
nest4d gen controller Product

# Service in a specific path
nest4d gen service Order --path ./src/modules/orders

# Overwrite existing file
nest4d gen repository Customer --overwrite
```

### Flat structure (without subfolders)

```bash
# Generates files directly in current folder
nest4d gen module Auth --flat
```

## ⚙️ Configuration

The `nest4d.json` file contains the project configurations:

```json
{
  "name": "MyApp",
  "description": "My Nest4D application",
  "version": "1.0.0",
  "homepage": "https://github.com/user/myapp",
  "srcmain": "src",
  "projects": ["MyApp.dpr"],
  "download": "https://github.com/user/myapp/archive/main.zip",
  "dependencies": [
    "https://github.com/HashLoad/horse.git",
    "https://github.com/HashLoad/evolution4d.git",
    "https://github.com/HashLoad/injector4d.git"
  ]
}
```

### Adding dependencies

Edit the `dependencies` array in `nest4d.json` and run:

```bash
nest4d install
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

## 🤝 Contributing

1. Fork the project
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Local Development

```bash
# Clone the repository
git clone https://github.com/your-repo/nest4d-cli.git
cd nest4d-cli

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

- 📖 [Documentation](https://nest4d.dev)
- 🐛 [Report Bug](https://github.com/your-repo/nest4d-cli/issues)
- 💡 [Request Feature](https://github.com/your-repo/nest4d-cli/issues)
- 💬 [Discussions](https://github.com/your-repo/nest4d-cli/discussions)

## 🎯 Roadmap

- [ ] Support for Linux and macOS
- [ ] Customizable templates
- [ ] IDE integration
- [ ] Automatic documentation generation
- [ ] Docker support
- [ ] Plugin system

---

**Made with ❤️ for the Delphi community**
