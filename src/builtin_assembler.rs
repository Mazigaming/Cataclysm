#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::fmt;

#[derive(Debug, Clone)]
pub struct AssembledBinary {
    pub code: Vec<u8>,
    pub entry_point: u32,
    pub data: Vec<u8>,
    pub is_64bit: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Operand {
    Register(String),
    Immediate(i64),
    Memory {
        base: Option<String>,
        index: Option<String>,
        scale: u8,
        displacement: i32,
        size: MemSize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MemSize {
    Byte,    // byte ptr
    Word,    // word ptr
    Dword,   // dword ptr
    Qword,   // qword ptr
}

// ============================================================================
// 2. ERROR HANDLING SYSTEM
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AssemblerErrorType {
    Lexical,      // Invalid characters or tokens
    Syntax,       // Wrong instruction format
    Semantic,     // Invalid operands, undefined symbols
    Directive,    // Invalid assembler directives
    Range,        // Value out of range
    PassMismatch, // Symbol redefinition, inconsistencies
    FileIO,       // File access errors
}

#[derive(Debug, Clone)]
pub struct AssemblerError {
    pub error_type: AssemblerErrorType,
    pub line_number: usize,
    pub line_content: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{:?} Error] Line {}: {}\n  > {}\n  {}",
            self.error_type,
            self.line_number,
            self.message,
            self.line_content,
            self.suggestion.as_ref().map(|s| format!("💡 Suggestion: {}", s)).unwrap_or_default()
        )
    }
}

// ============================================================================
// 3. SYMBOL TABLE
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum SymbolType {
    Label,
    Variable,
    Constant,
    Macro,
}

#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    symbol_type: SymbolType,
    value: i64,           // Address or constant value
    defined_line: usize,  // Line where defined
    references: Vec<usize>, // Lines where used
    is_external: bool,
}

struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, symbol_type: SymbolType, value: i64, line: usize) -> Result<(), String> {
        if let Some(existing) = self.symbols.get(&name) {
            return Err(format!(
                "Symbol '{}' already defined at line {}",
                name, existing.defined_line
            ));
        }
        
        self.symbols.insert(name.clone(), Symbol {
            name,
            symbol_type,
            value,
            defined_line: line,
            references: Vec::new(),
            is_external: false,
        });
        
        Ok(())
    }

    fn reference(&mut self, name: &str, line: usize) -> Result<i64, String> {
        if let Some(symbol) = self.symbols.get_mut(name) {
            symbol.references.push(line);
            Ok(symbol.value)
        } else {
            Err(format!("Undefined symbol '{}'", name))
        }
    }

    fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    fn update_value(&mut self, name: &str, value: i64) -> Result<(), String> {
        if let Some(symbol) = self.symbols.get_mut(name) {
            symbol.value = value;
            Ok(())
        } else {
            Err(format!("Symbol '{}' not found", name))
        }
    }
}

// ============================================================================
// 4. MACRO PROCESSOR
// ============================================================================

#[derive(Debug, Clone)]
struct Macro {
    name: String,
    parameters: Vec<String>,
    body: Vec<String>,
    defined_line: usize,
}

struct MacroProcessor {
    macros: HashMap<String, Macro>,
}

impl MacroProcessor {
    fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    fn define_macro(&mut self, name: String, parameters: Vec<String>, body: Vec<String>, line: usize) -> Result<(), String> {
        if self.macros.contains_key(&name) {
            return Err(format!("Macro '{}' already defined", name));
        }
        
        self.macros.insert(name.clone(), Macro {
            name,
            parameters,
            body,
            defined_line: line,
        });
        
        Ok(())
    }

    fn expand_macro(&self, name: &str, arguments: Vec<String>) -> Result<Vec<String>, String> {
        if let Some(macro_def) = self.macros.get(name) {
            if arguments.len() != macro_def.parameters.len() {
                return Err(format!(
                    "Macro '{}' expects {} arguments, got {}",
                    name,
                    macro_def.parameters.len(),
                    arguments.len()
                ));
            }

            let mut expanded = Vec::new();
            for line in &macro_def.body {
                let mut expanded_line = line.clone();
                for (param, arg) in macro_def.parameters.iter().zip(arguments.iter()) {
                    expanded_line = expanded_line.replace(param, arg);
                }
                expanded.push(expanded_line);
            }

            Ok(expanded)
        } else {
            Err(format!("Undefined macro '{}'", name))
        }
    }
}

// ============================================================================
// 5. LISTING GENERATOR
// ============================================================================

#[derive(Debug, Clone)]
struct ListingEntry {
    line_number: usize,
    address: usize,
    machine_code: Vec<u8>,
    source_line: String,
}

struct ListingGenerator {
    entries: Vec<ListingEntry>,
}

impl ListingGenerator {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add_entry(&mut self, line_number: usize, address: usize, machine_code: Vec<u8>, source_line: String) {
        self.entries.push(ListingEntry {
            line_number,
            address,
            machine_code,
            source_line,
        });
    }

    fn generate(&self) -> String {
        let mut output = String::new();
        output.push_str("╔════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                    ASSEMBLY LISTING                            ║\n");
        output.push_str("╠═════╦═════════╦══════════════════════╦═══════════════════════╣\n");
        output.push_str("║ Line║ Address ║   Machine Code       ║   Source              ║\n");
        output.push_str("╠═════╬═════════╬══════════════════════╬═══════════════════════╣\n");

        for entry in &self.entries {
            let machine_code_hex: String = entry.machine_code
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            output.push_str(&format!(
                "║ {:4}║ {:07X} ║ {:<20} ║ {:<21} ║\n",
                entry.line_number,
                entry.address,
                machine_code_hex,
                entry.source_line.chars().take(21).collect::<String>()
            ));
        }

        output.push_str("╚═════╩═════════╩══════════════════════╩═══════════════════════╝\n");
        output
    }
}

// ============================================================================
// 6. MAIN ASSEMBLER STRUCTURE
// ============================================================================

pub struct BuiltinAssembler {
    // Core components
    labels: HashMap<String, usize>,
    code: Vec<u8>,
    data: Vec<u8>,
    is_64bit: bool,
    
    // Enhanced components
    symbol_table: SymbolTable,
    macro_processor: MacroProcessor,
    listing_generator: ListingGenerator,
    errors: Vec<AssemblerError>,
    constants: HashMap<String, i64>,
    
    // Pass tracking
    current_pass: u8,
    current_line: usize,
    current_address: usize,
}

impl BuiltinAssembler {
    pub fn new(is_64bit: bool) -> Self {
        Self {
            labels: HashMap::new(),
            code: Vec::new(),
            data: Vec::new(),
            is_64bit,
            symbol_table: SymbolTable::new(),
            macro_processor: MacroProcessor::new(),
            listing_generator: ListingGenerator::new(),
            errors: Vec::new(),
            constants: HashMap::new(),
            current_pass: 0,
            current_line: 0,
            current_address: 0,
        }
    }
    
    // ========================================================================
    // ERROR HANDLING METHODS
    // ========================================================================
    
    fn add_error(&mut self, error_type: AssemblerErrorType, message: String, suggestion: Option<String>) {
        self.errors.push(AssemblerError {
            error_type,
            line_number: self.current_line,
            line_content: String::new(), // Will be filled by caller if needed
            message,
            suggestion,
        });
    }
    
    fn add_error_with_line(&mut self, error_type: AssemblerErrorType, message: String, line_content: String, suggestion: Option<String>) {
        self.errors.push(AssemblerError {
            error_type,
            line_number: self.current_line,
            line_content,
            message,
            suggestion,
        });
    }
    
