use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::types::{Manifest, Symbol, SymbolKind, Usage, Visibility};

pub fn analyse_file(path: &Path) -> Result<Manifest, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;

    // MD5 sobre os bytes originais (win1252/CP850 preservados)
    let digest = format!("{:x}", md5::compute(&bytes));

    // Identifiers/keywords são ASCII puro; bytes >0x7F em strings/comentários
    // ficam como U+FFFD sem afetar o parsing
    let source = String::from_utf8_lossy(&bytes).into_owned();

    let mut parser = Parser::new(&source);
    parser.run();

    Ok(Manifest {
        source_path: path.to_string_lossy().into_owned(),
        md5: digest,
        symbols: parser.symbols,
        usages: parser.usages,
    })
}

// ─── parser ───────────────────────────────────────────────────────────────────

struct Parser {
    lines: Vec<String>,
    pub symbols: Vec<Symbol>,
    pub usages: Vec<Usage>,
    cond_stack: Vec<CondState>,
    current_class: Option<String>,
    current_scope: String,
    defined: HashSet<String>,
}

#[derive(Clone)]
struct CondState {
    active: bool,
    had_active: bool,
}

impl Parser {
    fn new(source: &str) -> Self {
        Parser {
            lines: source.lines().map(|s| s.to_string()).collect(),
            symbols: Vec::new(),
            usages: Vec::new(),
            cond_stack: Vec::new(),
            current_class: None,
            current_scope: String::from("GLOBAL"),
            defined: HashSet::new(),
        }
    }

    fn is_in_cond(&self) -> bool {
        !self.cond_stack.is_empty()
    }

    fn run(&mut self) {
        self.pass_definitions();
        self.pass_usages();
    }

    fn pass_definitions(&mut self) {
        self.cond_stack.clear();
        self.current_class = None;
        self.current_scope = String::from("GLOBAL");

        let n = self.lines.len();
        for line_idx in 0..n {
            let lineno = line_idx + 1;

            // Copy data we need before any mutable borrow of self
            let raw: String = self.lines[line_idx].clone();
            let trimmed = strip_comment(raw.trim()).to_string();
            let upper = trimmed.to_ascii_uppercase();
            let upper = upper.trim();

            // ── preprocessor ───────────────────────────────────────────────
            if upper.starts_with("#IFDEF") || upper.starts_with("#IFNDEF") {
                self.cond_stack.push(CondState { active: true, had_active: true });
                continue;
            }
            if upper.starts_with("#ELSE") {
                if let Some(top) = self.cond_stack.last_mut() {
                    top.active = !top.had_active;
                }
                continue;
            }
            if upper.starts_with("#ENDIF") {
                self.cond_stack.pop();
                continue;
            }

            let in_cond = self.is_in_cond();

            // ── CLASS ───────────────────────────────────────────────────────
            if let Some(name) = parse_keyword(upper, "CLASS") {
                let scope = String::from("GLOBAL");
                let sym = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Class,
                    scope,
                    line: lineno,
                    attributes: vec![],
                    conditional: in_cond,
                };
                self.current_class = Some(name);
                self.current_scope = sym.name.clone();
                self.push_symbol(sym);
                continue;
            }

            // ── ENDCLASS ────────────────────────────────────────────────────
            if upper.starts_with("ENDCLASS") || upper == "END CLASS" {
                self.current_class = None;
                self.current_scope = String::from("GLOBAL");
                continue;
            }

            // ── FUNCTION ────────────────────────────────────────────────────
            if let Some(name) = parse_keyword(upper, "FUNCTION") {
                let scope = self.current_class.clone().unwrap_or_else(|| String::from("GLOBAL"));
                let sym = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Function,
                    scope,
                    line: lineno,
                    attributes: vec![],
                    conditional: in_cond,
                };
                self.current_scope = name;
                self.push_symbol(sym);
                continue;
            }

