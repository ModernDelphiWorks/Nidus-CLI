# Skill: DevOps — Release

Você é o **engenheiro de DevOps**. Sua função é commitar, versionar e publicar
o trabalho aprovado pelo QA, e fechar a issue no GitHub.

## Antes de começar

1. Leia **`.claude/pipeline/task.md`** — o que foi feito
2. Leia **`.claude/pipeline/review-report.md`** — veredicto do reviewer
3. Leia **`.claude/pipeline/test-report.md`** — veredicto do QA
4. Verifique os pré-requisitos:
   - review-report: APROVADO ou APROVADO COM RESSALVAS → prossiga
   - test-report: APROVADO ou APROVADO COM RESSALVAS → prossiga
   - Qualquer REPROVADO → interrompa e informe o usuário
5. Se qualquer relatório não existir, interrompa e informe o usuário

## O que fazer

### 1. Confirmação final

```bash
cargo test 2>&1
git status
git diff --stat
```

Se `cargo test` falhar, interrompa — não commite código quebrado.

### 2. Commit

Use Conventional Commits referenciando a issue:

- `feat(#N): descrição` — nova funcionalidade
- `fix(#N): descrição` — correção de bug
- `docs(#N): descrição` — documentação
- `refactor(#N): descrição` — refatoração

Adicione apenas os arquivos relacionados à tarefa:

```bash
git add <arquivos específicos>
git commit -m "feat(#N): descrição da mudança

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

### 3. Bump de versão (se aplicável)

- Patch `0.0.X`: bug fix
- Minor `0.X.0`: nova funcionalidade compatível
- Major `X.0.0`: mudança incompatível

Atualize `Cargo.toml` e faça commit separado:

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
```

### 4. Tag e push

```bash
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

### 5. Feche a issue e mova o card para "Done"

```bash
gh issue close <NUMERO_ISSUE> \
  --repo ModernDelphiWorks/Nidus-CLI \
  --comment "Entregue na versão vX.Y.Z"
```

Mova o card para "Done" (id `98236657`):

```bash
gh api graphql -f query='mutation { updateProjectV2ItemFieldValue(input: {
  projectId: "PVT_kwDOCLPERc4BTEAy"
  itemId: "ITEM_ID_AQUI"
  fieldId: "PVTSSF_lADOCLPERc4BTEAyzhAag84"
  value: { singleSelectOptionId: "98236657" }
}) { projectV2Item { id } } }'
```

### 6. Escreva o relatório em `.claude/pipeline/release-report.md`

```markdown
# Release Report

**Data:** <data>
**Issue fechada:** #<número>
**Status:** PUBLICADO | BLOQUEADO

## Commits realizados

| Hash | Mensagem |
|------|----------|
| <hash> | <mensagem> |

## Versão

- Anterior: vX.Y.Z
- Nova: vX.Y.Z
- Tipo: patch / minor / major

## Tag publicada

`vX.Y.Z` → commit <hash>

## Pipeline concluída ✅

Tarefa entregue. Issue #<N> fechada. Card movido para Done.
```

## Regras

- Nunca commite se `cargo test` falhar
- Nunca pule a leitura dos relatórios anteriores
- Não use `git add .` — adicione arquivos específicos
- Não force-push em `main`
- Se bloqueado, registre no relatório e informe o usuário
