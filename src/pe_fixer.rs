// ============================================================================
// PE FIXER - Advanced PE Validation and Repair
// ============================================================================
// This module provides comprehensive PE validation and automatic fixing
// to prevent ACCESS_VIOLATION and other runtime errors
// ============================================================================

use goblin::pe::PE;

#[derive(Debug, Clone)]
pub struct PEValidationResult {
    pub is_valid: bool,
    pub issues: Vec<PEIssue>,
    pub warnings: Vec<String>,
    pub fixes_applied: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PEIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub fix_suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueSeverity {
    Critical,  // Will cause crash
    Error,     // Will cause malfunction
    Warning,   // May cause issues
    Info,      // Informational
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueCategory {
    EntryPoint,
    ImportTable,
    Relocations,
    SectionAlignment,
    CodeIntegrity,
    DataReferences,
}

/// Validate assembled code before writing to PE
pub fn validate_assembled_code(
    code: &[u8],
    _original_pe: &PE,
    entry_offset: usize,
) -> PEValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    
    println!("🔍 [PE_FIXER] Validating assembled code...");
    println!("   Code size: {} bytes", code.len());
    println!("   Entry offset: 0x{:x}", entry_offset);
    
    // 1. Check entry point validity
    if entry_offset >= code.len() {
        issues.push(PEIssue {
            severity: IssueSeverity::Critical,
            category: IssueCategory::EntryPoint,
            description: format!(
                "Entry point (0x{:x}) is beyond code size ({} bytes)",
                entry_offset, code.len()
            ),
            fix_suggestion: Some("Entry point should be at start of code (offset 0)".to_string()),
        });
    } else {
        // Check if entry point has valid instruction
        if entry_offset + 16 <= code.len() {
            let entry_bytes = &code[entry_offset..entry_offset + 16];
            println!("   Entry point bytes: {:02x?}", entry_bytes);
            
            // Check for common invalid patterns
            if entry_bytes[0] == 0x00 {
                issues.push(PEIssue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::EntryPoint,
                    description: "Entry point starts with 0x00 (invalid instruction)".to_string(),
                    fix_suggestion: Some("Code may not be properly assembled".to_string()),
                });
            }
            
            // Check for NOP sled (might indicate wrong entry point)
            if entry_bytes.iter().take(8).all(|&b| b == 0x90) {
                warnings.push("Entry point starts with NOP sled - may be incorrect".to_string());
            }
        }
    }
    
    // 2. Check for null bytes in critical sections
    let null_count = code.iter().filter(|&&b| b == 0x00).count();
    let null_percentage = (null_count as f64 / code.len() as f64) * 100.0;
    
    if null_percentage > 50.0 {
        issues.push(PEIssue {
            severity: IssueSeverity::Error,
            category: IssueCategory::CodeIntegrity,
            description: format!(
                "Code contains {:.1}% null bytes - likely not properly assembled",
                null_percentage
            ),
            fix_suggestion: Some("Check assembler output for errors".to_string()),
        });
    } else if null_percentage > 20.0 {
        warnings.push(format!(
            "Code contains {:.1}% null bytes - may indicate issues",
            null_percentage
        ));
    }
    
    // 3. Check for valid x86-64 instruction patterns
    let valid_patterns = check_instruction_patterns(code, entry_offset);
    if !valid_patterns {
        issues.push(PEIssue {
            severity: IssueSeverity::Error,
            category: IssueCategory::CodeIntegrity,
            description: "Code does not contain valid x86-64 instruction patterns".to_string(),
            fix_suggestion: Some("Verify assembler is generating correct code".to_string()),
        });
    }
    
    // 4. Check for unresolved relocations (0xE8 00 00 00 00 pattern)
    let unresolved_calls = find_unresolved_calls(code);
    if !unresolved_calls.is_empty() {
        warnings.push(format!(
            "Found {} potentially unresolved CALL instructions",
            unresolved_calls.len()
        ));
        for offset in unresolved_calls.iter().take(3) {
            warnings.push(format!("  - At offset 0x{:x}", offset));
        }
    }
    
    // 5. Check for unresolved jumps (0xE9 00 00 00 00 pattern)
    let unresolved_jumps = find_unresolved_jumps(code);
    if !unresolved_jumps.is_empty() {
        warnings.push(format!(
            "Found {} potentially unresolved JMP instructions",
            unresolved_jumps.len()
        ));
    }
    
    let is_valid = issues.iter().all(|i| i.severity != IssueSeverity::Critical);
    
