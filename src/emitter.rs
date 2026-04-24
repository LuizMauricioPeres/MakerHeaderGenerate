use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{Manifest, Symbol, SymbolKind, Usage, Visibility};

/// Write the .mkh file next to the .prg inside cache_maker/
pub fn write_mkh(prg_path: &Path, manifest: &Manifest) -> Result<PathBuf, String> {
    let parent = prg_path
        .parent()
        .ok_or_else(|| format!("no parent dir for {}", prg_path.display()))?;

    let cache_dir = parent.join("cache_maker");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("mkdir {}: {}", cache_dir.display(), e))?;

    let stem = prg_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| String::from("invalid filename"))?;

    let out_path = cache_dir.join(format!("{}.mkh", stem));
    let content = render_mkh(manifest);

    fs::write(&out_path, content)
        .map_err(|e| format!("write {}: {}", out_path.display(), e))?;

    Ok(out_path)
}

/// Render the full .mkh content
fn render_mkh(m: &Manifest) -> String {
    let mut buf = String::with_capacity(4096);

    // ── header ────────────────────────────────────────────────────────────────
    buf.push_str("; ============================================================\n");
    buf.push_str("; MakerHeaderGenerate — símbolo manifesto (.mkh)\n");
    buf.push_str("; ============================================================\n");
    buf.push_str(&format!("; SOURCE  : {}\n", m.source_path));
    buf.push_str(&format!("; MD5     : {}\n", m.md5));
    buf.push_str(&format!("; SYMBOLS : {}\n", m.symbols.len()));
    buf.push_str(&format!("; USAGES  : {} ({} distintos)\n", m.usages.len(), group_usages(&m.usages).len()));
    buf.push_str("; ------------------------------------------------------------\n");
    buf.push_str("; FORMAT  : [SYMBOL] -> [TIPO] -> Nome | Escopo | Linha | Atributos\n");
    buf.push_str("; ------------------------------------------------------------\n\n");

    // ── symbol definitions ────────────────────────────────────────────────────
    buf.push_str("[DEFINITIONS]\n");
    for sym in &m.symbols {
        buf.push_str(&format_symbol(sym));
        buf.push('\n');
    }

    // ── external usages (agrupados por símbolo) ──────────────────────────────
    buf.push_str("\n[USAGES]\n");
    for (name, coords) in group_usages(&m.usages) {
        buf.push_str(&format_usage_grouped(&name, &coords));
        buf.push('\n');
    }

    buf
}

/// Agrupa usos pelo nome do símbolo, preservando ordem de primeira ocorrência.
/// Retorna vec ordenado alfabeticamente (BTreeMap garante isso).
fn group_usages(usages: &[Usage]) -> BTreeMap<String, Vec<(usize, usize)>> {
    let mut map: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
    for u in usages {
        map.entry(u.name.clone())
            .or_default()
            .push((u.line, u.col));
    }
    map
}

fn format_symbol(sym: &Symbol) -> String {
    let tipo = kind_str(&sym.kind);
    let mut attrs: Vec<String> = sym.attributes.clone();
    if sym.conditional {
        attrs.push(String::from("CONDITIONAL"));
    }
    if let SymbolKind::ClassVar { visibility } = &sym.kind {
        attrs.push(vis_str(visibility).to_string());
    }
    let attrs_str = if attrs.is_empty() {
        String::from("-")
    } else {
        attrs.join(",")
    };
    format!(
        "[SYMBOL] -> [{}] -> {} | {} | {} | {}",
        tipo, sym.name, sym.scope, sym.line, attrs_str
    )
}

fn format_usage_grouped(name: &str, coords: &[(usize, usize)]) -> String {
    let locs: Vec<String> = coords
        .iter()
        .map(|(l, c)| format!("[Linha:{}, Coluna:{}]", l, c))
        .collect();
    format!("[+] {} | {{ {} }}", name, locs.join(", "))
}

fn kind_str(k: &SymbolKind) -> &'static str {
    match k {
        SymbolKind::Function => "FUNCTION",
        SymbolKind::Procedure => "PROCEDURE",
        SymbolKind::Method => "METHOD",
        SymbolKind::Public => "PUBLIC",
        SymbolKind::Static => "STATIC",
        SymbolKind::Memvar => "MEMVAR",
        SymbolKind::ClassVar { .. } => "VAR",
        SymbolKind::Access => "ACCESS",
        SymbolKind::Assign => "ASSIGN",
        SymbolKind::Class => "CLASS",
    }
}

