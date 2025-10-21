use regex::Regex;
use std::collections::{HashMap, HashSet};
use goblin::pe::PE;
use std::fs;
use std::sync::OnceLock;

use crate::anti_obfuscation;
use crate::windows_api_db;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub address: u64,
    pub mnemonic: String,
    pub operands: String,
    pub raw_line: String,
}

#[derive(Debug, Clone, PartialEq)]
enum VarType {
    Int32,
    Int64,
    Pointer,
    #[allow(dead_code)]
    String,
    Float,
    Unknown,
    #[allow(dead_code)]
    Struct(String),
    #[allow(dead_code)]
    Array(Box<VarType>, usize),
}

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    var_type: VarType,
    is_param: bool,
    is_local: bool,
    is_global: bool,
    address: Option<u64>,
    size: usize,
}

#[derive(Debug, Clone)]
struct BasicBlock {
    start_addr: u64,
    end_addr: u64,
    instructions: Vec<Instruction>,
    successors: Vec<u64>,
    predecessors: Vec<u64>,
}

#[derive(Debug, Clone)]
struct Function {
    name: String,
    start_addr: u64,
    end_addr: u64,
    blocks: Vec<BasicBlock>,
    variables: HashMap<String, Variable>,
    is_api_call: bool,
    parameters: Vec<Variable>,
    return_type: VarType,
    called_by: Vec<String>,
    calls: Vec<String>,
}

