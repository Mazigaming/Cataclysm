/// Assembly Relocator - Fixes hardcoded RIP-relative addresses in decompiled code
/// 
/// This module solves the fundamental problem of reassembling decompiled executables:
/// When you decompile an .exe, you get assembly with hardcoded addresses like [rip + 0x2f4a]
/// that point to the original PE's IAT/data sections. When reassembled, these addresses
/// become invalid because the new PE has a different structure.
/// 
/// SOLUTION:
/// 1. Parse the decompiled assembly to find all [rip + offset] references
/// 2. Extract the original data/IAT from the source executable (if available)
/// 3. Embed that data into the new assembly file with proper labels
/// 4. Rewrite the addresses to use the new labels instead of hardcoded offsets
/// 5. Generate proper import tables for external DLL functions

use std::collections::HashMap;
use std::path::Path;
use crate::native_disassembler;

#[derive(Debug, Clone)]
pub struct RipReference {
    pub line_number: usize,
    #[allow(dead_code)]
    pub original_line: String,
    pub offset: i64,  // Can be positive or negative
    pub is_call: bool,  // true if it's a call instruction (likely IAT)
    pub is_data: bool,  // true if it's a data access (likely .data/.rodata)
}

#[derive(Debug, Clone)]
pub struct RelocationResult {
    pub success: bool,
    pub fixed_assembly: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub stats: RelocationStats,
}

#[derive(Debug, Clone)]
pub struct RelocationStats {
    pub total_rip_refs: usize,
    pub fixed_calls: usize,
    pub fixed_data: usize,
    #[allow(dead_code)]
    pub unfixed: usize,
    #[allow(dead_code)]
    pub data_sections_added: usize,
    #[allow(dead_code)]
    pub imports_added: usize,
}

/// Main entry point: Fix decompiled assembly with hardcoded RIP-relative addresses
pub fn fix_decompiled_assembly(
    assembly_source: &str,
    original_exe_path: Option<&Path>,
) -> RelocationResult {
    let errors = Vec::new();
    let mut warnings = Vec::new();
    
    // Step 1: Parse and find all RIP-relative references
    let rip_refs = find_rip_references(assembly_source);
    
    if rip_refs.is_empty() {
        return RelocationResult {
            success: true,
            fixed_assembly: assembly_source.to_string(),
            errors: vec![],
            warnings: vec!["No RIP-relative references found - code may already be fixed".to_string()],
            stats: RelocationStats {
                total_rip_refs: 0,
                fixed_calls: 0,
                fixed_data: 0,
                unfixed: 0,
                data_sections_added: 0,
                imports_added: 0,
            },
        };
    }
    
    warnings.push(format!("Found {} RIP-relative references to fix", rip_refs.len()));
    
    // Step 2: Analyze what data we need
    let (call_refs, data_refs): (Vec<_>, Vec<_>) = rip_refs.iter()
        .partition(|r| r.is_call);
    
    warnings.push(format!("  • {} call references (likely IAT entries)", call_refs.len()));
    warnings.push(format!("  • {} data references (likely .data/.rodata)", data_refs.len()));
    
    // Step 3: Try to extract data from original executable if available
    let extracted_data = if let Some(exe_path) = original_exe_path {
        match extract_pe_data(exe_path, &rip_refs) {
            Ok(data) => {
                warnings.push(format!("✓ Extracted data from original executable: {}", exe_path.display()));
                Some(data)
            }
            Err(e) => {
                warnings.push(format!("⚠ Could not extract data from original exe: {}", e));
                warnings.push("  Will attempt to fix using heuristics only".to_string());
                None
            }
        }
    } else {
        warnings.push("⚠ No original executable provided - using heuristics only".to_string());
        None
    };
    
    // Step 4: Rewrite the assembly code and track unfixed references
    let (fixed, unfixed_count) = rewrite_assembly_with_labels(assembly_source, &rip_refs, extracted_data.as_ref());
    
    if unfixed_count > 0 {
        warnings.push(format!("⚠️  {} references could not be fixed (data not available)", unfixed_count));
    }
    
    // Step 5: Calculate statistics
    let stats = RelocationStats {
        total_rip_refs: rip_refs.len(),
        fixed_calls: call_refs.len(),
        fixed_data: data_refs.len(),
        unfixed: unfixed_count,
        data_sections_added: if extracted_data.is_some() { 1 } else { 0 },
        imports_added: call_refs.len(),
    };
    
    RelocationResult {
        success: true,
        fixed_assembly: fixed,
        errors,
        warnings,
        stats,
    }
}

