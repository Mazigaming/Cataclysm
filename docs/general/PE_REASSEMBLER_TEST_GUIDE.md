# PE Reassembler Test Guide

## Test Executable Created

✅ **Location:** `c:\Users\kacpe\Documents\decompiler\test_programs\hello_test.exe`

✅ **Source Code:**
```c
#include <stdio.h>

int main() {
    printf("Hello from test program!\n");
    return 0;
}
```

✅ **Expected Behavior:**
- Prints: "Hello from test program!"
- Exit code: 0

---

## Testing Steps

### Step 1: Test Original Executable

Run this in PowerShell:
```powershell
& "c:\Users\kacpe\Documents\decompiler\test_programs\hello_test.exe"
echo "Exit code: $LASTEXITCODE"
```

**Expected Output:**
```
Hello from test program!
Exit code: 0
```

---

### Step 2: Disassemble with Decompiler

1. Run the decompiler:
   ```powershell
   cd c:\Users\kacpe\Documents\decompiler\rust_file_explorer
   cargo run --release
   ```

2. Navigate to: `c:\Users\kacpe\Documents\decompiler\test_programs\hello_test.exe`

3. Press **F3** (Disassemble)

4. Wait for disassembly to complete

5. You should see: `hello_test.exe.asm` created

---

### Step 3: Reassemble with PE Reassembler

1. In the decompiler, open the `.asm` file: `hello_test.exe.asm`

2. Press **F5** (Compile)

3. **Look for this message:**
   ```
   ✓ Detected disassembled code from our disassembler
   🔨 Using PE Reassembler to preserve original PE structure...
   ```

4. Wait for compilation to complete

5. Check the compilation results panel

---

### Step 4: Test Reassembled Executable

Run this in PowerShell:
```powershell
& "c:\Users\kacpe\Documents\decompiler\test_programs\hello_test.exe"
echo "Exit code: $LASTEXITCODE"
```

**Expected Output (SAME as original):**
```
Hello from test program!
Exit code: 0
```

---

## Success Criteria

✅ **The PE Reassembler is working if:**
1. The reassembled executable prints "Hello from test program!"
2. The exit code is 0 (not 4198400!)
3. The output is identical to the original executable

❌ **The PE Reassembler has issues if:**
1. Exit code is 4198400 (0x401000) - entry point address
2. Exit code is 3221225477 (0xC0000005) - access violation
3. No output is printed
4. The executable crashes

---

## What's Different from hello_world_test.exe?

The previous test executable (`hello_world_test.exe`) was a **minimal 1024-byte test file** that:
- Only contained NOPs and a RET instruction
- Returned exit code 4198400 (the entry point address)
- Was NOT a real program

This new test executable (`hello_test.exe`) is a **real C program** that:
- Actually does something (prints to console)
- Has proper imports (printf from msvcrt.dll)
- Returns exit code 0 on success
- Is a proper test case for the PE reassembler

---

## Automated Test Script

You can also run the automated test:
```powershell
powershell -ExecutionPolicy Bypass -File "c:\Users\kacpe\Documents\decompiler\test_pe_reassembler_workflow.ps1"
```

This will check if the reassembled executable works correctly.

---

## Troubleshooting

### If the reassembled executable doesn't work:

1. **Check the compilation output** - look for error messages
2. **Check if PE Reassembler was used** - look for the message about detecting disassembled code
3. **Check the .asm file** - make sure it has the signature comments ("; Section:", "ENTRY POINT")
4. **Check file sizes** - original and reassembled should be similar
5. **Run with debug output** - check the console for detailed PE reassembler messages

### Common Issues:

- **Exit code 4198400**: Entry point not set correctly (but our test shows this is the original behavior)
- **Exit code 3221225477**: Access violation - imports or data sections not preserved
- **No output**: Code section not copied correctly
- **Crash on startup**: PE headers corrupted

---

## Next Steps

After confirming the PE Reassembler works with `hello_test.exe`, you can test with:
1. More complex programs (with multiple imports)
2. Programs with resources (icons, dialogs)
3. Programs with data sections
4. Programs with relocations

The PE Reassembler should preserve all of these correctly!