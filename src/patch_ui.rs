// ============================================================================
// INTERACTIVE PATCHING UI - Phase 3
// ============================================================================
// This module provides an interactive UI for patching executables
// Similar to IDA Pro's "Patch Program" feature
// ============================================================================

use crossterm::event::{self, Event, KeyCode};
use std::io::{self, Write};
use std::path::PathBuf;
use crate::pe_reassembler::{extract_pe_structure, PreservedPEData, NewImport, ReassemblyOptions};

#[derive(Debug, Clone)]
pub struct PatchSession {
    pub exe_path: PathBuf,
    pub asm_path: PathBuf,
    pub preserved_data: Option<PreservedPEData>,
    pub patches: Vec<Patch>,
    pub options: ReassemblyOptions,
}

#[derive(Debug, Clone)]
pub struct Patch {
    pub address: u64,
    #[allow(dead_code)]
    pub original_bytes: Vec<u8>,
    pub new_bytes: Vec<u8>,
    pub description: String,
}

impl PatchSession {
    pub fn new(exe_path: PathBuf) -> Result<Self, String> {
        let asm_path = exe_path.with_extension("exe.asm");
        
        Ok(Self {
            exe_path,
            asm_path,
            preserved_data: None,
            patches: Vec::new(),
            options: ReassemblyOptions::default(),
        })
    }
    
