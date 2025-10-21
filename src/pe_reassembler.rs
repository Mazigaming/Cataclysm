// ============================================================================
// PE REASSEMBLER  "Apply Patches" Feature
// ============================================================================
// This module enables reassembling decompiled executables by:
// 1. Extracting all sections from the original PE (imports, data, resources)
// 2. Reassembling only the modified .text section (code)
// 3. Rebuilding the PE with the new code + preserved sections
//
// ============================================================================

use goblin::pe::PE;
use std::fs;
use std::path::Path;
use crate::pe_fixer;

#[derive(Debug, Clone)]
pub struct PreservedPEData {
    pub original_pe: Vec<u8>,
    #[allow(dead_code)]
    pub image_base: u64,
    pub entry_point: u32,
    pub sections: Vec<PreservedSection>,
    #[allow(dead_code)]
    pub imports: Vec<ImportDescriptor>,
    #[allow(dead_code)]
    pub exports: Option<Vec<u8>>,
    #[allow(dead_code)]
    pub resources: Option<Vec<u8>>,
    #[allow(dead_code)]
    pub relocations: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct PreservedSection {
    pub name: String,
    pub virtual_address: u32,
    pub virtual_size: u32,
    pub raw_data: Vec<u8>,
    pub characteristics: u32,
}

#[derive(Debug, Clone)]
pub struct ImportDescriptor {
    #[allow(dead_code)]
    pub dll_name: String,
    #[allow(dead_code)]
    pub functions: Vec<String>,
}

/// PHASE 2: New import to be added
#[derive(Debug, Clone)]
pub struct NewImport {
    pub dll_name: String,
    pub function_name: String,
}

/// PHASE 2: Options for reassembly
#[derive(Debug, Clone, Default)]
pub struct ReassemblyOptions {
    pub allow_expansion: bool,
    pub new_imports: Vec<NewImport>,
    pub preserve_timestamps: bool,
    pub recalculate_checksum: bool,
}

/// Extract all data from the original PE that needs to be preserved
pub fn extract_pe_structure(exe_path: &Path) -> Result<PreservedPEData, String> {
    println!("📦 [DEBUG] Extracting PE structure from: {}", exe_path.display());
    
    // Read the original executable
    let buffer = fs::read(exe_path)
        .map_err(|e| format!("Failed to read executable: {}", e))?;
    
    println!("   [DEBUG] File size: {} bytes", buffer.len());
    
    // Parse the PE
    let pe = PE::parse(&buffer)
        .map_err(|e| format!("Failed to parse PE: {}", e))?;
    
    println!("   [DEBUG] Image base: 0x{:x}", pe.image_base);
    println!("   [DEBUG] Entry point: 0x{:x}", pe.entry);
    println!("   [DEBUG] Sections: {}", pe.sections.len());
    println!("   [DEBUG] Is 64-bit: {}", pe.is_64);
    
    // Extract all sections
    let mut sections = Vec::new();
    for section in &pe.sections {
        let name = String::from_utf8_lossy(&section.name)
            .trim_end_matches('\0')
            .to_string();
        
        let raw_data = if section.size_of_raw_data > 0 {
            let start = section.pointer_to_raw_data as usize;
            let size = section.size_of_raw_data as usize;
            
            if start + size <= buffer.len() {
                buffer[start..start + size].to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        println!("   [DEBUG] • {} (RVA: 0x{:x}, VSize: {}, RawSize: {}, Characteristics: 0x{:x})", 
                 name, section.virtual_address, section.virtual_size, raw_data.len(), section.characteristics);
        
        sections.push(PreservedSection {
            name,
            virtual_address: section.virtual_address,
            virtual_size: section.virtual_size,
            raw_data,
            characteristics: section.characteristics,
        });
    }
    
    // Extract imports
    let mut imports = Vec::new();
    if let Some(import_data) = pe.import_data {
        println!("   [DEBUG] Extracting imports...");
        for import in import_data.import_data {
            let dll_name = import.name.to_string();
            let functions = Vec::new();
            
            // Extract function names from import lookup table
            // This is a simplified version - full implementation would parse IAT/ILT
            println!("      [DEBUG] DLL: {}", dll_name);
            
            imports.push(ImportDescriptor {
                dll_name,
                functions,
            });
        }
        println!("   [DEBUG] Total imports: {} DLLs", imports.len());
    } else {
        println!("   [DEBUG] No imports found");
    }
    
    // Extract exports (if present)
    let exports = if let Some(_export_data) = pe.export_data {
        // Find the export directory in the data directories
        let export_table_opt = pe.header.optional_header.as_ref()
            .and_then(|oh| oh.data_directories.get_export_table().as_ref());
        
        if let Some(export_table) = export_table_opt {
            let export_rva = export_table.virtual_address as usize;
            let export_size = export_table.size as usize;
            
            // Find which section contains the export directory
            if let Some(section) = pe.sections.iter().find(|s| {
                let start = s.virtual_address as usize;
                let end = start + s.virtual_size as usize;
                export_rva >= start && export_rva < end
            }) {
                let offset_in_section = export_rva - section.virtual_address as usize;
                let file_offset = section.pointer_to_raw_data as usize + offset_in_section;
                
                if file_offset + export_size <= buffer.len() {
                    let export_bytes = buffer[file_offset..file_offset + export_size].to_vec();
                    println!("   [DEBUG] ✅ Exports extracted: {} bytes", export_bytes.len());
                    Some(export_bytes)
                } else {
                    println!("   [DEBUG] ⚠️  Export data out of bounds");
                    None
                }
            } else {
                println!("   [DEBUG] ⚠️  Could not find section containing exports");
                None
            }
        } else {
            println!("   [DEBUG] ⚠️  Export table not found in data directories");
            None
        }
    } else {
        println!("   [DEBUG] No exports");
        None
    };
    
    // Extract resources (if present)
    let resource_table_opt = pe.header.optional_header.as_ref()
        .and_then(|oh| oh.data_directories.get_resource_table().as_ref());
    
    let resources = if let Some(resource_table) = resource_table_opt {
        let resource_rva = resource_table.virtual_address as usize;
        let resource_size = resource_table.size as usize;
        
        if resource_size > 0 {
            // Find which section contains the resource directory
            if let Some(section) = pe.sections.iter().find(|s| {
                let start = s.virtual_address as usize;
                let end = start + s.virtual_size as usize;
                resource_rva >= start && resource_rva < end
            }) {
                let offset_in_section = resource_rva - section.virtual_address as usize;
                let file_offset = section.pointer_to_raw_data as usize + offset_in_section;
                
                if file_offset + resource_size <= buffer.len() {
                    let resource_bytes = buffer[file_offset..file_offset + resource_size].to_vec();
                    println!("   [DEBUG] ✅ Resources extracted: {} bytes", resource_bytes.len());
                    Some(resource_bytes)
                } else {
                    println!("   [DEBUG] ⚠️  Resource data out of bounds");
                    None
                }
            } else {
                println!("   [DEBUG] ⚠️  Could not find section containing resources");
                None
            }
        } else {
            println!("   [DEBUG] No resources");
            None
        }
    } else {
        println!("   [DEBUG] No resources");
        None
    };
    
    // Extract relocations (if present)
    let reloc_table_opt = pe.header.optional_header.as_ref()
        .and_then(|oh| oh.data_directories.get_base_relocation_table().as_ref());
    
    let relocations = if let Some(reloc_table) = reloc_table_opt {
        let reloc_rva = reloc_table.virtual_address as usize;
        let reloc_size = reloc_table.size as usize;
        
        if reloc_size > 0 {
            // Find which section contains the relocation table
            if let Some(section) = pe.sections.iter().find(|s| {
                let start = s.virtual_address as usize;
                let end = start + s.virtual_size as usize;
                reloc_rva >= start && reloc_rva < end
            }) {
                let offset_in_section = reloc_rva - section.virtual_address as usize;
                let file_offset = section.pointer_to_raw_data as usize + offset_in_section;
                
                if file_offset + reloc_size <= buffer.len() {
                    let reloc_bytes = buffer[file_offset..file_offset + reloc_size].to_vec();
                    println!("   [DEBUG] ✅ Relocations extracted: {} bytes (ASLR support)", reloc_bytes.len());
                    Some(reloc_bytes)
                } else {
                    println!("   [DEBUG] ⚠️  Relocation data out of bounds");
                    None
                }
            } else {
                println!("   [DEBUG] ⚠️  Could not find section containing relocations");
                None
            }
        } else {
            println!("   [DEBUG] No relocations");
            None
        }
    } else {
        println!("   [DEBUG] No relocations");
        None
    };
    
    println!("   [DEBUG] ✅ PE structure extraction complete!");
    
    Ok(PreservedPEData {
        original_pe: buffer.clone(),
        image_base: pe.image_base as u64,
        entry_point: pe.entry as u32,
        sections,
        imports,
        exports,
        resources,
        relocations,
    })
}

/// Reassemble the executable with new code but preserved sections
/// Phase 2: Now supports section expansion!
pub fn reassemble_with_preserved_data(
    preserved: &PreservedPEData,
    mut new_code: Vec<u8>,
    output_path: &Path,
) -> Result<(), String> {
    // CRITICAL: Validate and fix code before reassembly
    println!("🔍 [PE_REASSEMBLER] Validating assembled code...");
    let pe = PE::parse(&preserved.original_pe)
        .map_err(|e| format!("Failed to parse PE: {}", e))?;
    
    let validation = pe_fixer::validate_assembled_code(&new_code, &pe, 0);
    
    if !validation.is_valid {
        println!("⚠️  [PE_REASSEMBLER] Validation found critical issues!");
        println!("{}", pe_fixer::format_validation_report(&validation));
        
        // Try to auto-fix
        println!("🔧 [PE_REASSEMBLER] Attempting automatic fixes...");
        let fixes = pe_fixer::auto_fix_code(&mut new_code, &pe, &validation);
        
        if !fixes.is_empty() {
            println!("✅ [PE_REASSEMBLER] Applied {} fixes", fixes.len());
            for fix in &fixes {
                println!("   ✓ {}", fix);
            }
            
            // Re-validate after fixes
            let revalidation = pe_fixer::validate_assembled_code(&new_code, &pe, 0);
            if !revalidation.is_valid {
                return Err(format!(
                    "Code validation failed even after automatic fixes:\n{}",
                    pe_fixer::format_validation_report(&revalidation)
                ));
            }
        } else {
            return Err(format!(
                "Code validation failed and no automatic fixes available:\n{}",
                pe_fixer::format_validation_report(&validation)
            ));
        }
    } else {
        println!("✅ [PE_REASSEMBLER] Code validation passed!");
        if !validation.warnings.is_empty() {
            println!("⚠️  Warnings:");
            for warning in &validation.warnings {
                println!("   {}", warning);
            }
        }
    }
    
    reassemble_with_options(preserved, new_code, output_path, false)
}

/// Reassemble with section expansion support (Phase 2 feature)
#[allow(dead_code)]
pub fn reassemble_with_expansion(
    preserved: &PreservedPEData,
    new_code: Vec<u8>,
    output_path: &Path,
) -> Result<(), String> {
    reassemble_with_options(preserved, new_code, output_path, true)
}

/// Internal function with expansion support
fn reassemble_with_options(
    preserved: &PreservedPEData,
    new_code: Vec<u8>,
    output_path: &Path,
    allow_expansion: bool,
) -> Result<(), String> {
    println!("🔨 [DEBUG] Reassembling PE with new code...");
    println!("   [DEBUG] Original PE size: {} bytes", preserved.original_pe.len());
    println!("   [DEBUG] New code size: {} bytes", new_code.len());
    println!("   [DEBUG] Allow expansion: {}", allow_expansion);
    
    // Strategy: Clone the original PE and replace only the .text section
    let mut new_pe = preserved.original_pe.clone();
    
    // Find the .text section in the original PE
    let text_section = preserved.sections.iter()
        .find(|s| s.name == ".text")
        .ok_or("No .text section found in original PE")?;
    
    println!("   [DEBUG] Original .text section:");
    println!("      [DEBUG] RVA: 0x{:x}", text_section.virtual_address);
    println!("      [DEBUG] Virtual Size: {} bytes", text_section.virtual_size);
    println!("      [DEBUG] Raw Data Size: {} bytes", text_section.raw_data.len());
    println!("      [DEBUG] Characteristics: 0x{:x}", text_section.characteristics);
    
    // Calculate where the .text section is in the file
    // We need to find the file offset from the section headers
    let pe = PE::parse(&preserved.original_pe)
        .map_err(|e| format!("Failed to re-parse PE: {}", e))?;
    
    let text_section_header = pe.sections.iter()
        .find(|s| {
            let name = String::from_utf8_lossy(&s.name);
            name.trim_end_matches('\0') == ".text"
        })
        .ok_or("No .text section header found")?;
    
    let file_offset = text_section_header.pointer_to_raw_data as usize;
    let old_size = text_section_header.size_of_raw_data as usize;
    
    println!("   [DEBUG] File offset: 0x{:x}", file_offset);
    println!("   [DEBUG] Old size: {} bytes", old_size);
    
    // Check if new code fits in the existing section
    if new_code.len() > old_size {
        println!("   [DEBUG] ⚠️  New code ({} bytes) > old size ({} bytes)", new_code.len(), old_size);
        
        if !allow_expansion {
            return Err(format!(
                "New code ({} bytes) is larger than original .text section ({} bytes).\n\
                 💡 Tip: Use reassemble_with_expansion() to enable section expansion.\n\
                 Or consider using a binary patcher for large modifications.",
                new_code.len(), old_size
            ));
        }
        
        // PHASE 2: Section expansion!
        println!("   [DEBUG] ⚠️  New code is larger than original section");
        println!("   [DEBUG] 🔧 Expanding .text section...");
        
        return expand_section_and_reassemble(preserved, new_code, output_path, file_offset, old_size);
    }
    
    // 🔧 CRITICAL FIX: The disassembly includes the ENTIRE .text section
    // (including any padding before the entry point), so we write from the start
    let text_section_rva = text_section.virtual_address;
    let entry_point_rva = preserved.entry_point;
    let entry_offset_in_section = if entry_point_rva >= text_section_rva {
        (entry_point_rva - text_section_rva) as usize
    } else {
        0
    };
    
    println!("   [DEBUG] 🎯 Entry point analysis:");
    println!("      [DEBUG] Entry RVA: 0x{:x}", entry_point_rva);
    println!("      [DEBUG] .text section RVA: 0x{:x}", text_section_rva);
    println!("      [DEBUG] Entry offset within .text: 0x{:x} ({} bytes)", entry_offset_in_section, entry_offset_in_section);
    
    // Replace the code in the .text section
    println!("   [DEBUG] ✓ New code fits in existing section");
    
    // 🔧 CRITICAL FIX: The disassembler may skip padding before the entry point!
    // If the assembled code is smaller than the entry point offset, it means the
    // disassembly didn't include all the padding. In this case, we need to:
    // 1. Keep the original padding bytes before the entry point
    // 2. Write the assembled code starting FROM the entry point
    
    let write_offset: usize;
    let available_space: usize;
    
    if entry_offset_in_section > 0 && new_code.len() < entry_offset_in_section {
        // The assembled code doesn't reach the entry point!
        // This means the disassembler skipped the initial padding.
        println!("   [DEBUG] ⚠️  Assembled code ({} bytes) < entry offset ({} bytes)", 
                 new_code.len(), entry_offset_in_section);
        println!("   [DEBUG] 🔧 FIX: Preserving original padding, writing code at entry point");
        
        // Write the assembled code starting AT the entry point
        write_offset = file_offset + entry_offset_in_section;
        available_space = old_size - entry_offset_in_section;
        
        println!("   [DEBUG] Writing {} bytes at file offset 0x{:x} (entry point)", new_code.len(), write_offset);
        println!("   [DEBUG] Available space from entry point: {} bytes", available_space);
        
        // The original padding before the entry point is automatically preserved
        // because we're not overwriting it!
    } else {
        // The assembled code includes padding (or entry is at start)
        println!("   [DEBUG] ℹ️  Assembled code includes padding before entry point");
        write_offset = file_offset;  // Start of .text section
        available_space = old_size;
        
        println!("   [DEBUG] Writing {} bytes at file offset 0x{:x} (start of .text)", new_code.len(), write_offset);
        println!("   [DEBUG] Available space in .text section: {} bytes", available_space);
    }
    
    if new_code.len() > available_space {
        return Err(format!(
            "New code ({} bytes) is larger than available space ({} bytes).",
            new_code.len(), available_space
        ));
    }
    
    // Copy new code
    println!("   [DEBUG] Copying {} bytes of new code...", new_code.len());
    
    // 🔍 DEBUG: Show first bytes of assembled code
    print!("   [DEBUG] 🔍 First 16 bytes of assembled code: ");
    for i in 0..16.min(new_code.len()) {
        print!("{:02x} ", new_code[i]);
    }
    println!();
    
    let mut bytes_copied = 0;
    for (i, &byte) in new_code.iter().enumerate() {
        if write_offset + i < new_pe.len() {
            new_pe[write_offset + i] = byte;
            bytes_copied += 1;
        } else {
            println!("   [DEBUG] ⚠️  Reached end of PE buffer at byte {}", i);
            break;
        }
    }
    println!("   [DEBUG] Copied {} bytes", bytes_copied);
    
    // 🔍 VERIFICATION: Check what's at the entry point now
    let entry_file_offset = file_offset + entry_offset_in_section;
    if entry_file_offset + 16 <= new_pe.len() {
        print!("   [DEBUG] 🔍 First 16 bytes at entry point (file offset 0x{:x}): ", entry_file_offset);
        for i in 0..16 {
            print!("{:02x} ", new_pe[entry_file_offset + i]);
        }
        println!();
        
        // Also show what we wrote at the start of .text
        print!("   [DEBUG] 🔍 First 16 bytes at .text start (file offset 0x{:x}): ", file_offset);
        for i in 0..16 {
            print!("{:02x} ", new_pe[file_offset + i]);
        }
        println!();
    }
    
    // Fill remaining space with NOPs (0x90)
    let nop_start = new_code.len();
    let nop_count = old_size - nop_start;
    println!("   [DEBUG] Filling {} bytes with NOPs (from offset 0x{:x})...", nop_count, nop_start);
    for i in nop_start..old_size {
        if file_offset + i < new_pe.len() {
            new_pe[file_offset + i] = 0x90; // NOP
        }
    }
    
    // Write the modified PE
    println!("   [DEBUG] Writing modified PE to: {}", output_path.display());
    fs::write(output_path, &new_pe)
        .map_err(|e| format!("Failed to write output: {}", e))?;
    
    println!("   [DEBUG] ✅ Successfully reassembled!");
    println!("   [DEBUG] Output: {}", output_path.display());
    println!("   [DEBUG] Output size: {} bytes", new_pe.len());
    
    Ok(())
}

/// PHASE 2: Expand the .text section when new code is larger
fn expand_section_and_reassemble(
    preserved: &PreservedPEData,
    new_code: Vec<u8>,
    output_path: &Path,
    _file_offset: usize,
    old_size: usize,
) -> Result<(), String> {
    println!("   [DEBUG] 📏 Old section size: {} bytes", old_size);
    println!("   [DEBUG] 📏 New code size: {} bytes", new_code.len());
    println!("   [DEBUG] 📏 Additional space needed: {} bytes", new_code.len() - old_size);
    
    // Calculate new section size (aligned to 512 bytes - file alignment)
    const FILE_ALIGNMENT: usize = 512;
    let new_size = ((new_code.len() + FILE_ALIGNMENT - 1) / FILE_ALIGNMENT) * FILE_ALIGNMENT;
    println!("   [DEBUG] 📏 New section size (aligned): {} bytes", new_size);
    
    // Parse PE to get section headers
    let pe = PE::parse(&preserved.original_pe)
        .map_err(|e| format!("Failed to parse PE: {}", e))?;
    
    // Find .text section index
    let text_section_idx = pe.sections.iter()
        .position(|s| {
            let name = String::from_utf8_lossy(&s.name);
            name.trim_end_matches('\0') == ".text"
        })
        .ok_or("No .text section found")?;
    
    println!("   [DEBUG] .text section index: {}", text_section_idx);
    
    // Strategy: Create new PE buffer with expanded .text section
    // 1. Copy everything before .text section
    // 2. Write expanded .text section with new code
    // 3. Copy everything after .text section (shifted by expansion amount)
    
    let expansion_size = new_size - old_size;
    let mut new_pe = Vec::with_capacity(preserved.original_pe.len() + expansion_size);
    
    println!("   [DEBUG] Expansion size: {} bytes", expansion_size);
    println!("   [DEBUG] New PE capacity: {} bytes", new_pe.capacity());
    
    // Copy DOS header, PE signature, COFF header, optional header, and section headers
    // These are all before the first section's file data
    let first_section_offset = pe.sections.iter()
        .map(|s| s.pointer_to_raw_data as usize)
        .min()
        .unwrap_or(0);
    
    println!("   [DEBUG] First section offset: 0x{:x}", first_section_offset);
    
    // Copy headers
    new_pe.extend_from_slice(&preserved.original_pe[..first_section_offset]);
    
    // Now copy sections, expanding .text
    for (idx, section) in pe.sections.iter().enumerate() {
        let section_offset = section.pointer_to_raw_data as usize;
        let section_size = section.size_of_raw_data as usize;
        
        if idx == text_section_idx {
            // This is .text - write new code
            println!("   [DEBUG] Writing expanded .text section at offset 0x{:x}", new_pe.len());
            new_pe.extend_from_slice(&new_code);
            
            // Pad to alignment
            let padding = new_size - new_code.len();
            println!("   [DEBUG] Adding {} bytes of padding (NOPs)", padding);
            new_pe.extend(vec![0x90; padding]); // NOP padding
        } else {
            // Copy other sections as-is
            if section_size > 0 && section_offset + section_size <= preserved.original_pe.len() {
                println!("   [DEBUG] Copying section {} at offset 0x{:x} ({} bytes)", 
                         idx, section_offset, section_size);
                new_pe.extend_from_slice(&preserved.original_pe[section_offset..section_offset + section_size]);
            }
        }
    }
    
    // Update section headers with new offsets
    println!("   [DEBUG] 🔧 Updating section headers...");
    
    // Calculate section header offset
    // PE structure: DOS header (64 bytes) + DOS stub + PE signature (4 bytes) + COFF header (20 bytes) + Optional header + Section headers
    let _dos_header_size = 64;
    let pe_signature_offset = u32::from_le_bytes([
        preserved.original_pe[0x3C],
        preserved.original_pe[0x3D],
        preserved.original_pe[0x3E],
        preserved.original_pe[0x3F],
    ]) as usize;
    
    let coff_header_offset = pe_signature_offset + 4; // After "PE\0\0"
    let optional_header_size = u16::from_le_bytes([
        preserved.original_pe[coff_header_offset + 16],
        preserved.original_pe[coff_header_offset + 17],
    ]) as usize;
    
    let section_headers_offset = coff_header_offset + 20 + optional_header_size;
    let section_header_size = 40; // Each section header is 40 bytes
    
    println!("   [DEBUG] Section headers start at offset: 0x{:x}", section_headers_offset);
    
    // Update each section header's PointerToRawData
    let mut cumulative_offset = first_section_offset;
    
    for (idx, section) in pe.sections.iter().enumerate() {
        let header_offset = section_headers_offset + (idx * section_header_size);
        
        if idx == text_section_idx {
            // Update .text section size
            let size_of_raw_data_offset = header_offset + 16;
            let pointer_to_raw_data_offset = header_offset + 20;
            
            // Update SizeOfRawData
            new_pe[size_of_raw_data_offset..size_of_raw_data_offset + 4]
                .copy_from_slice(&(new_size as u32).to_le_bytes());
            
            // Update PointerToRawData
            new_pe[pointer_to_raw_data_offset..pointer_to_raw_data_offset + 4]
                .copy_from_slice(&(cumulative_offset as u32).to_le_bytes());
            
            println!("   [DEBUG] Updated .text section header:");
            println!("      [DEBUG] • SizeOfRawData: {} bytes", new_size);
            println!("      [DEBUG] • PointerToRawData: 0x{:x}", cumulative_offset);
            
            cumulative_offset += new_size;
        } else {
            // Update other sections' PointerToRawData (shifted by expansion)
            let pointer_to_raw_data_offset = header_offset + 20;
            let section_size = section.size_of_raw_data as usize;
            
            if section_size > 0 {
                new_pe[pointer_to_raw_data_offset..pointer_to_raw_data_offset + 4]
                    .copy_from_slice(&(cumulative_offset as u32).to_le_bytes());
                
                println!("   [DEBUG] Updated section {} header: PointerToRawData = 0x{:x}", 
                         idx, cumulative_offset);
                
                cumulative_offset += section_size;
            }
        }
    }
    
    println!("   [DEBUG] ✅ Section headers updated successfully!");
    
    // Write output
    fs::write(output_path, &new_pe)
        .map_err(|e| format!("Failed to write output: {}", e))?;
    
    println!("   [DEBUG] ✅ Expanded PE written successfully!");
    println!("   [DEBUG] Output size: {} bytes", new_pe.len());
    println!("   [DEBUG] ✅ Section headers have been updated for proper loading");
    
    Ok(())
}

/// High-level function: Reassemble decompiled code like IDA/Ghidra
pub fn reassemble_decompiled_exe(
    original_exe: &Path,
    new_code: Vec<u8>,
    output_path: &Path,
) -> Result<(), String> {
    // Step 1: Extract structure from original
    let preserved = extract_pe_structure(original_exe)?;
    
    // Step 2: Reassemble with new code
    reassemble_with_preserved_data(&preserved, new_code, output_path)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_pe_structure() {
        // This would need a test PE file
        // For now, just ensure the module compiles
    }
}
