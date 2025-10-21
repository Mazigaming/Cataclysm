# Cataclysm

A production-grade, terminal-based decompiler that converts x86-64 Windows executables into readable pseudo-code, C, and Rust with Themes , Scripting and a ready compiler systems.

**v0.0.1** | [Setup Guide](SETUP.md) | [Contributing](CONTRIBUTING.md) | [Docs](docs/)

---

## What It Does

Takes a compiled `.exe`, `.dll`, or system binary and converts it back to human-readable code in multiple formats:

- **Assembly** - Complete disassembly with addresses
- **Pseudo-code** - High-level control flow representation  
- **C** - Compilable C source code
- **Rust** - Memory-safe Rust equivalent
- **PE Metadata** - Binary headers, sections, imports/exports

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/yourusername/rust-decompiler.git
cd rust-decompiler/rust_file_explorer

# Build release
cargo build --release

# Run
./target/release/rust_file_explorer  # Linux
.\target\release\rust_file_explorer.exe  # Windows
```

### Usage

1. Navigate to a file with arrow keys
2. Press Enter to select
3. Choose output format (Pseudo/C/Rust)
4. Results auto-open in `decompiler/projects/{name}/`

```
Output Structure:
├── name_full.asm              # Complete disassembly
├── name_decompiled.pseudo     # Pseudo-code
├── name_decompiled.c          # C source
├── name_decompiled.rs         # Rust source
├── name_pe_info.txt           # PE metadata
└── README.md                  # Generated summary
```

---

## Code Structure

### Core Components

```
src/
├── main.rs                      # Entry point, TUI initialization
├── lib.rs                       # Public module exports
│
├── decompiler.rs                # Decompilation engine
├── enhanced_disasm.rs           # High-level disassembly formatting
├── native_disassembler.rs       # C FFI bindings to capstone
│
├── pe_builder.rs                # Builds valid PE executables  
├── pe_fixer.rs                  # Validates/repairs PE structure
├── pe_reassembler.rs            # Reassembles PE from components
│
├── builtin_assembler.rs         # x86-64 assembler
├── assembler.rs                 # Assembler interface
├── assembly_relocator.rs        # Fixes relocatable code
│
├── cross_platform_compiler.rs   # C/Rust compilation (Windows/Linux)
├── compiler_tester.rs           # Compiler detection/validation
├── custom_compiler.rs           # Custom compiler integration
│
├── menu_system.rs               # TUI menu/navigation
├── theme_engine.rs              # Color/styling system
├── keybinds.rs                  # Input handling
├── loading_animation.rs         # UI animations
├── patch_ui.rs                  # Binary patching interface
│
├── scripting_api.rs             # Python/Lua script execution
├── script_editor.rs             # In-app script editor
│
├── anti_obfuscation.rs          # Code deobfuscation
├── windows_api_db.rs            # Windows API database
│
native/
└── disassembler.c               # Native C disassembler (capstone FFI)
```

### Navigation Guide

**For Decompilation Features:**
- Start in `decompiler.rs` for the main analysis logic
- `enhanced_disasm.rs` for output formatting
- `native_disassembler.rs` for C/capstone integration

**For PE/Binary Handling:**
- `pe_builder.rs` → creates executables
- `pe_fixer.rs` → validates structure
- `pe_reassembler.rs` → reconstructs from components

**For Compilation:**
- `cross_platform_compiler.rs` → compile decompiled code
- `builtin_assembler.rs` → x86-64 assembly
- `assembly_relocator.rs` → fix relocations

**For UI/UX:**
- `main.rs` → entry point and flow
- `menu_system.rs` → navigation
- `keybinds.rs` → controls

---

## Key Features

### Decompilation
- ✅ Multi-format output (Pseudo/C/Rust/ASM)
- ✅ Automatic function detection
- ✅ Control flow recovery
- ✅ Type inference

### Binary Handling
- ✅ PE file parsing (headers, sections, imports, exports)
- ✅ Support: EXE, DLL, SYS, OCX, SCR, DRV, EFI
- ✅ x86 and x64 architectures
- ✅ Automatic entry point detection

### Compilation
- ✅ Compile decompiled C code back to binary
- ✅ Compile generated Rust code
- ✅ Cross-platform: Windows, Linux
- ✅ Auto-fix decompiled code for compilation
- ✅ Multiple compiler support (MSVC, GCC, Clang)

### Development
- ✅ x86-64 assembler (Intel/AT&T syntax)
- ✅ Assembly relocation handling
- ✅ Binary patching UI
- ✅ Script execution (automation)
- ✅ Anti-obfuscation tools

---

## Examples

### Example 1: Decompile Windows Binary

```rust
use rust_file_explorer::decompiler;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary = std::fs::read("notepad.exe")?;
    let result = decompiler::decompile(&binary)?;
    
    println!("Functions found: {}", result.functions.len());
    println!("Assembly:\n{}", result.assembly);
    println!("Pseudo-code:\n{}", result.pseudocode);
    
    Ok(())
}
```

### Example 2: Compile Decompiled C Code

```rust
use rust_file_explorer::cross_platform_compiler;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = cross_platform_compiler::compile_c(
        Path::new("decompiled.c"),
        "O2"  // Optimization level
    )?;
    
    if result.success {
        println!("✓ Compiled: {:?}", result.executable_path);
    } else {
        eprintln!("Compilation failed:\n{}", result.errors);
    }
    
    Ok(())
}
```

### Example 3: Assemble and Create PE

```rust
use rust_file_explorer::builtin_assembler::{BuiltinAssembler, create_pe_executable};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asm = r#"
        mov eax, 1
        mov ecx, 0
        cmp eax, ecx
        je exit
        mov eax, 42
        ret
    exit:
        xor eax, eax
        ret
    "#;
    
    let mut assembler = BuiltinAssembler::new(true);
    let binary = assembler.assemble(asm)?;
    create_pe_executable(&binary, Path::new("output.exe"))?;
    
    println!("✓ Created PE executable");
    Ok(())
}
```

### Example 4: Extract PE Metadata

```rust
use rust_file_explorer::pe_builder;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary = fs::read("app.exe")?;
    let info = pe_builder::extract_pe_info(&binary)?;
    
    println!("Entry Point: 0x{:X}", info.entry_point);
    println!("Image Base: 0x{:X}", info.image_base);
    println!("Sections: {}", info.sections.len());
    
    for section in &info.sections {
        println!("  - {}: {} bytes", section.name, section.size);
    }
    
    Ok(())
}
```

Run examples: `cargo run --example analyze_pe`

---

## Building from Source

### Requirements

- **Rust:** Latest stable (install from [rustup.rs](https://rustup.rs))
- **C Compiler:** 
  - Windows: MSVC or MinGW
  - Linux: GCC or Clang

### Build Options

```bash
# Development build (faster compilation)
cargo build

