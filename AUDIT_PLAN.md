# Nidus-CLI — Plano de Auditoria e Melhorias

> Gerado em: 2026-03-26
> Modo: **proposta por proposta** — nenhuma alteração sem confirmação explícita.

---

## Status Geral

| Fase | Status |
|------|--------|
| Phase 1 — Discovery (leitura do código) | ✅ Concluída |
| Phase 2 — cargo test | ✅ Concluída |
| Phase 3 — cargo clippy | ✅ Concluída |
| Phase 4 — Análise de arquitetura | ✅ Concluída |
| Phase 5 — Alinhamento CLI | ✅ Concluída |
| Phase 6 — Aplicação das melhorias | ✅ Concluída |

---

## Resultados dos Diagnósticos

### cargo test
- **8 testes de integração** — 2 falhavam (corrigidos em P1.1)
- **4 testes unitários** — todos passando
- Lacunas: zero cobertura em cmd_install, cmd_update, cmd_template, core_generate_*

### cargo clippy
- **49 erros** com `-D warnings`
- Principais: redundant closures, len_zero, snake_case, new_without_default, get_first

---

## Prioridade 1 — Bugs Críticos (sem breaking change)

| Item | Descrição | Arquivo | Status |
|------|-----------|---------|--------|
| P1.1 | Corrigir `--project` → arg posicional em testes | `tests/integration_tests.rs` | ✅ Aplicado |
| P1.2 | Corrigir `extract_repo_name()` para URLs sem `.git` | `src/core/core_utils.rs` | ✅ Aplicado |
| P1.3 | Remover `env::set_var("RUST_BACKTRACE", "1")` hardcoded | `src/main.rs` | ✅ Aplicado |
| P1.4 | Corrigir prefixo `_` em `_check_option_tag` | `src/main.rs` | ✅ Aplicado |
| P1.5 | Remover `len() < 1` dead code após `is_empty()` | `src/validation.rs` | ✅ Aplicado |

---

## Prioridade 2 — Qualidade de Código

| Item | Descrição | Arquivo | Status |
|------|-----------|---------|--------|
| P2.1 | Extrair `get_templates_directory()` duplicado para utilitário | `core_generate_module.rs`, `core_generate_project.rs` | ✅ Aplicado |
| P2.2 | Extrair `get_template_content()` duplicado | `core_generate_module.rs`, `core_generate_project.rs` | ✅ Aplicado |
| P2.3 | Substituir `.unwrap()` críticos — auto-fixado pelo clippy | múltiplos | ✅ Aplicado |
| P2.4 | Renomear `ICommand` → `CliCommand`; remover `fn new()` do trait | `cmd_trait.rs` + todos os impls | ✅ Aplicado |
| P2.5 | Remover `tokio` e `reqwest` sem uso | `Cargo.toml` | ✅ Aplicado |
| P2.6 | Renomear crate `Nidus` → `nidus` (snake_case, manter binário `Nidus`) | `Cargo.toml` + imports | ✅ Aplicado |
| P2.7 | Corrigir 49 erros do clippy (redundant closures, etc.) | múltiplos | ✅ Aplicado |

---

## Prioridade 3 — Eliminar `process::exit()`

| Item | Descrição | Arquivo | Status |
|------|-----------|---------|--------|
| P3.1 | `println_panic` removida; `check_init_json_exist` → `Result<()>` | `src/core/core_utils.rs` | ✅ Aplicado |
| P3.2 | `ensure_project_dpr_exists` → `Result<PathBuf>` | `src/core/core_generate_project.rs` | ✅ Aplicado |
| P3.3 | `cmd_template::execute` → `handle_error(e)` | `src/commands/cmd_template.rs` | ✅ Aplicado |
| P3.4 | `main()` → padrão `run()` + `handle_error` | `src/main.rs` | ✅ Aplicado |
| P3.5 | `config_global_dto._load_config_from_file` → `Result<()>` | `src/dto/config_global_dto.rs` | ✅ Aplicado |
| P3.6 | `cmd_add_paths` → `handle_error` | `src/commands/cmd_add_paths.rs` | ✅ Aplicado |

> `process::exit` agora existe em **2 lugares**:
> - `core_utils::handle_error` — ponto de saída centralizado (intencional)
> - `cmd_gen.rs:142` — saída após erro de geração (intencional, dentro de closure)

---

## Prioridade 4 — Estrutura CLI (breaking changes)

| Item | Descrição | Arquivo | Status |
|------|-----------|---------|--------|
| P4.1 | Promover `--info`, `--templates`, `--pattern` a subcommands | `src/main.rs` + options/ | ✅ Aplicado |
| P4.2 | Resolver conflito `template install/update` vs raiz `install/update` | `src/commands/` | ✅ Aplicado |
| P4.3 | Renomear `add-paths` → `sync` | `src/commands/cmd_add_paths.rs` | ✅ Aplicado |
| P4.4 | Documentar `gen handler` no README | `README.md` | ✅ Aplicado |

---

## Prioridade 5 — Cobertura de Testes

| Item | Descrição | Arquivo | Status |
|------|-----------|---------|--------|
| P5.1 | Testes de integração corretos para `new` | `tests/integration_tests.rs` | ✅ Incluído em P1.1 |
| P5.2 | Testes para `gen module`, `gen controller`, `gen service` | `tests/integration_tests.rs` | ✅ Aplicado |
| P5.3 | Testes unitários para `extract_repo_name` (com e sem `.git`) | `tests/` | ✅ Aplicado |
| P5.4 | Testes para `template list` e `template info` | `tests/integration_tests.rs` | ✅ Aplicado |

---

## Estado Atual

```
cargo clippy -- -D warnings  →  0 erros
cargo test                   →  27/27 passando (14 integração + 13 unitários)
process::exit espalhados     →  10 → 2 (apenas pontos controlados)
```

## ✅ Auditoria concluída — todos os itens aplicados
