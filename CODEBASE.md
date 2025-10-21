# Codebase Guide

A comprehensive overview of the project structure and how to navigate the code.

## Quick Navigation

- **Entry Point:** `src/main.rs` - Application initialization and TUI setup
- **Public API:** `src/lib.rs` - Module exports for library usage
- **Core Logic:** `src/decompiler.rs` - Main decompilation engine
- **Examples:** `examples/` - Working code samples
- **Tests:** `tests/` - Integration tests

## Directory Structure

```
rust_file_explorer/
├── src/                          # Source code
│   ├── main.rs                   # Entry point, TUI loop
│   ├── lib.rs                    # Module exports
│   │
│   ├── DECOMPILATION ENGINE
│   ├── decompiler.rs             # Core analysis & code generation
│   ├── enhanced_disasm.rs        # High-level output formatting
│   ├── native_disassembler.rs    # C FFI to capstone
│   │
│   ├── PE/BINARY HANDLING
│   ├── pe_builder.rs             # PE executable creation
│   ├── pe_fixer.rs               # PE validation & repair
│   ├── pe_reassembler.rs         # Reconstruct PE from components
│   │
│   ├── ASSEMBLY & COMPILATION
│   ├── builtin_assembler.rs      # x86-64 assembler (main)
│   ├── assembler.rs              # Assembler interface
│   ├── assembly_relocator.rs     # Fix relocatable code
│   ├── cross_platform_compiler.rs # C/Rust compilation
│   ├── compiler_tester.rs        # Compiler detection
│   ├── custom_compiler.rs        # Custom compiler support
│   │
│   ├── USER INTERFACE
│   ├── main.rs (continued)       # Main TUI event loop
│   ├── menu_system.rs            # Menu navigation
│   ├── keybinds.rs               # Input handling
│   ├── theme_engine.rs           # Styling/colors
│   ├── loading_animation.rs      # Spinner animations
│   ├── patch_ui.rs               # Binary patching UI
│   │
│   ├── ADVANCED FEATURES
│   ├── scripting_api.rs          # Python/Lua execution
│   ├── script_editor.rs          # In-app script editor
│   ├── anti_obfuscation.rs       # Code deobfuscation
│   ├── windows_api_db.rs         # Windows API names
│
├── native/                       # C/C++ code
│   └── disassembler.c            # Capstone FFI bindings
│
├── examples/                     # Working examples
│   ├── decompile_binary.rs       # Decompile a binary
│   ├── compile_code.rs           # Compile C code
│   ├── assemble_code.rs          # Assemble x86-64
│   └── analyze_pe.rs             # Extract PE metadata
│
├── tests/                        # Integration tests
│   └── test_pe_reassembler.rs    # PE reassembly tests
│
├── docs/                         # Documentation
│   ├── general/                  # General guides
│   ├── v2.0/, v3.0/, v3.1/      # Version docs
│   ├── v3.2/, v3.2.1/           # Latest docs
│   └── README.md                 # Doc index
│
├── build.rs                      # Build script (C compilation)
├── Cargo.toml                    # Project manifest
├── Cargo.lock                    # Dependency lock file
└── .github/workflows/            # CI/CD pipelines
    ├── ci.yml                    # Testing pipeline
    └── release.yml               # Release automation
```

## Module Map

### Core Decompilation Flow

```
User Input (select binary)
    ↓
pe_builder::extract_pe_info()
    ↓
decompiler::decompile()
    ├→ parse binary as assembly
    ├→ filter junk instructions
    ├→ analyze control flow
    ├→ infer types
    └→ generate pseudo-code/C/Rust
    ↓
enhanced_disasm::format_output()
    ↓
Output: { pseudocode, c_code, rust_code, assembly }
```

### Compilation Flow

```
Generated C/Rust Source
    ↓
cross_platform_compiler::compile_*()
    ├→ Detect available compilers
    ├→ Auto-fix code if needed
    ├→ Invoke compiler
    └→ Collect errors/warnings
    ↓
Binary Output or Errors
```

## Module Details

### Entry Point: `main.rs`

Handles:
- Application initialization
- File browser UI loop
- User input/navigation
- Output directory management

**Key Functions:**
- `main()` - Entry point
- TUI event loop - Handles key presses

### Decompilation: `decompiler.rs`

Core analysis engine performing:
1. Instruction parsing
2. Function boundary detection
3. Control flow analysis
4. Type inference
5. Code generation

**Public Functions:**
- `translate_to_pseudo()` - Generate pseudo-code
- `translate_to_c()` - Generate C code
- `translate_to_rust()` - Generate Rust code

### PE Handling: `pe_builder.rs`

Parses PE headers and creates executables:
- Extract metadata (entry point, sections, imports)
- Create valid PE from binary components
- Handle both PE32 and PE32+ formats

**Key Functions:**
- `extract_pe_info()` - Parse PE structure
- `create_pe_executable()` - Build PE file
- `validate_pe()` - Check PE integrity

### Assembly: `builtin_assembler.rs`

Full x86-64 assembler:
- Parse Intel syntax assembly
- Resolve labels and relocations
- Generate machine code
- Create executable sections

