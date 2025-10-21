# 🗺️ Decompiler Roadmap: v3.2 → v4.0

## Current Status: v3.2

✅ **Completed:**
- Phase 1: PE Parsing & IAT Resolution
- Phase 3: Junk Instruction Filtering
- Multi-file navigation (v3.1)
- Control flow recovery
- Type inference (basic)

⏳ **In Progress:**
- None

🔮 **Planned:**
- Phase 2: Improved Function Discovery
- Phase 4: CFG Improvements
- Phase 5: Type & Calling Convention Recovery
- Phase 6: Output Polish

---

## Phase 2: Improved Function Discovery

### Goal
Better detection of function boundaries, handling overlapping code and multi-entry functions.

### Tasks

#### 2.1: Hybrid Function Discovery
- [ ] Start from PE entry point
- [ ] Follow all CALL instructions recursively
- [ ] Use PE exports as function starts
- [ ] Detect prologues in unvisited regions
- [ ] Build call graph

**Estimated Effort:** 4-6 hours
**Impact:** High - Fixes most function boundary issues

#### 2.2: Multi-Entry Function Detection
- [ ] Detect when code jumps into middle of function
- [ ] Split functions at alternate entry points
- [ ] Mark as "thunk" or "multi-entry"
- [ ] Handle tail calls

**Estimated Effort:** 2-3 hours
**Impact:** Medium - Handles unusual code patterns

#### 2.3: Better Epilogue Detection
- [ ] Detect `ret` with stack cleanup
- [ ] Detect `jmp` tail calls
- [ ] Detect exception handlers
- [ ] Handle non-standard returns

**Estimated Effort:** 2-3 hours
**Impact:** Medium - More accurate function ends

### Implementation Plan

```rust
// New structures
struct CallGraph {
    nodes: HashMap<u64, CallGraphNode>,
    edges: Vec<(u64, u64)>,
}

struct CallGraphNode {
    address: u64,
    name: String,
    is_export: bool,
    is_import: bool,
    callers: Vec<u64>,
    callees: Vec<u64>,
}

// New functions
fn build_call_graph(instructions: &[Instruction], pe_info: &PEInfo) -> CallGraph;
fn recursive_descent_disasm(start: u64, instructions: &[Instruction]) -> Vec<u64>;
fn detect_multi_entry_functions(functions: &[Function]) -> Vec<Function>;
fn improve_function_boundaries(functions: &mut Vec<Function>, call_graph: &CallGraph);
```

### Expected Improvements

**Before:**
```c
void func_401000() {
    // Contains multiple prologues
    push ebp
    mov ebp, esp
    // ... code ...
    ret
    // More code (actually separate function)
    push ebp
    mov ebp, esp
    // ... code ...
    ret
}
```

**After:**
```c
void func_401000() {
    push ebp
    mov ebp, esp
    // ... code ...
    ret
}

void func_401050() {  // Split correctly
    push ebp
    mov ebp, esp
    // ... code ...
    ret
}
```

---

## Phase 4: CFG Improvements

### Goal
Better control flow graph reconstruction, dead code elimination, and loop detection.

### Tasks

#### 4.1: Basic Block Merging
- [ ] Merge trivial blocks (single jump)
- [ ] Remove empty blocks
- [ ] Collapse fallthrough chains
- [ ] Simplify unconditional jumps

**Estimated Effort:** 3-4 hours
**Impact:** High - Cleaner control flow

#### 4.2: Dead Code Elimination
- [ ] Compute reachability from entry point
- [ ] Mark unreachable blocks
- [ ] Remove or comment out dead code
- [ ] Detect and remove dead stores

**Estimated Effort:** 2-3 hours
**Impact:** Medium - Smaller output

#### 4.3: Loop Detection
- [ ] Detect natural loops (back edges)
- [ ] Identify loop headers
- [ ] Detect while/do-while/for patterns
- [ ] Reconstruct loop conditions

**Estimated Effort:** 4-5 hours
**Impact:** High - Much more readable