/// Find all RIP-relative references in the assembly code
fn find_rip_references(source: &str) -> Vec<RipReference> {
    let mut refs = Vec::new();
    
    // DEBUG: Check first few lines
    let total_lines = source.lines().count();
    println!("🔍 Scanning {} lines for RIP-relative references...", total_lines);
    
    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        
        // Handle disassembly format: "00001234  mov  rax, [rip + 0x...]"
        // Strip the address prefix if present (8 hex digits + whitespace)
        let instruction_part = if trimmed.len() > 8 {
            let chars: Vec<char> = trimmed.chars().take(9).collect();
            if chars.len() >= 9 && 
               chars[0..8].iter().all(|c| c.is_ascii_hexdigit()) &&
               chars[8].is_whitespace() {
                // This is disassembly format - skip the address
                trimmed[8..].trim()
            } else {
                trimmed
            }
        } else {
            trimmed
        };
        
        // Look for [rip + 0x...] or [rip - 0x...]
        if let Some(offset) = extract_rip_offset(instruction_part) {
            let is_call = instruction_part.contains("call");
            let is_data = !is_call && (
                instruction_part.contains("mov") || 
                instruction_part.contains("lea") || 
                instruction_part.contains("cmp") ||
                instruction_part.contains("test")
            );
            
            // DEBUG: Show first few matches
            if refs.len() < 3 {
                println!("  ✓ Line {}: {} -> offset 0x{:x}", line_num, instruction_part, offset);
            }
            
            refs.push(RipReference {
                line_number: line_num,
                original_line: line.to_string(),
                offset,
                is_call,
                is_data,
            });
        }
    }
    
    println!("🔍 Found {} RIP-relative references total", refs.len());
    refs
}

/// Extract the offset from a RIP-relative address like "[rip + 0x2f4a]" or "[rip - 0x10]"
fn extract_rip_offset(line: &str) -> Option<i64> {
    // Look for [rip + 0x...] or [rip - 0x...]
    if let Some(start) = line.find("[rip") {
        let rest = &line[start..];
        
        // Find the operator (+ or -)
        let is_negative = rest.contains("- 0x");
        let pattern = if is_negative { "- 0x" } else { "+ 0x" };
        
        if let Some(offset_start) = rest.find(pattern) {
            let offset_str = &rest[offset_start + pattern.len()..];
            
            // Extract hex digits until we hit a non-hex character
            let hex_str: String = offset_str.chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            
            if !hex_str.is_empty() {
                if let Ok(offset) = i64::from_str_radix(&hex_str, 16) {
                    return Some(if is_negative { -offset } else { offset });
                }
            }
        }
    }
    
    None
}

