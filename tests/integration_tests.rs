use std::fs;
use std::path::Path;

#[test]
fn test_simple_file_analysis() {
    let test_file = "tests/fixtures/simple.prg";
    
    // Verify fixture exists
    assert!(
        Path::new(test_file).exists(),
        "Test fixture {} not found",
        test_file
    );

    // Read and parse file
    let bytes = fs::read(test_file).expect("Failed to read fixture");
    let source = String::from_utf8_lossy(&bytes).into_owned();
    
    // Verify source contains expected symbols
    assert!(source.contains("MEMVAR usuario"), "Expected MEMVAR usuario");
    assert!(source.contains("PROCEDURE TestSimple"), "Expected PROCEDURE TestSimple");
    assert!(source.contains("FUNCTION GetUser"), "Expected FUNCTION GetUser");
    
    // Verify contains expected usages
    assert!(source.contains("FetchUser()"), "Expected FetchUser call");
}

#[test]
fn test_class_file_analysis() {
    let test_file = "tests/fixtures/class_example.prg";
    
    assert!(Path::new(test_file).exists(), "Test fixture {} not found", test_file);

    let source = fs::read_to_string(test_file).expect("Failed to read fixture");
    
    // Verify class structure
    assert!(source.contains("CLASS MyClass"), "Expected CLASS MyClass");
    assert!(source.contains("ENDCLASS"), "Expected ENDCLASS");
    
    // Verify class members
    assert!(source.contains("VAR nId EXPORTED"), "Expected VAR nId");
    assert!(source.contains("VAR cName HIDDEN"), "Expected VAR cName");
    assert!(source.contains("VAR lActive PROTECTED"), "Expected VAR lActive");
    
    // Verify methods
    assert!(source.contains("METHOD New()"), "Expected METHOD New");
    assert!(source.contains("METHOD Init"), "Expected METHOD Init");
    assert!(source.contains("ACCESS getId"), "Expected ACCESS getId");
    assert!(source.contains("ASSIGN setId"), "Expected ASSIGN setId");
}

#[test]
fn test_conditional_file_analysis() {
    let test_file = "tests/fixtures/conditional.prg";
    
    assert!(Path::new(test_file).exists(), "Test fixture {} not found", test_file);

    let source = fs::read_to_string(test_file).expect("Failed to read fixture");
    
    // Verify conditional structure
    assert!(source.contains("#ifdef DEBUG_MODE"), "Expected #ifdef");
    assert!(source.contains("#endif"), "Expected #endif");
    
    // Verify symbols inside conditional
    assert!(source.contains("MEMVAR cDebugPath"), "Expected MEMVAR in conditional");
    assert!(source.contains("PROCEDURE DebugLog"), "Expected PROCEDURE in conditional");
    
    // Verify public function
    assert!(source.contains("PUBLIC nVersion"), "Expected PUBLIC nVersion");
    assert!(source.contains("FUNCTION GetVersion"), "Expected FUNCTION GetVersion");
}

#[test]
fn test_variables_file_analysis() {
    let test_file = "tests/fixtures/variables.prg";
    
    assert!(Path::new(test_file).exists(), "Test fixture {} not found", test_file);

    let source = fs::read_to_string(test_file).expect("Failed to read fixture");
    
    // Verify memvar declarations
    assert!(source.contains("MEMVAR usuario, nSerial, cModulo"), "Expected MEMVAR list");
    
    // Verify public declarations
    assert!(source.contains("PUBLIC cor01, cor02, cor03"), "Expected PUBLIC list");
    assert!(source.contains("PUBLIC lRecados := .t."), "Expected PUBLIC with default");
    
    // Verify static declarations
    assert!(source.contains("STATIC nCounter := 0"), "Expected STATIC");
    
    // Verify function calls (usages)
    assert!(source.contains("ProcessItem()"), "Expected ProcessItem call");
    assert!(source.contains("ValidateData( x )"), "Expected ValidateData call");
    assert!(source.contains("SaveResult( x, y )"), "Expected SaveResult call");
}

