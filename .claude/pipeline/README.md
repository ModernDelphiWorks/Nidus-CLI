# Pipeline — Handoff Documents

Esta pasta contém os documentos de handoff gerados por cada agente durante a execução da pipeline.

## Fluxo

```
Você (/task)
    └──► implement-report.md  (/implement)
              └──► review-report.md  (/review)
                        └──► test-report.md  (/test)
                                  └──► release-report.md  (/release)
```

## Arquivos

| Arquivo | Gerado por | Lido por |
|---------|-----------|----------|
| `task.md` | Você (`/task`) | implement, review, test, release |
| `implement-report.md` | `/implement` | review, test |
| `review-report.md` | `/review` | test, release |
| `test-report.md` | `/test` | release |
| `release-report.md` | `/release` | você (resultado final) |

## Como usar

1. Descreva o que quer para o agente: `/task adicionar comando X que faz Y`
2. Execute em sequência: `/implement` → `/review` → `/test` → `/release`
3. Se um agente retornar REPROVADO, volte para `/implement` com o feedback
4. Os relatórios ficam aqui como histórico da entrega

## Observação

Os arquivos `*-report.md` são sobrescritos a cada nova pipeline.
Para manter histórico, faça commit da pasta após cada ciclo completo.
