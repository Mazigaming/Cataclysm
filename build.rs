// Build script to compile native C code with cross-platform support
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let _out_path = PathBuf::from(&out_dir);
    
    // Compile native module based on target platform
    #[cfg(target_os = "windows")]
    compile_c_for_windows(&out_dir);
    
    #[cfg(target_os = "linux")]
    compile_c_for_linux(&out_dir);
    
    #[cfg(target_os = "macos")]
    compile_c_for_macos(&out_dir);
    
    println!("cargo:rerun-if-changed=native/disassembler.c");
}

#[cfg(target_os = "windows")]
fn compile_c_for_windows(out_dir: &str) {
    use std::process::Command;
    
    // Try with MSVC first (cl.exe)
    let msvc_result = Command::new("cl.exe")
        .args(&[
            "/c",
            "native/disassembler.c",
            &format!("/Fo{}/disassembler.obj", out_dir),
            "/O2",
            "/W3",
        ])
        .status();
    
    if msvc_result.is_ok() {
        // Link with MSVC linker
        let _ = Command::new("lib.exe")
            .args(&[
                &format!("{}/disassembler.obj", out_dir),
                &format!("/OUT:{}/disassembler.lib", out_dir),
            ])
            .status();
        
        println!("cargo:rustc-link-lib=disassembler");
        println!("cargo:rustc-link-search=native={}", out_dir);
        return;
    }
    
    // Fall back to gcc/clang
    let gcc_result = Command::new("gcc")
        .args(&[
            "-c",
            "native/disassembler.c",
            "-o",
            &format!("{}/disassembler.o", out_dir),
            "-O2",
            "-Wall",
        ])
        .status();
    
    if gcc_result.is_ok() {
        let _ = Command::new("ar")
            .args(&[
                "rcs",
                &format!("{}/libdisassembler.a", out_dir),
                &format!("{}/disassembler.o", out_dir),
            ])
            .status();
        
        println!("cargo:rustc-link-lib=static=disassembler");
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
}

#[cfg(target_os = "linux")]
fn compile_c_for_linux(out_dir: &str) {
    use std::process::Command;
    
    // Try with gcc first
    let gcc_result = Command::new("gcc")
        .args(&[
            "-c",
            "native/disassembler.c",
            "-o",
            &format!("{}/disassembler.o", out_dir),
            "-O2",
            "-Wall",
            "-fPIC",
        ])
        .status();
    
    if gcc_result.is_ok() {
        let _ = Command::new("ar")
            .args(&[
                "rcs",
                &format!("{}/libdisassembler.a", out_dir),
                &format!("{}/disassembler.o", out_dir),
            ])
            .status();
        
        println!("cargo:rustc-link-lib=static=disassembler");
        println!("cargo:rustc-link-search=native={}", out_dir);
        return;
    }
    
    // Try with clang as fallback
    let clang_result = Command::new("clang")
        .args(&[
            "-c",
            "native/disassembler.c",
            "-o",
            &format!("{}/disassembler.o", out_dir),
            "-O2",
            "-Wall",
            "-fPIC",
        ])
        .status();
    
    if clang_result.is_ok() {
        let _ = Command::new("ar")
            .args(&[
                "rcs",
                &format!("{}/libdisassembler.a", out_dir),
                &format!("{}/disassembler.o", out_dir),
            ])
            .status();
        
        println!("cargo:rustc-link-lib=static=disassembler");
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
}

#[cfg(target_os = "macos")]
fn compile_c_for_macos(out_dir: &str) {
    use std::process::Command;
    
    // Try with clang (usually available on macOS)
    let clang_result = Command::new("clang")
        .args(&[
            "-c",
            "native/disassembler.c",
            "-o",
            &format!("{}/disassembler.o", out_dir),
            "-O2",
            "-Wall",
            "-fPIC",
        ])
        .status();
    
    if clang_result.is_ok() {
        let _ = Command::new("ar")
            .args(&[
                "rcs",
                &format!("{}/libdisassembler.a", out_dir),
                &format!("{}/disassembler.o", out_dir),
            ])
            .status();
        
        println!("cargo:rustc-link-lib=static=disassembler");
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn compile_c_for_windows(_out_dir: &str) {
    // On unsupported platforms, skip C compilation
}