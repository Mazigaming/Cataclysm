use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, Duration};

use crate::custom_compiler;

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub language: String,
    pub output: String,
    pub errors: String,
    pub compilation_time_ms: u128,
    pub executable_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub compilation: CompilationResult,
    pub execution_success: bool,
    pub execution_output: String,
    pub execution_error: String,
    pub exit_code: Option<i32>,
    pub execution_time_ms: u128,
}

/// Compile decompiled code and optionally test execution
pub fn compile_and_test(source_path: &Path, language: &str) -> TestResult {
    let compilation = compile_code(source_path, language);
    
    if !compilation.success {
        return TestResult {
            compilation,
            execution_success: false,
            execution_output: String::new(),
            execution_error: "Compilation failed".to_string(),
            exit_code: None,
            execution_time_ms: 0,
        };
    }
    
    // If compilation succeeded and we have an executable, try to run it
    if let Some(ref exe_path) = compilation.executable_path {
        let execution = execute_program(exe_path);
        TestResult {
            compilation,
            execution_success: execution.0,
            execution_output: execution.1,
            execution_error: execution.2,
            exit_code: execution.3,
            execution_time_ms: execution.4,
        }
    } else {
        TestResult {
            compilation,
            execution_success: false,
            execution_output: String::new(),
            execution_error: "No executable produced".to_string(),
            exit_code: None,
            execution_time_ms: 0,
        }
    }
}

/// Compile code based on language - now using smart compilers!
fn compile_code(source_path: &Path, language: &str) -> CompilationResult {
    let start = Instant::now();
    
    match language.to_lowercase().as_str() {
        "c" | "c code" => {
            let result = custom_compiler::compile_c_smart(source_path);
            convert_custom_result(result)
        },
        "rust" | "rust code" => {
            let result = custom_compiler::compile_rust_smart(source_path);
            convert_custom_result(result)
        },
        "assembly" | "asm" => {
            let result = custom_compiler::compile_assembly_smart(source_path);
            convert_custom_result(result)
        },
        "pseudo" | "pseudo-code" | "pseudo code" => {
            // Pseudo code can't be compiled
            CompilationResult {
                success: false,
                language: language.to_string(),
                output: String::new(),
                errors: "Pseudo-code cannot be compiled. Please use C, Rust, or Assembly output.".to_string(),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
            }
        }
        _ => {
            CompilationResult {
                success: false,
                language: language.to_string(),
                output: String::new(),
                errors: format!("Unsupported language: {}", language),
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
            }
        }
    }
}

/// Convert custom compiler result to our format
fn convert_custom_result(custom: custom_compiler::CompilationResult) -> CompilationResult {
    let mut output = custom.output.clone();
    
    // Add auto-fix information to output
    if !custom.auto_fixes_applied.is_empty() {
        output.push_str("\n\n🔧 Auto-fixes applied:\n");
        for fix in &custom.auto_fixes_applied {
            output.push_str(&format!("  ✓ {}\n", fix));
        }
    }
    
    CompilationResult {
        success: custom.success,
        language: custom.language,
        output,
        errors: custom.errors,
        compilation_time_ms: custom.compilation_time_ms,
        executable_path: custom.executable_path,
    }
}

