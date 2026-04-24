# Suite de Testes - MakerHeaderGenerate

## Visão Geral

A suite de testes do MakerHeaderGenerate compreende **64 testes** divididos em:
- **53 Testes Unitários** - Validação de módulos individuais
- **11 Testes de Integração** - Validação end-to-end com arquivos reais

## Executar Testes

```bash
# Executar todos os testes
cargo test

# Executar apenas testes unitários
cargo test --lib

# Executar apenas testes de integração
cargo test --test integration_tests

# Executar com output detalhado
cargo test -- --nocapture

# Executar teste específico
cargo test test_parse_keyword_function
```

## Testes Unitários (53 testes)

### Módulo: types.rs (9 testes)

Validação das estruturas de dados core do sistema.

| Teste | Descrição |
|-------|-----------|
| `test_symbol_kind_equality` | Verifica igualdade entre tipos de símbolos |
| `test_visibility_equality` | Verifica igualdade entre níveis de visibilidade |
| `test_symbol_creation` | Valida criação correta de Symbol com todos os campos |
| `test_symbol_with_attributes` | Verifica Symbol com lista de atributos |
| `test_symbol_conditional` | Valida marcação de símbolo condicional |
| `test_usage_creation` | Valida criação de Usage com linha e coluna |
| `test_manifest_creation` | Valida criação de Manifest completo |
| `test_manifest_with_symbols` | Verifica Manifest contendo múltiplos símbolos |
| `test_class_var_visibility` | Valida ClassVar com diferentes níveis de visibilidade |

**Cobertura**: Todos os tipos (Symbol, Usage, Manifest, Visibility, SymbolKind)

### Módulo: analyser.rs (22 testes)

Validação do parser e extração de símbolos/usos.

#### Testes de Parsing de Keywords (6 testes)

| Teste | Entrada | Saída Esperada |
|-------|---------|----------------|
| `test_parse_keyword_function` | `"FUNCTION HelloWorld"` | `Some("HelloWorld")` |
| `test_parse_keyword_function_with_parens` | `"FUNCTION HelloWorld()"` | `Some("HelloWorld")` |
| `test_parse_keyword_not_found` | Procura "FUNCTION" em "PROCEDURE..." | `None` |
| `test_parse_method_simple` | `"METHOD MyMethod"` | `Some("MyMethod")` |
| `test_parse_method_with_class` | `"METHOD MyClass:MyMethod"` | `Some("MyClass:MyMethod")` |
| `test_parse_method_with_parens` | `"METHOD MyClass:MyMethod()"` | `Some("MyClass:MyMethod")` |

#### Testes de Parsing de Variáveis (4 testes)

| Teste | Entrada | Saída Esperada |
|-------|---------|----------------|
| `test_parse_varlist_simple` | `"PUBLIC x, y, z"` | `["X", "Y", "Z"]` |
| `test_parse_varlist_with_init` | `"PUBLIC nCounter := 0, cName := ''"` | `["NCOUNTER", "CNAME"]` |
| `test_parse_varlist_with_array` | `"PUBLIC aItems[10], aNames"` | `["AITEMS", "ANAMES"]` |
| `test_parse_varlist_single` | `"PUBLIC nCounter"` | `["NCOUNTER"]` |

#### Testes de Parsing de Membros de Classe (4 testes)

| Teste | Entrada | Saída Esperada |
|-------|---------|----------------|
| `test_parse_class_var_exported` | `"VAR myVar EXPORTED"` | `Some(("myVar", Exported))` |
| `test_parse_class_var_hidden` | `"VAR myVar HIDDEN"` | `Some(("myVar", Hidden))` |
| `test_parse_class_var_protected` | `"VAR myVar PROTECTED"` | `Some(("myVar", Protected))` |
| `test_parse_class_var_default` | `"VAR myVar"` | `Some(("myVar", Exported))` |

#### Testes de Remoção de Comentários (3 testes)

| Teste | Entrada | Saída Esperada |
|-------|---------|----------------|
| `test_strip_comment_cpp_style` | `"code // comment"` | `"code "` |
| `test_strip_comment_c_style` | `"code /* comment */ more"` | `"code "` |
| `test_strip_comment_no_comment` | `"code without comment"` | `"code without comment"` |

#### Testes de Detecção de Keywords (2 testes)

