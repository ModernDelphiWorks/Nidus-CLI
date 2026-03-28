# Nidus-CLI — Plano de Melhorias

> Estado atual: **151 testes passando**, 0 falhas, 0 warnings de clippy.
> **Nenhuma alteração será feita sem aprovação prévia.**

---

## Fluxo correto do Nidus (referência para todo o plano)

```text
1. Nidus new MyApp   → cria estrutura do projeto (.dpr, AppModule.pas, src/modules/)
2. Nidus install     → cria nidus.json + clona framework/deps + sync automático no .dproj
3. Nidus gen ...     → gera componentes + adiciona units ao .dpr
4. Nidus update      → atualiza repos (pull) + sync automático se houver pastas novas
```

`sync` continua existindo como comando autônomo para uso manual/forçado,
mas **não precisa mais ser chamado explicitamente** pelo usuário no fluxo normal.

---

## Grupo A — Bugs (algo declarado que não funciona)

---

### A1 · `install` — credenciais hardcoded + branch ignorado

**Arquivo:** `src/commands/cmd_install.rs`

**Problema atual:**

```rust
// Sempre tenta SSH agent — falha em repos HTTPS públicos
callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_agent("git"));

// version é exibido no log mas nunca passado ao RepoBuilder
let full_command = format!("git clone {} -b {} {}", repo_url, version, dest);
builder.clone(repo_url, Path::new(&destination_folder))  // branch ignorado
```

**O que corrigir:**

1. Credenciais — aplicar o mesmo padrão já usado em `cmd_update.rs`:

```rust
callbacks.credentials(|_url, username, allowed| {
    if allowed.is_ssh_key() {
        git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
    } else {
        git2::Cred::default()
    }
});
```

1. Branch — passar `version` ao `RepoBuilder` quando não estiver vazio:

```rust
if !version.is_empty() {
    builder.branch(version);
}
```

**Impacto:** Repos HTTPS públicos (GitHub sem SSH configurado) falham hoje.
**Risco:** Baixo — mesma lógica já validada em `update`.
**Testes a adicionar:** 1 teste de clone via HTTPS simulado com `file://`.

---

### A2 · `--overwrite` e `--flat` em `gen` — flags parseadas, nunca usadas

**Arquivo:** `src/core/core_generate_module.rs`, `src/commands/cmd_gen.rs`

**Problema atual:**
Ambas as flags são parseadas, armazenadas no DTO e ignoradas completamente.
`write_file_with_stats` sempre sobrescreve; `flat` nunca afeta a estrutura.

**O que corrigir:**

Para `--overwrite`:

- Sem a flag: se o arquivo já existe → **pular e avisar** (não sobrescrever).
- Com a flag: sobrescrever sem aviso (comportamento atual).

Para `--flat` — **decisão fechada: REMOVER da interface.**

O Nidus segue arquitetura modular (NestJS-inspired). Cada módulo tem sua pasta
em `src/modules/<name>/`. Flat quebraria essa convenção e confundiria o usuário
(passa a flag, nada muda). Remover `--flat` do `Command`, do `parse_generate_dto`
e do `CommandGenerateDTO`.

**Impacto:** Médio — usuário passa `--overwrite` esperando comportamento específico.
**Risco:** Baixo.
**Testes a adicionar:** 2 testes para `overwrite` (pula existente / sobrescreve com flag).

---

## Grupo B — Inconsistências (funciona, mas de forma inesperada)

---

### B1 · `install` não executa `sync` automaticamente

**Arquivos:** `src/commands/cmd_install.rs`, `src/commands/cmd_add_paths.rs`

**Problema atual:**
Após clonar todos os repos, `install` encerra. O usuário precisa rodar
`Nidus sync` manualmente para adicionar os paths ao `.dproj`. Isso é um
passo extra desnecessário: o ambiente só fica 100% pronto para o Delphi
compilar depois do sync.

**O que corrigir:**
Ao final de `CommandInstall::execute`, após o loop de clones, chamar
a lógica de sync automaticamente:

```rust
// Após o loop de clones:
println!("{}", "\n🔗 Updating .dproj search paths...\n".cyan());
if let Err(e) = dproj::update_all_dprojs_in_cwd() {
    eprintln!("{} {}", "⚠️  Could not update .dproj:".yellow(), e);
    // aviso, não erro fatal — .dproj pode não existir ainda
}
```

O sync após install é um **aviso, não erro fatal**: o `.dproj` só existe
depois que o usuário cria o projeto no Delphi IDE. Portanto, se não houver
`.dproj`, o install ainda conclui com sucesso e mostra o aviso.

