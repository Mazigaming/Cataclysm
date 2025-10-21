use std::fs;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use crossterm::event::{self, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use tui_textarea::TextArea;
use goblin::pe;
use capstone::prelude::*;
use arboard::Clipboard;

mod decompiler;
mod anti_obfuscation;
mod scripting_api;
mod theme_engine;
mod script_editor;
mod assembler;
mod keybinds;
mod menu_system;
mod compiler_tester;
mod custom_compiler;
mod cross_platform_compiler;
mod builtin_assembler;
mod loading_animation;
mod windows_api_db;
mod pe_builder;
mod assembly_relocator;
mod pe_reassembler;
mod patch_ui;
mod native_disassembler;
#[allow(dead_code)]
mod pe_fixer;

#[derive(Clone)]
struct FileItem {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: Option<u64>,
}

#[derive(Clone)]
struct ThemeEditorState {
    selected_section: usize,
    sections: Vec<String>,
    #[allow(dead_code)]
    editing_field: Option<String>,
    modified: bool,
    theme: theme_engine::Theme,
    selected_field: usize,
    fields: Vec<(String, String)>,  // (field_name, field_value)
    editing_value: Option<String>,
}

impl ThemeEditorState {
    fn new() -> Self {
        let theme = theme_engine::create_dark_theme();
        Self {
            selected_section: 0,
            sections: vec![
                "Colors".to_string(),
                "Styles".to_string(),
                "Syntax".to_string(),
                "Metadata".to_string(),
            ],
            editing_field: None,
            modified: false,
            theme,
            selected_field: 0,
            fields: Vec::new(),
            editing_value: None,
        }
    }
    
    fn from_theme(theme: theme_engine::Theme) -> Self {
        let mut state = Self::new();
        state.theme = theme;
        state.update_fields();
        state
    }
    
    fn update_fields(&mut self) {
        self.fields.clear();
        match self.selected_section {
            0 => {
                // Colors section
                self.fields = vec![
                    ("Background".to_string(), self.theme.colors.background.clone()),
                    ("Foreground".to_string(), self.theme.colors.foreground.clone()),
                    ("Primary".to_string(), self.theme.colors.primary.clone()),
                    ("Secondary".to_string(), self.theme.colors.secondary.clone()),
                    ("Accent".to_string(), self.theme.colors.accent.clone()),
                    ("Border".to_string(), self.theme.colors.border.clone()),
                    ("Border Focused".to_string(), self.theme.colors.border_focused.clone()),
                    ("Selection".to_string(), self.theme.colors.selection.clone()),
                    ("Cursor".to_string(), self.theme.colors.cursor.clone()),
                ];
            }
            1 => {
                // Styles section
                self.fields = vec![
                    ("File List FG".to_string(), self.theme.styles.file_list.fg.clone()),
                    ("File List BG".to_string(), self.theme.styles.file_list.bg.clone()),
                    ("Selected FG".to_string(), self.theme.styles.file_list_selected.fg.clone()),
                    ("Selected BG".to_string(), self.theme.styles.file_list_selected.bg.clone()),
                    ("Editor FG".to_string(), self.theme.styles.editor.fg.clone()),
                    ("Editor BG".to_string(), self.theme.styles.editor.bg.clone()),
                ];
            }
            2 => {
                // Syntax section
                self.fields = vec![
                    ("Keyword".to_string(), self.theme.colors.keyword.clone()),
                    ("Function".to_string(), self.theme.colors.function.clone()),
                    ("Variable".to_string(), self.theme.colors.variable.clone()),
                    ("Constant".to_string(), self.theme.colors.constant.clone()),
                    ("String".to_string(), self.theme.colors.string.clone()),
                    ("Comment".to_string(), self.theme.colors.comment.clone()),
                    ("Operator".to_string(), self.theme.colors.operator.clone()),
                    ("Type".to_string(), self.theme.colors.type_name.clone()),
                ];
            }
            3 => {
                // Metadata section
                self.fields = vec![
                    ("Name".to_string(), self.theme.name.clone()),
                    ("Author".to_string(), self.theme.author.clone()),
                    ("Version".to_string(), self.theme.version.clone()),
                    ("Description".to_string(), self.theme.description.clone()),
                ];
            }
            _ => {}
        }
    }
    
    fn save_field_value(&mut self, value: String) {
        if self.selected_field >= self.fields.len() {
            return;
        }
        
        let field_name = &self.fields[self.selected_field].0;
        
        match self.selected_section {
            0 => {
                // Colors section
                match field_name.as_str() {
                    "Background" => self.theme.colors.background = value,
                    "Foreground" => self.theme.colors.foreground = value,
                    "Primary" => self.theme.colors.primary = value,
                    "Secondary" => self.theme.colors.secondary = value,
                    "Accent" => self.theme.colors.accent = value,
                    "Border" => self.theme.colors.border = value,
                    "Border Focused" => self.theme.colors.border_focused = value,
                    "Selection" => self.theme.colors.selection = value,
                    "Cursor" => self.theme.colors.cursor = value,
                    _ => {}
                }
            }
            1 => {
                // Styles section
                match field_name.as_str() {
                    "File List FG" => self.theme.styles.file_list.fg = value,
                    "File List BG" => self.theme.styles.file_list.bg = value,
                    "Selected FG" => self.theme.styles.file_list_selected.fg = value,
                    "Selected BG" => self.theme.styles.file_list_selected.bg = value,
                    "Editor FG" => self.theme.styles.editor.fg = value,
                    "Editor BG" => self.theme.styles.editor.bg = value,
                    _ => {}
                }
            }
            2 => {
                // Syntax section
                match field_name.as_str() {
                    "Keyword" => self.theme.colors.keyword = value,
                    "Function" => self.theme.colors.function = value,
                    "Variable" => self.theme.colors.variable = value,
                    "Constant" => self.theme.colors.constant = value,
                    "String" => self.theme.colors.string = value,
                    "Comment" => self.theme.colors.comment = value,
                    "Operator" => self.theme.colors.operator = value,
                    "Type" => self.theme.colors.type_name = value,
                    _ => {}
                }
            }
            3 => {
                // Metadata section
                match field_name.as_str() {
                    "Name" => self.theme.name = value,
                    "Author" => self.theme.author = value,
                    "Version" => self.theme.version = value,
                    "Description" => self.theme.description = value,
                    _ => {}
                }
            }
            _ => {}
        }
        
        self.modified = true;
        self.update_fields();
    }
}

// Settings editor state
struct SettingsEditorState {
    category: menu_system::SettingsCategory,
    fields: Vec<(String, String)>,  // (field_name, field_value)
    selected_field: usize,
    editing_value: Option<String>,
    modified: bool,
}

impl SettingsEditorState {
    fn new(category: menu_system::SettingsCategory) -> Self {
        let mut state = Self {
            category: category.clone(),
            fields: Vec::new(),
            selected_field: 0,
            editing_value: None,
            modified: false,
        };
        state.update_fields();
        state
    }
    
    fn update_fields(&mut self) {
        self.fields.clear();
        match self.category {
            menu_system::SettingsCategory::General => {
                self.fields.push(("Auto Save".to_string(), "true".to_string()));
                self.fields.push(("Auto Backup".to_string(), "false".to_string()));
                self.fields.push(("Show Hidden Files".to_string(), "false".to_string()));
                self.fields.push(("Default Language".to_string(), "Rust".to_string()));
                self.fields.push(("Line Numbers".to_string(), "true".to_string()));
            }
            menu_system::SettingsCategory::Appearance => {
                self.fields.push(("Font Size".to_string(), "14".to_string()));
                self.fields.push(("Tab Size".to_string(), "4".to_string()));
                self.fields.push(("Show Whitespace".to_string(), "false".to_string()));
                self.fields.push(("Cursor Style".to_string(), "Block".to_string()));
                self.fields.push(("Line Wrap".to_string(), "false".to_string()));
            }
            menu_system::SettingsCategory::Decompiler => {
                self.fields.push(("Optimization Level".to_string(), "2".to_string()));
                self.fields.push(("Show Assembly".to_string(), "true".to_string()));
                self.fields.push(("Detect Crypto".to_string(), "true".to_string()));
                self.fields.push(("Filter Junk".to_string(), "true".to_string()));
                self.fields.push(("Max Instructions".to_string(), "5000".to_string()));
            }
            menu_system::SettingsCategory::Scripts => {
                self.fields.push(("Auto Run Scripts".to_string(), "false".to_string()));
                self.fields.push(("Script Timeout".to_string(), "30".to_string()));
                self.fields.push(("Enable Python".to_string(), "true".to_string()));
                self.fields.push(("Enable Lua".to_string(), "true".to_string()));
                self.fields.push(("Script Directory".to_string(), "./scripts".to_string()));
            }
            menu_system::SettingsCategory::Advanced => {
                self.fields.push(("Debug Mode".to_string(), "false".to_string()));
                self.fields.push(("Log Level".to_string(), "Info".to_string()));
                self.fields.push(("Cache Size (MB)".to_string(), "100".to_string()));
                self.fields.push(("Thread Count".to_string(), "4".to_string()));
                self.fields.push(("Memory Limit (MB)".to_string(), "512".to_string()));
            }
            _ => {}
        }
    }
    
    fn save_field_value(&mut self, value: String) {
        if self.selected_field < self.fields.len() {
            self.fields[self.selected_field].1 = value;
            self.modified = true;
        }
    }
}

// Confirmation dialog state
struct ConfirmDialog {
    message: String,
    on_confirm: ConfirmAction,
    #[allow(dead_code)]
    previous_mode: Box<Mode>,
}

#[derive(Clone)]
enum ConfirmAction {
    ResetSettings,
    #[allow(dead_code)]
    DeleteTheme(String),
    DeleteScript(String),
}

// File dialog state for import/export
struct FileDialog {
    title: String,
    file_name: String,
    action: FileDialogAction,
    #[allow(dead_code)]
    previous_mode: Box<Mode>,
}

#[derive(Clone)]
enum FileDialogAction {
    ImportSettings,
    ExportSettings,
    ImportTheme,
    ExportTheme(String),
    ImportScript,
    ExportScript(String),
    ImportKeybinds,
    ExportKeybinds,
}

enum Mode {
    List,
    LanguageSelect { options: Vec<String>, selected: usize, file_path: PathBuf },
    OutputModeSelect { options: Vec<String>, selected: usize, file_path: PathBuf, language_idx: usize },
    Edit { textarea: TextArea<'static>, file_path: PathBuf, #[allow(dead_code)] language: String },
    MultiFileEdit { 
        files: Vec<(String, String)>,  // (filename, content) pairs
        current_file_idx: usize,
        textarea: TextArea<'static>,
        file_path: PathBuf,
        language: String,
    },
    CompilationResults {
        results: String,
        #[allow(dead_code)]
        previous_mode: Box<Mode>,
        test_result: Option<crate::compiler_tester::TestResult>,
    },
    #[allow(dead_code)]
    ScriptEditor { editor: script_editor::ScriptEditor },
    ThemeSelector { menu: menu_system::Menu, theme_engine: theme_engine::ThemeEngine },
    ScriptManager { menu: menu_system::Menu },
    Settings { menu: menu_system::Menu },
    SettingsEditor { editor_state: SettingsEditorState },
    KeybindsEditor { menu: menu_system::Menu },
    ThemeEditor { theme_name: String, editor_state: ThemeEditorState },
    ConfirmDialog { dialog: ConfirmDialog },
    FileDialog { dialog: FileDialog },
}

fn is_text_file(path: &PathBuf) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), 
            "txt" | "rs" | "md" | "json" | "toml" | "yml" | "yaml" | "xml" | 
"html" | "css" | "js" | "py" | "sh" | "bat" | "c" | "h" | "cpp" | 
            "hpp" | "asm" | "s" | "pseudo" | "dctheme" | "lua" | "dcscript"
        )
    } else {
        false
    }
}

