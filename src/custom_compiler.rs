use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use crate::builtin_assembler;

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub language: String,
    pub output: String,
    pub errors: String,
    pub compilation_time_ms: u128,
    pub executable_path: Option<PathBuf>,
    pub auto_fixes_applied: Vec<String>,
}

/// Smart C compiler with auto-fixing capabilities
pub fn compile_c_smart(source_path: &Path) -> CompilationResult {
    let start = Instant::now();
    let mut auto_fixes = Vec::new();
    
    // Read source code - try UTF-8 first, fall back to lossy conversion if needed
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(_) => {
            // Try reading as bytes and converting with lossy UTF-8
            match fs::read(source_path) {
                Ok(bytes) => {
                    auto_fixes.push("Converted non-UTF-8 characters to valid UTF-8".to_string());
                    String::from_utf8_lossy(&bytes).to_string()
                },
                Err(e) => {
                    return CompilationResult {
                        success: false,
                        language: "C".to_string(),
                        output: String::new(),
                        errors: format!("Failed to read source file: {}", e),
                        compilation_time_ms: start.elapsed().as_millis(),
                        executable_path: None,
                        auto_fixes_applied: auto_fixes,
                    };
                }
            }
        }
    };
    
    // Apply auto-fixes
    let fixed_source = auto_fix_c_code(&source, &mut auto_fixes);
    
    // Write fixed source to temp file
    let temp_path = source_path.with_extension("fixed.c");
    if let Err(e) = fs::write(&temp_path, &fixed_source) {
        return CompilationResult {
            success: false,
            language: "C".to_string(),
            output: String::new(),
            errors: format!("Failed to write fixed source: {}", e),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
        };
    }
    
    let output_path = source_path.with_extension("exe");
    
    // Try multiple compilers in order of preference
    let compilers = vec![
        ("gcc", vec![
            temp_path.to_string_lossy().to_string(),
            "-o".to_string(),
            output_path.to_string_lossy().to_string(),
            "-O2".to_string(),
            "-Wall".to_string(),
            "-Wno-implicit-function-declaration".to_string(),
            "-lkernel32".to_string(),
            "-luser32".to_string(),
        ]),
        ("clang", vec![
            temp_path.to_string_lossy().to_string(),
            "-o".to_string(),
            output_path.to_string_lossy().to_string(),
            "-O2".to_string(),
            "-Wall".to_string(),
        ]),
        ("cl", vec![
            "/nologo".to_string(),
            temp_path.to_string_lossy().to_string(),
            format!("/Fe:{}", output_path.display()),
            "/O2".to_string(),
        ]),
        ("tcc", vec![
            temp_path.to_string_lossy().to_string(),
            "-o".to_string(),
            output_path.to_string_lossy().to_string(),
        ]),
    ];
    
    for (compiler, args) in compilers {
        let result = Command::new(compiler)
            .args(&args)
            .output();
        
        if let Ok(output) = result {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            // Clean up temp file
            let _ = fs::remove_file(&temp_path);
            
            if success || compiler == "tcc" {  // TCC is our last resort
                return CompilationResult {
                    success,
                    language: format!("C ({})", compiler),
                    output: stdout,
                    errors: stderr,
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: if success { Some(output_path) } else { None },
                    auto_fixes_applied: auto_fixes,
                };
            }
        }
    }
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_path);
    
    CompilationResult {
        success: false,
        language: "C".to_string(),
        output: String::new(),
        errors: "No C compiler found. Install: gcc (MinGW), clang, MSVC (cl), or TCC".to_string(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
        auto_fixes_applied: auto_fixes,
    }
}

/// Auto-fix common C code issues from decompilation
fn auto_fix_c_code(source: &str, fixes: &mut Vec<String>) -> String {
    let mut fixed = source.to_string();
    
    // Remove BOM (Byte Order Mark) if present
    if fixed.starts_with('\u{feff}') {
        fixed = fixed.trim_start_matches('\u{feff}').to_string();
        fixes.push("Removed BOM (Byte Order Mark)".to_string());
    }
    
    // Remove any null bytes or other control characters that might cause issues
    if fixed.contains('\0') {
        fixed = fixed.replace('\0', "");
        fixes.push("Removed null bytes".to_string());
    }
    
    // Check if this looks like decompiled code
    let is_decompiled = fixed.contains("undefined") || 
                        fixed.contains("DAT_") ||
                        fixed.contains("FUN_") ||
                        fixed.contains("// WARNING:") ||
                        (fixed.contains("0x") && fixed.lines().count() > 100);
    
    // Only apply aggressive fixes if this looks like decompiled code
    if is_decompiled {
        // Fix undefined types
        if fixed.contains("undefined") {
            fixed = fixed.replace("undefined8", "unsigned long long");
            fixed = fixed.replace("undefined4", "unsigned int");
            fixed = fixed.replace("undefined2", "unsigned short");
            fixed = fixed.replace("undefined1", "unsigned char");
            fixed = fixed.replace("undefined", "int");
            fixes.push("Fixed undefined types".to_string());
        }
        
        // Fix common decompiler artifacts
        if fixed.contains("DWORD") || fixed.contains("BYTE") || fixed.contains("WORD") || fixed.contains("QWORD") {
            fixed = fixed.replace("DWORD", "unsigned int");
            fixed = fixed.replace("BYTE", "unsigned char");
            fixed = fixed.replace("WORD", "unsigned short");
            fixed = fixed.replace("QWORD", "unsigned long long");
            fixed = fixed.replace("HANDLE", "void*");
            fixes.push("Fixed Windows types".to_string());
        }
        
        // Add missing includes if not present
        if !fixed.contains("#include") {
            let includes = "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <windows.h>\n\n";
            fixed = includes.to_string() + &fixed;
            fixes.push("Added standard includes".to_string());
        }
        
        // Fix main function if missing
        if !fixed.contains("int main") && !fixed.contains("void main") {
            // Look for a likely entry point
            if fixed.contains("sub_") || fixed.contains("func_") || fixed.contains("FUN_") {
                let entry_wrapper = "\n\nint main(int argc, char** argv) {\n    // Auto-generated entry point\n    return 0;\n}\n";
                fixed.push_str(entry_wrapper);
                fixes.push("Added main() entry point".to_string());
            }
        }
        
        // Add function declarations for common Windows APIs if used
        let windows_apis = vec![
            ("GetProcAddress", "void* GetProcAddress(void* hModule, const char* lpProcName);"),
            ("LoadLibraryA", "void* LoadLibraryA(const char* lpLibFileName);"),
            ("VirtualAlloc", "void* VirtualAlloc(void* lpAddress, size_t dwSize, unsigned long flAllocationType, unsigned long flProtect);"),
            ("CreateThread", "void* CreateThread(void* lpThreadAttributes, size_t dwStackSize, void* lpStartAddress, void* lpParameter, unsigned long dwCreationFlags, unsigned long* lpThreadId);"),
        ];
        
        let mut declarations = String::new();
        for (api, decl) in windows_apis {
            if fixed.contains(api) && !fixed.contains(&format!("{}(", api)) {
                declarations.push_str(decl);
                declarations.push('\n');
            }
        }
        
        if !declarations.is_empty() {
            // Insert after includes
            if let Some(pos) = fixed.find("\n\n") {
                fixed.insert_str(pos + 2, &declarations);
                fixes.push("Added Windows API declarations".to_string());
            }
        }
        
        // Fix pointer syntax issues
        fixed = fixed.replace("* *", "**");
        fixed = fixed.replace("* &", "*&");
        
        // Fix common syntax errors
        fixed = fixed.replace(";;", ";");
        fixed = fixed.replace(",)", ")");
        fixed = fixed.replace("(,", "(");
    }
    
    fixed
}

