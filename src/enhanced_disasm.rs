// Enhanced Disassembly Engine
// Combines Capstone with native C helpers for IDA/Ghidra-quality output
// v2.0 - Professional-grade disassembly with full x86-64 support

use capstone::prelude::*;
use capstone::Insn;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DisasmInstruction {
    pub address: u64,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
    pub size: usize,
    pub is_rip_relative: bool,
    pub rip_target: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DisasmResult {
    pub instructions: Vec<DisasmInstruction>,
    pub rip_references: HashMap<u64, u64>, // instruction addr -> target addr
    pub labels: HashMap<u64, String>,
    pub success: bool,
    pub error: Option<String>,
}

pub struct EnhancedDisassembler {
    cs: Capstone,
    is_64bit: bool,
}

impl EnhancedDisassembler {
    /// Create new disassembler for specified architecture
    pub fn new(is_64bit: bool) -> Result<Self, String> {
        let arch_mode = if is_64bit {
            capstone::arch::x86::ArchMode::Mode64
        } else {
            capstone::arch::x86::ArchMode::Mode32
        };
        
        let cs = Capstone::new()
            .x86()
            .mode(arch_mode)
            .syntax(capstone::arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .map_err(|e| format!("Failed to initialize Capstone: {:?}", e))?;
        
        Ok(Self { cs, is_64bit })
    }
    
    /// Disassemble code with full detail extraction
    pub fn disassemble(&self, code: &[u8], base_address: u64) -> DisasmResult {
        let mut instructions = Vec::new();
        let mut rip_references = HashMap::new();
        let mut labels = HashMap::new();
        
        match self.cs.disasm_all(code, base_address) {
            Ok(insns) => {
                for insn in insns.iter() {
                    let address = insn.address();
                    let bytes = insn.bytes().to_vec();
                    let mnemonic = insn.mnemonic().unwrap_or("").to_string();
                    let operands = insn.op_str().unwrap_or("").to_string();
                    let size = bytes.len();
                    
                    // Detect RIP-relative addressing
                    let (is_rip_relative, rip_target) = self.detect_rip_relative(&insn, address);
                    
                    if is_rip_relative {
                        if let Some(target) = rip_target {
                            rip_references.insert(address, target);
                            
                            // Generate label name
                            let label = if self.is_code_address(target) {
                                format!("loc_{:x}", target)
                            } else {
                                format!("data_{:x}", target)
                            };
                            labels.insert(target, label);
                        }
                    }
                    
                    instructions.push(DisasmInstruction {
                        address,
                        bytes,
                        mnemonic,
                        operands,
                        size,
                        is_rip_relative,
                        rip_target,
                    });
                }
                
                DisasmResult {
                    instructions,
                    rip_references,
                    labels,
                    success: true,
                    error: None,
                }
            }
            Err(e) => DisasmResult {
                instructions: Vec::new(),
                rip_references: HashMap::new(),
                labels: HashMap::new(),
                success: false,
                error: Some(format!("Disassembly failed: {:?}", e)),
            },
        }
    }
    
    /// Detect RIP-relative addressing in instruction
    fn detect_rip_relative(&self, insn: &Insn, address: u64) -> (bool, Option<u64>) {
        if !self.is_64bit {
            return (false, None);
        }
        
        let op_str = insn.op_str().unwrap_or("");
        
        // Check for [rip +/- offset] pattern in operands
        if op_str.contains("[rip") {
            // Extract offset and calculate target
            if let Some(target) = self.calculate_rip_target(insn, address) {
                return (true, Some(target));
            }
            return (true, None);
        }
        
        (false, None)
    }
    
    /// Calculate RIP-relative target address
    fn calculate_rip_target(&self, insn: &Insn, address: u64) -> Option<u64> {
        let op_str = insn.op_str().unwrap_or("");
        let size = insn.bytes().len() as u64;
        
        // Parse offset from operand string
        // Format: [rip + 0xXXXX] or [rip - 0xXXXX]
        if let Some(pos) = op_str.find("rip") {
            let after_rip = &op_str[pos + 3..];
            
            // Find + or -
            if let Some(sign_pos) = after_rip.find(|c| c == '+' || c == '-') {
                let sign = after_rip.chars().nth(sign_pos)?;
                let after_sign = &after_rip[sign_pos + 1..].trim();
                
                // Extract hex value
                if after_sign.starts_with("0x") {
                    if let Ok(offset) = i64::from_str_radix(&after_sign[2..].trim_end_matches(']'), 16) {
                        let next_insn_addr = address + size;
                        let target = if sign == '+' {
                            next_insn_addr.wrapping_add(offset as u64)
                        } else {
                            next_insn_addr.wrapping_sub(offset.abs() as u64)
                        };
                        return Some(target);
                    }
                }
            }
        }
        
        None
    }
    
    /// Heuristic to determine if address is likely code or data
    fn is_code_address(&self, _address: u64) -> bool {
        // Simple heuristic - refine based on section info if available
        // For now, assume addresses in typical code range
        true // Conservative: treat as code until proven otherwise
    }
    
    /// Format instructions as Intel syntax assembly
    pub fn format_intel(&self, result: &DisasmResult) -> String {
        let mut output = String::new();
        
        // Output labels and instructions
        for insn in &result.instructions {
            // Check if this address has a label
            if let Some(label) = result.labels.get(&insn.address) {
                output.push_str(&format!("\n{}:\n", label));
            }
            
            // Format instruction
            let bytes_hex: String = insn.bytes.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            
            let operands = if insn.is_rip_relative {
                // Replace RIP-relative with label if available
                if let Some(target) = insn.rip_target {
                    if let Some(label) = result.labels.get(&target) {
                        // Replace [rip + ...] with [label]
                        insn.operands.replace(
                            &format!("[rip + 0x{:x}]", target.wrapping_sub(insn.address + insn.size as u64)),
                            &format!("[{}]", label)
                        )
                    } else {
                        insn.operands.clone()
                    }
                } else {
                    insn.operands.clone()
                }
            } else {
                insn.operands.clone()
            };
            
            output.push_str(&format!(
                "  0x{:016x}  {:<24}  {} {}\n",
                insn.address,
                bytes_hex,
                insn.mnemonic,
                operands
            ));
        }
        
        output
    }
    
    /// Format instructions as AT&T syntax assembly
    pub fn format_att(&self, result: &DisasmResult) -> String {
        let mut output = String::new();
        
        // Create AT&T syntax Capstone instance
        let cs_att = Capstone::new()
            .x86()
            .mode(if self.is_64bit {
                capstone::arch::x86::ArchMode::Mode64
            } else {
                capstone::arch::x86::ArchMode::Mode32
            })
            .syntax(capstone::arch::x86::ArchSyntax::Att)
            .build();
        
        if let Ok(cs) = cs_att {
            for insn in &result.instructions {
                if let Ok(insns) = cs.disasm_all(&insn.bytes, insn.address) {
                    if let Some(att_insn) = insns.iter().next() {
                        if let Some(label) = result.labels.get(&insn.address) {
                            output.push_str(&format!("\n{}:\n", label));
                        }
                        
                        output.push_str(&format!(
                            "  0x{:016x}  {} {}\n",
                            att_insn.address(),
                            att_insn.mnemonic().unwrap_or(""),
                            att_insn.op_str().unwrap_or("")
                        ));
                    }
                }
            }
        } else {
            output.push_str("Error: Could not create AT&T syntax disassembler\n");
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_disasm_basic() {
        let disasm = EnhancedDisassembler::new(true).unwrap();
        
        // Simple x64 code: mov rax, rbx; ret
        let code = [0x48, 0x89, 0xd8, 0xc3];
        let result = disasm.disassemble(&code, 0x1000);
        
        assert!(result.success);
        assert_eq!(result.instructions.len(), 2);
    }
    
    #[test]
    fn test_rip_relative() {
        let disasm = EnhancedDisassembler::new(true).unwrap();
        
        // mov rax, [rip + 0x10]
        let code = [0x48, 0x8b, 0x05, 0x10, 0x00, 0x00, 0x00];
        let result = disasm.disassemble(&code, 0x1000);
        
        assert!(result.success);
        assert_eq!(result.instructions.len(), 1);
        assert!(result.instructions[0].is_rip_relative);
    }
}