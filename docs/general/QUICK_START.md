# 🚀 Quick Start Guide - Advanced Decompiler

## Getting Started in 60 Seconds

### 1. Build the Project
```powershell
cd c:\Users\kacpe\Documents\decompiler\rust_file_explorer
cargo build --release
```

### 2. Run the File Explorer
```powershell
cargo run --release
```

### 3. Navigate and Decompile
- Use **↑/↓** arrow keys to navigate
- Press **Enter** on any PE file (`.exe`, `.dll`, `.sys`, `.ocx`, `.cpl`, `.scr`, `.drv`, `.efi`)
- Choose decompilation format:
  - **Assembly** - Raw disassembly
  - **Pseudo Code** - High-level pseudo-code  
  - **C Code** - Compilable C source
  - **Rust Code** ✨ NEW! - Memory-safe Rust source

### 4. View and Edit
- View the decompiled code
- Press **Ctrl+S** to save
- Press **Esc** to go back

---

## 🎯 Example Workflow

### Decompiling a Simple Program

1. **Find an executable or DLL**
   ```
   Navigate to: C:\Windows\System32\notepad.exe
   Or try: C:\Windows\System32\kernel32.dll
   ```

2. **Select decompilation mode**
   ```
   Press Enter → Choose "Rust Code" or "C Code"
   ```

3. **View the output**
   ```c
   /*
    * ═══════════════════════════════════════════════════════════════
    * ADVANCED DECOMPILER OUTPUT
    * ═══════════════════════════════════════════════════════════════
    * Functions detected: 15
    * API calls detected: 8
    * Features: Control Flow Recovery, Type Inference, Pattern Recognition
    * ═══════════════════════════════════════════════════════════════
    */
   
   #include <stdio.h>
   #include <windows.h>
   
   void func_401000() {
       i32 local_16;
       local_16 = 0;
       
       while (local_16 < 10) {
           local_16 += 1;
           MessageBoxA();  // Display message box (ANSI)
       }
       return;
   }
   ```

4. **Save the result**
   ```
   Press Ctrl+S → Saves as notepad.c
   ```

---

## 🎨 Output Examples

### Assembly Output
```asm
0x401000: push    ebp
0x401001: mov     ebp, esp
0x401003: sub     esp, 0x10
0x401006: mov     dword ptr [ebp - 4], 0
0x40100d: jmp     0x401018
```

### Pseudo-Code Output
```
┌─ Function: func_401000 (0x401000) ─┐
│
│ Variables:
│   local local_4 : Int32
│
│ Code:
│ local_4 = 0
│ while (true) {
│   compare(local_4, 10)
│   if (greater_or_equal) {
│     goto 0x401025
│   }
│   local_4 = local_4 + 1
│ }
│ return
│
└────────────────────────────────┘
```

### C Code Output
```c
// ═══════════════════════════════════════════════════════════════
// Function: func_401000 (Address: 0x401000)
// ═══════════════════════════════════════════════════════════════
void func_401000() {
    // Local variables
    i32 local_4;
    
    local_4 = 0;
    while (local_4 < 10) {
        local_4 += 1;
    }
    return;
}
```

### Rust Code Output ✨ NEW!
```rust
// ═══════════════════════════════════════════════════════════════
// Function: func_401000 (Address: 0x401000)
// ═══════════════════════════════════════════════════════════════
unsafe fn func_401000() {
    // Local variables
    let mut local_4: I32;
    
    local_4 = 0;
    while local_4 < 10 {
        local_4 += 1;
    }
    return;
}
```

---

## ⌨️ Keyboard Shortcuts

### File Explorer Mode
| Key | Action |
|-----|--------|
| ↑/↓ | Navigate files |
| Enter | Open file/directory |
| Esc/Q | Quit application |

### Language Selection Mode
| Key | Action |
|-----|--------|
| ↑/↓ | Select option |
| Enter | Confirm selection |
| Esc | Go back |

### Editor Mode
| Key | Action |
|-----|--------|
| Ctrl+S | Save file |
| Esc | Save and exit |
| Arrow Keys | Navigate text |
| Type | Edit content |

---

## 🎯 Best Practices