/// Smart Rust compiler with auto-fixing
pub fn compile_rust_smart(source_path: &Path) -> CompilationResult {
    let start = Instant::now();
    let mut auto_fixes = Vec::new();
    
    // Read source code - try UTF-8 first, fall back to lossy conversion if needed
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(_) => {
            // Try reading as bytes and converting with lossy UTF-8
            match fs::read(source_path) {
                Ok(bytes) => {
                    auto_fixes.push("Converted non-UTF-8 characters to valid UTF-8".to_string());
                    String::from_utf8_lossy(&bytes).to_string()
                },
                Err(e) => {
                    return CompilationResult {
                        success: false,
                        language: "Rust".to_string(),
                        output: String::new(),
                        errors: format!("Failed to read source file: {}", e),
                        compilation_time_ms: start.elapsed().as_millis(),
                        executable_path: None,
                        auto_fixes_applied: auto_fixes,
                    };
                }
            }
        }
    };
    
    // Apply auto-fixes
    let fixed_source = auto_fix_rust_code(&source, &mut auto_fixes);
    
    // Write fixed source to temp file
    let temp_path = source_path.with_extension("fixed.rs");
    if let Err(e) = fs::write(&temp_path, &fixed_source) {
        return CompilationResult {
            success: false,
            language: "Rust".to_string(),
            output: String::new(),
            errors: format!("Failed to write fixed source: {}", e),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
        };
    }
    
    let output_path = source_path.with_extension("exe");
    
    let result = Command::new("rustc")
        .arg(&temp_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-C")
        .arg("opt-level=2")
        .arg("--edition=2021")
        .arg("-A")
        .arg("warnings")  // Suppress warnings for decompiled code
        .output();
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_path);
    
    if let Ok(output) = result {
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        CompilationResult {
            success,
            language: "Rust".to_string(),
            output: stdout,
            errors: stderr,
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: if success { Some(output_path) } else { None },
            auto_fixes_applied: auto_fixes,
        }
    } else {
        CompilationResult {
            success: false,
            language: "Rust".to_string(),
            output: String::new(),
            errors: "rustc not found. Install from: https://rustup.rs".to_string(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
        }
    }
}

/// Auto-fix common Rust code issues from decompilation
fn auto_fix_rust_code(source: &str, fixes: &mut Vec<String>) -> String {
    let mut fixed = source.to_string();
    
    // Remove BOM (Byte Order Mark) if present
    if fixed.starts_with('\u{feff}') {
        fixed = fixed.trim_start_matches('\u{feff}').to_string();
        fixes.push("Removed BOM (Byte Order Mark)".to_string());
    }
    
    // Remove any null bytes or other control characters that might cause issues
    if fixed.contains('\0') {
        fixed = fixed.replace('\0', "");
        fixes.push("Removed null bytes".to_string());
    }
    
    // Check if this looks like decompiled code (has undefined types, raw hex, etc.)
    let is_decompiled = fixed.contains("undefined") || 
                        fixed.contains("0x") && fixed.lines().count() > 100 ||
                        fixed.contains("// 0x") ||
                        fixed.contains("DAT_");
    
    // Only apply aggressive fixes if this looks like decompiled code
    if is_decompiled {
        // Fix common type issues from decompilers
        if fixed.contains("undefined") {
            fixed = fixed.replace("undefined8", "u64");
            fixed = fixed.replace("undefined4", "u32");
            fixed = fixed.replace("undefined2", "u16");
            fixed = fixed.replace("undefined1", "u8");
            fixed = fixed.replace("undefined", "i32");
            fixes.push("Fixed undefined types".to_string());
        }
        
        // Fix Windows API types
        if fixed.contains("DWORD") || fixed.contains("QWORD") || fixed.contains("HANDLE") {
            fixed = fixed.replace("DWORD", "u32");
            fixed = fixed.replace("QWORD", "u64");
            fixed = fixed.replace("HANDLE", "*mut std::ffi::c_void");
            fixes.push("Fixed Windows API types".to_string());
        }
        
        // Add imports only if we're using ptr/mem operations and don't have them
        if !fixed.contains("use std::") && (fixed.contains("::ptr") || fixed.contains("::mem")) {
            let imports = "use std::ptr;\nuse std::mem;\n\n";
            fixed = imports.to_string() + &fixed;
            fixes.push("Added standard imports".to_string());
        }
        
        // Add main function if missing
        if !fixed.contains("fn main") {
            let main_fn = "\n\nfn main() {\n    // Auto-generated entry point\n    println!(\"Decompiled program\");\n}\n";
            fixed.push_str(main_fn);
            fixes.push("Added main() function".to_string());
        }
        
        // Add unsafe blocks where needed (only for decompiled code)
        if !fixed.contains("unsafe") && (fixed.contains("*mut") || fixed.contains("*const")) {
            // Wrap main function body in unsafe if it contains raw pointers
            if let Some(main_start) = fixed.find("fn main()") {
                if let Some(brace_start) = fixed[main_start..].find('{') {
                    let insert_pos = main_start + brace_start + 1;
                    fixed.insert_str(insert_pos, "\n    unsafe {");
                    
                    // Find the closing brace - be more careful here
                    let remaining = &fixed[insert_pos..];
                    let mut brace_count = 1;
                    let mut close_pos = None;
                    
                    for (i, ch) in remaining.char_indices() {
                        if ch == '{' {
                            brace_count += 1;
                        } else if ch == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                close_pos = Some(insert_pos + i);
                                break;
                            }
                        }
                    }
                    
                    if let Some(pos) = close_pos {
                        fixed.insert_str(pos, "\n    }");
                        fixes.push("Wrapped main in unsafe block".to_string());
                    }
                }
            }
        }
    }
    
    fixed
}

