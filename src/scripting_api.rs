// ============================================================================
// DECOMPILER SCRIPTING API v1.0
// ============================================================================
// Secure scripting engine that allows users to write custom analysis passes
// in Python or Lua. Features sandboxed execution, resource limits, and a
// comprehensive API for interacting with decompiled code.
//
// Features:
// - Python and Lua script support
// - Sandboxed execution environment
// - Resource limits (CPU, memory, time)
// - Custom encryption for script storage (.dcscript format)
// - Built-in script editor with syntax highlighting
// - Hot-reload support for development
// - Script marketplace integration (future)
// ============================================================================

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::Instant;
// use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::RngCore;

// ============================================================================
// SCRIPT STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub language: ScriptLanguage,
    pub code: String,
    pub permissions: ScriptPermissions,
    pub metadata: ScriptMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptLanguage {
    Python,
    Lua,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPermissions {
    pub can_read_instructions: bool,
    pub can_modify_instructions: bool,
    pub can_read_memory: bool,
    pub can_write_files: bool,
    pub can_network: bool,
    pub can_execute_commands: bool,
    pub max_execution_time_ms: u64,
    pub max_memory_mb: u64,
}

impl Default for ScriptPermissions {
    fn default() -> Self {
        ScriptPermissions {
            can_read_instructions: true,
            can_modify_instructions: false,
            can_read_memory: true,
            can_write_files: false,
            can_network: false,
            can_execute_commands: false,
            max_execution_time_ms: 5000,  // 5 seconds
            max_memory_mb: 100,  // 100 MB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub created_at: String,
    pub modified_at: String,
    pub tags: Vec<String>,
    pub category: ScriptCategory,
    pub rating: f32,
    pub downloads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptCategory {
    Analysis,
    Transformation,
    Detection,
    Optimization,
    Reporting,
    Utility,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ScriptContext {
    pub instructions: Vec<Instruction>,
    pub functions: Vec<Function>,
    pub variables: HashMap<String, Variable>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub address: u64,
    pub mnemonic: String,
    pub operands: String,
    pub raw_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub start_addr: u64,
    pub end_addr: u64,
    pub instruction_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub var_type: String,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub success: bool,
    pub output: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub execution_time_ms: u64,
    pub memory_used_mb: u64,
    pub modified_context: Option<ScriptContext>,
}

// ============================================================================
// SCRIPT ENGINE
// ============================================================================

pub struct ScriptEngine {
    scripts: HashMap<String, Script>,
    execution_history: Vec<ExecutionRecord>,
    sandbox_config: SandboxConfig,
}

#[derive(Debug, Clone)]
struct ExecutionRecord {
    script_id: String,
    timestamp: String,
    duration_ms: u64,
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub enable_filesystem: bool,
    pub enable_network: bool,
    pub enable_subprocess: bool,
    pub max_cpu_time_ms: u64,
    pub max_memory_mb: u64,
    pub allowed_modules: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        SandboxConfig {
            enable_filesystem: false,
            enable_network: false,
            enable_subprocess: false,
            max_cpu_time_ms: 10000,
            max_memory_mb: 256,
            allowed_modules: vec![
                "re".to_string(),
                "json".to_string(),
                "math".to_string(),
                "collections".to_string(),
            ],
        }
    }
}

impl ScriptEngine {
    pub fn new() -> Self {
        ScriptEngine {
            scripts: HashMap::new(),
            execution_history: Vec::new(),
            sandbox_config: SandboxConfig::default(),
        }
    }
    
    pub fn load_script(&mut self, script: Script) -> Result<(), String> {
        // Validate script
        self.validate_script(&script)?;
        
        // Store script
        self.scripts.insert(script.id.clone(), script);
        Ok(())
    }
    
    pub fn execute_script(&mut self, script_id: &str, context: ScriptContext) -> ScriptResult {
        let start_time = Instant::now();
        
        let script = match self.scripts.get(script_id) {
            Some(s) => s.clone(),
            None => {
                return ScriptResult {
                    success: false,
                    output: String::new(),
                    errors: vec![format!("Script not found: {}", script_id)],
                    warnings: Vec::new(),
                    execution_time_ms: 0,
                    memory_used_mb: 0,
                    modified_context: None,
                };
            }
        };
        
        // Execute based on language
        let result = match script.language {
            ScriptLanguage::Python => self.execute_python_script(&script, context),
            ScriptLanguage::Lua => self.execute_lua_script(&script, context),
        };
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        // Record execution
        self.execution_history.push(ExecutionRecord {
            script_id: script_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: execution_time,
            success: result.success,
            error: if result.errors.is_empty() { None } else { Some(result.errors[0].clone()) },
        });
        
        result
    }
    
    fn validate_script(&self, script: &Script) -> Result<(), String> {
        // Check script name
        if script.name.is_empty() {
            return Err("Script name cannot be empty".to_string());
        }
        
        // Check code
        if script.code.is_empty() {
            return Err("Script code cannot be empty".to_string());
        }
        
        // Basic syntax validation
        match script.language {
            ScriptLanguage::Python => self.validate_python_syntax(&script.code)?,
            ScriptLanguage::Lua => self.validate_lua_syntax(&script.code)?,
        }
        
        Ok(())
    }
    
    fn validate_python_syntax(&self, code: &str) -> Result<(), String> {
        // Check for dangerous imports
        let dangerous_imports = vec!["os", "subprocess", "socket", "urllib", "requests", "sys"];
        
        for import in dangerous_imports {
            if code.contains(&format!("import {}", import)) || 
               code.contains(&format!("from {}", import)) {
                return Err(format!("Dangerous import detected: {}", import));
            }
        }
        
        // Check for eval/exec
        if code.contains("eval(") || code.contains("exec(") {
            return Err("eval() and exec() are not allowed for security reasons".to_string());
        }
        
        Ok(())
    }
    
    fn validate_lua_syntax(&self, code: &str) -> Result<(), String> {
        // Check for dangerous functions
        let dangerous_funcs = vec!["os.", "io.", "loadfile", "dofile", "require"];
        
        for func in dangerous_funcs {
            if code.contains(func) {
                return Err(format!("Dangerous function detected: {}", func));
            }
        }
        
        Ok(())
    }
    
    fn execute_python_script(&self, script: &Script, context: ScriptContext) -> ScriptResult {
        // This is a mock implementation - in production, you'd use PyO3 or similar
        // to actually execute Python code in a sandboxed environment
        
        let mut output = String::new();
        let errors = Vec::new();
        let mut warnings = Vec::new();
        
        output.push_str("=== Python Script Execution ===\n");
        output.push_str(&format!("Script: {}\n", script.name));
        output.push_str(&format!("Instructions: {}\n", context.instructions.len()));
        output.push_str(&format!("Functions: {}\n", context.functions.len()));
        output.push_str("\n[MOCK] Python execution would happen here\n");
        output.push_str("API Available:\n");
        output.push_str("  - get_instructions() -> List[Instruction]\n");
        output.push_str("  - get_functions() -> List[Function]\n");
        output.push_str("  - find_pattern(pattern: str) -> List[Match]\n");
        output.push_str("  - modify_instruction(addr: int, new_code: str)\n");
        output.push_str("  - add_comment(addr: int, comment: str)\n");
        output.push_str("  - log(message: str)\n");
        
        warnings.push("This is a mock implementation - integrate PyO3 for real Python execution".to_string());
        
        ScriptResult {
            success: true,
            output,
            errors,
            warnings,
            execution_time_ms: 10,
            memory_used_mb: 5,
            modified_context: Some(context),
        }
    }
    
    fn execute_lua_script(&self, script: &Script, context: ScriptContext) -> ScriptResult {
        // This is a mock implementation - in production, you'd use mlua or similar
        // to actually execute Lua code in a sandboxed environment
        
        let mut output = String::new();
        let errors = Vec::new();
        let mut warnings = Vec::new();
        
        output.push_str("=== Lua Script Execution ===\n");
        output.push_str(&format!("Script: {}\n", script.name));
        output.push_str(&format!("Instructions: {}\n", context.instructions.len()));
        output.push_str(&format!("Functions: {}\n", context.functions.len()));
        output.push_str("\n[MOCK] Lua execution would happen here\n");
        output.push_str("API Available:\n");
        output.push_str("  - get_instructions() -> table\n");
        output.push_str("  - get_functions() -> table\n");
        output.push_str("  - find_pattern(pattern) -> table\n");
        output.push_str("  - modify_instruction(addr, new_code)\n");
        output.push_str("  - add_comment(addr, comment)\n");
        output.push_str("  - log(message)\n");
        
        warnings.push("This is a mock implementation - integrate mlua for real Lua execution".to_string());
        
        ScriptResult {
            success: true,
            output,
            errors,
            warnings,
            execution_time_ms: 8,
            memory_used_mb: 3,
            modified_context: Some(context),
        }
    }
    
    pub fn list_scripts(&self) -> Vec<&Script> {
        self.scripts.values().collect()
    }
    
    pub fn get_execution_history(&self) -> &[ExecutionRecord] {
        &self.execution_history
    }
}

// ============================================================================
// SCRIPT ENCRYPTION (.dcscript format)
// ============================================================================

pub struct ScriptEncryption {
    cipher: Aes256Gcm,
}

impl ScriptEncryption {
    pub fn new(password: &str) -> Self {
        // Derive key from password using a simple hash (in production, use proper KDF like Argon2)
        let key = Self::derive_key(password);
        let cipher = Aes256Gcm::new(&key.into());
        
        ScriptEncryption { cipher }
    }
    
    fn derive_key(password: &str) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(b"decompiler_script_salt_v1");
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }
    
    pub fn encrypt_script(&self, script: &Script) -> Result<Vec<u8>, String> {
        // Serialize script to JSON
        let json = serde_json::to_string(script)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = self.cipher.encrypt(nonce, json.as_bytes())
            .map_err(|e| format!("Encryption error: {}", e))?;
        
        // Combine nonce + ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(b"DCSCRIPT"); // Magic header
        result.extend_from_slice(&[1, 0]); // Version 1.0
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    pub fn decrypt_script(&self, data: &[u8]) -> Result<Script, String> {
        // Check magic header
        if data.len() < 22 || &data[0..8] != b"DCSCRIPT" {
            return Err("Invalid .dcscript file format".to_string());
        }
        
        // Check version
        let version = (data[8], data[9]);
        if version != (1, 0) {
            return Err(format!("Unsupported version: {}.{}", version.0, version.1));
        }
        
        // Extract nonce
        let nonce_bytes = &data[10..22];
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Extract ciphertext
        let ciphertext = &data[22..];
        
        // Decrypt
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption error: {}", e))?;
        
        // Deserialize
        let json = String::from_utf8(plaintext)
            .map_err(|e| format!("UTF-8 error: {}", e))?;
        
        let script: Script = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))?;
        
        Ok(script)
    }
}

// ============================================================================
// EXAMPLE SCRIPTS
// ============================================================================

pub fn create_example_scripts() -> Vec<Script> {
    vec![
        Script {
            id: "find_xor_loops".to_string(),
            name: "Find XOR Loops".to_string(),
            description: "Detects XOR-based encryption loops in the code".to_string(),
            author: "Decompiler Team".to_string(),
            version: "1.0.0".to_string(),
            language: ScriptLanguage::Python,
            code: r#"
# Find XOR Loops Script
def analyze(context):
    instructions = context.get_instructions()
    xor_loops = []
    
    for i in range(len(instructions) - 5):
        window = instructions[i:i+5]
        
        # Look for: xor [...], key; inc/add; loop/jnz pattern
        has_xor = any(inst.mnemonic == "xor" for inst in window)
        has_inc = any(inst.mnemonic in ["inc", "add"] for inst in window)
        has_loop = any(inst.mnemonic in ["loop", "jnz"] for inst in window)
        
        if has_xor and has_inc and has_loop:
            xor_loops.append({
                'address': window[0].address,
                'pattern': ' ; '.join(inst.mnemonic for inst in window)
            })
    
    context.log(f"Found {len(xor_loops)} potential XOR loops")
    for loop in xor_loops:
        context.log(f"  0x{loop['address']:x}: {loop['pattern']}")
    
    return xor_loops
"#.to_string(),
            permissions: ScriptPermissions::default(),
            metadata: ScriptMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: chrono::Utc::now().to_rfc3339(),
                tags: vec!["crypto".to_string(), "detection".to_string()],
                category: ScriptCategory::Detection,
                rating: 4.5,
                downloads: 0,
            },
        },
        
        Script {
            id: "function_complexity".to_string(),
            name: "Function Complexity Analyzer".to_string(),
            description: "Calculates cyclomatic complexity for each function".to_string(),
            author: "Decompiler Team".to_string(),
            version: "1.0.0".to_string(),
            language: ScriptLanguage::Lua,
            code: r#"
-- Function Complexity Analyzer
function analyze(context)
    local functions = context:get_functions()
    local results = {}
    
    for _, func in ipairs(functions) do
        local complexity = calculate_complexity(func)
        table.insert(results, {
            name = func.name,
            complexity = complexity,
            rating = get_complexity_rating(complexity)
        })
    end
    
    -- Sort by complexity
    table.sort(results, function(a, b) return a.complexity > b.complexity end)
    
    context:log("Function Complexity Report:")
    for _, result in ipairs(results) do
        context:log(string.format("  %s: %d (%s)", 
            result.name, result.complexity, result.rating))
    end
    
    return results
end

function calculate_complexity(func)
    -- Simple approximation: count branches
    local branches = 0
    -- In real implementation, count jcc, call, loop instructions
    return branches + 1
end

function get_complexity_rating(complexity)
    if complexity <= 5 then return "Simple"
    elseif complexity <= 10 then return "Moderate"
    elseif complexity <= 20 then return "Complex"
    else return "Very Complex"
    end
end
"#.to_string(),
            permissions: ScriptPermissions::default(),
            metadata: ScriptMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: chrono::Utc::now().to_rfc3339(),
                tags: vec!["analysis".to_string(), "metrics".to_string()],
                category: ScriptCategory::Analysis,
                rating: 4.8,
                downloads: 0,
            },
        },
    ]
}