### For Best Results

1. **Start Small**: Test with simple executables first
2. **Compare Outputs**: Check all four formats (Assembly, Pseudo, C, Rust)
3. **Verify Logic**: Cross-reference with known behavior
4. **Save Work**: Always save important decompilations
5. **Iterate**: Review and refine understanding
6. **Try DLLs**: Analyze system DLLs to understand Windows APIs

### Recommended Test Files

Good files to practice with:
- ✅ Simple console apps
- ✅ Calculator programs
- ✅ Hello World executables
- ✅ Small utilities
- ✅ System DLLs (kernel32.dll, user32.dll) ✨ NEW!
- ✅ Third-party DLLs ✨ NEW!
- ✅ Device drivers (.sys files) ✨ NEW!

Avoid initially:
- ❌ Large system files
- ❌ Heavily obfuscated code
- ❌ Packed executables
- ❌ .NET/Java bytecode

---

## 🔍 Understanding the Output

### Variable Names
- `local_X` - Local variable at stack offset X
- `param_X` - Function parameter at offset X
- `eax`, `ebx`, etc. - CPU registers

### Types
- `i32` - 32-bit signed integer
- `i64` - 64-bit signed integer
- `u32` - 32-bit unsigned integer
- `void*` - Pointer type

### Control Flow
- `while (condition)` - Loop structure
- `if (condition)` - Conditional branch
- `goto label` - Unconditional jump
- `return` - Function exit

---

## 🐛 Troubleshooting

### Issue: "Failed to parse PE"
**Solution**: File is not a valid Windows PE file
- Check file type
- Ensure it's a PE file (.exe, .dll, .sys, etc.)
- Try a different file

### Issue: "Error disassembling"
**Solution**: Executable section not found
- File may be packed
- Try unpacking first
- Use a different executable

### Issue: Output looks garbled
**Solution**: Complex or obfuscated code
- Try Pseudo-Code format first
- Review Assembly output
- May need manual analysis

### Issue: No functions detected
**Solution**: Non-standard calling convention
- Code still decompiles
- Treated as single function
- Check Assembly output

---

## 📊 Feature Checklist

What the decompiler can handle:

✅ **Standard x86/x64 executables**
✅ **DLL files** ✨ NEW!
✅ **System drivers (.sys)** ✨ NEW!
✅ **ActiveX controls (.ocx)** ✨ NEW!
✅ **Control Panel items (.cpl)** ✨ NEW!
✅ **Screen savers (.scr)** ✨ NEW!
✅ **Device drivers (.drv)** ✨ NEW!
✅ **EFI applications (.efi)** ✨ NEW!
✅ **MSVC compiled programs**
✅ **GCC compiled programs**
✅ **Windows API calls**
✅ **C Runtime functions**
✅ **Loops and conditionals**
✅ **Function calls**
✅ **Stack variables**
✅ **Arithmetic operations**
✅ **Bitwise operations**
✅ **Memory operations**
✅ **Rust code generation** ✨ NEW!

---

## 🎓 Learning Path

### Beginner
1. Decompile simple "Hello World" programs
2. Compare with original source code
3. Understand basic patterns

### Intermediate
1. Analyze programs with loops
2. Study conditional logic
3. Identify function calls

### Advanced
1. Reverse engineer complex algorithms
2. Analyze API usage patterns
3. Reconstruct data structures

---

## 💡 Pro Tips

### Tip 1: Compare Formats
Always check Pseudo-Code, C, and Rust output. They complement each other and provide different perspectives.

### Tip 2: Look for Patterns
Recognize common patterns:
- `xor eax, eax` = zero initialization
- `push ebp; mov ebp, esp` = function start
- `mov esp, ebp; pop ebp; ret` = function end

### Tip 3: API Calls are Key
API calls reveal program behavior:
- `CreateFile` = file operations
- `VirtualAlloc` = memory allocation
- `MessageBox` = user interaction

### Tip 4: Follow the Data
Track how data flows through variables to understand logic.

### Tip 5: Use Comments
The decompiler adds helpful comments - read them!