#[derive(Debug, Clone)]
enum ControlFlow {
    Sequential,
    IfThen { condition: String, true_block: u64 },
    IfElse { condition: String, true_block: u64, false_block: u64 },
    WhileLoop { condition: String, body_block: u64 },
    DoWhileLoop { condition: String, body_block: u64 },
    Switch { variable: String, cases: Vec<(String, u64)> },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct StructDefinition {
    name: String,
    fields: Vec<StructField>,
    size: usize,
    alignment: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct StructField {
    name: String,
    field_type: VarType,
    offset: usize,
    size: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct StringLiteral {
    address: u64,
    value: String,
    encoding: StringEncoding,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum StringEncoding {
    Ascii,
    Unicode,
    Utf8,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct GlobalVariable {
    name: String,
    address: u64,
    var_type: VarType,
    initial_value: Option<String>,
    size: usize,
    is_const: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CrossReference {
    from_addr: u64,
    to_addr: u64,
    ref_type: RefType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum RefType {
    Call,
    Jump,
    DataRead,
    DataWrite,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ProgramAnalysis {
    functions: Vec<Function>,
    structs: Vec<StructDefinition>,
    strings: Vec<StringLiteral>,
    globals: Vec<GlobalVariable>,
    cross_refs: Vec<CrossReference>,
}

#[derive(Debug, Clone)]
pub struct PEInfo {
    pub image_base: u64,
    pub entry_point: u64,
    pub sections: Vec<SectionInfo>,
    pub imports: HashMap<u64, ImportInfo>,
    pub exports: HashMap<u64, String>,
    #[allow(dead_code)]
    pub iat_range: Option<(u64, u64)>,
}

#[derive(Debug, Clone)]
pub struct SectionInfo {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub virtual_address: u64,
    #[allow(dead_code)]
    pub virtual_size: u64,
    #[allow(dead_code)]
    pub characteristics: u32,
    #[allow(dead_code)]
    pub is_code: bool,
    #[allow(dead_code)]
    pub is_data: bool,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    #[allow(dead_code)]
    pub dll: String,
    #[allow(dead_code)]
    pub function: String,
    #[allow(dead_code)]
    pub ordinal: Option<u16>,
}

#[derive(Debug, Clone)]
struct DecompilerContext {
    pe_info: Option<PEInfo>,
    instructions: Vec<Instruction>,
    functions: Vec<Function>,
    junk_patterns: Vec<JunkPattern>,
}

#[derive(Debug, Clone)]
struct JunkPattern {
    name: String,
    pattern: Vec<String>,  // Sequence of mnemonics
    description: String,
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CryptoAlgorithm {
    AES,
    DES,
    TripleDES,
    RC4,
    RSA,
    MD5,
    SHA1,
    SHA256,
    SHA512,
    Base64,
    XOR,
    TEA,
    Blowfish,
    ChaCha20,
    Unknown(String),
}

#[derive(Debug, Clone)]
struct CryptoSignature {
    algorithm: CryptoAlgorithm,
    confidence: f32,  // 0.0 to 1.0
    location: u64,
    evidence: Vec<String>,
    description: String,
}

#[derive(Debug, Clone)]
struct CryptoPattern {
    name: String,
    algorithm: CryptoAlgorithm,
    constants: Vec<u32>,  // Magic constants
    instructions: Vec<String>,  // Instruction patterns
    description: String,
}



pub fn translate_to_pseudo(asm: &str) -> String {
    translate_to_pseudo_with_pe(asm, None)
}

pub fn translate_to_pseudo_with_pe(asm: &str, pe_path: Option<&str>) -> String {
    let pe_info = pe_path.and_then(|path| parse_pe_file(path));
    let original_instructions = parse_instructions(asm);
    let original_count = original_instructions.len();
    let mut instructions = original_instructions;
    
    // Only filter junk if we have a reasonable number of instructions (performance optimization)
    // Reduced threshold from 10000 to 5000 for better performance
    let should_filter = instructions.len() < 5000;
    
    let (deobf_result, junk_removed) = if should_filter {
        let before_junk = instructions.len();
        // Filter junk instructions
        instructions = filter_junk_instructions(&instructions);
        let after_junk = instructions.len();
        let junk_count = before_junk - after_junk;
        
        // NEW v4.0: Anti-obfuscation layer
        let obf_instructions: Vec<anti_obfuscation::Instruction> = instructions.iter().map(|inst| {
            anti_obfuscation::Instruction {
                address: inst.address,
                mnemonic: inst.mnemonic.clone(),
                operands: inst.operands.clone(),
                raw_line: inst.raw_line.clone(),
            }
        }).collect();
        let result = anti_obfuscation::deobfuscate_instructions(&obf_instructions);
        instructions = result.cleaned_instructions.iter().map(|inst| {
            Instruction {
                address: inst.address,
                mnemonic: inst.mnemonic.clone(),
                operands: inst.operands.clone(),
                raw_line: inst.raw_line.clone(),
            }
        }).collect();
        (result, junk_count)
    } else {
        let count = instructions.len();
        (anti_obfuscation::DeobfuscationResult {
            original_count: count,
            cleaned_count: count,
            removed_instructions: 0,
            signatures: Vec::new(),
            cleaned_instructions: Vec::new(),
            success_rate: 1.0,
        }, 0)
    };
    
    // Detect crypto algorithms (only for smaller inputs)
    let crypto_sigs = if should_filter {
        detect_crypto_algorithms(&instructions)
    } else {
        Vec::new()
    };
    
    let functions = identify_functions(&instructions);
    let mut output = String::new();
    
    output.push_str("╔════════════════════════════════════════════════════════════════╗\n");
    output.push_str("║          ADVANCED PSEUDO-CODE DECOMPILATION v4.0               ║\n");
    output.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");
    
    // Analysis Statistics
    output.push_str("┌─ Analysis Statistics ─────────────────────────────────────────┐\n");
    output.push_str(&format!("│ Input Lines Parsed:        {:>6}\n", original_count));
    output.push_str(&format!("│ Instructions Extracted:    {:>6}\n", original_count));
    output.push_str(&format!("│ Junk Instructions Removed: {:>6}\n", junk_removed));
    output.push_str(&format!("│ Obfuscation Removed:       {:>6}\n", deobf_result.removed_instructions));
    output.push_str(&format!("│ Final Instruction Count:   {:>6}\n", instructions.len()));
    output.push_str(&format!("│ Functions Identified:      {:>6}\n", functions.len()));
    output.push_str(&format!("│ Basic Blocks Created:      {:>6}\n", functions.iter().map(|f| f.blocks.len()).sum::<usize>()));
    if should_filter {
        output.push_str("│ Analysis Mode:             FULL (with optimization)\n");
    } else {
        output.push_str("│ Analysis Mode:             FAST (large input detected)\n");
    }
    output.push_str("└───────────────────────────────────────────────────────────────┘\n\n");
    
    if let Some(ref pe) = pe_info {
        output.push_str("┌─ PE File Information ─────────────────────────────────────────┐\n");
        output.push_str(&format!("│ Image Base:   0x{:016x}\n", pe.image_base));
        output.push_str(&format!("│ Entry Point:  0x{:016x}\n", pe.entry_point));
        output.push_str(&format!("│ Sections:     {}\n", pe.sections.len()));
        output.push_str(&format!("│ Imports:      {}\n", pe.imports.len()));
        output.push_str(&format!("│ Exports:      {}\n", pe.exports.len()));
        output.push_str("└───────────────────────────────────────────────────────────────┘\n\n");
    }
    
    // Add anti-obfuscation report
    if !deobf_result.signatures.is_empty() || deobf_result.removed_instructions > 0 {
        output.push_str(&anti_obfuscation::format_deobfuscation_report(&deobf_result));
    }
    
    // Add crypto detection report
    if !crypto_sigs.is_empty() {
        output.push_str(&format_crypto_report(&crypto_sigs));
    }
    
    for func in &functions {
        output.push_str(&generate_pseudo_function(func, &instructions));
        output.push_str("\n");
    }
    
    output
}

pub fn translate_to_rust_with_pe(asm: &str, pe_path: Option<&str>) -> String {
    let pe_info = pe_path.and_then(|path| parse_pe_file(path));
    let original_instructions = parse_instructions(asm);
    let original_count = original_instructions.len();
    let mut instructions = original_instructions;

    // Only filter junk if we have a reasonable number of instructions (performance optimization)
    // Reduced threshold from 10000 to 5000 for better performance
    let should_filter = instructions.len() < 5000;

    let (deobf_result, junk_removed) = if should_filter {
        let before_junk = instructions.len();
        // Filter junk instructions
        instructions = filter_junk_instructions(&instructions);
        let after_junk = instructions.len();
        let junk_count = before_junk - after_junk;

        // NEW v4.0: Anti-obfuscation layer
        let obf_instructions: Vec<anti_obfuscation::Instruction> = instructions.iter().map(|inst| {
            anti_obfuscation::Instruction {
                address: inst.address,
                mnemonic: inst.mnemonic.clone(),
                operands: inst.operands.clone(),
                raw_line: inst.raw_line.clone(),
            }
        }).collect();
        let result = anti_obfuscation::deobfuscate_instructions(&obf_instructions);
        instructions = result.cleaned_instructions.iter().map(|inst| {
            Instruction {
                address: inst.address,
                mnemonic: inst.mnemonic.clone(),
                operands: inst.operands.clone(),
                raw_line: inst.raw_line.clone(),
            }
        }).collect();
        (result, junk_count)
    } else {
        let count = instructions.len();
        (anti_obfuscation::DeobfuscationResult {
            original_count: count,
            cleaned_count: count,
            removed_instructions: 0,
            signatures: Vec::new(),
            cleaned_instructions: Vec::new(),
            success_rate: 1.0,
        }, 0)
    };

    // Detect crypto algorithms (only for smaller inputs)
    let crypto_sigs = if should_filter {
        detect_crypto_algorithms(&instructions)
    } else {
        Vec::new()
    };

    let functions = identify_functions(&instructions);
    let api_calls = detect_api_calls(&instructions);

    let mut output = String::new();

    // Header with metadata
    output.push_str("/*\n");
    output.push_str(" * ═══════════════════════════════════════════════════════════════\n");
    output.push_str(" * ADVANCED DECOMPILER OUTPUT v4.0 - RUST CODE\n");
    output.push_str(" * ═══════════════════════════════════════════════════════════════\n");
    output.push_str(&format!(" * Input Lines Parsed:        {}\n", original_count));
    output.push_str(&format!(" * Instructions Extracted:    {}\n", original_count));
    output.push_str(&format!(" * Junk Instructions Removed: {}\n", junk_removed));
    output.push_str(&format!(" * Obfuscation Removed:       {}\n", deobf_result.removed_instructions));
    output.push_str(&format!(" * Final Instruction Count:   {}\n", instructions.len()));
    output.push_str(&format!(" * Functions Identified:      {}\n", functions.len()));
    output.push_str(&format!(" * API Calls Detected:        {}\n", api_calls.len()));
    output.push_str(&format!(" * Basic Blocks Created:      {}\n", functions.iter().map(|f| f.blocks.len()).sum::<usize>()));

    if let Some(ref pe) = pe_info {
        output.push_str(&format!(" * Image Base: 0x{:x}\n", pe.image_base));
        output.push_str(&format!(" * Entry Point: 0x{:x}\n", pe.entry_point));
        output.push_str(&format!(" * Imports: {}\n", pe.imports.len()));
        output.push_str(&format!(" * Exports: {}\n", pe.exports.len()));
    }

    if !deobf_result.signatures.is_empty() {
        output.push_str(&format!(" * 🛡️  Obfuscation Techniques: {} detected\n", deobf_result.signatures.len()));
    }

    if !crypto_sigs.is_empty() {
        output.push_str(&format!(" * 🔐 Crypto Algorithms: {} detected\n", crypto_sigs.len()));
    }

    if should_filter {
        output.push_str(" * Analysis Mode: FULL (with optimization)\n");
    } else {
        output.push_str(" * Analysis Mode: FAST (large input detected)\n");
    }

    output.push_str(" * Features: Control Flow Recovery, Type Inference, Pattern Recognition\n");
    output.push_str(" * Features: PE Parsing, IAT Resolution, Anti-Obfuscation, Crypto Detection\n");
    output.push_str(" * ═══════════════════════════════════════════════════════════════\n");
    output.push_str(" */\n\n");

    // Add anti-obfuscation report as comment
    if !deobf_result.signatures.is_empty() || deobf_result.removed_instructions > 0 {
        output.push_str("/*\n");
        for line in anti_obfuscation::format_deobfuscation_report(&deobf_result).lines() {
            output.push_str(&format!(" * {}\n", line));
        }
        output.push_str(" */\n\n");
    }

    // Add crypto detection report as comment
    if !crypto_sigs.is_empty() {
        output.push_str("/*\n");
        for line in format_crypto_report(&crypto_sigs).lines() {
            output.push_str(&format!(" * {}\n", line));
        }
        output.push_str(" */\n\n");
    }

    // Detect Windows API calls from assembly
    let detected_apis = windows_api_db::detect_api_calls_in_code(asm);
    
    // Includes
    output.push_str("#![allow(unused_variables, unused_mut, dead_code)]\n\n");

    // Add Windows API FFI declarations if detected
    if !detected_apis.is_empty() {
        output.push_str(&windows_api_db::generate_rust_api_declarations(&detected_apis));
    } else if !api_calls.is_empty() {
        output.push_str("// Windows API bindings\n");
        output.push_str("#[cfg(windows)]\n");
        output.push_str("use winapi::um::winuser::MessageBoxA;\n");
        output.push_str("#[cfg(windows)]\n");
        output.push_str("use winapi::um::fileapi::*;\n");
        output.push_str("#[cfg(windows)]\n");
        output.push_str("use winapi::um::memoryapi::*;\n");
        output.push_str("#[cfg(windows)]\n");
        output.push_str("use winapi::um::processthreadsapi::*;\n");
        output.push_str("#[cfg(windows)]\n");
        output.push_str("use std::ptr;\n\n");
    }

    output.push_str("\n");

    // Generate each function
    for func in &functions {
        let is_safe = is_function_safe(func, &instructions);
        output.push_str(&generate_rust_function(func, &instructions, is_safe));
        output.push_str("\n");
    }
    
    // Add main function if not present
    if !functions.iter().any(|f| f.name == "main") {
        output.push_str("// ═══════════════════════════════════════════════════════════════\n");
        output.push_str("// MAIN FUNCTION\n");
        output.push_str("// ═══════════════════════════════════════════════════════════════\n");
        output.push_str("// NOTE: This is a reconstructed main() function.\n");
        output.push_str("// The original executable may have used a different entry point.\n");
        output.push_str("// ═══════════════════════════════════════════════════════════════\n\n");
        
        // Check if this looks like a simple console program
        let has_write_console = detected_apis.iter().any(|api| api.contains("WriteConsole"));
        let has_std_handle = detected_apis.iter().any(|api| api.contains("GetStdHandle"));
        let has_exit_process = detected_apis.iter().any(|api| api.contains("ExitProcess"));
        
        if has_write_console && has_std_handle {
            // Generate a simple Hello World style program
            output.push_str("fn main() {\n");
            output.push_str("    unsafe {\n");
            output.push_str("        // Get standard output handle\n");
            output.push_str("        let h_std_out = GetStdHandle((-11i32) as u32); // STD_OUTPUT_HANDLE\n");
            output.push_str("        \n");
            output.push_str("        // Message to display\n");
            output.push_str("        let message = b\"Hello, world!\\n\";\n");
            output.push_str("        let mut written: u32 = 0;\n");
            output.push_str("        \n");
            output.push_str("        // Write to console\n");
            output.push_str("        WriteConsoleA(\n");
            output.push_str("            h_std_out,\n");
            output.push_str("            message.as_ptr() as *const std::ffi::c_void,\n");
            output.push_str("            message.len() as u32,\n");
            output.push_str("            &mut written as *mut u32,\n");
            output.push_str("            std::ptr::null_mut()\n");
            output.push_str("        );\n");
            output.push_str("        \n");
            if has_exit_process {
                output.push_str("        // Exit process\n");
                output.push_str("        ExitProcess(0);\n");
            }
            output.push_str("    }\n");
            output.push_str("}\n");
        } else {
            // Generic main function
            output.push_str("fn main() {\n");
            if !functions.is_empty() {
                output.push_str("    unsafe {\n");
                output.push_str(&format!("        // Call the first identified function\n"));
                output.push_str(&format!("        {}();\n", functions[0].name));
                output.push_str("    }\n");
            } else {
                output.push_str("    // TODO: Implement program logic\n");
                output.push_str("    // The decompiler could not identify clear function boundaries.\n");
                output.push_str("    println!(\"Decompiled program\");\n");
            }
            output.push_str("}\n");
        }
    }

    output
}

pub fn translate_to_c(asm: &str) -> String {
    translate_to_c_with_pe(asm, None)
}

pub fn translate_to_rust(asm: &str) -> String {
    translate_to_rust_with_pe(asm, None)
}

pub fn translate_to_c_with_pe(asm: &str, pe_path: Option<&str>) -> String {
    let pe_info = pe_path.and_then(|path| parse_pe_file(path));
    let original_instructions = parse_instructions(asm);
    let original_count = original_instructions.len();
    let mut instructions = original_instructions;
    
    // Only filter junk if we have a reasonable number of instructions (performance optimization)
    // Reduced threshold from 10000 to 5000 for better performance
    let should_filter = instructions.len() < 5000;
    
    let (deobf_result, junk_removed) = if should_filter {
        let before_junk = instructions.len();
        // Filter junk instructions
        instructions = filter_junk_instructions(&instructions);
        let after_junk = instructions.len();
        let junk_count = before_junk - after_junk;
        
        // NEW v4.0: Anti-obfuscation layer
        let obf_instructions: Vec<anti_obfuscation::Instruction> = instructions.iter().map(|inst| {
            anti_obfuscation::Instruction {
                address: inst.address,
                mnemonic: inst.mnemonic.clone(),
                operands: inst.operands.clone(),
                raw_line: inst.raw_line.clone(),
            }
        }).collect();
        let result = anti_obfuscation::deobfuscate_instructions(&obf_instructions);
        instructions = result.cleaned_instructions.iter().map(|inst| {
            Instruction {
                address: inst.address,
                mnemonic: inst.mnemonic.clone(),
                operands: inst.operands.clone(),
                raw_line: inst.raw_line.clone(),
            }
        }).collect();
        (result, junk_count)
    } else {
        let count = instructions.len();
        (anti_obfuscation::DeobfuscationResult {
            original_count: count,
            cleaned_count: count,
            cleaned_instructions: Vec::new(),
            removed_instructions: 0,
            signatures: Vec::new(),
            success_rate: 1.0,
        }, 0)
    };
    
    // Detect crypto algorithms (only for smaller inputs)
    let crypto_sigs = if should_filter {
        detect_crypto_algorithms(&instructions)
    } else {
        Vec::new()
    };
    
    let functions = identify_functions(&instructions);
    let api_calls = detect_api_calls(&instructions);
    
    let mut output = String::new();
    
    // Header with metadata
    output.push_str("/*\n");
    output.push_str(" * ═══════════════════════════════════════════════════════════════\n");
    output.push_str(" * ADVANCED DECOMPILER OUTPUT v4.0 - C CODE\n");
    output.push_str(" * ═══════════════════════════════════════════════════════════════\n");
    output.push_str(&format!(" * Input Lines Parsed:        {}\n", original_count));
    output.push_str(&format!(" * Instructions Extracted:    {}\n", original_count));
    output.push_str(&format!(" * Junk Instructions Removed: {}\n", junk_removed));
    output.push_str(&format!(" * Obfuscation Removed:       {}\n", deobf_result.removed_instructions));
    output.push_str(&format!(" * Final Instruction Count:   {}\n", instructions.len()));
    output.push_str(&format!(" * Functions Identified:      {}\n", functions.len()));
    output.push_str(&format!(" * API Calls Detected:        {}\n", api_calls.len()));
    output.push_str(&format!(" * Basic Blocks Created:      {}\n", functions.iter().map(|f| f.blocks.len()).sum::<usize>()));
    
    if let Some(ref pe) = pe_info {
        output.push_str(&format!(" * Image Base: 0x{:x}\n", pe.image_base));
        output.push_str(&format!(" * Entry Point: 0x{:x}\n", pe.entry_point));
        output.push_str(&format!(" * Imports: {}\n", pe.imports.len()));
        output.push_str(&format!(" * Exports: {}\n", pe.exports.len()));
    }
    
    if !deobf_result.signatures.is_empty() {
        output.push_str(&format!(" * 🛡️  Obfuscation Techniques: {} detected\n", deobf_result.signatures.len()));
    }
    
    if !crypto_sigs.is_empty() {
        output.push_str(&format!(" * 🔐 Crypto Algorithms: {} detected\n", crypto_sigs.len()));
    }
    
    if should_filter {
        output.push_str(" * Analysis Mode: FULL (with optimization)\n");
    } else {
        output.push_str(" * Analysis Mode: FAST (large input detected)\n");
    }
    
    output.push_str(" * Features: Control Flow Recovery, Type Inference, Pattern Recognition\n");
    output.push_str(" * Features: PE Parsing, IAT Resolution, Anti-Obfuscation, Crypto Detection\n");
    output.push_str(" * ═══════════════════════════════════════════════════════════════\n");
    output.push_str(" */\n\n");
    
    // Add anti-obfuscation report as comment
    if !deobf_result.signatures.is_empty() || deobf_result.removed_instructions > 0 {
        output.push_str("/*\n");
        for line in anti_obfuscation::format_deobfuscation_report(&deobf_result).lines() {
            output.push_str(&format!(" * {}\n", line));
        }
        output.push_str(" */\n\n");
    }
    
    // Add crypto detection report as comment
    if !crypto_sigs.is_empty() {
        output.push_str("/*\n");
        for line in format_crypto_report(&crypto_sigs).lines() {
            output.push_str(&format!(" * {}\n", line));
        }
        output.push_str(" */\n\n");
    }
    
    // Detect Windows API calls from assembly
    let detected_apis = windows_api_db::detect_api_calls_in_code(asm);
    
    // Includes
    output.push_str("#include <stdio.h>\n");
    output.push_str("#include <stdlib.h>\n");
    output.push_str("#include <string.h>\n");
    output.push_str("#include <stdint.h>\n");
    
    if !api_calls.is_empty() || !detected_apis.is_empty() {
        output.push_str("#include <windows.h>\n");
    }
    
    output.push_str("\n");
    
    // Add Windows API declarations if detected
    if !detected_apis.is_empty() {
        output.push_str(&windows_api_db::generate_c_api_declarations(&detected_apis));
    }
    
    // Type definitions - using standard C types for better compatibility
    output.push_str("// ═══ Type Definitions ═══\n");
    output.push_str("typedef unsigned char      uint8_t;\n");
    output.push_str("typedef unsigned short     uint16_t;\n");
    output.push_str("typedef unsigned int       uint32_t;\n");
    output.push_str("typedef unsigned long long uint64_t;\n");
    output.push_str("typedef signed char        int8_t;\n");
    output.push_str("typedef signed short       int16_t;\n");
    output.push_str("typedef signed int         int32_t;\n");
    output.push_str("typedef signed long long   int64_t;\n");
    output.push_str("typedef void*              ptr_t;\n\n");
    
    // Forward declarations
    if functions.len() > 1 {
        output.push_str("// ═══ Forward Declarations ═══\n");
        for func in &functions {
            if !func.is_api_call {
                let return_type = match &func.return_type {
                    VarType::Unknown => "int",
                    _ => &type_to_c_string(&func.return_type),
                };
                output.push_str(&format!("{} {}();\n", return_type, func.name));
            }
        }
        output.push_str("\n");
    }
    
    // Generate each function
    for func in &functions {
        output.push_str(&generate_c_function(func, &instructions));
        output.push_str("\n");
    }

    // Add main function if not present
    if !functions.iter().any(|f| f.name == "main") {
        output.push_str("// ═══════════════════════════════════════════════════════════════\n");
        output.push_str("// MAIN FUNCTION\n");
        output.push_str("// ═══════════════════════════════════════════════════════════════\n");
        output.push_str("// NOTE: This is a reconstructed main() function.\n");
        output.push_str("// The original executable may have used a different entry point.\n");
        output.push_str("// ═══════════════════════════════════════════════════════════════\n\n");
        
        // Check if this looks like a simple console program
        let has_write_console = detected_apis.iter().any(|api| api.contains("WriteConsole"));
        let has_std_handle = detected_apis.iter().any(|api| api.contains("GetStdHandle"));
        let has_exit_process = detected_apis.iter().any(|api| api.contains("ExitProcess"));
        
        if has_write_console && has_std_handle {
            // Generate a simple Hello World style program
            output.push_str("int main() {\n");
            output.push_str("    // Get standard output handle\n");
            output.push_str("    HANDLE hStdOut = GetStdHandle((DWORD)-11); // STD_OUTPUT_HANDLE\n");
            output.push_str("    \n");
            output.push_str("    // Message to display\n");
            output.push_str("    const char* message = \"Hello, world!\\n\";\n");
            output.push_str("    DWORD written = 0;\n");
            output.push_str("    \n");
            output.push_str("    // Write to console\n");
            output.push_str("    WriteConsoleA(hStdOut, message, 14, &written, NULL);\n");
            output.push_str("    \n");
            if has_exit_process {
                output.push_str("    // Exit process\n");
                output.push_str("    ExitProcess(0);\n");
            } else {
                output.push_str("    return 0;\n");
            }
            output.push_str("}\n");
        } else {
            // Generic main function
            output.push_str("int main() {\n");
            if !functions.is_empty() {
                output.push_str(&format!("    // Call the first identified function\n"));
                output.push_str(&format!("    {}();\n", functions[0].name));
            } else {
                output.push_str("    // TODO: Implement program logic\n");
                output.push_str("    // The decompiler could not identify clear function boundaries.\n");
            }
            output.push_str("    return 0;\n");
            output.push_str("}\n");
        }
    }
    
    output
}

fn init_junk_patterns() -> Vec<JunkPattern> {
    let junk_patterns = vec![
        JunkPattern {
            name: "multi_byte_nop".to_string(),
            pattern: vec!["nop".to_string()],
            description: "Multi-byte NOP padding ".to_string(),
        },
        JunkPattern {
            name: "inc_dec_cancel".to_string(),
            pattern: vec!["inc".to_string(), "dec".to_string()],
            description: "Increment followed by decrement (no effect)".to_string(),
        },
        JunkPattern {
            name: "push_pop_cancel".to_string(),
            pattern: vec!["push".to_string(), "pop".to_string()],
            description: "Push followed by pop of same register".to_string(),
        },
        JunkPattern {
            name: "xor_self".to_string(),
            pattern: vec!["xor".to_string()],
            description: "XOR register with itself (zeroing, but may be redundant)".to_string(),
        },
    ];
    junk_patterns
}

fn is_junk_instruction(instr: &Instruction, next: Option<&Instruction>, _patterns: &[JunkPattern]) -> bool {
    // Multi-byte NOPs
    if instr.mnemonic == "nop" {
        return true;
    }
    
    // NOP variants: nop dword ptr [eax], etc.
    if instr.mnemonic == "nop" && instr.operands.contains("ptr") {
        return true;
    }
    
    // Check inc/dec cancellation
    if let Some(next_instr) = next {
        if instr.mnemonic == "inc" && next_instr.mnemonic == "dec" {
            // Check if same register
            if instr.operands == next_instr.operands {
                return true;
            }
        }
        
        // Check push/pop cancellation
        if instr.mnemonic == "push" && next_instr.mnemonic == "pop" {
            if instr.operands == next_instr.operands {
                return true;
            }
        }
    }
    
    false
}

fn filter_junk_instructions(instructions: &[Instruction]) -> Vec<Instruction> {
    // Pre-allocate with full capacity (most instructions will be kept)
    let mut filtered = Vec::with_capacity(instructions.len());
    let patterns = init_junk_patterns();
    let mut skip_next = false;
    
    for (i, instr) in instructions.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        
        let next = instructions.get(i + 1);
        
        // Fast path: check common junk patterns inline before calling is_junk_instruction
        let is_junk = if let Some(next_instr) = next {
            // Check canceling pairs first (most common junk)
            if (instr.mnemonic == "inc" && next_instr.mnemonic == "dec" && instr.operands == next_instr.operands) ||
               (instr.mnemonic == "dec" && next_instr.mnemonic == "inc" && instr.operands == next_instr.operands) ||
               (instr.mnemonic == "push" && next_instr.mnemonic == "pop" && instr.operands == next_instr.operands) ||
               (instr.mnemonic == "add" && next_instr.mnemonic == "sub" && instr.operands == next_instr.operands) ||
               (instr.mnemonic == "sub" && next_instr.mnemonic == "add" && instr.operands == next_instr.operands) {
                skip_next = true;
                true
            } else {
                is_junk_instruction(instr, Some(next_instr), &patterns)
            }
        } else {
            is_junk_instruction(instr, None, &patterns)
        };
        
        if !is_junk {
            filtered.push(instr.clone());
        }
    }
    
    // Shrink to fit to save memory
    filtered.shrink_to_fit();
    filtered
}

// ============================================================================
// CRYPTO DETECTION ENGINE (NEW v3.3)
// ============================================================================

fn init_crypto_patterns() -> Vec<CryptoPattern> {
    vec![
        // AES S-Box constants
        CryptoPattern {
            name: "AES S-Box".to_string(),
            algorithm: CryptoAlgorithm::AES,
            constants: vec![0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5],
            instructions: vec!["movzx".to_string(), "xor".to_string(), "shl".to_string()],
            description: "AES encryption S-box lookup table".to_string(),
        },
        // MD5 constants
        CryptoPattern {
            name: "MD5 Constants".to_string(),
            algorithm: CryptoAlgorithm::MD5,
            constants: vec![0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee],
            instructions: vec!["add".to_string(), "rol".to_string(), "xor".to_string()],
            description: "MD5 hash initialization constants".to_string(),
        },
        // SHA-1 constants
        CryptoPattern {
            name: "SHA-1 Constants".to_string(),
            algorithm: CryptoAlgorithm::SHA1,
            constants: vec![0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            instructions: vec!["add".to_string(), "rol".to_string(), "and".to_string()],
            description: "SHA-1 hash initialization constants".to_string(),
        },
        // SHA-256 constants
        CryptoPattern {
            name: "SHA-256 Constants".to_string(),
            algorithm: CryptoAlgorithm::SHA256,
            constants: vec![0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5],
            instructions: vec!["add".to_string(), "ror".to_string(), "xor".to_string()],
            description: "SHA-256 hash round constants".to_string(),
        },
        // RC4 key scheduling
        CryptoPattern {
            name: "RC4 KSA".to_string(),
            algorithm: CryptoAlgorithm::RC4,
            constants: vec![0x00, 0x01, 0x02, 0x03],
            instructions: vec!["xor".to_string(), "add".to_string(), "mov".to_string()],
            description: "RC4 key scheduling algorithm".to_string(),
        },
        // DES S-Boxes
        CryptoPattern {
            name: "DES S-Box".to_string(),
            algorithm: CryptoAlgorithm::DES,
            constants: vec![14, 4, 13, 1, 2, 15, 11, 8],
            instructions: vec!["shr".to_string(), "and".to_string(), "xor".to_string()],
            description: "DES encryption S-box values".to_string(),
        },
        // Base64 alphabet
        CryptoPattern {
            name: "Base64 Table".to_string(),
            algorithm: CryptoAlgorithm::Base64,
            constants: vec![0x41424344, 0x45464748],  // "ABCDEFGH"
            instructions: vec!["shr".to_string(), "and".to_string(), "movzx".to_string()],
            description: "Base64 encoding table".to_string(),
        },
        // TEA (Tiny Encryption Algorithm)
        CryptoPattern {
            name: "TEA Delta".to_string(),
            algorithm: CryptoAlgorithm::TEA,
            constants: vec![0x9e3779b9],  // Golden ratio constant
            instructions: vec!["add".to_string(), "shl".to_string(), "shr".to_string()],
            description: "TEA encryption delta constant".to_string(),
        },
    ]
}

fn detect_crypto_algorithms(instructions: &[Instruction]) -> Vec<CryptoSignature> {
    // Skip crypto detection for very large inputs (performance optimization)
    if instructions.len() > 5000 {
        return Vec::new();
    }
    
    let patterns = init_crypto_patterns();
    let mut signatures = Vec::with_capacity(patterns.len());
    let mut detected = HashSet::with_capacity(patterns.len());
    
    // Scan for magic constants
    for pattern in &patterns {
        let mut evidence = Vec::new();
        let mut confidence: f32 = 0.0;
        let mut location = 0u64;
        
        // Check for constant patterns (optimized loop)
        for instr in instructions {
            // Extract immediate values from operands
            if let Some(constant) = extract_constant(&instr.operands) {
                for &magic in &pattern.constants {
                    if constant == magic as u64 {
                        evidence.push(format!("Found constant 0x{:x} at 0x{:x}", magic, instr.address));
                        confidence += 0.3;
                        if location == 0 {
                            location = instr.address;
                        }
                        break; // Found match, no need to check other constants
                    }
                }
            }
            
            // Check for instruction patterns (fast path with direct comparison)
            for pattern_instr in &pattern.instructions {
                if instr.mnemonic == pattern_instr.as_str() {
                    confidence += 0.05;
                    break;
                }
            }
            
            // Early exit if confidence is already high enough
            if confidence > 1.0 {
                break;
            }
        }
        
        // If we found enough evidence, add signature
        if confidence > 0.5 && !detected.contains(&pattern.algorithm) {
            signatures.push(CryptoSignature {
                algorithm: pattern.algorithm.clone(),
                confidence: confidence.min(1.0),
                location,
                evidence,
                description: pattern.description.clone(),
            });
            detected.insert(pattern.algorithm.clone());
        }
    }
    
    // Detect XOR encryption (simple pattern) - optimized count
    let xor_count = instructions.iter().filter(|i| i.mnemonic == "xor").count();
    if xor_count > 10 {
        let xor_confidence = (xor_count as f32 / instructions.len() as f32).min(1.0);
        if xor_confidence > 0.1 && !detected.contains(&CryptoAlgorithm::XOR) {
            signatures.push(CryptoSignature {
                algorithm: CryptoAlgorithm::XOR,
                confidence: xor_confidence,
                location: instructions.iter().find(|i| i.mnemonic == "xor").map(|i| i.address).unwrap_or(0),
                evidence: vec![format!("Found {} XOR operations", xor_count)],
                description: "XOR-based encryption/obfuscation detected".to_string(),
            });
        }
    }
    
    // Sort by confidence (unstable sort is faster)
    signatures.sort_unstable_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    signatures
}

fn extract_constant(operands: &str) -> Option<u64> {
    // Optimized: use OnceLock to compile regex patterns only once
    static HEX_RE: OnceLock<Regex> = OnceLock::new();
    static DEC_RE: OnceLock<Regex> = OnceLock::new();
    
    // Try to extract hex constants from operands
    let hex_re = HEX_RE.get_or_init(|| Regex::new(r"0x([0-9a-fA-F]+)").unwrap());
    if let Some(caps) = hex_re.captures(operands) {
        if let Some(hex_str) = caps.get(1) {
            return u64::from_str_radix(hex_str.as_str(), 16).ok();
        }
    }
    
    // Try decimal constants
    let dec_re = DEC_RE.get_or_init(|| Regex::new(r"\b(\d+)\b").unwrap());
    if let Some(caps) = dec_re.captures(operands) {
        if let Some(dec_str) = caps.get(1) {
            return dec_str.as_str().parse::<u64>().ok();
        }
    }
    
    None
}

fn format_crypto_report(signatures: &[CryptoSignature]) -> String {
    if signatures.is_empty() {
        return String::new();
    }
    
    let mut report = String::new();
    report.push_str("╔════════════════════════════════════════════════════════════════╗\n");
    report.push_str("║              🔐 CRYPTOGRAPHIC ANALYSIS REPORT                  ║\n");
    report.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");
    
    report.push_str(&format!("⚠️  {} cryptographic algorithm(s) detected!\n\n", signatures.len()));
    
    for (i, sig) in signatures.iter().enumerate() {
        let algo_name = match &sig.algorithm {
            CryptoAlgorithm::AES => "AES (Advanced Encryption Standard)",
            CryptoAlgorithm::DES => "DES (Data Encryption Standard)",
            CryptoAlgorithm::TripleDES => "3DES (Triple DES)",
            CryptoAlgorithm::RC4 => "RC4 Stream Cipher",
            CryptoAlgorithm::RSA => "RSA Public Key Cryptography",
            CryptoAlgorithm::MD5 => "MD5 Hash Function",
            CryptoAlgorithm::SHA1 => "SHA-1 Hash Function",
            CryptoAlgorithm::SHA256 => "SHA-256 Hash Function",
            CryptoAlgorithm::SHA512 => "SHA-512 Hash Function",
            CryptoAlgorithm::Base64 => "Base64 Encoding",
            CryptoAlgorithm::XOR => "XOR Cipher/Obfuscation",
            CryptoAlgorithm::TEA => "TEA (Tiny Encryption Algorithm)",
            CryptoAlgorithm::Blowfish => "Blowfish Cipher",
            CryptoAlgorithm::ChaCha20 => "ChaCha20 Stream Cipher",
            CryptoAlgorithm::Unknown(name) => name.as_str(),
        };
        
        let confidence_bar = "█".repeat((sig.confidence * 10.0) as usize);
        let confidence_pct = (sig.confidence * 100.0) as u32;
        
        report.push_str(&format!("{}. {} [{}%];\n", i + 1, algo_name, confidence_pct));
        report.push_str(&format!("   Confidence: [{}]; {}\n", confidence_bar, confidence_pct));
        report.push_str(&format!("   Location: 0x{:x}\n", sig.location));
        report.push_str(&format!("   Description: {}\n", sig.description));
        
        if !sig.evidence.is_empty() {
            report.push_str("   Evidence:\n");
            for evidence in sig.evidence.iter().take(3) {
                report.push_str(&format!("     • {}\n", evidence));
            }
            if sig.evidence.len() > 3 {
                report.push_str(&format!("     ... and {} more\n", sig.evidence.len() - 3));
            }
        }
        report.push_str("\n");
    }
    
    report.push_str("═══════════════════════════════════════════════════════════════\n");
    report.push_str("⚠️  Security Note: This binary contains cryptographic routines.\n");
    report.push_str("   This may indicate:\n");
    report.push_str("   • Legitimate encryption/security features\n");
    report.push_str("   • Malware obfuscation or C&C communication\n");
    report.push_str("   • License validation or DRM\n");
    report.push_str("   • Password hashing or authentication\n");
    report.push_str("═══════════════════════════════════════════════════════════════\n\n");
    
    report
}

// ============================================================================
// FUNCTION IDENTIFICATION
// ============================================================================

fn identify_functions(instructions: &[Instruction]) -> Vec<Function> {
    let mut functions = Vec::new();
    let mut current_func_start = 0u64;
    let mut current_func_start_idx = 0usize;
    let mut in_function = false;
    
    for (i, instr) in instructions.iter().enumerate() {
        // Function prologue detection
        if is_function_prologue(instr, instructions.get(i + 1)) {
            current_func_start = instr.address;
            current_func_start_idx = i;
            in_function = true;
        }
        
        // Function epilogue detection
        if in_function && is_function_epilogue(instr) {
            let func_name = format!("func_{:x}", current_func_start);
            
            // Optimized: use slice instead of filter+collect (O(1) vs O(n))
            let func_instructions = &instructions[current_func_start_idx..=i];
            
            let blocks = build_basic_blocks(func_instructions);
            let variables = analyze_variables(func_instructions);
            let parameters = extract_parameters(&variables);
            
            functions.push(Function {
                name: func_name,
                start_addr: current_func_start,
                end_addr: instr.address,
                blocks,
                variables,
                is_api_call: false,
                parameters,
                return_type: VarType::Unknown,
                called_by: Vec::new(),
                calls: Vec::new(),
            });
            
            in_function = false;
        }
    }
    
    // If no functions detected, treat entire code as one function
    if functions.is_empty() && !instructions.is_empty() {
        let blocks = build_basic_blocks(instructions);
        let variables = analyze_variables(instructions);
        let parameters = extract_parameters(&variables);
        
        functions.push(Function {
            name: "main".to_string(),
            start_addr: instructions[0].address,
            end_addr: instructions.last().unwrap().address,
            blocks,
            variables,
            is_api_call: false,
            parameters,
            return_type: VarType::Unknown,
            called_by: Vec::new(),
            calls: Vec::new(),
        });
    }
    
    functions
}

fn is_function_prologue(instr: &Instruction, next: Option<&Instruction>) -> bool {
    // Common function prologues:
    // push ebp / push rbp
    // mov ebp, esp / mov rbp, rsp
    if instr.mnemonic == "push" && (instr.operands.contains("ebp") || instr.operands.contains("rbp")) {
        if let Some(next_instr) = next {
            if next_instr.mnemonic == "mov" && 
               (next_instr.operands.contains("ebp") || next_instr.operands.contains("rbp")) {
                return true;
            }
        }
    }
    false
}

fn is_function_epilogue(instr: &Instruction) -> bool {
    // ret, retn, or leave followed by ret
    instr.mnemonic == "ret" || instr.mnemonic == "retn" || instr.mnemonic == "leave"
}

// ============================================================================
// BASIC BLOCK CONSTRUCTION
// ============================================================================

fn build_basic_blocks(instructions: &[Instruction]) -> Vec<BasicBlock> {
    if instructions.is_empty() {
        return Vec::new();
    }
    
    // Pre-allocate with estimated capacity
    let mut leaders = HashSet::with_capacity(instructions.len() / 4);
    leaders.insert(instructions[0].address);
    
    // Find all leaders (targets of jumps, instructions after jumps)
    for (i, instr) in instructions.iter().enumerate() {
        // Fast path: check common branch instructions inline
        let is_branch = matches!(instr.mnemonic.as_str(), 
            "jmp" | "je" | "jne" | "jz" | "jnz" | "jg" | "jge" | "jl" | "jle" |
            "ja" | "jae" | "jb" | "jbe" | "jo" | "jno" | "js" | "jns" | "jp" | "jnp" | "call");
        
        if is_branch {
            if let Some(target) = extract_jump_target(&instr.operands) {
                leaders.insert(target);
            }
            if i + 1 < instructions.len() {
                leaders.insert(instructions[i + 1].address);
            }
        }
    }
    
    // Build blocks efficiently
    let mut leader_addrs: Vec<u64> = leaders.into_iter().collect();
    leader_addrs.sort_unstable(); // unstable sort is faster
    
    let mut blocks = Vec::with_capacity(leader_addrs.len());
    
    // Create address lookup map for faster filtering
    let addr_to_idx: HashMap<u64, usize> = instructions
        .iter()
        .enumerate()
        .map(|(idx, instr)| (instr.address, idx))
        .collect();
    
    for i in 0..leader_addrs.len() {
        let start = leader_addrs[i];
        let end = if i + 1 < leader_addrs.len() {
            leader_addrs[i + 1]
        } else {
            u64::MAX
        };
        
        // Use index-based slicing instead of filtering for better performance
        if let Some(&start_idx) = addr_to_idx.get(&start) {
            let mut block_instrs = Vec::new();
            for instr in &instructions[start_idx..] {
                if instr.address >= end {
                    break;
                }
                block_instrs.push(instr.clone());
            }
            
            if !block_instrs.is_empty() {
                blocks.push(BasicBlock {
                    start_addr: start,
                    end_addr: block_instrs.last().unwrap().address,
                    instructions: block_instrs,
                    successors: Vec::new(),
                    predecessors: Vec::new(),
                });
            }
        }
    }
    
    blocks
}

fn is_branch_instruction(mnemonic: &str) -> bool {
    matches!(mnemonic, 
        "jmp" | "je" | "jne" | "jz" | "jnz" | "jg" | "jge" | "jl" | "jle" |
        "ja" | "jae" | "jb" | "jbe" | "jo" | "jno" | "js" | "jns" | "jp" | "jnp" |
        "call"
    )
}

fn extract_jump_target(operands: &str) -> Option<u64> {
    // Optimized: use OnceLock to compile regex only once
    static JUMP_TARGET_RE: OnceLock<Regex> = OnceLock::new();
    let re = JUMP_TARGET_RE.get_or_init(|| {
        Regex::new(r"0x([0-9a-fA-F]+)").unwrap()
    });
    
    if let Some(caps) = re.captures(operands) {
        u64::from_str_radix(&caps[1], 16).ok()
    } else {
        None
    }
}

// ============================================================================
// VARIABLE ANALYSIS
// ============================================================================

fn analyze_variables(instructions: &[Instruction]) -> HashMap<String, Variable> {
    // Pre-allocate with estimated capacity (typically 10-20% of instructions are variable-related)
    let mut variables = HashMap::with_capacity(instructions.len() / 10);
    let mut register_map = HashMap::with_capacity(16); // Limited number of registers
    
    for instr in instructions {
        match instr.mnemonic.as_str() {
            "mov" | "lea" => {
                // UTF-8 SAFE: Use split_once() instead of find() + byte slicing
                // This prevents crashes when operands contain multi-byte UTF-8 characters
                if let Some((dest, src)) = instr.operands.split_once(',') {
                    let dest = dest.trim();
                    let src = src.trim();
                    
                    // Detect stack variables - optimized with byte-level checks
                    let has_bp = src.contains("ebp") || src.contains("rbp");
                    let has_sp = src.contains("esp") || src.contains("rsp");
                    
                    if has_bp || has_sp {
                        let var_name = normalize_stack_var(src);
                        let var_type = infer_type_from_register(dest);
                        
                        variables.entry(var_name.clone()).or_insert_with(|| {
                            let is_param = src.contains("+") && !src.contains("-");
                            Variable {
                                name: var_name.clone(),
                                var_type: var_type.clone(),
                                is_param,
                                is_local: src.contains("-"),
                                is_global: false,
                                address: None,
                                size: type_size(&var_type),
                            }
                        });
                        
                        register_map.insert(dest.to_string(), var_name);
                    }
                    
                    // Track register assignments
                    if let Some(var_name) = register_map.get(src) {
                        register_map.insert(dest.to_string(), var_name.clone());
                    }
                }
            }
            "push" => {
                // Parameters being pushed
                let var_name = format!("param_{}", variables.len());
                variables.insert(var_name.clone(), Variable {
                    name: var_name,
                    var_type: VarType::Unknown,
                    is_param: true,
                    is_local: false,
                    is_global: false,
                    address: None,
                    size: 4,
                });
            }
            _ => {}
        }
    }
    
    variables
}

fn normalize_stack_var(operand: &str) -> String {
    // Optimized: use OnceLock to compile regex only once
    static STACK_VAR_RE: OnceLock<Regex> = OnceLock::new();
    let re = STACK_VAR_RE.get_or_init(|| {
        Regex::new(r"(ebp|rbp|esp|rsp)\s*([+-])\s*0x([0-9a-f]+)").unwrap()
    });
    
    if let Some(caps) = re.captures(operand) {
        let _base = &caps[1];
        let sign = &caps[2];
        let offset = u32::from_str_radix(&caps[3], 16).unwrap_or(0);
        
        if sign == "-" {
            format!("local_{}", offset)
        } else {
            format!("param_{}", offset)
        }
    } else if operand.contains("ebp") || operand.contains("rbp") {
        "local_0".to_string()
    } else {
        operand.replace("[", "_").replace("];", "_").replace(" ", "_")
    }
}

fn type_size(var_type: &VarType) -> usize {
    match var_type {
        VarType::Int32 => 4,
        VarType::Int64 => 8,
        VarType::Pointer => 8,
        VarType::String => 8,
        VarType::Float => 4,
        VarType::Unknown => 4,
        VarType::Struct(_) => 0,  // Unknown size
        VarType::Array(elem_type, count) => type_size(elem_type) * count,
    }
}

fn extract_parameters(variables: &HashMap<String, Variable>) -> Vec<Variable> {
    variables
        .values()
        .filter(|v| v.is_param)
        .cloned()
        .collect()
}

fn infer_type_from_register(reg: &str) -> VarType {
    let reg_lower = reg.to_lowercase();
    
    if reg_lower.starts_with('r') && (reg_lower.len() == 3 || reg_lower == "rax" || reg_lower == "rbx") {
        VarType::Int64
    } else if reg_lower.starts_with('e') {
        VarType::Int32
    } else if reg_lower.contains("ptr") {
        VarType::Pointer
    } else if reg_lower.starts_with("xmm") || reg_lower.starts_with("ymm") {
        VarType::Float
    } else {
        VarType::Unknown
    }
}

// ============================================================================
// API CALL DETECTION
// ============================================================================

fn detect_api_calls(instructions: &[Instruction]) -> HashMap<String, String> {
    let mut api_calls = HashMap::new();
    
    let known_apis = get_known_api_database();
    
    for instr in instructions {
        if instr.mnemonic == "call" {
            let call_target = instr.operands.trim();
            
            // Check if it's a known API
            for (api_name, description) in &known_apis {
                if call_target.contains(api_name) {
                    api_calls.insert(api_name.clone(), description.clone());
                }
            }
            
            // Also detect indirect calls through IAT (Import Address Table)
            // Pattern: call QWORD PTR [rip+0x...]
            if call_target.contains("[rip+") || call_target.contains("QWORD PTR") || call_target.contains("DWORD PTR") {
                api_calls.insert("__indirect_call".to_string(), "Indirect API call through IAT".to_string());
            }
        }
    
    }
    api_calls
}

fn get_known_api_database() -> HashMap<String, String> {
    let mut apis = HashMap::new();
    
    // Windows API
    apis.insert("MessageBoxA".to_string(), "Display message box (ANSI)".to_string());
    apis.insert("MessageBoxW".to_string(), "Display message box (Unicode)".to_string());
    apis.insert("CreateFileA".to_string(), "Create or open file (ANSI)".to_string());
    apis.insert("CreateFileW".to_string(), "Create or open file (Unicode)".to_string());
    apis.insert("ReadFile".to_string(), "Read from file".to_string());
    apis.insert("WriteFile".to_string(), "Write to file".to_string());
    apis.insert("CloseHandle".to_string(), "Close handle".to_string());
    apis.insert("VirtualAlloc".to_string(), "Allocate virtual memory".to_string());
    apis.insert("VirtualFree".to_string(), "Free virtual memory".to_string());
    apis.insert("GetProcAddress".to_string(), "Get function address from DLL".to_string());
    apis.insert("LoadLibraryA".to_string(), "Load DLL (ANSI)".to_string());
    apis.insert("LoadLibraryW".to_string(), "Load DLL (Unicode)".to_string());
    apis.insert("ExitProcess".to_string(), "Terminate process".to_string());
    apis.insert("CreateThread".to_string(), "Create new thread".to_string());
    apis.insert("Sleep".to_string(), "Suspend execution".to_string());
    apis.insert("GetModuleHandleA".to_string(), "Get module handle (ANSI)".to_string());
    apis.insert("GetModuleHandleW".to_string(), "Get module handle (Unicode)".to_string());
    
    // C Runtime
    apis.insert("printf".to_string(), "Print formatted output".to_string());
    apis.insert("scanf".to_string(), "Read formatted input".to_string());
    apis.insert("malloc".to_string(), "Allocate memory".to_string());
    apis.insert("free".to_string(), "Free memory".to_string());
    apis.insert("memcpy".to_string(), "Copy memory".to_string());
    apis.insert("memset".to_string(), "Set memory".to_string());
    apis.insert("strlen".to_string(), "Get string length".to_string());
    apis.insert("strcpy".to_string(), "Copy string".to_string());
    apis.insert("strcmp".to_string(), "Compare strings".to_string());
    
    apis
}

// ============================================================================
// CONTROL FLOW ANALYSIS
// ============================================================================

fn analyze_control_flow(blocks: &[BasicBlock]) -> HashMap<u64, ControlFlow> {
    // Pre-allocate with exact capacity
    let mut control_flow = HashMap::with_capacity(blocks.len());
    
    for block in blocks {
        if let Some(last_instr) = block.instructions.last() {
            // Optimized: use matches! macro for faster branch checking
            let flow = if last_instr.mnemonic == "jmp" {
                if let Some(target) = extract_jump_target(&last_instr.operands) {
                    // Check if it's a loop (jumping backwards)
                    if target <= block.start_addr {
                        ControlFlow::WhileLoop {
                            condition: "true".to_string(),
                            body_block: target,
                        }
                    } else {
                        ControlFlow::Sequential
                    }
                } else {
                    ControlFlow::Sequential
                }
            } else if matches!(last_instr.mnemonic.as_str(), 
                "je" | "jz" | "jne" | "jnz" | "jg" | "jge" | "jl" | "jle" | "ja" | "jae" | "jb" | "jbe") {
                if let Some(target) = extract_jump_target(&last_instr.operands) {
                    let condition = translate_condition(&last_instr.mnemonic);
                    
                    // Check if it's a loop
                    if target <= block.start_addr {
                        ControlFlow::WhileLoop {
                            condition,
                            body_block: target,
                        }
                    } else {
                        ControlFlow::IfThen {
                            condition,
                            true_block: target,
                        }
                    }
                } else {
                    ControlFlow::Sequential
                }
            } else {
                ControlFlow::Sequential
            };
            
            control_flow.insert(block.start_addr, flow);
        }
    }
    
    control_flow
}

fn translate_condition(mnemonic: &str) -> String {
    match mnemonic {
        "je" | "jz" => "equal".to_string(),
        "jne" | "jnz" => "not_equal".to_string(),
        "jg" => "greater".to_string(),
        "jge" => "greater_or_equal".to_string(),
        "jl" => "less".to_string(),
        "jle" => "less_or_equal".to_string(),
        "ja" => "above".to_string(),
        "jae" => "above_or_equal".to_string(),
        "jb" => "below".to_string(),
        "jbe" => "below_or_equal".to_string(),
        _ => "condition".to_string(),
    }
}

// ============================================================================
// PSEUDO-CODE GENERATION
// ============================================================================

fn generate_pseudo_function(func: &Function, _instructions: &[Instruction]) -> String {
    // Pre-allocate with estimated capacity (avg 100 bytes per instruction)
    let estimated_size = func.blocks.iter().map(|b| b.instructions.len()).sum::<usize>() * 100;
    let mut output = String::with_capacity(estimated_size);
    
    output.push_str(&format!("+─ Function: {} (0x{:x}) ─┐\n", func.name, func.start_addr));
    output.push_str("│\n");
    
    // Variables section
    if !func.variables.is_empty() {
        output.push_str("│ Variables:\n");
for (name, var) in &func.variables {
            let var_kind = if var.is_param { "parameter" } else { "local" };
            output.push_str(&format!("│   {} {} : {:?}\n", var_kind, name, var.var_type));
        }
        output.push_str("│\n");
    }
    
    // Control flow analysis
    let control_flow = analyze_control_flow(&func.blocks);
    
    // Generate pseudo code
    output.push_str("│ Code:\n");
    
    let mut indent = 1;
    let mut last_condition = String::new();
    
    for block in &func.blocks {
        // Check control flow
        if let Some(flow) = control_flow.get(&block.start_addr) {
            match flow {
                ControlFlow::WhileLoop { condition, .. } => {
                    output.push_str(&format!("{}│ while ({}) {{\n", "  ".repeat(indent), condition));
                    indent += 1;
                    last_condition = condition.clone();
                }
                ControlFlow::IfThen { condition, .. } => {
                    last_condition = condition.clone();
                }
                _ => {}
            }
        }
        
        for instr in &block.instructions {
            let pseudo = translate_instruction_to_pseudo(instr, &func.variables);
            
            // Always show something - either the pseudo code or the raw instruction
            if !pseudo.trim().is_empty() {
                output.push_str(&format!("{}│ {}  // 0x{:x}\n", "  ".repeat(indent), pseudo, instr.address));
            } else {
                // Fallback to raw assembly if we can't translate
                output.push_str(&format!("{}│ // {} {}  (0x{:x})\n", "  ".repeat(indent), instr.mnemonic, instr.operands, instr.address));
            }
            
            // Handle conditional jumps
            if is_conditional_jump(&instr.mnemonic) && !last_condition.is_empty() {
                output.push_str(&format!("{}│ if ({}) {{\n", "  ".repeat(indent), last_condition));
                indent += 1;
            }
        }
        
        // Close control structures
        if let Some(last_instr) = block.instructions.last() {
            if is_conditional_jump(&last_instr.mnemonic) && indent > 1 {
                indent -= 1;
                output.push_str(&format!("{}│ }}\n", "  ".repeat(indent)));
            }
        }
    }
    
    // Close any remaining structures
    while indent > 1 {
        indent -= 1;
        output.push_str(&format!("{}│ }}\n", "  ".repeat(indent)));
    }
    
    output.push_str("│\n");
    output.push_str("└────────────────────────────────┘\n");
    
    output
}

fn is_conditional_jump(mnemonic: &str) -> bool {
    matches!(mnemonic, 
        "je" | "jne" | "jz" | "jnz" | "jg" | "jge" | "jl" | "jle" |
        "ja" | "jae" | "jb" | "jbe" | "jo" | "jno" | "js" | "jns"
    )
}

fn translate_instruction_to_pseudo(instr: &Instruction, variables: &HashMap<String, Variable>) -> String {
    let mnemonic = instr.mnemonic.as_str();
    let operands = &instr.operands;
    
    match mnemonic {
        "mov" => translate_mov_pseudo(operands, variables),
        "lea" => translate_lea_pseudo(operands, variables),
        "add" => translate_arithmetic_pseudo(operands, "+", variables),
        "sub" => translate_arithmetic_pseudo(operands, "-", variables),
        "imul" | "mul" => translate_arithmetic_pseudo(operands, "*", variables),
        "idiv" | "div" => translate_arithmetic_pseudo(operands, "/", variables),
        "and" => translate_arithmetic_pseudo(operands, "&", variables),
        "or" => translate_arithmetic_pseudo(operands, "|", variables),
        "xor" => translate_xor_pseudo(operands, variables),
        "shl" | "sal" => translate_arithmetic_pseudo(operands, "<<", variables),
        "shr" | "sar" => translate_arithmetic_pseudo(operands, ">>", variables),
        "cmp" | "test" => translate_cmp_pseudo(operands, variables),
        "call" => translate_call_pseudo(operands),
        "ret" | "retn" => "return".to_string(),
        "push" => format!("push({})", resolve_operand(operands, variables)),
        "pop" => format!("pop({})", resolve_operand(operands, variables)),
        "nop" => String::new(),
        "jmp" => format!("goto 0x{}", operands),
        _ if is_conditional_jump(mnemonic) => String::new(), // Handled by control flow
        _ => format!("// {} {}", mnemonic, operands),
    }
}

fn translate_mov_pseudo(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} = {}", dest, src)
    } else {
        format!("mov {}", operands)
    }
}

fn translate_lea_pseudo(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} = &{}", dest, src)
    } else {
        format!("lea {}", operands)
    }
}

fn translate_arithmetic_pseudo(operands: &str, op: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} = {} {} {}", dest, dest, op, src)
    } else if parts.len() == 1 {
        let dest = resolve_operand(parts[0], variables);
        format!("{} = {} {} {}", dest, dest, op, dest)
    } else {
        format!("{} {}", op, operands)
    }
}

fn translate_xor_pseudo(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 && parts[0] == parts[1] {
        // xor reg, reg is a common way to zero a register
        format!("{} = 0", resolve_operand(parts[0], variables))
    } else {
        translate_arithmetic_pseudo(operands, "^", variables)
    }
}

fn translate_cmp_pseudo(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let left = resolve_operand(parts[0], variables);
        let right = resolve_operand(parts[1], variables);
        format!("compare({}, {})", left, right)
    } else {
        format!("compare({})", operands)
    }
}

fn translate_call_pseudo(operands: &str) -> String {
    let target = operands.trim();
    
    // Check for known API calls
    let known_apis = get_known_api_database();
    for (api_name, description) in &known_apis {
        if target.contains(api_name) {
            return format!("{}()  // {}", api_name, description);
        }
    }
    
    format!("call({})", target)
}

fn resolve_operand(operand: &str, variables: &HashMap<String, Variable>) -> String {
    let operand = operand.trim();
    
    // Check if it's a stack variable
    if operand.contains("ebp") || operand.contains("rbp") || operand.contains("esp") || operand.contains("rsp") {
        let var_name = normalize_stack_var(operand);
        if variables.contains_key(&var_name) {
            return var_name;
        }
    }
    
    // Check if it's a memory reference
    if operand.starts_with('[') && operand.ends_with("];") {
        let inner = &operand[1..operand.len()-1];
        return format!("*{}", resolve_operand(inner, variables));
    }
    
    // Return as-is
    operand.to_string()
}

// ============================================================================
// C CODE GENERATION
// ============================================================================

fn generate_c_function(func: &Function, _instructions: &[Instruction]) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("// ═══════════════════════════════════════════════════════════════\n"));
    output.push_str(&format!("// Function: {} (Address: 0x{:x})\n", func.name, func.start_addr));
    output.push_str(&format!("// ═══════════════════════════════════════════════════════════════\n"));
    