**Impacto:** Alto — elimina passo manual do fluxo.
**Risco:** Baixo — `update_all_dprojs_in_cwd` já existe e é testada.
**Testes a adicionar:**

- 1 teste: `install` com `.dproj` presente → paths adicionados automaticamente.
- 1 teste: `install` sem `.dproj` → conclui com sucesso (aviso, não falha).

---

### B2 · `update` não executa `sync` automaticamente

**Arquivo:** `src/commands/cmd_update.rs`

**Problema atual:**
`update` faz fast-forward dos repos. Se o upstream trouxer novas subpastas
dentro de `src/` (novos módulos do framework, por exemplo), essas pastas
não ficam no `DCC_UnitSearchPath` do `.dproj` — o Delphi não as encontra.
O usuário precisa lembrar de rodar `sync` manualmente após cada `update`.

**O que corrigir:**
Ao final de `CommandUpdate::execute`, após o loop de atualizações, executar
sync automaticamente (mesma lógica do B1):

```rust
// Após o loop de updates:
println!("{}", "\n🔗 Refreshing .dproj search paths...\n".cyan());
if let Err(e) = dproj::update_all_dprojs_in_cwd() {
    eprintln!("{} {}", "⚠️  Could not refresh .dproj:".yellow(), e);
}
```

**Impacto:** Alto — sem isso, `update` pode deixar o projeto sem compilar.
**Risco:** Baixo — mesma função já testada.
**Testes a adicionar:**

- 1 teste: `update` com nova pasta no repo → path aparece no `.dproj`.

---

### B3 · `sync` usa path hardcoded, ignora `mainsrc` do `nidus.json`

**Arquivo:** `src/commands/cmd_add_paths.rs`

**Problema atual:**

```rust
let dep_paths = collect_dependency_paths("./dependencies")?;
```

O campo `mainsrc` do `nidus.json` (que pode ser `"./libs"`, `"./vendor"`, etc.)
é completamente ignorado. O `install` respeita o `mainsrc`; o `sync` não.
Se B1 e B2 forem implementados, o sync interno também usaria o path errado.

**O que corrigir:**
Ler `mainsrc` do `_global_dto` em vez de hardcodar:

```rust
let mainsrc = _global_dto
    .get_command_install()
    .map(|c| c.mainsrc.as_str())
    .unwrap_or("./dependencies");

let (dproj_files, dep_paths) =
    dproj::find_dproj_and_collect_paths(mainsrc).unwrap_or_default();
```

**Impacto:** Alto — sem isso B1 e B2 usariam path errado para quem customizou `mainsrc`.
**Risco:** Baixo.
**Testes a adicionar:** 1 teste de integração com `mainsrc` customizado.

---

### B4 · `AppModule.pas` — comportamento confirmado correto, falta comentário

**Arquivo:** `src/core/core_generate_module.rs`

**Situação atual — decisão fechada: comportamento está CORRETO.**

Os templates confirmam a arquitetura em dois níveis:

```text
AppModule                              ← conhece só Module + Handler
├── RouteHandlers: [TUserRouteHandler] ← entrada HTTP da aplicação
└── Routes:        [TUserModule]       ← composição de módulos

UserModule                             ← encapsula os detalhes internos
└── Binds: [TUserInfra, TUserRepository, TUserService, TUserController]
```

O `Handler` acessa o `Controller` via `GetNidus.Get<TUserController>` — o IoC
do Nidus resolve porque o `Module` já registrou tudo via `Binds`. O `AppModule`
não precisa conhecer Controller/Service/Repository; registrá-los ali quebraria
o encapsulamento do módulo.

**O que corrigir:**
Nenhuma mudança de comportamento. Apenas adicionar um comentário no código
para evitar que um desenvolvedor futuro "corrija" isso achando que é um bug:

```rust
// Apenas module e handler são registrados no AppModule — comportamento intencional.
// Controller/Service/Repository/Infra são registrados DENTRO do próprio Module
// via Binds(), seguindo o padrão IoC do Nidus. O AppModule não deve conhecer
// os detalhes internos de cada módulo.
let generated_for_appmodule: Vec<&str> = to_generate
    .iter()
    .filter(|c| **c == "module" || **c == "handler")
    .copied()
    .collect();
```

**Impacto:** Nenhum no comportamento. Apenas clareza no código.
**Risco:** Nenhum.

---

### B5 · `ensure_project_dpr_exists()` — seleção arbitrária com múltiplos `.dpr`

