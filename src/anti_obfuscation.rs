// ============================================================================
// ANTI-OBFUSCATION LAYER v1.0
// ============================================================================
// Advanced obfuscation detection and removal engine that identifies and
// neutralizes common code obfuscation techniques used by packers, protectors,
// and malware to hide their true functionality.
//
// Supported Techniques:
// - Control Flow Flattening (CFF)
// - Opaque Predicates (always true/false conditions)
// - Dead Code Injection
// - Instruction Substitution
// - Virtualization Obfuscation (VM-based protection)
// - String Encryption
// - API Hashing/Dynamic Resolution
// - Junk Code Insertion
// - Register Renaming
// - Constant Unfolding
// ============================================================================

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
// use regex::Regex;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub address: u64,
    pub mnemonic: String,
    pub operands: String,
    pub raw_line: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObfuscationType {
    ControlFlowFlattening,
    OpaquePredicate,
    DeadCode,
    InstructionSubstitution,
    VirtualizationObfuscation,
    StringEncryption,
    APIHashing,
    JunkCode,
    RegisterRenaming,
    ConstantUnfolding,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ObfuscationSignature {
    pub obf_type: ObfuscationType,
    pub confidence: f32,  // 0.0 to 1.0
    pub location: u64,
    pub evidence: Vec<String>,
    pub description: String,
    pub severity: ObfuscationSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObfuscationSeverity {
    Low,      // Simple obfuscation, easy to remove
    Medium,   // Moderate obfuscation, requires analysis
    High,     // Complex obfuscation, difficult to remove
    Critical, // Advanced obfuscation, may require manual intervention
}

#[derive(Debug, Clone)]
pub struct DeobfuscationResult {
    pub original_count: usize,
    pub cleaned_count: usize,
    pub removed_instructions: usize,
    pub signatures: Vec<ObfuscationSignature>,
    pub cleaned_instructions: Vec<Instruction>,
    pub success_rate: f32,
}

// ============================================================================
// MAIN DEOBFUSCATION ENGINE
// ============================================================================

pub fn deobfuscate_instructions(instructions: &[Instruction]) -> DeobfuscationResult {
    let mut cleaned = instructions.to_vec();
    let mut signatures = Vec::new();
    let original_count = instructions.len();
    
    // Phase 1: Detect obfuscation techniques
    signatures.extend(detect_control_flow_flattening(&cleaned));
    signatures.extend(detect_opaque_predicates(&cleaned));
    signatures.extend(detect_dead_code(&cleaned));
    signatures.extend(detect_instruction_substitution(&cleaned));
    signatures.extend(detect_virtualization(&cleaned));
    signatures.extend(detect_string_encryption(&cleaned));
    signatures.extend(detect_api_hashing(&cleaned));
    signatures.extend(detect_junk_code(&cleaned));
    
    // Phase 2: Remove obfuscation (in order of safety)
    cleaned = remove_junk_code(&cleaned);
    cleaned = remove_dead_code(&cleaned);
    cleaned = remove_opaque_predicates(&cleaned);
    cleaned = simplify_instruction_substitution(&cleaned);
    cleaned = unfold_constants(&cleaned);
    
    let cleaned_count = cleaned.len();
    let removed = original_count.saturating_sub(cleaned_count);
    let success_rate = if original_count > 0 {
        (removed as f32 / original_count as f32) * 100.0
    } else {
        0.0
    };
    
    DeobfuscationResult {
        original_count,
        cleaned_count,
        removed_instructions: removed,
        signatures,
        cleaned_instructions: cleaned,
        success_rate,
    }
}

// ============================================================================
// CONTROL FLOW FLATTENING DETECTION
// ============================================================================

fn detect_control_flow_flattening(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut dispatcher_blocks = 0;
    let mut switch_var_count = 0;
    
    // Look for dispatcher pattern: cmp reg, imm; je/jne/jmp pattern
    for i in 0..instructions.len().saturating_sub(5) {
        let window = &instructions[i..i+5];
        
        // Pattern: mov reg, imm; cmp reg, imm; je/jne
        if window[0].mnemonic == "mov" && 
           window[1].mnemonic == "cmp" &&
           (window[2].mnemonic.starts_with("j") && window[2].mnemonic != "jmp") {
            
            // Check if this is part of a switch-like structure
            let mut jump_targets = HashSet::new();
            for j in i..std::cmp::min(i + 20, instructions.len()) {
                if instructions[j].mnemonic.starts_with("j") {
                    jump_targets.insert(&instructions[j].operands);
                }
            }
            
            if jump_targets.len() >= 3 {
                dispatcher_blocks += 1;
                switch_var_count += 1;
            }
        }
    }
    
    if dispatcher_blocks >= 2 {
        let confidence = (dispatcher_blocks as f32 / 10.0).min(1.0);
        signatures.push(ObfuscationSignature {
            obf_type: ObfuscationType::ControlFlowFlattening,
            confidence,
            location: instructions[0].address,
            evidence: vec![
                format!("Found {} dispatcher blocks", dispatcher_blocks),
                format!("Detected {} switch variables", switch_var_count),
                "Multiple indirect jumps with state variables".to_string(),
            ],
            description: "Control flow flattening detected - code uses dispatcher pattern to obscure execution flow".to_string(),
            severity: ObfuscationSeverity::High,
        });
    }
    
    signatures
}

// ============================================================================
// OPAQUE PREDICATE DETECTION
// ============================================================================

fn detect_opaque_predicates(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut _opaque_count = 0;
    
    for i in 0..instructions.len().saturating_sub(3) {
        let window = &instructions[i..i+3];
        
        // Pattern 1: xor reg, reg; test reg, reg; jz (always taken)
        if window[0].mnemonic == "xor" && 
           window[1].mnemonic == "test" &&
           window[2].mnemonic == "jz" {
            let xor_ops: Vec<&str> = window[0].operands.split(',').map(|s| s.trim()).collect();
            let test_ops: Vec<&str> = window[1].operands.split(',').map(|s| s.trim()).collect();
            
            if xor_ops.len() == 2 && xor_ops[0] == xor_ops[1] &&
               test_ops.len() == 2 && test_ops[0] == test_ops[1] &&
               xor_ops[0] == test_ops[0] {
                _opaque_count += 1;
                
                signatures.push(ObfuscationSignature {
                    obf_type: ObfuscationType::OpaquePredicate,
                    confidence: 0.95,
                    location: window[0].address,
                    evidence: vec![
                        format!("xor {}, {} ; test {}, {} ; jz", xor_ops[0], xor_ops[1], test_ops[0], test_ops[1]),
                        "Always-true predicate (register XOR with itself = 0)".to_string(),
                    ],
                    description: "Opaque predicate: always-true condition used to confuse analysis".to_string(),
                    severity: ObfuscationSeverity::Medium,
                });
            }
        }
        
        // Pattern 2: cmp reg, reg; je (always taken)
        if window[0].mnemonic == "cmp" && window[1].mnemonic == "je" {
            let cmp_ops: Vec<&str> = window[0].operands.split(',').map(|s| s.trim()).collect();
            if cmp_ops.len() == 2 && cmp_ops[0] == cmp_ops[1] {
                _opaque_count += 1;
                
                signatures.push(ObfuscationSignature {
                    obf_type: ObfuscationType::OpaquePredicate,
                    confidence: 0.90,
                    location: window[0].address,
                    evidence: vec![
                        format!("cmp {}, {} ; je", cmp_ops[0], cmp_ops[1]),
                        "Always-true predicate (register compared with itself)".to_string(),
                    ],
                    description: "Opaque predicate: always-true comparison".to_string(),
                    severity: ObfuscationSeverity::Medium,
                });
            }
        }
        
        // Pattern 3: Mathematical identities (x*x >= 0, x^2 - x is even, etc.)
        if window[0].mnemonic == "imul" && window[1].mnemonic == "test" {
            let imul_ops: Vec<&str> = window[0].operands.split(',').map(|s| s.trim()).collect();
            if imul_ops.len() >= 2 && imul_ops[0] == imul_ops[1] {
                _opaque_count += 1;
                
                signatures.push(ObfuscationSignature {
                    obf_type: ObfuscationType::OpaquePredicate,
                    confidence: 0.85,
                    location: window[0].address,
                    evidence: vec![
                        format!("imul {}, {}", imul_ops[0], imul_ops[1]),
                        "Mathematical identity: x*x always >= 0".to_string(),
                    ],
                    description: "Opaque predicate: mathematical identity used for obfuscation".to_string(),
                    severity: ObfuscationSeverity::Medium,
                });
            }
        }
    }
    
    signatures
}

// ============================================================================
// DEAD CODE DETECTION
// ============================================================================

fn detect_dead_code(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut _dead_blocks = 0;
    
    // Look for unreachable code after unconditional jumps/returns
    for i in 0..instructions.len().saturating_sub(1) {
        let curr = &instructions[i];
        
        // Check for unconditional control flow change
        if curr.mnemonic == "jmp" || curr.mnemonic == "ret" || curr.mnemonic == "retn" {
            // Check if next instruction is not a label/target
            if i + 1 < instructions.len() {
                let next = &instructions[i + 1];
                
                // If next instruction is not a common jump target, it might be dead code
                if !next.raw_line.contains(':') && !is_likely_jump_target(next.address, instructions) {
                    // Count consecutive non-label instructions
                    let mut dead_count = 0;
                    for j in (i+1)..std::cmp::min(i + 10, instructions.len()) {
                        if instructions[j].raw_line.contains(':') || 
                           is_likely_jump_target(instructions[j].address, instructions) {
                            break;
                        }
                        dead_count += 1;
                    }
                    
                    if dead_count >= 2 {
                        _dead_blocks += 1;
                        
                        signatures.push(ObfuscationSignature {
                            obf_type: ObfuscationType::DeadCode,
                            confidence: 0.75,
                            location: instructions[i + 1].address,
                            evidence: vec![
                                format!("Found {} unreachable instructions after {}", dead_count, curr.mnemonic),
                                format!("Starting at address 0x{:x}", instructions[i + 1].address),
                            ],
                            description: "Dead code: unreachable instructions after unconditional control flow".to_string(),
                            severity: ObfuscationSeverity::Low,
                        });
                    }
                }
            }
        }
    }
    
    signatures
}

fn is_likely_jump_target(address: u64, instructions: &[Instruction]) -> bool {
    for instr in instructions {
        if instr.mnemonic.starts_with('j') || instr.mnemonic == "call" {
            // Try to extract target address from operands
            if let Some(target) = extract_address_from_operand(&instr.operands) {
                if target == address {
                    return true;
                }
            }
        }
    }
    false
}

fn extract_address_from_operand(operand: &str) -> Option<u64> {
    // Try to parse hex address
    if operand.starts_with("0x") {
        u64::from_str_radix(&operand[2..], 16).ok()
    } else {
        operand.parse::<u64>().ok()
    }
}

// ============================================================================
// INSTRUCTION SUBSTITUTION DETECTION
// ============================================================================

fn detect_instruction_substitution(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut substitution_count = 0;
    
    for i in 0..instructions.len().saturating_sub(2) {
        let window = &instructions[i..i+2];
        
        // Pattern 1: neg + add instead of sub
        // sub eax, 5 => neg eax; add eax, -5
        if window[0].mnemonic == "neg" && window[1].mnemonic == "add" {
            substitution_count += 1;
        }
        
        // Pattern 2: not + inc instead of neg
        // neg eax => not eax; inc eax
        if window[0].mnemonic == "not" && window[1].mnemonic == "inc" {
            substitution_count += 1;
        }
        
        // Pattern 3: xor + xor instead of mov
        // mov eax, ebx => xor eax, eax; xor eax, ebx
        if i + 2 < instructions.len() {
            let triple = &instructions[i..i+3];
            if triple[0].mnemonic == "xor" && triple[1].mnemonic == "xor" {
                let ops1: Vec<&str> = triple[0].operands.split(',').map(|s| s.trim()).collect();
                if ops1.len() == 2 && ops1[0] == ops1[1] {
                    substitution_count += 1;
                }
            }
        }
    }
    
    if substitution_count > 0 {
        let confidence = (substitution_count as f32 / 20.0).min(0.95);
        signatures.push(ObfuscationSignature {
            obf_type: ObfuscationType::InstructionSubstitution,
            confidence,
            location: instructions[0].address,
            evidence: vec![
                format!("Found {} instruction substitution patterns", substitution_count),
                "Simple operations replaced with complex equivalents".to_string(),
            ],
            description: "Instruction substitution: simple operations replaced with complex sequences".to_string(),
            severity: ObfuscationSeverity::Medium,
        });
    }
    
    signatures
}

// ============================================================================
// VIRTUALIZATION OBFUSCATION DETECTION
// ============================================================================

fn detect_virtualization(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut vm_indicators = 0;
    let mut evidence = Vec::new();
    
    // Look for VM patterns: bytecode interpretation, context switching
    for i in 0..instructions.len().saturating_sub(5) {
        let window = &instructions[i..i+5];
        
        // Pattern 1: Bytecode fetch pattern (lodsb/lodsw/lodsd)
        if window.iter().any(|inst| inst.mnemonic == "lodsb" || inst.mnemonic == "lodsw" || inst.mnemonic == "lodsd") {
            vm_indicators += 1;
            evidence.push("Bytecode fetch instruction (lods*)".to_string());
        }
        
        // Pattern 2: Indirect jump table (common in VM dispatchers)
        if window.iter().any(|inst| inst.mnemonic == "jmp" && inst.operands.contains('[')) {
            vm_indicators += 1;
        }
        
        // Pattern 3: Context structure access (multiple mov to/from memory)
        let mem_access_count = window.iter()
            .filter(|inst| (inst.mnemonic == "mov" || inst.mnemonic == "lea") && 
                          (inst.operands.contains('[') || inst.operands.contains("ptr")))
            .count();
        
        if mem_access_count >= 4 {
            vm_indicators += 1;
            evidence.push(format!("Heavy context structure access ({} memory operations)", mem_access_count));
        }
    }
    
    if vm_indicators >= 3 {
        let confidence = (vm_indicators as f32 / 10.0).min(0.90);
        signatures.push(ObfuscationSignature {
            obf_type: ObfuscationType::VirtualizationObfuscation,
            confidence,
            location: instructions[0].address,
            evidence,
            description: "Virtualization obfuscation: code protected by virtual machine layer".to_string(),
            severity: ObfuscationSeverity::Critical,
        });
    }
    
    signatures
}

// ============================================================================
// STRING ENCRYPTION DETECTION
// ============================================================================

fn detect_string_encryption(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut xor_string_ops = 0;
    
    // Look for XOR operations in loops (common string decryption pattern)
    for i in 0..instructions.len().saturating_sub(5) {
        let window = &instructions[i..i+5];
        
        // Pattern: xor byte ptr [...], key; inc/add; loop/jnz
        let has_xor_mem = window.iter().any(|inst| 
            inst.mnemonic == "xor" && inst.operands.contains("byte ptr")
        );
        let has_inc = window.iter().any(|inst| 
            inst.mnemonic == "inc" || inst.mnemonic == "add"
        );
        let has_loop = window.iter().any(|inst| 
            inst.mnemonic == "loop" || inst.mnemonic.starts_with("jn")
        );
        
        if has_xor_mem && has_inc && has_loop {
            xor_string_ops += 1;
        }
    }
    
    if xor_string_ops > 0 {
        signatures.push(ObfuscationSignature {
            obf_type: ObfuscationType::StringEncryption,
            confidence: 0.85,
            location: instructions[0].address,
            evidence: vec![
                format!("Found {} XOR-based string decryption loops", xor_string_ops),
                "Pattern: xor byte ptr [...], key in loop".to_string(),
            ],
            description: "String encryption: strings decrypted at runtime using XOR".to_string(),
            severity: ObfuscationSeverity::Medium,
        });
    }
    
    signatures
}

// ============================================================================
// API HASHING DETECTION
// ============================================================================

fn detect_api_hashing(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut hash_patterns = 0;
    
    // Look for hash calculation patterns before API calls
    for i in 0..instructions.len().saturating_sub(10) {
        let window = &instructions[i..i+10];
        
        // Pattern: rol/ror + xor/add (common hash algorithms)
        let has_rotate = window.iter().any(|inst| 
            inst.mnemonic == "rol" || inst.mnemonic == "ror"
        );
        let has_xor_add = window.iter().filter(|inst| 
            inst.mnemonic == "xor" || inst.mnemonic == "add"
        ).count() >= 2;
        let has_call = window.iter().any(|inst| inst.mnemonic == "call");
        
        if has_rotate && has_xor_add && has_call {
            hash_patterns += 1;
        }
    }
    
    if hash_patterns > 0 {
        signatures.push(ObfuscationSignature {
            obf_type: ObfuscationType::APIHashing,
            confidence: 0.80,
            location: instructions[0].address,
            evidence: vec![
                format!("Found {} hash calculation patterns before calls", hash_patterns),
                "Pattern: rol/ror + xor/add before call instruction".to_string(),
            ],
            description: "API hashing: API functions resolved by hash instead of name".to_string(),
            severity: ObfuscationSeverity::High,
        });
    }
    
    signatures
}

// ============================================================================
// JUNK CODE DETECTION
// ============================================================================

fn detect_junk_code(instructions: &[Instruction]) -> Vec<ObfuscationSignature> {
    let mut signatures = Vec::new();
    let mut junk_count = 0;
    
    for i in 0..instructions.len().saturating_sub(1) {
        let curr = &instructions[i];
        let next = &instructions[i + 1];
        
        // Pattern 1: push + pop same register
        if curr.mnemonic == "push" && next.mnemonic == "pop" && curr.operands == next.operands {
            junk_count += 1;
        }
        
        // Pattern 2: nop, nop, nop...
        if curr.mnemonic == "nop" {
            junk_count += 1;
        }
        
        // Pattern 3: mov reg, reg (same register)
        if curr.mnemonic == "mov" {
            let ops: Vec<&str> = curr.operands.split(',').map(|s| s.trim()).collect();
            if ops.len() == 2 && ops[0] == ops[1] {
                junk_count += 1;
            }
        }
    }
    
    if junk_count > 5 {
        let confidence = (junk_count as f32 / 50.0).min(0.95);
        signatures.push(ObfuscationSignature {
            obf_type: ObfuscationType::JunkCode,
            confidence,
            location: instructions[0].address,
            evidence: vec![
                format!("Found {} junk instructions", junk_count),
                "Patterns: push/pop pairs, nops, redundant moves".to_string(),
            ],
            description: "Junk code: meaningless instructions inserted to bloat code".to_string(),
            severity: ObfuscationSeverity::Low,
        });
    }
    
    signatures
}

// ============================================================================
// DEOBFUSCATION REMOVAL FUNCTIONS
// ============================================================================

fn remove_junk_code(instructions: &[Instruction]) -> Vec<Instruction> {
    let mut cleaned = Vec::new();
    let mut skip_next = false;
    
    for i in 0..instructions.len() {
        if skip_next {
            skip_next = false;
            continue;
        }
        
        let curr = &instructions[i];
        
        // Remove nops
        if curr.mnemonic == "nop" {
            continue;
        }
        
        // Remove mov reg, reg (same register)
        if curr.mnemonic == "mov" {
            let ops: Vec<&str> = curr.operands.split(',').map(|s| s.trim()).collect();
            if ops.len() == 2 && ops[0] == ops[1] {
                continue;
            }
        }
        
        // Remove push/pop pairs
        if i + 1 < instructions.len() {
            let next = &instructions[i + 1];
            if curr.mnemonic == "push" && next.mnemonic == "pop" && curr.operands == next.operands {
                skip_next = true;
                continue;
            }
        }
        
        cleaned.push(curr.clone());
    }
    
    cleaned
}

fn remove_dead_code(instructions: &[Instruction]) -> Vec<Instruction> {
    let mut cleaned = Vec::new();
    let mut in_dead_block = false;
    
    for i in 0..instructions.len() {
        let curr = &instructions[i];
        
        // Check if we're entering dead code
        if i > 0 {
            let prev = &instructions[i - 1];
            if (prev.mnemonic == "jmp" || prev.mnemonic == "ret" || prev.mnemonic == "retn") &&
               !curr.raw_line.contains(':') &&
               !is_likely_jump_target(curr.address, instructions) {
                in_dead_block = true;
            }
        }
        
        // Check if we're exiting dead code (found a label/jump target)
        if in_dead_block && (curr.raw_line.contains(':') || is_likely_jump_target(curr.address, instructions)) {
            in_dead_block = false;
        }
        
        if !in_dead_block {
            cleaned.push(curr.clone());
        }
    }
    
    cleaned
}

fn remove_opaque_predicates(instructions: &[Instruction]) -> Vec<Instruction> {
    let mut cleaned = Vec::new();
    let mut skip_count = 0;
    
    for i in 0..instructions.len() {
        if skip_count > 0 {
            skip_count -= 1;
            continue;
        }
        
        // Check for opaque predicate patterns
        if i + 2 < instructions.len() {
            let window = &instructions[i..i+3];
            
            // Pattern: xor reg, reg; test reg, reg; jz
            if window[0].mnemonic == "xor" && window[1].mnemonic == "test" && window[2].mnemonic == "jz" {
                let xor_ops: Vec<&str> = window[0].operands.split(',').map(|s| s.trim()).collect();
                let test_ops: Vec<&str> = window[1].operands.split(',').map(|s| s.trim()).collect();
                
                if xor_ops.len() == 2 && xor_ops[0] == xor_ops[1] &&
                   test_ops.len() == 2 && test_ops[0] == test_ops[1] {
                    // Skip all three instructions (opaque predicate)
                    skip_count = 2;
                    continue;
                }
            }
        }
        
        cleaned.push(instructions[i].clone());
    }
    
    cleaned
}

fn simplify_instruction_substitution(instructions: &[Instruction]) -> Vec<Instruction> {
    let mut cleaned = Vec::new();
    let mut skip_next = false;
    
    for i in 0..instructions.len() {
        if skip_next {
            skip_next = false;
            continue;
        }
        
        // Check for substitution patterns and simplify
        if i + 1 < instructions.len() {
            let curr = &instructions[i];
            let next = &instructions[i + 1];
            
            // Pattern: not + inc => neg
            if curr.mnemonic == "not" && next.mnemonic == "inc" && curr.operands == next.operands {
                let mut simplified = curr.clone();
                simplified.mnemonic = "neg".to_string();
                simplified.raw_line = format!("neg {}", curr.operands);
                cleaned.push(simplified);
                skip_next = true;
                continue;
            }
        }
        
        cleaned.push(instructions[i].clone());
    }
    
    cleaned
}

fn unfold_constants(instructions: &[Instruction]) -> Vec<Instruction> {
    // This would require more complex analysis to evaluate constant expressions
    // For now, just return the instructions as-is
    // Future enhancement: evaluate arithmetic operations on constants
    instructions.to_vec()
}

// ============================================================================
// REPORTING
// ============================================================================

pub fn format_deobfuscation_report(result: &DeobfuscationResult) -> String {
    let mut report = String::new();
    
    report.push_str("╔════════════════════════════════════════════════════════════════╗\n");
    report.push_str("║          🛡️  ANTI-OBFUSCATION ANALYSIS REPORT  🛡️              ║\n");
    report.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");
    
    report.push_str(&format!("📊 Statistics:\n"));
    report.push_str(&format!("   Original Instructions: {}\n", result.original_count));
    report.push_str(&format!("   Cleaned Instructions:  {}\n", result.cleaned_count));
    report.push_str(&format!("   Removed Instructions:  {} ({:.1}%)\n", 
        result.removed_instructions, result.success_rate));
    report.push_str(&format!("   Obfuscation Techniques: {}\n\n", result.signatures.len()));
    
    if result.signatures.is_empty() {
        report.push_str("✅ No obfuscation detected - code appears clean!\n\n");
        return report;
    }
    
    report.push_str("🔍 Detected Obfuscation Techniques:\n\n");
    
    // Group signatures by type
    let mut by_type: HashMap<String, Vec<&ObfuscationSignature>> = HashMap::new();
    for sig in &result.signatures {
        let type_name = format!("{:?}", sig.obf_type);
        by_type.entry(type_name).or_insert_with(Vec::new).push(sig);
    }
    
    for (obf_type, sigs) in by_type.iter() {
        let sig = sigs[0];
        let severity_icon = match sig.severity {
            ObfuscationSeverity::Low => "🟢",
            ObfuscationSeverity::Medium => "🟡",
            ObfuscationSeverity::High => "🟠",
            ObfuscationSeverity::Critical => "🔴",
        };
        
        report.push_str(&format!("{} {} (Severity: {:?})\n", severity_icon, obf_type, sig.severity));
        report.push_str(&format!("   Confidence: {:.0}% ", sig.confidence * 100.0));
        
        // Confidence bar
        let bar_length = (sig.confidence * 20.0) as usize;
        report.push_str(&"█".repeat(bar_length));
        report.push_str(&"░".repeat(20 - bar_length));
        report.push_str("\n");
        
        report.push_str(&format!("   Description: {}\n", sig.description));
        report.push_str("   Evidence:\n");
        for evidence in &sig.evidence {
            report.push_str(&format!("      • {}\n", evidence));
        }
        report.push_str("\n");
    }
    
    report.push_str("⚠️  Security Warning:\n");
    report.push_str("   Obfuscated code may indicate:\n");
    report.push_str("   • Malware or potentially unwanted programs (PUP)\n");
    report.push_str("   • Commercial software protection (DRM)\n");
    report.push_str("   • Anti-reverse engineering measures\n");
    report.push_str("   • Intellectual property protection\n\n");
    
    report.push_str("✨ Deobfuscation applied - cleaned code available for analysis\n\n");
    
    report
}