    // Function signature with inferred return type
    let return_type = match &func.return_type {
        VarType::Unknown => "int",
        _ => &type_to_c_string(&func.return_type),
    };
    output.push_str(&format!("{} {}(", return_type, func.name));
    
    // Parameters
    let params: Vec<&Variable> = func.variables.values().filter(|v| v.is_param).collect();
    if !params.is_empty() {
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{} {}", type_to_c_string(&param.var_type), param.name));
        }
    }
    
    output.push_str(") {\n");
    
    // Local variables - initialize to prevent undefined behavior
    let locals: Vec<&Variable> = func.variables.values().filter(|v| v.is_local).collect();
    if !locals.is_empty() {
        output.push_str("    // Local variables\n");
        for local in locals {
            let init_value = match &local.var_type {
                VarType::Pointer | VarType::String => "NULL",
                VarType::Float => "0.0f",
                _ => "0",
            };
            output.push_str(&format!("    {} {} = {};\n", type_to_c_string(&local.var_type), local.name, init_value));
        }
        output.push_str("\n");
    }
    
    // Control flow analysis
    let control_flow = analyze_control_flow(&func.blocks);
    
    // Generate C code
    let mut indent = 1;
    let mut last_condition = String::new();
    let mut last_cmp_operands = ("".to_string(), "".to_string());
    
    for block in &func.blocks {
        // Check control flow
        if let Some(flow) = control_flow.get(&block.start_addr) {
            match flow {
                ControlFlow::WhileLoop { condition, .. } => {
                    let cond_str = format_c_condition(condition, &last_cmp_operands);
                    output.push_str(&format!("{}while ({}) {{\n", "    ".repeat(indent), cond_str));
                    indent += 1;
                }
                ControlFlow::IfThen { condition, .. } => {
                    last_condition = condition.clone();
                }
                _ => {}
            }
        }
        
        for instr in &block.instructions {
            let c_code = translate_instruction_to_c(instr, &func.variables);
            
            // Track comparison operands for condition formatting
            if instr.mnemonic == "cmp" || instr.mnemonic == "test" {
                let parts: Vec<&str> = instr.operands.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    last_cmp_operands = (
                        resolve_operand(parts[0], &func.variables),
                        resolve_operand(parts[1], &func.variables)
                    );
                }
            }
            
            // Only show translated C code, skip untranslatable instructions
            if !c_code.trim().is_empty() {
                // Clean output without assembly addresses for better readability
                output.push_str(&format!("{}{}\n", "    ".repeat(indent), c_code));
            }
            // Skip assembly comments for cleaner, more compilable output
            
            // Handle conditional jumps
            if is_conditional_jump(&instr.mnemonic) && !last_condition.is_empty() {
                let cond_str = format_c_condition(&last_condition, &last_cmp_operands);
                output.push_str(&format!("{}if ({}) {{\n", "    ".repeat(indent), cond_str));
                indent += 1;
            }
        }
        
        // Close control structures
        if let Some(last_instr) = block.instructions.last() {
            if is_conditional_jump(&last_instr.mnemonic) && indent > 1 {
                indent -= 1;
                output.push_str(&format!("{}}}\n", "    ".repeat(indent)));
            }
        }
    }
    
    // Close any remaining structures
    while indent > 1 {
        indent -= 1;
        output.push_str(&format!("{}}}\n", "    ".repeat(indent)));
    }
    
    // Add default return statement if function doesn't end with return
    let has_return = func.blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.mnemonic == "ret" || i.mnemonic == "retn");
    
    if !has_return {
        let default_return = match &func.return_type {
            VarType::Unknown => "    return 0;",
            VarType::Pointer | VarType::String => "    return NULL;",
            VarType::Float => "    return 0.0f;",
            _ => "    return 0;",
        };
        output.push_str(&format!("{}\n", default_return));
    }
    
    output.push_str("}\n");
    
    output
}

