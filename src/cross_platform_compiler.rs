/// Cross-platform C and Rust code compiler with perfect compilation support
/// Supports Windows, Linux, and macOS with automatic toolchain detection

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    C,
    Rust,
}

#[derive(Debug, Clone)]
pub struct CompilerInfo {
    pub name: String,
    pub version: String,
    pub available: bool,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub language: Language,
    pub compiler_used: String,
    pub platform: Platform,
    pub output: String,
    pub errors: String,
    pub warnings: String,
    pub compilation_time_ms: u128,
    pub executable_path: Option<PathBuf>,
    pub auto_fixes_applied: Vec<String>,
    pub optimization_level: String,
}

/// Detect current platform
pub fn detect_platform() -> Platform {
    #[cfg(target_os = "windows")]
    {
        Platform::Windows
    }
    #[cfg(target_os = "linux")]
    {
        Platform::Linux
    }
    #[cfg(target_os = "macos")]
    {
        Platform::MacOS
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Platform::Unknown
    }
}

/// Get appropriate executable extension for platform
pub fn get_executable_extension() -> &'static str {
    match detect_platform() {
        Platform::Windows => ".exe",
        Platform::Linux | Platform::MacOS => "",
        Platform::Unknown => "",
    }
}

/// Detect available C compilers on the system
pub fn detect_c_compilers() -> Vec<CompilerInfo> {
    let mut compilers = Vec::new();
    let platform = detect_platform();

    let compiler_names = match platform {
        Platform::Windows => {
            vec![
                ("gcc", "GNU GCC (MinGW)", true),
                ("clang", "LLVM Clang", true),
                ("cl", "Microsoft MSVC", true),
                ("tcc", "Tiny C Compiler", false),
            ]
        }
        Platform::Linux => {
            vec![
                ("gcc", "GNU GCC", true),
                ("clang", "LLVM Clang", true),
                ("icc", "Intel C Compiler", false),
                ("tcc", "Tiny C Compiler", false),
            ]
        }
        Platform::MacOS => {
            vec![
                ("clang", "LLVM Clang (Xcode)", true),
                ("gcc", "GNU GCC (via Homebrew)", true),
                ("icc", "Intel C Compiler", false),
            ]
        }
        Platform::Unknown => vec![],
    };

    for (cmd, name, is_primary) in compiler_names {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("Unknown version")
                    .to_string();
                
                compilers.push(CompilerInfo {
                    name: name.to_string(),
                    version,
                    available: true,
                    is_primary,
                });
            }
        }
    }

    compilers
}

/// Detect available Rust toolchain
pub fn detect_rust_toolchain() -> Option<String> {
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if output.status.success() {
            return Some(
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string(),
            );
        }
    }
    None
}

