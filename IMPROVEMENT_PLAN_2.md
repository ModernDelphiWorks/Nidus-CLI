# Nidus-CLI — Plano de Melhorias (Parte 2)

> Estado atual: **151 testes passando**, 0 falhas, 0 warnings de clippy.
> **Nenhuma alteração será feita sem aprovação prévia.**

---

## Resumo e ordem de execução proposta

| # | Item | Categoria | Risco | Esforço |
|---|------|-----------|-------|---------|
| 1 | D5 — all user-facing messages and comments in EN | Requisito | Nenhum | Baixo |
| 2 | D2 — `remove` corrige trailing comma no `.dpr` | Bug | Alto | Baixo |
| 3 | D6 — `template install`/`update` são stubs silenciosos | Bug | Alto | Médio |
| 4 | D7 — `check_init_json_exist` cria `nidus.json` em diretório errado | Bug | Médio | Baixo |
| 5 | D1 — `gen` adota `handle_error` em vez de `process::exit` | Qualidade | Baixo | Baixo |
| 6 | D3 — `ConfigGlobalDTO::default()` remove panic | Qualidade | Baixo | Baixo |
| 7 | D4 — `doctor` verifica paths desatualizados no `.dproj` | Feature | Baixo | Médio |
| 8 | D8 — `Cargo.toml` pronto para `cargo publish` | Publicação | Nenhum | Baixo |

---

## D1 · `cmd_gen.rs` — `process::exit(1)` fora do padrão

**Arquivo:** `src/commands/cmd_gen.rs`

**Problema atual:**

```rust
Err(err) => {
    eprintln!("{} Failed to generate module: {}", "❌".red(), err);
    std::process::exit(1);  // bypassa handle_error e o stack de Result
}
```

Todo o resto da CLI usa `Result<()>` + `utils::handle_error` centralizado.
Este `exit` é o último que foge ao padrão (após o refactor P3 anterior).

**O que corrigir:**

Fazer `execute` retornar o erro em vez de terminar abruptamente:

```rust
Err(err) => {
    utils::handle_error(crate::error::CliError::IoError(
        std::io::Error::other(err.to_string())
    ));
}
```

Ou, mais limpo, propagar como `CliError::IoError` via `?` — requereria
mudar a assinatura de `execute` no trait `CliCommand` para `Result<()>`.
Como essa mudança afetaria todos os comandos, a abordagem conservadora é
usar `handle_error` diretamente (sem mudar o trait).

**Impacto:** Baixo — comportamento visível idêntico, mas código consistente.
**Risco:** Baixo.
**Testes a adicionar:** Nenhum novo — os testes de `gen` existentes cobrem
os caminhos de erro.

---

## D2 · `cmd_remove.rs` — trailing comma pode corromper o `.dpr`

**Arquivo:** `src/commands/cmd_remove.rs`

**Problema atual:**

O `.dpr` gerado pelo Nidus tem este formato no bloco `uses`:

```pascal
uses
  MyProject,
  UserModule in 'src/modules/user/UserModule.pas',
  UserHandler in 'src/modules/user/UserHandler.pas',
  OrderModule in 'src/modules/order/OrderModule.pas',
  OrderHandler in 'src/modules/order/OrderHandler.pas';
```

Ao remover `UserModule` e `UserHandler`, a lógica atual filtra as linhas
e o resultado fica:

```pascal
uses
  MyProject,
  OrderModule in 'src/modules/order/OrderModule.pas',
  OrderHandler in 'src/modules/order/OrderHandler.pas';
```

Isso está correto **somente se** as linhas removidas não eram as últimas.
Se forem as últimas antes do `;`, o arquivo fica:

```pascal
uses
  MyProject,    ← vírgula pendurada — Pascal não compila
```

**O que corrigir:**

Após filtrar as linhas, percorrer o resultado e garantir que:

1. A última unit do bloco `uses` termine com `;` (não `,`).
2. Se uma linha terminava com `,` e a próxima foi removida, promover
   essa linha para terminar com `;`.

Algoritmo sugerido:

```rust
// Após filtrar, re-normalizar separadores do bloco uses
fn fix_uses_trailing_separator(lines: &mut Vec<String>) {
    // Encontra o índice da última linha que é declaração de unit
    // (não termina com "uses", "begin", "end.", etc.)
    // Substitui "," por ";" nessa linha, remove ";" de todas as anteriores
    // que são units (já têm "," — deixa como está).
    //
    // Na prática: encontra a última linha que contém " in '" ou termina com ","
    // e troca para ";".
}
```

**Impacto:** Alto — sem este fix, `remove module` pode gerar `.dpr` que
não compila no Delphi.
**Risco:** Médio — manipulação de texto em arquivo Pascal.
**Testes a adicionar:**
- 1 teste: remove o último módulo da lista → `;` correto na penúltima unit.
- 1 teste: remove módulo do meio → separadores intactos.
- 1 teste: remove único módulo (só ele no uses) → bloco `uses` limpo com `;`.

---

## D3 · `ConfigGlobalDTO::default()` — panic sem mensagem amigável

**Arquivo:** `src/dto/config_global_dto.rs`

**Problema atual:**

```rust
impl Default for ConfigGlobalDTO {
    fn default() -> Self {
        Self::new().expect("Failed to load nidus.json — run `Nidus new <project>` first")
    }
}
```

`Default` nunca é chamado pelo `main.rs` (que usa `::new()` diretamente),
mas existe no código e pode ser invocado em testes futuros ou por código
de terceiros. Se `nidus.json` não existir ou for inválido, produz um `panic`
em vez de uma mensagem de erro legível.

**O que corrigir:**

Remover a impl `Default` completamente — ela não é necessária e é
enganosa, pois implica que criar um `ConfigGlobalDTO` sem argumentos é
uma operação infalível:

```rust
// Remover este bloco inteiro:
impl Default for ConfigGlobalDTO { ... }
```

Se em algum teste for preciso um DTO vazio, criar um construtor
`ConfigGlobalDTO::empty()` explícito que não tenta ler arquivo.

**Impacto:** Nenhum no comportamento atual.
**Risco:** Baixo — verificar se há chamadas a `ConfigGlobalDTO::default()`
no codebase antes de remover.
**Testes a adicionar:** Nenhum — verificação via `grep` antes da remoção.

---

## D4 · `doctor` — verificar paths desatualizados no `.dproj`

**Arquivo:** `src/commands/cmd_doctor.rs`

**Problema atual:**

O `doctor` implementado verifica a existência de arquivos e pastas, mas
não verifica se o `DCC_UnitSearchPath` do `.dproj` está em sincronia com
as dependências clonadas. Este era o item mais valioso do diagnóstico
original:

```text
⚠️  .dproj paths  — 3 paths ausentes no DCC_UnitSearchPath
```

**O que adicionar:**

Após verificar que o `.dproj` existe, ler o `DCC_UnitSearchPath` atual
(via a lógica já existente em `core_add_paths_dproj`) e comparar com
`collect_dependency_paths(mainsrc)`:

```rust
// Paths que deveriam estar mas não estão
let expected = dproj::collect_dependency_paths(mainsrc)?;
let current  = dproj::read_search_paths(dproj_path)?;  // novo helper

let missing: Vec<_> = expected
    .iter()
    .filter(|p| !current.contains(p))
    .collect();

if missing.is_empty() {
    println!("✅  .dproj paths        — all {} paths present", expected.len());
} else {
    println!("⚠️   .dproj paths        — {} path(s) missing (run `Nidus sync`)",
             missing.len());
    for p in &missing {
        println!("      • {}", p);
    }
}
```

Isso requer expor `collect_dependency_paths` como `pub(crate)` e criar
um helper `read_search_paths(dproj_path)` que lê o valor atual do
`DCC_UnitSearchPath` e retorna os itens como `Vec<String>`.

**Impacto:** Alto — é o check mais útil do `doctor` para o usuário real.
**Risco:** Baixo — somente leitura, sem escrita.
**Testes a adicionar:**
- 1 teste de integração: `doctor` com paths em sincronia → mensagem `✅`.
- 1 teste de integração: `doctor` com paths desatualizados → mensagem `⚠️`
  listando os ausentes.

---

## D5 · All user-facing messages and code comments in EN

**Escopo:** todos os arquivos `src/`

**Motivação:**