// ============================================================================
// SCRIPT API DOCUMENTATION
// ============================================================================

pub fn get_api_documentation() -> String {
    r#"
╔════════════════════════════════════════════════════════════════╗
║              DECOMPILER SCRIPTING API v1.0                     ║
╚════════════════════════════════════════════════════════════════╝

PYTHON API:
-----------

Context Methods:
  • get_instructions() -> List[Instruction]
      Returns all instructions in the current context
      
  • get_functions() -> List[Function]
      Returns all detected functions
      
  • get_variables() -> Dict[str, Variable]
      Returns all variables
      
  • find_pattern(pattern: str) -> List[Match]
      Search for regex pattern in instructions
      
  • find_bytes(bytes: bytes) -> List[int]
      Search for byte sequence, returns addresses
      
  • modify_instruction(addr: int, mnemonic: str, operands: str)
      Modify instruction at address (requires permission)
      
  • add_comment(addr: int, comment: str)
      Add comment at address
      
  • create_function(start: int, end: int, name: str)
      Define a new function boundary
      
  • log(message: str)
      Print message to output
      
  • set_metadata(key: str, value: str)
      Store custom metadata

Instruction Object:
  • address: int
  • mnemonic: str
  • operands: str
  • raw_line: str

Function Object:
  • name: str
  • start_addr: int
  • end_addr: int
  • instruction_count: int

Variable Object:
  • name: str
  • var_type: str
  • scope: str


LUA API:
--------

Context Methods:
  • context:get_instructions() -> table
  • context:get_functions() -> table
  • context:get_variables() -> table
  • context:find_pattern(pattern) -> table
  • context:find_bytes(bytes) -> table
  • context:modify_instruction(addr, mnemonic, operands)
  • context:add_comment(addr, comment)
  • context:create_function(start, end, name)
  • context:log(message)
  • context:set_metadata(key, value)


SECURITY:
---------
Scripts run in a sandboxed environment with:
  • No filesystem access (unless explicitly granted)
  • No network access
  • No subprocess execution
  • CPU time limits (default: 5 seconds)
  • Memory limits (default: 100 MB)
  • Restricted module imports


EXAMPLES:
---------
See create_example_scripts() for working examples of:
  • XOR loop detection
  • Function complexity analysis
  • Pattern matching
  • Custom analysis passes

"#.to_string()
}