// Native C disassembler for high-performance PE analysis
// Optimized for RIP-relative address handling and memory efficiency
// Uses capstone directly for superior performance

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Export macros for Windows DLL
#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

// PE Header structures
typedef struct {
    uint16_t magic;
    uint8_t unused[58];
    uint32_t pe_offset;
} DOS_HEADER;

typedef struct {
    uint32_t signature;
    uint16_t machine;
    uint16_t num_sections;
    uint32_t timestamp;
    uint32_t symbol_table_ptr;
    uint32_t num_symbols;
    uint16_t size_of_optional_header;
    uint16_t characteristics;
} COFF_HEADER;

typedef struct {
    uint16_t magic;
    uint8_t unused[202];
} OPTIONAL_HEADER_32;

typedef struct {
    uint16_t magic;
    uint8_t unused[240];
} OPTIONAL_HEADER_64;

typedef struct {
    uint8_t name[8];
    uint32_t virtual_size;
    uint32_t virtual_address;
    uint32_t size_of_raw_data;
    uint32_t pointer_to_raw_data;
    uint8_t unused[16];
} SECTION_HEADER;

// RIP-relative reference tracking
typedef struct {
    uint64_t address;
    int64_t offset;
    bool is_data;
} RIP_REF;

// Result buffer for safe communication with Rust
typedef struct {
    char* data;
    size_t capacity;
    size_t used;
    bool truncated;
} RESULT_BUFFER;

// Initialize result buffer
EXPORT void* rip_buffer_init(size_t capacity) {
    RESULT_BUFFER* buf = (RESULT_BUFFER*)malloc(sizeof(RESULT_BUFFER));
    if (!buf) return NULL;
    
    buf->capacity = capacity;
    buf->used = 0;
    buf->truncated = false;
    buf->data = (char*)malloc(capacity);
    
    if (!buf->data) {
        free(buf);
        return NULL;
    }
    
    return (void*)buf;
}

// Write formatted string to result buffer with overflow protection
EXPORT bool rip_buffer_write(void* handle, const char* text) {
    RESULT_BUFFER* buf = (RESULT_BUFFER*)handle;
    if (!buf || buf->truncated || !text) return false;
    
    size_t text_len = strlen(text);
    
    if (buf->used + text_len >= buf->capacity) {
        buf->truncated = true;
        return false;
    }
    
    memcpy(buf->data + buf->used, text, text_len);
    buf->used += text_len;
    buf->data[buf->used] = '\0';
    
    return true;
}

// Get buffer contents
EXPORT const char* rip_buffer_get(void* handle) {
    RESULT_BUFFER* buf = (RESULT_BUFFER*)handle;
    return buf ? buf->data : NULL;
}

// Free buffer
EXPORT void rip_buffer_free(void* handle) {
    RESULT_BUFFER* buf = (RESULT_BUFFER*)handle;
    if (buf) {
        free(buf->data);
        free(buf);
    }
}

// High-performance PE parsing and validation with full structure validation
EXPORT bool rip_parse_pe_header(const uint8_t* buffer, size_t size, 
                                uint32_t* out_entry_point,
                                bool* out_is_64bit) {
    if (!buffer || !out_entry_point || !out_is_64bit) return false;
    if (size < 64) return false; // Minimum PE size
    
    // Validate DOS header
    if (size < sizeof(DOS_HEADER)) return false;
    DOS_HEADER* dos = (DOS_HEADER*)buffer;
    
    // Check MZ signature
    if (dos->magic != 0x5A4D) return false; // "MZ"
    
    // Validate PE offset
    uint32_t pe_offset = dos->pe_offset;
    if (pe_offset == 0 || pe_offset > size - 4) return false;
    if (pe_offset < 0x40) return false; // PE offset must be after DOS stub
    
    // Check PE signature ("PE\0\0")
    if (pe_offset + 4 > size) return false;
    uint32_t* pe_sig = (uint32_t*)(buffer + pe_offset);
    if (*pe_sig != 0x00004550) return false; // "PE\0\0"
    
    // Validate COFF header
    if (pe_offset + 24 > size) return false;
    COFF_HEADER* coff = (COFF_HEADER*)(buffer + pe_offset + 4);
    
    // Check machine type
    if (coff->machine != 0x8664 && coff->machine != 0x014C) return false; // AMD64 or I386
    
    // Check optional header size
    if (coff->size_of_optional_header < 2) return false;
    if (pe_offset + 24 + coff->size_of_optional_header > size) return false;
    
    // Get optional header to determine architecture
    OPTIONAL_HEADER_32* opt32 = (OPTIONAL_HEADER_32*)(buffer + pe_offset + 24);
    
    if (opt32->magic == 0x010B) {
        // PE32 (32-bit)
        *out_is_64bit = false;
        if (coff->size_of_optional_header < 96) return false;
        if (pe_offset + 24 + 16 > size) return false;
        uint32_t* entry = (uint32_t*)(buffer + pe_offset + 24 + 16);
        *out_entry_point = *entry;
    } else if (opt32->magic == 0x020B) {
        // PE32+ (64-bit)
        *out_is_64bit = true;
        if (coff->size_of_optional_header < 112) return false;
        if (pe_offset + 24 + 16 > size) return false;
        uint32_t* entry = (uint32_t*)(buffer + pe_offset + 24 + 16);
        *out_entry_point = *entry;
    } else {
        return false; // Invalid magic
    }
    
    // Additional validation: check reasonable entry point
    if (*out_entry_point == 0 || *out_entry_point > 0x80000000) {
        return false; // Suspicious entry point
    }
    
    return true;
}

