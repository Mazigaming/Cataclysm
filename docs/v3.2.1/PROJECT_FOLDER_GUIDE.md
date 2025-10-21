# Project Folder System - Visual Guide

## 📂 Directory Structure

```
C:\Users\kacpe\Documents\decompiler\
│
├── rust_file_explorer\          # Main application
│   ├── src\
│   ├── target\
│   └── Cargo.toml
│
└── projects\                     # 🆕 Auto-generated project folders
    │
    ├── notepad\                  # Project for notepad.exe
    │   ├── notepad_full.asm      # Complete disassembly
    │   ├── notepad_decompiled.pseudo
    │   ├── notepad_decompiled.c
    │   ├── notepad_decompiled.rs
    │   ├── notepad_pe_info.txt
    │   └── README.md
    │
    ├── calc\                     # Project for calc.exe
    │   ├── calc_full.asm
    │   ├── calc_decompiled.pseudo
    │   ├── calc_decompiled.c
    │   ├── calc_decompiled.rs
    │   ├── calc_pe_info.txt
    │   └── README.md
    │
    └── myapp\                    # Project for myapp.exe
        ├── myapp_full.asm
        ├── myapp_decompiled.pseudo
        ├── myapp_decompiled.c
        ├── myapp_decompiled.rs
        ├── myapp_pe_info.txt
        └── README.md
```

---

## 🎬 Workflow Visualization

### Scenario 1: External EXE (e.g., C:\Windows\System32\notepad.exe)

```
┌─────────────────────────────────────────────────────────────┐
│  Step 1: Navigate to C:\Windows\System32\                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ File Explorer - C:\Windows\System32\                  │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │ Files                                                  │  │
│  │ > <DIR> ..                                             │  │
│  │   calc.exe (217088 bytes)                             │  │
│  │ ► notepad.exe (217088 bytes)  ← SELECT THIS          │  │
│  │   cmd.exe (289792 bytes)                              │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 2: Choose Language                                     │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Select Language for notepad.exe                       │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │ Options                                                │  │
│  │ ► Assembly                                             │  │
│  │   Pseudo Code                                          │  │
│  │   C Code                                               │  │
│  │   Rust Code                                            │  │
│  └───────────────────────────────────────────────────────┘  │
│  Note: All formats will be generated regardless            │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 3: Choose Output Mode                                 │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Select Output Mode - notepad.exe                      │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │ Output Mode                                            │  │
│  │ ► Single File                                          │  │
│  │   Multi-File (by type)                                 │  │
│  │   Multi-File (by function)                             │  │
│  └───────────────────────────────────────────────────────┘  │
│  Note: All formats will be generated regardless            │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 4: Automatic Processing                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ ⚙️  Disassembling notepad.exe...                       │  │
│  │ ⚙️  Parsing PE headers...                              │  │
│  │ ⚙️  Filtering junk instructions...                     │  │
│  │ ⚙️  Generating pseudo-code...                          │  │
│  │ ⚙️  Generating C code...                               │  │
│  │ ⚙️  Generating Rust code...                            │  │
│  │ ⚙️  Extracting PE information...                       │  │
│  │ ⚙️  Creating project folder...                         │  │
│  │ ✅ Saved to: decompiler/projects/notepad/             │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 5: Auto-Navigate to Project Folder                    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ File Explorer - decompiler/projects/notepad/          │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │ Files                                                  │  │
│  │ > <DIR> ..                                             │  │
│  │   notepad_full.asm (1.2 MB)                           │  │
│  │   notepad_decompiled.pseudo (450 KB)                  │  │
│  │   notepad_decompiled.c (520 KB)                       │  │
│  │   notepad_decompiled.rs (580 KB)                      │  │
│  │   notepad_pe_info.txt (12 KB)                         │  │
│  │   README.md (2 KB)                                     │  │
│  └───────────────────────────────────────────────────────┘  │
│  ✅ All files ready for viewing!                            │
└─────────────────────────────────────────────────────────────┘
```

---

### Scenario 2: Internal EXE (e.g., decompiler/test_files/myapp.exe)