fn vis_str(v: &Visibility) -> &'static str {
    match v {
        Visibility::Exported => "EXPORTED",
        Visibility::Hidden => "HIDDEN",
        Visibility::Protected => "PROTECTED",
    }
}

/// Human-readable stdout rendering (for --verbose)
pub fn render_stdout(m: &Manifest) -> String {
    let mut out = String::new();
    out.push_str(&format!("=== {} (md5: {})\n", m.source_path, m.md5));
    out.push_str(&format!("  Symbols  : {}\n", m.symbols.len()));
    out.push_str(&format!("  Usages   : {}\n", m.usages.len()));
    for sym in &m.symbols {
        out.push_str(&format!("  {}\n", format_symbol(sym)));
    }
    for (name, coords) in group_usages(&m.usages) {
        out.push_str(&format!("  {}\n", format_usage_grouped(&name, &coords)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_str_function() {
        assert_eq!(kind_str(&SymbolKind::Function), "FUNCTION");
    }

    #[test]
    fn test_kind_str_all_types() {
        assert_eq!(kind_str(&SymbolKind::Function), "FUNCTION");
        assert_eq!(kind_str(&SymbolKind::Procedure), "PROCEDURE");
        assert_eq!(kind_str(&SymbolKind::Method), "METHOD");
        assert_eq!(kind_str(&SymbolKind::Public), "PUBLIC");
        assert_eq!(kind_str(&SymbolKind::Static), "STATIC");
        assert_eq!(kind_str(&SymbolKind::Memvar), "MEMVAR");
        assert_eq!(kind_str(&SymbolKind::Access), "ACCESS");
        assert_eq!(kind_str(&SymbolKind::Assign), "ASSIGN");
        assert_eq!(kind_str(&SymbolKind::Class), "CLASS");
    }

    #[test]
    fn test_kind_str_classvar() {
        let classvar = SymbolKind::ClassVar {
            visibility: Visibility::Exported,
        };
        assert_eq!(kind_str(&classvar), "VAR");
    }

    #[test]
    fn test_vis_str_exported() {
        assert_eq!(vis_str(&Visibility::Exported), "EXPORTED");
    }

    #[test]
    fn test_vis_str_all_types() {
        assert_eq!(vis_str(&Visibility::Exported), "EXPORTED");
        assert_eq!(vis_str(&Visibility::Hidden), "HIDDEN");
        assert_eq!(vis_str(&Visibility::Protected), "PROTECTED");
    }

    #[test]
    fn test_format_symbol_simple() {
        let symbol = Symbol {
            name: "MyFunc".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 42,
            attributes: vec![],
            conditional: false,
        };

        let formatted = format_symbol(&symbol);
        assert!(formatted.contains("[SYMBOL]"));
        assert!(formatted.contains("[FUNCTION]"));
        assert!(formatted.contains("MyFunc"));
        assert!(formatted.contains("GLOBAL"));
        assert!(formatted.contains("42"));
        assert!(formatted.contains("-"));
    }

    #[test]
    fn test_format_symbol_conditional() {
        let symbol = Symbol {
            name: "DebugVar".to_string(),
            kind: SymbolKind::Memvar,
            scope: "GLOBAL".to_string(),
            line: 20,
            attributes: vec![],
            conditional: true,
        };

        let formatted = format_symbol(&symbol);
        assert!(formatted.contains("CONDITIONAL"));
    }

    #[test]
    fn test_format_symbol_with_attributes() {
        let symbol = Symbol {
            name: "MyVar".to_string(),
            kind: SymbolKind::Public,
            scope: "GLOBAL".to_string(),
            line: 10,
            attributes: vec!["DEFAULT".to_string(), "INIT".to_string()],
            conditional: false,
        };

        let formatted = format_symbol(&symbol);
        assert!(formatted.contains("DEFAULT"));
        assert!(formatted.contains("INIT"));
    }

    #[test]
    fn test_format_symbol_classvar_exported() {
        let symbol = Symbol {
            name: "pubVar".to_string(),
            kind: SymbolKind::ClassVar {
                visibility: Visibility::Exported,
            },
            scope: "MyClass".to_string(),
            line: 8,
            attributes: vec![],
            conditional: false,
        };

        let formatted = format_symbol(&symbol);
        assert!(formatted.contains("VAR"));
        assert!(formatted.contains("EXPORTED"));
    }

    #[test]
    fn test_format_symbol_classvar_hidden() {
        let symbol = Symbol {
            name: "privVar".to_string(),
            kind: SymbolKind::ClassVar {
                visibility: Visibility::Hidden,
            },
            scope: "MyClass".to_string(),
            line: 9,
            attributes: vec![],
            conditional: false,
        };

        let formatted = format_symbol(&symbol);
        assert!(formatted.contains("HIDDEN"));
    }

    #[test]
    fn test_format_usage_grouped() {
        let coords = vec![(10, 5), (20, 3), (30, 8)];
        let formatted = format_usage_grouped("MYFUNCTION", &coords);

        assert!(formatted.contains("[+]"));
        assert!(formatted.contains("MYFUNCTION"));
        assert!(formatted.contains("Linha:10"));
        assert!(formatted.contains("Coluna:5"));
        assert!(formatted.contains("Linha:20"));
    }

    #[test]
    fn test_format_usage_grouped_single() {
        let coords = vec![(15, 7)];
        let formatted = format_usage_grouped("FUNC", &coords);

        assert!(formatted.contains("[+] FUNC"));
        assert!(formatted.contains("Linha:15, Coluna:7"));
    }

    #[test]
    fn test_group_usages_empty() {
        let usages: Vec<Usage> = vec![];
        let grouped = group_usages(&usages);
        assert!(grouped.is_empty());
    }

    #[test]
    fn test_group_usages_single() {
        let usages = vec![Usage {
            name: "FUNC".to_string(),
            line: 10,
            col: 5,
        }];

        let grouped = group_usages(&usages);
        assert_eq!(grouped.len(), 1);
        assert!(grouped.contains_key("FUNC"));
        assert_eq!(grouped["FUNC"], vec![(10, 5)]);
    }

    #[test]
    fn test_group_usages_multiple() {
        let usages = vec![
            Usage {
                name: "FUNC".to_string(),
                line: 10,
                col: 5,
            },
            Usage {
                name: "FUNC".to_string(),
                line: 20,
                col: 3,
            },
            Usage {
                name: "OTHER".to_string(),
                line: 15,
                col: 7,
            },
        ];

        let grouped = group_usages(&usages);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped["FUNC"], vec![(10, 5), (20, 3)]);
        assert_eq!(grouped["OTHER"], vec![(15, 7)]);
    }

    #[test]
    fn test_render_mkh_header() {
        let manifest = Manifest {
            source_path: "test.prg".to_string(),
            md5: "abc123def456".to_string(),
            symbols: vec![],
            usages: vec![],
        };

        let rendered = render_mkh(&manifest);
        assert!(rendered.contains("MakerHeaderGenerate"));
        assert!(rendered.contains("SOURCE  : test.prg"));
        assert!(rendered.contains("MD5     : abc123def456"));
        assert!(rendered.contains("SYMBOLS : 0"));
        assert!(rendered.contains("[DEFINITIONS]"));
        assert!(rendered.contains("[USAGES]"));
    }

    #[test]
    fn test_render_mkh_with_symbols() {
        let symbol = Symbol {
            name: "MYFUNCTION".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 5,
            attributes: vec![],
            conditional: false,
        };

        let manifest = Manifest {
            source_path: "test.prg".to_string(),
            md5: "abc123".to_string(),
            symbols: vec![symbol],
            usages: vec![],
        };

        let rendered = render_mkh(&manifest);
        assert!(rendered.contains("SYMBOLS : 1"));
        assert!(rendered.contains("[SYMBOL]"));
        assert!(rendered.contains("[FUNCTION]"));
        assert!(rendered.contains("MYFUNCTION"));
    }

    #[test]
    fn test_render_stdout() {
        let symbol = Symbol {
            name: "FUNC".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 10,
            attributes: vec![],
            conditional: false,
        };

        let usage = Usage {
            name: "HELPER".to_string(),
            line: 20,
            col: 5,
        };

        let manifest = Manifest {
            source_path: "test.prg".to_string(),
            md5: "abc123".to_string(),
            symbols: vec![symbol],
            usages: vec![usage],
        };

        let output = render_stdout(&manifest);
        assert!(output.contains("test.prg"));
        assert!(output.contains("md5: abc123"));
        assert!(output.contains("Symbols  : 1"));
        assert!(output.contains("Usages   : 1"));
        assert!(output.contains("FUNC"));
        assert!(output.contains("HELPER"));
    }
}