#### 4.4: Switch Statement Detection
- [ ] Detect jump tables
- [ ] Extract case values
- [ ] Reconstruct switch statement
- [ ] Handle sparse vs dense tables

**Estimated Effort:** 3-4 hours
**Impact:** Medium - Better readability

### Implementation Plan

```rust
// New structures
struct ControlFlowGraph {
    blocks: HashMap<u64, BasicBlock>,
    edges: Vec<CFGEdge>,
    dominators: HashMap<u64, HashSet<u64>>,
    loops: Vec<Loop>,
}

struct CFGEdge {
    from: u64,
    to: u64,
    edge_type: EdgeType,
}

enum EdgeType {
    Fallthrough,
    ConditionalTrue,
    ConditionalFalse,
    Unconditional,
    Call,
    Return,
}

struct Loop {
    header: u64,
    body: Vec<u64>,
    back_edges: Vec<u64>,
    loop_type: LoopType,
}

enum LoopType {
    While,
    DoWhile,
    For,
    Infinite,
}

// New functions
fn build_cfg(function: &Function) -> ControlFlowGraph;
fn compute_dominators(cfg: &ControlFlowGraph) -> HashMap<u64, HashSet<u64>>;
fn detect_loops(cfg: &ControlFlowGraph) -> Vec<Loop>;
fn detect_switch_tables(cfg: &ControlFlowGraph) -> Vec<SwitchStatement>;
fn eliminate_dead_code(cfg: &mut ControlFlowGraph);
fn merge_basic_blocks(cfg: &mut ControlFlowGraph);
```

### Expected Improvements

**Before:**
```c
void func_401000() {
    int i = 0;
label_1:
    if (i >= 10) goto label_2;
    printf("%d\n", i);
    i++;
    goto label_1;
label_2:
    return;
}
```

**After:**
```c
void func_401000() {
    for (int i = 0; i < 10; i++) {
        printf("%d\n", i);
    }
}
```

---

## Phase 5: Type & Calling Convention Recovery

### Goal
Infer types, detect calling conventions, and reconstruct struct definitions.

### Tasks

#### 5.1: Calling Convention Detection
- [ ] Detect cdecl (stack cleanup by caller)
- [ ] Detect stdcall (stack cleanup by callee)
- [ ] Detect fastcall (first 2 args in ECX/EDX)
- [ ] Detect x64 calling convention
- [ ] Infer parameter count

**Estimated Effort:** 4-5 hours
**Impact:** High - Correct function signatures

#### 5.2: Type Inference
- [ ] Track register types through data flow
- [ ] Detect pointer vs integer
- [ ] Detect signed vs unsigned
- [ ] Detect float vs integer
- [ ] Propagate types through operations

**Estimated Effort:** 5-6 hours
**Impact:** High - More accurate code

#### 5.3: Struct Detection
- [ ] Cluster memory accesses by base register
- [ ] Group offsets into struct fields
- [ ] Infer field types
- [ ] Generate struct definitions
- [ ] Name structs meaningfully

**Estimated Effort:** 4-5 hours
**Impact:** High - Much more readable

#### 5.4: String Detection
- [ ] Scan .rdata section for strings
- [ ] Detect ASCII vs Unicode
- [ ] Find string references in code
- [ ] Replace addresses with string literals

**Estimated Effort:** 2-3 hours
**Impact:** Medium - Better readability

### Implementation Plan

```rust
// New structures
struct CallingConvention {
    name: String,
    param_registers: Vec<String>,
    return_register: String,
    stack_cleanup: StackCleanup,
}

enum StackCleanup {
    Caller,
    Callee,
}

struct TypeInference {
    register_types: HashMap<String, VarType>,
    memory_types: HashMap<u64, VarType>,
    confidence: HashMap<String, f32>,
}

struct StructCandidate {
    name: String,
    fields: Vec<StructField>,
    confidence: f32,
    usage_count: usize,
}

// New functions
fn detect_calling_convention(function: &Function) -> CallingConvention;
fn infer_types(function: &Function) -> TypeInference;
fn detect_structs(functions: &[Function]) -> Vec<StructCandidate>;
fn extract_strings(pe_info: &PEInfo) -> Vec<StringLiteral>;
fn propagate_types(function: &mut Function, type_info: &TypeInference);
```