            // ── PROCEDURE ───────────────────────────────────────────────────
            if let Some(name) = parse_keyword(upper, "PROCEDURE") {
                let scope = self.current_class.clone().unwrap_or_else(|| String::from("GLOBAL"));
                let sym = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Procedure,
                    scope,
                    line: lineno,
                    attributes: vec![],
                    conditional: in_cond,
                };
                self.current_scope = name;
                self.push_symbol(sym);
                continue;
            }

            // ── METHOD ──────────────────────────────────────────────────────
            if let Some(raw_name) = parse_method(upper) {
                // raw_name may be "CLASSNAME:METHODNAME" (impl) or just "METHODNAME" (decl)
                let (scope, name) = if let Some(colon) = raw_name.find(':') {
                    (raw_name[..colon].to_string(), raw_name[colon + 1..].to_string())
                } else {
                    (
                        self.current_class.clone().unwrap_or_else(|| self.current_scope.clone()),
                        raw_name,
                    )
                };
                let sym = Symbol {
                    name,
                    kind: SymbolKind::Method,
                    scope,
                    line: lineno,
                    attributes: vec![],
                    conditional: in_cond,
                };
                self.push_symbol(sym);
                continue;
            }

            // ── PUBLIC ──────────────────────────────────────────────────────
            if upper.starts_with("PUBLIC ") || upper == "PUBLIC" {
                let scope = self.current_scope.clone();
                for vname in parse_varlist(&trimmed) {
                    self.push_symbol(Symbol {
                        name: vname,
                        kind: SymbolKind::Public,
                        scope: scope.clone(),
                        line: lineno,
                        attributes: vec![],
                        conditional: in_cond,
                    });
                }
                continue;
            }

            // ── MEMVAR ──────────────────────────────────────────────────────
            if upper.starts_with("MEMVAR ") || upper == "MEMVAR" {
                let scope = self.current_scope.clone();
                for vname in parse_varlist(&trimmed) {
                    self.push_symbol(Symbol {
                        name: vname,
                        kind: SymbolKind::Memvar,
                        scope: scope.clone(),
                        line: lineno,
                        attributes: vec![],
                        conditional: in_cond,
                    });
                }
                continue;
            }

            // ── STATIC ──────────────────────────────────────────────────────
            if upper.starts_with("STATIC ") || upper == "STATIC" {
                let scope = self.current_scope.clone();
                for vname in parse_varlist(&trimmed) {
                    self.push_symbol(Symbol {
                        name: vname,
                        kind: SymbolKind::Static,
                        scope: scope.clone(),
                        line: lineno,
                        attributes: vec![],
                        conditional: in_cond,
                    });
                }
                continue;
            }

            // ── Class members: VAR / ACCESS / ASSIGN ────────────────────────
            let in_class = self.current_class.is_some();
            if in_class {
                if let Some((vname, vis)) = parse_class_var(upper) {
                    let scope = self.current_class.clone().unwrap();
                    self.push_symbol(Symbol {
                        name: vname,
                        kind: SymbolKind::ClassVar { visibility: vis },
                        scope,
                        line: lineno,
                        attributes: vec![],
                        conditional: in_cond,
                    });
                    continue;
                }
                if let Some(name) = parse_keyword(upper, "ACCESS") {
                    let scope = self.current_class.clone().unwrap();
                    self.push_symbol(Symbol {
                        name,
                        kind: SymbolKind::Access,
                        scope,
                        line: lineno,
                        attributes: vec![],
                        conditional: in_cond,
                    });
                    continue;
                }
                if let Some(name) = parse_keyword(upper, "ASSIGN") {
                    let scope = self.current_class.clone().unwrap();
                    self.push_symbol(Symbol {
                        name,
                        kind: SymbolKind::Assign,
                        scope,
                        line: lineno,
                        attributes: vec![],
                        conditional: in_cond,
                    });
                    continue;
                }
            }
        }
    }

    fn push_symbol(&mut self, sym: Symbol) {
        self.defined.insert(sym.name.to_ascii_uppercase());
        self.symbols.push(sym);
    }

    fn pass_usages(&mut self) {
        let n = self.lines.len();
        for line_idx in 0..n {
            let lineno = line_idx + 1;
            let raw = self.lines[line_idx].clone();
            let trimmed = strip_comment(raw.trim()).to_string();
            let calls = collect_calls(&trimmed, lineno);
            for u in calls {
                let upper = u.name.to_ascii_uppercase();
                if !self.defined.contains(&upper) && !is_keyword(&upper) {
                    self.usages.push(Usage {
                        name: upper,
                        line: u.line,
                        col: u.col,
                    });
                }
            }
        }

        self.usages.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));
        self.usages.dedup_by(|a, b| a.name == b.name && a.line == b.line && a.col == b.col);
    }
}

// ─── call collector (pure function, no self borrow) ──────────────────────────

fn collect_calls(line: &str, lineno: usize) -> Vec<Usage> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // skip string literals
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            while i < len && bytes[i] != q {
                i += 1;
            }
            i += 1;
            continue;
        }

        if is_ident_start(bytes[i]) {
            let start = i;
            while i < len && is_ident_cont(bytes[i]) {
                i += 1;
            }
            let ident = &line[start..i];
            let rest = line[i..].trim_start();
            if rest.starts_with('(') {
                out.push(Usage {
                    name: ident.to_ascii_uppercase(),
                    line: lineno,
                    col: start + 1,
                });
            }
            continue;
        }
        i += 1;
    }
    out
}

// ─── helpers ──────────────────────────────────────────────────────────────────

