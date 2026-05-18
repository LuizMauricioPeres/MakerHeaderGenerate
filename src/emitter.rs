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
    let sites_map = group_usages(&m.call_sites);
    buf.push_str("[DEFINITIONS]\n");
    for sym in &m.symbols {
        let key = sym.name.to_ascii_uppercase();
        let sites = sites_map.get(&key).map(|v| v.as_slice()).unwrap_or(&[]);
        buf.push_str(&format_symbol(sym, sites));
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

fn format_symbol(sym: &Symbol, call_sites: &[(usize, usize)]) -> String {
    let tipo = kind_str(&sym.kind);
    let mut attrs: Vec<String> = sym.attributes.clone();
    if sym.conditional {
        attrs.push(String::from("CONDITIONAL"));
    }
    if let SymbolKind::ClassVar { visibility } = &sym.kind {
        attrs.push(vis_str(visibility).to_string());
    }

    let last_field = if call_sites.is_empty() {
        if attrs.is_empty() { String::from("-") } else { attrs.join(",") }
    } else {
        let locs: Vec<String> = call_sites
            .iter()
            .map(|(l, c)| format!("[Linha:{}, Coluna:{}]", l, c))
            .collect();
        let usos = format!("USOS: {{ {} }}", locs.join(", "));
        if attrs.is_empty() { usos } else { format!("{} | {}", attrs.join(","), usos) }
    };

    format!(
        "[SYMBOL] -> [{}] -> {} | {} | {} | {}",
        tipo, sym.name, sym.scope, sym.line, last_field
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

/// Render all manifests as `.vscode/hpts.json` symbol index.
pub fn render_hpts(manifests: &[&Manifest]) -> String {
    let mut entries: Vec<(&str, &str, usize, &'static str)> = Vec::new();

    for m in manifests {
        for sym in &m.symbols {
            entries.push((&sym.name, &m.source_path, sym.line, hpts_type(&sym.kind)));
        }
    }

    if entries.is_empty() {
        return String::from("[]");
    }

    let mut buf = String::with_capacity(entries.len() * 80 + 4);
    buf.push_str("[\n");

    for (i, (name, path, line, typ)) in entries.iter().enumerate() {
        let comma = if i + 1 < entries.len() { "," } else { "" };
        buf.push_str(&format!(
            "  {{ \"name\": \"{}\", \"filename\": \"{}\", \"line\": {}, \"type\": \"{}\" }}{}\n",
            json_escape(name),
            json_escape(path),
            line,
            typ,
            comma,
        ));
    }

    buf.push(']');
    buf
}

fn hpts_type(k: &SymbolKind) -> &'static str {
    match k {
        SymbolKind::Function => "function",
        SymbolKind::Procedure => "procedure",
        SymbolKind::Method => "method",
        SymbolKind::Class => "class",
        SymbolKind::Public => "public",
        SymbolKind::Static => "static",
        SymbolKind::Memvar => "memvar",
        SymbolKind::ClassVar { .. } => "var",
        SymbolKind::Access => "access",
        SymbolKind::Assign => "assign",
    }
}

fn json_escape(s: &str) -> String {
    s.chars().flat_map(|c| match c {
        '"'  => vec!['\\', '"'],
        '\\' => vec!['\\', '\\'],
        '\n' => vec!['\\', 'n'],
        '\r' => vec!['\\', 'r'],
        '\t' => vec!['\\', 't'],
        c    => vec![c],
    }).collect()
}

/// Render all manifests as a Universal ctags (format 2) `tags` file.
/// Sort is foldcase (case-insensitive), matching Harbour's case-insensitivity.
pub fn render_ctags(manifests: &[&Manifest]) -> String {
    let mut entries: Vec<(String, &str, usize, &'static str, Option<&str>)> = Vec::new();

    for m in manifests {
        for sym in &m.symbols {
            let scope = match sym.scope.as_str() {
                "GLOBAL" | "STATIC" => None,
                s => Some(s),
            };
            entries.push((sym.name.clone(), &m.source_path, sym.line, ctags_kind(&sym.kind), scope));
        }
    }

    entries.sort_by(|a, b| a.0.to_ascii_lowercase().cmp(&b.0.to_ascii_lowercase()));

    let mut buf = String::with_capacity(entries.len() * 80 + 256);
    buf.push_str("!_TAG_FILE_FORMAT\t2\t/extended format/\n");
    buf.push_str("!_TAG_FILE_SORTED\t2\t/0=unsorted, 1=sorted, 2=foldcase/\n");
    buf.push_str("!_TAG_PROGRAM_NAME\tmaker_header_gen\t//\n");

    for (name, path, line, kind, scope) in &entries {
        if let Some(s) = scope {
            buf.push_str(&format!("{}\t{}\t{};\"\t{}\tclass:{}\n", name, path, line, kind, s));
        } else {
            buf.push_str(&format!("{}\t{}\t{};\"\t{}\n", name, path, line, kind));
        }
    }

    buf
}

fn ctags_kind(k: &SymbolKind) -> &'static str {
    match k {
        SymbolKind::Function => "f",
        SymbolKind::Procedure => "p",
        SymbolKind::Method | SymbolKind::Access | SymbolKind::Assign => "m",
        SymbolKind::Class => "c",
        SymbolKind::Public | SymbolKind::Static | SymbolKind::Memvar => "v",
        SymbolKind::ClassVar { .. } => "m",
    }
}

/// Human-readable stdout rendering (for --verbose)
pub fn render_stdout(m: &Manifest) -> String {
    let mut out = String::new();
    out.push_str(&format!("=== {} (md5: {})\n", m.source_path, m.md5));
    out.push_str(&format!("  Symbols  : {}\n", m.symbols.len()));
    out.push_str(&format!("  Usages   : {}\n", m.usages.len()));
    let sites_map = group_usages(&m.call_sites);
    for sym in &m.symbols {
        let key = sym.name.to_ascii_uppercase();
        let sites = sites_map.get(&key).map(|v| v.as_slice()).unwrap_or(&[]);
        out.push_str(&format!("  {}\n", format_symbol(sym, sites)));
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

        let formatted = format_symbol(&symbol, &[]);
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

        let formatted = format_symbol(&symbol, &[]);
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

        let formatted = format_symbol(&symbol, &[]);
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

        let formatted = format_symbol(&symbol, &[]);
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

        let formatted = format_symbol(&symbol, &[]);
        assert!(formatted.contains("HIDDEN"));
    }

    #[test]
    fn test_format_symbol_with_call_sites() {
        let symbol = Symbol {
            name: "MYHELPER".to_string(),
            kind: SymbolKind::Function,
            scope: "STATIC".to_string(),
            line: 10,
            attributes: vec![],
            conditional: false,
        };

        let sites = [(20, 5), (30, 3)];
        let formatted = format_symbol(&symbol, &sites);
        assert!(formatted.contains("USOS:"));
        assert!(formatted.contains("Linha:20, Coluna:5"));
        assert!(formatted.contains("Linha:30, Coluna:3"));
    }

    #[test]
    fn test_format_symbol_conditional_with_call_sites() {
        let symbol = Symbol {
            name: "OPTFUNC".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 5,
            attributes: vec![],
            conditional: true,
        };

        let sites = [(15, 1)];
        let formatted = format_symbol(&symbol, &sites);
        assert!(formatted.contains("CONDITIONAL"));
        assert!(formatted.contains("USOS:"));
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
            call_sites: vec![],
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
            call_sites: vec![],
        };

        let rendered = render_mkh(&manifest);
        assert!(rendered.contains("SYMBOLS : 1"));
        assert!(rendered.contains("[SYMBOL]"));
        assert!(rendered.contains("[FUNCTION]"));
        assert!(rendered.contains("MYFUNCTION"));
    }

    #[test]
    fn test_hpts_type_mapping() {
        assert_eq!(hpts_type(&SymbolKind::Function), "function");
        assert_eq!(hpts_type(&SymbolKind::Procedure), "procedure");
        assert_eq!(hpts_type(&SymbolKind::Method), "method");
        assert_eq!(hpts_type(&SymbolKind::Class), "class");
        assert_eq!(hpts_type(&SymbolKind::Public), "public");
        assert_eq!(hpts_type(&SymbolKind::Static), "static");
        assert_eq!(hpts_type(&SymbolKind::Memvar), "memvar");
        assert_eq!(hpts_type(&SymbolKind::Access), "access");
        assert_eq!(hpts_type(&SymbolKind::Assign), "assign");
        assert_eq!(hpts_type(&SymbolKind::ClassVar { visibility: Visibility::Hidden }), "var");
    }

    #[test]
    fn test_render_hpts_empty() {
        let manifest = Manifest {
            source_path: "src/foo.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_hpts(&[&manifest]);
        assert_eq!(out.trim(), "[]");
    }

    #[test]
    fn test_render_hpts_single_symbol() {
        let symbol = Symbol {
            name: "GetUser".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 12,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "src/foo.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![symbol],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_hpts(&[&manifest]);
        assert!(out.contains("\"name\": \"GetUser\""));
        assert!(out.contains("\"filename\": \"src/foo.prg\""));
        assert!(out.contains("\"line\": 12"));
        assert!(out.contains("\"type\": \"function\""));
        // single entry → sem vírgula no final
        assert!(!out.trim_end().trim_end_matches(']').trim_end().ends_with(','));
    }

    #[test]
    fn test_render_hpts_multiple_symbols_trailing_comma() {
        let make = |name: &str, line: usize| Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "src/a.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![make("Foo", 1), make("Bar", 5)],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_hpts(&[&manifest]);
        // Foo deve ter vírgula, Bar não
        let foo_line = out.lines().find(|l| l.contains("\"Foo\"")).unwrap();
        let bar_line = out.lines().find(|l| l.contains("\"Bar\"")).unwrap();
        assert!(foo_line.ends_with(','));
        assert!(!bar_line.ends_with(','));
    }

    #[test]
    fn test_render_hpts_json_escape() {
        let symbol = Symbol {
            name: "My\"Func".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 1,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "path\\to\\file.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![symbol],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_hpts(&[&manifest]);
        assert!(out.contains("My\\\"Func"));
        assert!(out.contains("path\\\\to\\\\file.prg"));
    }

    #[test]
    fn test_render_hpts_valid_json_structure() {
        let symbol = Symbol {
            name: "Init".to_string(),
            kind: SymbolKind::Method,
            scope: "MyClass".to_string(),
            line: 20,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "src/myclass.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![symbol],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_hpts(&[&manifest]);
        assert!(out.starts_with('['));
        assert!(out.ends_with(']'));
        assert!(out.contains("\"type\": \"method\""));
    }

    #[test]
    fn test_ctags_kind_mapping() {
        assert_eq!(ctags_kind(&SymbolKind::Function), "f");
        assert_eq!(ctags_kind(&SymbolKind::Procedure), "p");
        assert_eq!(ctags_kind(&SymbolKind::Method), "m");
        assert_eq!(ctags_kind(&SymbolKind::Class), "c");
        assert_eq!(ctags_kind(&SymbolKind::Public), "v");
        assert_eq!(ctags_kind(&SymbolKind::Static), "v");
        assert_eq!(ctags_kind(&SymbolKind::Memvar), "v");
        assert_eq!(ctags_kind(&SymbolKind::Access), "m");
        assert_eq!(ctags_kind(&SymbolKind::Assign), "m");
        assert_eq!(ctags_kind(&SymbolKind::ClassVar { visibility: Visibility::Exported }), "m");
    }

    #[test]
    fn test_render_ctags_header() {
        let manifest = Manifest {
            source_path: "src/foo.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_ctags(&[&manifest]);
        assert!(out.contains("!_TAG_FILE_FORMAT\t2"));
        assert!(out.contains("!_TAG_FILE_SORTED\t2"));
        assert!(out.contains("!_TAG_PROGRAM_NAME\tmaker_header_gen"));
    }

    #[test]
    fn test_render_ctags_global_symbol() {
        let symbol = Symbol {
            name: "GetUser".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 12,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "src/foo.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![symbol],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_ctags(&[&manifest]);
        assert!(out.contains("GetUser\tsrc/foo.prg\t12;\"\tf\n"));
        assert!(!out.contains("class:"));
    }

    #[test]
    fn test_render_ctags_class_member_has_class_field() {
        let symbol = Symbol {
            name: "Save".to_string(),
            kind: SymbolKind::Method,
            scope: "MyClass".to_string(),
            line: 30,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "src/foo.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![symbol],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_ctags(&[&manifest]);
        assert!(out.contains("Save\tsrc/foo.prg\t30;\"\tm\tclass:MyClass"));
    }

    #[test]
    fn test_render_ctags_static_scope_no_class_field() {
        let symbol = Symbol {
            name: "InternalHelper".to_string(),
            kind: SymbolKind::Function,
            scope: "STATIC".to_string(),
            line: 5,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "src/foo.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![symbol],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_ctags(&[&manifest]);
        assert!(out.contains("InternalHelper\tsrc/foo.prg\t5;\"\tf\n"));
        assert!(!out.contains("class:"));
    }

    #[test]
    fn test_render_ctags_sorted_foldcase() {
        let make_sym = |name: &str, line: usize| Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line,
            attributes: vec![],
            conditional: false,
        };
        let manifest = Manifest {
            source_path: "src/foo.prg".to_string(),
            md5: "abc".to_string(),
            symbols: vec![make_sym("Zebra", 3), make_sym("apple", 1), make_sym("Banana", 2)],
            usages: vec![],
            call_sites: vec![],
        };
        let out = render_ctags(&[&manifest]);
        let pos_apple = out.find("apple").unwrap();
        let pos_banana = out.find("Banana").unwrap();
        let pos_zebra = out.find("Zebra").unwrap();
        assert!(pos_apple < pos_banana && pos_banana < pos_zebra);
    }

    #[test]
    fn test_render_ctags_multiple_manifests() {
        let make_manifest = |path: &str, name: &str, line: usize| Manifest {
            source_path: path.to_string(),
            md5: "abc".to_string(),
            symbols: vec![Symbol {
                name: name.to_string(),
                kind: SymbolKind::Function,
                scope: "GLOBAL".to_string(),
                line,
                attributes: vec![],
                conditional: false,
            }],
            usages: vec![],
            call_sites: vec![],
        };
        let m1 = make_manifest("src/a.prg", "FuncA", 1);
        let m2 = make_manifest("src/b.prg", "FuncB", 5);
        let out = render_ctags(&[&m1, &m2]);
        assert!(out.contains("FuncA\tsrc/a.prg\t1;\"\tf"));
        assert!(out.contains("FuncB\tsrc/b.prg\t5;\"\tf"));
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
            call_sites: vec![],
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
