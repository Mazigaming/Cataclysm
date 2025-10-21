// ============================================================================
// WINDOWS API DATABASE
// ============================================================================
// This module contains information about common Windows API functions
// to generate proper declarations in C and Rust code.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ApiFunction {
    pub name: String,
    pub dll: String,
    pub return_type: String,
    pub parameters: Vec<ApiParameter>,
    pub c_declaration: String,
    pub rust_declaration: String,
}

#[derive(Debug, Clone)]
pub struct ApiParameter {
    pub name: String,
    pub param_type: String,
}

pub fn get_windows_api_database() -> HashMap<String, ApiFunction> {
    let mut db = HashMap::new();
    
    // Kernel32.dll functions
    db.insert("ExitProcess".to_string(), ApiFunction {
        name: "ExitProcess".to_string(),
        dll: "kernel32.dll".to_string(),
        return_type: "void".to_string(),
        parameters: vec![
            ApiParameter { name: "uExitCode".to_string(), param_type: "UINT".to_string() }
        ],
        c_declaration: "void ExitProcess(UINT uExitCode);".to_string(),
        rust_declaration: "fn ExitProcess(uExitCode: u32);".to_string(),
    });
    
    db.insert("GetStdHandle".to_string(), ApiFunction {
        name: "GetStdHandle".to_string(),
        dll: "kernel32.dll".to_string(),
        return_type: "HANDLE".to_string(),
        parameters: vec![
            ApiParameter { name: "nStdHandle".to_string(), param_type: "DWORD".to_string() }
        ],
        c_declaration: "HANDLE GetStdHandle(DWORD nStdHandle);".to_string(),
        rust_declaration: "fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;".to_string(),
    });
    
    db.insert("WriteFile".to_string(), ApiFunction {
        name: "WriteFile".to_string(),
        dll: "kernel32.dll".to_string(),
        return_type: "BOOL".to_string(),
        parameters: vec![
            ApiParameter { name: "hFile".to_string(), param_type: "HANDLE".to_string() },
            ApiParameter { name: "lpBuffer".to_string(), param_type: "LPCVOID".to_string() },
            ApiParameter { name: "nNumberOfBytesToWrite".to_string(), param_type: "DWORD".to_string() },
            ApiParameter { name: "lpNumberOfBytesWritten".to_string(), param_type: "LPDWORD".to_string() },
            ApiParameter { name: "lpOverlapped".to_string(), param_type: "LPOVERLAPPED".to_string() },
        ],
        c_declaration: "BOOL WriteFile(HANDLE hFile, LPCVOID lpBuffer, DWORD nNumberOfBytesToWrite, LPDWORD lpNumberOfBytesWritten, LPOVERLAPPED lpOverlapped);".to_string(),
        rust_declaration: "fn WriteFile(hFile: *mut std::ffi::c_void, lpBuffer: *const std::ffi::c_void, nNumberOfBytesToWrite: u32, lpNumberOfBytesWritten: *mut u32, lpOverlapped: *mut std::ffi::c_void) -> i32;".to_string(),
    });
    
    db.insert("WriteConsoleA".to_string(), ApiFunction {
        name: "WriteConsoleA".to_string(),
        dll: "kernel32.dll".to_string(),
        return_type: "BOOL".to_string(),
        parameters: vec![
            ApiParameter { name: "hConsoleOutput".to_string(), param_type: "HANDLE".to_string() },
            ApiParameter { name: "lpBuffer".to_string(), param_type: "const void*".to_string() },
            ApiParameter { name: "nNumberOfCharsToWrite".to_string(), param_type: "DWORD".to_string() },
            ApiParameter { name: "lpNumberOfCharsWritten".to_string(), param_type: "LPDWORD".to_string() },
            ApiParameter { name: "lpReserved".to_string(), param_type: "LPVOID".to_string() },
        ],
        c_declaration: "BOOL WriteConsoleA(HANDLE hConsoleOutput, const void* lpBuffer, DWORD nNumberOfCharsToWrite, LPDWORD lpNumberOfCharsWritten, LPVOID lpReserved);".to_string(),
        rust_declaration: "fn WriteConsoleA(hConsoleOutput: *mut std::ffi::c_void, lpBuffer: *const std::ffi::c_void, nNumberOfCharsToWrite: u32, lpNumberOfCharsWritten: *mut u32, lpReserved: *mut std::ffi::c_void) -> i32;".to_string(),
    });
    
    db.insert("GetLastError".to_string(), ApiFunction {
        name: "GetLastError".to_string(),
        dll: "kernel32.dll".to_string(),
        return_type: "DWORD".to_string(),
        parameters: vec![],
        c_declaration: "DWORD GetLastError(void);".to_string(),
        rust_declaration: "fn GetLastError() -> u32;".to_string(),
    });
    
    db.insert("GetModuleHandleA".to_string(), ApiFunction {
        name: "GetModuleHandleA".to_string(),
        dll: "kernel32.dll".to_string(),
        return_type: "HMODULE".to_string(),
        parameters: vec![
            ApiParameter { name: "lpModuleName".to_string(), param_type: "LPCSTR".to_string() }
        ],
        c_declaration: "HMODULE GetModuleHandleA(LPCSTR lpModuleName);".to_string(),
        rust_declaration: "fn GetModuleHandleA(lpModuleName: *const i8) -> *mut std::ffi::c_void;".to_string(),
    });
    
    db.insert("GetProcAddress".to_string(), ApiFunction {
        name: "GetProcAddress".to_string(),
        dll: "kernel32.dll".to_string(),
        return_type: "FARPROC".to_string(),
        parameters: vec![
            ApiParameter { name: "hModule".to_string(), param_type: "HMODULE".to_string() },
            ApiParameter { name: "lpProcName".to_string(), param_type: "LPCSTR".to_string() }
        ],
        c_declaration: "FARPROC GetProcAddress(HMODULE hModule, LPCSTR lpProcName);".to_string(),
        rust_declaration: "fn GetProcAddress(hModule: *mut std::ffi::c_void, lpProcName: *const i8) -> *mut std::ffi::c_void;".to_string(),
    });
    
    // User32.dll functions
    db.insert("MessageBoxA".to_string(), ApiFunction {
        name: "MessageBoxA".to_string(),
        dll: "user32.dll".to_string(),
        return_type: "int".to_string(),
        parameters: vec![
            ApiParameter { name: "hWnd".to_string(), param_type: "HWND".to_string() },
            ApiParameter { name: "lpText".to_string(), param_type: "LPCSTR".to_string() },
            ApiParameter { name: "lpCaption".to_string(), param_type: "LPCSTR".to_string() },
            ApiParameter { name: "uType".to_string(), param_type: "UINT".to_string() }
        ],
        c_declaration: "int MessageBoxA(HWND hWnd, LPCSTR lpText, LPCSTR lpCaption, UINT uType);".to_string(),
        rust_declaration: "fn MessageBoxA(hWnd: *mut std::ffi::c_void, lpText: *const i8, lpCaption: *const i8, uType: u32) -> i32;".to_string(),
    });
    
    // MSVCRT.dll functions
    db.insert("printf".to_string(), ApiFunction {
        name: "printf".to_string(),
        dll: "msvcrt.dll".to_string(),
        return_type: "int".to_string(),
        parameters: vec![
            ApiParameter { name: "format".to_string(), param_type: "const char*".to_string() }
        ],
        c_declaration: "int printf(const char* format, ...);".to_string(),
        rust_declaration: "fn printf(format: *const i8, ...) -> i32;".to_string(),
    });
    
    db.insert("puts".to_string(), ApiFunction {
        name: "puts".to_string(),
        dll: "msvcrt.dll".to_string(),
        return_type: "int".to_string(),
        parameters: vec![
            ApiParameter { name: "str".to_string(), param_type: "const char*".to_string() }
        ],
        c_declaration: "int puts(const char* str);".to_string(),
        rust_declaration: "fn puts(str: *const i8) -> i32;".to_string(),
    });
    
    db.insert("malloc".to_string(), ApiFunction {
        name: "malloc".to_string(),
        dll: "msvcrt.dll".to_string(),
        return_type: "void*".to_string(),
        parameters: vec![
            ApiParameter { name: "size".to_string(), param_type: "size_t".to_string() }
        ],
        c_declaration: "void* malloc(size_t size);".to_string(),
        rust_declaration: "fn malloc(size: usize) -> *mut std::ffi::c_void;".to_string(),
    });
    
    db.insert("free".to_string(), ApiFunction {
        name: "free".to_string(),
        dll: "msvcrt.dll".to_string(),
        return_type: "void".to_string(),
        parameters: vec![
            ApiParameter { name: "ptr".to_string(), param_type: "void*".to_string() }
        ],
        c_declaration: "void free(void* ptr);".to_string(),
        rust_declaration: "fn free(ptr: *mut std::ffi::c_void);".to_string(),
    });
    
    db.insert("memcpy".to_string(), ApiFunction {
        name: "memcpy".to_string(),
        dll: "msvcrt.dll".to_string(),
        return_type: "void*".to_string(),
        parameters: vec![
            ApiParameter { name: "dest".to_string(), param_type: "void*".to_string() },
            ApiParameter { name: "src".to_string(), param_type: "const void*".to_string() },
            ApiParameter { name: "n".to_string(), param_type: "size_t".to_string() }
        ],
        c_declaration: "void* memcpy(void* dest, const void* src, size_t n);".to_string(),
        rust_declaration: "fn memcpy(dest: *mut std::ffi::c_void, src: *const std::ffi::c_void, n: usize) -> *mut std::ffi::c_void;".to_string(),
    });
    
    db.insert("exit".to_string(), ApiFunction {
        name: "exit".to_string(),
        dll: "msvcrt.dll".to_string(),
        return_type: "void".to_string(),
        parameters: vec![
            ApiParameter { name: "status".to_string(), param_type: "int".to_string() }
        ],
        c_declaration: "void exit(int status);".to_string(),
        rust_declaration: "fn exit(status: i32);".to_string(),
    });
    
    db
}