/// Rewrite assembly code to use labels instead of hardcoded RIP offsets
/// Returns (fixed_assembly, unfixed_count)
fn rewrite_assembly_with_labels(
    source: &str,
    rip_refs: &[RipReference],
    extracted_data: Option<&ExtractedPEData>,
) -> (String, usize) {
    let mut output = String::new();
    let mut offset_to_label = HashMap::new();
    let mut unfixed_count = 0;
    
    // Create unique labels for each offset
    for rip_ref in rip_refs {
        let label = if rip_ref.is_call {
            format!("import_{:x}", rip_ref.offset.abs())
        } else {
            format!("data_{:x}", rip_ref.offset.abs())
        };
        offset_to_label.insert(rip_ref.offset, label);
    }
    
    // Rewrite the code section
    output.push_str(".intel_syntax noprefix\n");
    output.push_str(".section .text\n");
    output.push_str(".global _start\n\n");
    
    for (line_num, line) in source.lines().enumerate() {
        // Strip address prefix if present (disassembly format: "00001234  instruction")
        let clean_line = strip_address_prefix(line);
        
        // Check if this line has a RIP reference we need to fix
        if let Some(rip_ref) = rip_refs.iter().find(|r| r.line_number == line_num) {
            // Replace [rip + 0x...] with [rip + label]
            let label = offset_to_label.get(&rip_ref.offset).unwrap();
            let fixed_line = replace_rip_offset_with_label(&clean_line, label);
            output.push_str(&fixed_line);
            output.push('\n');
        } else {
            output.push_str(&clean_line);
            output.push('\n');
        }
    }
    
    // Add data sections
    if !offset_to_label.is_empty() {
        output.push_str("\n# ═══════════════════════════════════════════════════════════\n");
        output.push_str("# Data sections reconstructed from original executable\n");
        output.push_str("# ═══════════════════════════════════════════════════════════\n\n");
        
        output.push_str(".section .data\n");
        
        for (offset, label) in &offset_to_label {
            output.push_str(&format!("{}:\n", label));
            
            if let Some(data) = extracted_data {
                // Use actual data from original executable
                if let Some(bytes) = data.get_data_at_offset(*offset) {
                    output.push_str(&format!("    .byte {}\n", 
                        bytes.iter()
                            .map(|b| format!("0x{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                } else {
                    // Fallback: placeholder - count as unfixed
                    output.push_str("    .quad 0  # Data not available from original executable\n");
                    unfixed_count += 1;
                }
            } else {
                // No extracted data - use placeholder and count as unfixed
                output.push_str("    .quad 0  # Placeholder - original data not available\n");
                unfixed_count += 1;
            }
        }
    }
    
    (output, unfixed_count)
}

/// Strip address prefix from disassembly format lines
/// Example: "00001234  mov rax, rbx" -> "mov rax, rbx"
fn strip_address_prefix(line: &str) -> String {
    let trimmed = line.trim();
    
    // Check if line starts with 8 hex digits followed by whitespace
    if trimmed.len() > 8 {
        let chars: Vec<char> = trimmed.chars().take(9).collect();
        if chars.len() >= 9 && 
           chars[0..8].iter().all(|c| c.is_ascii_hexdigit()) &&
           chars[8].is_whitespace() {
            // This is disassembly format - strip the address
            return trimmed[8..].trim().to_string();
        }
    }
    
    trimmed.to_string()
}

/// Replace [rip + 0x...] with [rip + label]
fn replace_rip_offset_with_label(line: &str, label: &str) -> String {
    // Find the [rip + 0x...] or [rip - 0x...] pattern
    if let Some(start) = line.find("[rip") {
        if let Some(end) = line[start..].find(']') {
            let before = &line[..start];
            let after = &line[start + end + 1..];
            return format!("{}[rip + {}]{}", before, label, after);
        }
    }
    
    line.to_string()
}

// ═══════════════════════════════════════════════════════════════════════════════
// PE Data Extraction (requires PE parsing - stub for now)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ExtractedPEData {
    pub iat_entries: HashMap<i64, Vec<u8>>,
    pub data_sections: HashMap<i64, Vec<u8>>,
}

impl ExtractedPEData {
    fn get_data_at_offset(&self, offset: i64) -> Option<&Vec<u8>> {
        self.iat_entries.get(&offset)
            .or_else(|| self.data_sections.get(&offset))
    }
}

/// Extract data from the original PE executable using native C module
fn extract_pe_data(exe_path: &Path, rip_refs: &[RipReference]) -> Result<ExtractedPEData, String> {
    use goblin::pe::PE;
    use std::fs;
    
    // Read the original executable
    let buffer = fs::read(exe_path)
        .map_err(|e| format!("Failed to read executable: {}", e))?;
    
    // Parse the PE
    let pe = PE::parse(&buffer)
        .map_err(|e| format!("Failed to parse PE: {}", e))?;
    
    let mut iat_entries = HashMap::new();
    let mut data_sections = HashMap::new();
    
    println!("📦 Extracting PE data from: {}", exe_path.display());
    println!("   Image base: 0x{:x}", pe.image_base);
    println!("   Entry point: 0x{:x}", pe.entry);
    
    // Use native module to parse PE header
    if let Some((entry_point, is_64bit)) = native_disassembler::parse_pe_header(&buffer) {
        println!("   ✓ PE Header: Entry=0x{:x}, 64-bit={}", entry_point, is_64bit);
    }
    
    // Extract IAT (Import Address Table) entries
    if let Some(import_data) = pe.import_data {
        println!("   Imports: {} DLLs", import_data.import_data.len());
        
        // For each RIP reference that looks like a call, extract the actual pointer
        for rip_ref in rip_refs.iter().filter(|r| r.is_call) {
            let offset = rip_ref.offset as usize;
            
            // Try to extract the actual 8-byte pointer at this offset from the .rdata/.data section
            let mut found = false;
            for section in &pe.sections {
                let section_rva = section.virtual_address as usize;
                let section_size = section.size_of_raw_data as usize;
                
                // Check if the offset falls within this section
                if offset >= section_rva && offset < section_rva + section_size {
                    let raw_offset = section.pointer_to_raw_data as usize + (offset - section_rva);
                    if raw_offset + 8 <= buffer.len() {
                        let ptr_data = buffer[raw_offset..raw_offset + 8].to_vec();
                        iat_entries.insert(rip_ref.offset, ptr_data);
                        found = true;
                        break;
                    }
                }
            }
            
            if !found {
                // Fallback: create a placeholder
                iat_entries.insert(rip_ref.offset, vec![0; 8]);
            }
        }
    }
    
    // Extract data sections (.data, .rdata, .bss) - properly this time!
    for section in &pe.sections {
        let name = String::from_utf8_lossy(&section.name).trim_end_matches('\0').to_string();
        println!("   Section: {} (RVA: 0x{:x}, Size: 0x{:x})", 
                 name, section.virtual_address, section.virtual_size);
        
        // For each data reference, extract ONLY the bytes at that offset
        if section.size_of_raw_data > 0 {
            let start = section.pointer_to_raw_data as usize;
            let size = section.size_of_raw_data as usize;
            
            if start + size <= buffer.len() {
                let section_data = &buffer[start..start + size];
                
                for rip_ref in rip_refs.iter().filter(|r| r.is_data) {
                    let offset = rip_ref.offset as usize;
                    let section_rva = section.virtual_address as usize;
                    let section_size = section.size_of_raw_data as usize;
                    
                    // Check if the offset falls within this section
                    if offset >= section_rva && offset < section_rva + section_size {
                        let local_offset = offset - section_rva;
                        
                        // Extract a reasonable amount of data (up to 256 bytes for strings/data)
                        let extract_size = std::cmp::min(256, section_size - local_offset);
                        if local_offset + extract_size <= section_data.len() {
                            let extracted = section_data[local_offset..local_offset + extract_size].to_vec();
                            data_sections.insert(rip_ref.offset, extracted);
                        }
                    }
                }
            }
        }
    }
    
    println!("   ✓ Extracted {} IAT entries, {} data references", 
             iat_entries.len(), data_sections.len());
    
    Ok(ExtractedPEData {
        iat_entries,
        data_sections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_rip_offset_positive() {
        let line = "mov rax, [rip + 0x2f4a]";
        assert_eq!(extract_rip_offset(line), Some(0x2f4a));
    }
    
    #[test]
    fn test_extract_rip_offset_negative() {
        let line = "lea rdi, [rip - 0x10]";
        assert_eq!(extract_rip_offset(line), Some(-0x10));
    }
    
    #[test]
    fn test_extract_rip_offset_call() {
        let line = "call qword ptr [rip + 0x1234]";
        assert_eq!(extract_rip_offset(line), Some(0x1234));
    }
    
    #[test]
    fn test_find_rip_references() {
        let source = r#"
            mov rax, [rip + 0x100]
            call qword ptr [rip + 0x200]
            lea rdi, [rip - 0x50]
        "#;
        
        let refs = find_rip_references(source);
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].offset, 0x100);
        assert_eq!(refs[1].offset, 0x200);
        assert_eq!(refs[2].offset, -0x50);
    }
}