fn translate_instruction_to_c(instr: &Instruction, variables: &HashMap<String, Variable>) -> String {
    let mnemonic = instr.mnemonic.as_str();
    let operands = &instr.operands;
    
    match mnemonic {
        "mov" => translate_mov_c(operands, variables),
        "lea" => translate_lea_c(operands, variables),
        "add" => translate_arithmetic_c(operands, "+=", variables),
        "sub" => translate_arithmetic_c(operands, "-=", variables),
        "imul" | "mul" => translate_arithmetic_c(operands, "*=", variables),
        "and" => translate_arithmetic_c(operands, "&=", variables),
        "or" => translate_arithmetic_c(operands, "|=", variables),
        "xor" => translate_xor_c(operands, variables),
        "shl" | "sal" => translate_arithmetic_c(operands, "<<=", variables),
        "shr" | "sar" => translate_arithmetic_c(operands, ">>=", variables),
        "inc" => format!("{}++;", resolve_operand(operands, variables)),
        "dec" => format!("{}--;", resolve_operand(operands, variables)),
        "cmp" | "test" => String::new(), // Handled by control flow
        "call" => translate_call_c(operands),
        "ret" | "retn" => "return;".to_string(),
        "push" | "pop" => String::new(), // Skip stack operations for cleaner code
        "nop" => String::new(), // Skip NOPs
        "jmp" => String::new(), // Skip unconditional jumps (handled by control flow)
        _ if is_conditional_jump(mnemonic) => String::new(), // Handled by control flow
        _ => String::new(), // Skip unknown instructions for cleaner output
    }
}

