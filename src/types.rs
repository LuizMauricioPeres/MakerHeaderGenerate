/// Tipo de símbolo extraído do fonte Harbour
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Procedure,
    Method,
    Public,
    Static,
    Memvar,
    ClassVar { visibility: Visibility },
    Access,
    Assign,
    Class,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Exported,
    Hidden,
    Protected,
}

/// Um símbolo definido no fonte
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub scope: String,
    pub line: usize,
    pub attributes: Vec<String>,
    /// true se este símbolo está dentro de um bloco condicional
    pub conditional: bool,
}

/// Um uso de símbolo (chamada, referência)
#[derive(Debug, Clone)]
pub struct Usage {
    pub name: String,
    pub line: usize,
    pub col: usize,
}

/// Manifesto completo de um arquivo .prg
#[derive(Debug)]
pub struct Manifest {
    pub source_path: String,
    pub md5: String,
    pub symbols: Vec<Symbol>,
    pub usages: Vec<Usage>,
}