    println!("   Validation result: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
    println!("   Issues: {} critical, {} errors, {} warnings",
             issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count(),
             issues.iter().filter(|i| i.severity == IssueSeverity::Error).count(),
             warnings.len());
    
    PEValidationResult {
        is_valid,
        issues,
        warnings,
        fixes_applied: Vec::new(),
    }
}

/// Check if code contains valid x86-64 instruction patterns
fn check_instruction_patterns(code: &[u8], entry_offset: usize) -> bool {
    if entry_offset >= code.len() {
        return false;
    }
    
    // Check first few bytes at entry point for common valid patterns
    let check_bytes = &code[entry_offset..code.len().min(entry_offset + 32)];
    
    // Common valid x86-64 instruction prefixes and opcodes
    let valid_first_bytes = [
        0x48, 0x49, 0x4C, 0x4D, // REX prefixes
        0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, // PUSH reg
        0x8B, 0x89, // MOV
        0xE8, 0xE9, // CALL, JMP
        0x83, 0x81, // ADD, SUB, CMP with immediate
        0xC3, // RET
        0x90, // NOP
        0x0F, // Two-byte opcode prefix
        0xCC, // INT3 (breakpoint)
        0x31, 0x33, // XOR
        0x85, // TEST
        0xFF, // Various (CALL, JMP indirect, etc.)
    ];
    
    // Check if first instruction byte is valid
    if check_bytes.is_empty() {
        return false;
    }
    
    let first_byte = check_bytes[0];
    valid_first_bytes.contains(&first_byte)
}

/// Find potentially unresolved CALL instructions (E8 00 00 00 00)
fn find_unresolved_calls(code: &[u8]) -> Vec<usize> {
    let mut unresolved = Vec::new();
    
    for i in 0..code.len().saturating_sub(5) {
        if code[i] == 0xE8 && 
           code[i+1] == 0x00 && 
           code[i+2] == 0x00 && 
           code[i+3] == 0x00 && 
           code[i+4] == 0x00 {
            unresolved.push(i);
        }
    }
    
    unresolved
}

/// Find potentially unresolved JMP instructions (E9 00 00 00 00)
fn find_unresolved_jumps(code: &[u8]) -> Vec<usize> {
    let mut unresolved = Vec::new();
    
    for i in 0..code.len().saturating_sub(5) {
        if code[i] == 0xE9 && 
           code[i+1] == 0x00 && 
           code[i+2] == 0x00 && 
           code[i+3] == 0x00 && 
           code[i+4] == 0x00 {
            unresolved.push(i);
        }
    }
    
    unresolved
}

/// Attempt to automatically fix common issues
pub fn auto_fix_code(
    code: &mut Vec<u8>,
    _original_pe: &PE,
    _validation: &PEValidationResult,
) -> Vec<String> {
    let mut fixes = Vec::new();
    
    println!("🔧 [PE_FIXER] Attempting automatic fixes...");
    
    // Fix 1: If entry point is all zeros, add a minimal valid stub
    if code.len() >= 16 {
        let first_16 = &code[0..16];
        if first_16.iter().all(|&b| b == 0x00) {
            println!("   Fixing: Entry point is all zeros");
            // Add minimal stub: mov eax, 0; ret
            code[0] = 0xB8; // MOV EAX, imm32
            code[1] = 0x00;
            code[2] = 0x00;
            code[3] = 0x00;
            code[4] = 0x00;
            code[5] = 0xC3; // RET
            fixes.push("Added minimal entry point stub (mov eax, 0; ret)".to_string());
        }
    }
    
    // Fix 2: Ensure code ends with RET if it doesn't
    if !code.is_empty() {
        let last_byte = code[code.len() - 1];
        if last_byte != 0xC3 && last_byte != 0xCB && last_byte != 0xC2 {
            // Not a RET instruction - might cause issues
            // Don't auto-fix this as it might be intentional
            println!("   Warning: Code doesn't end with RET instruction");
        }
    }
    
    println!("   Applied {} automatic fixes", fixes.len());
    fixes
}

/// Generate detailed validation report
pub fn format_validation_report(validation: &PEValidationResult) -> String {
    let mut report = String::new();
    
    report.push_str("╔════════════════════════════════════════════════════════════════╗\n");
    report.push_str("║           PE VALIDATION REPORT                                 ║\n");
    report.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");
    
    if validation.is_valid {
        report.push_str("✅ Overall Status: VALID\n\n");
    } else {
        report.push_str("❌ Overall Status: INVALID (will likely crash)\n\n");
    }
    
    if !validation.issues.is_empty() {
        report.push_str("Issues Found:\n");
        for (i, issue) in validation.issues.iter().enumerate() {
            let severity_icon = match issue.severity {
                IssueSeverity::Critical => "🔴",
                IssueSeverity::Error => "🟠",
                IssueSeverity::Warning => "🟡",
                IssueSeverity::Info => "🔵",
            };
            
            report.push_str(&format!(
                "{}. {} [{:?}] {:?}\n   {}\n",
                i + 1,
                severity_icon,
                issue.severity,
                issue.category,
                issue.description
            ));
            
            if let Some(ref suggestion) = issue.fix_suggestion {
                report.push_str(&format!("   💡 {}\n", suggestion));
            }
            report.push('\n');
        }
    }
    
    if !validation.warnings.is_empty() {
        report.push_str("Warnings:\n");
        for warning in &validation.warnings {
            report.push_str(&format!("⚠️  {}\n", warning));
        }
        report.push('\n');
    }
    
    if !validation.fixes_applied.is_empty() {
        report.push_str("Automatic Fixes Applied:\n");
        for fix in &validation.fixes_applied {
            report.push_str(&format!("✓ {}\n", fix));
        }
        report.push('\n');
    }
    
    report
}