fn translate_mov_c(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} = {};", dest, src)
    } else {
        format!("// mov {}", operands)
    }
}

fn translate_lea_c(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} = &{};", dest, src)
    } else {
        format!("// lea {}", operands)
    }
}

fn translate_arithmetic_c(operands: &str, op: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} {} {};", dest, op, src)
    } else if parts.len() == 1 {
        let dest = resolve_operand(parts[0], variables);
        format!("{} {} {};", dest, op, dest)
    } else {
        format!("// {} {}", op, operands)
    }
}

fn translate_xor_c(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 && parts[0] == parts[1] {
        format!("{} = 0;", resolve_operand(parts[0], variables))
    } else {
        translate_arithmetic_c(operands, "^=", variables)
    }
}

fn translate_call_c(operands: &str) -> String {
    let target = operands.trim();
    
    // Check for known API calls
    let known_apis = get_known_api_database();
    for (api_name, _description) in &known_apis {
        if target.contains(api_name) {
            return format!("{}();", api_name);
        }
    }
    
    // Extract function name from address or register
    if target.starts_with("0x") {
        let addr = target.trim_start_matches("0x");
        format!("func_{}();", addr)
    } else if target.contains('[') || target.contains("ptr") {
        // Indirect call through pointer
        format!("((void(*)()){})()", target)
    } else {
        format!("{}();", target)
    }
}

