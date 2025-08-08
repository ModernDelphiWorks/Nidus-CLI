# Nest4D CLI

🚀 **Nest4D Framework for Delphi** - Uma ferramenta de linha de comando moderna para desenvolvimento rápido de aplicações Delphi inspirada no NestJS.

## 📋 Índice

- [Instalação](#instalação)
- [Início Rápido](#início-rápido)
- [Comandos Disponíveis](#comandos-disponíveis)
- [Estrutura do Projeto](#estrutura-do-projeto)
- [Exemplos](#exemplos)
- [Configuração](#configuração)
- [Contribuindo](#contribuindo)
- [Licença](#licença)

## 🔧 Instalação

### Pré-requisitos

- Delphi 10.3 ou superior
- Git instalado
- Windows (suporte para outras plataformas em desenvolvimento)

### Instalação via Cargo

```bash
cargo install nest4d-cli
```

### Instalação via Binário

Baixe o binário mais recente da [página de releases](https://github.com/your-repo/nest4d-cli/releases) e adicione ao seu PATH.

## 🚀 Início Rápido

### 1. Criar um novo projeto

```bash
nest4d new --project MyApp --path ./
```

### 2. Instalar dependências

```bash
cd MyApp
nest4d install
```

### 3. Gerar um módulo

```bash
nest4d gen module User
```

### 4. Gerar componentes específicos

```bash
nest4d gen controller User
nest4d gen service User
nest4d gen repository User
```

## 📚 Comandos Disponíveis

### `new` - Criar novo projeto

```bash
nest4d new --project <nome> --path <caminho> [--with-tests]
```

**Opções:**
- `--project, -p`: Nome do projeto
- `--path`: Caminho onde criar o projeto (deve começar com `./`)
- `--with-tests`: Incluir estrutura de testes

### `gen` - Gerar componentes

```bash
nest4d gen <tipo> <nome> [opções]
```

**Tipos disponíveis:**
- `module`: Gera um módulo completo
- `controller`: Gera apenas o controller
- `service`: Gera apenas o service
- `repository`: Gera apenas o repository
- `interface`: Gera apenas a interface
- `infra`: Gera apenas a infraestrutura
- `handler`: Gera apenas o handler
- `scaffold`: Gera estrutura completa
- `all`: Gera todos os componentes

**Opções:**
- `--flat`: Não criar subpasta para o módulo
- `--path <caminho>`: Caminho específico para geração
- `--overwrite`: Sobrescrever arquivos existentes

### `install` - Instalar dependências

```bash
nest4d install
```

Instala todas as dependências definidas no `nest4d.json`.

### Outros comandos

```bash
nest4d --version    # Mostra a versão
nest4d --help       # Mostra ajuda
```

## 🏗️ Estrutura do Projeto

Um projeto Nest4D típico tem a seguinte estrutura:

```
MyApp/
├── MyApp.dpr              # Arquivo principal do projeto
├── nest4d.json            # Configuração do projeto
├── src/
│   ├── AppModule.pas       # Módulo principal da aplicação
│   └── modules/
│       └── user/
│           ├── UserModule.pas
│           ├── UserController.pas
│           ├── UserService.pas
│           ├── UserRepository.pas
│           ├── UserInterface.pas
│           └── UserInfra.pas
└── test/                   # Testes (se --with-tests foi usado)
    └── ...
```

## 💡 Exemplos

### Criando um módulo de usuários completo

```bash
# Gera módulo, controller, service, repository, interface e infra
nest4d gen module User
```

### Gerando componentes individuais

```bash
# Apenas o controller
nest4d gen controller Product

# Service em um caminho específico
nest4d gen service Order --path ./src/modules/orders

# Sobrescrever arquivo existente
nest4d gen repository Customer --overwrite
```

### Estrutura flat (sem subpastas)

```bash
# Gera arquivos diretamente na pasta atual
nest4d gen module Auth --flat
```

## ⚙️ Configuração

O arquivo `nest4d.json` contém as configurações do projeto:

```json
{
  "name": "MyApp",
  "description": "Minha aplicação Nest4D",
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

### Adicionando dependências

Edite o array `dependencies` no `nest4d.json` e execute:

```bash
nest4d install
```

## 🧪 Executando Testes

```bash
# Executar todos os testes
cargo test

# Executar testes específicos
cargo test validation
cargo test integration

# Executar com output detalhado
cargo test -- --nocapture
```

## 🤝 Contribuindo

1. Faça um fork do projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

### Desenvolvimento Local

```bash
# Clone o repositório
git clone https://github.com/your-repo/nest4d-cli.git
cd nest4d-cli

# Instale dependências
cargo build

# Execute testes
cargo test

# Execute o CLI localmente
cargo run -- --help
```

## 📝 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## 🆘 Suporte

- 📖 [Documentação](https://nest4d.dev)
- 🐛 [Reportar Bug](https://github.com/your-repo/nest4d-cli/issues)
- 💡 [Solicitar Feature](https://github.com/your-repo/nest4d-cli/issues)
- 💬 [Discussões](https://github.com/your-repo/nest4d-cli/discussions)

## 🎯 Roadmap

- [ ] Suporte para Linux e macOS
- [ ] Templates customizáveis
- [ ] Integração com IDEs
- [ ] Geração de documentação automática
- [ ] Suporte para Docker
- [ ] Plugin system

---

**Feito com ❤️ para a comunidade Delphi**
