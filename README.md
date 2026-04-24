# MakerHeaderGenerate

Ferramenta em Rust para gerar manifestos de símbolos de arquivos fonte Harbour (.prg), criando arquivos de manifesto (.mkh) com análise estática de definições e usos.

## Propósito

O **MakerHeaderGenerate** analisa arquivos Harbour (.prg) e extrai:
- **Definições de símbolos**: funções, procedimentos, métodos, classes, variáveis públicas/memvar, etc.
- **Usos de símbolos**: chamadas de funções e referências de símbolos externos

Os resultados são gravados em um arquivo `.mkh` (MakerHeaderGenerate manifest) que pode ser utilizado por ferramentas de build, análise de dependências ou IDE.

## Instalação

### Pré-requisitos
- Rust 1.70+ (edition 2021)

### Build
```bash
cargo build --release
```

O executável será gerado em `target/release/maker_header_gen`.

## Uso

### Análise de um arquivo único
```bash
maker_header_gen arquivo.prg
```

### Análise recursiva de diretório
```bash
maker_header_gen /caminho/para/fontes/
```

Todos os arquivos `.prg` no diretório e subdiretórios serão processados.

### Modo verbose
```bash
maker_header_gen arquivo.prg --verbose
```

Exibe símbolos e usos no stdout além de gerar o arquivo `.mkh`.

## Formato de Entrada (.PRG)

O analisador suporta código-fonte Harbour com:

### Declarações reconhecidas
- `FUNCTION nomeFuncao` - Define função
- `PROCEDURE nomeProcedure` - Define procedimento
- `METHOD nomeMetodo` - Define método de classe
- `CLASS NomeClasse` - Define classe
- `PUBLIC variavel1, variavel2` - Declara variáveis públicas
- `MEMVAR variavel1, variavel2` - Declara variáveis de memória
- `STATIC variavel` - Declara variável estática
- `VAR nome [EXPORTED|HIDDEN|PROTECTED]` - Propriedade de classe
- `ACCESS nome` - Accessor de classe
- `ASSIGN nome` - Atribuidor de classe

### Suporte a pré-processador
- `#IFDEF identificador` / `#ENDIF` - Blocos condicionais
- `#ELSE` - Alternativa condicional
- Símbolos em blocos condicionais são marcados com atributo `CONDITIONAL`

### Exemplos de código reconhecido

**Declaração de variáveis memvar:**
```harbour
MEMVAR u_cor01, u_cor02, u_cor03, usuario, nSerial
```

**Declaração de variáveis públicas:**
```harbour
PUBLIC cNomeMenu := '', cModulo := '', lSql := .f.
PUBLIC nFilBase, nTimeExec
```

**Procedimento com variáveis locais:**
```harbour
PROCEDURE DeclaraVariaveisPublicas()
   LOCAL oParams, aProjects
   // ...
RETURN
```

**Detecção de uso externo:**
Chamadas de função são detectadas pela presença de `(` após identificador:
```harbour
aadd( aProjects, oProjectInfo )  // Detecta uso de AADD
BuildProject()                   // Detecta uso de BUILDPROJECT
```

## Formato de Saída (.MKH)

O arquivo manifesto é gravado em `cache_maker/` ao lado de cada arquivo `.prg`.

### Estrutura

```
; ============================================================
; MakerHeaderGenerate — símbolo manifesto (.mkh)
; ============================================================
; SOURCE  : caminho/do/arquivo.prg
; MD5     : hash-md5-do-arquivo
; SYMBOLS : quantidade de símbolos
; USAGES  : quantidade total de usos (distintos)
; ...

[DEFINITIONS]
[SYMBOL] -> [TIPO] -> NomeSimbolo | Escopo | Linha | Atributos
...

[USAGES]
[+] NomeFuncao | { [Linha:X, Coluna:Y], [Linha:Z, Coluna:W] }
...
```

### Exemplo de manifesto gerado

Para um arquivo com declarações memvar e procedure:

```
[DEFINITIONS]
[SYMBOL] -> [MEMVAR] -> USUARIO | GLOBAL | 19 | -
[SYMBOL] -> [MEMVAR] -> NSERIAL | GLOBAL | 24 | -
[SYMBOL] -> [PROCEDURE] -> DECLARACOESPUBLICAS | GLOBAL | 29 | -
[SYMBOL] -> [PUBLIC] -> USUARIO | DECLARACOESPUBLICAS | 35 | -

[USAGES]
[+] CTOD | { [Linha:36, Coluna:83] }
[+] AADD | { [Linha:73, Coluna:1] }
[+] BUILDPROJECT | { [Linha:78, Coluna:12] }
```