```
┌─────────────────────────────────────────────────────────────┐
│  Step 1: Navigate to decompiler/test_files/                 │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ File Explorer - decompiler/test_files/                │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │ Files                                                  │  │
│  │ > <DIR> ..                                             │  │
│  │ ► myapp.exe (45056 bytes)  ← SELECT THIS             │  │
│  │   test.exe (32768 bytes)                              │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 2-3: Choose Language & Output Mode (same as above)    │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 4: In-Place Saving (Old Behavior)                     │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ ⚙️  Disassembling myapp.exe...                         │  │
│  │ ⚙️  Generating all formats...                          │  │
│  │ ✅ Saved to: decompiler/test_files/                   │  │
│  └───────────────────────────────────────────────────────┘  │
│  Note: Files saved next to original EXE                     │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 5: Stay in Same Directory                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ File Explorer - decompiler/test_files/                │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │ Files                                                  │  │
│  │ > <DIR> ..                                             │  │
│  │   myapp.exe (45056 bytes)                             │  │
│  │   myapp_full.asm (120 KB)                             │  │
│  │   myapp_decompiled.pseudo (45 KB)                     │  │
│  │   myapp_decompiled.c (52 KB)                          │  │
│  │   myapp_decompiled.rs (58 KB)                         │  │
│  │   myapp_pe_info.txt (8 KB)                            │  │
│  │   README.md (2 KB)                                     │  │
│  │   test.exe (32768 bytes)                              │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 📄 File Contents Preview

### 1. `{name}_full.asm` - Complete Disassembly
```asm
; Complete disassembly of notepad.exe
; Generated by Advanced Decompiler v3.2.1

.text section (0x401000 - 0x41a000):
0x401000: push    ebp
0x401001: mov     ebp, esp
0x401003: sub     esp, 0x40
0x401006: push    ebx
0x401007: push    esi
0x401008: push    edi
0x401009: mov     edi, dword ptr [ebp + 8]
0x40100c: test    edi, edi
0x40100e: je      0x401050
...
[12,000+ lines of assembly]
```

### 2. `{name}_decompiled.pseudo` - Pseudo-Code
```
╔════════════════════════════════════════════════════════════════╗
║          ADVANCED PSEUDO-CODE DECOMPILATION v3.2               ║
╚════════════════════════════════════════════════════════════════╝

// Image Base: 0x400000
// Entry Point: 0x14e0
// Sections: 5
// Imports: 234
// Exports: 0

function sub_401000(arg1, arg2) {
    var1 = arg1
    var2 = arg2
    if (var1 == 0) {
        return 0
    }
    ...
}
```

### 3. `{name}_decompiled.c` - C Code
```c
/*
 * ═══════════════════════════════════════════════════════════════
 * ADVANCED DECOMPILER OUTPUT v3.2
 * ═══════════════════════════════════════════════════════════════
 * Functions detected: 45
 * API calls detected: 12
 * Image Base: 0x400000
 * Entry Point: 0x14e0
 * Imports: 234
 * Exports: 0
 * Features: Control Flow Recovery, Type Inference, Pattern Recognition
 * Features: PE Parsing, IAT Resolution, Junk Filtering
 * ═══════════════════════════════════════════════════════════════
 */

#include <stdio.h>
#include <stdlib.h>
#include <windows.h>

void sub_401000() {
    u32 var1 = 0;
    u32 var2 = 0;
    ...
}
```

### 4. `{name}_decompiled.rs` - Rust Code
```rust
//! ═══════════════════════════════════════════════════════════════
//! ADVANCED DECOMPILER OUTPUT v3.2 - RUST EDITION
//! ═══════════════════════════════════════════════════════════════
//! Functions detected: 45
//! API calls detected: 12
//! Image Base: 0x400000
//! Entry Point: 0x14e0
//! Imports: 234
//! Exports: 0
//! Features: Control Flow Recovery, Type Inference, Pattern Recognition
//! Features: PE Parsing, IAT Resolution, Junk Filtering
//! ═══════════════════════════════════════════════════════════════

#![allow(unused_variables, unused_mut, dead_code)]

unsafe fn sub_401000() {
    let mut var1: U32 = 0;
    let mut var2: U32 = 0;
    ...
}
```

### 5. `{name}_pe_info.txt` - PE Metadata
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
  .rsrc - VA: 0x21000, Size: 0x3000, Characteristics: 0x40000040
  .reloc - VA: 0x24000, Size: 0x1000, Characteristics: 0x42000040

═══ IMPORTS ═══
  GetProcAddress (kernel32.dll)
  LoadLibraryA (kernel32.dll)
  GetModuleHandleA (kernel32.dll)
  ExitProcess (kernel32.dll)
  MessageBoxA (user32.dll)
  CreateWindowExA (user32.dll)
  ...
  [234 total imports]

═══ EXPORTS ═══
  (No exports)
```