**Key Functions:**
- `BuiltinAssembler::assemble()` - Assemble source
- `create_pe_executable()` - Build PE from binary

### Compilation: `cross_platform_compiler.rs`

Cross-platform C and Rust compilation:
- Detect available compilers (MSVC, GCC, Clang)
- Auto-fix decompiled code
- Invoke compiler with proper flags
- Parse compilation results

**Key Functions:**
- `compile_c()` - Compile C source
- `compile_rust()` - Compile Rust source
- `detect_c_compilers()` - Find installed compilers

### UI: `menu_system.rs`, `theme_engine.rs`, `keybinds.rs`

Terminal UI components:
- File browser navigation
- Menu rendering
- Theme customization
- Input handling

## How to Add Features

### Adding a New Decompilation Pass

1. Add logic to `decompiler.rs`
2. Call new function from appropriate translation routine
3. Update output generators
4. Add test in `tests/`

Example:
```rust
fn analyze_new_pattern(instructions: &[Instruction]) -> Vec<NewAnalysis> {
    // Implementation
}

pub fn translate_to_pseudo_with_pe(...) {
    // ... existing code ...
    let new_analysis = analyze_new_pattern(&instructions);
    // Use in pseudo-code generation
}
```

### Adding Compiler Support

1. Add detection in `cross_platform_compiler.rs`
2. Add auto-fix rules if needed
3. Update `detect_c_compilers()` function

Example:
```rust
fn detect_custom_compiler() -> Option<CompilerInfo> {
    // Check for compiler in PATH
}
```

### Adding New Output Format

1. Create new generator function in `decompiler.rs`
2. Add public export in `lib.rs`
3. Call from `main.rs` when user selects format
4. Save result to project folder

Example:
```rust
pub fn translate_to_format(...) -> String {
    // Generate format
}
```

## Testing

### Run All Tests
```bash
cargo test --release
```

### Run Specific Test
```bash
cargo test test_name
```

### Run with Output
```bash
cargo test -- --nocapture
```

## Code Style

### Conventions
- Use `snake_case` for functions/variables
- Use `PascalCase` for types/traits
- Group related functions together
- Use meaningful names (no single-letter vars except loops)

### Documentation
- Add doc comments to public functions
- Complex algorithms should have inline comments
- Keep comments focused on WHY, not WHAT

### Error Handling
- Return `Result<T, E>` for fallible operations
- Use `?` operator to propagate errors
- Log errors appropriately

## Performance Considerations

### Hot Paths
- `decompiler.rs` instruction parsing - Called for every instruction
- `enhanced_disasm.rs` formatting - Called for entire output
- `builtin_assembler.rs` parsing - Called for each assembly line

### Optimization Points
- Instruction parsing uses regex (could use bytecode parser)
- Function detection is linear (could use heuristics)
- Type inference is basic (could use static analysis)

## Dependencies

### Key Crates
| Crate | Purpose | Version |
|-------|---------|---------|
| crossterm | Terminal UI | ~0.27 |
| goblin | PE parsing | ~0.8 |
| iced-x86 | Disassembly | ~1.20 |
| regex | Pattern matching | ~1.10 |

**Note:** Capstone is used via C FFI in `native/disassembler.c`

## Debugging Tips

### Enable Verbose Output
```bash
RUST_LOG=debug cargo run --example decompile_binary notepad.exe
```

### Inspect Intermediate Results
- Add `println!()` or use `dbg!()` macro
- Run with `--nocapture`: `cargo test -- --nocapture`

### Profile Performance
```bash
cargo flamegraph -- notepad.exe
```

### View Generated Assembly
```bash
rustc --emit asm src/main.rs
```

## Common Tasks

### Adding a New Example
1. Create `examples/my_example.rs`
2. Implement `main() -> Result<(), Box<dyn std::error::Error>>`
3. Run: `cargo run --example my_example`

### Running in Development
```bash
cargo run -- /path/to/binary.exe
```

### Building Release
```bash
cargo build --release
# Binary: target/release/rust_file_explorer.exe
```

### Checking Code Quality
```bash
cargo fmt          # Format code
cargo clippy       # Lint code
cargo test         # Run tests
```

## Project Statistics

| Metric | Value |
|--------|-------|
| Total Lines of Code | ~15,000 |
| Rust Modules | 23 |
| Examples | 4 |
| Supported Formats | 5 (ASM, Pseudo, C, Rust, PE Info) |
| Architectures | x86-64 |

## Resources

- **Rust Book:** https://doc.rust-lang.org/book/
- **x86-64 ISA:** https://www.amd.com/system/files/TechDocs/AMD64_Architecture_Programmers_Manual.pdf
- **PE Format:** https://en.wikipedia.org/wiki/Portable_Executable
- **Capstone:** http://www.capstone-engine.org/

## Getting Help

1. Check existing issues on GitHub
2. Search documentation in `docs/`
3. Look at examples in `examples/`
4. Read relevant module documentation
5. Ask in project discussions

---

**Last Updated:** January 2025 | **Version:** 3.2.1