### Expected Improvements

**Before:**
```c
void func_401000(int arg1, int arg2) {
    int eax = *(int*)(arg1 + 0);
    int ebx = *(int*)(arg1 + 4);
    int ecx = *(int*)(arg1 + 8);
    eax = eax + ebx + ecx;
}
```

**After:**
```c
struct Point {
    int x;      // offset 0
    int y;      // offset 4
    int z;      // offset 8
};

int func_401000(struct Point* p, int unused) {
    return p->x + p->y + p->z;
}
```

---

## Phase 6: Output Polish

### Goal
Make output more professional and user-friendly.

### Tasks

#### 6.1: Confidence Scores
- [ ] Assign confidence to each reconstruction
- [ ] Show confidence in comments
- [ ] Highlight uncertain code
- [ ] Provide alternative interpretations

**Estimated Effort:** 2-3 hours
**Impact:** Low - Nice to have

#### 6.2: Better Variable Naming
- [ ] Use meaningful names for known patterns
- [ ] Name loop counters as i, j, k
- [ ] Name pointers with _ptr suffix
- [ ] Name booleans with is_/has_ prefix

**Estimated Effort:** 2-3 hours
**Impact:** Medium - Better readability

#### 6.3: Annotated Comments
- [ ] Show original assembly in comments
- [ ] Explain complex operations
- [ ] Mark auto-generated code
- [ ] Add TODO comments for uncertain code

**Estimated Effort:** 2-3 hours
**Impact:** Medium - Helps understanding

#### 6.4: Side-by-Side View
- [ ] Add option for assembly + decompiled view
- [ ] Align assembly with source lines
- [ ] Highlight corresponding lines
- [ ] Add cross-references

**Estimated Effort:** 4-5 hours
**Impact:** Medium - Great for learning

### Implementation Plan

```rust
// New structures
struct ConfidenceScore {
    value: f32,  // 0.0 to 1.0
    reason: String,
}

struct AnnotatedLine {
    source_code: String,
    assembly: Vec<String>,
    confidence: ConfidenceScore,
    comments: Vec<String>,
}

// New functions
fn assign_confidence_scores(function: &Function) -> HashMap<usize, ConfidenceScore>;
fn generate_meaningful_names(function: &mut Function);
fn add_explanatory_comments(function: &mut Function);
fn generate_side_by_side_view(function: &Function) -> Vec<AnnotatedLine>;
```

### Expected Improvements

**Before:**
```c
void func_401000() {
    int var_1 = 0;
    int var_2 = 10;
    while (var_1 < var_2) {
        var_1++;
    }
}
```

**After:**
```c
void func_401000() {
    // Confidence: 85% - Loop pattern detected
    int counter = 0;        // 0x401000: xor eax, eax
    int limit = 10;         // 0x401002: mov ebx, 0xa
    
    // Loop: while (counter < limit)
    while (counter < limit) {  // 0x401007: cmp eax, ebx
        counter++;              // 0x401009: inc eax
    }
    // 0x40100b: ret
}
```

---

## Timeline Estimate

### Phase 2: Improved Function Discovery
- **Duration:** 2-3 days
- **Complexity:** Medium-High
- **Dependencies:** Phase 1 (PE parsing)

### Phase 4: CFG Improvements
- **Duration:** 3-4 days
- **Complexity:** High
- **Dependencies:** Phase 2 (better functions)

### Phase 5: Type & Calling Convention Recovery
- **Duration:** 4-5 days
- **Complexity:** Very High
- **Dependencies:** Phase 4 (better CFG)

### Phase 6: Output Polish
- **Duration:** 2-3 days
- **Complexity:** Low-Medium
- **Dependencies:** Phase 5 (better types)

### Total Estimated Time
- **Minimum:** 11 days
- **Maximum:** 15 days
- **Average:** 13 days