### Tip 6: Analyze DLLs ✨ NEW!
Decompile system DLLs to understand Windows internals:
- `kernel32.dll` = Core Windows functions
- `user32.dll` = UI and window management
- `ntdll.dll` = Native API layer

### Tip 7: Try Rust Output ✨ NEW!
Use Rust code generation for:
- Modern, type-safe analysis
- Memory-safe reverse engineering
- Integration with Rust projects

---

## 🚀 Advanced Usage

### Batch Processing
Create a script to decompile multiple files:
```powershell
# Example: Decompile all .exe files in a directory
Get-ChildItem *.exe | ForEach-Object {
    # Your decompilation logic here
}
```

### Integration
Use the decompiler module in your own Rust projects:
```rust
use decompiler::{translate_to_c, translate_to_pseudo, translate_to_rust};

let asm = read_disassembly("program.exe");
let c_code = translate_to_c(&asm);
let rust_code = translate_to_rust(&asm);
save_to_file("output.c", &c_code);
save_to_file("output.rs", &rust_code);
```

---

## 📚 Additional Resources

### Documentation
- `DECOMPILER_FEATURES.md` - Complete feature guide
- `UPGRADE_SUMMARY.md` - What's new and improved
- `QUICK_START.md` - This guide
- `RUST_DLL_SUPPORT.md` - Rust & DLL features ✨ NEW!
- `CHANGELOG.md` - Version history ✨ NEW!
- `VERSION_2.0_SUMMARY.md` - Complete v2.0 summary ✨ NEW!

### Code Structure
```
src/
├── main.rs          - File explorer UI
├── decompiler.rs    - Decompilation engine
└── ...
```

### Key Functions
- `translate_to_pseudo()` - Generate pseudo-code
- `translate_to_c()` - Generate C code
- `translate_to_rust()` - Generate Rust code ✨ NEW!
- `parse_instructions()` - Parse assembly
- `identify_functions()` - Find functions
- `analyze_control_flow()` - Recover structure

---

## 🎯 Success Metrics

You're successfully using the decompiler when you can:

✅ Navigate the file explorer smoothly
✅ Decompile executables and DLLs in all four formats
✅ Understand the generated pseudo-code
✅ Recognize common patterns
✅ Identify API calls and their purposes
✅ Save and review decompiled code
✅ Compare output with expected behavior
✅ Generate compilable Rust code ✨ NEW!
✅ Analyze system DLLs ✨ NEW!

---

## 🏆 Next Steps

1. **Practice**: Decompile various programs
2. **Learn**: Study the output patterns
3. **Experiment**: Try different executables
4. **Contribute**: Enhance the decompiler
5. **Share**: Help others learn reverse engineering

---

## 📞 Getting Help

### If You're Stuck

1. **Check Documentation**: Read the feature guide
2. **Review Examples**: Look at sample outputs
3. **Start Simple**: Use basic test programs
4. **Debug**: Check the Assembly output first
5. **Iterate**: Try different approaches

### Common Questions

**Q: Can it decompile .NET executables?**
A: No, this is for native x86/x64 code only.

**Q: Will it recover original variable names?**
A: No, but it creates logical names (local_X, param_X).

**Q: Can I compile the C output?**
A: Yes! The C code is designed to be compilable.

**Q: Can I compile the Rust output?** ✨ NEW!
A: Yes! Add winapi to Cargo.toml and it should compile.

**Q: Can it decompile DLLs?** ✨ NEW!
A: Yes! All PE formats are supported (.dll, .sys, .ocx, etc.).

**Q: How accurate is the decompilation?**
A: Very accurate for standard code. Complex optimizations may need review.

---

## 🎉 You're Ready!

You now have everything you need to start decompiling programs like a pro!

**Key Takeaways:**
- 🚀 Fast and powerful decompilation
- 🎨 Beautiful, readable output
- 🔍 Professional-grade analysis
- 💡 Easy to use interface
- 📚 Comprehensive documentation
- 🦀 Rust code generation ✨ NEW!
- 📚 DLL & system file support ✨ NEW!

**Happy Reverse Engineering! 🔍🦀**

---

*Last Updated: 2024*
*Version: 2.0 - Rust & DLL Support Edition*