# Contributing to Advanced Rust Decompiler

Thank you in advance for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Respect intellectual property rights
- Focus on responsible disclosure

## How to Contribute
Just add me on discord: archangel1911
and let me know

### Reporting Issues

When reporting bugs or issues:

1. **Check existing issues** - Avoid duplicates
2. **Be descriptive** - Include:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Your environment (OS, Rust version, etc.)
3. **Include logs** - Attach error messages or debug output
4. **Minimal reproducible example** - If possible

### Suggesting Features

For feature requests:

1. **Check roadmap** - See if already planned in docs/general/ROADMAP_V3.2_TO_V4.0.md
2. **Describe the use case** - Why is this feature needed?
3. **Provide examples** - How would users interact with it?
4. **Consider scope** - Will this benefit the broader community?

### Submitting Code

#### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-decompiler.git
cd rust-decompiler/rust_file_explorer

# Install Rust (if not installed)
# https://rustup.rs/

# Build the project
cargo build

# Run tests
cargo test

# Run with debug output
cargo run
```

#### Development Workflow

1. **Create a branch**
   ```bash
   git checkout -b feature/description-or-fix/issue-number
   ```

2. **Make changes**
   - Write clear, idiomatic Rust code
   - Add comments for complex logic
   - Follow existing code style

3. **Test your changes**
   ```bash
   cargo test
   cargo build --release
   cargo clippy -- -D warnings
   cargo fmt
   ```

4. **Commit with clear messages**
   ```bash
   git commit -m "Brief description

   Detailed explanation if needed
   - Bullet point 1
   - Bullet point 2
   
   Fixes #issue-number (if applicable)
   ```

5. **Push and create Pull Request**
   ```bash
   git push origin feature/description
   ```

#### Code Style Guidelines

- **Rust formatting**: Use `cargo fmt`
- **Linting**: Pass `cargo clippy` without warnings
- **Documentation**: Document public APIs and complex logic
- **Comments**: Explain "why", not "what"
- **Naming**: Use clear, descriptive names

Example:
```rust
/// Detects platform and returns appropriate compiler flags
/// 
/// # Returns
/// A vector of compiler-specific optimization flags
pub fn get_platform_flags() -> Vec<String> {
    // Implementation
}
```

#### Testing

- Add tests for new functionality
- Test on Windows, Linux, and macOS if possible
- Include both positive and negative test cases

```rust
#[test]
fn test_c_compilation_windows() {
    // Test C compilation on Windows
}

#[test]
fn test_rust_compilation_cross_platform() {
    // Test Rust compilation across platforms
}
```

### PR Guidelines

- **Keep PRs focused** - One feature/fix per PR
- **Update documentation** - Include relevant docs changes
- **Reference issues** - Link to related issues
- **Be responsive** - Address review feedback promptly
- **Sign commits** - Use GPG signing if possible: `git commit -S -m "message"`

## Project Structure

```
rust_file_explorer/
├── src/
│   ├── main.rs                    # Entry point
│   ├── lib.rs                     # Library exports
│   ├── decompiler.rs             # Core decompilation
│   ├── custom_compiler.rs        # Compilation support
│   ├── cross_platform_compiler.rs # Cross-platform compilation
│   ├── pe_reassembler.rs         # PE file reassembly
│   └── ... (other modules)
├── native/
│   └── disassembler.c            # Native C code
├── docs/                         # Documentation
├── examples/                     # Example code
└── tests/                        # Integration tests
```

## Documentation

When contributing, ensure documentation is updated:

- Update README.md if user-facing changes
- Update relevant docs in docs/ directory
- Add inline code comments for complex logic
- Update CHANGELOG if significant changes

## Performance Considerations

- Profile code before optimization
- Use `cargo bench` for benchmarking
- Document performance implications
- Consider memory usage for large files

## Cross-Platform Development

Test on multiple platforms:

- **Windows 10/11** - Primary target
- **Ubuntu/Debian** - Linux support
- **macOS** (optional) - Additional support

Platform-specific code:
```rust
#[cfg(target_os = "windows")]
fn platform_specific_function() { }

#[cfg(target_os = "linux")]
fn platform_specific_function() { }

#[cfg(target_os = "macos")]
fn platform_specific_function() { }
```

## Security

- Don't commit credentials or secrets
- Report security vulnerabilities privately
- Follow responsible disclosure practices
- Respect intellectual property rights

## Questions?

- Check documentation: https://github.com/yourusername/rust-decompiler/tree/main/docs
- Review existing issues and discussions
- Start a new discussion for questions

## Commit Message Convention

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Example:
```
feat(compiler): add cross-platform C compilation support

- Implement Windows, Linux, macOS compiler detection
- Add platform-specific compilation flags
- Improve error reporting

Fixes #123
```

## Review Process

All pull requests go through:
1. Automated tests (CI/CD)
2. Code review
3. Approval from maintainers
4. Merge to main branch

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- GitHub contributors page
- Release notes (for significant contributions)

---

Thank you for contributing to making this project better! 🚀