O Nidus precisa ser acessível para desenvolvedores americanos e
internacionais. Qualquer comentário ou mensagem em PT-BR é uma barreira
de entrada — tanto para quem quer usar o framework quanto para quem quer
contribuir com a CLI.

**O que cobrir:**

1. **Mensagens ao usuário** (`println!`, `eprintln!`, strings de `about`,
   `long_about`, `help`) — qualquer texto que aparece no terminal.
2. **Comentários de código** (`//`, `///`) — documentação inline dos módulos.
3. **Strings de erro** (`CliError::validation_error(...)`,
   `CliError::ProjectNotFound` etc.) — mensagens que chegam ao usuário
   via `handle_error`.

**Não alterar:**

- Nomes de variáveis, funções, structs, enums — refactor de naming é
  escopo separado.
- Lógica de qualquer tipo.

**Arquivos com maior concentração de PT-BR:**

| Arquivo | Tipo de ocorrência |
| ------- | ------------------ |
| `cmd_install.rs` | comentários em todos os blocos do loop |
| `cmd_update.rs` | comentários nos testes unitários internos |
| `core_add_paths_dproj.rs` | docstrings `🇧🇷` / `🇺🇸` duplicadas |
| `core_generate_module.rs` | comentários no loop de geração |
| `core_generate_project.rs` | comentários nas seções DPR e AppModule |
| `core_utils.rs` | `warn!` e `info!` de log em PT |
| `config_global_dto.rs` | comentários em `load_config_from_file` |
| `validation.rs` | strings de erro de validação |

**Impacto:** Nenhum no comportamento.
**Risco:** Nenhum.
**Testes a adicionar:** Nenhum.

---

---

## D6 · `template install` e `template update` são stubs silenciosos

**Arquivo:** `src/commands/template.rs`

**Problema atual:**

Estes dois subcommands existem, aceitam argumentos, imprimem mensagens
de sucesso — mas **não fazem absolutamente nada**:

```rust
// install_template()
// TODO: Implementar download real
// TODO: Implementar instalação de template
println!("{} Template '{}' installed successfully!", "✓".green().bold(), template_name);
Ok(())   // ← retorna Ok sem instalar nada
```

```rust
// update_templates()
// TODO: Implementar atualização real
println!("{} Update completed!", "✓".green().bold());
Ok(())   // ← retorna Ok sem atualizar nada
```

Um desenvolvedor americano que leia `Nidus template install <url>` e veja
`✓ Template installed successfully!` vai assumir que o template foi
instalado — e vai perder tempo depurando por que o template não aparece.

**O que corrigir:**

Duas opções:

**Opção A (conservadora) — retornar erro honesto:**

```rust
fn install_template(...) -> Result<(), CliError> {
    Err(CliError::ValidationError(
        "template install is not yet implemented. \
         To use a custom template, copy the .pas files to ~/.Nidus/templates/<name>/".to_string()
    ))
}
```

**Opção B (completa) — implementar usando git2:**

- `install`: `git clone <source>` em `~/.Nidus/templates/<name>/`
- `update`: iterar pastas em `~/.Nidus/templates/` que sejam repositórios git
  e fazer fast-forward (reutilizando `update_repo` de `cmd_update.rs`)

A **Opção A** pode ser implementada imediatamente e é honesta com o usuário.
A **Opção B** é o comportamento correto e reutiliza código já testado.

**Decisão necessária:** implementar A agora e B depois, ou ir direto para B?

**Impacto:** Alto — hoje engana o usuário com falso sucesso.
**Risco:** Baixo para A / Médio para B.

**Testes a adicionar:**

- Opção A: 1 teste verificando que o erro é retornado.
- Opção B: 2 testes com repo local `file://` (mesma abordagem dos testes de `install`).

---

## D7 · `check_init_json_exist` cria `nidus.json` em qualquer diretório

**Arquivo:** `src/core/core_utils.rs`

**Problema atual:**

```rust
pub fn check_init_json_exist(_matches: &clap::ArgMatches) -> Result<()> {
    let path = Path::new("nidus.json");
    if !path.exists() {
        // Gera silenciosamente um nidus.json padrão
        fs::write(path, default_json)?;
        println!("{}", "✅ Default nidus.json created.".green());
    }
    Ok(())
}
```