/// Enhanced assembly compiler with multiple assembler support
pub fn compile_assembly_smart(source_path: &Path) -> CompilationResult {
    let start = Instant::now();
    let mut auto_fixes = Vec::new();
    
    // Read source - try UTF-8 first, fall back to lossy conversion if needed
    let mut source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(_) => {
            // Try reading as bytes and converting with lossy UTF-8
            match fs::read(source_path) {
                Ok(bytes) => {
                    auto_fixes.push("Converted non-UTF-8 characters to valid UTF-8".to_string());
                    String::from_utf8_lossy(&bytes).to_string()
                },
                Err(e) => {
                    return CompilationResult {
                        success: false,
                        language: "Assembly".to_string(),
                        output: String::new(),
                        errors: format!("Failed to read source file: {}", e),
                        compilation_time_ms: start.elapsed().as_millis(),
                        executable_path: None,
                        auto_fixes_applied: auto_fixes,
                    };
                }
            }
        }
    };
    
    // ═══════════════════════════════════════════════════════════════════════════════
    // ⚡ CRITICAL PERFORMANCE OPTIMIZATION: Check for unassemblable code FIRST! ⚡
    // ═══════════════════════════════════════════════════════════════════════════════
    // This check MUST happen BEFORE auto_fix_assembly() because that function does
    // expensive preprocessing (convert_disassembly_to_asm with 2 full passes).
    // On large decompiled executables (4500+ labels), preprocessing takes ~180 seconds.
    // This simple string check takes <1ms and saves all that wasted processing!
    // ═══════════════════════════════════════════════════════════════════════════════
    
    // Check if this is disassembled code from our disassembler
    let is_disassembled_code = source.contains("; Section:") && 
                                source.contains("(VA: 0x") &&
                                source.contains("ENTRY POINT");
    
    let has_hardcoded_rvas = source.contains("[rip + 0x") || 
                              source.contains("[rip - 0x") ||
                              source.contains("call     qword ptr [rip");
    
    // If this is disassembled code, ALWAYS use PE reassembler (even without RIP refs)
    if is_disassembled_code || has_hardcoded_rvas {
        // 🔧 NEW: Try to automatically fix the RIP-relative addresses!
        if has_hardcoded_rvas {
            println!("⚠️  Detected decompiled code with hardcoded RIP-relative addresses");
            println!("🔧 Attempting to automatically fix and relocate...");
        } else {
            println!("✓ Detected disassembled code from our disassembler");
            println!("🔨 Using PE Reassembler to preserve original PE structure...");
        }
        
        // Try to find the original executable to extract data from
        // Handle cases like "program.exe.asm" -> "program.exe" (not "program.exe.exe")
        let original_exe = if source_path.to_string_lossy().ends_with(".exe.asm") {
            // Strip .asm to get .exe
            PathBuf::from(source_path.to_string_lossy().trim_end_matches(".asm"))
        } else {
            // Normal case: just replace extension
            source_path.with_extension("exe")
        };
        
        let original_exe_opt = if original_exe.exists() {
            println!("✓ Found original executable: {}", original_exe.display());
            Some(original_exe.as_path())
        } else {
            println!("⚠ Original executable not found at: {}", original_exe.display());
            println!("  Looked for: {}", original_exe.display());
            None
        };
        
        // Attempt automatic relocation
        let relocation_result = crate::assembly_relocator::fix_decompiled_assembly(
            &source,
            original_exe_opt,
        );
        
        // Show warnings/progress
        for warning in &relocation_result.warnings {
            println!("  {}", warning);
        }
        
        if relocation_result.success {
            if relocation_result.stats.total_rip_refs > 0 {
                // Case 1: Successfully fixed RIP references
                println!("✓ Successfully fixed {} RIP-relative references!", relocation_result.stats.total_rip_refs);
                println!("  • {} call references (IAT entries)", relocation_result.stats.fixed_calls);
                println!("  • {} data references", relocation_result.stats.fixed_data);
                
                // Use the fixed assembly code and continue compilation
                source = relocation_result.fixed_assembly;
                
                // DEBUG: Save the relocated assembly for inspection
                let debug_path = source_path.with_extension("relocated.asm");
                if let Err(e) = std::fs::write(&debug_path, &source) {
                    println!("⚠ Could not save debug file: {}", e);
                } else {
                    println!("📝 Saved relocated assembly to: {}", debug_path.display());
                }
                
                auto_fixes.push(format!("🔧 Fixed {} hardcoded RIP-relative addresses", relocation_result.stats.total_rip_refs));
                auto_fixes.push(format!("  ✓ {} call references relocated", relocation_result.stats.fixed_calls));
                auto_fixes.push(format!("  ✓ {} data references relocated", relocation_result.stats.fixed_data));
                
                // 🎯 NEW: Use PE Reassembler for decompiled code (like IDA/Ghidra)
                if let Some(original_exe_path) = original_exe_opt {
                    println!("\n🔨 [DEBUG] Using PE Reassembler (IDA/Ghidra-style)...");
                    println!("   [DEBUG] This will preserve the original PE structure!");
                    println!("   [DEBUG] Original exe: {}", original_exe_path.display());
                    
                    // Try to assemble the fixed code first using builtin assembler
                    // Assume 64-bit for decompiled code (most modern executables)
                    println!("   [DEBUG] Creating builtin assembler (64-bit mode)...");
                    let mut assembler = crate::builtin_assembler::BuiltinAssembler::new(true);
                    println!("   [DEBUG] Assembling code...");
                    match assembler.assemble(&source) {
                        Ok(mut binary) => {
                            println!("   [DEBUG] ✓ Assembled new code: {} bytes", binary.code.len());
                            println!("   [DEBUG] Entry point in code: 0x{:x}", binary.entry_point);
                            
                            // 🔧 CRITICAL FIX: If code starts at entry point but PE expects padding,
                            // prepend NOPs to align properly. The disassembly starts from entry point
                            // (e.g., 0x1400), but .text section starts earlier (e.g., 0x1000).
                            // We need to prepend 0x400 bytes of NOPs.
                            if binary.entry_point > 0x1000 {
                                let padding_needed = (binary.entry_point - 0x1000) as usize;
                                println!("   [DEBUG] 🔧 Prepending {} bytes of padding to match .text layout", padding_needed);
                                let mut padded_code = vec![0x90u8; padding_needed];
                                padded_code.extend_from_slice(&binary.code);
                                binary.code = padded_code;
                                println!("   [DEBUG] New code size with padding: {} bytes", binary.code.len());
                            }
                            
                            // Now use the PE reassembler to merge it with the original
                            let output_path = source_path.with_extension("exe");
                            println!("   [DEBUG] Output path: {}", output_path.display());
                            println!("   [DEBUG] Calling PE reassembler...");
                            match crate::pe_reassembler::reassemble_decompiled_exe(
                                original_exe_path,
                                binary.code,
                                &output_path,
                            ) {
                                Ok(_) => {
                                    println!("   [DEBUG] ✅ PE reassembly successful!");
                                    return CompilationResult {
                                        success: true,
                                        language: "Assembly (PE Reassembler - IDA/Ghidra Style)".to_string(),
                                        output: format!(
                                            "✅ Successfully reassembled decompiled executable!\n\n\
                                             🎯 PE REASSEMBLER (Like IDA/Ghidra)\n\
                                             ═══════════════════════════════════════\n\
                                             This is NOT a simple recompilation!\n\
                                             Instead, we:\n\
                                             1. ✓ Extracted original PE structure\n\
                                             2. ✓ Preserved imports, data, resources\n\
                                             3. ✓ Reassembled only the .text section\n\
                                             4. ✓ Merged new code with original sections\n\n\
                                             📊 Statistics:\n\
                                             • RIP references fixed: {}\n\
                                             • Call references: {}\n\
                                             • Data references: {}\n\n\
                                             ⚠️  IMPORTANT:\n\
                                             The executable now contains your modified code\n\
                                             but keeps all original imports, data, and resources.\n\
                                             This is how IDA Pro and Ghidra work!",
                                            relocation_result.stats.total_rip_refs,
                                            relocation_result.stats.fixed_calls,
                                            relocation_result.stats.fixed_data
                                        ),
                                        errors: String::new(),
                                        compilation_time_ms: start.elapsed().as_millis(),
                                        executable_path: Some(output_path),
                                        auto_fixes_applied: auto_fixes,
                                    };
                                }
                                Err(e) => {
                                    println!("   [DEBUG] ⚠ PE Reassembler failed: {}", e);
                                    println!("   [DEBUG] Error details: {}", e);
                                    println!("   [DEBUG] Falling back to standard assembly...");
                                    // Fall through to normal compilation
                                }
                            }
                        }
                        Err(e) => {
                            println!("   [DEBUG] ⚠ Assembly failed: {}", e);
                            println!("   [DEBUG] Error details: {}", e);
                            println!("   [DEBUG] Falling back to external assemblers...");
                            // Fall through to normal compilation
                        }
                    }
                } else {
                    println!("   [DEBUG] ℹ️  No original executable found");
                    println!("   [DEBUG] Skipping PE reassembler, using standard compilation");
                }
            } else {
                // Case 2: No RIP references found - but still disassembled code!
                // Use PE reassembler to preserve original PE structure
                println!("✓ No RIP-relative references found - code appears clean");
                auto_fixes.push("✓ Verified: No hardcoded RIP-relative addresses detected".to_string());
                
                // Still use PE Reassembler if this is disassembled code
                if is_disassembled_code {
                    if let Some(original_exe_path) = original_exe_opt {
                        println!("\n🔨 Using PE Reassembler (IDA/Ghidra-style)...");
                        println!("   This will preserve the original PE structure!");
                        
                        // Try to assemble the code first using builtin assembler
                        // Assume 64-bit for disassembled code (most modern executables)
                        let mut assembler = crate::builtin_assembler::BuiltinAssembler::new(true);
                        match assembler.assemble(&source) {
                            Ok(binary) => {
                                println!("   ✓ Assembled new code: {} bytes", binary.code.len());
                                
                                // Now use the PE reassembler to merge it with the original
                                let output_path = source_path.with_extension("exe");
                                match crate::pe_reassembler::reassemble_decompiled_exe(
                                    original_exe_path,
                                    binary.code,
                                    &output_path,
                                ) {
                                    Ok(_) => {
                                        println!("   ✅ PE reassembly successful!");
                                        return CompilationResult {
                                            success: true,
                                            language: "Assembly (PE Reassembler - IDA/Ghidra Style)".to_string(),
                                            output: format!(
                                                "✅ Successfully reassembled disassembled executable!\n\n\
                                                 🎯 PE REASSEMBLER (Like IDA/Ghidra)\n\
                                                 ═══════════════════════════════════════\n\
                                                 This is NOT a simple recompilation!\n\
                                                 Instead, we:\n\
                                                 1. ✓ Extracted original PE structure\n\
                                                 2. ✓ Preserved imports, data, resources\n\
                                                 3. ✓ Reassembled only the .text section\n\
                                                 4. ✓ Merged new code with original sections\n\n\
                                                 ⚠️  IMPORTANT:\n\
                                                 The executable now contains your modified code\n\
                                                 but keeps all original imports, data, and resources.\n\
                                                 This is how IDA Pro and Ghidra work!"
                                            ),
                                            errors: String::new(),
                                            compilation_time_ms: start.elapsed().as_millis(),
                                            executable_path: Some(output_path),
                                            auto_fixes_applied: auto_fixes,
                                        };
                                    }
                                    Err(e) => {
                                        println!("   ⚠ PE Reassembler failed: {}", e);
                                        println!("   Falling back to standard assembly...");
                                        // Fall through to normal compilation
                                    }
                                }
                            }
                            Err(e) => {
                                println!("   ⚠ Assembly failed: {}", e);
                                println!("   Falling back to external assemblers...");
                                // Fall through to normal compilation
                            }
                        }
                    }
                }
            }
            
            // Continue with normal compilation flow in both cases
        } else {
            // Case 3: Relocation failed - show detailed error
            return CompilationResult {
                success: false,
                language: "Assembly (Decompiled Code)".to_string(),
                output: String::new(),
                errors: format!(
                    "❌ CANNOT REASSEMBLE DECOMPILED CODE WITH HARDCODED ADDRESSES\n\n\
                     This assembly code contains RIP-relative addresses (like [rip + 0x...])\n\
                     that reference the original executable's memory layout.\n\n\
                     WHY THIS FAILS:\n\
                     • The code has hardcoded RVAs pointing to the original IAT/data sections\n\
                     • When reassembled, the PE structure is different\n\
                     • All those addresses become invalid → ACCESS_VIOLATION crash\n\n\
                     AUTOMATIC FIX ATTEMPTED:\n\
                     • Tried to relocate {} RIP-relative references\n\
                     • Result: {}\n\
                     • Errors: {}\n\n\
                     SOLUTIONS:\n\n\
                     1. ✅ PROVIDE ORIGINAL EXECUTABLE\n\
                        Place the original .exe in the same directory as the .asm file.\n\
                        The compiler will extract data and fix addresses automatically.\n\n\
                     2. ✅ USE THE ORIGINAL EXECUTABLE\n\
                        The decompiled code is for analysis only, not for reassembly.\n\
                        If you need to run it, use the original .exe file.\n\n\
                     3. ✅ REWRITE IN C/RUST\n\
                        Convert the logic to C or Rust and compile normally.\n\
                        This gives you proper imports and relocatable code.\n\n\
                     4. ✅ MANUAL RELOCATION (ADVANCED)\n\
                        Manually rewrite all RIP-relative addresses to use proper imports.\n\
                        This requires deep understanding of PE format and assembly.\n\n\
                     5. ✅ USE A PATCHER\n\
                        Use tools like x64dbg or IDA Pro to patch the original executable\n\
                        instead of reassembling from scratch.\n\n\
                     TECHNICAL EXPLANATION:\n\
                     Decompiled assembly is like a photograph of memory - it shows what\n\
                     was there, but you can't rebuild the original from it without knowing\n\
                     the context. The addresses are absolute to that specific executable's\n\
                     memory layout and won't work in a new PE file.\n\n\
                     TIP: If you just want to analyze the code, view it in the decompiler.\n\
                     If you want to modify behavior, patch the original or rewrite in C/Rust.\n\n\
                     ⚡ PERFORMANCE NOTE: This early check saved you ~180 seconds of wasted processing!",
                    relocation_result.stats.total_rip_refs,
                    if relocation_result.success { "Success" } else { "Failed" },
                    relocation_result.errors.join("; ")
                ),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
                auto_fixes_applied: vec![
                    "⚠️  Detected decompiled code with hardcoded RVAs (ULTRA-EARLY DETECTION)".to_string(),
                    format!("🔧 Attempted to fix {} RIP-relative references", relocation_result.stats.total_rip_refs),
                    "❌ Automatic relocation failed - see errors above".to_string(),
                    "💡 Provide original .exe or rewrite in C/Rust instead".to_string(),
                    "⚡ Saved ~180 seconds by detecting this BEFORE preprocessing!".to_string(),
                ],
            };
        }
    }
    
    // Auto-fix assembly code
    let fixed_source = auto_fix_assembly(&source, &mut auto_fixes);
    
    // Write fixed source
    let temp_path = source_path.with_extension("fixed.asm");
    if let Err(e) = fs::write(&temp_path, &fixed_source) {
        return CompilationResult {
            success: false,
            language: "Assembly".to_string(),
            output: String::new(),
            errors: format!("Failed to write fixed source: {}", e),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
        };
    }
    
    let obj_path = source_path.with_extension("obj");
    let output_path = source_path.with_extension("exe");
    
    // Try GNU Assembler (as) + GCC - most compatible with decompiler output
    let gas_result = try_gas(&temp_path, &obj_path, &output_path);
    if gas_result.success {
        let _ = fs::remove_file(&temp_path);
        let _ = fs::remove_file(&obj_path);
        return CompilationResult {
            auto_fixes_applied: auto_fixes,
            ..gas_result
        };
    }
    
    // Try NASM
    let nasm_result = try_nasm(&temp_path, &obj_path, &output_path);
    if nasm_result.success {
        let _ = fs::remove_file(&temp_path);
        let _ = fs::remove_file(&obj_path);
        return CompilationResult {
            auto_fixes_applied: auto_fixes,
            ..nasm_result
        };
    }
    
    // Try MASM
    let masm_result = try_masm(&temp_path, &obj_path, &output_path);
    if masm_result.success {
        let _ = fs::remove_file(&temp_path);
        let _ = fs::remove_file(&obj_path);
        return CompilationResult {
            auto_fixes_applied: auto_fixes,
            ..masm_result
        };
    }
    
    // Try FASM
    let fasm_result = try_fasm(&temp_path, &output_path);
    if fasm_result.success {
        let _ = fs::remove_file(&temp_path);
        return CompilationResult {
            auto_fixes_applied: auto_fixes,
            ..fasm_result
        };
    }
    
    // Clean up
    let _ = fs::remove_file(&temp_path);
    let _ = fs::remove_file(&obj_path);
    
    // Try built-in assembler as fallback
    // IMPORTANT: Pass the ORIGINAL source, not the auto-fixed one!
    // The builtin assembler has its own wrapper detection that needs to see the original code
    let builtin_result = try_builtin_assembler(&source, &output_path, &start);
    if builtin_result.success {
        // Merge auto-fixes from builtin assembler
        let mut all_fixes = builtin_result.auto_fixes_applied.clone();
        all_fixes.extend(auto_fixes);
        return CompilationResult {
            auto_fixes_applied: all_fixes,
            ..builtin_result
        };
    }
    
    // If builtin assembler failed, return its error instead of generic message
    if !builtin_result.errors.is_empty() {
        return CompilationResult {
            auto_fixes_applied: auto_fixes,
            ..builtin_result
        };
    }
    
    // Only show "no assembler found" if builtin assembler didn't even try
    CompilationResult {
        success: false,
        language: "Assembly".to_string(),
        output: String::new(),
        errors: "No assembler found. Install: GNU as (binutils/GCC), NASM, MASM (Visual Studio), or FASM".to_string(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
        auto_fixes_applied: auto_fixes,
    }
}