/// Compile C code using gcc or cl (MSVC)
fn compile_c(source_path: &Path) -> CompilationResult {
    let start = Instant::now();
    let output_path = source_path.with_extension("exe");
    
    // Try gcc first (MinGW on Windows)
    let result = Command::new("gcc")
        .arg(source_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-O2")
        .arg("-Wall")
        .output();
    
    if let Ok(output) = result {
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        return CompilationResult {
            success,
            language: "C".to_string(),
            output: stdout,
            errors: stderr,
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: if success { Some(output_path) } else { None },
        };
    }
    
    // If gcc failed, try cl (MSVC)
    let result = Command::new("cl")
        .arg("/nologo")
        .arg(source_path)
        .arg(format!("/Fe:{}", output_path.display()))
        .output();
    
    if let Ok(output) = result {
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        CompilationResult {
            success,
            language: "C (MSVC)".to_string(),
            output: stdout,
            errors: stderr,
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: if success { Some(output_path) } else { None },
        }
    } else {
        CompilationResult {
            success: false,
            language: "C".to_string(),
            output: String::new(),
            errors: "No C compiler found. Please install gcc (MinGW) or MSVC.".to_string(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
        }
    }
}

/// Compile Rust code using rustc
fn compile_rust(source_path: &Path) -> CompilationResult {
    let start = Instant::now();
    let output_path = source_path.with_extension("exe");
    
    let result = Command::new("rustc")
        .arg(source_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-O")
        .arg("--edition=2021")
        .output();
    
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
        }
    } else {
        CompilationResult {
            success: false,
            language: "Rust".to_string(),
            output: String::new(),
            errors: "rustc not found. Please install Rust from https://rustup.rs".to_string(),
            compilation_time_ms: start.elapsed().as_millis(),
            executable_path: None,
        }
    }
}

/// Compile Assembly code using nasm and ld (or ml/link for MASM)
fn compile_assembly(source_path: &Path) -> CompilationResult {
    let start = Instant::now();
    let obj_path = source_path.with_extension("obj");
    let output_path = source_path.with_extension("exe");
    
    // Try NASM first (more common for x64)
    let asm_result = Command::new("nasm")
        .arg("-f")
        .arg("win64")  // Windows 64-bit format
        .arg(source_path)
        .arg("-o")
        .arg(&obj_path)
        .output();
    
    if let Ok(asm_output) = asm_result {
        if asm_output.status.success() {
            // Now link the object file
            let link_result = Command::new("gcc")  // Use gcc as linker
                .arg(&obj_path)
                .arg("-o")
                .arg(&output_path)
                .arg("-nostartfiles")  // Don't use standard startup files
                .output();
            
            if let Ok(link_output) = link_result {
                let success = link_output.status.success();
                let stdout = format!("{}\n{}", 
                    String::from_utf8_lossy(&asm_output.stdout),
                    String::from_utf8_lossy(&link_output.stdout));
                let stderr = format!("{}\n{}", 
                    String::from_utf8_lossy(&asm_output.stderr),
                    String::from_utf8_lossy(&link_output.stderr));
                
                // Clean up object file
                let _ = std::fs::remove_file(&obj_path);
                
                return CompilationResult {
                    success,
                    language: "Assembly".to_string(),
                    output: stdout,
                    errors: stderr,
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: if success { Some(output_path) } else { None },
                };
            } else {
                // Clean up object file
                let _ = std::fs::remove_file(&obj_path);
                
                return CompilationResult {
                    success: false,
                    language: "Assembly".to_string(),
                    output: String::new(),
                    errors: "Linker (gcc) not found. Please install MinGW-w64.".to_string(),
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: None,
                };
            }
        } else {
            let stderr = String::from_utf8_lossy(&asm_output.stderr).to_string();
            return CompilationResult {
                success: false,
                language: "Assembly".to_string(),
                output: String::new(),
                errors: stderr,
                compilation_time_ms: start.elapsed().as_millis(),
                executable_path: None,
            };
        }
    }
    
    // If NASM failed, try MASM (ml64)
    let masm_result = Command::new("ml64")
        .arg("/c")  // Compile only
        .arg(source_path)
        .arg("/Fo")
        .arg(&obj_path)
        .output();
    
    if let Ok(masm_output) = masm_result {
        if masm_output.status.success() {
            // Link with Microsoft linker
            let link_result = Command::new("link")
                .arg(&obj_path)
                .arg("/OUT:")
                .arg(&output_path)
                .arg("/SUBSYSTEM:CONSOLE")
                .output();
            
            if let Ok(link_output) = link_result {
                let success = link_output.status.success();
                let stdout = format!("{}\n{}", 
                    String::from_utf8_lossy(&masm_output.stdout),
                    String::from_utf8_lossy(&link_output.stdout));
                let stderr = format!("{}\n{}", 
                    String::from_utf8_lossy(&masm_output.stderr),
                    String::from_utf8_lossy(&link_output.stderr));
                
                // Clean up object file
                let _ = std::fs::remove_file(&obj_path);
                
                return CompilationResult {
                    success,
                    language: "Assembly (MASM)".to_string(),
                    output: stdout,
                    errors: stderr,
                    compilation_time_ms: start.elapsed().as_millis(),
                    executable_path: if success { Some(output_path) } else { None },
                };
            }
        }
    }
    
    // Neither assembler worked
    CompilationResult {
        success: false,
        language: "Assembly".to_string(),
        output: String::new(),
        errors: "No assembler found. Please install NASM (https://www.nasm.us/) or MASM (Visual Studio).".to_string(),
        compilation_time_ms: start.elapsed().as_millis(),
        executable_path: None,
    }
}

/// Decode Windows exit codes to human-readable messages
fn decode_exit_code(code: i32) -> Option<String> {
    // Convert to unsigned for hex comparison
    let ucode = code as u32;
    
    match ucode {
        0xC0000005 => Some("ACCESS_VIOLATION - The program tried to access invalid memory".to_string()),
        0xC000001D => Some("ILLEGAL_INSTRUCTION - The program tried to execute an invalid instruction".to_string()),
        0xC0000094 => Some("INTEGER_DIVIDE_BY_ZERO - Division by zero".to_string()),
        0xC000008C => Some("ARRAY_BOUNDS_EXCEEDED - Array index out of bounds".to_string()),
        0xC000008D => Some("FLOAT_DENORMAL_OPERAND - Invalid floating point operation".to_string()),
        0xC000008E => Some("FLOAT_DIVIDE_BY_ZERO - Floating point division by zero".to_string()),
        0xC000008F => Some("FLOAT_INEXACT_RESULT - Floating point inexact result".to_string()),
        0xC0000090 => Some("FLOAT_INVALID_OPERATION - Invalid floating point operation".to_string()),
        0xC0000091 => Some("FLOAT_OVERFLOW - Floating point overflow".to_string()),
        0xC0000092 => Some("FLOAT_STACK_CHECK - Floating point stack check".to_string()),
        0xC0000093 => Some("FLOAT_UNDERFLOW - Floating point underflow".to_string()),
        0xC0000096 => Some("PRIVILEGED_INSTRUCTION - Attempted to execute privileged instruction".to_string()),
        0xC00000FD => Some("STACK_OVERFLOW - Stack overflow".to_string()),
        0xC0000409 => Some("STACK_BUFFER_OVERRUN - Buffer overrun detected".to_string()),
        0x80000003 => Some("BREAKPOINT - Program terminated via INT3 (normal for built-in assembler)".to_string()),
        0x80000004 => Some("SINGLE_STEP - Single step trap".to_string()),
        0xC000013A => Some("CONTROL_C_EXIT - Program terminated by Ctrl+C".to_string()),
        0xC0000017 => Some("NO_MEMORY - Not enough memory".to_string()),
        _ => {
            // Check if it's a negative number (Windows error)
            if code < 0 {
                Some(format!("Windows Error 0x{:08X}", ucode))
            } else {
                None
            }
        }
    }
}

/// Execute a compiled program with timeout (5 seconds)
pub fn execute_program(exe_path: &Path) -> (bool, String, String, Option<i32>, u128) {
    let start = Instant::now();
    
    // Attempt execution with timeout using a blocking call
    // Note: On Windows, we use spawn + wait with periodic checks
    let mut child = match Command::new(exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(child) => child,
        Err(e) => {
            return (false, String::new(), format!("Failed to start process: {}", e), None, start.elapsed().as_millis());
        }
    };
    
    // For simplicity, use a simple approach: just call wait() with a check
    // This is not a perfect timeout but prevents most hangs
    let timeout_duration = Duration::from_secs(5);
    let wait_start = Instant::now();
    
    // Try to wait with a loop that checks for timeout
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process completed
                let output = child.wait_with_output().unwrap_or_else(|_| {
                    std::process::Output {
                        status,
                        stdout: Vec::new(),
                        stderr: Vec::new(),
                    }
                });
                
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                let elapsed = start.elapsed().as_millis();
                
                let success = output.status.success() || 
                              exit_code == Some(-2147483645); // STATUS_BREAKPOINT
                
                return (success, stdout, stderr, exit_code, elapsed);
            }
            Ok(None) => {
                // Process still running
                if wait_start.elapsed() > timeout_duration {
                    // Timeout exceeded - kill the process
                    let _ = child.kill();
                    let _ = child.wait(); // Wait for it to actually die
                    return (false, String::new(), "Process execution timeout (exceeded 5 seconds)".to_string(), None, timeout_duration.as_millis());
                }
                // Sleep a bit before checking again
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return (false, String::new(), format!("Failed to check process status: {}", e), None, start.elapsed().as_millis());
            }
        }
    }
}