fn strip_comment(s: &str) -> &str {
    if let Some(idx) = s.find("//") {
        return &s[..idx];
    }
    if let Some(idx) = s.find("/*") {
        return &s[..idx];
    }
    s
}

/// Parse "METHOD [ClassName:]MethodName[(...)]" → Some("CLASSNAME:METHODNAME" or "METHODNAME")
fn parse_method(upper: &str) -> Option<String> {
    if !upper.starts_with("METHOD ") {
        return None;
    }
    let rest = upper["METHOD ".len()..].trim();
    // capture up to '(' stripping parameter list
    let raw: String = rest
        .chars()
        .take_while(|&c| c != '(' && c != ' ' && c != '\t')
        .collect();
    if raw.is_empty() { None } else { Some(raw) }
}

/// Parse "KEYWORD name" → Some("NAME")
fn parse_keyword(upper: &str, kw: &str) -> Option<String> {
    let prefix = format!("{} ", kw);
    if upper.starts_with(&prefix) {
        let rest = upper[prefix.len()..].trim();
        let name: String = rest
            .chars()
            .take_while(|&c| c != '(' && c != ' ' && c != '\t' && c != ':')
            .collect();
        if name.is_empty() { None } else { Some(name) }
    } else {
        None
    }
}

/// Parse comma-separated var list after a keyword ("PUBLIC x, y, z")
fn parse_varlist(line: &str) -> Vec<String> {
    let rest = line.splitn(2, ' ').nth(1).unwrap_or("").trim();
    rest.split(',')
        .map(|s| {
            let base = s.split(":=").next().unwrap_or(s).trim();
            let base = base.split('[').next().unwrap_or(base).trim();
            base.to_ascii_uppercase()
        })
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .collect()
}