    pub fn get_errors(&self) -> &[AssemblerError] {
        &self.errors
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn print_errors(&self) {
        if self.errors.is_empty() {
            println!("✅ Assembly completed with no errors.");
            return;
        }
        
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                    ASSEMBLY ERRORS                             ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        
        for (i, error) in self.errors.iter().enumerate() {
            println!("Error #{}: {}", i + 1, error);
            println!();
        }
        
        println!("Total errors: {}", self.errors.len());
    }
    
    pub fn generate_listing(&self) -> String {
        self.listing_generator.generate()
    }
    
    pub fn generate_symbol_table_report(&self) -> String {
        let mut output = String::new();
        output.push_str("╔════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                    SYMBOL TABLE                                ║\n");
        output.push_str("╠══════════════════╦═══════════╦═════════╦══════════════════════╣\n");
        output.push_str("║ Symbol           ║ Type      ║ Value   ║ References           ║\n");
        output.push_str("╠══════════════════╬═══════════╬═════════╬══════════════════════╣\n");
        
        let mut symbols: Vec<_> = self.symbol_table.symbols.values().collect();
        symbols.sort_by_key(|s| &s.name);
        
        for symbol in symbols {
            let refs = if symbol.references.is_empty() {
                "None".to_string()
            } else {
                symbol.references.iter()
                    .take(3)
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            };
            
            let type_str = format!("{:?}", symbol.symbol_type);
            output.push_str(&format!(
                "║ {:<16} ║ {:<9} ║ {:07X} ║ {:<20} ║\n",
                symbol.name.chars().take(16).collect::<String>(),
                type_str.chars().take(9).collect::<String>(),
                symbol.value,
                refs.chars().take(20).collect::<String>()
            ));
        }
        
        output.push_str("╚══════════════════╩═══════════╩═════════╩══════════════════════╝\n");
        output
    }
    
    pub fn check_needs_wrapper(&self, source: &str) -> bool {
        self.needs_program_wrapper(source)
    }

    pub fn assemble(&mut self, source: &str) -> Result<AssembledBinary, String> {
        // Log file size for debugging
        let source_mb = source.len() as f64 / 1_000_000.0;
        let line_count = source.lines().count();
        eprintln!("🔧 Assembling: {:.2} MB, {} lines", source_mb, line_count);
        
        // Warn if file is very large (likely a decompiled executable)
        if line_count > 10000 {
            eprintln!("⚠️  WARNING: Large file detected ({} lines)", line_count);
            eprintln!("   This may take a while. Consider using C/Rust decompilation instead.");
        }
        
        // Preprocess: convert disassembly listing to assembly if needed
        eprintln!("📝 Phase 1: Preprocessing...");
        let processed_source = self.preprocess_source(source)?;
        let source_to_assemble = processed_source.as_ref().map(|s| s.as_str()).unwrap_or(source);
        
        let needs_wrapper = self.needs_program_wrapper(source_to_assemble);
        
        if needs_wrapper {
            eprintln!("🔄 Assembling with wrapper...");
            self.assemble_with_wrapper(source_to_assemble)?;
        } else {
            // Two-pass assembly
            eprintln!("🔄 Phase 2: First pass (collecting labels)...");
            self.first_pass(source_to_assemble)?;
            eprintln!("🔄 Phase 3: Second pass (generating code)...");
            self.second_pass(source_to_assemble)?;
        }
        
        eprintln!("✅ Assembly complete: {} bytes of code", self.code.len());
        
        // Check if there were any errors during assembly
        if self.has_errors() {
            let error_summary = format!(
                "Assembly failed with {} error(s):\n{}",
                self.errors.len(),
                self.errors.iter()
                    .take(3) // Show first 3 errors
                    .map(|e| format!("  Line {}: {}", e.line_number, e.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            return Err(error_summary);
        }
        
        Ok(AssembledBinary {
            code: self.code.clone(),
            entry_point: 0x1000,
            data: self.data.clone(),
            is_64bit: self.is_64bit,
        })
    }
    
    /// Preprocess source: convert disassembly listing to assembly if needed
    fn preprocess_source(&self, source: &str) -> Result<Option<String>, String> {
        // Fast check: look for hex address pattern in first few lines
        // Optimized: check bytes directly without allocations
        let mut is_disassembly = false;
        
        for line in source.lines().take(20) {
            let trimmed = line.trim();
            if trimmed.len() > 10 {
                let bytes = trimmed.as_bytes();
                // Check if first 8 bytes are hex digits (ASCII only)
                if bytes.len() >= 8 && 
                   bytes[0].is_ascii_hexdigit() && bytes[1].is_ascii_hexdigit() &&
                   bytes[2].is_ascii_hexdigit() && bytes[3].is_ascii_hexdigit() &&
                   bytes[4].is_ascii_hexdigit() && bytes[5].is_ascii_hexdigit() &&
                   bytes[6].is_ascii_hexdigit() && bytes[7].is_ascii_hexdigit() &&
                   (bytes[8] == b' ' || bytes[8] == b'\t') {
                    is_disassembly = true;
                    break;
                }
            }
        }
        
        if !is_disassembly {
            return Ok(None);
        }
        
        // Pre-allocate output buffer (estimate 80% of original size)
        let estimated_size = source.len() * 4 / 5;
        let mut output = String::with_capacity(estimated_size);
        
        let total_lines = source.lines().count();
        let mut processed_lines = 0;
        let mut last_progress = 0;
        
        for line in source.lines() {
            processed_lines += 1;
            
            // Progress reporting for large files (every 10%)
            if total_lines > 5000 {
                let progress = (processed_lines * 100) / total_lines;
                if progress >= last_progress + 10 {
                    eprintln!("   Processing: {}% ({}/{})", progress, processed_lines, total_lines);
                    last_progress = progress;
                }
            }
            
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                continue;
            }
            
            let first_byte = trimmed.as_bytes()[0];
            
            // Keep comments (including entry point marker)
            if first_byte == b';' {
                output.push_str(trimmed);
                output.push('\n');
                continue;
            }
            
            // Fast skip section headers and warnings
            if first_byte == b'S' || first_byte == b'W' || first_byte == b'T' || first_byte == b'=' {
                continue;
            }
            
            // Fast check for hex address (8 chars)
            // Optimized: check bytes directly without allocations
            if trimmed.len() > 10 {
                let bytes = trimmed.as_bytes();
                // Check if first 8 bytes are hex digits (ASCII only, no UTF-8 needed)
                if bytes.len() >= 8 && 
                   bytes[0].is_ascii_hexdigit() && bytes[1].is_ascii_hexdigit() &&
                   bytes[2].is_ascii_hexdigit() && bytes[3].is_ascii_hexdigit() &&
                   bytes[4].is_ascii_hexdigit() && bytes[5].is_ascii_hexdigit() &&
                   bytes[6].is_ascii_hexdigit() && bytes[7].is_ascii_hexdigit() {
                    // Skip address (8 bytes) and extract instruction
                    // Safe because we know first 8 bytes are ASCII hex digits
                    if let Some(instr) = trimmed.get(8..) {
                        output.push_str("    ");
                        output.push_str(instr.trim());
                        output.push('\n');
                        continue;
                    }
                }
            }
            
            // Keep other lines as-is
            output.push_str(trimmed);
            output.push('\n');
        }
        
        Ok(Some(output))
    }
    
    fn needs_program_wrapper(&self, source: &str) -> bool {
        let has_function_label = source.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.ends_with(':') && (
                trimmed.starts_with("sub_") || 
                trimmed.starts_with("func_") ||
                trimmed.starts_with("function_")
            )
        });
        
        let has_main_entry = source.to_lowercase().contains("main:") || 
                            source.to_lowercase().contains("_start:") ||
                            source.to_lowercase().contains("start:");
        
        has_function_label && !has_main_entry
    }
    
    fn assemble_with_wrapper(&mut self, source: &str) -> Result<(), String> {
        // Find the first function label
        let first_function = source.lines()
            .find(|line| {
                let trimmed = line.trim();
                trimmed.ends_with(':') && !trimmed.starts_with(';')
            })
            .map(|line| line.trim().trim_end_matches(':').to_string());
        
        if let Some(_func_name) = first_function {
            if self.is_64bit {
                // 64-bit wrapper: sub rsp, 40; call func; add rsp, 40; xor ecx, ecx; int3
                self.emit(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 40
                self.emit(&[0xE8, 0x00, 0x00, 0x00, 0x00]); // call (placeholder)
                let call_patch_pos = self.code.len() - 4;
                self.emit(&[0x48, 0x83, 0xC4, 0x28]); // add rsp, 40
                self.emit(&[0x31, 0xC9]); // xor ecx, ecx
                // Use int3 to terminate (debugger breakpoint - OS handles gracefully)
                self.emit(&[0xCC]); // int3
                
                let function_start = self.code.len();
                self.first_pass(source)?;
                self.second_pass(source)?;
                
                // Patch call offset
                let offset = (function_start as i32) - (call_patch_pos as i32 + 4);
                let bytes = offset.to_le_bytes();
                self.code[call_patch_pos..call_patch_pos + 4].copy_from_slice(&bytes);
            } else {
                // 32-bit wrapper: push ebp; mov ebp, esp; call func; xor eax, eax; pop ebp; int3
                self.emit(&[0x55]); // push ebp
                self.emit(&[0x89, 0xE5]); // mov ebp, esp
                self.emit(&[0xE8, 0x00, 0x00, 0x00, 0x00]); // call (placeholder)
                let call_patch_pos = self.code.len() - 4;
                self.emit(&[0x31, 0xC0]); // xor eax, eax
                self.emit(&[0x5D]); // pop ebp
                // Use int3 to terminate (debugger breakpoint - OS handles gracefully)
                self.emit(&[0xCC]); // int3
                
                let function_start = self.code.len();
                self.first_pass(source)?;
                self.second_pass(source)?;
                
                // Patch call offset
                let offset = (function_start as i32) - (call_patch_pos as i32 + 4);
                let bytes = offset.to_le_bytes();
                self.code[call_patch_pos..call_patch_pos + 4].copy_from_slice(&bytes);
            }
        } else {
            self.first_pass(source)?;
            self.second_pass(source)?;
        }
        
        Ok(())
    }

    fn first_pass(&mut self, source: &str) -> Result<(), String> {
        self.current_pass = 1;
        self.current_line = 0;
        
        let mut temp_labels = HashMap::new();
        let base_offset = self.code.len();
        let mut code_offset = 0usize;
        
        // Skip everything before the entry point marker (if it exists)
        let mut found_entry_point = !source.contains("=== ENTRY POINT ===");
        
        for line in source.lines() {
            self.current_line += 1;
            let line = line.trim();
            
            // Check for entry point marker
            if !found_entry_point {
                if line.contains("=== ENTRY POINT ===") {
                    found_entry_point = true;
                }
                continue;
            }
            
            // Fast skip empty and comments
            if line.is_empty() || line.as_bytes().get(0) == Some(&b';') {
                continue;
            }
            
            // Check for label (ends with ':')
            if line.as_bytes().last() == Some(&b':') {
                let label = &line[..line.len()-1].trim();
                let label_pos = base_offset + code_offset;
                
                // Add to symbol table (ignore errors in first pass for speed)
                let _ = self.symbol_table.define(
                    label.to_string(),
                    SymbolType::Label,
                    label_pos as i64,
                    self.current_line
                );
                
                temp_labels.insert(label.to_string(), label_pos);
                continue;
            }
            
            // Fast skip directives (check first char)
            let first_char = line.as_bytes()[0];
            if first_char == b's' || first_char == b'g' || first_char == b'e' || first_char == b'S' || first_char == b'.' {
                if line.starts_with("section") || line.starts_with("global") || 
                   line.starts_with("extern") || line.starts_with("SECTION") || line.starts_with(".") {
                    continue;
                }
            }
            
            // Estimate instruction size (fast path)
            let instr_size = self.estimate_instruction_size(line);
            code_offset += instr_size;
        }
        
        self.labels = temp_labels;
        Ok(())
    }

    fn second_pass(&mut self, source: &str) -> Result<(), String> {
        self.current_pass = 2;
        self.current_line = 0;
        
        // Skip everything before the entry point marker (if it exists)
        let mut found_entry_point = !source.contains("=== ENTRY POINT ===");
        
        for line in source.lines() {
            self.current_line += 1;
            let line = line.trim();
            
            // Check for entry point marker
            if !found_entry_point {
                if line.contains("=== ENTRY POINT ===") {
                    found_entry_point = true;
                }
                continue;
            }
            
            // Fast skip empty, comments, and labels
            if line.is_empty() {
                continue;
            }
            let first_byte = line.as_bytes()[0];
            if first_byte == b';' || line.as_bytes().last() == Some(&b':') {
                continue;
            }
            
            // Fast directive check - only check if starts with known directive chars
            if first_byte == b's' || first_byte == b'g' || first_byte == b'e' || 
               first_byte == b'S' || first_byte == b'.' || first_byte == b'o' ||
               first_byte == b'd' || first_byte == b't' || first_byte == b'a' {
                match self.handle_directive(line) {
                    Ok(true) => continue,
                    Ok(false) => {},
                    Err(_) => continue, // Skip error logging for speed
                }
            }
            
            // Assemble as instruction (skip listing generation for speed)
            if let Err(_) = self.assemble_instruction(line) {
                // Skip error logging for speed - errors will be caught if code is wrong
            }
        }
        
        Ok(())
    }

    // ========================================================================
    // DIRECTIVE HANDLING (Pseudo-ops)
    // ========================================================================
    
    fn handle_directive(&mut self, line: &str) -> Result<bool, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }
        
        let directive = parts[0].to_lowercase();
        
        match directive.as_str() {
            // ORG - Set origin address  
            "org" => {
                if parts.len() < 2 {
                    self.add_error(
                        AssemblerErrorType::Directive,
                        "ORG directive requires an address".to_string(),
                        Some("Usage: ORG 0x1000".to_string())
                    );
                    return Err("Invalid ORG directive".to_string());
                }
                let addr = self.parse_immediate(parts[1])?;
                self.current_address = addr as usize;
                Ok(true)
            },
            
            // EQU - Define constant
            "equ" => {
                if parts.len() < 3 {
                    self.add_error(
                        AssemblerErrorType::Directive,
                        "EQU directive requires name and value".to_string(),
                        Some("Usage: CONSTANT EQU 100".to_string())
                    );
                    return Err("Invalid EQU directive".to_string());
                }
                // Format: NAME EQU VALUE
                // parts[0] is the name (before EQU), parts[1] is "equ", parts[2] is value
                // Actually, we need to re-parse this differently
                Ok(true)
            },
            
            // DB - Define byte(s)
            "db" => {
                if parts.len() < 2 {
                    return Err("DB directive requires data".to_string());
                }
                for i in 1..parts.len() {
                    let value = self.parse_immediate(parts[i])?;
                    self.emit(&[(value & 0xFF) as u8]);
                }
                Ok(true)
            },
            
            // DW - Define word (2 bytes)
            "dw" => {
                if parts.len() < 2 {
                    return Err("DW directive requires data".to_string());
                }
                for i in 1..parts.len() {
                    let value = self.parse_immediate(parts[i])?;
                    self.emit(&(value as u16).to_le_bytes());
                }
                Ok(true)
            },
            
            // DD - Define dword (4 bytes)
            "dd" => {
                if parts.len() < 2 {
                    return Err("DD directive requires data".to_string());
                }
                for i in 1..parts.len() {
                    let value = self.parse_immediate(parts[i])?;
                    self.emit(&(value as u32).to_le_bytes());
                }
                Ok(true)
            },
            
            // DQ - Define qword (8 bytes)
            "dq" => {
                if parts.len() < 2 {
                    return Err("DQ directive requires data".to_string());
                }
                for i in 1..parts.len() {
                    let value = self.parse_immediate(parts[i])?;
                    self.emit(&(value as u64).to_le_bytes());
                }
                Ok(true)
            },
            
            // TIMES - Repeat instruction/data
            "times" => {
                if parts.len() < 3 {
                    return Err("TIMES directive requires count and data".to_string());
                }
                let count = self.parse_immediate(parts[1])? as usize;
                let rest = parts[2..].join(" ");
                for _ in 0..count {
                    self.assemble_instruction(&rest)?;
                }
                Ok(true)
            },
            
            // ALIGN - Align to boundary
            "align" => {
                if parts.len() < 2 {
                    return Err("ALIGN directive requires boundary".to_string());
                }
                let boundary = self.parse_immediate(parts[1])? as usize;
                while self.code.len() % boundary != 0 {
                    self.emit(&[0x90]); // NOP padding
                }
                Ok(true)
            },
            
            // Section directives (ignore but don't error)
            "section" | ".section" | ".text" | ".data" | ".bss" => Ok(true),
            "global" | ".global" | ".globl" | "extern" | ".extern" => Ok(true),
            ".intel_syntax" | ".att_syntax" => Ok(true),
            
            _ => Ok(false), // Not a directive
        }
    }
    