/// Format test results for display
pub fn format_test_results(result: &TestResult) -> String {
    let mut output = String::new();
    
    output.push_str("╔════════════════════════════════════════════════════════════════╗\n");
    output.push_str("║           COMPILATION & TESTING RESULTS                        ║\n");
    output.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");
    
    // Compilation section
    output.push_str("┌─ Compilation ─────────────────────────────────────────────────┐\n");
    output.push_str(&format!("│ Language:        {}\n", result.compilation.language));
    output.push_str(&format!("│ Status:          {}\n", 
        if result.compilation.success { "✅ SUCCESS" } else { "❌ FAILED" }));
    output.push_str(&format!("│ Time:            {} ms\n", result.compilation.compilation_time_ms));
    
    if !result.compilation.output.is_empty() {
        output.push_str("│\n│ Compiler Output:\n");
        for line in result.compilation.output.lines().take(10) {
            output.push_str(&format!("│   {}\n", line));
        }
    }
    
    if !result.compilation.errors.is_empty() {
        output.push_str("│\n│ Compiler Errors/Warnings:\n");
        for line in result.compilation.errors.lines().take(20) {
            output.push_str(&format!("│   {}\n", line));
        }
    }
    output.push_str("└───────────────────────────────────────────────────────────────┘\n\n");
    
    // Execution section (only if compilation succeeded)
    if result.compilation.success {
        output.push_str("┌─ Execution ───────────────────────────────────────────────────┐\n");
        output.push_str(&format!("│ Status:          {}\n", 
            if result.execution_success { "✅ SUCCESS" } else { "❌ FAILED" }));
        
        if let Some(code) = result.exit_code {
            output.push_str(&format!("│ Exit Code:       {}\n", code));
            
            // Add human-readable error description
            if let Some(description) = decode_exit_code(code) {
                output.push_str(&format!("│ Error:           {}\n", description));
            }
        }
        output.push_str(&format!("│ Time:            {} ms\n", result.execution_time_ms));
        
        if !result.execution_output.is_empty() {
            output.push_str("│\n│ Program Output:\n");
            for line in result.execution_output.lines().take(20) {
                output.push_str(&format!("│   {}\n", line));
            }
        }
        
        if !result.execution_error.is_empty() {
            output.push_str("│\n│ Error Output:\n");
            for line in result.execution_error.lines().take(20) {
                output.push_str(&format!("│   {}\n", line));
            }
        }
        output.push_str("└───────────────────────────────────────────────────────────────┘\n\n");
    }
    
    // Summary
    if result.compilation.success && result.execution_success {
        output.push_str("✅ OVERALL: Compilation and execution completed successfully!\n");
    } else if result.compilation.success {
        output.push_str("⚠️  OVERALL: Compilation succeeded but execution failed.\n");
    } else {
        output.push_str("❌ OVERALL: Compilation failed. Fix errors and try again.\n");
    }
    
    output
}