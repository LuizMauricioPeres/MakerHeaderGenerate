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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_kind_equality() {
        assert_eq!(SymbolKind::Function, SymbolKind::Function);
        assert_ne!(SymbolKind::Function, SymbolKind::Procedure);
        assert_eq!(SymbolKind::Method, SymbolKind::Method);
    }

    #[test]
    fn test_visibility_equality() {
        assert_eq!(Visibility::Exported, Visibility::Exported);
        assert_ne!(Visibility::Exported, Visibility::Hidden);
        assert_ne!(Visibility::Protected, Visibility::Hidden);
    }

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol {
            name: "MyFunction".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 42,
            attributes: vec![],
            conditional: false,
        };

        assert_eq!(symbol.name, "MyFunction");
        assert_eq!(symbol.kind, SymbolKind::Function);
        assert_eq!(symbol.scope, "GLOBAL");
        assert_eq!(symbol.line, 42);
        assert!(!symbol.conditional);
    }

    #[test]
    fn test_symbol_with_attributes() {
        let symbol = Symbol {
            name: "MyVar".to_string(),
            kind: SymbolKind::Public,
            scope: "GLOBAL".to_string(),
            line: 10,
            attributes: vec!["DEFAULT".to_string()],
            conditional: false,
        };

        assert_eq!(symbol.attributes.len(), 1);
        assert_eq!(symbol.attributes[0], "DEFAULT");
    }

    #[test]
    fn test_symbol_conditional() {
        let symbol = Symbol {
            name: "DebugVar".to_string(),
            kind: SymbolKind::Memvar,
            scope: "GLOBAL".to_string(),
            line: 20,
            attributes: vec![],
            conditional: true,
        };

        assert!(symbol.conditional);
    }

    #[test]
    fn test_usage_creation() {
        let usage = Usage {
            name: "CallMe".to_string(),
            line: 55,
            col: 10,
        };

        assert_eq!(usage.name, "CallMe");
        assert_eq!(usage.line, 55);
        assert_eq!(usage.col, 10);
    }

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest {
            source_path: "test.prg".to_string(),
            md5: "abc123".to_string(),
            symbols: vec![],
            usages: vec![],
        };

        assert_eq!(manifest.source_path, "test.prg");
        assert_eq!(manifest.md5, "abc123");
        assert!(manifest.symbols.is_empty());
        assert!(manifest.usages.is_empty());
    }

    #[test]
    fn test_manifest_with_symbols() {
        let symbol1 = Symbol {
            name: "Func1".to_string(),
            kind: SymbolKind::Function,
            scope: "GLOBAL".to_string(),
            line: 5,
            attributes: vec![],
            conditional: false,
        };

        let symbol2 = Symbol {
            name: "Func2".to_string(),
            kind: SymbolKind::Procedure,
            scope: "GLOBAL".to_string(),
            line: 10,
            attributes: vec![],
            conditional: false,
        };

        let manifest = Manifest {
            source_path: "test.prg".to_string(),
            md5: "abc123".to_string(),
            symbols: vec![symbol1, symbol2],
            usages: vec![],
        };

        assert_eq!(manifest.symbols.len(), 2);
        assert_eq!(manifest.symbols[0].name, "Func1");
        assert_eq!(manifest.symbols[1].name, "Func2");
    }

    #[test]
    fn test_class_var_visibility() {
        let exported = Symbol {
            name: "pubVar".to_string(),
            kind: SymbolKind::ClassVar {
                visibility: Visibility::Exported,
            },
            scope: "MyClass".to_string(),
            line: 8,
            attributes: vec![],
            conditional: false,
        };

        let hidden = Symbol {
            name: "privVar".to_string(),
            kind: SymbolKind::ClassVar {
                visibility: Visibility::Hidden,
            },
            scope: "MyClass".to_string(),
            line: 9,
            attributes: vec![],
            conditional: false,
        };

        let protected = Symbol {
            name: "protVar".to_string(),
            kind: SymbolKind::ClassVar {
                visibility: Visibility::Protected,
            },
            scope: "MyClass".to_string(),
            line: 10,
            attributes: vec![],
            conditional: false,
        };

        if let SymbolKind::ClassVar { visibility } = &exported.kind {
            assert_eq!(*visibility, Visibility::Exported);
        } else {
            panic!("Expected ClassVar");
        }

        if let SymbolKind::ClassVar { visibility } = &hidden.kind {
            assert_eq!(*visibility, Visibility::Hidden);
        } else {
            panic!("Expected ClassVar");
        }

        if let SymbolKind::ClassVar { visibility } = &protected.kind {
            assert_eq!(*visibility, Visibility::Protected);
        } else {
            panic!("Expected ClassVar");
        }
    }
}
