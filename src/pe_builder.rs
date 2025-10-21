// ============================================================================
// ADVANCED PE BUILDER WITH FULL IMPORT TABLE SUPPORT
// ============================================================================
// This module creates fully functional PE executables with:
// - Import Directory Table (IDT)
// - Import Address Table (IAT)
// - Import Lookup Table (ILT)
// - Multiple DLL imports
// - Proper section layout
// - Full Windows API support
// ============================================================================

use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImportFunction {
    pub name: String,
    pub ordinal: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ImportDll {
    pub name: String,
    pub functions: Vec<ImportFunction>,
}

#[derive(Debug, Clone)]
pub struct PEBuilder {
    pub code: Vec<u8>,
    pub data: Vec<u8>,
    pub entry_point_rva: u32,
    pub is_64bit: bool,
    pub imports: Vec<ImportDll>,
    pub image_base: u64,
}

impl PEBuilder {
    pub fn new(is_64bit: bool) -> Self {
        Self {
            code: Vec::new(),
            data: Vec::new(),
            entry_point_rva: 0x1000,
            is_64bit,
            imports: Vec::new(),
            image_base: if is_64bit { 0x140000000 } else { 0x400000 },
        }
    }

    pub fn add_code(&mut self, code: Vec<u8>) {
        self.code = code;
    }

    pub fn add_import(&mut self, dll_name: String, function_name: String) {
        // Find or create DLL entry
        if let Some(dll) = self.imports.iter_mut().find(|d| d.name.eq_ignore_ascii_case(&dll_name)) {
            // Check if function already exists
            if !dll.functions.iter().any(|f| f.name == function_name) {
                dll.functions.push(ImportFunction {
                    name: function_name,
                    ordinal: None,
                });
            }
        } else {
            // Create new DLL entry
            self.imports.push(ImportDll {
                name: dll_name,
                functions: vec![ImportFunction {
                    name: function_name,
                    ordinal: None,
                }],
            });
        }
    }

    pub fn build(&self, output_path: &Path) -> Result<(), String> {
        let mut pe = Vec::new();

        // ====================================================================
        // 1. DOS HEADER
        // ====================================================================
        self.write_dos_header(&mut pe);

        // ====================================================================
        // 2. PE SIGNATURE & COFF HEADER
        // ====================================================================
        let num_sections = if self.imports.is_empty() { 1 } else { 3 }; // .text, .rdata, .idata
        self.write_pe_signature(&mut pe);
        self.write_coff_header(&mut pe, num_sections);

        // ====================================================================
        // 3. OPTIONAL HEADER
        // ====================================================================
        let _optional_header_start = pe.len();
        self.write_optional_header_start(&mut pe);

        // Calculate section layout
        let section_alignment = 0x1000u32;
        let file_alignment = 0x200u32;

        // Section RVAs
        let text_rva = section_alignment; // 0x1000
        let rdata_rva = text_rva + self.align(self.code.len() as u32, section_alignment);
        let idata_rva = rdata_rva + self.align(self.calculate_rdata_size(), section_alignment);

        // Calculate import directory size
        let (import_dir_rva, import_dir_size) = if !self.imports.is_empty() {
            (idata_rva, self.calculate_import_directory_size())
        } else {
            (0, 0)
        };

        // Calculate image size
        let image_size = if self.imports.is_empty() {
            text_rva + self.align(self.code.len() as u32, section_alignment)
        } else {
            idata_rva + self.align(import_dir_size, section_alignment)
        };

        // Write data directories
        self.write_data_directories(&mut pe, import_dir_rva, import_dir_size, image_size);

        // ====================================================================
        // 4. SECTION HEADERS
        // ====================================================================
        let headers_size = self.align(pe.len() as u32 + (num_sections as u32 * 40) + 40, file_alignment);
        
        // .text section
        self.write_section_header(
            &mut pe,
            b".text\0\0\0",
            self.code.len() as u32,
            text_rva,
            self.align(self.code.len() as u32, file_alignment),
            headers_size,
            0x60000020, // CODE | EXECUTE | READ
        );

        if !self.imports.is_empty() {
            // .rdata section (for import names and DLL names)
            let rdata_size = self.calculate_rdata_size();
            let rdata_file_offset = headers_size + self.align(self.code.len() as u32, file_alignment);
            self.write_section_header(
                &mut pe,
                b".rdata\0\0",
                rdata_size,
                rdata_rva,
                self.align(rdata_size, file_alignment),
                rdata_file_offset,
                0x40000040, // INITIALIZED_DATA | READ
            );

            // .idata section (for import tables)
            let idata_file_offset = rdata_file_offset + self.align(rdata_size, file_alignment);
            self.write_section_header(
                &mut pe,
                b".idata\0\0",
                import_dir_size,
                idata_rva,
                self.align(import_dir_size, file_alignment),
                idata_file_offset,
                0xC0000040, // INITIALIZED_DATA | READ | WRITE
            );
        }

        // ====================================================================
        // 5. PAD TO FILE ALIGNMENT (END OF HEADERS)
        // ====================================================================
        while pe.len() < headers_size as usize {
            pe.push(0);
        }

        // ====================================================================
        // 6. .TEXT SECTION (CODE)
        // ====================================================================
        pe.extend_from_slice(&self.code);
        while pe.len() < (headers_size + self.align(self.code.len() as u32, file_alignment)) as usize {
            pe.push(0);
        }

        // ====================================================================
        // 7. .RDATA SECTION (IMPORT NAMES & DLL NAMES)
        // ====================================================================
        if !self.imports.is_empty() {
            let rdata_start = pe.len();
            let rdata_content = self.build_rdata_section(rdata_rva);
            pe.extend_from_slice(&rdata_content);
            
            let rdata_size = self.calculate_rdata_size();
            while pe.len() < rdata_start + self.align(rdata_size, file_alignment) as usize {
                pe.push(0);
            }
        }

        // ====================================================================
        // 8. .IDATA SECTION (IMPORT TABLES)
        // ====================================================================
        if !self.imports.is_empty() {
            let idata_start = pe.len();
            let idata_content = self.build_idata_section(idata_rva, rdata_rva);
            pe.extend_from_slice(&idata_content);
            
            while pe.len() < idata_start + self.align(import_dir_size, file_alignment) as usize {
                pe.push(0);
            }
        }

        // ====================================================================
        // 9. WRITE TO FILE
        // ====================================================================
        fs::write(output_path, &pe)
            .map_err(|e| format!("Failed to write PE file: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    fn align(&self, value: u32, alignment: u32) -> u32 {
        (value + alignment - 1) & !(alignment - 1)
    }

    fn write_dos_header(&self, pe: &mut Vec<u8>) {
        // DOS Header
        pe.extend_from_slice(b"MZ"); // e_magic
        for _ in 0..58 {
            pe.push(0);
        }
        pe.extend_from_slice(&0x80u32.to_le_bytes()); // e_lfanew (PE header offset)
        
        // DOS Stub
        while pe.len() < 0x80 {
            pe.push(0);
        }
    }

    fn write_pe_signature(&self, pe: &mut Vec<u8>) {
        pe.extend_from_slice(b"PE\0\0");
    }

    fn write_coff_header(&self, pe: &mut Vec<u8>, num_sections: u16) {
        let machine = if self.is_64bit { 0x8664u16 } else { 0x014Cu16 };
        pe.extend_from_slice(&machine.to_le_bytes());
        pe.extend_from_slice(&num_sections.to_le_bytes());
        pe.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
        pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToSymbolTable
        pe.extend_from_slice(&0u32.to_le_bytes()); // NumberOfSymbols
        
        let optional_header_size = if self.is_64bit { 0xF0u16 } else { 0xE0u16 };
        pe.extend_from_slice(&optional_header_size.to_le_bytes());
        pe.extend_from_slice(&0x22u16.to_le_bytes()); // Characteristics: EXECUTABLE | LARGE_ADDRESS_AWARE
    }

    fn write_optional_header_start(&self, pe: &mut Vec<u8>) {
        let magic = if self.is_64bit { 0x20Bu16 } else { 0x10Bu16 };
        pe.extend_from_slice(&magic.to_le_bytes());
        pe.extend_from_slice(&14u8.to_le_bytes()); // MajorLinkerVersion
        pe.extend_from_slice(&0u8.to_le_bytes()); // MinorLinkerVersion
        pe.extend_from_slice(&(self.code.len() as u32).to_le_bytes()); // SizeOfCode
        
        let init_data_size = if self.imports.is_empty() {
            0u32
        } else {
            self.calculate_rdata_size() + self.calculate_import_directory_size()
        };
        pe.extend_from_slice(&init_data_size.to_le_bytes()); // SizeOfInitializedData
        pe.extend_from_slice(&0u32.to_le_bytes()); // SizeOfUninitializedData
        pe.extend_from_slice(&self.entry_point_rva.to_le_bytes()); // AddressOfEntryPoint
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // BaseOfCode
        
        if !self.is_64bit {
            pe.extend_from_slice(&0x1000u32.to_le_bytes()); // BaseOfData (32-bit only)
        }
    }

    fn write_data_directories(&self, pe: &mut Vec<u8>, import_dir_rva: u32, import_dir_size: u32, image_size: u32) {
        // ImageBase
        if self.is_64bit {
            pe.extend_from_slice(&self.image_base.to_le_bytes());
        } else {
            pe.extend_from_slice(&(self.image_base as u32).to_le_bytes());
        }
        
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // SectionAlignment
        pe.extend_from_slice(&0x200u32.to_le_bytes()); // FileAlignment
        pe.extend_from_slice(&6u16.to_le_bytes()); // MajorOperatingSystemVersion
        pe.extend_from_slice(&0u16.to_le_bytes()); // MinorOperatingSystemVersion
        pe.extend_from_slice(&0u16.to_le_bytes()); // MajorImageVersion
        pe.extend_from_slice(&0u16.to_le_bytes()); // MinorImageVersion
        pe.extend_from_slice(&6u16.to_le_bytes()); // MajorSubsystemVersion
        pe.extend_from_slice(&0u16.to_le_bytes()); // MinorSubsystemVersion
        pe.extend_from_slice(&0u32.to_le_bytes()); // Win32VersionValue
        pe.extend_from_slice(&image_size.to_le_bytes()); // SizeOfImage
        pe.extend_from_slice(&0x200u32.to_le_bytes()); // SizeOfHeaders
        pe.extend_from_slice(&0u32.to_le_bytes()); // CheckSum
        pe.extend_from_slice(&3u16.to_le_bytes()); // Subsystem (CONSOLE)
        pe.extend_from_slice(&0x8160u16.to_le_bytes()); // DllCharacteristics: DYNAMIC_BASE | NX_COMPAT | TERMINAL_SERVER_AWARE
        
        if self.is_64bit {
            pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfStackReserve
            pe.extend_from_slice(&0x1000u64.to_le_bytes()); // SizeOfStackCommit
            pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfHeapReserve
            pe.extend_from_slice(&0x1000u64.to_le_bytes()); // SizeOfHeapCommit
        } else {
            pe.extend_from_slice(&0x100000u32.to_le_bytes());
            pe.extend_from_slice(&0x1000u32.to_le_bytes());
            pe.extend_from_slice(&0x100000u32.to_le_bytes());
            pe.extend_from_slice(&0x1000u32.to_le_bytes());
        }
        
        pe.extend_from_slice(&0u32.to_le_bytes()); // LoaderFlags
        pe.extend_from_slice(&16u32.to_le_bytes()); // NumberOfRvaAndSizes
        
        // Data Directories (16 entries, 8 bytes each)
        // 0: Export Table
        pe.extend_from_slice(&0u32.to_le_bytes());
        pe.extend_from_slice(&0u32.to_le_bytes());
        
        // 1: Import Table
        pe.extend_from_slice(&import_dir_rva.to_le_bytes());
        pe.extend_from_slice(&import_dir_size.to_le_bytes());
        
        // 2-15: Other directories (all zeros)
        for _ in 0..14 {
            pe.extend_from_slice(&0u64.to_le_bytes());
        }
    }

    fn write_section_header(
        &self,
        pe: &mut Vec<u8>,
        name: &[u8; 8],
        virtual_size: u32,
        virtual_address: u32,
        size_of_raw_data: u32,
        pointer_to_raw_data: u32,
        characteristics: u32,
    ) {
        pe.extend_from_slice(name);
        pe.extend_from_slice(&virtual_size.to_le_bytes());
        pe.extend_from_slice(&virtual_address.to_le_bytes());
        pe.extend_from_slice(&size_of_raw_data.to_le_bytes());
        pe.extend_from_slice(&pointer_to_raw_data.to_le_bytes());
        pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToRelocations
        pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToLinenumbers
        pe.extend_from_slice(&0u16.to_le_bytes()); // NumberOfRelocations
        pe.extend_from_slice(&0u16.to_le_bytes()); // NumberOfLinenumbers
        pe.extend_from_slice(&characteristics.to_le_bytes());
    }

    fn calculate_rdata_size(&self) -> u32 {
        let mut size = 0u32;
        
        // DLL names
        for dll in &self.imports {
            size += dll.name.len() as u32 + 1; // +1 for null terminator
        }
        
        // Function names (with hint/name table entries)
        for dll in &self.imports {
            for func in &dll.functions {
                size += 2; // Hint (u16)
                size += func.name.len() as u32 + 1; // Name + null terminator
                // Align to 2 bytes
                if size % 2 != 0 {
                    size += 1;
                }
            }
        }
        
        size
    }

    fn calculate_import_directory_size(&self) -> u32 {
        let ptr_size = if self.is_64bit { 8 } else { 4 };
        
        // Import Directory Table: (num_dlls + 1) * 20 bytes
        let idt_size = (self.imports.len() + 1) * 20;
        
        // Import Lookup Table (ILT) and Import Address Table (IAT)
        let mut tables_size = 0;
        for dll in &self.imports {
            // ILT: (num_functions + 1) * ptr_size
            tables_size += (dll.functions.len() + 1) * ptr_size;
            // IAT: (num_functions + 1) * ptr_size
            tables_size += (dll.functions.len() + 1) * ptr_size;
        }
        
        (idt_size + tables_size) as u32
    }

    fn build_rdata_section(&self, rdata_rva: u32) -> Vec<u8> {
        let mut rdata = Vec::new();
        let mut current_rva = rdata_rva;
        
        // Build a map of string -> RVA
        let mut string_rvas = HashMap::new();
        
        // Write DLL names
        for dll in &self.imports {
            string_rvas.insert(format!("dll:{}", dll.name), current_rva);
            rdata.extend_from_slice(dll.name.as_bytes());
            rdata.push(0);
            current_rva += dll.name.len() as u32 + 1;
        }
        
        // Write function names (Hint/Name Table)
        for dll in &self.imports {
            for func in &dll.functions {
                string_rvas.insert(format!("func:{}:{}", dll.name, func.name), current_rva);
                
                // Hint (ordinal hint, we use 0)
                rdata.extend_from_slice(&0u16.to_le_bytes());
                current_rva += 2;
                
                // Function name
                rdata.extend_from_slice(func.name.as_bytes());
                rdata.push(0);
                current_rva += func.name.len() as u32 + 1;
                
                // Align to 2 bytes
                if current_rva % 2 != 0 {
                    rdata.push(0);
                    current_rva += 1;
                }
            }
        }
        
        rdata
    }

    fn build_idata_section(&self, idata_rva: u32, rdata_rva: u32) -> Vec<u8> {
        let mut idata = Vec::new();
        let ptr_size = if self.is_64bit { 8 } else { 4 };
        
        // Calculate RVAs for tables
        let idt_size = (self.imports.len() + 1) * 20;
        let ilt_rva = idata_rva + idt_size as u32;
        let mut iat_rva = ilt_rva;
        
        // Calculate total ILT size
        for dll in &self.imports {
            iat_rva += ((dll.functions.len() + 1) * ptr_size) as u32;
        }
        
        // Track current positions
        let mut current_ilt_rva;
        let mut current_iat_rva;
        let mut current_name_rva = rdata_rva;
        
        // Build string RVA map
        let mut dll_name_rvas = HashMap::new();
        let mut func_name_rvas = HashMap::new();
        
        // Calculate DLL name RVAs
        for dll in &self.imports {
            dll_name_rvas.insert(dll.name.clone(), current_name_rva);
            current_name_rva += dll.name.len() as u32 + 1;
        }
        
        // Calculate function name RVAs
        for dll in &self.imports {
            for func in &dll.functions {
                func_name_rvas.insert(format!("{}:{}", dll.name, func.name), current_name_rva);
                current_name_rva += 2; // Hint
                current_name_rva += func.name.len() as u32 + 1; // Name + null
                if current_name_rva % 2 != 0 {
                    current_name_rva += 1; // Alignment
                }
            }
        }
        
        // ====================================================================
        // 1. IMPORT DIRECTORY TABLE
        // ====================================================================
        current_ilt_rva = ilt_rva;
        current_iat_rva = iat_rva;
        
        for dll in &self.imports {
            // Import Directory Entry (20 bytes)
            idata.extend_from_slice(&current_ilt_rva.to_le_bytes()); // OriginalFirstThunk (ILT)
            idata.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
            idata.extend_from_slice(&0u32.to_le_bytes()); // ForwarderChain
            idata.extend_from_slice(&dll_name_rvas[&dll.name].to_le_bytes()); // Name RVA
            idata.extend_from_slice(&current_iat_rva.to_le_bytes()); // FirstThunk (IAT)
            
            current_ilt_rva += ((dll.functions.len() + 1) * ptr_size) as u32;
            current_iat_rva += ((dll.functions.len() + 1) * ptr_size) as u32;
        }
        
        // Null terminator entry
        for _ in 0..20 {
            idata.push(0);
        }
        
        // ====================================================================
        // 2. IMPORT LOOKUP TABLE (ILT)
        // ====================================================================
        for dll in &self.imports {
            for func in &dll.functions {
                let func_rva = func_name_rvas[&format!("{}:{}", dll.name, func.name)];
                if self.is_64bit {
                    idata.extend_from_slice(&(func_rva as u64).to_le_bytes());
                } else {
                    idata.extend_from_slice(&func_rva.to_le_bytes());
                }
            }
            // Null terminator
            if self.is_64bit {
                idata.extend_from_slice(&0u64.to_le_bytes());
            } else {
                idata.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        
        // ====================================================================
        // 3. IMPORT ADDRESS TABLE (IAT)
        // ====================================================================
        for dll in &self.imports {
            for func in &dll.functions {
                let func_rva = func_name_rvas[&format!("{}:{}", dll.name, func.name)];
                if self.is_64bit {
                    idata.extend_from_slice(&(func_rva as u64).to_le_bytes());
                } else {
                    idata.extend_from_slice(&func_rva.to_le_bytes());
                }
            }
            // Null terminator
            if self.is_64bit {
                idata.extend_from_slice(&0u64.to_le_bytes());
            } else {
                idata.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        
        idata
    }
}

/// Parse assembly code to detect external API calls
pub fn detect_external_calls(asm: &str) -> Vec<(String, String)> {
    let mut calls = Vec::new();
    
    // Common Windows API patterns
    let api_patterns = vec![
        ("kernel32.dll", vec!["ExitProcess", "GetStdHandle", "WriteFile", "WriteConsoleA", "GetLastError", "GetModuleHandleA", "GetProcAddress", "LoadLibraryA", "VirtualAlloc", "VirtualFree", "CreateFileA", "ReadFile", "CloseHandle"]),
        ("user32.dll", vec!["MessageBoxA", "MessageBoxW", "CreateWindowExA", "ShowWindow", "UpdateWindow", "GetMessageA", "DispatchMessageA"]),
        ("msvcrt.dll", vec!["printf", "scanf", "puts", "malloc", "free", "memcpy", "memset", "strlen", "strcpy", "strcmp", "exit"]),
        ("ntdll.dll", vec!["NtQuerySystemInformation", "RtlGetVersion", "NtAllocateVirtualMemory"]),
    ];
    
    for (dll, functions) in api_patterns {
        for func in functions {
            if asm.contains(func) {
                calls.push((dll.to_string(), func.to_string()));
            }
        }
    }
    
    calls.sort();
    calls.dedup();
    calls
}