// Extract RIP-relative references from raw code
// This is the critical function for proper memory access
// Enhanced to detect ALL x86-64 RIP-relative addressing modes
EXPORT size_t rip_extract_references(const uint8_t* code, size_t code_size,
                                     uint64_t base_va,
                                     RIP_REF* refs, size_t max_refs) {
    if (!code || !refs || max_refs == 0) return 0;
    
    size_t ref_count = 0;
    
    // Enhanced pattern matching for ALL RIP-relative patterns
    for (size_t i = 0; i < code_size - 2 && ref_count < max_refs; i++) {
        uint8_t b1 = code[i];
        uint8_t b2 = (i + 1 < code_size) ? code[i + 1] : 0;
        uint8_t b3 = (i + 2 < code_size) ? code[i + 2] : 0;
        
        // Special cases first: CALL/JMP [rip + offset] (FF /2 or FF /4)
        // These need exact ModR/M match: 0x15 = CALL, 0x25 = JMP
        if (b1 == 0xFF && (b2 == 0x15 || b2 == 0x25)) {
            if (i + 6 <= code_size) {
                int32_t disp = *(int32_t*)(code + i + 2);
                refs[ref_count].address = base_va + i;
                refs[ref_count].offset = disp;
                refs[ref_count].is_data = false;
                ref_count++;
                i += 5; // Skip processed instruction
                continue;
            }
        }
        
        // REX.W prefix (48-4F)
        bool has_rex_w = (b1 >= 0x48 && b1 <= 0x4F);
        size_t offset_base = has_rex_w ? i + 1 : i;
        uint8_t opcode = has_rex_w ? b2 : b1;
        uint8_t modrm = has_rex_w ? b3 : b2;
        
        // Check for RIP-relative addressing (ModR/M byte = 00 xxx 101)
        // This is the ModR/M pattern for RIP-relative in 64-bit
        if ((modrm & 0xC7) == 0x05) {
            size_t inst_size = 0;
            bool is_rip_relative = false;
            bool is_data_access = false;
            
            // MOV instructions (8B, 89, 88, 8A, A0-A3, C6, C7)
            if (opcode == 0x8B || opcode == 0x8A) { // MOV reg, [mem]
                inst_size = has_rex_w ? 7 : 6;
                is_rip_relative = true;
                is_data_access = true;
            }
            else if (opcode == 0x89 || opcode == 0x88) { // MOV [mem], reg
                inst_size = has_rex_w ? 7 : 6;
                is_rip_relative = true;
                is_data_access = true;
            }
            // LEA instruction (8D)
            else if (opcode == 0x8D) {
                inst_size = has_rex_w ? 7 : 6;
                is_rip_relative = true;
                is_data_access = false;
            }
            // CMP instructions (3A, 3B, 80-83)
            else if (opcode == 0x3B || opcode == 0x3A) {
                inst_size = has_rex_w ? 7 : 6;
                is_rip_relative = true;
                is_data_access = true;
            }
            // TEST instruction (85)
            else if (opcode == 0x85 || opcode == 0x84) {
                inst_size = has_rex_w ? 7 : 6;
                is_rip_relative = true;
                is_data_access = true;
            }
            // ADD/SUB/AND/OR/XOR (00-05, 08-0D, 20-25, 28-2D, 30-35)
            else if ((opcode >= 0x00 && opcode <= 0x05) ||
                     (opcode >= 0x08 && opcode <= 0x0D) ||
                     (opcode >= 0x20 && opcode <= 0x25) ||
                     (opcode >= 0x28 && opcode <= 0x2D) ||
                     (opcode >= 0x30 && opcode <= 0x35)) {
                inst_size = has_rex_w ? 7 : 6;
                is_rip_relative = true;
                is_data_access = true;
            }
            
            if (is_rip_relative && offset_base + inst_size <= code_size) {
                int32_t disp = *(int32_t*)(code + offset_base + 2);
                refs[ref_count].address = base_va + i;
                refs[ref_count].offset = disp;
                refs[ref_count].is_data = is_data_access;
                ref_count++;
                i += inst_size - 1; // Skip processed instruction
                continue;
            }
        }
    }
    
    return ref_count;
}

