use rust_file_explorer::cross_platform_compiler;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "decompiled.c".to_string());

    let optimization = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "O2".to_string());

    if !Path::new(&source_file).exists() {
        eprintln!("Error: Source file not found: {}", source_file);
        return Err("File not found".into());
    }

    println!("Compiling {} with optimization level {}...\n", source_file, optimization);

    let result = cross_platform_compiler::compile_c(
        Path::new(&source_file),
        &optimization
    )?;

    println!("{:─^60}", " COMPILATION RESULTS ");
    println!("Status: {}", if result.success { "✓ SUCCESS" } else { "✗ FAILED" });
    println!("Time: {:.2}s", result.compilation_time);

    if let Some(ref exe) = result.executable_path {
        println!("Output: {}\n", exe.display());
    }

    if !result.warnings.is_empty() {
        println!("{:─^60}", " WARNINGS ");
        for warning in &result.warnings {
            println!("  {}", warning);
        }
        println!();
    }

    if !result.errors.is_empty() {
        println!("{:─^60}", " ERRORS ");
        for error in &result.errors {
            println!("  {}", error);
        }
        println!();
    }

    println!("Compiler: {}", result.compiler_used);
    println!("Optimization: {}", result.optimization_level);

    Ok(())
}