fn auto_fix_assembly(source: &str, fixes: &mut Vec<String>) -> String {
    let mut fixed = source.to_string();
    
    // Remove BOM (Byte Order Mark) if present
    if fixed.starts_with('\u{feff}') {
        fixed = fixed.trim_start_matches('\u{feff}').to_string();
        fixes.push("Removed BOM (Byte Order Mark)".to_string());
    }
    
    // Remove any null bytes or other control characters that might cause issues
    if fixed.contains('\0') {
        fixed = fixed.replace('\0', "");
        fixes.push("Removed null bytes".to_string());
    }
    
    // Check if this is a disassembly listing (has addresses like "00001000  ret")
    let is_disassembly = source.lines()
        .take(20)
        .any(|line| {
            let trimmed = line.trim();
            // Check for hex address at start followed by instruction
            // Use byte-safe operations to avoid UTF-8 boundary issues
            if trimmed.len() < 10 {
                return false;
            }
            
            // Collect first 9 chars safely
            let chars: Vec<char> = trimmed.chars().take(9).collect();
            if chars.len() < 9 {
                return false;
            }
            
            // Check if first 8 are hex digits and 9th is whitespace
            chars[0..8].iter().all(|c| c.is_ascii_hexdigit()) &&
            chars[8].is_whitespace()
        });
    
    if is_disassembly {
        fixed = convert_disassembly_to_asm(&fixed, fixes);
    }
    
    // Remove decorative box-drawing characters that might cause issues
    let decorative_chars = ['╔', '║', '═', '╚', '╗', '╝', '┌', '│', '─', '└', '┐', '┘', '├', '┤', '┬', '┴', '┼'];
    let mut removed_decorative = false;
    for ch in decorative_chars {
        if fixed.contains(ch) {
            removed_decorative = true;
        }
    }
    if removed_decorative {
        // Remove lines that are purely decorative
        fixed = fixed.lines()
            .filter(|line| !line.chars().all(|c| decorative_chars.contains(&c) || c.is_whitespace()))
            .collect::<Vec<_>>()
            .join("\n");
        fixes.push("Removed decorative box-drawing characters".to_string());
    }
    
    // Add basic structure if missing
    if !fixed.contains("section") && !fixed.contains("SECTION") && !fixed.contains(".section") {
        let header = ".intel_syntax noprefix\n.section .text\n.global _start\n_start:\n";
        fixed = header.to_string() + &fixed;
        fixes.push("Added section and entry point".to_string());
    }
    
    // Fix common syntax differences
    if fixed.contains("OFFSET") {
        fixed = fixed.replace("OFFSET ", "");
        fixes.push("Removed OFFSET keywords".to_string());
    }
    
    // Convert Intel syntax directives to GNU as syntax
    if fixed.contains("section .") {
        fixed = fixed.replace("section .text", ".section .text");
        fixed = fixed.replace("section .data", ".section .data");
        fixed = fixed.replace("section .bss", ".section .bss");
        fixes.push("Converted to GNU assembler syntax".to_string());
    }
    
    // Fix global/extern declarations
    if fixed.contains("global ") {
        fixed = fixed.replace("global ", ".global ");
        fixes.push("Fixed global declarations".to_string());
    }
    if fixed.contains("extern ") {
        fixed = fixed.replace("extern ", ".extern ");
        fixes.push("Fixed extern declarations".to_string());
    }
    
    fixed
}

