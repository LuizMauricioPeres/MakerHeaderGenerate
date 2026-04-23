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