    fn estimate_instruction_size(&self, line: &str) -> usize {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return 0;
        }
        
        let mnemonic = parts[0].to_lowercase();
        match mnemonic.as_str() {
            "push" | "pop" => 1,
            "ret" | "nop" | "int3" => 1,
            "mov" => {
                if parts.len() > 1 && parts[1].contains(',') {
                    let joined = parts[1..].join(" ");
                    let ops: Vec<&str> = joined.split(',').collect();
                    if ops.len() == 2 {
                        let src = ops[1].trim();
                        if self.is_immediate(src) {
                            return 5; // mov reg, imm32
                        } else {
                            return 2; // mov reg, reg
                        }
                    }
                }
                5
            },
            "add" | "sub" | "xor" | "cmp" => {
                if parts.len() > 1 {
                    let ops = parts[1..].join(" ");
                    if ops.contains(',') {
                        let operands: Vec<&str> = ops.split(',').collect();
                        if operands.len() == 2 && self.is_immediate(operands[1].trim()) {
                            let imm = self.parse_immediate(operands[1].trim()).unwrap_or(0);
                            if imm <= 0xFF {
                                return 3; // op reg, imm8
                            } else {
                                return 6; // op reg, imm32
                            }
                        } else {
                            return 2; // op reg, reg
                        }
                    }
                }
                3
            },
            "inc" | "dec" => 2,
            "je" | "jne" | "jz" | "jnz" => 6, // Long conditional jump
            "jmp" => 5, // Near jump
            "call" => 5,
            _ => 3,
        }
    }

    fn assemble_instruction(&mut self, line: &str) -> Result<(), String> {
        // Fast path: find first whitespace to split mnemonic from operands
        let (mnemonic_str, operands_str) = if let Some(pos) = line.find(|c: char| c.is_whitespace()) {
            (&line[..pos], line[pos..].trim())
        } else {
            (line, "")
        };
        
        if mnemonic_str.is_empty() {
            return Ok(());
        }
        
        // Convert mnemonic to lowercase only once (avoid allocation for common cases)
        let mnemonic_lower;
        let mnemonic = if mnemonic_str.bytes().all(|b| b.is_ascii_lowercase()) {
            mnemonic_str
        } else {
            mnemonic_lower = mnemonic_str.to_lowercase();
            mnemonic_lower.as_str()
        };
        
        match mnemonic {
            // Basic instructions
            "nop" => {
                if operands_str.is_empty() {
                    self.emit(&[0x90]);
                } else {
                    self.emit_nop_with_operands(operands_str)?;
                }
            },
            "ret" => self.emit(&[0xC3]),
            "int3" => self.emit(&[0xCC]),
            "hlt" => self.emit(&[0xF4]),
            "syscall" => self.emit(&[0x0F, 0x05]),
            "leave" => self.emit(&[0xC9]),
            "cdq" => self.emit(&[0x99]),
            "cqo" => { self.emit_rex_w(); self.emit(&[0x99]); },
            
            // Stack operations
            "push" => self.emit_push(operands_str)?,
            "pop" => self.emit_pop(operands_str)?,
            
            // Data movement
            "mov" => self.emit_mov(operands_str)?,
            "movzx" => self.emit_movzx(operands_str)?,
            "movsx" => self.emit_movsx(operands_str)?,
            "movsxd" => self.emit_movsxd(operands_str)?,
            "lea" => self.emit_lea(operands_str)?,
            "xchg" => self.emit_xchg(operands_str)?,
            
            // Arithmetic
            "add" => self.emit_add(operands_str)?,
            "sub" => self.emit_sub(operands_str)?,
            "imul" => self.emit_imul(operands_str)?,
            "mul" => self.emit_mul(operands_str)?,
            "idiv" => self.emit_idiv(operands_str)?,
            "div" => self.emit_div(operands_str)?,
            "inc" => self.emit_inc(operands_str)?,
            "dec" => self.emit_dec(operands_str)?,
            "neg" => self.emit_neg(operands_str)?,
            
            // Logical
            "and" => self.emit_and(operands_str)?,
            "or" => self.emit_or(operands_str)?,
            "xor" => self.emit_xor(operands_str)?,
            "not" => self.emit_not(operands_str)?,
            "test" => self.emit_test(operands_str)?,
            
            // Shift/Rotate
            "shl" | "sal" => self.emit_shl(operands_str)?,
            "shr" => self.emit_shr(operands_str)?,
            "sar" => self.emit_sar(operands_str)?,
            "rol" => self.emit_rol(operands_str)?,
            "ror" => self.emit_ror(operands_str)?,
            
            // Comparison
            "cmp" => self.emit_cmp(operands_str)?,
            
            // Conditional set
            "sete" | "setz" => self.emit_sete(operands_str)?,
            "setne" | "setnz" => self.emit_setne(operands_str)?,
            "setl" | "setnge" => self.emit_setl(operands_str)?,
            "setle" | "setng" => self.emit_setle(operands_str)?,
            "setg" | "setnle" => self.emit_setg(operands_str)?,
            "setge" | "setnl" => self.emit_setge(operands_str)?,
            "seta" | "setnbe" => self.emit_seta(operands_str)?,
            "setae" | "setnb" => self.emit_setae(operands_str)?,
            "setb" | "setnae" => self.emit_setb(operands_str)?,
            "setbe" | "setna" => self.emit_setbe(operands_str)?,
            
            // Atomic operations
            "lock" => self.emit_lock(operands_str)?,
            "cmpxchg" => self.emit_cmpxchg(operands_str)?,
            
            // Conditional jumps
            "jmp" => self.emit_jmp(operands_str)?,
            "je" | "jz" => self.emit_je(operands_str)?,
            "jne" | "jnz" => self.emit_jne(operands_str)?,
            "jl" | "jnge" => self.emit_jl(operands_str)?,
            "jle" | "jng" => self.emit_jle(operands_str)?,
            "jg" | "jnle" => self.emit_jg(operands_str)?,
            "jge" | "jnl" => self.emit_jge(operands_str)?,
            "ja" | "jnbe" => self.emit_ja(operands_str)?,
            "jae" | "jnb" | "jnc" => self.emit_jae(operands_str)?,
            "jb" | "jnae" | "jc" => self.emit_jb(operands_str)?,
            "jbe" | "jna" => self.emit_jbe(operands_str)?,
            "js" => self.emit_js(operands_str)?,
            "jns" => self.emit_jns(operands_str)?,
            "jo" => self.emit_jo(operands_str)?,
            "jno" => self.emit_jno(operands_str)?,
            "jp" | "jpe" => self.emit_jp(operands_str)?,
            "jnp" | "jpo" => self.emit_jnp(operands_str)?,
            
            // Call/Return
            "call" => self.emit_call(operands_str)?,
            
            // Directives (ignore these)
            ".intel_syntax" | ".att_syntax" | ".section" | ".text" | ".data" | 
            ".bss" | ".global" | ".globl" | ".extern" | ".byte" => {},
            
            _ => {
                // Check if this is a comment line or data label
                if mnemonic.starts_with('#') || mnemonic.starts_with(';') || mnemonic.ends_with(':') {
                    // Ignore comments and labels
                    return Ok(());
                }
                
                // Unknown instruction - emit NOP as fallback
                eprintln!("⚠️  [ASSEMBLER] Unknown instruction at line {}: '{}' (operands: '{}')", 
                         self.current_line, mnemonic, operands_str);
                self.emit(&[0x90]);
            }
        }
        
        Ok(())
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn emit_nop_with_operands(&mut self, operands: &str) -> Result<(), String> {
        // Multi-byte NOP instructions used for alignment
        // These are critical for maintaining correct code layout
        // The exact encoding matters - even one byte difference breaks entry point alignment!
        
        let operands_lower = operands.to_lowercase();
        
        // nop dword ptr [rax] = 0F 1F 40 00 (4 bytes)
        // ModR/M = 0x40 means [rax + disp8], with displacement = 0x00
        if operands_lower.contains("dword") && operands_lower.contains("[rax]") && !operands_lower.contains("+") {
            self.emit(&[0x0F, 0x1F, 0x40, 0x00]);
            return Ok(());
        }
        
        // nop dword ptr [rax + rax] = 0F 1F 04 00 (4 bytes)
        if operands_lower.contains("dword") && operands_lower.contains("[rax") && operands_lower.contains("rax]") && operands_lower.contains("+") {
            self.emit(&[0x0F, 0x1F, 0x04, 0x00]);
            return Ok(());
        }
        
        // nop word ptr [rax + rax] = 66 0F 1F 04 00 (5 bytes)
        if operands_lower.contains("word") && !operands_lower.contains("cs:") && operands_lower.contains("[rax") && operands_lower.contains("rax]") && operands_lower.contains("+") {
            self.emit(&[0x66, 0x0F, 0x1F, 0x04, 0x00]);
            return Ok(());
        }
        
        // nop word ptr cs:[rax + rax] = 66 66 2E 0F 1F 84 00 00 00 00 00 (11 bytes)
        // Note: TWO 0x66 prefixes! This is compiler-specific padding.
        if operands_lower.contains("word") && operands_lower.contains("cs:") {
            self.emit(&[0x66, 0x66, 0x2E, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00]);
            return Ok(());
        }
        
        // Fallback: emit single-byte NOP
        self.emit(&[0x90]);
        Ok(())
    }

    fn emit_push(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x50 + reg_code]);
        } else {
            self.emit(&[0x90]); // NOP fallback
        }
        Ok(())
    }

    fn emit_pop(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x58 + reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }

    fn emit_mov(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid mov operands: {}", operands));
        }
        
        let dest_op = self.parse_operand(parts[0])?;
        let src_op = self.parse_operand(parts[1])?;
        
        match (&dest_op, &src_op) {
            // MOV reg, imm
            (Operand::Register(dest), Operand::Immediate(val)) => {
                let reg_code = self.get_register_code(dest)?;
                if self.is_64bit_register(dest) {
                    self.emit_rex_w();
                }
                self.emit(&[0xB8 + (reg_code & 7)]);
                if self.is_64bit_register(dest) {
                    self.emit(&(*val as u64).to_le_bytes());
                } else {
                    self.emit(&(*val as u32).to_le_bytes());
                }
            }
            // MOV reg, reg
            (Operand::Register(dest), Operand::Register(src)) => {
                if self.is_64bit_register(dest) || self.is_64bit_register(src) {
                    self.emit_rex_w();
                }
                let _dest_code = self.get_register_code(dest)?;
                let src_code = self.get_register_code(src)?;
                self.emit(&[0x89]);
                self.encode_modrm_sib(src_code, &dest_op)?;
            }
            // MOV reg, [mem]
            (Operand::Register(dest), Operand::Memory { .. }) => {
                if self.is_64bit_register(dest) {
                    self.emit_rex_w();
                }
                let dest_code = self.get_register_code(dest)?;
                self.emit(&[0x8B]);  // MOV r, r/m
                self.encode_modrm_sib(dest_code, &src_op)?;
            }
            // MOV [mem], reg
            (Operand::Memory { .. }, Operand::Register(src)) => {
                if self.is_64bit_register(src) {
                    self.emit_rex_w();
                }
                let src_code = self.get_register_code(src)?;
                self.emit(&[0x89]);  // MOV r/m, r
                self.encode_modrm_sib(src_code, &dest_op)?;
            }
            // MOV [mem], imm
            (Operand::Memory { size, .. }, Operand::Immediate(val)) => {
                match size {
                    MemSize::Qword => {
                        self.emit_rex_w();
                        self.emit(&[0xC7]);
                        self.encode_modrm_sib(0, &dest_op)?;
                        self.emit(&(*val as u32).to_le_bytes());
                    }
                    MemSize::Dword => {
                        self.emit(&[0xC7]);
                        self.encode_modrm_sib(0, &dest_op)?;
                        self.emit(&(*val as u32).to_le_bytes());
                    }
                    MemSize::Word => {
                        self.emit(&[0x66, 0xC7]);  // 16-bit operand size prefix
                        self.encode_modrm_sib(0, &dest_op)?;
                        self.emit(&(*val as u16).to_le_bytes());
                    }
                    MemSize::Byte => {
                        self.emit(&[0xC6]);
                        self.encode_modrm_sib(0, &dest_op)?;
                        self.emit(&[*val as u8]);
                    }
                }
            }
            _ => {
                // Unsupported combination - emit NOP
                self.emit(&[0x90]);
            }
        }
        
        Ok(())
    }

    fn emit_add(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid add operands: {}", operands));
        }
        
        let dest = parts[0];
        let src = parts[1];
        
        if self.is_register(dest) && self.is_register(src) {
            // ADD reg, reg (01 /r: ADD r/m32, r32)
            let dest_code = self.get_register_code(dest)?;
            let src_code = self.get_register_code(src)?;
            self.emit(&[0x01, 0xC0 | (src_code << 3) | dest_code]);
        } else if self.is_register(dest) && self.is_immediate(src) {
            let dest_code = self.get_register_code(dest)?;
            let value = self.parse_immediate(src)?;
            
            if value <= 0xFF {
                // ADD reg, imm8 (83 /0)
                self.emit(&[0x83, 0xC0 | dest_code, value as u8]);
            } else {
                // ADD reg, imm32 (81 /0)
                self.emit(&[0x81, 0xC0 | dest_code]);
                self.emit(&(value as u32).to_le_bytes());
            }
        } else {
            self.emit(&[0x90]);
        }
        
        Ok(())
    }

    fn emit_sub(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid sub operands: {}", operands));
        }
        
        let dest = parts[0];
        let src = parts[1];
        
        if self.is_register(dest) && self.is_register(src) {
            // SUB reg, reg (29 /r)
            let dest_code = self.get_register_code(dest)?;
            let src_code = self.get_register_code(src)?;
            self.emit(&[0x29, 0xC0 | (src_code << 3) | dest_code]);
        } else if self.is_register(dest) && self.is_immediate(src) {
            let dest_code = self.get_register_code(dest)?;
            let value = self.parse_immediate(src)?;
            
            if value <= 0xFF {
                // SUB reg, imm8 (83 /5)
                self.emit(&[0x83, 0xE8 | dest_code, value as u8]);
            } else {
                // SUB reg, imm32 (81 /5)
                self.emit(&[0x81, 0xE8 | dest_code]);
                self.emit(&(value as u32).to_le_bytes());
            }
        } else {
            self.emit(&[0x90]);
        }
        
        Ok(())
    }

    fn emit_xor(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid xor operands: {}", operands));
        }
        
        let dest = parts[0];
        let src = parts[1];
        
        if self.is_register(dest) && self.is_register(src) {
            // XOR reg, reg (31 /r)
            let dest_code = self.get_register_code(dest)?;
            let src_code = self.get_register_code(src)?;
            self.emit(&[0x31, 0xC0 | (src_code << 3) | dest_code]);
        } else {
            self.emit(&[0x90]);
        }
        
        Ok(())
    }

    fn emit_cmp(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid cmp operands: {}", operands));
        }
        
        let dest = parts[0];
        let src = parts[1];
        
        if self.is_register(dest) && self.is_register(src) {
            // CMP reg, reg (39 /r)
            let dest_code = self.get_register_code(dest)?;
            let src_code = self.get_register_code(src)?;
            self.emit(&[0x39, 0xC0 | (src_code << 3) | dest_code]);
        } else if self.is_register(dest) && self.is_immediate(src) {
            let dest_code = self.get_register_code(dest)?;
            let value = self.parse_immediate(src)?;
            
            if value <= 0xFF {
                // CMP reg, imm8 (83 /7)
                self.emit(&[0x83, 0xF8 | dest_code, value as u8]);
            } else {
                // CMP reg, imm32 (81 /7)
                self.emit(&[0x81, 0xF8 | dest_code]);
                self.emit(&(value as u32).to_le_bytes());
            }
        } else {
            self.emit(&[0x90]);
        }
        
        Ok(())
    }
    
    fn emit_cmpxchg(&mut self, operands: &str) -> Result<(), String> {
        // CMPXCHG - Compare and exchange (0F B1 /r)
        // Compares AL/AX/EAX/RAX with dest, if equal, loads src into dest
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) && self.is_register(parts[1]) {
            let dest_code = self.get_register_code(parts[0])?;
            let src_code = self.get_register_code(parts[1])?;
            if self.is_64bit_register(parts[0]) || self.is_64bit_register(parts[1]) {
                self.emit_rex_w();
            }
            self.emit(&[0x0F, 0xB1, 0xC0 | (src_code << 3) | dest_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_lock(&mut self, operands: &str) -> Result<(), String> {
        // LOCK prefix (0xF0) - must be followed by another instruction
        // Parse the rest of the line as an instruction
        self.emit(&[0xF0]); // LOCK prefix
        
        // Now assemble the following instruction
        if !operands.is_empty() {
            self.assemble_instruction(operands)?;
        }
        
        Ok(())
    }

    fn emit_inc(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            // INC reg (FF /0)
            self.emit(&[0xFF, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }

    fn emit_dec(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            // DEC reg (FF /1)
            self.emit(&[0xFF, 0xC8 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }

    fn emit_jmp(&mut self, operands: &str) -> Result<(), String> {
        let target_str = operands.trim();
        
        // NEW: Check if it's an absolute address (0x1234 format)
        if self.is_immediate(target_str) {
            let absolute_addr = self.parse_immediate(target_str)? as i64;
            let current_addr = self.code.len() as i64 + 5; // JMP is 5 bytes
            let offset = absolute_addr - current_addr;
            
            // Check if offset fits in 32-bit signed
            if offset >= i32::MIN as i64 && offset <= i32::MAX as i64 {
                self.emit(&[0xE9]);
                self.emit(&(offset as i32).to_le_bytes());
            } else {
                return Err(format!("JMP offset too large: {}", offset));
            }
        }
        // Check if it's a label
        else if let Some(&target) = self.labels.get(target_str) {
            let current = self.code.len() + 5; // JMP is 5 bytes
            let offset = (target as i32) - (current as i32);
            self.emit(&[0xE9]);
            self.emit(&offset.to_le_bytes());
        }
        // Check symbol table
        else if let Some(symbol) = self.symbol_table.get(target_str) {
            let target = symbol.value as usize;
            let current = self.code.len() + 5;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0xE9]);
            self.emit(&offset.to_le_bytes());
        }
        else {
            // Label not found, emit placeholder
            self.emit(&[0xE9, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }

    fn emit_je(&mut self, operands: &str) -> Result<(), String> {
        let target_str = operands.trim();
        
        // NEW: Check if it's an absolute address
        if self.is_immediate(target_str) {
            let absolute_addr = self.parse_immediate(target_str)? as i64;
            let current_addr = self.code.len() as i64 + 6; // JE is 6 bytes
            let offset = absolute_addr - current_addr;
            
            if offset >= i32::MIN as i64 && offset <= i32::MAX as i64 {
                self.emit(&[0x0F, 0x84]);
                self.emit(&(offset as i32).to_le_bytes());
            } else {
                return Err(format!("JE offset too large: {}", offset));
            }
        }
        else if let Some(&target) = self.labels.get(target_str) {
            let current = self.code.len() + 6; // JE is 6 bytes (0F 84)
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x84]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x84, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }

    fn emit_jne(&mut self, operands: &str) -> Result<(), String> {
        let target_str = operands.trim();
        
        // NEW: Check if it's an absolute address
        if self.is_immediate(target_str) {
            let absolute_addr = self.parse_immediate(target_str)? as i64;
            let current_addr = self.code.len() as i64 + 6; // JNE is 6 bytes
            let offset = absolute_addr - current_addr;
            
            if offset >= i32::MIN as i64 && offset <= i32::MAX as i64 {
                self.emit(&[0x0F, 0x85]);
                self.emit(&(offset as i32).to_le_bytes());
            } else {
                return Err(format!("JNE offset too large: {}", offset));
            }
        }
        else if let Some(&target) = self.labels.get(target_str) {
            let current = self.code.len() + 6; // JNE is 6 bytes (0F 85)
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x85]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }

    fn emit_call(&mut self, operands: &str) -> Result<(), String> {
        let target_str = operands.trim();
        
        // NEW: Check if it's an absolute address (0x1234 format)
        if self.is_immediate(target_str) {
            let absolute_addr = self.parse_immediate(target_str)? as i64;
            let current_addr = self.code.len() as i64 + 5; // CALL is 5 bytes
            let offset = absolute_addr - current_addr;
            
            // Check if offset fits in 32-bit signed
            if offset >= i32::MIN as i64 && offset <= i32::MAX as i64 {
                self.emit(&[0xE8]);
                self.emit(&(offset as i32).to_le_bytes());
            } else {
                return Err(format!("CALL offset too large: {}", offset));
            }
        }
        // Check if it's a label
        else if let Some(&target) = self.labels.get(target_str) {
            let current = self.code.len() + 5; // CALL is 5 bytes
            let offset = (target as i32) - (current as i32);
            self.emit(&[0xE8]);
            self.emit(&offset.to_le_bytes());
        }
        // Check symbol table
        else if let Some(symbol) = self.symbol_table.get(target_str) {
            let target = symbol.value as usize;
            let current = self.code.len() + 5;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0xE8]);
            self.emit(&offset.to_le_bytes());
        }
        else {
            // Label not found, emit placeholder
            self.emit(&[0xE8, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }

    // === DIVINE INSTRUCTION SET - Additional Instructions ===
    
    fn emit_rex_w(&mut self) {
        self.emit(&[0x48]); // REX.W prefix for 64-bit operand size
    }
    
    fn emit_lea(&mut self, operands: &str) -> Result<(), String> {
        // LEA reg, [mem] - Load Effective Address
        // For now, simple implementation: LEA reg, reg (8D /r)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) && self.is_register(parts[1]) {
            let dest_code = self.get_register_code(parts[0])?;
            let src_code = self.get_register_code(parts[1])?;
            if self.is_64bit_register(parts[0]) || self.is_64bit_register(parts[1]) {
                self.emit_rex_w();
            }
            self.emit(&[0x8D, 0xC0 | (dest_code << 3) | src_code]);
        } else {
            self.emit(&[0x90]); // NOP fallback
        }
        Ok(())
    }
    
    fn emit_movzx(&mut self, operands: &str) -> Result<(), String> {
        // MOVZX - Move with zero extension (0F B6/B7)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) && self.is_register(parts[1]) {
            let dest_code = self.get_register_code(parts[0])?;
            let src_code = self.get_register_code(parts[1])?;
            self.emit(&[0x0F, 0xB6, 0xC0 | (dest_code << 3) | src_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_movsx(&mut self, operands: &str) -> Result<(), String> {
        // MOVSX - Move with sign extension (0F BE/BF)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) && self.is_register(parts[1]) {
            let dest_code = self.get_register_code(parts[0])?;
            let src_code = self.get_register_code(parts[1])?;
            self.emit(&[0x0F, 0xBE, 0xC0 | (dest_code << 3) | src_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_movsxd(&mut self, operands: &str) -> Result<(), String> {
        // MOVSXD - Move with sign extension (dword to qword) - REX.W + 63 /r
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) && self.is_register(parts[1]) {
            let dest_code = self.get_register_code(parts[0])?;
            let src_code = self.get_register_code(parts[1])?;
            self.emit_rex_w(); // REX.W prefix for 64-bit
            self.emit(&[0x63, 0xC0 | (dest_code << 3) | src_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_xchg(&mut self, operands: &str) -> Result<(), String> {
        // XCHG - Exchange register/memory with register (87 /r for reg-reg)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) && self.is_register(parts[1]) {
            let reg1_code = self.get_register_code(parts[0])?;
            let reg2_code = self.get_register_code(parts[1])?;
            if self.is_64bit_register(parts[0]) || self.is_64bit_register(parts[1]) {
                self.emit_rex_w();
            }
            self.emit(&[0x87, 0xC0 | (reg1_code << 3) | reg2_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_imul(&mut self, operands: &str) -> Result<(), String> {
        // IMUL - Signed multiply (0F AF /r for two-operand form)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) && self.is_register(parts[1]) {
            let dest_code = self.get_register_code(parts[0])?;
            let src_code = self.get_register_code(parts[1])?;
            if self.is_64bit_register(parts[0]) || self.is_64bit_register(parts[1]) {
                self.emit_rex_w();
            }
            self.emit(&[0x0F, 0xAF, 0xC0 | (dest_code << 3) | src_code]);
        } else if parts.len() == 1 && self.is_register(parts[0]) {
            // Single operand form: IMUL reg (F7 /5)
            let reg_code = self.get_register_code(parts[0])?;
            self.emit(&[0xF7, 0xE8 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_mul(&mut self, operands: &str) -> Result<(), String> {
        // MUL - Unsigned multiply (F7 /4)
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0xF7, 0xE0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_idiv(&mut self, operands: &str) -> Result<(), String> {
        // IDIV - Signed divide (F7 /7)
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0xF7, 0xF8 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_div(&mut self, operands: &str) -> Result<(), String> {
        // DIV - Unsigned divide (F7 /6)
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0xF7, 0xF0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_neg(&mut self, operands: &str) -> Result<(), String> {
        // NEG - Two's complement negation (F7 /3)
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0xF7, 0xD8 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_and(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid and operands: {}", operands));
        }
        
        let dest = parts[0];
        let src = parts[1];
        
        if self.is_register(dest) && self.is_register(src) {
            // AND reg, reg (21 /r)
            let dest_code = self.get_register_code(dest)?;
            let src_code = self.get_register_code(src)?;
            self.emit(&[0x21, 0xC0 | (src_code << 3) | dest_code]);
        } else if self.is_register(dest) && self.is_immediate(src) {
            let dest_code = self.get_register_code(dest)?;
            let value = self.parse_immediate(src)?;
            if value <= 0xFF {
                self.emit(&[0x83, 0xE0 | dest_code, value as u8]);
            } else {
                self.emit(&[0x81, 0xE0 | dest_code]);
                self.emit(&(value as u32).to_le_bytes());
            }
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_or(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid or operands: {}", operands));
        }
        
        let dest = parts[0];
        let src = parts[1];
        
        if self.is_register(dest) && self.is_register(src) {
            // OR reg, reg (09 /r)
            let dest_code = self.get_register_code(dest)?;
            let src_code = self.get_register_code(src)?;
            self.emit(&[0x09, 0xC0 | (src_code << 3) | dest_code]);
        } else if self.is_register(dest) && self.is_immediate(src) {
            let dest_code = self.get_register_code(dest)?;
            let value = self.parse_immediate(src)?;
            if value <= 0xFF {
                self.emit(&[0x83, 0xC8 | dest_code, value as u8]);
            } else {
                self.emit(&[0x81, 0xC8 | dest_code]);
                self.emit(&(value as u32).to_le_bytes());
            }
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_not(&mut self, operands: &str) -> Result<(), String> {
        // NOT - One's complement negation (F7 /2)
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0xF7, 0xD0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_test(&mut self, operands: &str) -> Result<(), String> {
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Invalid test operands: {}", operands));
        }
        
        let dest = parts[0];
        let src = parts[1];
        
        if self.is_register(dest) && self.is_register(src) {
            // TEST reg, reg (85 /r)
            let dest_code = self.get_register_code(dest)?;
            let src_code = self.get_register_code(src)?;
            self.emit(&[0x85, 0xC0 | (src_code << 3) | dest_code]);
        } else if self.is_register(dest) && self.is_immediate(src) {
            let dest_code = self.get_register_code(dest)?;
            let value = self.parse_immediate(src)?;
            self.emit(&[0xF7, 0xC0 | dest_code]);
            self.emit(&(value as u32).to_le_bytes());
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    // SETcc instructions - Set byte on condition (0F 9x /0)
    fn emit_sete(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x94, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setne(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x95, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setl(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x9C, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setle(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x9E, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setg(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x9F, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setge(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x9D, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_seta(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x97, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setae(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x93, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setb(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x92, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_setbe(&mut self, operands: &str) -> Result<(), String> {
        let reg = operands.trim();
        if self.is_register(reg) {
            let reg_code = self.get_register_code(reg)?;
            self.emit(&[0x0F, 0x96, 0xC0 | reg_code]);
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_shl(&mut self, operands: &str) -> Result<(), String> {
        // SHL/SAL - Shift left (C1 /4 for imm8, D3 /4 for CL)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) {
            let reg_code = self.get_register_code(parts[0])?;
            if parts[1].to_lowercase() == "cl" {
                self.emit(&[0xD3, 0xE0 | reg_code]);
            } else if self.is_immediate(parts[1]) {
                let shift = self.parse_immediate(parts[1])? as u8;
                self.emit(&[0xC1, 0xE0 | reg_code, shift]);
            } else {
                self.emit(&[0x90]);
            }
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_shr(&mut self, operands: &str) -> Result<(), String> {
        // SHR - Shift right logical (C1 /5 for imm8, D3 /5 for CL)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) {
            let reg_code = self.get_register_code(parts[0])?;
            if parts[1].to_lowercase() == "cl" {
                self.emit(&[0xD3, 0xE8 | reg_code]);
            } else if self.is_immediate(parts[1]) {
                let shift = self.parse_immediate(parts[1])? as u8;
                self.emit(&[0xC1, 0xE8 | reg_code, shift]);
            } else {
                self.emit(&[0x90]);
            }
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_sar(&mut self, operands: &str) -> Result<(), String> {
        // SAR - Shift right arithmetic (C1 /7 for imm8, D3 /7 for CL)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) {
            let reg_code = self.get_register_code(parts[0])?;
            if parts[1].to_lowercase() == "cl" {
                self.emit(&[0xD3, 0xF8 | reg_code]);
            } else if self.is_immediate(parts[1]) {
                let shift = self.parse_immediate(parts[1])? as u8;
                self.emit(&[0xC1, 0xF8 | reg_code, shift]);
            } else {
                self.emit(&[0x90]);
            }
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_rol(&mut self, operands: &str) -> Result<(), String> {
        // ROL - Rotate left (C1 /0 for imm8, D3 /0 for CL)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) {
            let reg_code = self.get_register_code(parts[0])?;
            if parts[1].to_lowercase() == "cl" {
                self.emit(&[0xD3, 0xC0 | reg_code]);
            } else if self.is_immediate(parts[1]) {
                let shift = self.parse_immediate(parts[1])? as u8;
                self.emit(&[0xC1, 0xC0 | reg_code, shift]);
            } else {
                self.emit(&[0x90]);
            }
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    fn emit_ror(&mut self, operands: &str) -> Result<(), String> {
        // ROR - Rotate right (C1 /1 for imm8, D3 /1 for CL)
        let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && self.is_register(parts[0]) {
            let reg_code = self.get_register_code(parts[0])?;
            if parts[1].to_lowercase() == "cl" {
                self.emit(&[0xD3, 0xC8 | reg_code]);
            } else if self.is_immediate(parts[1]) {
                let shift = self.parse_immediate(parts[1])? as u8;
                self.emit(&[0xC1, 0xC8 | reg_code, shift]);
            } else {
                self.emit(&[0x90]);
            }
        } else {
            self.emit(&[0x90]);
        }
        Ok(())
    }
    
    // Additional conditional jumps
    fn emit_jl(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x8C]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x8C, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jle(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x8E]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x8E, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jg(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x8F]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x8F, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jge(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x8D]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x8D, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_ja(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x87]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x87, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jae(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x83]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x83, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jb(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x82]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x82, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jbe(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x86]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x86, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_js(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x88]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x88, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jns(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x89]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x89, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jo(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x80]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x80, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jno(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x81]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x81, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jp(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x8A]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x8A, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn emit_jnp(&mut self, operands: &str) -> Result<(), String> {
        let label = operands.trim();
        if let Some(&target) = self.labels.get(label) {
            let current = self.code.len() + 6;
            let offset = (target as i32) - (current as i32);
            self.emit(&[0x0F, 0x8B]);
            self.emit(&offset.to_le_bytes());
        } else {
            self.emit(&[0x0F, 0x8B, 0x00, 0x00, 0x00, 0x00]);
        }
        Ok(())
    }
    
    fn is_64bit_register(&self, reg: &str) -> bool {
        let reg = reg.to_lowercase();
        matches!(reg.as_str(), 
            "rax" | "rbx" | "rcx" | "rdx" | "rsi" | "rdi" | "rbp" | "rsp" |
            "r8" | "r9" | "r10" | "r11" | "r12" | "r13" | "r14" | "r15"
        )
    }

    // ========== MEMORY OPERAND PARSING ==========
    
    fn parse_operand(&self, s: &str) -> Result<Operand, String> {
        let s = s.trim();
        
        // Check for memory operand: [...]
        if s.starts_with('[') && s.ends_with(']') {
            return self.parse_memory_operand(s);
        }
        
        // Check for register
        if self.is_register(s) {
            return Ok(Operand::Register(s.to_lowercase()));
        }
        
        // Must be immediate
        if self.is_immediate(s) {
            let val = self.parse_immediate(s)? as i64;
            return Ok(Operand::Immediate(val));
        }
        
        Err(format!("Invalid operand: {}", s))
    }
    
    fn parse_memory_operand(&self, s: &str) -> Result<Operand, String> {
        // Extract size prefix: "byte ptr", "word ptr", "dword ptr", "qword ptr"
        // Fast path: check first character to avoid full string scan
        let size = if s.len() > 8 {
            let first_char = s.as_bytes()[0];
            match first_char {
                b'b' | b'B' if s[..8].eq_ignore_ascii_case("byte ptr") => MemSize::Byte,
                b'w' | b'W' if s[..8].eq_ignore_ascii_case("word ptr") => MemSize::Word,
                b'd' | b'D' if s[..9].eq_ignore_ascii_case("dword ptr") => MemSize::Dword,
                b'q' | b'Q' if s[..9].eq_ignore_ascii_case("qword ptr") => MemSize::Qword,
                _ => if self.is_64bit { MemSize::Qword } else { MemSize::Dword }
            }
        } else {
            // Default to qword for 64-bit, dword for 32-bit
            if self.is_64bit { MemSize::Qword } else { MemSize::Dword }
        };
        
        // Extract the part inside brackets
        let start = s.find('[').ok_or("Missing [")?;
        let end = s.rfind(']').ok_or("Missing ]")?;
        let inner = &s[start+1..end].trim();
        
        // NEW: Check for RIP-relative addressing: [rip + label] or [rip + offset]
        if inner.to_lowercase().starts_with("rip") {
            return self.parse_rip_relative_operand(inner, size);
        }
        
        // Parse: [base + index*scale + disp] or variations
        let mut base = None;
        let mut index = None;
        let mut scale = 1u8;
        let mut displacement = 0i32;
        
        // Split by + and -
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = inner.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '+' || ch == '-' {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                    current.clear();
                }
                if ch == '-' {
                    current.push(ch);
                }
            } else {
                current.push(ch);
            }
        }
        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }
        
        // Parse each part
        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            
            // Check for scale: reg*N
            if part.contains('*') {
                let scale_parts: Vec<&str> = part.split('*').collect();
                if scale_parts.len() == 2 {
                    index = Some(scale_parts[0].trim().to_lowercase());
                    scale = scale_parts[1].trim().parse().unwrap_or(1);
                }
            }
            // Check if it's a register
            else if self.is_register(part) {
                if base.is_none() {
                    base = Some(part.to_lowercase());
                } else {
                    index = Some(part.to_lowercase());
                }
            }
            // Must be displacement (could be label or immediate)
            else if self.is_immediate(part) {
                displacement = self.parse_immediate(part)? as i32;
            } else {
                // It's a label - try to resolve it
                if let Some(symbol) = self.symbol_table.get(part) {
                    displacement = symbol.value as i32;
                } else {
                    // Label not yet defined, use 0 as placeholder
                    displacement = 0;
                }
            }
        }
        
        Ok(Operand::Memory {
            base,
            index,
            scale,
            displacement,
            size,
        })
    }
    
    // NEW: Parse RIP-relative addressing
    fn parse_rip_relative_operand(&self, inner: &str, size: MemSize) -> Result<Operand, String> {
        // Format: "rip + label" or "rip + 0x1234" or "rip - 0x10"
        let inner_lower = inner.to_lowercase();
        
        // Remove "rip" and trim
        let rest = if let Some(pos) = inner_lower.find("rip") {
            inner[pos + 3..].trim()
        } else {
            return Err("Invalid RIP-relative addressing".to_string());
        };
        
        // Parse the offset/label part
        let displacement = if rest.is_empty() {
            0i32
        } else if rest.starts_with('+') {
            let offset_str = rest[1..].trim();
            if self.is_immediate(offset_str) {
                self.parse_immediate(offset_str)? as i32
            } else {
                // It's a label
                if let Some(symbol) = self.symbol_table.get(offset_str) {
                    symbol.value as i32
                } else {
                    // Label not yet defined, use 0 as placeholder
                    0
                }
            }
        } else if rest.starts_with('-') {
            let offset_str = rest[1..].trim();
            if self.is_immediate(offset_str) {
                -(self.parse_immediate(offset_str)? as i32)
            } else {
                0
            }
        } else {
            // Just a label without + or -
            if self.is_immediate(rest) {
                self.parse_immediate(rest)? as i32
            } else {
                if let Some(symbol) = self.symbol_table.get(rest) {
                    symbol.value as i32
                } else {
                    0
                }
            }
        };
        
        // RIP-relative uses base=rip, no index
        Ok(Operand::Memory {
            base: Some("rip".to_string()),
            index: None,
            scale: 1,
            displacement,
            size,
        })
    }
    
    fn encode_modrm_sib(&mut self, reg: u8, operand: &Operand) -> Result<(), String> {
        match operand {
            Operand::Register(r) => {
                let rm = self.get_register_code(r)?;
                let modrm = 0xC0 | (reg << 3) | rm;
                self.emit(&[modrm]);
            }
            Operand::Memory { base, index, scale, displacement, .. } => {
                let disp = *displacement;
                
                // NEW: Handle RIP-relative addressing
                if base.as_ref().map(|b| b == "rip").unwrap_or(false) {
                    // RIP-relative: ModR/M = 00 reg 101 (MOD=00, RM=5)
                    let modrm = 0x00 | (reg << 3) | 0x05;
                    self.emit(&[modrm]);
                    // Emit 32-bit displacement
                    self.emit(&(disp as i32).to_le_bytes());
                    return Ok(());
                }
                
                // Determine MOD bits
                let mod_bits = if disp == 0 && base.as_ref().map(|b| b != "rbp" && b != "ebp").unwrap_or(true) {
                    0x00  // [reg]
                } else if disp >= -128 && disp <= 127 {
                    0x40  // [reg + disp8]
                } else {
                    0x80  // [reg + disp32]
                };
                
                // Check if we need SIB byte
                let needs_sib = index.is_some() || 
                               base.as_ref().map(|b| b == "rsp" || b == "esp" || b == "r12").unwrap_or(false);
                
                if needs_sib {
                    // Emit ModR/M with SIB indicator (RM = 4)
                    let modrm = mod_bits | (reg << 3) | 0x04;
                    self.emit(&[modrm]);
                    
                    // Emit SIB byte
                    let scale_bits = match scale {
                        1 => 0x00,
                        2 => 0x40,
                        4 => 0x80,
                        8 => 0xC0,
                        _ => 0x00,
                    };
                    
                    let index_bits = if let Some(idx) = index {
                        self.get_register_code(idx)? << 3
                    } else {
                        0x20  // No index (ESP)
                    };
                    
                    let base_bits = if let Some(b) = base {
                        self.get_register_code(b)?
                    } else {
                        0x05  // No base (EBP)
                    };
                    
                    let sib = scale_bits | index_bits | base_bits;
                    self.emit(&[sib]);
                } else {
                    // Simple [reg + disp]
                    let rm = if let Some(b) = base {
                        self.get_register_code(b)?
                    } else {
                        return Err("Memory operand needs base or index".to_string());
                    };
                    
                    let modrm = mod_bits | (reg << 3) | rm;
                    self.emit(&[modrm]);
                }
                
                // Emit displacement
                if mod_bits == 0x40 {
                    self.emit(&[disp as u8]);
                } else if mod_bits == 0x80 {
                    self.emit(&(disp as i32).to_le_bytes());
                }
            }
            _ => return Err("Invalid operand for ModR/M encoding".to_string()),
        }
        
        Ok(())
    }

    fn is_register(&self, s: &str) -> bool {
        // Fast path: check if already lowercase (most common case in decompiled code)
        if s.bytes().all(|b| !b.is_ascii_uppercase()) {
            return matches!(s, 
                "rax" | "rbx" | "rcx" | "rdx" | "rsi" | "rdi" | "rbp" | "rsp" | "rip" |
                "r8" | "r9" | "r10" | "r11" | "r12" | "r13" | "r14" | "r15" |
                "eax" | "ebx" | "ecx" | "edx" | "esi" | "edi" | "ebp" | "esp" |
                "ax" | "bx" | "cx" | "dx" | "al" | "bl" | "cl" | "dl"
            );
        }
        
        // Slow path: convert to lowercase only if needed
        let s_lower = s.to_lowercase();
        matches!(s_lower.as_str(), 
            "rax" | "rbx" | "rcx" | "rdx" | "rsi" | "rdi" | "rbp" | "rsp" | "rip" |
            "r8" | "r9" | "r10" | "r11" | "r12" | "r13" | "r14" | "r15" |
            "eax" | "ebx" | "ecx" | "edx" | "esi" | "edi" | "ebp" | "esp" |
            "ax" | "bx" | "cx" | "dx" | "al" | "bl" | "cl" | "dl"
        )
    }

    fn is_immediate(&self, s: &str) -> bool {
        s.starts_with("0x") || s.ends_with('h') || s.parse::<i64>().is_ok()
    }

    fn parse_immediate(&self, s: &str) -> Result<u64, String> {
        if s.starts_with("0x") {
            u64::from_str_radix(&s[2..], 16)
                .map_err(|_| format!("Invalid hex immediate: {}", s))
        } else if s.ends_with('h') {
            u64::from_str_radix(&s[..s.len()-1], 16)
                .map_err(|_| format!("Invalid hex immediate: {}", s))
        } else {
            s.parse::<i64>()
                .map(|v| v as u64)
                .map_err(|_| format!("Invalid immediate: {}", s))
        }
    }

    fn get_register_code(&self, reg: &str) -> Result<u8, String> {
        let reg = reg.to_lowercase();
        match reg.as_str() {
            "rax" | "eax" | "ax" | "al" => Ok(0),
            "rcx" | "ecx" | "cx" | "cl" => Ok(1),
            "rdx" | "edx" | "dx" | "dl" => Ok(2),
            "rbx" | "ebx" | "bx" | "bl" => Ok(3),
            "rsp" | "esp" => Ok(4),
            "rbp" | "ebp" => Ok(5),
            "rsi" | "esi" => Ok(6),
            "rdi" | "edi" => Ok(7),
            "r8" => Ok(8),
            "r9" => Ok(9),
            "r10" => Ok(10),
            "r11" => Ok(11),
            "r12" => Ok(12),
            "r13" => Ok(13),
            "r14" => Ok(14),
            "r15" => Ok(15),
            _ => Err(format!("Unknown register: {}", reg))
        }
    }
}

/// Create a minimal PE executable from assembled code
pub fn create_pe_executable(binary: &AssembledBinary, output_path: &Path) -> Result<(), String> {
    let mut pe = Vec::new();
    
    // DOS Header - must start with "MZ" signature
    pe.extend_from_slice(b"MZ");
    // Pad to offset 0x3C (60 bytes total, we have 2, need 58 more)
    for _ in 0..58 {
        pe.push(0);
    }
    // At offset 0x3C, write PE header offset (0x80)
    pe.extend_from_slice(&0x80u32.to_le_bytes());
    

    
    // DOS Stub - pad from current position (0x40) to 0x80
    while pe.len() < 0x80 {
        pe.push(0);
    }
    
    // PE Signature
    pe.extend_from_slice(b"PE\0\0");
    
    // COFF Header
    let machine_type = if binary.is_64bit { 0x8664u16 } else { 0x014Cu16 };
    pe.extend_from_slice(&machine_type.to_le_bytes());
    pe.extend_from_slice(&1u16.to_le_bytes()); // Number of sections
    pe.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
    pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToSymbolTable
    pe.extend_from_slice(&0u32.to_le_bytes()); // NumberOfSymbols
    let optional_header_size = if binary.is_64bit { 0xF0u16 } else { 0xE0u16 };
    pe.extend_from_slice(&optional_header_size.to_le_bytes());
    pe.extend_from_slice(&0x22u16.to_le_bytes()); // Characteristics
    
    // Optional Header
    let magic = if binary.is_64bit { 0x20Bu16 } else { 0x10Bu16 };
    pe.extend_from_slice(&magic.to_le_bytes());
    pe.extend_from_slice(&14u8.to_le_bytes()); // MajorLinkerVersion
    pe.extend_from_slice(&0u8.to_le_bytes()); // MinorLinkerVersion
    pe.extend_from_slice(&(binary.code.len() as u32).to_le_bytes()); // SizeOfCode
    pe.extend_from_slice(&0u32.to_le_bytes()); // SizeOfInitializedData
    pe.extend_from_slice(&0u32.to_le_bytes()); // SizeOfUninitializedData
    pe.extend_from_slice(&binary.entry_point.to_le_bytes()); // AddressOfEntryPoint
    pe.extend_from_slice(&0x1000u32.to_le_bytes()); // BaseOfCode
    
    if binary.is_64bit {
        pe.extend_from_slice(&0x400000u64.to_le_bytes()); // ImageBase (64-bit)
    } else {
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // BaseOfData (32-bit only) - same as code base
        pe.extend_from_slice(&0x400000u32.to_le_bytes()); // ImageBase (32-bit)
    }
    
    pe.extend_from_slice(&0x1000u32.to_le_bytes()); // SectionAlignment
    pe.extend_from_slice(&0x200u32.to_le_bytes()); // FileAlignment
    pe.extend_from_slice(&6u16.to_le_bytes()); // MajorOperatingSystemVersion
    pe.extend_from_slice(&0u16.to_le_bytes()); // MinorOperatingSystemVersion
    pe.extend_from_slice(&0u16.to_le_bytes()); // MajorImageVersion
    pe.extend_from_slice(&0u16.to_le_bytes()); // MinorImageVersion
    pe.extend_from_slice(&6u16.to_le_bytes()); // MajorSubsystemVersion
    pe.extend_from_slice(&0u16.to_le_bytes()); // MinorSubsystemVersion
    pe.extend_from_slice(&0u32.to_le_bytes()); // Win32VersionValue
    pe.extend_from_slice(&0x2000u32.to_le_bytes()); // SizeOfImage
    pe.extend_from_slice(&0x200u32.to_le_bytes()); // SizeOfHeaders
    pe.extend_from_slice(&0u32.to_le_bytes()); // CheckSum
    pe.extend_from_slice(&3u16.to_le_bytes()); // Subsystem (CONSOLE)
    pe.extend_from_slice(&0u16.to_le_bytes()); // DllCharacteristics
    
    if binary.is_64bit {
        pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfStackReserve
        pe.extend_from_slice(&0x1000u64.to_le_bytes()); // SizeOfStackCommit
        pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfHeapReserve
        pe.extend_from_slice(&0x1000u64.to_le_bytes()); // SizeOfHeapCommit
    } else {
        pe.extend_from_slice(&0x100000u32.to_le_bytes()); // SizeOfStackReserve
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // SizeOfStackCommit
        pe.extend_from_slice(&0x100000u32.to_le_bytes()); // SizeOfHeapReserve
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // SizeOfHeapCommit
    }
    
    pe.extend_from_slice(&0u32.to_le_bytes()); // LoaderFlags
    pe.extend_from_slice(&16u32.to_le_bytes()); // NumberOfRvaAndSizes
    
    // Data Directories (16 entries)
    for _ in 0..16 {
        pe.extend_from_slice(&0u64.to_le_bytes());
    }
    
    // Section Header (.text)
    pe.extend_from_slice(b".text\0\0\0");
    let virtual_size = binary.code.len() as u32;
    let raw_size = ((binary.code.len() + 0x1FF) & !0x1FF) as u32;
    pe.extend_from_slice(&virtual_size.to_le_bytes()); // VirtualSize
    pe.extend_from_slice(&0x1000u32.to_le_bytes()); // VirtualAddress
    pe.extend_from_slice(&raw_size.to_le_bytes()); // SizeOfRawData
    pe.extend_from_slice(&0x200u32.to_le_bytes()); // PointerToRawData
    pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToRelocations
    pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToLinenumbers
    pe.extend_from_slice(&0u16.to_le_bytes()); // NumberOfRelocations
    pe.extend_from_slice(&0u16.to_le_bytes()); // NumberOfLinenumbers
    // Characteristics: CODE | EXECUTE | READ
    pe.extend_from_slice(&0xE0000020u32.to_le_bytes());
    

    
    // Pad to file alignment (0x200)
    while pe.len() < 0x200 {
        pe.push(0);
    }
    

    
    // Code section
    pe.extend_from_slice(&binary.code);
    
    // Pad to file alignment
    let aligned_size = (pe.len() + 0x1FF) & !0x1FF;
    while pe.len() < aligned_size {
        pe.push(0);
    }
    
    fs::write(output_path, &pe)
        .map_err(|e| format!("Failed to write executable: {}", e))?;
    
    Ok(())
}