### 6. `README.md` - Project Documentation
```markdown
# Decompilation Project: notepad

## Files Generated:
- `notepad_full.asm` - Complete disassembly of all executable sections
- `notepad_decompiled.pseudo` - Pseudo-code representation
- `notepad_decompiled.c` - C code decompilation
- `notepad_decompiled.rs` - Rust code decompilation
- `notepad_pe_info.txt` - PE file metadata and structure

## Source:
Original file: C:\Windows\System32\notepad.exe

## Decompiler Version:
Advanced Decompiler v3.2
Features: PE Parsing, IAT Resolution, Junk Filtering, CFG Recovery
```

---

## 🎯 Quick Reference

### When is a Project Folder Created?

| Scenario | Project Folder? | Location |
|----------|----------------|----------|
| EXE from `C:\Windows\System32\` | ✅ Yes | `decompiler/projects/{name}/` |
| EXE from `C:\Program Files\` | ✅ Yes | `decompiler/projects/{name}/` |
| EXE from Desktop | ✅ Yes | `decompiler/projects/{name}/` |
| EXE from `decompiler/test_files/` | ❌ No | `decompiler/test_files/` (in-place) |
| EXE from `decompiler/samples/` | ❌ No | `decompiler/samples/` (in-place) |

**Rule:** If the EXE is **outside** the decompiler directory, a project folder is created.

---

## 🔍 Finding Your Projects

### Method 1: File Explorer (in TUI)
1. Launch the decompiler
2. Navigate to the decompiler root directory
3. Enter the `projects` folder
4. Browse project folders by name

### Method 2: Windows Explorer
1. Open Windows Explorer
2. Navigate to: `C:\Users\kacpe\Documents\decompiler\projects\`
3. Browse folders

### Method 3: Command Line
```powershell
cd C:\Users\kacpe\Documents\decompiler\projects
dir
```

---

## 💡 Tips & Tricks

### Tip 1: Quick Access
Bookmark the `projects` folder in Windows Explorer for quick access to all decompilation sessions.

### Tip 2: Compare Formats
Open multiple files side-by-side to compare assembly with decompiled code:
- Left: `{name}_full.asm`
- Right: `{name}_decompiled.c`

### Tip 3: Search Across Files
Use Windows Search or `grep` to find specific functions across all decompiled files:
```powershell
cd C:\Users\kacpe\Documents\decompiler\projects\notepad
Select-String -Pattern "MessageBox" -Path *.c, *.rs, *.pseudo
```

### Tip 4: Archive Projects
Compress project folders to save space:
```powershell
Compress-Archive -Path "projects\notepad" -DestinationPath "archives\notepad_2024-12.zip"
```

### Tip 5: Diff Projects
Compare two decompilation sessions of the same executable:
```powershell
fc projects\notepad_v1\notepad_decompiled.c projects\notepad_v2\notepad_decompiled.c
```

---

## 🚀 Advanced Usage

### Batch Decompilation
Decompile multiple executables and organize them automatically:
1. Copy all EXEs to a temporary folder outside decompiler
2. Decompile each one through the TUI
3. All projects are automatically organized in `projects/`

### Project Cleanup
Remove old projects you no longer need:
```powershell
Remove-Item -Recurse -Force "projects\old_project"
```

### Export for Sharing
Share a project with colleagues:
```powershell
Compress-Archive -Path "projects\myapp" -DestinationPath "myapp_decompiled.zip"
```

---

## 📊 Storage Estimates

Typical project folder sizes:

| Executable Size | Project Folder Size | Breakdown |
|----------------|---------------------|-----------|
| 50 KB | ~500 KB | Assembly: 200 KB, Decompiled: 250 KB, PE info: 10 KB |
| 500 KB | ~5 MB | Assembly: 2 MB, Decompiled: 2.5 MB, PE info: 50 KB |
| 5 MB | ~50 MB | Assembly: 20 MB, Decompiled: 25 MB, PE info: 200 KB |

**Note:** Decompiled output is typically 10x the size of the original executable due to verbose formatting and comments.

---

## 🔧 Troubleshooting

### Issue: Project folder not created
**Cause:** EXE is inside decompiler directory  
**Solution:** Move EXE to external location (e.g., Desktop) and try again

### Issue: Files not appearing in project folder
**Cause:** Decompilation failed or was interrupted  
**Solution:** Check for error messages, try decompiling again

### Issue: Can't find projects folder
**Cause:** Decompiler root detection failed  
**Solution:** Manually create `decompiler/projects/` folder

### Issue: Permission denied when saving
**Cause:** Insufficient write permissions  
**Solution:** Run decompiler as administrator or change save location

---

## 📞 Support

For issues or questions:
1. Check the README.md in each project folder
2. Review the PE info file for metadata
3. Compare full assembly with decompiled output
4. Refer to VERSION_3.2.1_CHANGELOG.md for feature details

---

**Happy Decompiling! 🎉**