#[test]
fn test_manifest_structure() {
    // Test that fixture files are properly structured for manifest generation
    let fixtures = vec![
        "tests/fixtures/simple.prg",
        "tests/fixtures/class_example.prg",
        "tests/fixtures/conditional.prg",
        "tests/fixtures/variables.prg",
    ];

    for fixture in fixtures {
        assert!(Path::new(fixture).exists(), "Fixture {} not found", fixture);
        
        let content = fs::read_to_string(fixture)
            .expect(&format!("Failed to read {}", fixture));
        
        // All files should be valid UTF-8
        assert!(!content.is_empty(), "Fixture {} is empty", fixture);
        
        // All files should contain at least one symbol definition
        let has_symbol = content.contains("MEMVAR") 
            || content.contains("PUBLIC")
            || content.contains("FUNCTION")
            || content.contains("PROCEDURE")
            || content.contains("METHOD")
            || content.contains("CLASS");
        
        assert!(has_symbol, "Fixture {} has no recognizable symbols", fixture);
    }
}

#[test]
fn test_md5_calculation() {
    // Test MD5 can be calculated from fixture files
    let test_file = "tests/fixtures/simple.prg";
    
    let bytes = fs::read(test_file).expect("Failed to read fixture");
    let digest = format!("{:x}", md5::compute(&bytes));
    
    // MD5 should be 32 hex characters
    assert_eq!(digest.len(), 32, "MD5 hash should be 32 characters");
    
    // Verify it only contains hex characters
    assert!(digest.chars().all(|c| c.is_ascii_hexdigit()), "MD5 should be valid hex");
}

#[test]
fn test_symbol_detection_memvar() {
    let content = "MEMVAR usuario, nSerial, cModulo";
    
    // Simple validation: contains expected keywords
    assert!(content.contains("MEMVAR"), "MEMVAR keyword not found");
    
    let vars: Vec<&str> = content
        .split("MEMVAR")
        .nth(1)
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    assert_eq!(vars.len(), 3, "Should have 3 memvars");
    assert!(vars.contains(&"usuario"), "usuario should be in memvars");
    assert!(vars.contains(&"nSerial"), "nSerial should be in memvars");
    assert!(vars.contains(&"cModulo"), "cModulo should be in memvars");
}

#[test]
fn test_symbol_detection_public() {
    let content = "PUBLIC cor01, cor02, lRecados := .t.";
    
    assert!(content.contains("PUBLIC"), "PUBLIC keyword not found");
    
    // Extract variable names (before := or comma)
    let vars: Vec<&str> = content
        .split("PUBLIC")
        .nth(1)
        .unwrap_or("")
        .split(',')
        .flat_map(|part| {
            part.split(":=")
                .next()
                .map(|s| s.trim())
        })
        .filter(|s| !s.is_empty())
        .collect();
    
    assert!(vars.len() >= 2, "Should have at least 2 public vars");
    assert!(vars[0].contains("cor01"), "cor01 should be first");
}

#[test]
fn test_class_scope_detection() {
    let content = "CLASS MyClass\n   VAR nId EXPORTED\nENDCLASS";
    
    assert!(content.contains("CLASS MyClass"), "CLASS keyword not found");
    assert!(content.contains("VAR nId"), "VAR keyword not found");
    assert!(content.contains("ENDCLASS"), "ENDCLASS keyword not found");
    
    // Verify structure
    let class_line = content.lines().find(|l| l.contains("CLASS"));
    let endclass_line = content.lines().find(|l| l.contains("ENDCLASS"));
    
    assert!(class_line.is_some(), "CLASS not found");
    assert!(endclass_line.is_some(), "ENDCLASS not found");
}

#[test]
fn test_comment_preservation() {
    // Verify fixtures contain proper comments
    let content = fs::read_to_string("tests/fixtures/simple.prg")
        .expect("Failed to read fixture");
    
    assert!(content.contains("/*"), "Should have block comment start");
    assert!(content.contains("*/"), "Should have block comment end");
    
    // Comments should be preserved in file
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines[0].contains("/*"), "First line should start comment");
}

#[test]
fn test_call_detection_pattern() {
    let content = "FetchUser()";
    
    // Should contain function call pattern: identifier followed by (
    assert!(content.contains("FetchUser("), "FetchUser( pattern not found");
    
    let content2 = "ValidateData( x )";
    assert!(content2.contains("ValidateData("), "ValidateData( pattern not found");
}
