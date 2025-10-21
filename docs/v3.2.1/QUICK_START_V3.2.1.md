# Quick Start Guide - Version 3.2.1

## 🚀 Getting Started in 60 Seconds

### Step 1: Build the Decompiler (if not already built)
```powershell
cd c:\Users\kacpe\Documents\decompiler\rust_file_explorer
cargo build --release
```

### Step 2: Run the Decompiler
```powershell
.\target\release\rust_file_explorer.exe
```

### Step 3: Decompile Your First EXE
1. **Navigate** to any folder with an EXE (e.g., `C:\Windows\System32\`)
2. **Select** an EXE file (e.g., `notepad.exe`)
3. **Choose** any language (all formats will be generated)
4. **Choose** any output mode (all formats will be generated)
5. **Wait** for automatic processing (~5 seconds)
6. **Browse** the generated project folder!

---

## 📂 What You'll Get

After decompiling `notepad.exe`, you'll find:

```
c:\Users\kacpe\Documents\decompiler\projects\notepad\
├── notepad_full.asm          ← Complete disassembly
├── notepad_decompiled.pseudo ← Pseudo-code
├── notepad_decompiled.c      ← C code
├── notepad_decompiled.rs     ← Rust code
├── notepad_pe_info.txt       ← PE metadata
└── README.md                 ← Project info
```

---

## 🎯 Common Use Cases

### Use Case 1: Analyze System Executable
```
Goal: Understand how notepad.exe works

1. Navigate to C:\Windows\System32\
2. Select notepad.exe
3. Choose "C Code" (or any language)
4. Choose "Single File" (or any mode)
5. Open notepad_decompiled.c in the project folder
6. Read the decompiled C code
7. Cross-reference with notepad_full.asm for details
8. Check notepad_pe_info.txt for imports/exports
```

### Use Case 2: Reverse Engineer Custom Application
```
Goal: Decompile a custom application

1. Copy myapp.exe to Desktop
2. Navigate to Desktop in the decompiler
3. Select myapp.exe
4. Choose "Rust Code"
5. Choose "Multi-File (by function)"
6. Browse the project folder
7. Open myapp_decompiled.rs
8. Analyze the Rust code
```

### Use Case 3: Compare Multiple Versions
```
Goal: Compare two versions of the same executable

1. Decompile version1.exe → projects/version1/
2. Decompile version2.exe → projects/version2/
3. Use diff tool to compare:
   fc projects\version1\version1_decompiled.c projects\version2\version2_decompiled.c
4. Identify changes between versions
```

### Use Case 4: Extract PE Metadata
```
Goal: Get detailed PE information

1. Decompile any EXE
2. Open {name}_pe_info.txt
3. Review:
   - Image base and entry point
   - Section layout
   - Import table (DLLs and functions)
   - Export table
   - PE headers
```

---

## ⌨️ Keyboard Shortcuts

### In File List Mode
- `↑/↓` - Navigate files
- `Enter` - Select file/folder
- `Esc` or `q` - Exit application

### In Language Selection Mode
- `↑/↓` - Navigate options
- `Enter` - Confirm selection
- `Esc` - Go back

### In Output Mode Selection Mode
- `↑/↓` - Navigate options
- `Enter` - Start decompilation
- `Esc` - Go back

### In Edit Mode (if fallback occurs)
- `Ctrl+S` - Save file
- `Esc` - Save and exit
- Normal text editing keys

---

## 🔍 Finding Your Projects

### Method 1: Auto-Navigation (Recommended)
After decompilation, the file explorer automatically navigates to the project folder. Just browse the files!

### Method 2: Manual Navigation
1. In the decompiler, navigate to the decompiler root
2. Enter the `projects` folder
3. Enter the project folder (named after the EXE)
4. Browse the 6 generated files

### Method 3: Windows Explorer
```
1. Open Windows Explorer
2. Navigate to: c:\Users\kacpe\Documents\decompiler\projects\
3. Open the project folder
4. Double-click any file to view
```

---

## 📖 Reading the Output

### 1. Full Assembly (`{name}_full.asm`)
**Best for:** Low-level analysis, understanding exact instructions

**Example:**
```asm
0x401000: push    ebp
0x401001: mov     ebp, esp
0x401003: sub     esp, 0x40
```

**When to use:**
- Need exact instruction sequence
- Debugging specific behavior
- Understanding compiler optimizations
- Analyzing obfuscated code

### 2. Pseudo-Code (`{name}_decompiled.pseudo`)
**Best for:** Quick understanding, high-level overview

**Example:**
```
function sub_401000(arg1, arg2) {
    var1 = arg1
    if (var1 == 0) {
        return 0
    }
}
```

**When to use:**
- First pass analysis
- Understanding control flow
- Quick function overview
- Teaching/documentation

### 3. C Code (`{name}_decompiled.c`)
**Best for:** Recompilation, detailed analysis

**Example:**
```c
void sub_401000() {
    u32 var1 = 0;
    if (var1 == 0) {
        return;
    }
}
```

**When to use:**
- Recompiling the code
- Detailed type analysis
- Integration with C projects
- Professional reverse engineering

### 4. Rust Code (`{name}_decompiled.rs`)
**Best for:** Safe recompilation, modern analysis

**Example:**
```rust
unsafe fn sub_401000() {
    let mut var1: U32 = 0;
    if var1 == 0 {
        return;
    }
}
```

**When to use:**
- Recompiling with safety checks
- Modern Rust projects
- Memory safety analysis
- Learning Rust from assembly

### 5. PE Info (`{name}_pe_info.txt`)
**Best for:** Understanding file structure, imports/exports

**Example:**
```
Image Base: 0x400000
Entry Point: 0x14e0
Imports: GetProcAddress (kernel32.dll)
```

**When to use:**
- Understanding dependencies
- Finding entry points
- Analyzing imports/exports
- PE structure analysis

### 6. README (`README.md`)
**Best for:** Project overview, quick reference

**When to use:**
- First time opening project
- Remembering what was decompiled
- Sharing project with others

---

## 💡 Pro Tips

### Tip 1: Start with PE Info
Always read `{name}_pe_info.txt` first to understand:
- What DLLs are imported
- What functions are called
- Entry point location
- Section layout

### Tip 2: Use Multiple Formats
Don't rely on just one format:
- Start with **pseudo-code** for overview
- Use **C code** for detailed analysis
- Reference **full assembly** for exact behavior
- Check **PE info** for context

### Tip 3: Search for Specific Functions
Use Windows Search or grep to find functions:
```powershell
cd projects\notepad
Select-String -Pattern "MessageBox" -Path *.c, *.rs, *.pseudo, *.asm
```

### Tip 4: Compare with Original
Keep the original EXE for reference:
- Run it in a debugger
- Compare behavior with decompiled code
- Verify assumptions

### Tip 5: Annotate Your Findings
Copy the decompiled code to a new file and add comments:
```c
// This function initializes the main window
void sub_401000() {
    // var1 stores the window handle
    u32 var1 = 0;
    ...
}
```

### Tip 6: Use Version Control
Track your analysis progress:
```powershell
cd projects\myapp
git init
git add .
git commit -m "Initial decompilation"
# Make annotations
git commit -am "Added function comments"
```

### Tip 7: Archive Completed Projects
Compress projects you're done with:
```powershell
Compress-Archive -Path "projects\notepad" -DestinationPath "archives\notepad_analyzed.zip"
Remove-Item -Recurse "projects\notepad"
```

---

## 🎓 Learning Path

### Beginner: Start Here
1. Decompile a simple EXE (e.g., `calc.exe`)
2. Read the `README.md` in the project folder
3. Open `{name}_decompiled.pseudo` for overview
4. Compare with `{name}_full.asm` to see the difference
5. Check `{name}_pe_info.txt` to understand imports

### Intermediate: Go Deeper
1. Decompile a more complex application
2. Read `{name}_decompiled.c` for detailed analysis
3. Cross-reference with `{name}_full.asm`
4. Identify functions by their behavior
5. Map imports to function calls

### Advanced: Master It
1. Decompile obfuscated or packed executables
2. Use all formats simultaneously
3. Annotate the decompiled code
4. Recompile and test
5. Compare behavior with original

---

## 🐛 Troubleshooting

### Problem: No project folder created
**Solution:** The EXE is inside the decompiler directory. Move it to an external location (e.g., Desktop).

### Problem: Files are incomplete
**Solution:** The decompilation may have failed. Check the console for errors and try again.

### Problem: Can't find projects folder
**Solution:** Navigate to `c:\Users\kacpe\Documents\decompiler\projects\` manually.

### Problem: Decompiled code looks wrong
**Solution:** This is normal for complex executables. Use multiple formats and cross-reference with assembly.

### Problem: Application freezes during decompilation
**Solution:** Large executables take time. Wait 30-60 seconds. If still frozen, restart and try a smaller EXE.

---

## 📊 What to Expect

### Typical Decompilation Times
| EXE Size | Time | Project Size |
|----------|------|--------------|
| 50 KB | 1-2 sec | ~500 KB |
| 500 KB | 3-5 sec | ~5 MB |
| 5 MB | 10-20 sec | ~50 MB |
| 50 MB | 60-120 sec | ~500 MB |

### Output Quality
- **Pseudo-code:** High-level, easy to read, may miss details
- **C code:** Detailed, compilable (with fixes), good for analysis
- **Rust code:** Safe, modern, requires unsafe blocks
- **Assembly:** Exact, complete, requires expertise

### Limitations
- Variable names are generic (`var1`, `var2`, etc.)
- Function names are addresses (`sub_401000`, etc.)
- Type inference is basic (mostly `u32`, `u64`)
- Control flow may use gotos instead of loops
- Some optimizations may be hard to understand

---

## 🎯 Next Steps

### After Your First Decompilation
1. ✅ Read the generated README.md
2. ✅ Browse all 6 files
3. ✅ Compare different formats
4. ✅ Try decompiling another EXE

### To Learn More
1. 📖 Read `PROJECT_FOLDER_GUIDE.md` for detailed workflows
2. 📖 Read `VERSION_3.2.1_CHANGELOG.md` for feature details
3. 📖 Read `ROADMAP_V3.2_TO_V4.0.md` for future plans

### To Contribute
1. 🔧 Report issues or bugs
2. 💡 Suggest new features
3. 🚀 Submit pull requests
4. 📝 Improve documentation

---

## 🎉 You're Ready!

You now know how to:
- ✅ Decompile any Windows executable
- ✅ Navigate project folders
- ✅ Read different output formats
- ✅ Use the decompiler effectively

**Happy Decompiling! 🚀**

---

## 📞 Quick Reference

| Task | Command/Action |
|------|----------------|
| Run decompiler | `.\target\release\rust_file_explorer.exe` |
| Navigate files | `↑/↓` arrows |
| Select file | `Enter` |
| Go back | `Esc` |
| Exit | `q` or `Esc` |
| Find projects | `c:\Users\kacpe\Documents\decompiler\projects\` |
| Search in project | `Select-String -Pattern "text" -Path *.c` |
| Archive project | `Compress-Archive -Path "projects\name" -Destination "name.zip"` |

---

**Version:** 3.2.1  
**Last Updated:** December 2024  
**Status:** Ready to Use ✅