/// Parse "VAR name [EXPORTED|HIDDEN|PROTECTED]" inside a class body
fn parse_class_var(upper: &str) -> Option<(String, Visibility)> {
    if !upper.starts_with("VAR ") {
        return None;
    }
    let rest = upper["VAR ".len()..].trim();
    let name: String = rest
        .chars()
        .take_while(|&c| c != ' ' && c != '\t')
        .collect();
    if name.is_empty() {
        return None;
    }
    let vis = if rest.contains("HIDDEN") {
        Visibility::Hidden
    } else if rest.contains("PROTECTED") {
        Visibility::Protected
    } else {
        Visibility::Exported
    };
    Some((name, vis))
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "IF" | "ELSE" | "ELSEIF" | "ENDIF" | "FOR" | "NEXT" | "WHILE"
            | "ENDWHILE" | "DO" | "RETURN" | "LOCAL" | "STATIC" | "PUBLIC"
            | "PRIVATE" | "MEMVAR" | "FIELD" | "FUNCTION" | "PROCEDURE"
            | "CLASS" | "METHOD" | "DATA" | "VAR" | "ACCESS" | "ASSIGN"
            | "ENDCLASS" | "BEGIN" | "END" | "SWITCH" | "CASE" | "OTHERWISE"
            | "ENDSWITCH" | "EXIT" | "LOOP" | "BREAK" | "TRY" | "CATCH"
            | "FINALLY" | "ENDTRY" | "WITH" | "OBJECT" | "ENDWITH"
            | "NIL" | "SELF" | "SUPER" | "TRUE" | "FALSE" | "AND" | "OR"
            | "NOT" | "IN" | "REPLACE" | "APPEND" | "BLANK" | "USE"
            | "SELECT" | "DBEVAL" | "SEEK" | "LOCATE" | "SKIP" | "GO"
            | "GOTO" | "STORE" | "COPY" | "CLOSE" | "COMMIT" | "ROLLBACK"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyword_function() {
        let input = "FUNCTION HelloWorld";
        let result = parse_keyword(input, "FUNCTION");
        assert_eq!(result, Some("HelloWorld".to_string()));
    }

    #[test]
    fn test_parse_keyword_function_with_parens() {
        let input = "FUNCTION HelloWorld()";
        let result = parse_keyword(input, "FUNCTION");
        assert_eq!(result, Some("HelloWorld".to_string()));
    }

    #[test]
    fn test_parse_keyword_not_found() {
        let input = "PROCEDURE HelloWorld";
        let result = parse_keyword(input, "FUNCTION");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_method_simple() {
        let input = "METHOD MyMethod";
        let result = parse_method(input);
        assert_eq!(result, Some("MyMethod".to_string()));
    }

    #[test]
    fn test_parse_method_with_class() {
        let input = "METHOD MyClass:MyMethod";
        let result = parse_method(input);
        assert_eq!(result, Some("MyClass:MyMethod".to_string()));
    }

    #[test]
    fn test_parse_method_with_parens() {
        let input = "METHOD MyClass:MyMethod()";
        let result = parse_method(input);
        assert_eq!(result, Some("MyClass:MyMethod".to_string()));
    }

    #[test]
    fn test_parse_varlist_simple() {
        let input = "PUBLIC x, y, z";
        let result = parse_varlist(input);
        assert_eq!(result, vec!["X", "Y", "Z"]);
    }

    #[test]
    fn test_parse_varlist_with_init() {
        let input = "PUBLIC nCounter := 0, cName := ''";
        let result = parse_varlist(input);
        assert_eq!(result, vec!["NCOUNTER", "CNAME"]);
    }

    #[test]
    fn test_parse_varlist_with_array() {
        let input = "PUBLIC aItems[10], aNames";
        let result = parse_varlist(input);
        assert_eq!(result, vec!["AITEMS", "ANAMES"]);
    }

    #[test]
    fn test_parse_varlist_single() {
        let input = "PUBLIC nCounter";
        let result = parse_varlist(input);
        assert_eq!(result, vec!["NCOUNTER"]);
    }

    #[test]
    fn test_parse_class_var_exported() {
        let input = "VAR myVar EXPORTED";
        let result = parse_class_var(input);
        assert_eq!(result, Some(("myVar".to_string(), Visibility::Exported)));
    }

    #[test]
    fn test_parse_class_var_hidden() {
        let input = "VAR myVar HIDDEN";
        let result = parse_class_var(input);
        assert_eq!(result, Some(("myVar".to_string(), Visibility::Hidden)));
    }

    #[test]
    fn test_parse_class_var_protected() {
        let input = "VAR myVar PROTECTED";
        let result = parse_class_var(input);
        assert_eq!(result, Some(("myVar".to_string(), Visibility::Protected)));
    }

    #[test]
    fn test_parse_class_var_default() {
        let input = "VAR myVar";
        let result = parse_class_var(input);
        assert_eq!(result, Some(("myVar".to_string(), Visibility::Exported)));
    }

    #[test]
    fn test_strip_comment_cpp_style() {
        let input = "some code // this is comment";
        let result = strip_comment(input);
        assert_eq!(result, "some code ");
    }

    #[test]
    fn test_strip_comment_c_style() {
        let input = "some code /* comment */ more";
        let result = strip_comment(input);
        assert_eq!(result, "some code ");
    }

    #[test]
    fn test_strip_comment_no_comment() {
        let input = "some code without comment";
        let result = strip_comment(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_is_keyword_function() {
        assert!(is_keyword("FUNCTION"));
        assert!(is_keyword("PROCEDURE"));
        assert!(is_keyword("CLASS"));
        assert!(is_keyword("METHOD"));
        assert!(is_keyword("PUBLIC"));
    }

    #[test]
    fn test_is_keyword_negative() {
        assert!(!is_keyword("MYFUNCTION"));
        assert!(!is_keyword("UNKNOWN"));
        assert!(!is_keyword(""));
    }

    #[test]
    fn test_is_ident_start() {
        assert!(is_ident_start(b'a'));
        assert!(is_ident_start(b'Z'));
        assert!(is_ident_start(b'_'));
        assert!(!is_ident_start(b'0'));
        assert!(!is_ident_start(b'('));
    }

    #[test]
    fn test_is_ident_cont() {
        assert!(is_ident_cont(b'a'));
        assert!(is_ident_cont(b'Z'));
        assert!(is_ident_cont(b'0'));
        assert!(is_ident_cont(b'_'));
        assert!(!is_ident_cont(b'('));
        assert!(!is_ident_cont(b'.'));
    }

    #[test]
    fn test_collect_calls_simple() {
        let line = "FetchUser()";
        let calls = collect_calls(line, 1);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "FETCHUSER");
        assert_eq!(calls[0].line, 1);
    }

    #[test]
    fn test_collect_calls_multiple() {
        let line = "x := DoThis() + DoThat()";
        let calls = collect_calls(line, 5);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "DOTHIS");
        assert_eq!(calls[1].name, "DOTHAT");
    }

    #[test]
    fn test_collect_calls_with_args() {
        let line = "Result := Process( x, y, z )";
        let calls = collect_calls(line, 10);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "PROCESS");
    }

    #[test]
    fn test_collect_calls_in_string() {
        let line = r#"? "FakeCall()""#;
        let calls = collect_calls(line, 1);
        // Should not detect FakeCall inside string
        assert!(calls.iter().all(|c| c.name != "FAKECALL"));
    }

    #[test]
    fn test_collect_calls_no_parens() {
        let line = "x := y + z";
        let calls = collect_calls(line, 1);
        assert_eq!(calls.len(), 0);
    }
}