**Arquivo:** `src/core/core_generate_project.rs`

**Problema atual:**

```rust
Ok(dpr_files.into_iter().next().expect("checked non-empty above"))
```

Com dois `.dpr` na pasta raiz, pega o primeiro em ordem arbitrária do OS.

**O que corrigir:**
Preferir o `.dpr` cujo nome bate com o nome da pasta atual:

```rust
let cwd_name = std::env::current_dir()
    .ok()
    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

let best = cwd_name
    .and_then(|name| dpr_files.iter().find(|p| {
        p.file_stem().map(|s| s.to_string_lossy() == name).unwrap_or(false)
    }))
    .or_else(|| dpr_files.first())
    .cloned()
    .expect("checked non-empty");

Ok(best)
```

**Impacto:** Baixo (caso raro), mas evita bug silencioso.
**Risco:** Baixo.
**Testes a adicionar:** 1 teste com dois `.dpr` na pasta.

---

## Grupo C — Recursos novos

---

### C1 · `Nidus doctor` — comando de diagnóstico

**Arquivo a criar:** `src/commands/cmd_doctor.rs`

**O que faria:**

```text
Nidus doctor

✅ nidus.json        — encontrado e válido
✅ .dpr              — MyProject.dpr encontrado
✅ .dproj            — MyProject.dproj encontrado
⚠️  dependencies     — 2 de 4 repos clonados (Horse, Nidus ausentes)
⚠️  .dproj paths     — 3 paths ausentes no DCC_UnitSearchPath
✅ AppModule.pas     — encontrado
⚠️  modules          — UserModule gerado mas não registrado no .dpr
```

**Valor:** Hoje o usuário só descobre inconsistências quando o Delphi não compila.
**Impacto:** Alto (experiência do desenvolvedor).
**Risco:** Baixo — apenas leitura, sem alterações.
**Esforço:** Médio-alto.

---

### C2 · `Nidus remove module <name>` — desfazer um `gen`

**Arquivo a criar:** `src/commands/cmd_remove.rs`

**O que faria:**

- Apagar `src/modules/<name>/` e todos os arquivos dentro.
- Remover as units do bloco `uses` do `.dpr`.
- Remover o módulo do `AppModule.pas`.

**Comportamento:** Pede confirmação antes de deletar. Flag `--yes` para skip.

**Impacto:** Médio — útil em desenvolvimento iterativo.
**Risco:** Alto — operação destrutiva. Exige confirmação obrigatória.
**Esforço:** Médio.

---

### C3 · Validação explícita de `nidus.json`

**Arquivo:** `src/dto/config_global_dto.rs`

**Problema atual:**
Erros do serde chegam ao usuário como mensagens internas incompreensíveis.

**O que adicionar:**
Após o parse, validar campos obrigatórios com mensagens acionáveis:

```text
❌ nidus.json inválido: campo "mainsrc" está ausente.
   Adicione: "mainsrc": "./dependencies"
```

**Impacto:** Médio (DX). **Risco:** Baixo. **Esforço:** Baixo.

---

## Resumo e ordem de execução proposta

| # | Item | Grupo | Impacto | Esforço | Decisão necessária |
|---|------|-------|---------|---------|-------------------|
| 1 | A1 — `install` credenciais + branch | Bug | Alto | Baixo | Não |
| 2 | B3 — `sync` lê `mainsrc` do config | Inconsistência | Alto | Baixo | Não |
| 3 | B1 — `install` chama sync automático | Inconsistência | Alto | Baixo | Não |
| 4 | B2 — `update` chama sync automático | Inconsistência | Alto | Baixo | Não |
| 5 | A2 — `--overwrite` funcional | Bug | Médio | Baixo | Semântica de `--flat`? |
| 6 | B5 — seleção do `.dpr` por nome | Inconsistência | Baixo | Baixo | Não |
| 7 | C3 — validação de `nidus.json` | Novo | Médio | Baixo | Não |
| 8 | B4 — `AppModule` componentes | Inconsistência | Médio | Médio | Quais registrar? |
| 9 | C1 — `Nidus doctor` | Novo | Alto | Alto | Não |
| 10 | C2 — `Nidus remove` | Novo | Médio | Alto | Não |

---

## Itens que dependem de decisão sua antes de prosseguir

1. **A2 / `--flat`**: remover da interface ou implementar? Se implementar, qual semântica?
2. **B4 / `AppModule`**: apenas `module`+`handler` no registro está correto para o Nidus?

---

*Atualizado em 2026-03-27 — aguardando aprovação para iniciar implementação.*