/// Detect which Windows APIs are being called in the assembly code
pub fn detect_api_calls_in_code(asm: &str) -> Vec<String> {
    let db = get_windows_api_database();
    let mut detected = Vec::new();
    
    for (api_name, _) in db.iter() {
        if asm.contains(api_name) {
            detected.push(api_name.clone());
        }
    }
    
    detected.sort();
    detected.dedup();
    detected
}

/// Generate C header declarations for detected APIs
pub fn generate_c_api_declarations(api_names: &[String]) -> String {
    let db = get_windows_api_database();
    let mut output = String::new();
    
    output.push_str("// ═══ Windows API Declarations ═══\n");
    output.push_str("// These functions are imported from Windows DLLs\n\n");
    
    // Group by DLL
    let mut kernel32_apis = Vec::new();
    let mut user32_apis = Vec::new();
    let mut msvcrt_apis = Vec::new();
    
    for api_name in api_names {
        if let Some(api) = db.get(api_name) {
            match api.dll.as_str() {
                "kernel32.dll" => kernel32_apis.push(api),
                "user32.dll" => user32_apis.push(api),
                "msvcrt.dll" => msvcrt_apis.push(api),
                _ => {}
            }
        }
    }
    
    if !kernel32_apis.is_empty() {
        output.push_str("// kernel32.dll\n");
        for api in kernel32_apis {
            output.push_str(&format!("{}\n", api.c_declaration));
        }
        output.push_str("\n");
    }
    
    if !user32_apis.is_empty() {
        output.push_str("// user32.dll\n");
        for api in user32_apis {
            output.push_str(&format!("{}\n", api.c_declaration));
        }
        output.push_str("\n");
    }
    
    if !msvcrt_apis.is_empty() {
        output.push_str("// msvcrt.dll (C Runtime)\n");
        for api in msvcrt_apis {
            output.push_str(&format!("{}\n", api.c_declaration));
        }
        output.push_str("\n");
    }
    
    output
}