/// Convert disassembly listing to actual assembly source code
fn convert_disassembly_to_asm(disasm: &str, fixes: &mut Vec<String>) -> String {
    let mut output = String::new();
    let mut label_count = 0;
    let mut address_to_label = std::collections::HashMap::new();
    
    // First pass: identify all jump/call targets and create labels
    for line in disasm.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines, comments, and section headers
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with("Section:") || trimmed.starts_with("WARNING:") {
            continue;
        }
        
        // Parse address and instruction
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        // Check if first part is a hex address (use char count, not byte length)
        if parts[0].chars().count() == 8 && parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
            let address = parts[0];
            
            // Check for jump/call instructions with target addresses
            if parts.len() >= 3 {
                let mnemonic = parts[1];
                if mnemonic.starts_with("j") || mnemonic == "call" {
                    // Extract target address (e.g., "0x1234" or "1234")
                    let target = parts[2].trim_start_matches("0x");
                    if target.chars().all(|c| c.is_ascii_hexdigit()) {
                        address_to_label.entry(target.to_string()).or_insert_with(|| {
                            label_count += 1;
                            format!("loc_{}", target)
                        });
                    }
                }
            }
            
            // Check if this address is marked as entry point
            if trimmed.contains("ENTRY POINT") {
                address_to_label.insert(address.to_string(), "_start".to_string());
            }
        }
    }
    
    fixes.push(format!("Converted disassembly listing to assembly source ({} labels created)", label_count));
    
    // Second pass: generate assembly code
    output.push_str(".intel_syntax noprefix\n");
    output.push_str(".section .text\n");
    output.push_str(".global _start\n\n");
    
    for line in disasm.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines, section headers, and warnings
        if trimmed.is_empty() || trimmed.starts_with("Section:") || trimmed.starts_with("WARNING:") {
            continue;
        }
        
        // Keep comments
        if trimmed.starts_with(';') {
            output.push_str(&format!("    {}\n", trimmed));
            continue;
        }
        
        // Parse address and instruction
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        // Check if first part is a hex address (use char count, not byte length)
        if parts[0].chars().count() == 8 && parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
            let address = parts[0];
            
            // Add label if this address is a jump target
            if let Some(label) = address_to_label.get(address) {
                output.push_str(&format!("{}:\n", label));
            }
            
            // Extract instruction (everything after address)
            let instruction_start = trimmed.find(char::is_whitespace).unwrap_or(0);
            let instruction = trimmed[instruction_start..].trim();
            
            // Skip if no instruction
            if instruction.is_empty() {
                continue;
            }
            
            // Replace absolute addresses in jump/call instructions with labels
            let mut fixed_instruction = instruction.to_string();
            for (addr, label) in &address_to_label {
                fixed_instruction = fixed_instruction.replace(&format!("0x{}", addr), label);
                fixed_instruction = fixed_instruction.replace(addr, label);
            }
            
            // Remove byte encodings and extra whitespace
            // Format: "mnemonic  operands  ; comment" or just "mnemonic  operands"
            let inst_parts: Vec<&str> = fixed_instruction.split_whitespace().collect();
            if !inst_parts.is_empty() {
                let mnemonic = inst_parts[0];
                let operands = inst_parts[1..].join(" ");
                
                // Remove inline comments (everything after semicolon)
                let clean_operands = if let Some(pos) = operands.find(';') {
                    operands[..pos].trim()
                } else {
                    operands.trim()
                };
                
                if clean_operands.is_empty() {
                    output.push_str(&format!("    {}\n", mnemonic));
                } else {
                    output.push_str(&format!("    {} {}\n", mnemonic, clean_operands));
                }
            }
        }
    }
    
    output
}

