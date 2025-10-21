# Setup Guide - Advanced Rust Decompiler

Complete setup instructions for Windows, Linux, and macOS with C/Rust compilation support.

## Quick Start

### Windows
```powershell
# Prerequisites: Install Rust
# https://rustup.rs/

# Install C compiler (choose one)
# Option 1: MinGW (recommended)
choco install mingw

# Option 2: MSVC (via Visual Studio)
# https://visualstudio.microsoft.com/

# Clone and build
git clone https://github.com/yourusername/rust-decompiler.git
cd rust-decompiler/rust_file_explorer
cargo build --release

# Run
.\target\release\rust_file_explorer.exe
```

### Linux (Ubuntu/Debian)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install C compiler
sudo apt update
sudo apt install build-essential gcc clang

# Clone and build
git clone https://github.com/yourusername/rust-decompiler.git
cd rust-decompiler/rust_file_explorer
cargo build --release

# Run
./target/release/rust_file_explorer
```

### macOS
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Xcode Command Line Tools
xcode-select --install

# Clone and build
git clone https://github.com/yourusername/rust-decompiler.git
cd rust-decompiler/rust_file_explorer
cargo build --release

# Run
./target/release/rust_file_explorer
```

---

## Detailed Setup

### 1. Install Rust

#### Windows
```powershell
# Download and run rustup installer
# https://rustup.rs/
# Or use chocolatey:
choco install rust
```

#### Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### macOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install C Compiler

#### Windows

**Option 1: MinGW (Recommended)**
```powershell
# Using Chocolatey
choco install mingw

# Verify installation
gcc --version
```

**Option 2: MSVC (Microsoft Visual C++)**
```powershell
# Install Visual Studio Build Tools
# https://visualstudio.microsoft.com/downloads/
# Select "C++ build tools" during installation

# Verify installation
cl.exe
```

**Option 3: Clang**
```powershell
choco install llvm
```

#### Linux (Ubuntu/Debian)

```bash
# GCC
sudo apt update
sudo apt install build-essential gcc
gcc --version

# Or Clang
sudo apt install clang
clang --version
```

#### Linux (Fedora/CentOS/RHEL)

```bash
# GCC
sudo dnf install gcc make

# Or Clang
sudo dnf install clang
```

#### macOS

```bash
# Xcode Command Line Tools (includes clang)
xcode-select --install

# Or via Homebrew
brew install gcc llvm

# Verify
clang --version
```

### 3. Clone Repository

```bash
git clone https://github.com/yourusername/rust-decompiler.git
cd rust-decompiler/rust_file_explorer
```

### 4. Build Project

#### Development Build
```bash
cargo build
```

#### Release Build (Optimized)
```bash
cargo build --release
# Binary at: target/release/rust_file_explorer[.exe]
```

#### Clean Build
```bash
cargo clean
cargo build --release
```

### 5. Run Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test compilation_test

# Run tests in release mode
cargo test --release
```

### 6. Code Quality Checks

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint with clippy
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

---

## Compilation Features

### Compile C Code

The decompiler can compile generated C code:

```rust
use rust_file_explorer::cross_platform_compiler::{compile_c, detect_c_compilers};

// Detect available compilers
let compilers = detect_c_compilers();

// Compile C code
let result = compile_c(
    Path::new("output.c"),
    "O2"  // Optimization level
);

if result.success {
    println!("Compiled successfully: {:?}", result.executable_path);
} else {
    eprintln!("Compilation failed: {}", result.errors);
}
```

### Compile Rust Code

```rust
use rust_file_explorer::cross_platform_compiler::compile_rust;

let result = compile_rust(
    Path::new("output.rs"),
    "release"  // Optimization level
);