Isso é chamado em `main.rs` antes de despachar qualquer comando. Se o
usuário rodar `Nidus gen module User` em `~/Desktop/` por engano, a CLI
cria um `nidus.json` com defaults nessa pasta sem pedir permissão.

O comportamento correto para a maioria dos comandos é **falhar com
mensagem clara** se não houver `nidus.json`:

```text
❌ No nidus.json found.
   Run `Nidus new <ProjectName>` to create a new project first.
```

**Exceções** (comandos que não precisam de `nidus.json`):

- `Nidus new` — cria o projeto do zero
- `Nidus doctor` — diagnóstico, deve rodar mesmo sem projeto
- `Nidus template *` — gerenciamento de templates é global

**O que corrigir:**

Verificar o subcommand antes de criar o arquivo. Se o comando não for
um dos acima, retornar erro em vez de criar:

```rust
pub fn check_init_json_exist(matches: &clap::ArgMatches) -> Result<()> {
    // Comandos que não dependem de nidus.json
    let skip = matches.subcommand_name()
        .map(|s| matches!(s, "new" | "doctor" | "template"))
        .unwrap_or(false);

    if skip {
        return Ok(());
    }

    let path = Path::new("nidus.json");
    if !path.exists() {
        return Err(CliError::validation_error(
            "No nidus.json found.\n   Run `Nidus new <ProjectName>` to create a new project first."
        ));
    }
    Ok(())
}
```

**Impacto:** Alto — hoje o comportamento é surpreendente e pode poluir
qualquer diretório do sistema.
**Risco:** Baixo — mudança simples e localizada.

**Testes a adicionar:**

- 1 teste: `gen` sem `nidus.json` → erro com mensagem clara (não cria arquivo).
- 1 teste: `new` sem `nidus.json` → executa normalmente.
- 1 teste: `doctor` sem `nidus.json` → executa normalmente.

---

## D8 · `Cargo.toml` pronto para `cargo publish`

**Arquivo:** `Cargo.toml`

**Estado atual:**

```toml
[package]
name = "nidus"
version = "0.1.0"
edition = "2021"
```

Faltam campos obrigatórios ou fortemente recomendados pelo crates.io:

| Campo | Situação | Valor sugerido |
| ----- | -------- | -------------- |
| `description` | ausente | `"CLI for scaffolding Delphi projects using the Nidus framework"` |
| `license` | ausente | `"MIT"` ou `"Apache-2.0"` |
| `repository` | ausente | URL do repositório GitHub |
| `homepage` | ausente | URL do site ou repositório |
| `keywords` | ausente | `["delphi", "nidus", "scaffold", "codegen", "cli"]` |
| `categories` | ausente | `["development-tools", "command-line-utilities"]` |
| `readme` | ausente | `"README.md"` |
| `version` | `0.1.0` | Considerar `1.0.0` dado o nível de maturidade |

**Dependências possivelmente não usadas** (verificar antes de publicar):

| Crate | Suspeita |
| ----- | -------- |
| `chrono` | Não vi uso no código |
| `uuid` | Não vi uso no código |
| `pathdiff` | Uso incerto |

Dependências sem uso aumentam o tempo de compilação e a superfície de
ataque — o crates.io penaliza pacotes com deps excessivas.

**O que corrigir:**

1. Preencher os campos ausentes no `[package]`.
2. Rodar `cargo +nightly udeps` ou revisar manualmente para remover dependências sem uso.
3. Decidir a versão de lançamento.

**Decisão necessária:** qual licença? qual versão inicial (`0.1.0` ou `1.0.0`)?

**Impacto:** Sem isso o `cargo publish` é rejeitado ou o pacote não aparece nas buscas do crates.io.

**Risco:** Nenhum.

**Testes a adicionar:** Nenhum.

---

## Itens sem decisão pendente

Os itens D5, D2, D1, D3 e D4 podem ser implementados imediatamente.

Os itens abaixo precisam de uma decisão sua antes de prosseguir:

- **D6**: implementar `template install`/`update` como erro honesto (Opção A)
  ou implementação completa via git2 (Opção B)?
- **D8**: qual licença e qual versão de lançamento?

---

*Atualizado em 2026-03-27 — aguardando aprovação para iniciar implementação.*
