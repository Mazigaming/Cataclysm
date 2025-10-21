use rust_file_explorer::builtin_assembler::{BuiltinAssembler, create_pe_executable};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asm_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "input.asm".to_string());

    let output_file = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "output.exe".to_string());

    if !Path::new(&asm_file).exists() {
        eprintln!("Error: Assembly file not found: {}", asm_file);
        return Err("File not found".into());
    }

    println!("Reading {}...", asm_file);
    let asm_code = fs::read_to_string(&asm_file)?;

    println!("Assembling...");
    let mut assembler = BuiltinAssembler::new(true);

    match assembler.assemble(&asm_code) {
        Ok(binary) => {
            println!("✓ Assembled {} bytes\n", binary.code.len());

            println!("Creating PE executable: {}", output_file);
            create_pe_executable(&binary, Path::new(&output_file))?;
            println!("✓ Success\n");

            println!("{:─^60}", " ASSEMBLY DETAILS ");
            println!("Code section: {} bytes", binary.code.len());
            println!("Data section: {} bytes", binary.data.len());
            println!("Entry point: 0x{:X}", binary.entry_point);
            println!("Architecture: {}", if binary.is_64bit { "x64" } else { "x86" });
        }
        Err(e) => {
            eprintln!("✗ Assembly failed:\n{}", e);
            return Err(e.into());
        }
    }

    Ok(())
}