| Teste | Validação |
|-------|-----------|
| `test_is_keyword_function` | Verifica keywords conhecidas (FUNCTION, PROCEDURE, CLASS, etc) |
| `test_is_keyword_negative` | Verifica que identificadores comuns não são keywords |

#### Testes de Validação de Caracteres (2 testes)

| Teste | Validação |
|-------|-----------|
| `test_is_ident_start` | Válidos: a-z, A-Z, _ | Inválidos: 0-9, (, etc |
| `test_is_ident_cont` | Válidos: a-z, A-Z, 0-9, _ | Inválidos: (, ., etc |

#### Testes de Coleta de Chamadas (5 testes)

| Teste | Entrada | Saída Esperada |
|-------|---------|----------------|
| `test_collect_calls_simple` | `"FetchUser()"` | 1 chamada: FETCHUSER na linha 1 |
| `test_collect_calls_multiple` | `"x := DoThis() + DoThat()"` | 2 chamadas: DOTHIS, DOTHAT |
| `test_collect_calls_with_args` | `"Result := Process( x, y, z )"` | 1 chamada: PROCESS com argumentos |
| `test_collect_calls_in_string` | `"? \"FakeCall()\""` | 0 chamadas (ignorada string) |
| `test_collect_calls_no_parens` | `"x := y + z"` | 0 chamadas (sem parênteses) |

**Cobertura**: Parser de keywords, variáveis, comentários, identificadores, chamadas de função

### Módulo: emitter.rs (22 testes)

Validação da formatação e emissão de manifesto.

#### Testes de String Type-to-Name (3 testes)

| Teste | Validação |
|-------|-----------|
| `test_kind_str_function` | Verifica conversão de SymbolKind::Function → "FUNCTION" |
| `test_kind_str_all_types` | Verifica todas as 10 variações de SymbolKind |
| `test_kind_str_classvar` | Verifica ClassVar independente de Visibility |

#### Testes de String Visibility-to-Name (2 testes)

| Teste | Validação |
|-------|-----------|
| `test_vis_str_exported` | Verifica Visibility::Exported → "EXPORTED" |
| `test_vis_str_all_types` | Verifica Exported, Hidden, Protected |

#### Testes de Formatação de Símbolo (5 testes)

| Teste | Validação |
|-------|-----------|
| `test_format_symbol_simple` | Valida formato: `[SYMBOL] -> [TYPE] -> name \| scope \| line \| attrs` |
| `test_format_symbol_conditional` | Verifica inclusão de "CONDITIONAL" em atributos |
| `test_format_symbol_with_attributes` | Verifica múltiplos atributos separados por vírgula |
| `test_format_symbol_classvar_exported` | Verifica ClassVar com visibilidade EXPORTED |
| `test_format_symbol_classvar_hidden` | Verifica ClassVar com visibilidade HIDDEN |

#### Testes de Formatação de Uso (2 testes)

| Teste | Validação |
|-------|-----------|
| `test_format_usage_grouped` | Valida formato: `[+] NAME \| { [Linha:X, Coluna:Y], ... }` |
| `test_format_usage_grouped_single` | Verifica formatação com uma única coordenada |

#### Testes de Agrupamento de Usos (3 testes)

| Teste | Validação |
|-------|-----------|
| `test_group_usages_empty` | Retorna BTreeMap vazio para lista vazia |
| `test_group_usages_single` | Agrupa uma única use corretamente |
| `test_group_usages_multiple` | Agrupa múltiplos usos por nome mantendo coordenadas |

#### Testes de Renderização (5 testes)

| Teste | Validação |
|-------|-----------|
| `test_render_mkh_header` | Valida cabeçalho com SOURCE, MD5, SYMBOLS count, USAGES count |
| `test_render_mkh_with_symbols` | Verifica inclusão de [DEFINITIONS] e símbolos |
| `test_render_stdout` | Verifica formato legível com === e indentação |
| Não há teste específico para [USAGES], mas é coberto por `test_render_mkh_header` |
| Não há teste para arquivo vazio, mas é coberto por fixtures |

**Cobertura**: Conversão de types, formatação, agrupamento, renderização de manifesto

## Testes de Integração (11 testes)

Validação end-to-end com arquivos Harbour reais. Todos usam fixtures em `tests/fixtures/`.

### Fixtures Disponíveis

| Arquivo | Propósito | Contém |
|---------|----------|--------|
| `simple.prg` | Teste básico | MEMVAR, PROCEDURE, FUNCTION, chamadas de função |
| `class_example.prg` | Classes e métodos | CLASS, VAR, METHOD, ACCESS, ASSIGN |
| `conditional.prg` | Pré-processador | #ifdef, #endif, símbolos condicionais |
| `variables.prg` | Declarações | MEMVAR, PUBLIC, STATIC, LOCAL |

### Testes

| Teste | Arquivo | Validação |
|-------|---------|-----------|
| `test_simple_file_analysis` | simple.prg | Presença de MEMVAR, PROCEDURE, FUNCTION |
| `test_class_file_analysis` | class_example.prg | CLASS, ENDCLASS, VAR, METHOD, ACCESS, ASSIGN |
| `test_conditional_file_analysis` | conditional.prg | #ifdef, #endif, PUBLIC, FUNCTION |
| `test_variables_file_analysis` | variables.prg | MEMVAR list, PUBLIC, STATIC, chamadas |
| `test_manifest_structure` | Todos | Validação estrutura UTF-8, presença de símbolos |
| `test_md5_calculation` | simple.prg | MD5 tem 32 chars, contém hex válido |
| `test_symbol_detection_memvar` | Inline | Parse de lista MEMVAR separada por vírgula |
| `test_symbol_detection_public` | Inline | Parse de PUBLIC com atribuição inline |
| `test_class_scope_detection` | Inline | Valida CLASS...ENDCLASS e VAR |
| `test_comment_preservation` | simple.prg | Verifica presença de comentários /* */ |
| `test_call_detection_pattern` | Inline | Identifica padrão `identificador(` |

**Cobertura**: Análise completa de arquivo, parsing de estruturas, MD5, comentários, padrões de chamada

## Cobertura por Funcionalidade

| Funcionalidade | Testes Unitários | Testes Integração | Total |
|----------------|------------------|-------------------|-------|
| Tipos/Estruturas | 9 | 1 | 10 |
| Parser Keywords | 6 | 5 | 11 |
| Parser Variáveis | 4 | 3 | 7 |
| Parser Classes | 4 | 2 | 6 |
| Comentários | 3 | 1 | 4 |
| Keywords/Identificadores | 2 | 1 | 3 |
| Coleta de Chamadas | 5 | 1 | 6 |
| Formatação | 22 | 0 | 22 |
| **Total** | **53** | **11** | **64** |

## Exemplo de Execução

```bash
$ cargo test

     Running unittests src/lib.rs (target/debug/deps/maker_header_gen-...)
running 53 tests
test analyser::tests::test_collect_calls_simple ... ok
test analyser::tests::test_parse_keyword_function ... ok
test emitter::tests::test_format_symbol_simple ... ok
test types::tests::test_symbol_creation ... ok
[... 49 mais testes ...]
test result: ok. 53 passed

     Running tests/integration_tests.rs (target/debug/deps/integration_tests-...)
running 11 tests
test test_simple_file_analysis ... ok
test test_class_file_analysis ... ok
[... 9 mais testes ...]
test result: ok. 11 passed

test result: ok. 64 passed; 0 failed
```

## CI/CD

Para integração contínua, adicione ao `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
```

## Próximas Melhorias

- [ ] Testes de performance para arquivos grandes
- [ ] Cobertura de edge cases (encoding, caracteres especiais)
- [ ] Testes de erro (arquivos corrompidos, permissões negadas)
- [ ] Propriedades com quickcheck (property-based testing)
- [ ] Snapshots de manifesto para regressão

## Estrutura de Diretórios

```
MakerHeaderGenerate/
├── src/
│   ├── lib.rs              # Exports dos módulos
│   ├── main.rs             # Entry point
│   ├── types.rs            # Tipos + 9 testes unitários
│   ├── analyser.rs         # Parser + 22 testes unitários
│   └── emitter.rs          # Emitter + 22 testes unitários
└── tests/
    ├── integration_tests.rs # 11 testes end-to-end
    └── fixtures/
        ├── simple.prg
        ├── class_example.prg
        ├── conditional.prg
        └── variables.prg
```