### Tipos de símbolo

| Tipo | Significado | Escopo |
|------|-------------|--------|
| FUNCTION | Função | Global ou dentro de classe |
| PROCEDURE | Procedimento | Global ou dentro de classe |
| METHOD | Método de classe | Classe pai |
| CLASS | Definição de classe | Global |
| PUBLIC | Variável pública declarada | Scope onde foi declarada |
| MEMVAR | Variável de memória | Global |
| STATIC | Variável estática | Scope onde foi declarada |
| VAR | Propriedade de classe | Classe |
| ACCESS | Accessor de classe | Classe |
| ASSIGN | Atribuidor de classe | Classe |

### Atributos de símbolo

- `CONDITIONAL` - Símbolo definido dentro de bloco `#IFDEF`/`#IFNDEF`
- `EXPORTED` - Membro de classe com visibilidade exportada (padrão)
- `HIDDEN` - Membro de classe com visibilidade privada
- `PROTECTED` - Membro de classe com visibilidade protegida

## Checksum

Cada manifesto inclui o hash MD5 do arquivo source original (bytes brutos preservados em encoding CP850/Win1252). Isso permite validar se o `.mkh` está sincronizado com o `.prg`.

## Limitações conhecidas

- **UTF-8 fallback**: Caracteres não-UTF8 são convertidos para U+FFFD durante parsing
- **Parsing de chamadas**: Detecta chamadas baseado em padrão `identificador(` - casos complexos podem não ser reconhecidos
- **Macros**: Não expande `#define` com parâmetros
- **Closures/Blocks**: Sintaxe `{ |x| ... }` não é analisada em detalhes
- **Modificadores de método**: `OVERRIDE`, `VIRTUAL`, etc. não são extraídos como atributos

## Estrutura de diretórios

```
MakerHeaderGenerate/
├── Cargo.toml          # Configuração do projeto
├── src/
│   ├── main.rs        # Ponto de entrada
│   ├── types.rs       # Definições de tipos
│   ├── analyser.rs    # Parser Harbour
│   └── emitter.rs     # Gerador de .mkh
└── target/
    └── release/
        └── maker_header_gen  # Executável compilado
```

## Exemplos de uso prático

### Gerar manifesto para todos os fontes de um projeto
```bash
maker_header_gen ./src/
```

Todos os arquivos `.prg` em `./src/` e subdiretórios terão `.mkh` gerado em `./src/**/cache_maker/`.

### Gerar e exibir resultado no console
```bash
maker_header_gen main.prg --verbose
```

Output:
```
=== ./main.prg (md5: 233c9ef6666c12a74206a46df3ebb2d2)
  Symbols  : 3
  Usages   : 32
  [SYMBOL] -> [STATIC] -> S_CONFIG | GLOBAL | 27 | -
  [SYMBOL] -> [PROCEDURE] -> MAIN | GLOBAL | 29 | -
  ...
```

## Troubleshooting

### Arquivo `.mkh` não é gerado
- Verifique se o arquivo `.prg` existe e é legível
- Verifique permissões de escrita no diretório
- Execute com `--verbose` para ver mensagens de erro

### Símbolos não são detectados
- Confirme que seguem a sintaxe exata: `FUNCTION nomeFuncao(`, `PUBLIC var1, var2`
- Verifique se não estão dentro de strings entre `"` ou `'`
- Verifique se não estão comentados (`//`, `/* */`)

### Coluna de uso incorreta
- Colunas são calculadas em caracteres ASCII - caracteres multi-byte podem deslocar o resultado

## Licença

Este projeto é parte do sistema interno de build. Propriedade da empresa.

## Próximas melhorias planejadas

- [ ] Suporte a múltiplos encodings (CP850, Win1252, UTF-8)
- [ ] Parsing de macros com parâmetros
- [ ] Detecção de closures/blocks
- [ ] CI/CD automatizado
- [ ] Suite de testes completa
- [ ] Exportação em múltiplos formatos (JSON, XML)
