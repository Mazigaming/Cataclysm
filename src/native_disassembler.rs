// Rust FFI bindings for native C disassembler
// High-performance PE analysis with RIP-relative address handling
// NOTE: This module is optional - if C compilation fails, FFI calls will return None

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// C structure for RIP-relative references
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RipRef {
    pub address: u64,
    pub offset: i64,
    pub is_data: bool,
}

// Conditional compilation: Only link C code if it was successfully built
#[cfg(target_os = "windows")]
#[link(name = "disassembler")]
extern "C" {
    // Buffer management
    fn rip_buffer_init(capacity: usize) -> *mut std::ffi::c_void;
    #[allow(dead_code)]
    fn rip_buffer_write(handle: *mut std::ffi::c_void, text: *const c_char) -> bool;
    fn rip_buffer_get(handle: *mut std::ffi::c_void) -> *const c_char;
    fn rip_buffer_free(handle: *mut std::ffi::c_void);
    
    // PE parsing and validation
    fn rip_parse_pe_header(
        buffer: *const u8,
        size: usize,
        out_entry_point: *mut u32,
        out_is_64bit: *mut bool,
    ) -> bool;
    
    // RIP reference extraction
    fn rip_extract_references(
        code: *const u8,
        code_size: usize,
        base_va: u64,
        refs: *mut RipRef,
        max_refs: usize,
    ) -> usize;
    
    // RIP reference fixing
    fn rip_fix_references(
        asm_code: *const c_char,
        refs: *const RipRef,
        ref_count: usize,
        output_buffer: *mut std::ffi::c_void,
    ) -> bool;
    
    // Section validation
    fn rip_validate_section(code: *const u8, size: usize) -> bool;
    
    // Version info
    fn rip_get_version() -> *const c_char;
}

// Non-Windows or fallback implementations (stubs)
#[cfg(not(target_os = "windows"))]
mod native_stubs {
    use crate::native_disassembler::RipRef;
    use std::ffi::CStr;
    use std::os::raw::c_char;
    
    pub unsafe fn parse_pe_header_impl(_buffer: &[u8]) -> Option<(u32, bool)> {
        None // Fallback: not available
    }
    
    pub unsafe fn extract_rip_references_impl(
        _code: &[u8],
        _base_va: u64,
    ) -> Vec<RipRef> {
        Vec::new() // Fallback: return empty
    }
    
    pub unsafe fn fix_rip_references_impl(_asm_code: &str, _refs: &[RipRef]) -> Option<String> {
        None // Fallback: not available
    }
    
    pub unsafe fn validate_section_impl(_code: &[u8]) -> bool {
        false // Fallback: assume not valid
    }
    
    pub fn get_version_impl() -> String {
        "fallback (C module not available)".to_string()
    }
}

/// Wrapper for safe PE header parsing with validation
/// Returns: (entry_point_rva, is_64bit)
pub fn parse_pe_header(buffer: &[u8]) -> Option<(u32, bool)> {
    if buffer.len() < 64 {
        return None; // Too small to be valid PE
    }
    
    #[cfg(target_os = "windows")]
    {
        let mut entry_point = 0u32;
        let mut is_64bit = false;
        
        unsafe {
            if rip_parse_pe_header(
                buffer.as_ptr(),
                buffer.len(),
                &mut entry_point,
                &mut is_64bit,
            ) {
                // Additional sanity check
                if entry_point > 0 && entry_point < 0x80000000 {
                    Some((entry_point, is_64bit))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    unsafe {
        native_stubs::parse_pe_header_impl(buffer)
    }
}

/// Extract RIP-relative references from code section with automatic buffer expansion
pub fn extract_rip_references(
    code: &[u8],
    base_va: u64,
) -> Vec<RipRef> {
    #[cfg(target_os = "windows")]
    {
        // Dynamic sizing based on code size - estimate ~1 RIP ref per 50 bytes
        let estimated_refs = (code.len() / 50).max(100).min(10000);
        let mut refs = vec![RipRef {
            address: 0,
            offset: 0,
            is_data: false,
        }; estimated_refs];
        
        let count = unsafe {
            rip_extract_references(
                code.as_ptr(),
                code.len(),
                base_va,
                refs.as_mut_ptr(),
                refs.len(),
            )
        };
        
        refs.truncate(count);
        refs
    }
    
    #[cfg(not(target_os = "windows"))]
    unsafe {
        native_stubs::extract_rip_references_impl(code, base_va)
    }
}

/// Fix RIP-relative references in assembly code with dynamic buffer sizing
pub fn fix_rip_references(asm_code: &str, refs: &[RipRef]) -> Option<String> {
    if asm_code.is_empty() {
        return Some(String::new());
    }
    
    #[cfg(target_os = "windows")]
    {
        // Dynamic buffer size based on input (2x input size, min 1MB, max 16MB)
        let buffer_size = (asm_code.len() * 2).max(1024 * 1024).min(16 * 1024 * 1024);
        
        let asm_c = CString::new(asm_code).ok()?;
        
        let buffer = unsafe { rip_buffer_init(buffer_size) };
        if buffer.is_null() {
            return None;
        }
        
        let result = unsafe {
            rip_fix_references(
                asm_c.as_ptr(),
                refs.as_ptr(),
                refs.len(),
                buffer,
            )
        };
        
        if !result {
            unsafe { rip_buffer_free(buffer) };
            return None;
        }
        
        let result_ptr = unsafe { rip_buffer_get(buffer) };
        if result_ptr.is_null() {
            unsafe { rip_buffer_free(buffer) };
            return None;
        }
        
        let result_str = unsafe { CStr::from_ptr(result_ptr) }
            .to_string_lossy()
            .to_string();
        
        unsafe { rip_buffer_free(buffer) };
        
        Some(result_str)
    }
    
    #[cfg(not(target_os = "windows"))]
    unsafe {
        native_stubs::fix_rip_references_impl(asm_code, refs)
    }
}

/// Validate if a code section is likely executable code
pub fn validate_section(code: &[u8]) -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        rip_validate_section(code.as_ptr(), code.len())
    }
    
    #[cfg(not(target_os = "windows"))]
    unsafe {
        native_stubs::validate_section_impl(code)
    }
}

/// Get the native module version
pub fn get_version() -> String {
    #[cfg(target_os = "windows")]
    unsafe {
        let version_ptr = rip_get_version();
        if version_ptr.is_null() {
            "unknown".to_string()
        } else {
            CStr::from_ptr(version_ptr)
                .to_string_lossy()
                .to_string()
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        native_stubs::get_version_impl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
        println!("Native module version: {}", version);
    }
}