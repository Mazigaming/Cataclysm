# Version 3.2.1 - Project Folder Management & Full Assembly Display

## Release Date
December 2024

## Overview
Version 3.2.1 adds automatic project folder organization and complete assembly preservation, making it easier to manage decompilation sessions and access full disassembly output.

---

## 🎯 New Features

### 1. **Automatic Project Folder Creation**
When decompiling executables from outside the decompiler directory, all output is automatically saved to organized project folders:

```
decompiler/
└── projects/
    ├── notepad/
    │   ├── notepad_full.asm
    │   ├── notepad_decompiled.pseudo
    │   ├── notepad_decompiled.c
    │   ├── notepad_decompiled.rs
    │   ├── notepad_pe_info.txt
    │   └── README.md
    ├── calc/
    │   ├── calc_full.asm
    │   ├── calc_decompiled.pseudo
    │   ├── calc_decompiled.c
    │   ├── calc_decompiled.rs
    │   ├── calc_pe_info.txt
    │   └── README.md
    └── myapp/
        └── ...
```

**Benefits:**
- ✅ No more scattered output files
- ✅ Easy to find previous decompilation sessions
- ✅ All formats saved automatically (no need to choose)
- ✅ Complete PE metadata preserved
- ✅ Automatic navigation to project folder after decompilation

### 2. **Full Assembly Display**
Every decompilation now includes a `{exe_name}_full.asm` file containing:
- **Complete disassembly** of all executable sections
- **All instructions** (not just decompiled functions)
- **Original addresses** preserved
- **Raw assembly** for manual analysis

**Example:**
```asm
0x401000: push    ebp
0x401001: mov     ebp, esp
0x401003: sub     esp, 0x40
0x401006: push    ebx
0x401007: push    esi
0x401008: push    edi
...
```

### 3. **Comprehensive PE Information**
Each project includes a `{exe_name}_pe_info.txt` file with:
- Image base and entry point
- Section details (name, VA, size, characteristics)
- Complete import table (DLL and function names)
- Export table (if present)
- PE header metadata

**Example:**
```
╔════════════════════════════════════════════════════════════════╗
║                    PE FILE INFORMATION                         ║
╚════════════════════════════════════════════════════════════════╝

File: C:\Windows\System32\notepad.exe
Size: 217088 bytes

═══ HEADERS ═══
Image Base: 0x400000
Entry Point: 0x14e0
Subsystem: WindowsGui
Machine: I386

═══ SECTIONS ═══
  .text - VA: 0x1000, Size: 0x1a000, Characteristics: 0x60000020
  .data - VA: 0x1b000, Size: 0x2000, Characteristics: 0xc0000040
  .rdata - VA: 0x1d000, Size: 0x4000, Characteristics: 0x40000040
  ...

═══ IMPORTS ═══
  GetProcAddress (kernel32.dll)
  LoadLibraryA (kernel32.dll)
  MessageBoxA (user32.dll)
  ...

═══ EXPORTS ═══
  (No exports)
```

### 4. **Automatic README Generation**
Each project folder includes a `README.md` with:
- List of generated files and their purposes
- Source executable path
- Decompiler version and features used
- Quick reference for navigating the project

---

## 🔧 Technical Implementation

### New Functions

#### `get_decompiler_root() -> PathBuf`
Determines the root directory of the decompiler installation.

#### `create_project_folder(exe_path: &PathBuf) -> Result<PathBuf, ...>`
Creates a project folder in `decompiler/projects/{exe_name}/`.

#### `should_use_project_folder(exe_path: &PathBuf, current_path: &PathBuf) -> bool`
Determines whether to use project folder (true for external EXEs) or save in-place (false for EXEs inside decompiler directory).

#### `save_complete_decompilation(exe_path: &PathBuf, current_path: &PathBuf, asm: &str) -> Result<PathBuf, ...>`
Main function that:
1. Determines save location (project folder or in-place)
2. Saves full assembly to `{name}_full.asm`
3. Generates and saves all decompiled formats (pseudo, C, Rust)
4. Extracts and saves PE information
5. Creates README.md
6. Returns the project folder path

#### `extract_pe_info(exe_path: &PathBuf) -> String`
Parses PE file and generates comprehensive metadata report.

---

## 🎮 User Experience Changes

### Before v3.2.1:
1. Select EXE file
2. Choose language (Assembly/Pseudo/C/Rust)
3. Choose output mode (Single/Multi-file)
4. View/edit in TUI
5. Press Esc to save
6. File saved next to original EXE

### After v3.2.1:
1. Select EXE file
2. Choose language (still required for UI compatibility)
3. Choose output mode (still required for UI compatibility)
4. **Automatic save** of all formats to project folder
5. **Automatic navigation** to project folder
6. Browse all generated files immediately

**Note:** Language and output mode selections are still shown for UI consistency, but the decompiler now saves ALL formats regardless of selection.

---

## 📁 File Naming Convention

All generated files follow a consistent naming pattern:

| File | Description |
|------|-------------|
| `{name}_full.asm` | Complete disassembly of all executable sections |
| `{name}_decompiled.pseudo` | Pseudo-code representation with control flow |
| `{name}_decompiled.c` | C code decompilation with type definitions |
| `{name}_decompiled.rs` | Rust code decompilation with safety annotations |
| `{name}_pe_info.txt` | PE file metadata and structure information |
| `README.md` | Project documentation and file guide |

---

## 🔄 Backward Compatibility

### In-Place Saving
If you decompile an EXE that's **inside** the decompiler directory, the old behavior is preserved:
- Files are saved next to the original EXE
- No project folder is created
- You can still edit in the TUI before saving

### External EXE Handling
If you decompile an EXE from **outside** the decompiler directory (e.g., `C:\Windows\System32\notepad.exe`):
- Project folder is automatically created
- All formats are saved immediately
- You're navigated to the project folder
- No TUI editing (direct save)

---

## 🚀 Usage Examples

### Example 1: Decompiling System Executable
```
1. Navigate to C:\Windows\System32\
2. Select notepad.exe
3. Choose any language (all will be generated)
4. Choose any output mode (all will be generated)
5. Automatically saved to: decompiler/projects/notepad/
6. File explorer navigates to project folder
7. View any of the 6 generated files
```

### Example 2: Decompiling Local Test File
```
1. Navigate to decompiler/test_files/
2. Select myapp.exe
3. Choose language and output mode
4. Files saved next to myapp.exe (old behavior)
5. Edit in TUI if desired
```

---

## 🎯 Benefits Summary

| Feature | Benefit |
|---------|---------|
| **Project Folders** | Organized workspace, easy to find previous sessions |
| **Full Assembly** | Complete disassembly for manual analysis |
| **All Formats** | No need to re-decompile for different output formats |
| **PE Info** | Comprehensive metadata for reverse engineering |
| **Auto-Navigation** | Immediate access to generated files |
| **README** | Self-documenting projects |

---

## 🔮 Future Enhancements

Potential improvements for v3.3:
- [ ] Timestamp-based project folders for multiple decompilation sessions
- [ ] Project comparison tool (diff between sessions)
- [ ] Export project as ZIP archive
- [ ] Import/export project settings
- [ ] Side-by-side assembly/decompiled view in TUI
- [ ] Search functionality within project files
- [ ] Annotation system for adding notes to decompiled code

---

## 📊 Version Comparison

| Feature | v3.2 | v3.2.1 |
|---------|------|--------|
| PE Parsing | ✅ | ✅ |
| IAT Resolution | ✅ | ✅ |
| Junk Filtering | ✅ | ✅ |
| CFG Recovery | ✅ | ✅ |
| Full Assembly Output | ❌ | ✅ |
| Project Folders | ❌ | ✅ |
| Auto-Save All Formats | ❌ | ✅ |
| PE Info Export | ❌ | ✅ |
| Auto-Navigation | ❌ | ✅ |

---

## 🐛 Known Issues

1. **Unreachable Code Warning**: The main loop never exits normally (by design), causing a compiler warning. This is harmless.

2. **Unused Variables**: Some Mode enum fields are unused in certain contexts. These warnings are cosmetic and don't affect functionality.

3. **Future Feature Warnings**: Structures for Phase 2-6 features (not yet implemented) generate "never used" warnings. These are intentional placeholders.

---

## 🔧 Build Information

- **Build Status**: ✅ Success
- **Warnings**: 30 (all expected, none critical)
- **Build Time**: ~10 seconds (release mode)
- **Binary Size**: ~2.5 MB (optimized)
- **Target**: Windows x86_64

---

## 📝 Migration Guide

### For Users
No migration needed! The new features work automatically:
- External EXEs → Project folders (new behavior)
- Internal EXEs → In-place saving (old behavior)

### For Developers
If you're integrating the decompiler as a library:

**Old API (still works):**
```rust
let pseudo = decompiler::translate_to_pseudo(&asm);
let c_code = decompiler::translate_to_c(&asm);
let rust_code = decompiler::translate_to_rust(&asm);
```

**New API (with PE parsing):**
```rust
let pseudo = decompiler::translate_to_pseudo_with_pe(&asm, Some("path/to/file.exe"));
let c_code = decompiler::translate_to_c_with_pe(&asm, Some("path/to/file.exe"));
let rust_code = decompiler::translate_to_rust_with_pe(&asm, Some("path/to/file.exe"));
```

---

## 🙏 Acknowledgments

This release builds upon the solid foundation of v3.2, which introduced:
- PE parsing with goblin crate
- IAT resolution
- Junk instruction filtering
- Enhanced CFG reconstruction

Version 3.2.1 focuses on improving the user experience and making decompilation output more accessible and organized.

---

## 📞 Support

For issues, questions, or feature requests:
- Check the README.md in each project folder
- Review the PE info file for metadata
- Compare the full assembly with decompiled output
- Refer to ROADMAP_V3.2_TO_V4.0.md for planned features

---

**Version**: 3.2.1  
**Release**: December 2024  
**Status**: Stable  
**Next Version**: 3.3 (Planned - Enhanced UI and project management)