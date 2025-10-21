# 📁 Version 3.2.1 Documentation

## Project Folders & Full Assembly Display

**Release Date:** January 2025  
**Status:** ✅ Production Ready

---

## 🎯 What's New in v3.2.1

### Major Features
1. **🗂️ Automatic Project Folder Organization**
   - External EXEs saved to `decompiler/projects/{exe_name}/`
   - Internal EXEs saved in-place (backward compatible)
   - Auto-navigation to project folders after decompilation

2. **📄 Full Assembly Display**
   - Complete disassembly saved in `{name}_full.asm`
   - All executable sections included
   - Original addresses preserved

3. **📊 Complete PE Metadata Export**
   - Image base and entry point
   - Section details
   - Import/Export tables
   - Saved in `{name}_pe_info.txt`

4. **🎯 All Formats Auto-Saved**
   - Pseudo-code (`.pseudo`)
   - C code (`.c`)
   - Rust code (`.rs`)
   - Full assembly (`.asm`)
   - PE metadata (`.txt`)
   - Project README (`.md`)

---

## 📚 Documentation Files

### 🚀 Getting Started
- **[QUICK_START_V3.2.1.md](QUICK_START_V3.2.1.md)**
  - 60-second tutorial
  - Common use cases
  - Keyboard shortcuts
  - Pro tips

### 📖 Complete Guide
- **[README_V3.2.1.md](README_V3.2.1.md)**
  - Complete package overview
  - Documentation guide
  - Technical specifications
  - Version history

### 🗺️ Visual Guide
- **[PROJECT_FOLDER_GUIDE.md](PROJECT_FOLDER_GUIDE.md)**
  - Directory structure diagrams
  - Workflow visualizations
  - File content previews
  - Tips & tricks

### 📝 Changelog
- **[VERSION_3.2.1_CHANGELOG.md](VERSION_3.2.1_CHANGELOG.md)**
  - New features
  - Benefits
  - Technical implementation
  - Migration guide

### 🔧 Technical Details
- **[IMPLEMENTATION_SUMMARY_V3.2.1.md](IMPLEMENTATION_SUMMARY_V3.2.1.md)**
  - Implementation details
  - Code statistics
  - Testing checklist
  - Known issues

---

## 🎯 Quick Navigation

### For New Users
1. Start here: [QUICK_START_V3.2.1.md](QUICK_START_V3.2.1.md)
2. Then read: [PROJECT_FOLDER_GUIDE.md](PROJECT_FOLDER_GUIDE.md)
3. Reference: [README_V3.2.1.md](README_V3.2.1.md)

### For Developers
1. Architecture: [IMPLEMENTATION_SUMMARY_V3.2.1.md](IMPLEMENTATION_SUMMARY_V3.2.1.md)
2. Changes: [VERSION_3.2.1_CHANGELOG.md](VERSION_3.2.1_CHANGELOG.md)
3. Workflow: [PROJECT_FOLDER_GUIDE.md](PROJECT_FOLDER_GUIDE.md)

### For Upgrading Users
1. What's new: [VERSION_3.2.1_CHANGELOG.md](VERSION_3.2.1_CHANGELOG.md)
2. Migration: [VERSION_3.2.1_CHANGELOG.md](VERSION_3.2.1_CHANGELOG.md#migration-guide)
3. Compatibility: [VERSION_3.2.1_CHANGELOG.md](VERSION_3.2.1_CHANGELOG.md#backward-compatibility)

---

## 📂 Project Folder Structure

After decompiling an external EXE, you'll get:

```
decompiler/projects/{exe_name}/
├── {name}_full.asm          # Complete disassembly
├── {name}_decompiled.pseudo # Pseudo-code
├── {name}_decompiled.c      # C code
├── {name}_decompiled.rs     # Rust code
├── {name}_pe_info.txt       # PE metadata
└── README.md                # Project documentation
```

**Learn more:** [PROJECT_FOLDER_GUIDE.md](PROJECT_FOLDER_GUIDE.md)

---

## ✨ Key Benefits

### 🎯 Organization
- No more scattered output files
- Easy to find previous decompilations
- Self-documenting projects

### 📊 Completeness
- All formats generated automatically
- Full assembly for manual analysis
- PE metadata for context

### 🚀 Productivity
- Auto-navigation to results
- No manual folder creation
- Consistent file naming

### 🔄 Compatibility
- Internal EXEs still work as before
- No breaking changes
- Smooth upgrade path

---

## 🔗 Related Documentation

### Previous Versions
- [v3.2 Documentation](../v3.2/) - PE Parsing & Junk Filtering
- [v3.1 Documentation](../v3.1/) - Enhanced Decompilation
- [v3.0 Documentation](../v3.0/) - Multi-Language Support

### General Documentation
- [Architecture](../general/ARCHITECTURE.md)
- [Feature List](../general/DECOMPILER_FEATURES.md)
- [Roadmap](../general/ROADMAP_V3.2_TO_V4.0.md)

---

## 📊 Statistics

- **Documentation:** 2,000+ lines across 5 files
- **Code Added:** ~163 lines in main.rs
- **Build Status:** ✅ 0 errors, 8 warnings (cosmetic)
- **Binary Size:** ~2.5 MB (release)

---

## 🎉 Status

**Version:** 3.2.1  
**Status:** ✅ Production Ready  
**Tested:** Yes  
**Documentation:** Complete  

**Get Started:** [QUICK_START_V3.2.1.md](QUICK_START_V3.2.1.md)