fn is_exe_file(path: &PathBuf) -> bool {
    // Support .exe, .dll, .sys, .ocx, .cpl, .scr and other PE files
    path.extension().map_or(false, |e| {
        let ext = e.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "exe" | "dll" | "sys" | "ocx" | "cpl" | "scr" | "drv" | "efi")
    })
}

fn disassemble_exe(path: &PathBuf) -> Result<String, String> {
    // SAFETY: Check file size before loading to prevent crashes on huge files
    const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB limit
    
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;
    
    let file_size = metadata.len();
    
    if file_size > MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {} MB (max {} MB)\n\n\
            Large debug builds cannot be decompiled efficiently.\n\
            Please compile with --release flag:\n\
            cargo build --release\n\n\
            Or strip debug symbols:\n\
            strip your_program.exe",
            file_size / (1024 * 1024),
            MAX_FILE_SIZE / (1024 * 1024)
        ));
    }
    
    let buffer = fs::read(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Validate PE file format before parsing
    if buffer.len() < 2 {
        return Err(
            "File is too small to be a valid PE executable.\n\n\
            This file does not appear to be a Windows executable (.exe/.dll).\n\
            Please select a valid PE file.".to_string()
        );
    }
    
    // Check for MZ signature (DOS header magic number)
    if buffer[0] != 0x4D || buffer[1] != 0x5A {  // "MZ" in ASCII
        let actual_sig = format!("0x{:02x}{:02x}", buffer[1], buffer[0]);
        return Err(format!(
            "Invalid PE file format (DOS signature mismatch).\n\n\
            Expected: 0x5a4d (\"MZ\" - valid Windows executable)\n\
            Found:    {} (not a PE file)\n\n\
            This file is NOT a Windows executable (.exe/.dll).\n\n\
            Possible causes:\n\
            • The file is corrupted or incomplete\n\
            • The file is a different format (ELF, Mach-O, script, etc.)\n\
            • The file was not compiled successfully\n\
            • You selected the wrong file\n\n\
            Please ensure you're selecting a valid Windows PE executable.",
            actual_sig
        ));
    }
    
    let pe = pe::PE::parse(&buffer)
        .map_err(|e| format!(
            "Failed to parse PE file structure: {}\n\n\
            The file has a valid DOS header but the PE structure is malformed.\n\n\
            Possible causes:\n\
            • The executable is corrupted\n\
            • The file is packed/encrypted (use a unpacker first)\n\
            • The PE headers are damaged\n\
            • The file is not a standard Windows executable\n\n\
            Try:\n\
            • Recompiling the program\n\
            • Using a different executable\n\
            • Unpacking if it's a packed executable",
            e
        ))?;
    
    let mut disassembly = String::new();

    // Detect architecture from PE header
    let is_64bit = pe.is_64;
    let arch_mode = if is_64bit {
        capstone::arch::x86::ArchMode::Mode64
    } else {
        capstone::arch::x86::ArchMode::Mode32
    };

    let cs = Capstone::new()
        .x86()
        .mode(arch_mode)
        .syntax(capstone::arch::x86::ArchSyntax::Intel)
        .detail(true)
        .build()
        .map_err(|e| format!("Failed to initialize disassembler: {}", e))?;

    // Get entry point to know where actual code starts
    let entry_point = pe.entry as u64;
    let _image_base = pe.image_base as u64;

    // PERFORMANCE FIX: Limit disassembly to reasonable size
    const MAX_INSTRUCTIONS: usize = 50000; // Limit to 50k instructions
    const MAX_SECTION_SIZE: usize = 1024 * 1024; // 1MB per section max
    let mut total_instructions = 0;
    let mut total_nops = 0; // Track NOP instructions to detect data disassembly

    for section in &pe.sections {
        if section.characteristics & pe::section_table::IMAGE_SCN_MEM_EXECUTE != 0 {
            let start = section.pointer_to_raw_data as usize;
            let virtual_size = section.virtual_size as usize;
            let raw_size = section.size_of_raw_data as usize;
            
            // Check if entry point is in this section FIRST
            let section_va = section.virtual_address as u64;
            let section_end_va = section_va + raw_size.max(virtual_size) as u64;
            let entry_in_section = entry_point >= section_va && entry_point < section_end_va;
            
            // OPTIMIZATION: For small executables, ONLY disassemble the section with the entry point
            // This prevents disassembling data sections that are marked as executable
            if file_size < 100_000 && !entry_in_section {
                disassembly.push_str(&format!("; Section: {} (VA: 0x{:X}) - SKIPPED (entry point not here)\n", 
                    String::from_utf8_lossy(&section.name).trim_end_matches('\0'),
                    section_va));
                continue;
            }
            
            // FIX: Use raw_size primarily, but cap at virtual_size if it's reasonable
            // Rust executables often have virtual_size < raw_size due to alignment
            let mut size = if virtual_size > 0 && virtual_size < raw_size && virtual_size > 0x100 {
                // If virtual_size is reasonable (>256 bytes), use it
                virtual_size
            } else {
                // Otherwise use raw_size (actual data on disk)
                raw_size
            };
            
            // PERFORMANCE FIX: Limit section size
            if size > MAX_SECTION_SIZE {
                size = MAX_SECTION_SIZE;
                disassembly.push_str(&format!("; WARNING: Section truncated to {} bytes for performance\n", MAX_SECTION_SIZE));
            }
            
            if start + size <= buffer.len() && size > 0 {
                // 🔧 CRITICAL FIX: Start disassembly FROM the entry point, not section start!
                // This prevents disassembling padding/data before actual code.
                let (code_start, disasm_va) = if entry_in_section && entry_point >= section_va {
                    let entry_offset_in_section = (entry_point - section_va) as usize;
                    if entry_offset_in_section < size {
                        // Start FROM the entry point
                        (start + entry_offset_in_section, entry_point)
                    } else {
                        // Entry point beyond section? Use section start
                        (start, section_va)
                    }
                } else {
                    // Not the entry section, disassemble from start
                    (start, section_va)
                };
                
                let code_end = start + size;
                if code_start >= code_end {
                    continue; // Skip if entry is at/beyond section end
                }
                
                let code = &buffer[code_start..code_end];
                
                disassembly.push_str(&format!("; Section: {} (VA: 0x{:X}, Size: 0x{:X}, Raw: 0x{:X}{})\n", 
                    String::from_utf8_lossy(&section.name).trim_end_matches('\0'),
                    section_va,
                    size,
                    raw_size,
                    if entry_in_section { ", ENTRY POINT HERE" } else { "" }));
                
                if entry_in_section && disasm_va != section_va {
                    disassembly.push_str(&format!("; 🎯 Starting disassembly from ENTRY POINT at VA: 0x{:X} (skipping {} bytes of padding)\n", 
                        disasm_va, disasm_va - section_va));
                }
                
                match cs.disasm_all(code, disasm_va) {
                    Ok(insns) => {
                        let mut last_addr = disasm_va; // 🔧 FIX: Start from disasm_va, not section_va
                        let mut section_insn_count = 0;
                        let mut consecutive_nops = 0;
                        const MAX_CONSECUTIVE_NOPS: usize = 50; // 🔧 Increased from 10 to 50 - legitimate code can have padding
                        let mut data_pattern_count = 0; // Track data-like patterns
                        
                        // 🔧 FIX: Since we start FROM entry point, ALL instructions are "real code" initially
                        disassembly.push_str("\n; === ENTRY POINT ===\n");
                        
                        for insn in insns.iter() {
                            // PERFORMANCE FIX: Stop if we hit instruction limit
                            if total_instructions >= MAX_INSTRUCTIONS {
                                disassembly.push_str(&format!("; [Truncated: Reached {} instruction limit for performance]\n", MAX_INSTRUCTIONS));
                                disassembly.push_str("; WARNING: This may indicate the disassembler is processing DATA as CODE.\n");
                                disassembly.push_str(";          Consider using C/Rust decompilation instead of assembly.\n");
                                break;
                            }
                            
                            let addr = insn.address();
                            
                            // Stop if we hit a long sequence of zeros (padding)
                            if addr > last_addr + 0x100 {
                                disassembly.push_str("; [padding detected - stopping disassembly]\n");
                                break;
                            }
                            
                            // Filter out obvious junk/padding patterns
                            let mnemonic = insn.mnemonic().unwrap_or("");
                            let operands = insn.op_str().unwrap_or("");
                            
                            // Skip invalid instructions
                            if mnemonic.is_empty() || mnemonic == "invalid" {
                                continue;
                            }
                            
                            // DATA PATTERN DETECTION: Detect when we're disassembling data, not code
                            // Data bytes often disassemble to: add, or, xor, adc, sbb with byte operands
                            let is_data_pattern = matches!(mnemonic, "add" | "or" | "xor" | "adc" | "sbb" | "and") 
                                && (operands.contains("byte ptr") || operands.contains("al,"));
                            
                            if is_data_pattern {
                                data_pattern_count += 1;
                                // If we see 20+ data patterns in a row, we're in data, not code
                                if data_pattern_count >= 20 {
                                    disassembly.push_str(&format!("; [Stopped: {} data-like instructions detected - this is DATA, not CODE]\n", data_pattern_count));
                                    break;
                                }
                            } else {
                                data_pattern_count = 0; // Reset on real instruction
                            }
                            
                            // NOP SEQUENCE DETECTION: Stop if we hit too many consecutive NOPs
                            // This indicates we've moved from code into data/padding
                            if mnemonic == "nop" {
                                consecutive_nops += 1;
                                total_nops += 1; // Track total NOPs for final statistics
                                
                                // 🔧 FIX: Stop on long NOP sequence - indicates padding/data, not code
                                if consecutive_nops >= MAX_CONSECUTIVE_NOPS {
                                    disassembly.push_str(&format!("; [Stopped: {} consecutive NOPs detected - reached end of code section]\n", consecutive_nops));
                                    break;
                                }
                            } else {
                                consecutive_nops = 0; // Reset counter on non-NOP instruction
                            }
                            
                            // UTF-8 SAFETY: Sanitize operands to prevent crashes in decompiler
                            // Binary data can contain invalid UTF-8, null bytes, or BOM characters
                            let operands_safe = operands
                                .replace('\0', "")  // Remove null bytes
                                .replace('\u{feff}', "")  // Remove BOM
                                .chars()
                                .filter(|c| !c.is_control() || c.is_whitespace())  // Keep only printable chars + whitespace
                                .collect::<String>();
                            
                            // 🔧 FIX: Don't add duplicate entry point marker (we already added it at start)
                            
                            disassembly.push_str(&format!("{:08X}  {:<8} {}\n", 
                                addr, mnemonic, operands_safe));
                            last_addr = addr;
                            section_insn_count += 1;
                            total_instructions += 1;
                        }
                        
                        disassembly.push_str(&format!("; Section instructions: {}\n", section_insn_count));
                    }
                    Err(_) => {
                        disassembly.push_str("; Failed to disassemble section\n");
                    }
                }
                disassembly.push_str("\n");
            }
            
            // PERFORMANCE FIX: Stop processing sections if we hit limit
            if total_instructions >= MAX_INSTRUCTIONS {
                disassembly.push_str("; [Remaining sections skipped for performance]\n");
                break;
            }
        }
    }
    
    if disassembly.is_empty() {
        disassembly.push_str("; No executable sections found\n");
    } else {
        disassembly.push_str(&format!("; Total instructions disassembled: {}\n", total_instructions));
        
        // Calculate NOP percentage and warn if suspiciously high
        if total_instructions > 0 {
            let nop_percentage = (total_nops as f64 / total_instructions as f64) * 100.0;
            disassembly.push_str(&format!("; NOP instructions: {} ({:.1}%)\n", total_nops, nop_percentage));
            
            // Warn if >50% NOPs - this indicates data being disassembled as code
            if nop_percentage > 50.0 && total_instructions > 100 {
                disassembly.push_str(";\n");
                disassembly.push_str("; ╔═══════════════════════════════════════════════════════════════════╗\n");
                disassembly.push_str("; ║                         ⚠️  WARNING  ⚠️                            ║\n");
                disassembly.push_str("; ╠═══════════════════════════════════════════════════════════════════╣\n");
                disassembly.push_str(&format!("; ║ This disassembly contains {:.1}% NOP instructions!              ║\n", nop_percentage));
                disassembly.push_str("; ║                                                                   ║\n");
                disassembly.push_str("; ║ This strongly indicates the disassembler is processing DATA       ║\n");
                disassembly.push_str("; ║ sections, padding, or resources as CODE.                          ║\n");
                disassembly.push_str("; ║                                                                   ║\n");
                disassembly.push_str("; ║ ❌ DO NOT attempt to reassemble this code - it will fail!         ║\n");
                disassembly.push_str("; ║                                                                   ║\n");
                disassembly.push_str("; ║ ✅ RECOMMENDED SOLUTION:                                          ║\n");
                disassembly.push_str("; ║    Use C or Rust decompilation instead of assembly output.       ║\n");
                disassembly.push_str("; ║    Decompiled code is compilable and produces working binaries.  ║\n");
                disassembly.push_str("; ╚═══════════════════════════════════════════════════════════════════╝\n");
            }
        }
    }
    
    Ok(disassembly)
}