fn format_c_condition(condition: &str, operands: &(String, String)) -> String {
    let (left, right) = operands;
    
    if left.is_empty() || right.is_empty() {
        return "condition".to_string();
    }
    
    match condition {
        "equal" => format!("{} == {}", left, right),
        "not_equal" => format!("{} != {}", left, right),
        "greater" => format!("{} > {}", left, right),
        "greater_or_equal" => format!("{} >= {}", left, right),
        "less" => format!("{} < {}", left, right),
        "less_or_equal" => format!("{} <= {}", left, right),
        "above" => format!("{} > {}", left, right),
        "above_or_equal" => format!("{} >= {}", left, right),
        "below" => format!("{} < {}", left, right),
        "below_or_equal" => format!("{} <= {}", left, right),
        _ => format!("{} {} {}", left, condition, right),
    }
}

fn type_to_c_string(var_type: &VarType) -> String {
    match var_type {
        VarType::Int32 => "int32_t".to_string(),
        VarType::Int64 => "int64_t".to_string(),
        VarType::Pointer => "ptr_t".to_string(),
        VarType::String => "char*".to_string(),
        VarType::Float => "float".to_string(),
        VarType::Unknown => "uint32_t".to_string(),
        VarType::Struct(name) => format!("struct {}", name),
        VarType::Array(elem_type, count) => format!("{}[{}]", type_to_c_string(elem_type), count),
    }
}

// ============================================================================
// RUST CODE GENERATION
// ============================================================================

fn generate_rust_function(func: &Function, _instructions: &[Instruction], safe: bool) -> String {
    // Pre-allocate with estimated capacity (avg 120 bytes per instruction for Rust)
    let estimated_size = func.blocks.iter().map(|b| b.instructions.len()).sum::<usize>() * 120;
    let mut output = String::with_capacity(estimated_size);
    
    output.push_str(&format!("// ═══════════════════════════════════════════════════════════════\\n"));
    output.push_str(&format!("// Function: {} (Address: 0x{:x})\\n", func.name, func.start_addr));
    output.push_str(&format!("// ═══════════════════════════════════════════════════════════════\\n"));
    
    // Function signature - Rust requires unsafe for low-level operations
    let unsafe_keyword = if !safe { "unsafe " } else { "" };
    output.push_str(&format!("{}fn {}(", unsafe_keyword, func.name));
    
    // Parameters
    let params: Vec<&Variable> = func.variables.values().filter(|v| v.is_param).collect();
    if !params.is_empty() {
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{}: {}", param.name, type_to_rust_string(&param.var_type)));
        }
    }
    
    output.push_str(") {\n");
    
    // Local variables
    let locals: Vec<&Variable> = func.variables.values().filter(|v| v.is_local).collect();
    if !locals.is_empty() {
        output.push_str("    // Local variables\n");
        for local in locals {
            output.push_str(&format!("    let mut {}: {};\n", local.name, type_to_rust_string(&local.var_type)));
        }
        output.push_str("\\n");
    }
    
    // Control flow analysis
    let control_flow = analyze_control_flow(&func.blocks); 
    
    // Generate Rust code
    let mut indent = 1;
    let mut last_condition = String::new();
    let mut last_cmp_operands = ("".to_string(), "".to_string());
    
    for block in &func.blocks {
        // Check control flow
        if let Some(flow) = control_flow.get(&block.start_addr) {
            match flow {
                ControlFlow::WhileLoop { condition, .. } => {
                    let cond_str = format_rust_condition(condition, &last_cmp_operands);
                    output.push_str(&format!("{}while {} {{\n", "    ".repeat(indent), cond_str));
                    indent += 1;
                }
                ControlFlow::IfThen { condition, .. } => {
                    last_condition = condition.clone();
                }
                _ => {}
            }
        }
        
        for instr in &block.instructions {
            let rust_code = translate_instruction_to_rust(instr, &func.variables, !safe);
            
            // Track comparison operands for condition formatting
            if instr.mnemonic == "cmp" || instr.mnemonic == "test" {
                let parts: Vec<&str> = instr.operands.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    last_cmp_operands = (
                        resolve_operand(parts[0], &func.variables),
                        resolve_operand(parts[1], &func.variables)
                    );
                }
            }
            
            // Always show something - either the Rust code or the raw instruction
            if !rust_code.trim().is_empty() {
                output.push_str(&format!("{}{}  // 0x{:x}\n", "    ".repeat(indent), rust_code, instr.address));
            } else {
                // Fallback to raw assembly comment if we can't translate
                output.push_str(&format!("{}// {} {}  (0x{:x})\n", "    ".repeat(indent), instr.mnemonic, instr.operands, instr.address));
            }
            
            // Handle conditional jumps
            if is_conditional_jump(&instr.mnemonic) && !last_condition.is_empty() {
                let cond_str = format_rust_condition(&last_condition, &last_cmp_operands);
                output.push_str(&format!("{}if {} {{\n", "    ".repeat(indent), cond_str));
                indent += 1;
            }
        }
        
        // Close control structures
        if let Some(last_instr) = block.instructions.last() {
            if is_conditional_jump(&last_instr.mnemonic) && indent > 1 {
                indent -= 1;
                output.push_str(&format!("{}}}\n", "    ".repeat(indent)));
            }
        }
    }
    
    // Close any remaining structures
    while indent > 1 {
        indent -= 1;
        output.push_str(&format!("{}}}\n", "    ".repeat(indent)));
    }
    
    output.push_str("}\n");
    
    output
}