---

## Priority Ranking

### High Priority (Must Have)
1. ✅ Phase 1: PE Parsing (DONE)
2. ✅ Phase 3: Junk Filtering (DONE)
3. ⏳ Phase 2: Function Discovery
4. ⏳ Phase 5: Type Inference

### Medium Priority (Should Have)
5. ⏳ Phase 4: CFG Improvements
6. ⏳ Phase 5: Calling Convention Detection
7. ⏳ Phase 5: Struct Detection

### Low Priority (Nice to Have)
8. ⏳ Phase 6: Confidence Scores
9. ⏳ Phase 6: Better Naming
10. ⏳ Phase 6: Side-by-Side View

---

## Version Milestones

### v3.2 (Current) ✅
- PE parsing
- Junk filtering
- Multi-file navigation

### v3.5 (Next Minor)
- Improved function discovery
- Basic CFG improvements
- Import resolution in output

### v3.8 (Following Minor)
- Loop detection
- Switch statement reconstruction
- Dead code elimination

### v4.0 (Major Release)
- Full type inference
- Calling convention detection
- Struct detection
- String extraction
- Professional output quality

---

## Testing Strategy

### Unit Tests
- [ ] PE parsing with various executables
- [ ] Junk instruction detection
- [ ] Function boundary detection
- [ ] CFG construction
- [ ] Type inference

### Integration Tests
- [ ] Simple console app
- [ ] Complex GUI app
- [ ] DLL with exports
- [ ] Obfuscated binary
- [ ] Large application (>1MB)

### Regression Tests
- [ ] Ensure v3.1 features still work
- [ ] Backward compatibility
- [ ] Performance benchmarks

---

## Success Metrics

### Code Quality
- **Target:** 90% of functions correctly identified
- **Current:** ~70%

### Output Readability
- **Target:** 9/10 subjective score
- **Current:** 8/10

### Junk Reduction
- **Target:** 95% of junk filtered
- **Current:** 90%

### Type Accuracy
- **Target:** 80% of types correctly inferred
- **Current:** 40%

### Performance
- **Target:** <5 seconds for 1MB binary
- **Current:** ~3 seconds

---

## Community Feedback Integration

### Requested Features
1. ✅ Multi-file output (v3.1)
2. ✅ Junk filtering (v3.2)
3. ⏳ Import resolution
4. ⏳ Better function names
5. ⏳ Struct detection

### Known Issues
1. ⚠️ Overlapping functions not handled
2. ⚠️ Some prologues not detected
3. ⚠️ Absolute addresses not resolved
4. ⚠️ No loop reconstruction
5. ⚠️ No struct detection

---

## Resources Needed

### Development
- Time: 11-15 days
- Skills: Rust, PE format, assembly, compiler theory

### Testing
- Various test binaries (simple to complex)
- Comparison with other decompilers (IDA, Ghidra)
- User feedback

### Documentation
- Update user guides
- Add examples
- Create tutorials
- Write API docs

---

## Risk Assessment

### High Risk
- **Type inference complexity** - May not be 100% accurate
- **CFG reconstruction** - Complex control flow is hard
- **Performance** - Large binaries may be slow

### Medium Risk
- **Struct detection** - Heuristics may produce false positives
- **Calling convention** - Mixed conventions in one binary
- **Testing coverage** - Hard to test all edge cases

### Low Risk
- **PE parsing** - Well-defined format
- **Junk filtering** - Simple patterns
- **Output formatting** - Straightforward

---

## Conclusion

**Current Status:** v3.2 with Phase 1 & 3 complete

**Next Steps:**
1. Implement Phase 2 (Function Discovery)
2. Test with real-world binaries
3. Gather user feedback
4. Implement Phase 4 (CFG Improvements)
5. Continue toward v4.0

**Target:** v4.0 release in 2-3 weeks with all phases complete

---

**Status:** 📋 **ROADMAP DEFINED**
**Progress:** 40% complete (2/5 phases)
**Next Milestone:** v3.5 with Phase 2 complete