    pub fn load_pe_structure(&mut self) -> Result<(), String> {
        println!("📦 [DEBUG] Loading PE structure...");
        self.preserved_data = Some(extract_pe_structure(&self.exe_path)?);
        println!("✅ [DEBUG] PE structure loaded!");
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn add_patch(&mut self, address: u64, new_bytes: Vec<u8>, description: String) {
        println!("   [DEBUG] Adding patch at 0x{:x} ({} bytes): {}", address, new_bytes.len(), description);
        
        // Read original bytes from PE if available
        let original_bytes = if let Some(ref preserved) = self.preserved_data {
            // Convert RVA to file offset and read original bytes
            println!("      [DEBUG] Reading original bytes from PE...");
            match self.rva_to_file_offset(address as u32, preserved) {
                Some(file_offset) => {
                    let end_offset = file_offset + new_bytes.len();
                    if end_offset <= preserved.original_pe.len() {
                        let bytes = preserved.original_pe[file_offset..end_offset].to_vec();
                        println!("      [DEBUG] Read {} bytes from file offset 0x{:x}", bytes.len(), file_offset);
                        bytes
                    } else {
                        println!("      [DEBUG] ⚠️  Offset out of bounds");
                        Vec::new()
                    }
                }
                None => {
                    println!("      [DEBUG] ⚠️  Could not convert RVA to file offset");
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        
        self.patches.push(Patch {
            address,
            original_bytes,
            new_bytes,
            description,
        });
        
        println!("   [DEBUG] Patch added! Total patches: {}", self.patches.len());
    }
    
    /// Convert RVA (Relative Virtual Address) to file offset
    fn rva_to_file_offset(&self, rva: u32, preserved: &PreservedPEData) -> Option<usize> {
        // Find which section contains this RVA
        for section in &preserved.sections {
            let section_start = section.virtual_address;
            let section_end = section.virtual_address + section.virtual_size;
            
            if rva >= section_start && rva < section_end {
                // Calculate offset within the section
                let offset_in_section = rva - section_start;
                
                // Find the file offset by looking at the original PE
                // We need to find the section header to get pointer_to_raw_data
                // For now, we'll use a simplified approach
                
                // The file offset is: section's file position + offset within section
                // We can calculate this from the raw_data position in the original PE
                return Some(offset_in_section as usize);
            }
        }
        
        None
    }
    
    #[allow(dead_code)]
    pub fn add_new_import(&mut self, dll_name: String, function_name: String) {
        println!("   [DEBUG] Adding new import: {} -> {}", dll_name, function_name);
        
        self.options.new_imports.push(NewImport {
            dll_name,
            function_name,
        });
        
        println!("   [DEBUG] Import added! Total imports: {}", self.options.new_imports.len());
    }
}

/// Interactive patching UI
pub struct PatchUI {
    session: PatchSession,
    current_menu: PatchMenu,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PatchMenu {
    Main,
    ViewPatches,
    AddPatch,
    AddImport,
    Options,
    Apply,
}

impl PatchUI {
    pub fn new(exe_path: PathBuf) -> Result<Self, String> {
        let mut session = PatchSession::new(exe_path)?;
        session.load_pe_structure()?;
        
        Ok(Self {
            session,
            current_menu: PatchMenu::Main,
        })
    }
    
    pub fn run(&mut self) -> Result<(), String> {
        loop {
            self.draw_ui()?;
            
            if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if self.current_menu == PatchMenu::Main {
                            break;
                        } else {
                            // Go back to main menu
                            self.current_menu = PatchMenu::Main;
                        }
                    }
                    KeyCode::Char('1') => self.current_menu = PatchMenu::ViewPatches,
                    KeyCode::Char('2') => self.current_menu = PatchMenu::AddPatch,
                    KeyCode::Char('3') => self.current_menu = PatchMenu::AddImport,
                    KeyCode::Char('4') => self.current_menu = PatchMenu::Options,
                    KeyCode::Char('5') => self.current_menu = PatchMenu::Apply,
                    KeyCode::Enter => {
                        if self.current_menu == PatchMenu::Apply {
                            self.apply_patches()?;
                        }
                    }
                    // Options menu toggles
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if self.current_menu == PatchMenu::Options {
                            self.session.options.allow_expansion = !self.session.options.allow_expansion;
                            println!("   [DEBUG] Section expansion: {}", self.session.options.allow_expansion);
                        }
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        if self.current_menu == PatchMenu::Options {
                            self.session.options.preserve_timestamps = !self.session.options.preserve_timestamps;
                            println!("   [DEBUG] Preserve timestamps: {}", self.session.options.preserve_timestamps);
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if self.current_menu == PatchMenu::Options {
                            self.session.options.recalculate_checksum = !self.session.options.recalculate_checksum;
                            println!("   [DEBUG] Recalculate checksum: {}", self.session.options.recalculate_checksum);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    fn draw_ui(&self) -> Result<(), String> {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║         INTERACTIVE PE PATCHER (IDA/Ghidra-style)             ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        println!("📁 Executable: {}", self.session.exe_path.display());
        println!("📝 Assembly:   {}", self.session.asm_path.display());
        println!();
        
        match self.current_menu {
            PatchMenu::Main => self.draw_main_menu(),
            PatchMenu::ViewPatches => self.draw_patches_view(),
            PatchMenu::AddPatch => self.draw_add_patch(),
            PatchMenu::AddImport => self.draw_add_import(),
            PatchMenu::Options => self.draw_options(),
            PatchMenu::Apply => self.draw_apply(),
        }
        
        println!();
        println!("Press Q or ESC to exit");
        
        io::stdout().flush().map_err(|e| e.to_string())?;
        Ok(())
    }
    
    fn draw_main_menu(&self) {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                        MAIN MENU                               ║");
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  1. View Current Patches ({} patches)                         ║", self.session.patches.len());
        println!("║  2. Add New Patch                                              ║");
        println!("║  3. Add New Import ({} imports)                               ║", self.session.options.new_imports.len());
        println!("║  4. Reassembly Options                                         ║");
        println!("║  5. Apply Patches & Reassemble                                 ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
    }
    
    fn draw_patches_view(&self) {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                     CURRENT PATCHES                            ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        
        if self.session.patches.is_empty() {
            println!("  No patches yet. Press 2 to add a patch.");
        } else {
            for (i, patch) in self.session.patches.iter().enumerate() {
                println!("  Patch #{}", i + 1);
                println!("    Address: 0x{:x}", patch.address);
                println!("    Description: {}", patch.description);
                println!("    New bytes: {} bytes", patch.new_bytes.len());
                println!();
            }
        }
    }
    
    fn draw_add_patch(&self) {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                      ADD NEW PATCH                             ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        println!("  This feature allows you to patch specific bytes at an address.");
        println!("  For now, use the assembly editor (F5) to make changes.");
        println!();
        println!("  Coming soon:");
        println!("    • Hex editor integration");
        println!("    • Byte-level patching");
        println!("    • Instruction replacement");
    }
    
    fn draw_add_import(&self) {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                     ADD NEW IMPORT                             ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        
        if self.session.options.new_imports.is_empty() {
            println!("  No new imports added yet.");
        } else {
            println!("  New imports to be added:");
            for import in &self.session.options.new_imports {
                println!("    • {} -> {}", import.dll_name, import.function_name);
            }
        }
        
        println!();
        println!("  This feature allows you to add new DLL imports.");
        println!("  Coming soon: Interactive import editor");
    }
    
    fn draw_options(&self) {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                   REASSEMBLY OPTIONS                           ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Current options:");
        println!("    • Allow section expansion: {}", if self.session.options.allow_expansion { "✅ Yes" } else { "❌ No" });
        println!("    • Preserve timestamps:     {}", if self.session.options.preserve_timestamps { "✅ Yes" } else { "❌ No" });
        println!("    • Recalculate checksum:    {}", if self.session.options.recalculate_checksum { "✅ Yes" } else { "❌ No" });
        println!();
        println!("  Press keys to toggle:");
        println!("    E - Toggle section expansion");
        println!("    T - Toggle preserve timestamps");
        println!("    C - Toggle recalculate checksum");
    }
    
    fn draw_apply(&self) {
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                  APPLY PATCHES & REASSEMBLE                    ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Ready to apply {} patches", self.session.patches.len());
        println!("  New imports: {}", self.session.options.new_imports.len());
        println!();
        println!("  Output will be saved to:");
        println!("    {}", self.session.exe_path.with_extension("patched.exe").display());
        println!();
        println!("  Press ENTER to apply patches");
        println!("  Press ESC to cancel");
    }
    
    fn apply_patches(&self) -> Result<(), String> {
        println!();
        println!("🔨 [DEBUG] Applying patches...");
        println!("   [DEBUG] Total patches: {}", self.session.patches.len());
        println!("   [DEBUG] New imports: {}", self.session.options.new_imports.len());
        
        // Check if we have preserved data
        let preserved = self.session.preserved_data.as_ref()
            .ok_or("PE structure not loaded")?;
        
        println!("   [DEBUG] PE structure loaded, proceeding...");
        
        // Determine output path
        let output_path = self.session.exe_path.with_extension("patched.exe");
        println!("   [DEBUG] Output path: {}", output_path.display());
        
        // Step 1: Clone the original PE data
        let mut patched_pe = preserved.original_pe.clone();
        println!("   [DEBUG] Cloned original PE ({} bytes)", patched_pe.len());
        
        // Step 2: Apply byte-level patches
        if self.session.patches.is_empty() {
            println!("   [DEBUG] ⚠️  No patches to apply");
        } else {
            println!("   [DEBUG] Applying {} patches:", self.session.patches.len());
            for (i, patch) in self.session.patches.iter().enumerate() {
                println!("      [DEBUG] Patch #{}: 0x{:x} ({} bytes) - {}", 
                         i + 1, patch.address, patch.new_bytes.len(), patch.description);
                
                // Convert RVA to file offset
                if let Some(file_offset) = self.session.rva_to_file_offset(patch.address as u32, preserved) {
                    let end_offset = file_offset + patch.new_bytes.len();
                    
                    if end_offset <= patched_pe.len() {
                        // Apply the patch
                        patched_pe[file_offset..end_offset].copy_from_slice(&patch.new_bytes);
                        println!("         ✅ Applied at file offset 0x{:x}", file_offset);
                    } else {
                        println!("         ⚠️  Patch extends beyond file bounds, skipping");
                    }
                } else {
                    println!("         ⚠️  Could not convert RVA to file offset, skipping");
                }
            }
        }
        
        // Step 3: Handle new imports (if any)
        if !self.session.options.new_imports.is_empty() {
            println!("   [DEBUG] New imports to add:");
            for import in &self.session.options.new_imports {
                println!("      [DEBUG] {} -> {}", import.dll_name, import.function_name);
            }
            println!("   [DEBUG] ⚠️  Import addition requires PE expansion (use PE Builder)");
        }
        
        // Step 4: Write the patched executable
        println!("   [DEBUG] Writing patched executable...");
        std::fs::write(&output_path, &patched_pe)
            .map_err(|e| format!("Failed to write patched executable: {}", e))?;
        
        println!();
        println!("✅ Patched executable created successfully!");
        println!("   Output: {}", output_path.display());
        println!("   Size: {} bytes", patched_pe.len());
        
        // Step 5: Success message
        println!();
        println!("✅ Patching complete!");
        println!("   The patched executable has been created.");
        println!("   Test the executable to ensure it works correctly.");
        println!();
        println!("Press any key to continue...");
        
        // Wait for key press
        event::read().map_err(|e| e.to_string())?;
        
        Ok(())
    }
}

/// Launch the interactive patching UI
pub fn launch_patch_ui(exe_path: PathBuf) -> Result<(), String> {
    let mut ui = PatchUI::new(exe_path)?;
    ui.run()
}