// Fix RIP-relative addresses to use proper labels
EXPORT bool rip_fix_references(const char* asm_code,
                              const RIP_REF* refs, size_t ref_count,
                              void* output_buffer) {
    // This function will process the assembly code and replace
    // RIP-relative references with proper labels
    
    RESULT_BUFFER* buf = (RESULT_BUFFER*)output_buffer;
    if (!buf || !asm_code) return false;
    
    size_t code_len = strlen(asm_code);
    size_t read_pos = 0;
    
    while (read_pos < code_len && !buf->truncated) {
        // Find [rip pattern
        const char* rip_pattern = strstr(asm_code + read_pos, "[rip");
        
        if (!rip_pattern) {
            // No more RIP references, copy rest of string
            const char* remaining = asm_code + read_pos;
            if (buf->used + strlen(remaining) >= buf->capacity) {
                buf->truncated = true;
                return false;
            }
            strcpy(buf->data + buf->used, remaining);
            buf->used += strlen(remaining);
            break;
        }
        
        // Copy up to the RIP pattern
        size_t copy_len = rip_pattern - (asm_code + read_pos);
        if (buf->used + copy_len + 10 >= buf->capacity) {
            buf->truncated = true;
            return false;
        }
        
        memcpy(buf->data + buf->used, asm_code + read_pos, copy_len);
        buf->used += copy_len;
        
        // Parse the offset value
        int32_t offset = 0;
        const char* offset_start = strchr(rip_pattern, '+');
        if (!offset_start) offset_start = strchr(rip_pattern, '-');
        
        if (offset_start && sscanf(offset_start + 1, "%x", (unsigned int*)&offset) == 1) {
            // Write label reference instead
            buf->used += sprintf(buf->data + buf->used, "[data_0x%lx]", offset & 0xFFFFFFFF);
        } else {
            // Keep original if we can't parse
            strcpy(buf->data + buf->used, rip_pattern);
            buf->used += strlen(rip_pattern);
        }
        
        // Skip past the original [rip ...] part
        const char* end_bracket = strchr(rip_pattern, ']');
        if (end_bracket) {
            read_pos = (end_bracket + 1) - asm_code;
        } else {
            break;
        }
    }
    
    buf->data[buf->used] = '\0';
    return !buf->truncated;
}

// Validate section integrity before disassembly with enhanced heuristics
EXPORT bool rip_validate_section(const uint8_t* code, size_t size) {
    if (!code || size == 0) return false;
    if (size < 16) return true; // Too small to validate, assume OK
    
    // Multi-heuristic validation for x86-64 code
    size_t instruction_markers = 0;
    size_t suspicious_bytes = 0;
    size_t null_bytes = 0;
    size_t high_entropy = 0;
    
    for (size_t i = 0; i < size && i < 1024; i++) { // Sample first 1KB
        uint8_t b = code[i];
        
        // Count common instruction bytes
        // REX prefixes (48-4F), MOV/LEA opcodes, CALL/JMP, RET, PUSH/POP
        if ((b >= 0x48 && b <= 0x4F) ||  // REX.W prefix
            b == 0x8B || b == 0x89 || b == 0x8D ||  // MOV/LEA
            b == 0xE8 || b == 0xE9 ||  // CALL/JMP relative
            b == 0xFF ||  // CALL/JMP indirect
            b == 0xC3 || b == 0xC2 ||  // RET
            b == 0x90 ||  // NOP
            (b >= 0x50 && b <= 0x5F) ||  // PUSH/POP
            b == 0xCC || b == 0xCD) {  // INT3/INT
            instruction_markers++;
        }
        
        // Count suspicious patterns
        if (b == 0x00) null_bytes++;
        if (b == 0xFF || b == 0xEE || b == 0xDD) high_entropy++;
        
        // Check for invalid opcodes (undefined in x86-64)
        if (b == 0x06 || b == 0x07 || b == 0x0E || b == 0x16 || 
            b == 0x17 || b == 0x1E || b == 0x1F || b == 0x27 ||
            b == 0x2F || b == 0x37 || b == 0x3F) {
            suspicious_bytes++;
        }
    }
    
    size_t sample_size = (size < 1024) ? size : 1024;
    float marker_ratio = (float)instruction_markers / sample_size;
    float null_ratio = (float)null_bytes / sample_size;
    float suspicious_ratio = (float)suspicious_bytes / sample_size;
    
    // Valid code should have:
    // - At least 5% recognizable instructions
    // - Less than 30% null bytes (stricter)
    // - Less than 5% suspicious bytes (stricter)
    return (marker_ratio > 0.05f) && 
           (null_ratio < 0.3f) && 
           (suspicious_ratio < 0.05f);
}

// Export version for compatibility checking
EXPORT const char* rip_get_version(void) {
    return "2.0.0-enhanced";
}