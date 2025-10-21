use rust_file_explorer::pe_builder;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "app.exe".to_string());

    if !Path::new(&binary_path).exists() {
        eprintln!("Error: Binary not found: {}", binary_path);
        return Err("File not found".into());
    }

    println!("Analyzing {}...\n", binary_path);
    let binary = fs::read(&binary_path)?;

    let info = pe_builder::extract_pe_info(&binary)?;

    println!("{:─^60}", " PE HEADER INFORMATION ");
    println!("Entry Point:    0x{:X}", info.entry_point);
    println!("Image Base:     0x{:X}", info.image_base);
    println!("Base of Code:   0x{:X}", info.base_of_code);
    println!("Base of Data:   0x{:X}", info.base_of_data);
    println!("Sections:       {}\n", info.sections.len());

    println!("{:─^60}", " SECTIONS ");
    for section in &info.sections {
        println!(
            "{:<8} VA: 0x{:08X}  Size: {:<8} Flags: 0x{:X}",
            section.name, section.virtual_address, section.size, section.flags
        );
    }

    if !info.imports.is_empty() {
        println!("\n{:─^60}", " IMPORTED FUNCTIONS ");
        for (dll, functions) in &info.imports.iter().fold(
            std::collections::HashMap::new(),
            |mut map, (d, f)| {
                map.entry(d.clone()).or_insert_with(Vec::new).push(f.clone());
                map
            }
        ) {
            println!("From {}:", dll);
            for func in functions.iter().take(5) {
                println!("  - {}", func);
            }
            if functions.len() > 5 {
                println!("  ... and {} more", functions.len() - 5);
            }
        }
    }

    if !info.exports.is_empty() {
        println!("\n{:─^60}", " EXPORTED FUNCTIONS ");
        for export in info.exports.iter().take(10) {
            println!("  0x{:X} - {}", export.1, export.0);
        }
        if info.exports.len() > 10 {
            println!("  ... and {} more", info.exports.len() - 10);
        }
    }

    Ok(())
}