fn try_gas(source: &Path, obj: &Path, exe: &Path) -> CompilationResult {
    let start = Instant::now();
    
    // GNU Assembler (as) - part of binutils, comes with GCC
    // This is the most compatible with Intel syntax assembly from decompilers
    let asm = Command::new("as")
        .args(&[
            source.to_str().unwrap(),
            "-o", obj.to_str().unwrap(),
            "--64",  // 64-bit mode
        ])
        .output();
    
    if let Ok(asm_out) = asm {
        if asm_out.status.success() {
            // Link with GCC
            let link = Command::new("gcc")
                .args(&[
                    obj.to_str().unwrap(),
                    "-o", exe.to_str().unwrap(),
                    "-nostartfiles",  // Don't use standard startup files
                    "-static",  // Static linking for portability
                ])
                .output();
            
            if let Ok(link_out) = link {
                let success = link_out.status.success();
                let mut output = String::from_utf8_lossy(&link_out.stdout).to_string();
                let mut errors = String::from_utf8_lossy(&link_out.stderr).to_string();
                
                // Combine assembler and linker output
                if !asm_out.stdout.is_empty() {
                    output = format!("Assembler: {}\n{}", String::from_utf8_lossy(&asm_out.stdout), output);
                }
                if !asm_out.stderr.is_empty() {
                    errors = format!("Assembler: {}\n{}", String::from_utf8_lossy(&asm_out.stderr), errors);
                }
                
                return CompilationResult {
                    success,
                    language: "Assembly (GNU as)".to_string(),
                    output,
                    errors,
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: if success { Some(exe.to_path_buf()) } else { None },
                    auto_fixes_applied: vec![],
                };
            }
        } else {
            // Return assembler errors if assembly failed
            return CompilationResult {
                success: false,
                language: "Assembly (GNU as)".to_string(),
                output: String::from_utf8_lossy(&asm_out.stdout).to_string(),
                errors: String::from_utf8_lossy(&asm_out.stderr).to_string(),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
                auto_fixes_applied: vec![],
            };
        }
    }
    
    CompilationResult {
        success: false,
        language: "Assembly".to_string(),
        output: String::new(),
        errors: String::new(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
        auto_fixes_applied: vec![],
    }
}

fn try_nasm(source: &Path, obj: &Path, exe: &Path) -> CompilationResult {
    let start = Instant::now();
    
    let asm = Command::new("nasm")
        .args(&["-f", "win64", source.to_str().unwrap(), "-o", obj.to_str().unwrap()])
        .output();
    
    if let Ok(asm_out) = asm {
        if asm_out.status.success() {
            let link = Command::new("gcc")
                .args(&[obj.to_str().unwrap(), "-o", exe.to_str().unwrap(), "-nostartfiles"])
                .output();
            
            if let Ok(link_out) = link {
                return CompilationResult {
                    success: link_out.status.success(),
                    language: "Assembly (NASM)".to_string(),
                    output: String::from_utf8_lossy(&link_out.stdout).to_string(),
                    errors: String::from_utf8_lossy(&link_out.stderr).to_string(),
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: if link_out.status.success() { Some(exe.to_path_buf()) } else { None },
                    auto_fixes_applied: vec![],
                };
            }
        }
    }
    
    CompilationResult {
        success: false,
        language: "Assembly".to_string(),
        output: String::new(),
        errors: String::new(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
        auto_fixes_applied: vec![],
    }
}

fn try_masm(source: &Path, obj: &Path, exe: &Path) -> CompilationResult {
    let start = Instant::now();
    
    let asm = Command::new("ml64")
        .args(&["/c", source.to_str().unwrap(), "/Fo", obj.to_str().unwrap()])
        .output();
    
    if let Ok(asm_out) = asm {
        if asm_out.status.success() {
            let link = Command::new("link")
                .args(&[obj.to_str().unwrap(), "/OUT:", exe.to_str().unwrap(), "/SUBSYSTEM:CONSOLE"])
                .output();
            
            if let Ok(link_out) = link {
                return CompilationResult {
                    success: link_out.status.success(),
                    language: "Assembly (MASM)".to_string(),
                    output: String::from_utf8_lossy(&link_out.stdout).to_string(),
                    errors: String::from_utf8_lossy(&link_out.stderr).to_string(),
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: if link_out.status.success() { Some(exe.to_path_buf()) } else { None },
                    auto_fixes_applied: vec![],
                };
            }
        }
    }
    
    CompilationResult {
        success: false,
        language: "Assembly".to_string(),
        output: String::new(),
        errors: String::new(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
        auto_fixes_applied: vec![],
    }
}

fn try_fasm(source: &Path, exe: &Path) -> CompilationResult {
    let start = Instant::now();
    
    let result = Command::new("fasm")
        .args(&[source.to_str().unwrap(), exe.to_str().unwrap()])
        .output();
    
    if let Ok(output) = result {
        return CompilationResult {
            success: output.status.success(),
            language: "Assembly (FASM)".to_string(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            errors: String::from_utf8_lossy(&output.stderr).to_string(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: if output.status.success() { Some(exe.to_path_buf()) } else { None },
            auto_fixes_applied: vec![],
        };
    }
    
    CompilationResult {
        success: false,
        language: "Assembly".to_string(),
        output: String::new(),
        errors: String::new(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
        auto_fixes_applied: vec![],
    }
}

fn try_builtin_assembler(source: &str, exe: &Path, start: &Instant) -> CompilationResult {
    use std::panic;
    
    // Detect if 64-bit or 32-bit based on source
    let is_64bit = source.contains("rax") || source.contains("rbx") || 
                   source.contains("rcx") || source.contains("rdx") ||
                   source.contains("r8") || source.contains("r9");
    
    // Show loading animation for large files (> 10MB)
    let show_animation = source.len() > 10_000_000;
    let _animation = if show_animation {
        println!("\n🦆 Starting assembly of large file ({:.1} MB)...", source.len() as f64 / 1_000_000.0);
        println!("This may take several minutes. Please be patient!\n");
        Some(crate::loading_animation::LoadingAnimation::new(
            &format!("Processing {:.1} MB assembly file...", source.len() as f64 / 1_000_000.0)
        ))
    } else {
        None
    };
    
    // Catch panics to prevent silent crashes
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut assembler = builtin_assembler::BuiltinAssembler::new(is_64bit);
        
        // Check if wrapper will be added
        let needs_wrapper = assembler.check_needs_wrapper(source);
        let mut fixes = vec![];
        if needs_wrapper {
            fixes.push("Added program entry point wrapper".to_string());
        }
        
        // NOTE: The hardcoded RVA check has been moved to compile_assembly_smart()
        // to happen BEFORE auto_fix_assembly() preprocessing (saves ~180 seconds)
        
        // Check if the source is mostly NOPs (indicates disassembled data/padding, not real code)
        let line_count = source.lines().count();
        let nop_count = source.lines().filter(|line| line.contains("nop")).count();
        let nop_percentage = if line_count > 0 {
            (nop_count as f64 / line_count as f64) * 100.0
        } else {
            0.0
        };
        
        // If more than 80% NOPs, this is likely disassembled data, not code
        if nop_percentage > 80.0 && line_count > 100 {
            return CompilationResult {
                success: false,
                language: "Assembly (Built-in)".to_string(),
                output: String::new(),
                errors: format!(
                    "❌ CANNOT ASSEMBLE: This appears to be disassembled DATA, not CODE.\n\n\
                     Analysis:\n\
                     • Total lines: {}\n\
                     • NOP instructions: {} ({:.1}%)\n\
                     • This is NOT executable code!\n\n\
                     ═══════════════════════════════════════════════════════════\n\
                     WHAT HAPPENED:\n\
                     ═══════════════════════════════════════════════════════════\n\
                     The disassembler tried to interpret the ENTIRE executable as code,\n\
                     including data sections, padding, and resources.\n\n\
                     When the disassembler encounters data bytes that look like NOP\n\
                     instructions (0x90), it outputs thousands of 'nop' lines.\n\n\
                     This is NOT real code - it's:\n\
                     • Data sections (.data, .rdata)\n\
                     • Padding between sections\n\
                     • Resources (icons, strings, etc.)\n\
                     • Debug information\n\
                     • Import/Export tables\n\n\
                     ═══════════════════════════════════════════════════════════\n\
                     WHY DISASSEMBLY FAILED:\n\
                     ═══════════════════════════════════════════════════════════\n\
                     The disassembler hit the 50,000 instruction limit while\n\
                     processing NOPs, which means:\n\n\
                     1. It never reached the actual code section\n\
                     2. The entry point detection failed\n\
                     3. It's disassembling from the wrong address\n\
                     4. The executable structure is too complex\n\n\
                     ═══════════════════════════════════════════════════════════\n\
                     SOLUTIONS:\n\
                     ═══════════════════════════════════════════════════════════\n\
                     ✅ Option 1: Use Decompilation (RECOMMENDED)\n\
                        • Select 'C Code' or 'Rust Code' from the dropdown\n\
                        • Press F5 to compile\n\
                        • Decompilation analyzes the PE structure properly\n\
                        • It finds the real entry point and code sections\n\
                        • You get readable, working code\n\n\
                     ✅ Option 2: Use Professional Disassembler\n\
                        • IDA Pro, Ghidra, Binary Ninja, radare2\n\
                        • These tools properly parse PE structure\n\
                        • They separate code from data\n\
                        • They handle complex executables\n\n\
                     ✅ Option 3: Fix the Disassembler\n\
                        • The built-in disassembler needs improvement\n\
                        • It should parse PE headers to find .text section\n\
                        • It should start from the actual entry point\n\
                        • It should stop at section boundaries\n\n\
                     ❌ DO NOT try to assemble this - it's not code!\n\n\
                     ═══════════════════════════════════════════════════════════\n\
                     TECHNICAL DETAILS:\n\
                     ═══════════════════════════════════════════════════════════\n\
                     A Windows PE executable has multiple sections:\n\
                     • .text   - Executable code (what you want)\n\
                     • .data   - Initialized data\n\
                     • .rdata  - Read-only data (strings, constants)\n\
                     • .rsrc   - Resources (icons, dialogs)\n\
                     • .reloc  - Relocation information\n\n\
                     The disassembler should ONLY disassemble the .text section,\n\
                     but it's currently trying to disassemble everything.\n\n\
                     ═══════════════════════════════════════════════════════════\n\
                     RECOMMENDATION: Use 'C Code' or 'Rust Code' decompilation\n\
                     ═══════════════════════════════════════════════════════════",
                    line_count,
                    nop_count,
                    nop_percentage
                ),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
                auto_fixes_applied: vec![],
            };
        }
        
        // NOTE: Hardcoded RVA check moved to the beginning (line ~987) for early detection
        // This saves ~180 seconds of wasted preprocessing on decompiled executables
        
        // Check if the source contains external calls (but not hardcoded RVAs)
        // OR if it's relocated code (contains import_/data_ labels from relocator)
        let has_external_calls = source.contains("call") && 
            (source.contains("kernel32") || source.contains("msvcrt") || 
             source.contains("user32") || source.contains("ntdll"));
        
        let is_relocated_code = source.contains("import_") || 
                                 (source.contains("data_") && source.contains("# Placeholder - original data not available"));
        
        // If it has external calls OR is relocated code, use the advanced PE builder with import tables
        if has_external_calls || is_relocated_code {
            if is_relocated_code {
                fixes.push("🔧 Detected relocated code - wrapping in PE structure".to_string());
            } else {
                fixes.push("🔧 Detected external API calls - using advanced PE builder with import tables".to_string());
            }
            
            // Detect which APIs are being called (only works for non-relocated code)
            let external_calls = if !is_relocated_code {
                crate::pe_builder::detect_external_calls(source)
            } else {
                vec![]  // Relocated code has labels, not DLL names
            };
            
            if !is_relocated_code && external_calls.is_empty() {
                fixes.push("⚠️  Warning: Detected call patterns but couldn't identify specific APIs".to_string());
                fixes.push("    The assembler will try to build anyway, but it may crash if APIs are missing".to_string());
            } else if !external_calls.is_empty() {
                fixes.push(format!("✓ Detected {} external API call(s):", external_calls.len()));
                for (dll, func) in &external_calls {
                    fixes.push(format!("  • {}!{}", dll, func));
                }
            }
            
            // Try to assemble the code first
            match assembler.assemble(source) {
                Ok(binary) => {
                    // Create PE with import tables
                    let mut pe_builder = crate::pe_builder::PEBuilder::new(binary.is_64bit);
                    pe_builder.add_code(binary.code.clone());
                    pe_builder.entry_point_rva = binary.entry_point;
                    
                    // Add detected imports (only for non-relocated code)
                    for (dll, func) in external_calls {
                        pe_builder.add_import(dll, func);
                    }
                    
                    match pe_builder.build(exe) {
                        Ok(_) => {
                            return CompilationResult {
                                success: true,
                                language: "Assembly (Advanced PE Builder)".to_string(),
                                output: format!(
                                    "✅ Successfully assembled with import tables!\n\n\
                                     Code size: {} bytes\n\
                                     Imported DLLs: {}\n\
                                     Total imports: {}\n\n\
                                     The executable now includes:\n\
                                     • Import Directory Table (IDT)\n\
                                     • Import Address Table (IAT)\n\
                                     • Import Lookup Table (ILT)\n\
                                     • Proper section layout (.text, .rdata, .idata)\n\n\
                                     ⚠️  NOTE: This works for NEW assembly code.\n\
                                     Decompiled code with hardcoded RVAs cannot be reassembled!",
                                    binary.code.len(),
                                    pe_builder.imports.len(),
                                    pe_builder.imports.iter().map(|d| d.functions.len()).sum::<usize>()
                                ),
                                errors: String::new(),
                                compilation_time_ms: start.elapsed().as_millis(),
                                executable_path: Some(exe.to_path_buf()),
                                auto_fixes_applied: fixes,
                            };
                        }
                        Err(e) => {
                            return CompilationResult {
                                success: false,
                                language: "Assembly (Advanced PE Builder)".to_string(),
                                output: String::new(),
                                errors: format!("Failed to create PE with import tables: {}", e),
                                compilation_time_ms: start.elapsed().as_millis(),
                                executable_path: None,
                                auto_fixes_applied: fixes,
                            };
                        }
                    }
                }
                Err(e) => {
                    return CompilationResult {
                        success: false,
                        language: "Assembly (Advanced PE Builder)".to_string(),
                        output: String::new(),
                        errors: format!("Assembly failed: {}", e),
                        compilation_time_ms: start.elapsed().as_millis(),
                        executable_path: None,
                        auto_fixes_applied: fixes,
                    };
                }
            }
        }
        
        match assembler.assemble(source) {
            Ok(binary) => {
                // Sanity check: if binary is too small, something went wrong
                if binary.code.len() < 10 {
                    return CompilationResult {
                        success: false,
                        language: "Assembly (Built-in)".to_string(),
                        output: String::new(),
                        errors: format!(
                            "Assembly produced suspiciously small output ({} bytes).\n\
                             This usually means the assembler couldn't parse the code.\n\n\
                             Possible reasons:\n\
                             • Unsupported instruction format\n\
                             • Invalid syntax\n\
                             • Missing section declarations\n\n\
                             Try using an external assembler (NASM, MASM, GAS) instead.",
                            binary.code.len()
                        ),
                        compilation_time_ms: start.elapsed().as_millis(),
                        executable_path: None,
                        auto_fixes_applied: vec![],
                    };
                }
                
                match builtin_assembler::create_pe_executable(&binary, exe) {
                    Ok(_) => {
                        CompilationResult {
                            success: true,
                            language: "Assembly (Built-in)".to_string(),
                            output: format!("Successfully assembled {} bytes of code", binary.code.len()),
                            errors: String::new(),
                            compilation_time_ms: start.elapsed().as_millis(),
                            executable_path: Some(exe.to_path_buf()),
                            auto_fixes_applied: fixes,
                        }
                    }
                    Err(e) => {
                        CompilationResult {
                            success: false,
                            language: "Assembly (Built-in)".to_string(),
                            output: String::new(),
                            errors: format!("Failed to create executable: {}", e),
                            compilation_time_ms: start.elapsed().as_millis(),
                            executable_path: None,
                            auto_fixes_applied: vec![],
                        }
                    }
                }
            }
            Err(e) => {
                CompilationResult {
                    success: false,
                    language: "Assembly (Built-in)".to_string(),
                    output: String::new(),
                    errors: format!("Assembly failed: {}", e),
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: None,
                    auto_fixes_applied: vec![],
                }
            }
        }
    }));
    
    // Stop animation before returning
    drop(_animation);
    
    match result {
        Ok(compilation_result) => compilation_result,
        Err(panic_info) => {
            let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };
            
            CompilationResult {
                success: false,
                language: "Assembly (Built-in)".to_string(),
                output: String::new(),
                errors: format!(
                    "CRASH DETECTED: The assembler panicked during compilation.\n\
                     Error: {}\n\n\
                     This is likely due to:\n\
                     1. Stack overflow (file too large for available stack)\n\
                     2. Out of memory (file requires more RAM than available)\n\
                     3. Malformed assembly causing infinite loop\n\n\
                     Suggestions:\n\
                     - Try increasing stack size: set RUST_MIN_STACK=16777216\n\
                     - Ensure you have at least 2GB free RAM\n\
                     - Check if the assembly file is valid\n\
                     - Try splitting the file into smaller parts",
                    panic_msg
                ),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
                auto_fixes_applied: vec![],
            }
        }
    }
}