# Release build (optimized binary)
cargo build --release

# Run tests
cargo test --release

# Generate documentation
cargo doc --open

# Check code quality
cargo fmt && cargo clippy -- -D warnings
```

### Platform-Specific Notes

**Windows:**
- Ensure Visual C++ Build Tools or MSVC is installed
- Or install MinGW: `choco install mingw`

**Linux (Ubuntu/Debian):**
```bash
sudo apt install build-essential gcc clang
```

---

## Architecture

### High-Level Flow

```
Binary Input
    ↓
PE Header Parsing ───→ Extract metadata, entry point
    ↓
Disassembly ───────→ Convert bytes to instructions
    ↓
Analysis Pass ──────→ Detect functions, imports, xrefs
    ↓
Decompilation ──────→ Generate pseudo-code
    ↓
Code Generation ────→ Output C/Rust/ASM
    ↓
Output Files ───────→ Save to project folder
```

### Module Dependencies

```
main.rs (entry point)
    ├── menu_system (UI)
    ├── decompiler (analysis)
    │   ├── enhanced_disasm
    │   ├── native_disassembler
    │   └── pe_builder
    └── cross_platform_compiler
        ├── builtin_assembler
        └── compiler detection
```

---

## Performance

| Metric | Value |
|--------|-------|
| Build Time | ~10s (release) |
| Binary Size | ~2.5 MB |
| Decompilation Speed | ~100KB/s |
| Memory Usage | ~50-100MB typical |

**Benchmarks:** See `docs/performance/`

---

## Supported Formats

### Input
- PE32 (32-bit executables)
- PE32+ (64-bit executables)  
- .EXE, .DLL, .SYS, .OCX, .SCR, .DRV, .EFI

### Output
- Plain text assembly (`.asm`)
- Pseudo-code (`.pseudo`)
- C source code (`.c`)
- Rust source code (`.rs`)
- PE metadata (`.txt`)
- Markdown documentation

### Architectures
- x86 (32-bit)
- x64 (64-bit)

---

## Use Cases

**Security & Malware Analysis**
- Reverse engineer malware
- Analyze suspicious binaries
- Vulnerability research

**Software Engineering**  
- Recover legacy code
- Understand closed-source libraries
- Binary auditing

**Education**
- Learn assembly language
- Understand compilation
- Study compiler optimizations

**Research**
- Binary format analysis
- Code pattern recognition
- Decompilation algorithms

---

## Technical Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (1.70+) |
| Disassembly | iced-x86 + capstone |
| PE Parsing | goblin |
| UI | crossterm + custom |
| Platform | Windows, Linux |

---

## Status

- **Version:** 3.2.1
- **Status:** ✅ Production Ready
- **Tests:** ✅ Passing
- **Documentation:** ✅ Complete
- **Platform Support:** ✅ Windows/Linux

---

## License

MIT License - See [LICENSE](LICENSE) for educational/research disclaimer.

**Educational Use Only** - This tool is for authorized security research and legitimate analysis. Unauthorized reverse engineering of copyrighted software may violate laws. Always have permission before analyzing binaries you don't own.

---

## Roadmap

### ✅ Completed (v0.0.1)
- Multi-format decompilation
- PE parsing & metadata
- Cross-platform compilation
- Project organization
- Full assembly output

### 🚧 In Progress (v0.0.2)
- Enhanced type inference
- Struct reconstruction
- Better import resolution

### 📋 Planned (v0.0.3)
- Plugin system
- Interactive debugger
- Advanced CFG analysis
- Incremental compilation
---

## Quick Links

**Getting Started**
- 📖 [Setup Guide](SETUP.md)

**Documentation**
- 📚 [Full Docs](docs/)

**Resources**
- 💻 [Examples](examples/)
- ❓ [FAQ](add me on discord: archangel1911)
- 📝 [Changelog](Not yet out)
---

**Made with 🦀 Rust** | Built for security research and education