if result.success {
    println!("Compiled: {:?}", result.executable_path);
}
```

---

## Platform-Specific Notes

### Windows

**Firewall Issues:**
- Compilation may require firewall access for first-time build
- Allow `cargo.exe` and compiler access if prompted

**Path Issues:**
- Use absolute paths or ensure proper escaping
- Environment variables should be set in PowerShell/CMD

**Linker Errors:**
- Ensure C compiler is in PATH
- Restart terminal after compiler installation
- Use `where gcc` to verify compiler is accessible

### Linux

**GCC Not Found:**
```bash
# Check if gcc is in PATH
which gcc

# If not, add to PATH
export PATH=$PATH:/usr/bin
```

**Permission Issues:**
```bash
# Ensure binary is executable
chmod +x target/release/rust_file_explorer

# Run with appropriate permissions
./target/release/rust_file_explorer
```

**GTK Dependencies (if using GUI):**
```bash
sudo apt install libgtk-3-dev
```

### macOS

**Xcode Acceptance:**
```bash
# Accept Xcode license
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
sudo xcodebuild -license accept
```

**Homebrew Packages:**
```bash
# If compilation fails, try Homebrew versions
brew install gcc
brew install llvm
```

**M1/ARM Macs:**
```bash
# Ensure correct architecture
rustup target add aarch64-apple-darwin

# Build for ARM
cargo build --target aarch64-apple-darwin
```

---

## Troubleshooting

### Build Fails with "cc1plus: error"

**Windows:**
```powershell
# Reinstall MinGW
choco uninstall mingw
choco install mingw
```

**Linux:**
```bash
sudo apt install g++ build-essential
```

### Cargo Not Found

```bash
# Check Rust installation
rustc --version
cargo --version

# Update Rust
rustup update

# Add to PATH if needed
export PATH="$HOME/.cargo/bin:$PATH"
```

### Compiler Not Found During Build

1. Verify compiler installation:
   ```bash
   gcc --version
   clang --version
   ```

2. Check PATH:
   ```bash
   # Windows
   echo %PATH%
   
   # Linux/macOS
   echo $PATH
   ```

3. Reinstall compiler if needed

### Out of Memory During Compilation

Reduce parallelism:
```bash
cargo build -j 2
```

### Permission Denied (Linux/macOS)

```bash
chmod +x target/release/rust_file_explorer
sudo ./target/release/rust_file_explorer
```

---

## Development Setup

### IDE Setup

**VS Code:**
1. Install "Rust Analyzer" extension
2. Install "CodeLLDB" for debugging
3. Create `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug",
            "cargo": {
                "args": ["build"],
                "filter": {
                    "name": "rust_file_explorer",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

**JetBrains IntelliJ IDEA:**
1. Install "Rust" plugin
2. Install "TOML" plugin
3. Open project in IDE

**Vim/Neovim:**
```bash
# Install rust-analyzer
rustup component add rust-analyzer

# Install completion plugin
# (see your specific plugin manager)
```

### Git Hooks

Set up pre-commit hooks:

```bash
# Create .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running fmt..."
cargo fmt -- --check

echo "Running clippy..."
cargo clippy -- -D warnings

echo "Running tests..."
cargo test

echo "All checks passed!"
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

---

## Performance Optimization

### Build Optimization

```bash
# Faster incremental builds
cargo build -j 4

# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache
cargo build --release
```

### Runtime Optimization

Edit `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## Contributing Setup

1. Fork repository
2. Clone your fork
3. Create feature branch: `git checkout -b feature/name`
4. Make changes
5. Run: `cargo fmt && cargo clippy && cargo test`
6. Push and create Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Support

- **Documentation**: See [docs/](docs/) directory
- **Issues**: [GitHub Issues](https://github.com/yourusername/rust-decompiler/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/rust-decompiler/discussions)

---

## Next Steps

1. Read [README.md](README.md) for features overview
2. Check [Quick Start](docs/v3.2.1/QUICK_START_V3.2.1.md)
3. Review [Contributing Guidelines](CONTRIBUTING.md)
4. Start using or contributing!