/// Generate Rust FFI declarations for detected APIs
pub fn generate_rust_api_declarations(api_names: &[String]) -> String {
    let db = get_windows_api_database();
    let mut output = String::new();
    
    output.push_str("// ═══ Windows API FFI Declarations ═══\n");
    output.push_str("#[cfg(windows)]\n");
    output.push_str("#[link(name = \"kernel32\")]\n");
    output.push_str("extern \"system\" {\n");
    
    for api_name in api_names {
        if let Some(api) = db.get(api_name) {
            if api.dll == "kernel32.dll" {
                output.push_str(&format!("    {}\n", api.rust_declaration));
            }
        }
    }
    
    output.push_str("}\n\n");
    
    // Check if we need user32
    let has_user32 = api_names.iter().any(|name| {
        db.get(name).map(|api| api.dll == "user32.dll").unwrap_or(false)
    });
    
    if has_user32 {
        output.push_str("#[cfg(windows)]\n");
        output.push_str("#[link(name = \"user32\")]\n");
        output.push_str("extern \"system\" {\n");
        
        for api_name in api_names {
            if let Some(api) = db.get(api_name) {
                if api.dll == "user32.dll" {
                    output.push_str(&format!("    {}\n", api.rust_declaration));
                }
            }
        }
        
        output.push_str("}\n\n");
    }
    
    // Check if we need msvcrt
    let has_msvcrt = api_names.iter().any(|name| {
        db.get(name).map(|api| api.dll == "msvcrt.dll").unwrap_or(false)
    });
    
    if has_msvcrt {
        output.push_str("#[cfg(windows)]\n");
        output.push_str("#[link(name = \"msvcrt\")]\n");
        output.push_str("extern \"C\" {\n");
        
        for api_name in api_names {
            if let Some(api) = db.get(api_name) {
                if api.dll == "msvcrt.dll" {
                    output.push_str(&format!("    {}\n", api.rust_declaration));
                }
            }
        }
        
        output.push_str("}\n\n");
    }
    
    output
}