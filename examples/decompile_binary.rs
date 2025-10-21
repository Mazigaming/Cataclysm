use rust_file_explorer::decompiler;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "notepad.exe".to_string());

    if !Path::new(&binary_path).exists() {
        eprintln!("Error: Binary not found: {}", binary_path);
        return Err("File not found".into());
    }

    println!("Loading {}...", binary_path);
    let binary = fs::read(&binary_path)?;

    println!("Decompiling...");
    let result = decompiler::decompile(&binary)?;

    println!("\n{:─^60}", " DECOMPILATION RESULTS ");
    println!("Functions detected: {}", result.functions.len());
    println!("API calls found: {}", result.api_calls.len());
    println!("Image base: 0x{:X}", result.image_base);
    println!("Entry point: 0x{:X}\n", result.entry_point);

    println!("{:─^60}", " PSEUDO-CODE ");
    println!("{}\n", result.pseudocode);

    println!("{:─^60}", " ASSEMBLY ");
    for line in result.assembly.lines().take(50) {
        println!("{}", line);
    }
    if result.assembly.lines().count() > 50 {
        println!("... ({} more lines)", result.assembly.lines().count() - 50);
    }

    Ok(())
}