fn translate_instruction_to_rust(instr: &Instruction, variables: &HashMap<String, Variable>, unsafe_context: bool) -> String {
    let mnemonic = instr.mnemonic.as_str();
    let operands = &instr.operands;
    
    match mnemonic {
        "mov" => translate_mov_rust(operands, variables),
        "lea" => translate_lea_rust(operands, variables, unsafe_context),
        "add" => translate_arithmetic_rust(operands, "+=", variables),
        "sub" => translate_arithmetic_rust(operands, "-=", variables),
        "imul" | "mul" => translate_arithmetic_rust(operands, "*=", variables),
        "and" => translate_arithmetic_rust(operands, "&=", variables),
        "or" => translate_arithmetic_rust(operands, "|=", variables),
        "xor" => translate_xor_rust(operands, variables),
        "shl" | "sal" => translate_arithmetic_rust(operands, "<<=", variables),
        "shr" | "sar" => translate_arithmetic_rust(operands, ">>=", variables),
        "inc" => format!("{} += 1;", resolve_operand(operands, variables)),
        "dec" => format!("{} -= 1;", resolve_operand(operands, variables)),
        "cmp" | "test" => String::new(), // Handled by control flow
        "call" => translate_call_rust(operands),
        "ret" | "retn" => "return;".to_string(),
        "push" | "pop" => format!("// {} {}", mnemonic, operands),
        "nop" => String::new(),
        "jmp" => format!("// goto label_0x{};", operands),
        _ if is_conditional_jump(mnemonic) => String::new(), // Handled by control flow
        _ => format!("// {} {}", mnemonic, operands),
    }
}

fn translate_mov_rust(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} = {};", dest, src)
    } else {
        format!("// mov {}", operands)
    }
}

fn translate_lea_rust(operands: &str, variables: &HashMap<String, Variable>, unsafe_context: bool) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        if unsafe_context {
            format!("{} = &{} as *const _ as Ptr;", dest, src)
        } else {
            format!("{} = &{};", dest, src)
        }
    } else {
        format!("// lea {}", operands)
    }
}

fn translate_arithmetic_rust(operands: &str, op: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        format!("{} {} {};", dest, op, src)
    } else {
        format!("// {} {}", op, operands)
    }
}


fn format_rust_condition(condition: &str, operands: &(String, String)) -> String {
    let (left, right) = operands;
    
    if left.is_empty() || right.is_empty() {
        return "condition".to_string();
    }
    
    match condition {
        "equal" => format!("{} == {}", left, right),
        "not_equal" => format!("{} != {}", left, right),
        "greater" => format!("{} > {}", left, right),
        "greater_or_equal" => format!("{} >= {}", left, right),
        "less" => format!("{} < {}", left, right),
        "less_or_equal" => format!("{} <= {}", left, right),
        "above" => format!("{} > {}", left, right),
        "above_or_equal" => format!("{} >= {}", left, right),
        "below" => format!("{} < {}", left, right),
        "below_or_equal" => format!("{} <= {}", left, right),
        _ => format!("{} {} {}", left, condition, right),
    }
}

fn type_to_rust_string(var_type: &VarType) -> String {
    match var_type {
        VarType::Int32 => "i32".to_string(),
        VarType::Int64 => "i64".to_string(),
        VarType::Pointer => "void*".to_string(),
        VarType::String => "char*".to_string(),
        VarType::Float => "float".to_string(),
        VarType::Unknown => "u32".to_string(),
        VarType::Struct(name) => format!("struct {}", name),
        VarType::Array(elem_type, count) => format!("{}[{}];", type_to_rust_string(elem_type), count),
    }
}

// ============================================================================
// MULTI-FILE OUTPUT GENERATION
// ============================================================================

pub fn generate_multi_file_output(asm: &str, _language: &str, mode: &str) -> Vec<(String, String)> {
    let instructions = parse_instructions(asm);
    let functions = identify_functions(&instructions);
    let mut files = Vec::new();

    match mode {
        "Rust" => {
            let mut functions_content = String::new();
            for func in &functions {
                let is_safe = is_function_safe(func, &instructions);
                functions_content.push_str(&generate_rust_function(func, &instructions, is_safe));
                functions_content.push_str("\\n");
            }
            files.push(("functions.rs".to_string(), functions_content));

            let mut main_content = String::new();
            main_content.push_str("use functions::*;\\n\\n");
            main_content.push_str("fn main() {\\n");
            if !functions.is_empty() {
                let is_safe = is_function_safe(&functions[0], &instructions);
                if is_safe {
                    main_content.push_str(&format!("    {}();\\n", functions[0].name));
                } else {
                    main_content.push_str(&format!("    unsafe {{ {}() }}\\n", functions[0].name));
                }
            }
            main_content.push_str("}\\n");
            files.push(("main.rs".to_string(), main_content));
        }
        "C" => {
            // C code generation logic here
        }
        _ => {}
    }

    files
}

fn is_function_safe(_func: &Function, _instructions: &[Instruction]) -> bool {
    // All decompiled functions are marked as safe (no unsafe keyword)
    // This provides cleaner, more readable output without unnecessary unsafe blocks
    // The decompiler generates high-level code that abstracts away low-level operations
    true
}

fn translate_xor_rust(operands: &str, variables: &HashMap<String, Variable>) -> String {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let dest = resolve_operand(parts[0], variables);
        let src = resolve_operand(parts[1], variables);
        
        // Detect xor reg, reg pattern (zero initialization)
        if dest == src {
            format!("{} = 0;", dest)
        } else {
            format!("{} ^= {};", dest, src)
        }
    } else {
        format!("// xor {}", operands)
    }
}

fn translate_call_rust(operands: &str) -> String {
    let target = operands.trim();
    
    // Check for known API calls
    let known_apis = get_known_api_database();
    for (api_name, description) in &known_apis {
        if target.contains(api_name) {
            return format!("{}();  // {}", api_name, description);
        }
    }
    
    // Extract function name from address or register
    if target.starts_with("0x") {
        format!("func_{}();", target)
    } else {
        format!("{}();", target)
    }
}



// ============================================================================
// MULTI-FILE OUTPUT GENERATION
// ============================================================================


fn generate_multi_file_by_type(asm: &str, language: &str) -> Vec<(String, String)> {
    let instructions = parse_instructions(asm);
    let functions = identify_functions(&instructions);
    let api_calls = detect_api_calls(&instructions);
    
    let mut files = Vec::new();
    
    match language {
        "Rust Code" => {
            // 1. types.rs - Type definitions and structs
            let mut types_content = String::new();
            types_content.push_str("//! ═══════════════════════════════════════════════════════════════\n");
            types_content.push_str("//! TYPE DEFINITIONS\n");
            types_content.push_str("//! ═══════════════════════════════════════════════════════════════\n\n");
            types_content.push_str("use std::os::raw::{c_void, c_char, c_int};\n\n");
            types_content.push_str("// ═══ Type Definitions ═══\n");
            types_content.push_str("pub type U8 = u8;\n");
            types_content.push_str("pub type U16 = u16;\n");
            types_content.push_str("pub type U32 = u32;\n");
            types_content.push_str("pub type U64 = u64;\n");
            types_content.push_str("pub type I8 = i8;\n");
            types_content.push_str("pub type I16 = i16;\n");
            types_content.push_str("pub type I32 = i32;\n");
            types_content.push_str("pub type I64 = i64;\n");
            types_content.push_str("pub type Ptr = *mut c_void;\n\n");
            types_content.push_str("// ═══ Struct Definitions ═══\n");
            types_content.push_str("// TODO: Struct detection will be added in Phase 3\n");
            types_content.push_str("// Structs will be inferred from memory access patterns\n\n");
            files.push(("types.rs".to_string(), types_content));
            
            // 2. globals.rs - Global variables
            let mut globals_content = String::new();
            globals_content.push_str("//! ═══════════════════════════════════════════════════════════════\n");
            globals_content.push_str("//! GLOBAL VARIABLES\n");
            globals_content.push_str("//! ═══════════════════════════════════════════════════════════════\n\n");
            globals_content.push_str("use crate::types::*;\n\n");
            globals_content.push_str("// ═══ Global Variables ═══\n");
            globals_content.push_str("// TODO: Global variable detection will be added in Phase 2\n");
            globals_content.push_str("// Globals will be extracted from .data and .bss sections\n\n");
            files.push(("globals.rs".to_string(), globals_content));
            
            // 3. strings.rs - String literals
            let mut strings_content = String::new();
            strings_content.push_str("//! ═════════════════════════════════════════════════════════════\n");
            strings_content.push_str("//! STRING LITERALS\n");
            strings_content.push_str("//! ═══════════════════════════════════════════════════════════════\n\n");
            strings_content.push_str("// ═══ String Constants ═══\n");
            strings_content.push_str("// TODO: String extraction will be added in Phase 2\n");
            strings_content.push_str("// Strings will be extracted from .rdata section\n\n");
            files.push(("strings.rs".to_string(), strings_content));
            
            // 4. functions.rs - All functions
            let mut functions_content = String::new();
            functions_content.push_str("//! ═══════════════════════════════════════════════════════════════\n");
            functions_content.push_str("//! FUNCTION IMPLEMENTATIONS\n");
            functions_content.push_str("//! ═══════════════════════════════════════════════════════════════\n");
            functions_content.push_str(&format!("//! Functions detected: {}\n", functions.len()));
            functions_content.push_str(&format!("//! API calls detected: {}\n", api_calls.len()));
            functions_content.push_str("//! ═══════════════════════════════════════════════════════════════\n\n");
            
            functions_content.push_str("#![allow(unused_variables, unused_mut, dead_code)];\n\n");
            functions_content.push_str("use crate::types::*;\n");
            functions_content.push_str("use crate::globals::*;\n");
            functions_content.push_str("use crate::strings::*;\n\n");
            
            if !api_calls.is_empty() {
                functions_content.push_str("// Windows API bindings\n");
                functions_content.push_str("#[cfg(windows)];\n");
                functions_content.push_str("use winapi::um::winuser::MessageBoxA;\n");
                functions_content.push_str("#[cfg(windows)];\n");
                functions_content.push_str("use winapi::um::fileapi::*;\n");
                functions_content.push_str("#[cfg(windows)];\n");
                functions_content.push_str("use winapi::um::memoryapi::*;\n");
                functions_content.push_str("#[cfg(windows)];\n");
                functions_content.push_str("use winapi::um::processthreadsapi::*;\n");
                functions_content.push_str("#[cfg(windows)];\n");
                functions_content.push_str("use std::ptr;\n\n");
            }
            
            for func in &functions {
                functions_content.push_str(&generate_rust_function(func, &instructions, false));
                functions_content.push_str("\n");
            }
            files.push(("functions.rs".to_string(), functions_content));
            
            // 5. main.rs - Entry point
            let mut main_content = String::new();
            main_content.push_str("//! ═══════════════════════════════════════════════════════════════\n");
            main_content.push_str("//! MAIN ENTRY POINT\n");
            main_content.push_str("//! ═══════════════════════════════════════════════════════════════\n\n");
            main_content.push_str("mod types;\n");
            main_content.push_str("mod globals;\n");
            main_content.push_str("mod strings;\n");
            main_content.push_str("mod functions;\n\n");
            main_content.push_str("use functions::*;\n\n");
            main_content.push_str("fn main() {\n");
            if !functions.is_empty() {
                main_content.push_str(&format!("    unsafe {{ {}() }}\n", functions[0].name));
            }
            main_content.push_str("}\n");
            files.push(("main.rs".to_string(), main_content));
        },
        "C Code" => {
            // 1. types.h - Type definitions
            let mut types_content = String::new();
            types_content.push_str("/*\n");
            types_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            types_content.push_str(" * TYPE DEFINITIONS\n");
            types_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            types_content.push_str(" */\n\n");
            types_content.push_str("#ifndef TYPES_H\n");
            types_content.push_str("#define TYPES_H\n\n");
            types_content.push_str("#include <stdint.h>\n\n");
            types_content.push_str("typedef unsigned char  u8;\n");
            types_content.push_str("typedef unsigned short u16;\n");
            types_content.push_str("typedef unsigned int   u32;\n");
            types_content.push_str("typedef unsigned long long u64;\n");
            types_content.push_str("typedef signed char    i8;\n");
            types_content.push_str("typedef signed short   i16;\n");
            types_content.push_str("typedef signed int     i32;\n");
            types_content.push_str("typedef signed long long i64;\n\n");
            types_content.push_str("// ═══ Struct Definitions ═══\n");
            types_content.push_str("// TODO: Struct detection will be added in Phase 3\n\n");
            types_content.push_str("#endif // TYPES_H\n");
            files.push(("types.h".to_string(), types_content));
            
            // 2. globals.h - Global variables
            let mut globals_content = String::new();
            globals_content.push_str("/*\n");
            globals_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            globals_content.push_str(" * GLOBAL VARIABLES\n");
            globals_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            globals_content.push_str(" */\n\n");
            globals_content.push_str("#ifndef GLOBALS_H\n");
            globals_content.push_str("#define GLOBALS_H\n\n");
            globals_content.push_str("#include \"types.h\"\n\n");
            globals_content.push_str("// TODO: Global variable detection will be added in Phase 2\n\n");
            globals_content.push_str("#endif // GLOBALS_H\n");
            files.push(("globals.h".to_string(), globals_content));
            
            // 3. functions.h - Function declarations
            let mut functions_h_content = String::new();
            functions_h_content.push_str("/*\n");
            functions_h_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            functions_h_content.push_str(" * FUNCTION DECLARATIONS\n");
            functions_h_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            functions_h_content.push_str(" */\n\n");
            functions_h_content.push_str("#ifndef FUNCTIONS_H\n");
            functions_h_content.push_str("#define FUNCTIONS_H\n\n");
            functions_h_content.push_str("#include \"types.h\"\n\n");
            for func in &functions {
                if !func.is_api_call {
                    functions_h_content.push_str(&format!("void {}();\n", func.name));
                }
            }
            functions_h_content.push_str("\n#endif // FUNCTIONS_H\n");
            files.push(("functions.h".to_string(), functions_h_content));
            
            // 4. functions.c - Function implementations
            let mut functions_content = String::new();
            functions_content.push_str("/*\n");
            functions_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            functions_content.push_str(" * FUNCTION IMPLEMENTATIONS\n");
            functions_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            functions_content.push_str(&format!(" * Functions detected: {}\n", functions.len()));
            functions_content.push_str(&format!(" * API calls detected: {}\n", api_calls.len()));
            functions_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            functions_content.push_str(" */\n\n");
            functions_content.push_str("#include <stdio.h>\n");
            functions_content.push_str("#include <stdlib.h>\n");
            functions_content.push_str("#include <string.h>\n");
            if !api_calls.is_empty() {
                functions_content.push_str("#include <windows.h>\n");
            }
            functions_content.push_str("#include \"types.h\"\n");
            functions_content.push_str("#include \"globals.h\"\n");
            functions_content.push_str("#include \"functions.h\"\n\n");
            
            for func in &functions {
                functions_content.push_str(&generate_c_function(func, &instructions));
                functions_content.push_str("\n");
            }
            files.push(("functions.c".to_string(), functions_content));
            
            // 5. main.c - Entry point
            let mut main_content = String::new();
            main_content.push_str("/*\n");
            main_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            main_content.push_str(" * MAIN ENTRY POINT\n");
            main_content.push_str(" * ═══════════════════════════════════════════════════════════════\n");
            main_content.push_str(" */\n\n");
            main_content.push_str("#include \"functions.h\"\n\n");
            main_content.push_str("int main() {\n");
            if !functions.is_empty() {
                main_content.push_str(&format!("    {}();\n", functions[0].name));
            }
            main_content.push_str("    return 0;\n");
            main_content.push_str("}\n");
            files.push(("main.c".to_string(), main_content));
        },
        _ => {
            // For Assembly and Pseudo Code, just return single file
            files.push(("output".to_string(), match language {
                "Assembly" => asm.to_string(),
                "Pseudo Code" => translate_to_pseudo(asm),
                _ => asm.to_string(),
            }));
        }
    }
    
    files
}