/// Compile C code with auto-fixes and cross-platform support
pub fn compile_c(source_path: &Path, optimization: &str) -> CompilationResult {
    let start = Instant::now();
    let mut auto_fixes = Vec::new();
    let platform = detect_platform();

    // Read source code
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            return CompilationResult {
                success: false,
                language: Language::C,
                compiler_used: "None".to_string(),
                platform,
                output: String::new(),
                errors: format!("Failed to read source file: {}", e),
                warnings: String::new(),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
                auto_fixes_applied: auto_fixes,
                optimization_level: optimization.to_string(),
            };
        }
    };

    // Apply auto-fixes
    let fixed_source = auto_fix_c_code(&source, &mut auto_fixes, &platform);

    // Write fixed source to temp file
    let temp_path = source_path.with_extension("fixed.c");
    if let Err(e) = fs::write(&temp_path, &fixed_source) {
        return CompilationResult {
            success: false,
            language: Language::C,
            compiler_used: "None".to_string(),
            platform,
            output: String::new(),
            errors: format!("Failed to write fixed source: {}", e),
            warnings: String::new(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
            optimization_level: optimization.to_string(),
        };
    }

    let ext = get_executable_extension();
    let output_path = source_path.with_file_name(
        source_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string() + ext,
    );

    // Get compiler list and try compilation
    let available_compilers = detect_c_compilers();
    
    for compiler_info in available_compilers.iter().filter(|c| c.available) {
        let compiler_name = extract_compiler_name(&compiler_info.name);
        
        let result = match platform {
            Platform::Windows => compile_c_windows(
                compiler_name,
                &temp_path,
                &output_path,
                optimization,
            ),
            Platform::Linux => compile_c_linux(
                compiler_name,
                &temp_path,
                &output_path,
                optimization,
            ),
            Platform::MacOS => compile_c_macos(
                compiler_name,
                &temp_path,
                &output_path,
                optimization,
            ),
            Platform::Unknown => continue,
        };

        if let Ok((success, stdout, stderr)) = result {
            let _ = fs::remove_file(&temp_path);
            
            if success {
                return CompilationResult {
                    success: true,
                    language: Language::C,
                    compiler_used: compiler_info.name.clone(),
                    platform,
                    output: stdout,
                    errors: stderr.clone(),
                    warnings: extract_warnings(&stderr),
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: Some(output_path),
                    auto_fixes_applied: auto_fixes,
                    optimization_level: optimization.to_string(),
                };
            }
        }
    }

    let _ = fs::remove_file(&temp_path);

    CompilationResult {
        success: false,
        language: Language::C,
        compiler_used: "None".to_string(),
        platform,
        output: String::new(),
        errors: "No C compiler found. Please install gcc, clang, or MSVC.".to_string(),
        warnings: String::new(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
        auto_fixes_applied: auto_fixes,
        optimization_level: optimization.to_string(),
    }
}

/// Compile Rust code with auto-fixes and cross-platform support
pub fn compile_rust(source_path: &Path, optimization: &str) -> CompilationResult {
    let start = Instant::now();
    let mut auto_fixes = Vec::new();
    let platform = detect_platform();

    // Check if rustc is available
    if detect_rust_toolchain().is_none() {
        return CompilationResult {
            success: false,
            language: Language::Rust,
            compiler_used: "rustc".to_string(),
            platform,
            output: String::new(),
            errors: "Rust toolchain not found. Please install Rust from https://rustup.rs/".to_string(),
            warnings: String::new(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
            optimization_level: optimization.to_string(),
        };
    }

    // Read source code
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            return CompilationResult {
                success: false,
                language: Language::Rust,
                compiler_used: "rustc".to_string(),
                platform,
                output: String::new(),
                errors: format!("Failed to read source file: {}", e),
                warnings: String::new(),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
                auto_fixes_applied: auto_fixes,
                optimization_level: optimization.to_string(),
            };
        }
    };

    // Apply auto-fixes
    let fixed_source = auto_fix_rust_code(&source, &mut auto_fixes);

    // Write fixed source to temp file
    let temp_path = source_path.with_extension("fixed.rs");
    if let Err(e) = fs::write(&temp_path, &fixed_source) {
        return CompilationResult {
            success: false,
            language: Language::Rust,
            compiler_used: "rustc".to_string(),
            platform,
            output: String::new(),
            errors: format!("Failed to write fixed source: {}", e),
            warnings: String::new(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
            optimization_level: optimization.to_string(),
        };
    }

    let ext = get_executable_extension();
    let output_path = source_path.with_file_name(
        source_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string() + ext,
    );

    // Compile with rustc
    let mut cmd = Command::new("rustc");
    cmd.arg(&temp_path).arg("-o").arg(&output_path);

    // Add optimization flags
    let opt_level = match optimization {
        "O3" | "fast" => "3",
        "O2" | "release" => "2",
        "O1" => "1",
        _ => "0",
    };
    cmd.arg("-C").arg(format!("opt-level={}", opt_level));

    let result = cmd.output();

    let _ = fs::remove_file(&temp_path);

    match result {
        Ok(output) => {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            CompilationResult {
                success,
                language: Language::Rust,
                compiler_used: "rustc".to_string(),
                platform,
                output: stdout,
                errors: stderr.clone(),
                warnings: extract_warnings(&stderr),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: if success { Some(output_path) } else { None },
                auto_fixes_applied: auto_fixes,
                optimization_level: optimization.to_string(),
            }
        }
        Err(e) => CompilationResult {
            success: false,
            language: Language::Rust,
            compiler_used: "rustc".to_string(),
            platform,
            output: String::new(),
            errors: format!("Failed to execute rustc: {}", e),
            warnings: String::new(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
            auto_fixes_applied: auto_fixes,
            optimization_level: optimization.to_string(),
        },
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

#[cfg(target_os = "windows")]
fn compile_c_windows(
    compiler: &str,
    source_path: &Path,
    output_path: &Path,
    optimization: &str,
) -> Result<(bool, String, String), String> {
    let mut cmd = Command::new(compiler);

    match compiler {
        "gcc" | "clang" => {
            cmd.arg(source_path)
                .arg("-o")
                .arg(output_path)
                .arg(format!("-{}", optimization))
                .arg("-Wall")
                .arg("-Wno-implicit-function-declaration");
        }
        "cl" => {
            cmd.arg("/nologo")
                .arg(source_path)
                .arg(format!("/Fe:{}", output_path.display()))
                .arg(format!("/{}", optimization));
        }
        _ => return Err("Unknown compiler".to_string()),
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((output.status.success(), stdout, stderr))
}

#[cfg(target_os = "linux")]
fn compile_c_linux(
    compiler: &str,
    source_path: &Path,
    output_path: &Path,
    optimization: &str,
) -> Result<(bool, String, String), String> {
    let mut cmd = Command::new(compiler);

    cmd.arg(source_path)
        .arg("-o")
        .arg(output_path)
        .arg(format!("-{}", optimization))
        .arg("-Wall")
        .arg("-Wno-implicit-function-declaration")
        .arg("-fPIE");

    let output = cmd.output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((output.status.success(), stdout, stderr))
}

#[cfg(target_os = "macos")]
fn compile_c_macos(
    compiler: &str,
    source_path: &Path,
    output_path: &Path,
    optimization: &str,
) -> Result<(bool, String, String), String> {
    let mut cmd = Command::new(compiler);

    cmd.arg(source_path)
        .arg("-o")
        .arg(output_path)
        .arg(format!("-{}", optimization))
        .arg("-Wall");

    let output = cmd.output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((output.status.success(), stdout, stderr))
}

#[cfg(not(target_os = "windows"))]
fn compile_c_windows(
    _compiler: &str,
    _source_path: &Path,
    _output_path: &Path,
    _optimization: &str,
) -> Result<(bool, String, String), String> {
    Err("Windows compilation not available on this platform".to_string())
}

#[cfg(not(target_os = "linux"))]
fn compile_c_linux(
    _compiler: &str,
    _source_path: &Path,
    _output_path: &Path,
    _optimization: &str,
) -> Result<(bool, String, String), String> {
    Err("Linux compilation not available on this platform".to_string())
}

#[cfg(not(target_os = "macos"))]
fn compile_c_macos(
    _compiler: &str,
    _source_path: &Path,
    _output_path: &Path,
    _optimization: &str,
) -> Result<(bool, String, String), String> {
    Err("macOS compilation not available on this platform".to_string())
}

/// Auto-fix common C code issues from decompilation
fn auto_fix_c_code(source: &str, fixes: &mut Vec<String>, platform: &Platform) -> String {
    let mut fixed = source.to_string();

    // Remove BOM if present
    if fixed.starts_with('\u{feff}') {
        fixed = fixed.trim_start_matches('\u{feff}').to_string();
        fixes.push("Removed BOM (Byte Order Mark)".to_string());
    }

    // Remove null bytes
    if fixed.contains('\0') {
        fixed = fixed.replace('\0', "");
        fixes.push("Removed null bytes".to_string());
    }

    // Fix undefined types
    if fixed.contains("undefined") {
        fixed = fixed.replace("undefined8", "unsigned long long");
        fixed = fixed.replace("undefined4", "unsigned int");
        fixed = fixed.replace("undefined2", "unsigned short");
        fixed = fixed.replace("undefined1", "unsigned char");
        fixed = fixed.replace("undefined", "int");
        fixes.push("Fixed undefined types".to_string());
    }

    // Fix Windows types if not on Windows
    if !matches!(platform, Platform::Windows) {
        if fixed.contains("HANDLE") {
            fixed = fixed.replace("HANDLE", "void*");
            fixes.push("Fixed Windows-specific types".to_string());
        }
    }

    // Add standard includes if missing
    if !fixed.contains("#include") {
        let includes = match platform {
            Platform::Windows => "#include <windows.h>\n#include <stdio.h>\n#include <stdlib.h>\n\n",
            Platform::Linux | Platform::MacOS => "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <unistd.h>\n\n",
            Platform::Unknown => "#include <stdio.h>\n#include <stdlib.h>\n\n",
        };
        fixed = includes.to_string() + &fixed;
        fixes.push("Added platform-specific includes".to_string());
    }

    // Add main function if missing
    if !fixed.contains("int main") && !fixed.contains("void main") {
        let entry_wrapper = "\n\nint main(int argc, char** argv) {\n    (void)argc; (void)argv;\n    return 0;\n}\n";
        fixed.push_str(entry_wrapper);
        fixes.push("Added main() entry point".to_string());
    }

    fixed
}

/// Auto-fix common Rust code issues from decompilation
fn auto_fix_rust_code(source: &str, fixes: &mut Vec<String>) -> String {
    let mut fixed = source.to_string();

    // Remove BOM if present
    if fixed.starts_with('\u{feff}') {
        fixed = fixed.trim_start_matches('\u{feff}').to_string();
        fixes.push("Removed BOM".to_string());
    }

    // Add main function if missing
    if !fixed.contains("fn main") {
        let entry = "\nfn main() {\n    // Auto-generated entry point\n}\n";
        fixed.push_str(entry);
        fixes.push("Added main() function".to_string());
    }

    // Fix common decompiler artifacts
    if fixed.contains("undefined") {
        fixed = fixed.replace("undefined8", "u64");
        fixed = fixed.replace("undefined4", "u32");
        fixed = fixed.replace("undefined2", "u16");
        fixed = fixed.replace("undefined1", "u8");
        fixed = fixed.replace("undefined", "i32");
        fixes.push("Fixed undefined type annotations".to_string());
    }

    fixed
}

/// Extract compiler name from full name
fn extract_compiler_name(full_name: &str) -> &str {
    match full_name {
        n if n.contains("GCC") => "gcc",
        n if n.contains("Clang") || n.contains("LLVM") => "clang",
        n if n.contains("MSVC") => "cl",
        n if n.contains("TCC") => "tcc",
        _ => "gcc",
    }
}

/// Extract warnings from compiler output
fn extract_warnings(output: &str) -> String {
    output
        .lines()
        .filter(|line| line.contains("warning") || line.contains("Warning"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let _platform = detect_platform();
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_c_autofixes() {
        let source = "undefined8 func() { return 0; }";
        let mut fixes = Vec::new();
        let fixed = auto_fix_c_code(source, &mut fixes, &Platform::Windows);
        assert!(fixed.contains("unsigned long long"));
        assert!(!fixes.is_empty());
    }

    #[test]
    fn test_rust_autofixes() {
        let source = "fn helper() {}";
        let mut fixes = Vec::new();
        let fixed = auto_fix_rust_code(source, &mut fixes);
        assert!(fixed.contains("fn main"));
    }
}