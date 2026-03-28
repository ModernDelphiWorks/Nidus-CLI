# Nidus-CLI — Project Knowledge

Este arquivo é lido por todos os agentes da pipeline. Contém o conhecimento
necessário para trabalhar neste projeto.

---

## O que é o projeto

**Nidus-CLI** é uma ferramenta de linha de comando escrita em Rust para geração
de código e gerenciamento de dependências de projetos Delphi que usam o
framework Nidus (IoC inspirado em NestJS).

Repositório: <https://github.com/ModernDelphiWorks/Nidus-CLI>

---

## Stack

| Tecnologia | Versão | Uso |
| ----------- | -------- | --- |
| Rust | stable | linguagem principal |
| clap 4.5 | derive + cargo features | parsing de argumentos CLI |
| clap_complete | 4.5 | geração de shell completions |
| git2 | 0.20 (vendored) | operações git (clone, fetch, diff) |
| serde / serde_json | 1.0 | serialização de JSON |
| colored | 3.0 | output colorido no terminal |
| dialoguer | 0.11 | menus interativos (TTY) |
| chrono | 0.4 | timestamps no lockfile |
| walkdir | 2.5 | varredura de diretórios |
| xmltree | 0.10 | manipulação de arquivos .dproj |

---

## Arquitetura

```text
src/
├── main.rs                  ← entry point, build_cli(), dispatch de comandos
├── lib.rs                   ← exports públicos
├── error.rs                 ← CliError (thiserror)
├── validation.rs            ← validate_git_url(), validate_name()
├── commands/
│   ├── mod.rs               ← pub mod de cada comando
│   ├── command_trait/
│   │   └── cmd_trait.rs     ← trait CliCommand { arg(), command(), execute() }
│   ├── cmd_new.rs           ← Nidus new
│   ├── cmd_init.rs          ← Nidus init
│   ├── cmd_install.rs       ← Nidus install (+ clone_repository_quiet pub(crate))
│   ├── cmd_update.rs        ← Nidus update
│   ├── cmd_gen.rs           ← Nidus gen
│   ├── cmd_remove.rs        ← Nidus remove / rm
│   ├── cmd_doctor.rs        ← Nidus doctor (--fix, --json)
│   ├── cmd_clean.rs         ← Nidus clean
│   ├── cmd_deps.rs          ← Nidus deps
│   ├── cmd_outdated.rs      ← Nidus outdated
│   ├── template.rs          ← Nidus template (subcomandos)
│   └── options/
│       ├── option_info.rs   ← Nidus info
│       ├── option_pattern.rs
│       └── option_template.rs
├── core/
│   ├── core_generate_module.rs   ← geração de arquivos .pas de módulo
│   ├── core_generate_project.rs  ← geração de projeto (.dpr, AppModule, .gitignore)
│   ├── core_add_paths_dproj.rs   ← atualiza DCC_UnitSearchPath no .dproj
│   ├── core_add_unit_module.rs   ← adiciona unit ao .dpr
│   ├── core_add_unit_project.rs
│   ├── core_lockfile.rs          ← read_commit_sha(), write_lock()
│   └── core_utils.rs             ← extract_repo_name(), handle_error(), etc
├── dto/
│   ├── config_global_dto.rs ← ConfigGlobalDTO (carrega nidus.json)
│   ├── cmd_gen_dto.rs       ← CmdGenDTO
│   └── lock_dto.rs          ← NidusLock, LockEntry
└── templates/
    ├── mod.rs
    ├── config.rs
    ├── processor.rs         ← substitui {{mod}}, {{project}}
    ├── template_manager.rs  ← TemplateConfig, TemplateManager
    ├── module/              ← templates .pas de módulo
    └── project/             ← templates .pas de projeto
```

---

## Padrão de implementação de comandos

Todo novo comando segue este padrão:

```rust
// src/commands/cmd_meucomando.rs
use super::command_trait::cmd_trait::CliCommand;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use clap::{Arg, Command};

pub struct CommandMeuComando;

impl CliCommand for CommandMeuComando {
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("meucomando")
            .about("📝 Descrição curta")
            .arg(
                Arg::new("flag")
                    .long("flag")
                    .action(clap::ArgAction::SetTrue)
                    .help("Descrição da flag"),
            )
    }

    fn execute(global_dto: &mut ConfigGlobalDTO, matches: &clap::ArgMatches) {
        // implementação
    }
}
```

Após criar o arquivo:

1. Adicionar `pub mod cmd_meucomando;` em `src/commands/mod.rs`
2. Registrar em `src/main.rs` no `build_cli()` e no dispatch

---

## Configuração do projeto (nidus.json)

```json
{
  "name": "MyApp",
  "mainsrc": "src/",
  "download": "https://github.com/ModernDelphiWorks/Nidus.git",
  "dependencies": {
    "https://github.com/HashLoad/Horse.git": ""
  }
}
```

- `mainsrc`: pasta onde as dependências são clonadas
- `download`: URL do framework principal
- `dependencies`: mapa `url → branch` (branch vazia = default)

---

## Lockfile (nidus.lock)

Gerado automaticamente pelo `install` e `update`. Registra o SHA exato de cada
dependência para builds reproduzíveis. Usado pelo `install --frozen`.

---

## Como rodar testes

```bash
# Todos os testes
cargo test

# Com output detalhado
cargo test -- --nocapture

# Só integração
cargo test --test integration_tests

# Só validação
cargo test --test validation_tests

# Lint
cargo clippy -- -D warnings
```

---

## Convenções

- Mensagens de erro com `eprintln!` + `"❌".red()`
- Mensagens de sucesso com `println!` + `"✅".green()`
- Avisos com `"⚠️".yellow()`
- Informações com `"ℹ️".cyan()`
- Saída JSON sempre via `serde_json::to_string_pretty`
- URLs git validadas com `validate_git_url()` antes de usar
- Operações destrutivas pedem confirmação (exceto com `--yes`/`-y`)
- Padrão dry-run por default para operações que deletam (ex: `clean`)

---

## Testes existentes

- `tests/integration_tests.rs` — testes end-to-end dos comandos principais
- `tests/validation_tests.rs` — testes unitários de validação de input

Ao adicionar funcionalidades, adicione testes em `integration_tests.rs`
seguindo o padrão existente (usa `assert_cmd` + `tempfile`).

---

## GitHub Project — IDs para a pipeline

| Recurso | Valor |
| ------- | ----- |
| Repositório | `ModernDelphiWorks/Nidus-CLI` |
| Projeto número | `11` |
| Project ID | `PVT_kwDOCLPERc4BTEAy` |
| Status field ID | `PVTSSF_lADOCLPERc4BTEAyzhAag84` |

| Status | Option ID |
| ------ | --------- |
| Backlog | `f75ad846` |
| Ready | `61e4505c` |
| In progress | `47fc9ee4` |
| In review | `df73e18b` |
| Done | `98236657` |

## Pipeline de agentes

Os agentes seguem a pipeline em `.claude/pipeline/`. Cada skill lê os relatórios
anteriores antes de executar. O fluxo é:

```text
/task → /implement → /review → /test → /release
```

Cada etapa gera um relatório em `.claude/pipeline/<etapa>-report.md` que serve
de entrada para a etapa seguinte.