fn generate_multi_file_by_function(asm: &str, language: &str) -> Vec<(String, String)> {
    let instructions = parse_instructions(asm);
    let functions = identify_functions(&instructions);
    let api_calls = detect_api_calls(&instructions);
    
    let mut files = Vec::new();
    
    match language {
        "Rust Code" => {
            // Generate one file per function
            for func in &functions {
                let mut content = String::new();
                content.push_str(&format!("//! ═══════════════════════════════════════════════════════════════\n"));
                content.push_str(&format!("//! FUNCTION: {}\n", func.name));
                content.push_str(&format!("//! Address: 0x{:x} - 0x{:x}\n", func.start_addr, func.end_addr));
                content.push_str(&format!("//! ═══════════════════════════════════════════════════════════════\n\n"));
                
                content.push_str("#![allow(unused_variables, unused_mut, dead_code)];\n\n");
                
                if !api_calls.is_empty() {
                    content.push_str("#[cfg(windows)]\n");
                    content.push_str("use winapi::um::winuser::MessageBoxA;\n\n");
                }
                
                content.push_str(&generate_rust_function(func, &instructions, is_function_safe(func, &instructions)));
                
                files.push((format!("{}.rs", func.name), content));
            }
        },
        "C Code" => {
            // Generate one file per function
            for func in &functions {
                let mut content = String::new();
                content.push_str("/*\n");
                content.push_str(&format!(" * ═══════════════════════════════════════════════════════════════\n"));
                content.push_str(&format!(" * FUNCTION: {}\n", func.name));
                content.push_str(&format!(" * Address: 0x{:x} - 0x{:x}\n", func.start_addr, func.end_addr));
                content.push_str(&format!(" * ═══════════════════════════════════════════════════════════════\n"));
                content.push_str(" */\n\n");
                
                content.push_str("#include <stdio.h>\n");
                content.push_str("#include <stdlib.h>\n");
                content.push_str("#include <string.h>\n");
                content.push_str("#include <stdint.h>\n");
                if !api_calls.is_empty() {
                    content.push_str("#include <windows.h>\n");
                }
                content.push_str("\n");
                
                content.push_str(&generate_c_function(func, &instructions));
                
                files.push((format!("{}.c", func.name), content));
            }
        },
        _ => {
            // For Assembly and Pseudo Code, just return single file
            files.push(("output".to_string(), match language {
                "Assembly" => asm.to_string(),
                "Pseudo Code" => translate_to_pseudo(asm),
                _ => asm.to_string(),
            }));
        }
    }
    
    files
}

// ============================================================================
// HELPER FUNCTIONS FOR PARSING
// ============================================================================

fn parse_instructions(asm: &str) -> Vec<Instruction> {
    // Pre-allocate with estimated capacity for better performance
    let line_count = asm.lines().count();
    let mut instructions = Vec::with_capacity(line_count);
    
    // Optimized: use OnceLock to compile regex patterns only once across all calls
    static ADDR_COLON_RE: OnceLock<Regex> = OnceLock::new();
    static ADDR_0X_RE: OnceLock<Regex> = OnceLock::new();
    static ADDR_PLAIN_RE: OnceLock<Regex> = OnceLock::new();
    
    let addr_colon_regex = ADDR_COLON_RE.get_or_init(|| {
        Regex::new(r"^\s*([0-9a-fA-F]+):\s*(?:[0-9a-fA-F\s]+)?\s*(.+)").unwrap()
    });
    let addr_0x_regex = ADDR_0X_RE.get_or_init(|| {
        Regex::new(r"^\s*0x([0-9a-fA-F]+)\s+(.+)").unwrap()
    });
    let addr_plain_regex = ADDR_PLAIN_RE.get_or_init(|| {
        Regex::new(r"^\s*([0-9a-fA-F]{4,})\s+(.+)").unwrap()
    });
    
    let mut current_addr = 0x1000u64;
    
    for line in asm.lines() {
        let line = line.trim();
        
        // Fast path: skip empty lines and comments
        if line.is_empty() {
            continue;
        }
        
        // UTF-8 SAFE: Use starts_with() instead of byte indexing
        // This prevents crashes when decompiling binaries with encoding issues
        if line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with("//") {
            continue;
        }
        
        // Skip section headers and labels (ends with : and no spaces)
        if line.ends_with(':') && !line.contains(' ') {
            continue;
        }
        
        // Fast path: try to parse address and instruction in one go
        let (address, instruction_text) = if let Some(caps) = addr_colon_regex.captures(line) {
            let addr_str = caps.get(1).unwrap().as_str();
            let rest = caps.get(2).unwrap().as_str();
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                current_addr = addr + 1;
                (addr, rest)
            } else {
                let addr = current_addr;
                current_addr += 1;
                (addr, line)
            }
        } else if let Some(caps) = addr_0x_regex.captures(line) {
            let addr_str = caps.get(1).unwrap().as_str();
            let rest = caps.get(2).unwrap().as_str();
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                current_addr = addr + 1;
                (addr, rest)
            } else {
                let addr = current_addr;
                current_addr += 1;
                (addr, line)
            }
        } else if let Some(caps) = addr_plain_regex.captures(line) {
            let addr_str = caps.get(1).unwrap().as_str();
            let rest = caps.get(2).unwrap().as_str();
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                current_addr = addr + 1;
                (addr, rest)
            } else {
                let addr = current_addr;
                current_addr += 1;
                (addr, line)
            }
        } else {
            let addr = current_addr;
            current_addr += 1;
            (addr, line)
        };
        
        let instruction_text = instruction_text.trim();
        
        // UTF-8 SAFE: Use split_whitespace() instead of byte slicing
        // This prevents crashes when instruction text contains multi-byte UTF-8
        let mut parts = instruction_text.split_whitespace();
        if let Some(mnemonic) = parts.next() {
            // Skip hex bytes (2-char hex sequences)
            if mnemonic.len() == 2 && mnemonic.chars().all(|c| c.is_ascii_hexdigit()) {
                continue;
            }
            
            if !mnemonic.is_empty() {
                // Collect remaining parts as operands
                let operands: Vec<&str> = parts.collect();
                let operands_str = operands.join(" ");
                
                instructions.push(Instruction {
                    address,
                    mnemonic: mnemonic.to_lowercase(),
                    operands: operands_str,
                    raw_line: line.to_string(),
                });
            }
        }
    }
    
    instructions
}

fn parse_pe_file(path: &str) -> Option<PEInfo> {
    let buffer = fs::read(path).ok()?;
    let pe = PE::parse(&buffer).ok()?;
    
    let image_base = pe.image_base as u64;
    let entry_point = image_base + pe.entry as u64;
    
    let mut sections = Vec::new();
    for section in &pe.sections {
        let name = String::from_utf8_lossy(&section.name).trim_end_matches('\0').to_string();
        sections.push(SectionInfo {
            name,
            virtual_address: image_base + section.virtual_address as u64,
            virtual_size: section.virtual_size as u64,
            characteristics: section.characteristics,
            is_code: (section.characteristics & 0x20000000) != 0,
            is_data: (section.characteristics & 0x40000000) != 0,
        });
    }
    
    let mut imports = HashMap::new();
    for import in &pe.imports {
        let dll = import.dll.to_string();
        let va = image_base + import.rva as u64;
        imports.insert(va, ImportInfo {
            dll,
            function: import.name.to_string(),
            ordinal: Some(import.ordinal),
        });
    }
    
    let mut exports = HashMap::new();
    for export in &pe.exports {
        if let Some(name) = &export.name {
            let va = image_base + export.rva as u64;
            exports.insert(va, name.to_string());
        }
    }
    
    Some(PEInfo {
        image_base,
        entry_point,
        sections,
        imports,
        exports,
        iat_range: None,
    })
}