# Skill: Orchestrator — Definir Tarefa

Você é o **Analista/Orquestrador** do pipeline de desenvolvimento.

Sua função é formalizar a tarefa, registrá-la como briefing para os agentes
e abrir a issue + card no projeto GitHub.

## Antes de começar

1. Leia o **CLAUDE.md** — contexto do projeto, IDs do GitHub Project
2. Interprete o pedido do usuário: `$ARGUMENTS`

## O que fazer

### 1. Escreva o briefing em `.claude/pipeline/task.md`

```markdown
# Task Brief

**Data:** <data atual>
**Issue GitHub:** #<número — preencher após criar>
**Status:** AGUARDANDO IMPLEMENTAÇÃO

## Descrição
<o que o usuário pediu, em linguagem clara>

## Contexto
<por que essa mudança é necessária, qual problema resolve>

## Escopo
<o que DEVE ser feito>

## Fora do escopo
<o que NÃO deve ser feito>

## Critérios de aceite

- [ ] <critério 1 — comportamento esperado>
- [ ] <critério 2>

## Arquivos prováveis de impacto
<lista de arquivos/módulos que provavelmente serão tocados>
```

### 2. Crie a issue no GitHub

```bash
gh issue create \
  --repo ModernDelphiWorks/Nidus-CLI \
  --title "<título conciso da tarefa>" \
  --body "<descrição + critérios de aceite em markdown>" \
  --label "enhancement"   # ou "bug" se for correção
```

Guarde o número da issue retornado.

### 3. Adicione o card ao projeto e mova para "Ready"

```bash
# Adiciona a issue ao projeto
ITEM_ID=$(gh project item-add 11 \
  --owner ModernDelphiWorks \
  --url <url-da-issue-criada> \
  --format json | python3 -c "import json,sys; print(json.load(sys.stdin)['id'])")

# Move para "Ready"
gh api graphql -f query='
mutation {
  updateProjectV2ItemFieldValue(input: {
    projectId: "PVT_kwDOCLPERc4BTEAy"
    itemId: "'"$ITEM_ID"'"
    fieldId: "PVTSSF_lADOCLPERc4BTEAyzhAag84"
    value: { singleSelectOptionId: "61e4505c" }
  }) { projectV2Item { id } }
}'
```

### 4. Atualize o task.md com o número da issue

Substitua `#<número>` pelo número real retornado pelo `gh issue create`.

### 5. Confirme para o usuário

Informe:
- Link da issue criada
- Que o card está em "Ready" no projeto
- Qual o próximo passo: `/implement`

## Regras

- Não comece a implementar — apenas registre e abra a issue
- Se a tarefa for um bug, use `--label "bug"` na issue
- Critérios de aceite na issue devem ser checkboxes markdown (`- [ ]`)