fn load_file_content(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    // Try to read as UTF-8 first
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(_) => {
            // If UTF-8 fails, read as bytes and convert lossy
            let bytes = fs::read(path)?;
            Ok(String::from_utf8_lossy(&bytes).to_string())
        }
    }
}

fn get_files(path: &PathBuf) -> Vec<FileItem> {
    let mut items = Vec::new();
// Add ".." if not root
    if path.parent().is_some() {
        items.push(FileItem {
            name: "..".to_string(),
            path: path.parent().unwrap().to_path_buf(),
            is_dir: true,
            size: None,
        });
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let size = if is_dir {
                    None
                } else {
                    entry.metadata().ok().map(|m| m.len())
                };
                items.push(FileItem { name, path, is_dir, size });
            }
}
    }
    items.sort_by(|a, b| {
        if a.name == ".." {
            std::cmp::Ordering::Less
        } else if b.name == ".." {
            std::cmp::Ordering::Greater
        } else if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });
    items
}

fn open_file(path: &PathBuf) {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd").args(&["/C", "start", "", &path.to_string_lossy()]).spawn();
    } else {
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
}



fn save_text_file(path: &PathBuf, content: &str, language: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Don't save error messages - they're read-only
    if language == "Error" {
        return Err("Cannot save error messages. Use Ctrl+C to copy the error text.".into());
    }
    
    let mut save_path = path.clone();
    match language {
        "Assembly" => { save_path.set_extension("asm"); },
        "Pseudo Code" => { save_path.set_extension("pseudo"); },
        "C Code" => { save_path.set_extension("c"); },
        "Rust Code" => { save_path.set_extension("rs"); },
        _ => {}
    }
    let mut file = fs::File::create(&save_path)?;
    // Write UTF-8 BOM for better compatibility with Windows tools
    file.write_all(&[0xEF, 0xBB, 0xBF])?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// ============================================================================
// PROJECT FOLDER MANAGEMENT
// ============================================================================

fn get_decompiler_root() -> PathBuf {
    // Get the decompiler root directory (where rust_file_explorer is located)
    env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .and_then(|target_path| target_path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| env::current_dir().unwrap())
}

fn create_project_folder(exe_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let decompiler_root = get_decompiler_root();
    let projects_dir = decompiler_root.join("projects");
    
    // Create projects directory if it doesn't exist
    fs::create_dir_all(&projects_dir)?;
    
    // Get exe name without extension
    let exe_name = exe_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    // Create project folder
    let project_folder = projects_dir.join(&exe_name);
    fs::create_dir_all(&project_folder)?;
    
    Ok(project_folder)
}

fn should_use_project_folder(_exe_path: &PathBuf, _current_path: &PathBuf) -> bool {
    // ALWAYS use project folders for better organization
    // This ensures all decompiled projects are in one place
    true
}

fn save_complete_decompilation(
    exe_path: &PathBuf,
    current_path: &PathBuf,
    asm: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let use_project_folder = should_use_project_folder(exe_path, current_path);
    
    let save_dir = if use_project_folder {
        create_project_folder(exe_path)?
    } else {
        exe_path.parent().unwrap_or(current_path).to_path_buf()
    };
    
    let exe_name = exe_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    // Save full assembly
    let asm_path = save_dir.join(format!("{}_full.asm", exe_name));
    fs::write(&asm_path, asm)?;
    
    // Save all decompiled formats
    let pseudo = decompiler::translate_to_pseudo_with_pe(asm, Some(&exe_path.to_string_lossy()));
    let pseudo_path = save_dir.join(format!("{}_decompiled.pseudo", exe_name));
    fs::write(&pseudo_path, pseudo)?;
    
    let c_code = decompiler::translate_to_c_with_pe(asm, Some(&exe_path.to_string_lossy()));
    let c_path = save_dir.join(format!("{}_decompiled.c", exe_name));
    fs::write(&c_path, c_code)?;
    
    let rust_code = decompiler::translate_to_rust_with_pe(asm, Some(&exe_path.to_string_lossy()));
    let rust_path = save_dir.join(format!("{}_decompiled.rs", exe_name));
    fs::write(&rust_path, rust_code)?;
    
    // Save PE info summary
    let pe_info = extract_pe_info(exe_path);
    let pe_info_path = save_dir.join(format!("{}_pe_info.txt", exe_name));
    fs::write(&pe_info_path, pe_info)?;
    
    // Create README
    let readme = format!(
        "# Decompilation Project: {}\n\n\
        ## Files Generated:\n\
        - `{}_full.asm` - Complete disassembly of all executable sections\n\
        - `{}_decompiled.pseudo` - Pseudo-code representation (with crypto detection)\n\
        - `{}_decompiled.c` - C code decompilation (with crypto detection)\n\
        - `{}_decompiled.rs` - Rust code decompilation (with crypto detection)\n\
        - `{}_pe_info.txt` - PE file metadata and structure\n\n\
        ## Source:\n\
        Original file: {}\n\n\
        ## Decompiler Version:\n\
        Advanced Decompiler v4.0\n\
        Features: PE Parsing, IAT Resolution, Junk Filtering, CFG Recovery, Crypto Detection, Anti-Obfuscation\n\n\
        ## New in v4.0:\n\
        - 🛡️ Anti-obfuscation layer with pattern detection and removal\n\
        - 🔐 Cryptographic algorithm detection (AES, DES, MD5, SHA, RC4, etc.)\n\
        - 📜 Scripting API for custom analysis (Python/Lua)\n\
        - 🎨 Theme engine with CSS-like customization\n\
        - ⚙️ Enhanced assembler with optimization passes\n",
        exe_name, exe_name, exe_name, exe_name, exe_name, exe_name,
        exe_path.display()
    );
    let readme_path = save_dir.join("README.md");
    fs::write(&readme_path, readme)?;
    
    Ok(save_dir)
}

fn extract_pe_info(exe_path: &PathBuf) -> String {
    let mut info = String::new();
    
    info.push_str("╔════════════════════════════════════════════════════════════════╗\n");
    info.push_str("║                    PE FILE INFORMATION                         ║\n");
    info.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");
    
    if let Ok(buffer) = fs::read(exe_path) {
        if let Ok(pe) = pe::PE::parse(&buffer) {
            info.push_str(&format!("File: {}\n", exe_path.display()));
            info.push_str(&format!("Size: {} bytes\n\n", buffer.len()));
            
            info.push_str("═══ HEADERS ═══\n");
            info.push_str(&format!("Image Base: 0x{:x}\n", pe.image_base));
            info.push_str(&format!("Entry Point: 0x{:x}\n", pe.entry));
            info.push_str(&format!("Subsystem: {:?}\n", pe.header.optional_header.unwrap().windows_fields.subsystem));
            info.push_str(&format!("Machine: {:?}\n\n", pe.header.coff_header.machine));
            
            info.push_str("═══ SECTIONS ═══\n");
            for section in &pe.sections {
                let name = String::from_utf8_lossy(&section.name);
                info.push_str(&format!("  {} - VA: 0x{:x}, Size: 0x{:x}, Characteristics: 0x{:x}\n",
                    name.trim_end_matches('\0'),
                    section.virtual_address,
                    section.virtual_size,
                    section.characteristics
                ));
            }
            info.push_str("\n");
            
            info.push_str("═══ IMPORTS ═══\n");
            for import in &pe.imports {
                info.push_str(&format!("  {} ({})\n", import.name, import.dll));
            }
            info.push_str("\n");
            
            info.push_str("═══ EXPORTS ═══\n");
            if pe.exports.is_empty() {
                info.push_str("  (No exports)\n");
            } else {
                for export in &pe.exports {
                    if let Some(name) = export.name {
                        info.push_str(&format!("  {} @ 0x{:x}\n", name, export.rva));
                    }
                }
            }
        } else {
            info.push_str("Error: Failed to parse PE file\n");
        }
    } else {
        info.push_str("Error: Failed to read file\n");
    }
    
    info
}

// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for CLI mode
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = PathBuf::from(&args[1]);
        
        // Check if this is a source file to compile (not an executable to decompile)
        let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match extension.to_lowercase().as_str() {
            "asm" => {
                // Compile assembly code
                println!("🔨 Compiling assembly: {}", file_path.display());
                let result = compiler_tester::compile_and_test(&file_path, "assembly");
                println!("{}", compiler_tester::format_test_results(&result));
                return if result.compilation.success { Ok(()) } else { Err("Compilation failed".into()) };
            }
            "c" => {
                // Compile C code
                println!("🔨 Compiling C code: {}", file_path.display());
                let result = compiler_tester::compile_and_test(&file_path, "c");
                println!("{}", compiler_tester::format_test_results(&result));
                return if result.compilation.success { Ok(()) } else { Err("Compilation failed".into()) };
            }
            "rs" => {
                // Compile Rust code
                println!("🔨 Compiling Rust code: {}", file_path.display());
                let result = compiler_tester::compile_and_test(&file_path, "rust");
                println!("{}", compiler_tester::format_test_results(&result));
                return if result.compilation.success { Ok(()) } else { Err("Compilation failed".into()) };
            }
            _ => {
                // Decompile executable
                println!("🔍 Decompiling: {}", file_path.display());
                
                match disassemble_exe(&file_path) {
                    Ok(asm) => {
                        let output_path = format!("{}.asm", file_path.display());
                        fs::write(&output_path, &asm)?;
                        println!("✅ Assembly saved to: {}", output_path);
                        
                        // Also generate C and Rust
                        let c_code = decompiler::translate_to_c(&asm);
                        let c_path = format!("{}.c", file_path.display());
                        fs::write(&c_path, &c_code)?;
                        println!("✅ C code saved to: {}", c_path);
                        
                        let rust_code = decompiler::translate_to_rust(&asm);
                        let rust_path = format!("{}.rs", file_path.display());
                        fs::write(&rust_path, &rust_code)?;
                        println!("✅ Rust code saved to: {}", rust_path);
                        
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("❌ Error: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }
    }
    
    // GUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_path = env::current_dir().unwrap();
    let mut files = get_files(&current_path);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut mode = Mode::List;
    
    // Initialize keybind manager and theme engine
    let mut keybind_manager = keybinds::KeyBindManager::new();
    let mut theme_engine = theme_engine::ThemeEngine::new();
    let mut current_theme_name = "Dark".to_string();
    let _ = theme_engine.set_theme(&current_theme_name);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            match &mut mode {
                Mode::List => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)].as_ref())
                        .split(size);

                    let title = format!("🗂️  File Explorer - {} | Theme: {}", current_path.display(), current_theme_name);
                    let title_style = theme_engine.get_style("title_bar", theme_engine.get_current_theme());
                    let block = Block::default().title(title).borders(Borders::ALL).style(title_style);
                    f.render_widget(block, chunks[0]);

                    let file_list_style = theme_engine.get_style("file_list", theme_engine.get_current_theme());
                    let items: Vec<ListItem> = files
                        .iter()
                        .map(|item| {
                            let style = if item.name == ".." {
                                file_list_style.fg(theme_engine.get_color("secondary", theme_engine.get_current_theme())).add_modifier(Modifier::BOLD)
                            } else if item.is_dir {
                                file_list_style.fg(theme_engine.get_color("primary", theme_engine.get_current_theme())).add_modifier(Modifier::BOLD)
                            } else if is_exe_file(&item.path) {
                                file_list_style.fg(theme_engine.get_color("accent", theme_engine.get_current_theme()))
                            } else if is_text_file(&item.path) {
                                file_list_style.fg(theme_engine.get_color("string", theme_engine.get_current_theme()))
                            } else {
                                file_list_style
                            };
                            let text = if item.name == ".." {
                                "<DIR> ..".to_string()
                            } else if let Some(size) = item.size {
                                format!("{} ({} bytes)", item.name, size)
                            } else {
                                format!("<DIR> {}", item.name)
                            };
                            ListItem::new(Span::styled(text, style))
                        })
                        .collect();

                    let file_list_selected_style = theme_engine.get_style("file_list_selected", theme_engine.get_current_theme());
                    let list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title("Files").style(file_list_style))
                        .highlight_style(file_list_selected_style);
                    f.render_stateful_widget(list, chunks[1], &mut state);

                    // Help bar
                    let help_text = "↑↓: Navigate | Enter: Open | Ctrl+T: Themes | Ctrl+E: Scripts | Ctrl+,: Settings | F1: Keybinds | Q/Esc: Quit";
                    let status_style = theme_engine.get_style("status_bar", theme_engine.get_current_theme());
                    let help_block = Block::default()
                        .borders(Borders::ALL)
                        .style(status_style);
                    let help_para = Paragraph::new(help_text).block(help_block);
                    f.render_widget(help_para, chunks[2]);
                }
                Mode::LanguageSelect { options, selected, file_path } => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(6)].as_ref())
                        .split(size);

                    let title = format!("Select Language for {}", file_path.display());
                    let block = Block::default().title(title).borders(Borders::ALL);
                    f.render_widget(block, chunks[0]);

                    let items: Vec<ListItem> = options
                        .iter()
                        .enumerate()
                        .map(|(i, opt)| {
                            let style = if i == *selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
                            ListItem::new(Span::styled(opt.clone(), style))
                        })
                        .collect();

                    let list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title("Options"))
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                    let mut state = ListState::default();
                    state.select(Some(*selected));
                    f.render_stateful_widget(list, chunks[1], &mut state);
                    
                    // Show helpful hints based on selected option
                    let hint_text = match *selected {
                        0 => "⚠️  Assembly: Raw assembly code. Built-in assembler CANNOT handle external API calls.\n\
                              Use this only for self-contained code. For real executables, use C or Rust output.\n\
                              ℹ️  Executables with Windows API calls will fail to reassemble.",
                        1 => "📝 Pseudo Code: High-level pseudocode representation.\n\
                              Easy to read but not compilable. Good for understanding program logic.\n\
                              ℹ️  Best for analysis and documentation.",
                        2 => "✅ C Code: Compilable C code with Windows API declarations.\n\
                              Includes proper headers and function declarations. Can be compiled with gcc/MSVC.\n\
                              ℹ️  RECOMMENDED for real executables with API calls.",
                        3 => "✅ Rust Code: Compilable Rust code with FFI bindings.\n\
                              Includes proper extern blocks and unsafe wrappers. Can be compiled with rustc.\n\
                              ℹ️  RECOMMENDED for real executables with API calls.",
                        _ => "",
                    };
                    
                    let hint_block = Block::default()
                        .title("💡 Info")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Yellow));
                    let hint_para = Paragraph::new(hint_text).block(hint_block);
                    f.render_widget(hint_para, chunks[2]);
                }
                Mode::OutputModeSelect { options, selected, file_path, language_idx: _ } => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                        .split(size);

                    let title = format!("Select Output Mode - {}", file_path.display());
                    let block = Block::default().title(title).borders(Borders::ALL);
                    f.render_widget(block, chunks[0]);

                    let items: Vec<ListItem> = options
                        .iter()
                        .enumerate()
                        .map(|(i, opt)| {
                            let style = if i == *selected { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
                            ListItem::new(Span::styled(opt.clone(), style))
                        })
                        .collect();

                    let list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title("Output Mode"))
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                    let mut state = ListState::default();
                    state.select(Some(*selected));
                    f.render_stateful_widget(list, chunks[1], &mut state);
                }
                Mode::Edit { textarea, file_path, language } => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                        .split(size);

                    let lang_lower = language.to_lowercase();
                    let can_compile = lang_lower.contains("c") || lang_lower.contains("rust") || lang_lower.contains("assembly") || lang_lower.contains("asm");
                    let compile_hint = if can_compile { " | F5: Compile & Test" } else { "" };
                    
                    // Special handling for error messages
                    let (title, block_color) = if language == "Error" {
                        (format!("Error: {} - Ctrl+C: Copy Error | Esc: Back", file_path.display()), Color::Red)
                    } else {
                        (format!("Editing: {} [{}] - Ctrl+C: Copy | Ctrl+S: Save | Esc: Save & Exit{}", 
                            file_path.display(), language, compile_hint), Color::Green)
                    };
                    let block = Block::default().title(title).borders(Borders::ALL).style(Style::default().fg(block_color));
                    f.render_widget(block, chunks[0]);

                    f.render_widget(textarea.widget(), chunks[1]);
                }
                Mode::MultiFileEdit { files, current_file_idx, textarea, .. } => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                        .split(size);

                    let title = format!("File: {} [{}/{}] - Use Ctrl+Left/Right to navigate | Esc to save & exit", 
                        files[*current_file_idx].0, 
                        *current_file_idx + 1, 
                        files.len());
                    let block = Block::default().title(title).borders(Borders::ALL).style(Style::default().fg(Color::Cyan));
                    f.render_widget(block, chunks[0]);

                    f.render_widget(textarea.widget(), chunks[1]);
                }
                Mode::ScriptEditor { .. } => {
                    // TODO: Implement full script editor UI
                    let block = Block::default()
                        .title("📜 Script Editor - Press F1 for help | Esc to exit")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Cyan));
                    f.render_widget(block, size);
                }
                Mode::ThemeSelector { menu, .. } => {
                    menu.render(f, size);
                }
                Mode::ScriptManager { menu } => {
                    menu.render(f, size);
                }
                Mode::Settings { menu } => {
                    menu.render(f, size);
                }
                Mode::KeybindsEditor { menu } => {
                    menu.render(f, size);
                }
                Mode::CompilationResults { results, test_result, .. } => {
                    let mut textarea = TextArea::new(results.lines().map(|s| s.to_string()).collect());
                    
                    // Check if we have an executable to run
                    let has_executable = test_result.as_ref()
                        .and_then(|tr| tr.compilation.executable_path.as_ref())
                        .is_some();
                    
                    let title = if has_executable {
                        "🔨 Compilation & Test Results - F5: Run Program | Ctrl+C: Copy | Any key: Return"
                    } else {
                        "🔨 Compilation & Test Results - Ctrl+C: Copy | Any key: Return"
                    };
                    
                    textarea.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(title)
                            .style(Style::default().fg(Color::Yellow))
                    );
                    f.render_widget(textarea.widget(), size);
                }
                Mode::ThemeEditor { theme_name, editor_state } => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)].as_ref())
                        .split(size);

                    let title = format!("🎨 Theme Editor: {} {}", theme_name, if editor_state.modified { "*" } else { "" });
                    let title_block = Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Cyan));
                    f.render_widget(title_block, chunks[0]);

                    // Split main area into sections and fields
                    let main_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                        .split(chunks[1]);

                    // Theme editor sections (left panel)
                    let sections: Vec<ListItem> = editor_state.sections
                        .iter()
                        .enumerate()
                        .map(|(i, section)| {
                            let style = if i == editor_state.selected_section {
                                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            };
                            ListItem::new(section.clone()).style(style)
                        })
                        .collect();

                    let list = List::new(sections)
                        .block(Block::default().borders(Borders::ALL).title("Sections"));
                    f.render_widget(list, main_chunks[0]);

                    // Fields (right panel)
                    if editor_state.editing_value.is_some() {
                        // Show input field for editing
                        let editing_value = editor_state.editing_value.as_ref().unwrap();
                        let field_name = if editor_state.selected_field < editor_state.fields.len() {
                            &editor_state.fields[editor_state.selected_field].0
                        } else {
                            "Unknown"
                        };
                        
                        let input_text = format!("Editing: {}\n\nValue: {}_", field_name, editing_value);
                        let input_para = Paragraph::new(input_text)
                            .block(Block::default()
                                .borders(Borders::ALL)
                                .title("Edit Field")
                                .style(Style::default().fg(Color::Yellow)));
                        f.render_widget(input_para, main_chunks[1]);
                    } else {
                        // Show field list
                        let fields: Vec<ListItem> = editor_state.fields
                            .iter()
                            .enumerate()
                            .map(|(i, (name, value))| {
                                let style = if i == editor_state.selected_field {
                                    Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(Color::White)
                                };
                                let text = format!("{}: {}", name, value);
                                ListItem::new(text).style(style)
                            })
                            .collect();

                        let fields_list = List::new(fields)
                            .block(Block::default().borders(Borders::ALL).title("Fields"));
                        f.render_widget(fields_list, main_chunks[1]);
                    }

                    let help_text = if editor_state.editing_value.is_some() {
                        "Type to edit | Enter: Save | Esc: Cancel"
                    } else {
                        "Tab: Switch panel | ↑↓: Navigate | Enter: Edit field | Ctrl+S: Save theme | Esc: Back"
                    };
                    let help_block = Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Gray));
                    let help_para = Paragraph::new(help_text).block(help_block);
                    f.render_widget(help_para, chunks[2]);
                }
                Mode::SettingsEditor { editor_state } => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)].as_ref())
                        .split(size);

                    let category_name = match editor_state.category {
                        menu_system::SettingsCategory::General => "General",
                        menu_system::SettingsCategory::Appearance => "Appearance",
                        menu_system::SettingsCategory::Keybinds => "Keybinds",
                        menu_system::SettingsCategory::Decompiler => "Decompiler",
                        menu_system::SettingsCategory::Scripts => "Scripts",
                        menu_system::SettingsCategory::Advanced => "Advanced",
                    };

                    let title = format!("⚙️  Settings: {} {}", category_name, if editor_state.modified { "*" } else { "" });
                    let title_block = Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Cyan));
                    f.render_widget(title_block, chunks[0]);

                    // Render fields
                    let fields: Vec<ListItem> = editor_state.fields
                        .iter()
                        .enumerate()
                        .map(|(i, (name, value))| {
                            let display_value = if editor_state.editing_value.is_some() && i == editor_state.selected_field {
                                editor_state.editing_value.as_ref().unwrap().clone()
                            } else {
                                value.clone()
                            };
                            
                            let style = if i == editor_state.selected_field {
                                if editor_state.editing_value.is_some() {
                                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                                }
                            } else {
                                Style::default().fg(Color::White)
                            };
                            
                            let text = format!("{}: {}", name, display_value);
                            ListItem::new(text).style(style)
                        })
                        .collect();

                    let fields_list = List::new(fields)
                        .block(Block::default().borders(Borders::ALL).title("Settings"))
                        .style(Style::default().fg(Color::White));
                    f.render_widget(fields_list, chunks[1]);

                    let help_text = if editor_state.editing_value.is_some() {
                        "Type to edit | Enter: Save | Esc: Cancel"
                    } else {
                        "↑↓: Navigate | Enter: Edit | Ctrl+S: Save settings | Esc: Back"
                    };
                    let help_block = Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Gray));
                    let help_para = Paragraph::new(help_text).block(help_block);
                    f.render_widget(help_para, chunks[2]);
                }
                Mode::ConfirmDialog { dialog } => {
                    // Center the dialog
                    let area = centered_rect(60, 30, size);
                    
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                        .split(area);

                    let message_block = Block::default()
                        .title("⚠️  Confirmation")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Yellow));
                    let message_para = Paragraph::new(dialog.message.as_str())
                        .block(message_block)
                        .wrap(ratatui::widgets::Wrap { trim: true });
                    f.render_widget(message_para, chunks[0]);

                    let help_block = Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Gray));
                    let help_para = Paragraph::new("Y: Confirm | N/Esc: Cancel").block(help_block);
                    f.render_widget(help_para, chunks[1]);
                }
                Mode::FileDialog { dialog } => {
                    // Center the dialog
                    let area = centered_rect(60, 30, size);
                    
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)].as_ref())
                        .split(area);

                    let title_block = Block::default()
                        .title(dialog.title.as_str())
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Cyan));
                    f.render_widget(title_block, chunks[0]);

                    let input_block = Block::default()
                        .title("File Name")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Green));
                    let input_para = Paragraph::new(dialog.file_name.as_str()).block(input_block);
                    f.render_widget(input_para, chunks[1]);

                    let help_block = Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Gray));
                    let help_para = Paragraph::new("Type filename | Enter: Confirm | Esc: Cancel").block(help_block);
                    f.render_widget(help_para, chunks[2]);
                }
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match &mut mode {
                    Mode::List => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Open theme selector
                                let themes = theme_engine.list_themes();
                                let menu = menu_system::create_theme_menu(themes, &current_theme_name);
                                mode = Mode::ThemeSelector { menu, theme_engine: theme_engine.clone() };
                            }
                            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Open script manager
                                let scripts = vec![
                                    ("crypto_detector.py".to_string(), "Python".to_string()),
                                    ("string_extractor.lua".to_string(), "Lua".to_string()),
                                ];
                                let menu = menu_system::create_script_menu(scripts);
                                mode = Mode::ScriptManager { menu };
                            }
                            KeyCode::Char(',') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Open settings
                                let menu = menu_system::create_settings_menu();
                                mode = Mode::Settings { menu };
                            }
                            KeyCode::F(1) => {
                                // Open keybinds editor
                                let menu = menu_system::create_keybinds_menu(&keybind_manager);
                                mode = Mode::KeybindsEditor { menu };
                            }
                            KeyCode::F(3) => {
                                // F3: Quick disassemble selected .exe file
                                if let Some(i) = state.selected() {
                                    let item = &files[i];
                                    if is_exe_file(&item.path) {
                                        println!("🔍 [F3] Disassembling: {}", item.path.display());
                                        
                                        match disassemble_exe(&item.path) {
                                            Ok(asm) => {
                                                let output_path = format!("{}.asm", item.path.display());
                                                if let Err(e) = fs::write(&output_path, &asm) {
                                                    println!("❌ Failed to write assembly: {}", e);
                                                } else {
                                                    println!("✅ Assembly saved to: {}", output_path);
                                                    
                                                    // Open in editor
                                                    let textarea = TextArea::new(asm.lines().map(|s| s.to_string()).collect());
                                                    mode = Mode::Edit { 
                                                        textarea, 
                                                        file_path: PathBuf::from(output_path), 
                                                        language: "Assembly".to_string() 
                                                    };
                                                }
                                            }
                                            Err(e) => {
                                                println!("❌ Disassembly failed: {}", e);
                                            }
                                        }
                                    } else {
                                        println!("⚠️  F3 only works on .exe files");
                                    }
                                }
                            }
                            KeyCode::F(6) => {
                                // F6: Launch interactive patch UI
                                if let Some(i) = state.selected() {
                                    let item = &files[i];
                                    if is_exe_file(&item.path) {
                                        println!("🔧 [F6] Launching patch UI for: {}", item.path.display());
                                        
                                        match patch_ui::launch_patch_ui(item.path.clone()) {
                                            Ok(_) => {
                                                println!("✅ Patch UI closed");
                                            }
                                            Err(e) => {
                                                println!("❌ Patch UI error: {}", e);
                                            }
                                        }
                                    } else {
                                        println!("⚠️  F6 only works on .exe files");
                                    }
                                }
                            }
                            KeyCode::Up => {
                                let i = state.selected().unwrap_or(0);
                                if i > 0 {
                                    state.select(Some(i - 1));
                                }
                            }
                            KeyCode::Down => {
                                let i = state.selected().unwrap_or(0);
                                if i < files.len() - 1 {
                                    state.select(Some(i + 1));
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(i) = state.selected() {
                                    let item = &files[i];
                                    if item.is_dir {
                                        current_path = item.path.clone();
                                        files = get_files(&current_path);
                                        state.select(if files.len() > 1 { Some(1) } else { Some(0) });
                                    } else if is_text_file(&item.path) {
                                        if let Ok(content) = load_file_content(&item.path) {
                                            let textarea = TextArea::new(content.lines().map(|s| s.to_string()).collect());
                                            // Detect language from file extension
                                            let language = if let Some(ext) = item.path.extension() {
                                                let ext = ext.to_string_lossy().to_lowercase();
                                                match ext.as_str() {
                                                    "asm" | "s" => "Assembly",
                                                    "c" | "h" => "C Code",
                                                    "rs" => "Rust Code",
                                                    "cpp" | "hpp" => "C++ Code",
                                                    "py" => "Python",
                                                    "js" => "JavaScript",
                                                    "pseudo" => "Pseudo Code",
                                                    _ => "Text",
                                                }
                                            } else {
                                                "Text"
                                            };
                                            mode = Mode::Edit { textarea, file_path: item.path.clone(), language: language.to_string() };
                                        }
                                    } else if is_exe_file(&item.path) {
                                        let options = vec!["Assembly".to_string(), "Pseudo Code".to_string(), "C Code".to_string(), "Rust Code".to_string()];
                                        mode = Mode::LanguageSelect { options, selected: 0, file_path: item.path.clone() };
                                    } else {
                                        open_file(&item.path);
                                    }
                                }
                            }
                            _ => {}
                        }
                    },
                    Mode::LanguageSelect { options, selected, file_path } => {
                    match key.code {
                        KeyCode::Up => {
                            if *selected > 0 {
                                *selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if *selected < options.len() - 1 {
                                *selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            // Move to output mode selection
                            let output_options = vec![
                                "Single File".to_string(), 
                                "Multi-File (by type)".to_string(), 
                                "Multi-File (by function)".to_string()
                            ];
                            mode = Mode::OutputModeSelect { 
                                options: output_options, 
                                selected: 0, 
                                file_path: file_path.clone(), 
                                language_idx: *selected 
                            };
                        }
                        KeyCode::Esc => {
                            mode = Mode::List;
                        }
                            _ => {}
                        }
                        },
                    Mode::OutputModeSelect { options, selected, file_path, language_idx } => {
                        match key.code {
                            KeyCode::Up => {
                                if *selected > 0 {
                                    *selected -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if *selected < options.len() - 1 {
                                    *selected += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let language_options = vec!["Assembly", "Pseudo Code", "C Code", "Rust Code"];
                                let language = language_options[*language_idx];
                                let output_mode = &options[*selected];
                                
                                // SAFETY: Properly handle disassembly errors instead of crashing
                                let asm = match disassemble_exe(file_path) {
                                    Ok(assembly) => assembly,
                                    Err(error_msg) => {
                                        // Show error in editor so user can see what went wrong
                                        let error_content = format!(
                                            "; ========================================\n\
                                             ; DECOMPILATION ERROR\n\
                                             ; ========================================\n\
                                             ;\n\
                                             ; {}\n\
                                             ;\n\
                                             ; ========================================\n",
                                            error_msg
                                        );
                                        let textarea = TextArea::new(error_content.lines().map(|s| s.to_string()).collect());
                                        mode = Mode::Edit { 
                                            textarea, 
                                            file_path: file_path.clone(), 
                                            language: "Error".to_string() 
                                        };
                                        continue;
                                    }
                                };
                                
                                let pe_path_str = file_path.to_string_lossy().to_string();
                                
                                if output_mode == "Single File" {
                                    // Single file mode - open in editor with full PE analysis
                                    let content = match *language_idx {
                                        0 => asm.clone(),
                                        1 => decompiler::translate_to_pseudo_with_pe(&asm, Some(&pe_path_str)),
                                        2 => decompiler::translate_to_c_with_pe(&asm, Some(&pe_path_str)),
                                        3 => decompiler::translate_to_rust_with_pe(&asm, Some(&pe_path_str)),
                                        _ => "Unknown option".to_string(),
                                    };
                                    
                                    // Generate proper output file path based on language
                                    let output_file_path = match *language_idx {
                                        0 => file_path.with_extension("asm"),
                                        1 => file_path.with_extension("pseudo"),
                                        2 => file_path.with_extension("c"),
                                        3 => file_path.with_extension("rs"),
                                        _ => file_path.with_extension("txt"),
                                    };
                                    
                                    let textarea = TextArea::new(content.lines().map(|s| s.to_string()).collect());
                                    mode = Mode::Edit { textarea, file_path: output_file_path, language: language.to_string() };
                                } else {
                                    // Multi-file mode - save to project folder and navigate
                                    if let Ok(project_dir) = save_complete_decompilation(file_path, &current_path, &asm) {
                                        // Successfully saved, now navigate to the project folder
                                        current_path = project_dir;
                                        files = get_files(&current_path);
                                        state.select(Some(0));
                                        mode = Mode::List;
                                    } else {
                                        // Fallback to multi-file edit mode
                                        let files = decompiler::generate_multi_file_output(&asm, language, output_mode);
                                        if !files.is_empty() {
                                            let textarea = TextArea::new(files[0].1.lines().map(|s| s.to_string()).collect());
                                            mode = Mode::MultiFileEdit {
                                                files,
                                                current_file_idx: 0,
                                                textarea,
                                                file_path: file_path.clone(),
                                                language: language.to_string(),
                                            };
                                        }
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                mode = Mode::List;
                            }
                            _ => {}
                        }
                    },
                Mode::Edit { textarea, file_path, language } => {
                    match key.code {
                        KeyCode::Esc => {
                            if let Ok(_) = save_text_file(file_path, &textarea.lines().join("\n"), language) {
                                // Saved
                            }
                            mode = Mode::List;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Copy all content to clipboard
                            let content = textarea.lines().join("\n");
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(content);
                                // Visual feedback could be added here
                            }
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if let Ok(_) = save_text_file(file_path, &textarea.lines().join("\n"), language) {
                                // Saved
                            }
                        }
                        KeyCode::F(5) => {
                            // Compile & Test - for C, Rust, and Assembly
                            let lang_lower = language.to_lowercase();
                            let can_compile = lang_lower.contains("c") || lang_lower.contains("rust") || lang_lower.contains("assembly") || lang_lower.contains("asm");
                            
                            if can_compile {
                                // Save first
                                if let Ok(_) = save_text_file(file_path, &textarea.lines().join("\n"), language) {
                                    // Determine the actual language for the compiler
                                    let compiler_lang = if lang_lower.contains("rust") {
                                        "rust"
                                    } else if lang_lower.contains("assembly") || lang_lower.contains("asm") {
                                        "assembly"
                                    } else if lang_lower.contains("c") {
                                        "c"
                                    } else {
                                        &lang_lower
                                    };
                                    
                                    // Run compilation and testing
                                    let test_result = compiler_tester::compile_and_test(file_path, compiler_lang);
                                    let formatted_result = compiler_tester::format_test_results(&test_result);
                                    
                                    // Save current mode and switch to results view
                                    let current_mode = Mode::Edit {
                                        textarea: textarea.clone(),
                                        file_path: file_path.clone(),
                                        language: language.clone(),
                                    };
                                    mode = Mode::CompilationResults {
                                        results: formatted_result,
                                        previous_mode: Box::new(current_mode),
                                        test_result: Some(test_result),
                                    };
                                }
                            }
                        }
                        _ => {
                            textarea.input(key);
                        }
                    }
                }
                Mode::MultiFileEdit { files, current_file_idx, textarea, file_path, language: _ } => {
                    match key.code {
                        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Navigate to previous file
                            if *current_file_idx > 0 {
                                *current_file_idx -= 1;
                                *textarea = TextArea::new(files[*current_file_idx].1.lines().map(|s| s.to_string()).collect());
                            }
                        }
                        KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Navigate to next file
                            if *current_file_idx < files.len() - 1 {
                                *current_file_idx += 1;
                                *textarea = TextArea::new(files[*current_file_idx].1.lines().map(|s| s.to_string()).collect());
                            }
                        }
                        KeyCode::Esc => {
                            // Save all files
                            for (filename, content) in files.iter() {
                                let mut save_path = file_path.clone();
                                save_path.set_file_name(filename);
                                if let Ok(mut file) = fs::File::create(&save_path) {
                                    let _ = file.write_all(content.as_bytes());
                                }
                            }
                            mode = Mode::List;
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Save all files
                            for (filename, content) in files.iter() {
                                let mut save_path = file_path.clone();
                                save_path.set_file_name(filename);
                                if let Ok(mut file) = fs::File::create(&save_path) {
                                    let _ = file.write_all(content.as_bytes());
                                }
                            }
                        }
                        _ => {
                            textarea.input(key);
                        }
                    }
                }
                Mode::ScriptEditor { .. } => {
                    // TODO: Implement full script editor key handling
                    match key.code {
                        KeyCode::Esc => {
                            mode = Mode::List;
                        }
                        _ => {}
                    }
                }
                Mode::ThemeSelector { menu, theme_engine: theme_eng } => {
                    match key.code {
                        KeyCode::Up => menu.move_up(),
                        KeyCode::Down => menu.move_down(),
                        KeyCode::Right | KeyCode::Enter => {
                            // Check if current item has a submenu
                            if let Some(item) = menu.items.get(menu.selected) {
                                if item.submenu.is_some() {
                                    // Enter submenu - replace current menu with submenu
                                    if let Some(submenu) = &item.submenu {
                                        mode = Mode::ThemeSelector { 
                                            menu: (**submenu).clone(), 
                                            theme_engine: theme_eng.clone() 
                                        };
                                    }
                                } else {
                                    // Execute action
                                    match &item.action {
                                        menu_system::MenuAction::ApplyTheme(theme_name) => {
                                            // Apply the theme to the engine
                                            if theme_eng.set_theme(&theme_name).is_ok() {
                                                current_theme_name = theme_name.clone();
                                            }
                                            theme_engine = theme_eng.clone();
                                            mode = Mode::List;
                                        }
                                        menu_system::MenuAction::EditTheme(theme_name) => {
                                            // Load the theme to edit
                                            let mut editor_state = if let Ok(()) = theme_eng.set_theme(&theme_name) {
                                                let theme = theme_eng.get_current_theme().clone();
                                                ThemeEditorState::from_theme(theme)
                                            } else {
                                                ThemeEditorState::new()
                                            };
                                            editor_state.update_fields();
                                            mode = Mode::ThemeEditor {
                                                theme_name: theme_name.clone(),
                                                editor_state,
                                            };
                                        }
                                        menu_system::MenuAction::CreateTheme => {
                                            let mut editor_state = ThemeEditorState::new();
                                            editor_state.theme.name = "New Theme".to_string();
                                            editor_state.theme.author = "Unknown".to_string();
                                            editor_state.update_fields();
                                            mode = Mode::ThemeEditor {
                                                theme_name: "New Theme".to_string(),
                                                editor_state,
                                            };
                                        }
                                        menu_system::MenuAction::ImportTheme => {
                                            let dialog = FileDialog {
                                                title: "Import Theme".to_string(),
                                                file_name: "theme.dctheme".to_string(),
                                                action: FileDialogAction::ImportTheme,
                                                previous_mode: Box::new(Mode::List),
                                            };
                                            mode = Mode::FileDialog { dialog };
                                        }
                                        menu_system::MenuAction::ExportTheme(theme_name) => {
                                            let dialog = FileDialog {
                                                title: "Export Theme".to_string(),
                                                file_name: format!("{}.dctheme", theme_name),
                                                action: FileDialogAction::ExportTheme(theme_name.clone()),
                                                previous_mode: Box::new(Mode::List),
                                            };
                                            mode = Mode::FileDialog { dialog };
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        KeyCode::Left => {
                            // Go back to main theme menu if we're in a submenu
                            if menu.title != "Themes" {
                                let themes = theme_eng.list_themes();
                                let mut main_menu = menu_system::create_theme_menu(themes, &current_theme_name);
                                main_menu.selected = menu.selected;
                                mode = Mode::ThemeSelector {
                                    menu: main_menu,
                                    theme_engine: theme_eng.clone(),
                                };
                            }
                        }
                        KeyCode::Esc => {
                            mode = Mode::List;
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Save current theme
                            if let Some(item) = menu.items.get(menu.selected) {
                                if let menu_system::MenuAction::ApplyTheme(theme_name) = &item.action {
                                    let _ = theme_eng.export_theme(&theme_name, &PathBuf::from(format!("{}.dctheme", theme_name)));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Mode::ScriptManager { menu } => {
                    match key.code {
                        KeyCode::Up => menu.move_up(),
                        KeyCode::Down => menu.move_down(),
                        KeyCode::Right | KeyCode::Enter => {
                            // Check if current item has a submenu
                            if let Some(item) = menu.items.get(menu.selected) {
                                if item.submenu.is_some() {
                                    // Enter submenu
                                    if let Some(submenu) = &item.submenu {
                                        mode = Mode::ScriptManager { 
                                            menu: (**submenu).clone()
                                        };
                                    }
                                } else {
                                    // Execute action
                                    match &item.action {
                                        menu_system::MenuAction::CreateScript(script_type) => {
                                            let ext = match script_type {
                                                menu_system::ScriptType::Python => "py",
                                                menu_system::ScriptType::Lua => "lua",
                                                menu_system::ScriptType::Template(_) => "dcscript",
                                            };
                                            let dialog = FileDialog {
                                                title: "Create New Script".to_string(),
                                                file_name: format!("new_script.{}", ext),
                                                action: FileDialogAction::ExportScript("new_script".to_string()),
                                                previous_mode: Box::new(Mode::List),
                                            };
                                            mode = Mode::FileDialog { dialog };
                                        }
                                        menu_system::MenuAction::ImportScript => {
                                            let dialog = FileDialog {
                                                title: "Import Script".to_string(),
                                                file_name: "script.py".to_string(),
                                                action: FileDialogAction::ImportScript,
                                                previous_mode: Box::new(Mode::List),
                                            };
                                            mode = Mode::FileDialog { dialog };
                                        }
                                        menu_system::MenuAction::RunScript(script_name) => {
                                            // Run the script (placeholder - would need actual script execution)
                                            let results = format!("Running script: {}\n\nScript execution not yet implemented.\nThis would execute the script and show results here.", script_name);
                                            mode = Mode::CompilationResults {
                                                results,
                                                previous_mode: Box::new(Mode::List),
                                                test_result: None,
                                            };
                                        }
                                        menu_system::MenuAction::EditScript(script_name) => {
                                            // Open script in editor (placeholder - would need to load actual script)
                                            let script_path = PathBuf::from(format!("scripts/{}", script_name));
                                            if script_path.exists() {
                                                if let Ok(content) = fs::read_to_string(&script_path) {
                                                    let mut textarea = TextArea::new(content.lines().map(|s| s.to_string()).collect());
                                                    textarea.set_cursor_line_style(Style::default());
                                                    let language = if script_name.ends_with(".py") {
                                                        "Python"
                                                    } else if script_name.ends_with(".lua") {
                                                        "Lua"
                                                    } else {
                                                        "Script"
                                                    };
                                                    mode = Mode::Edit {
                                                        textarea,
                                                        file_path: script_path,
                                                        language: language.to_string(),
                                                    };
                                                }
                                            }
                                        }
                                        menu_system::MenuAction::DeleteScript(script_name) => {
                                            let dialog = ConfirmDialog {
                                                message: format!("Are you sure you want to delete '{}'?\nThis action cannot be undone.", script_name),
                                                on_confirm: ConfirmAction::DeleteScript(script_name.clone()),
                                                previous_mode: Box::new(Mode::List),
                                            };
                                            mode = Mode::ConfirmDialog { dialog };
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        KeyCode::Char('/') => {
                            menu.show_search = !menu.show_search;
                        }
                        KeyCode::Left => {
                            // Go back to main script menu if we're in a submenu
                            if menu.title != "📜 Script Manager" {
                                let scripts = vec![
                                    ("crypto_detector.py".to_string(), "Python".to_string()),
                                    ("string_extractor.lua".to_string(), "Lua".to_string()),
                                ];
                                let new_menu = menu_system::create_script_menu(scripts);
                                mode = Mode::ScriptManager { menu: new_menu };
                            }
                        }
                        KeyCode::Esc => {
                            if menu.show_search {
                                menu.show_search = false;
                                menu.search_query.clear();
                            } else if menu.title != "📜 Script Manager" {
                                // Go back to main script menu if we're in a submenu
                                let scripts = vec![
                                    ("crypto_detector.py".to_string(), "Python".to_string()),
                                    ("string_extractor.lua".to_string(), "Lua".to_string()),
                                ];
                                let new_menu = menu_system::create_script_menu(scripts);
                                mode = Mode::ScriptManager { menu: new_menu };
                            } else {
                                mode = Mode::List;
                            }
                        }
                        _ => {}
                    }
                }
                Mode::Settings { menu } => {
                    match key.code {
                        KeyCode::Up => menu.move_up(),
                        KeyCode::Down => menu.move_down(),
                        KeyCode::Enter => {
                            if let Some(action) = menu.get_selected_action() {
                                match action {
                                    menu_system::MenuAction::OpenSettings(category) => {
                                        let editor_state = SettingsEditorState::new(category);
                                        mode = Mode::SettingsEditor { editor_state };
                                    }
                                    menu_system::MenuAction::ChangeKeybinds => {
                                        let keybinds_menu = menu_system::create_keybinds_menu(&keybind_manager);
                                        mode = Mode::KeybindsEditor { menu: keybinds_menu };
                                    }
                                    menu_system::MenuAction::ImportSettings => {
                                        let dialog = FileDialog {
                                            title: "Import Settings".to_string(),
                                            file_name: "settings.json".to_string(),
                                            action: FileDialogAction::ImportSettings,
                                            previous_mode: Box::new(Mode::List),
                                        };
                                        mode = Mode::FileDialog { dialog };
                                    }
                                    menu_system::MenuAction::ExportSettings => {
                                        let dialog = FileDialog {
                                            title: "Export Settings".to_string(),
                                            file_name: "settings.json".to_string(),
                                            action: FileDialogAction::ExportSettings,
                                            previous_mode: Box::new(Mode::List),
                                        };
                                        mode = Mode::FileDialog { dialog };
                                    }
                                    menu_system::MenuAction::ResetSettings => {
                                        let dialog = ConfirmDialog {
                                            message: "Are you sure you want to reset all settings to default?\nThis action cannot be undone.".to_string(),
                                            on_confirm: ConfirmAction::ResetSettings,
                                            previous_mode: Box::new(Mode::List),
                                        };
                                        mode = Mode::ConfirmDialog { dialog };
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Esc => {
                            mode = Mode::List;
                        }
                        _ => {}
                    }
                }
                Mode::KeybindsEditor { menu } => {
                    match key.code {
                        KeyCode::Up => menu.move_up(),
                        KeyCode::Down => menu.move_down(),
                        KeyCode::Enter => {
                            if let Some(action) = menu.get_selected_action() {
                                match action {
                                    menu_system::MenuAction::ImportSettings => {
                                        let dialog = FileDialog {
                                            title: "Import Keybinds".to_string(),
                                            file_name: "keybinds.json".to_string(),
                                            action: FileDialogAction::ImportKeybinds,
                                            previous_mode: Box::new(Mode::List),
                                        };
                                        mode = Mode::FileDialog { dialog };
                                    }
                                    menu_system::MenuAction::ExportSettings => {
                                        let dialog = FileDialog {
                                            title: "Export Keybinds".to_string(),
                                            file_name: "keybinds.json".to_string(),
                                            action: FileDialogAction::ExportKeybinds,
                                            previous_mode: Box::new(Mode::List),
                                        };
                                        mode = Mode::FileDialog { dialog };
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Esc => {
                            mode = Mode::List;
                        }
                        _ => {}
                    }
                }
                Mode::CompilationResults { results, previous_mode, test_result } => {
                    match key.code {
                        KeyCode::F(5) => {
                            // Run the program if we have an executable
                            if let Some(ref tr) = test_result {
                                if let Some(ref exe_path) = tr.compilation.executable_path {
                                    // Execute the program and capture output
                                    let execution = compiler_tester::execute_program(exe_path);
                                    
                                    // Append the execution results to current results
                                    let mut new_results = results.clone();
                                    new_results.push_str("\n\n");
                                    new_results.push_str("╔════════════════════════════════════════════════════════════════╗\n");
                                    new_results.push_str("║                    PROGRAM OUTPUT                              ║\n");
                                    new_results.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");
                                    
                                    new_results.push_str(&format!("┌─ Execution ───────────────────────────────────────────────────┐\n"));
                                    new_results.push_str(&format!("│ Status:          {}\n", if execution.0 { "✅ SUCCESS" } else { "❌ FAILED" }));
                                    new_results.push_str(&format!("│ Exit Code:       {}\n", execution.3.map_or("N/A".to_string(), |c| c.to_string())));
                                    new_results.push_str(&format!("│ Time:            {} ms\n", execution.4));
                                    new_results.push_str(&format!("└───────────────────────────────────────────────────────────────┘\n\n"));
                                    
                                    if !execution.1.is_empty() {
                                        new_results.push_str("┌─ Standard Output ─────────────────────────────────────────────┐\n");
                                        for line in execution.1.lines() {
                                            new_results.push_str(&format!("│ {}\n", line));
                                        }
                                        new_results.push_str("└───────────────────────────────────────────────────────────────┘\n\n");
                                    }
                                    
                                    if !execution.2.is_empty() {
                                        new_results.push_str("┌─ Standard Error ──────────────────────────────────────────────┐\n");
                                        for line in execution.2.lines() {
                                            new_results.push_str(&format!("│ {}\n", line));
                                        }
                                        new_results.push_str("└───────────────────────────────────────────────────────────────┘\n\n");
                                    }
                                    
                                    new_results.push_str("\n(Press F5 to run again, or any other key to return)");
                                    
                                    // Update the results in place
                                    *results = new_results;
                                }
                            }
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Copy results to clipboard
                            match arboard::Clipboard::new() {
                                Ok(mut clipboard) => {
                                    if let Err(e) = clipboard.set_text(results.clone()) {
                                        // If clipboard fails, just ignore silently
                                        eprintln!("Failed to copy to clipboard: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to access clipboard: {}", e);
                                }
                            }
                        }
                        _ => {
                            // Any other key returns to the previous mode (editor)
                            // We need to replace mode with a temporary value, then swap
                            let temp_mode = std::mem::replace(&mut mode, Mode::List);
                            if let Mode::CompilationResults { previous_mode, .. } = temp_mode {
                                mode = *previous_mode;
                            }
                        }
                    }
                }
                Mode::ThemeEditor { theme_name, editor_state } => {
                    if editor_state.editing_value.is_some() {
                        // Handle input mode
                        match key.code {
                            KeyCode::Char(c) => {
                                if let Some(ref mut value) = editor_state.editing_value {
                                    value.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                if let Some(ref mut value) = editor_state.editing_value {
                                    value.pop();
                                }
                            }
                            KeyCode::Enter => {
                                // Save the edited value
                                if let Some(value) = editor_state.editing_value.take() {
                                    editor_state.save_field_value(value);
                                }
                            }
                            KeyCode::Esc => {
                                // Cancel editing
                                editor_state.editing_value = None;
                            }
                            _ => {}
                        }
                    } else {
                        // Handle navigation mode
                        match key.code {
                            KeyCode::Up => {
                                if editor_state.selected_field > 0 {
                                    editor_state.selected_field -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if editor_state.selected_field < editor_state.fields.len().saturating_sub(1) {
                                    editor_state.selected_field += 1;
                                }
                            }
                            KeyCode::Left => {
                                if editor_state.selected_section > 0 {
                                    editor_state.selected_section -= 1;
                                    editor_state.selected_field = 0;
                                    editor_state.update_fields();
                                }
                            }
                            KeyCode::Right => {
                                if editor_state.selected_section < editor_state.sections.len() - 1 {
                                    editor_state.selected_section += 1;
                                    editor_state.selected_field = 0;
                                    editor_state.update_fields();
                                }
                            }
                            KeyCode::Tab => {
                                // Switch between sections
                                editor_state.selected_section = (editor_state.selected_section + 1) % editor_state.sections.len();
                                editor_state.selected_field = 0;
                                editor_state.update_fields();
                            }
                            KeyCode::Enter => {
                                // Start editing selected field
                                if editor_state.selected_field < editor_state.fields.len() {
                                    let current_value = editor_state.fields[editor_state.selected_field].1.clone();
                                    editor_state.editing_value = Some(current_value);
                                }
                            }
                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Save theme
                                let theme = editor_state.theme.clone();
                                if let Err(e) = theme_engine.load_custom_theme(theme.clone()) {
                                    // Show error (for now just ignore)
                                    let _ = e;
                                } else {
                                    // Export to file
                                    let theme_path = PathBuf::from(format!("themes/{}.dctheme", theme_name));
                                    let _ = theme_engine.export_theme(&theme_name, &theme_path);
                                    editor_state.modified = false;
                                }
                            }
                            KeyCode::Esc => {
                                if editor_state.modified {
                                    // For now, just go back (TODO: Show save confirmation dialog)
                                }
                                mode = Mode::List;
                            }
                            _ => {}
                        }
                    }
                }
                Mode::SettingsEditor { editor_state } => {
                    if editor_state.editing_value.is_some() {
                        // Handle input mode
                        match key.code {
                            KeyCode::Char(c) => {
                                if let Some(ref mut value) = editor_state.editing_value {
                                    value.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                if let Some(ref mut value) = editor_state.editing_value {
                                    value.pop();
                                }
                            }
                            KeyCode::Enter => {
                                // Save the edited value
                                if let Some(value) = editor_state.editing_value.take() {
                                    editor_state.save_field_value(value);
                                }
                            }
                            KeyCode::Esc => {
                                // Cancel editing
                                editor_state.editing_value = None;
                            }
                            _ => {}
                        }
                    } else {
                        // Handle navigation mode
                        match key.code {
                            KeyCode::Up => {
                                if editor_state.selected_field > 0 {
                                    editor_state.selected_field -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if editor_state.selected_field < editor_state.fields.len().saturating_sub(1) {
                                    editor_state.selected_field += 1;
                                }
                            }
                            KeyCode::Enter => {
                                // Start editing selected field
                                if editor_state.selected_field < editor_state.fields.len() {
                                    let current_value = editor_state.fields[editor_state.selected_field].1.clone();
                                    editor_state.editing_value = Some(current_value);
                                }
                            }
                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Save settings (placeholder - would need actual settings persistence)
                                editor_state.modified = false;
                            }
                            KeyCode::Esc => {
                                mode = Mode::List;
                            }
                            _ => {}
                        }
                    }
                }
                Mode::ConfirmDialog { dialog } => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // Execute the confirmation action
                            match &dialog.on_confirm {
                                ConfirmAction::ResetSettings => {
                                    // Reset settings (placeholder)
                                }
                                ConfirmAction::DeleteTheme(theme_name) => {
                                    // Delete theme (placeholder)
                                    let theme_path = PathBuf::from(format!("themes/{}.dctheme", theme_name));
                                    let _ = fs::remove_file(theme_path);
                                }
                                ConfirmAction::DeleteScript(script_name) => {
                                    // Delete script (placeholder)
                                    let script_path = PathBuf::from(format!("scripts/{}", script_name));
                                    let _ = fs::remove_file(script_path);
                                }
                            }
                            mode = Mode::List;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            // Cancel - return to previous mode
                            mode = Mode::List;
                        }
                        _ => {}
                    }
                }
                Mode::FileDialog { dialog } => {
                    match key.code {
                        KeyCode::Char(c) => {
                            dialog.file_name.push(c);
                        }
                        KeyCode::Backspace => {
                            dialog.file_name.pop();
                        }
                        KeyCode::Enter => {
                            // Execute the file action
                            match &dialog.action {
                                FileDialogAction::ImportSettings => {
                                    // Import settings from file (placeholder)
                                    let path = PathBuf::from(&dialog.file_name);
                                    if path.exists() {
                                        // Would load settings here
                                    }
                                }
                                FileDialogAction::ExportSettings => {
                                    // Export settings to file (placeholder)
                                    let path = PathBuf::from(&dialog.file_name);
                                    // Would save settings here
                                    let _ = fs::write(path, "{}");
                                }
                                FileDialogAction::ImportTheme => {
                                    // Import theme from file
                                    let path = PathBuf::from(&dialog.file_name);
                                    if path.exists() {
                                        let _ = theme_engine.import_theme(&path);
                                    }
                                }
                                FileDialogAction::ExportTheme(theme_name) => {
                                    // Export theme to file
                                    let path = PathBuf::from(&dialog.file_name);
                                    let _ = theme_engine.export_theme(theme_name, &path);
                                }
                                FileDialogAction::ImportScript => {
                                    // Import script from file (placeholder)
                                    let path = PathBuf::from(&dialog.file_name);
                                    if path.exists() {
                                        let dest = PathBuf::from(format!("scripts/{}", path.file_name().unwrap().to_string_lossy()));
                                        let _ = fs::copy(path, dest);
                                    }
                                }
                                FileDialogAction::ExportScript(script_name) => {
                                    // Create new script file (placeholder)
                                    let path = PathBuf::from(format!("scripts/{}", dialog.file_name));
                                    let template = format!("# New script: {}\n\n# Add your code here\n", script_name);
                                    let _ = fs::write(path, template);
                                }
                                FileDialogAction::ImportKeybinds => {
                                    // Import keybinds from file
                                    let path = PathBuf::from(&dialog.file_name);
                                    let _ = keybind_manager.import_config(&path);
                                }
                                FileDialogAction::ExportKeybinds => {
                                    // Export keybinds to file
                                    let path = PathBuf::from(&dialog.file_name);
                                    let _ = keybind_manager.export_config(&path);
                                }
                            }
                            mode = Mode::List;
                        }
                        KeyCode::Esc => {
                            // Cancel - return to previous mode
                            mode = Mode::List;
                        }
                        _ => {}
                    }
                }
            }
        }
        }
    }
}