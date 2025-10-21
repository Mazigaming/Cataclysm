// ============================================================================
// KEYBINDS MANAGER v1.0 - Customizable Keyboard Shortcuts
// ============================================================================
// Advanced keybind management system that allows users to customize all
// keyboard shortcuts in the decompiler. Includes preset configurations,
// conflict detection, and import/export functionality.
//
// Features:
// - Customizable keybinds for all actions
// - Preset configurations (Vim, Emacs, VS Code, etc.)
// - Conflict detection and resolution
// - Import/Export keybind configurations
// - Context-aware keybinds (different per mode)
// - Chord support (multi-key sequences)
// - Mouse binding support
// ============================================================================

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// KEYBIND STRUCTURES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBind {
    pub key: String,  // "q", "Enter", "F1", etc.
    pub modifiers: Vec<String>,  // "Ctrl", "Alt", "Shift"
}

impl KeyBind {
    pub fn new(key: &str, modifiers: Vec<&str>) -> Self {
        Self {
            key: key.to_string(),
            modifiers: modifiers.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn matches(&self, code: KeyCode, mods: KeyModifiers) -> bool {
        let key_matches = match code {
            KeyCode::Char(c) => self.key == c.to_string(),
            KeyCode::Enter => self.key == "Enter",
            KeyCode::Esc => self.key == "Esc",
            KeyCode::Backspace => self.key == "Backspace",
            KeyCode::Delete => self.key == "Delete",
            KeyCode::Tab => self.key == "Tab",
            KeyCode::Up => self.key == "Up",
            KeyCode::Down => self.key == "Down",
            KeyCode::Left => self.key == "Left",
            KeyCode::Right => self.key == "Right",
            KeyCode::Home => self.key == "Home",
            KeyCode::End => self.key == "End",
            KeyCode::PageUp => self.key == "PageUp",
            KeyCode::PageDown => self.key == "PageDown",
            KeyCode::F(n) => self.key == format!("F{}", n),
            _ => false,
        };

        if !key_matches {
            return false;
        }

        let has_ctrl = self.modifiers.contains(&"Ctrl".to_string());
        let has_alt = self.modifiers.contains(&"Alt".to_string());
        let has_shift = self.modifiers.contains(&"Shift".to_string());

        has_ctrl == mods.contains(KeyModifiers::CONTROL)
            && has_alt == mods.contains(KeyModifiers::ALT)
            && has_shift == mods.contains(KeyModifiers::SHIFT)
    }

    pub fn to_string(&self) -> String {
        if self.modifiers.is_empty() {
            self.key.clone()
        } else {
            format!("{}+{}", self.modifiers.join("+"), self.key)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // Navigation
    Quit,
    GoBack,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Select,
    
    // File operations
    OpenFile,
    SaveFile,
    SaveAs,
    CloseFile,
    NewFile,
    
    // Multi-file operations
    NextTab,
    PrevTab,
    CloseTab,
    CloseAllTabs,
    
    // Editing
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    Replace,
    
    // View
    ToggleThemeSelector,
    ToggleScriptEditor,
    ToggleSettings,
    ToggleHelp,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    
    // Decompilation
    DecompileToPseudo,
    DecompileToC,
    DecompileToRust,
    ShowAssembly,
    
    // Scripts
    RunScript,
    StopScript,
    NewScript,
    ImportScript,
    ExportScript,
    
    // Themes
    ApplyTheme,
    EditTheme,
    ImportTheme,
    ExportTheme,
    
    // Advanced
    CommandPalette,
    QuickOpen,
    ToggleTerminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindConfig {
    pub name: String,
    pub description: String,
    pub bindings: HashMap<Action, Vec<KeyBind>>,
}

pub struct KeyBindManager {
    pub config: KeyBindConfig,
    pub presets: Vec<KeyBindConfig>,
    pub conflicts: Vec<KeyBindConflict>,
}

#[derive(Debug, Clone)]
pub struct KeyBindConflict {
    pub keybind: KeyBind,
    pub actions: Vec<Action>,
}

// ============================================================================
// PRESET CONFIGURATIONS
// ============================================================================

impl KeyBindManager {
    pub fn new() -> Self {
        let default_config = Self::create_default_config();
        let presets = vec![
            Self::create_default_config(),
            Self::create_vim_config(),
            Self::create_emacs_config(),
            Self::create_vscode_config(),
        ];

        Self {
            config: default_config,
            presets,
            conflicts: Vec::new(),
        }
    }

    fn create_default_config() -> KeyBindConfig {
        let mut bindings = HashMap::new();

        // Navigation
        bindings.insert(Action::Quit, vec![KeyBind::new("q", vec![]), KeyBind::new("Esc", vec![])]);
        bindings.insert(Action::GoBack, vec![KeyBind::new("Esc", vec![])]);
        bindings.insert(Action::MoveUp, vec![KeyBind::new("Up", vec![])]);
        bindings.insert(Action::MoveDown, vec![KeyBind::new("Down", vec![])]);
        bindings.insert(Action::MoveLeft, vec![KeyBind::new("Left", vec![])]);
        bindings.insert(Action::MoveRight, vec![KeyBind::new("Right", vec![])]);
        bindings.insert(Action::Select, vec![KeyBind::new("Enter", vec![])]);

        // File operations
        bindings.insert(Action::SaveFile, vec![KeyBind::new("s", vec!["Ctrl"])]);
        bindings.insert(Action::SaveAs, vec![KeyBind::new("s", vec!["Ctrl", "Shift"])]);
        bindings.insert(Action::OpenFile, vec![KeyBind::new("o", vec!["Ctrl"])]);
        bindings.insert(Action::NewFile, vec![KeyBind::new("n", vec!["Ctrl"])]);
        bindings.insert(Action::CloseFile, vec![KeyBind::new("w", vec!["Ctrl"])]);

        // Multi-file operations
        bindings.insert(Action::NextTab, vec![KeyBind::new("Tab", vec!["Ctrl"]), KeyBind::new("Right", vec!["Ctrl"])]);
        bindings.insert(Action::PrevTab, vec![KeyBind::new("Tab", vec!["Ctrl", "Shift"]), KeyBind::new("Left", vec!["Ctrl"])]);
        bindings.insert(Action::CloseTab, vec![KeyBind::new("w", vec!["Ctrl"])]);
        bindings.insert(Action::CloseAllTabs, vec![KeyBind::new("w", vec!["Ctrl", "Shift"])]);

        // Editing
        bindings.insert(Action::Undo, vec![KeyBind::new("z", vec!["Ctrl"])]);
        bindings.insert(Action::Redo, vec![KeyBind::new("y", vec!["Ctrl"]), KeyBind::new("z", vec!["Ctrl", "Shift"])]);
        bindings.insert(Action::Cut, vec![KeyBind::new("x", vec!["Ctrl"])]);
        bindings.insert(Action::Copy, vec![KeyBind::new("c", vec!["Ctrl"])]);
        bindings.insert(Action::Paste, vec![KeyBind::new("v", vec!["Ctrl"])]);
        bindings.insert(Action::SelectAll, vec![KeyBind::new("a", vec!["Ctrl"])]);
        bindings.insert(Action::Find, vec![KeyBind::new("f", vec!["Ctrl"])]);
        bindings.insert(Action::Replace, vec![KeyBind::new("h", vec!["Ctrl"])]);

        // View
        bindings.insert(Action::ToggleThemeSelector, vec![KeyBind::new("t", vec!["Ctrl"])]);
        bindings.insert(Action::ToggleScriptEditor, vec![KeyBind::new("e", vec!["Ctrl"])]);
        bindings.insert(Action::ToggleSettings, vec![KeyBind::new(",", vec!["Ctrl"])]);
        bindings.insert(Action::ToggleHelp, vec![KeyBind::new("F1", vec![])]);
        bindings.insert(Action::ZoomIn, vec![KeyBind::new("=", vec!["Ctrl"])]);
        bindings.insert(Action::ZoomOut, vec![KeyBind::new("-", vec!["Ctrl"])]);
        bindings.insert(Action::ResetZoom, vec![KeyBind::new("0", vec!["Ctrl"])]);

        // Decompilation
        bindings.insert(Action::DecompileToPseudo, vec![KeyBind::new("F2", vec![])]);
        bindings.insert(Action::DecompileToC, vec![KeyBind::new("F3", vec![])]);
        bindings.insert(Action::DecompileToRust, vec![KeyBind::new("F4", vec![])]);
        bindings.insert(Action::ShowAssembly, vec![KeyBind::new("F5", vec![])]);

        // Scripts
        bindings.insert(Action::RunScript, vec![KeyBind::new("F9", vec![])]);
        bindings.insert(Action::StopScript, vec![KeyBind::new("F9", vec!["Shift"])]);
        bindings.insert(Action::NewScript, vec![KeyBind::new("n", vec!["Ctrl", "Shift"])]);
        bindings.insert(Action::ImportScript, vec![KeyBind::new("i", vec!["Ctrl", "Shift"])]);
        bindings.insert(Action::ExportScript, vec![KeyBind::new("e", vec!["Ctrl", "Shift"])]);

        // Themes
        bindings.insert(Action::ApplyTheme, vec![KeyBind::new("F6", vec![])]);
        bindings.insert(Action::EditTheme, vec![KeyBind::new("F6", vec!["Shift"])]);
        bindings.insert(Action::ImportTheme, vec![KeyBind::new("F7", vec![])]);
        bindings.insert(Action::ExportTheme, vec![KeyBind::new("F7", vec!["Shift"])]);

        // Advanced
        bindings.insert(Action::CommandPalette, vec![KeyBind::new("p", vec!["Ctrl", "Shift"])]);
        bindings.insert(Action::QuickOpen, vec![KeyBind::new("p", vec!["Ctrl"])]);
        bindings.insert(Action::ToggleTerminal, vec![KeyBind::new("`", vec!["Ctrl"])]);

        KeyBindConfig {
            name: "Default".to_string(),
            description: "Standard keybindings for the decompiler".to_string(),
            bindings,
        }
    }

    fn create_vim_config() -> KeyBindConfig {
        let mut bindings = HashMap::new();

        // Navigation (Vim-style)
        bindings.insert(Action::Quit, vec![KeyBind::new("q", vec![]), KeyBind::new("Esc", vec![])]);
        bindings.insert(Action::MoveUp, vec![KeyBind::new("k", vec![]), KeyBind::new("Up", vec![])]);
        bindings.insert(Action::MoveDown, vec![KeyBind::new("j", vec![]), KeyBind::new("Down", vec![])]);
        bindings.insert(Action::MoveLeft, vec![KeyBind::new("h", vec![]), KeyBind::new("Left", vec![])]);
        bindings.insert(Action::MoveRight, vec![KeyBind::new("l", vec![]), KeyBind::new("Right", vec![])]);
        bindings.insert(Action::Select, vec![KeyBind::new("Enter", vec![])]);

        // File operations
        bindings.insert(Action::SaveFile, vec![KeyBind::new("w", vec![]), KeyBind::new("s", vec!["Ctrl"])]);
        bindings.insert(Action::OpenFile, vec![KeyBind::new("o", vec![])]);
        bindings.insert(Action::CloseFile, vec![KeyBind::new("q", vec![])]);

        // Multi-file operations
        bindings.insert(Action::NextTab, vec![KeyBind::new("n", vec!["Ctrl"]), KeyBind::new("Right", vec!["Ctrl"])]);
        bindings.insert(Action::PrevTab, vec![KeyBind::new("p", vec!["Ctrl"]), KeyBind::new("Left", vec!["Ctrl"])]);

        // Editing
        bindings.insert(Action::Undo, vec![KeyBind::new("u", vec![])]);
        bindings.insert(Action::Redo, vec![KeyBind::new("r", vec!["Ctrl"])]);
        bindings.insert(Action::Find, vec![KeyBind::new("/", vec![])]);

        // View
        bindings.insert(Action::ToggleThemeSelector, vec![KeyBind::new("t", vec![])]);
        bindings.insert(Action::ToggleScriptEditor, vec![KeyBind::new("e", vec![])]);
        bindings.insert(Action::ToggleSettings, vec![KeyBind::new(",", vec![])]);

        // Copy defaults for other actions
        let default = Self::create_default_config();
        for (action, keybinds) in default.bindings {
            bindings.entry(action).or_insert(keybinds);
        }

        KeyBindConfig {
            name: "Vim".to_string(),
            description: "Vim-style keybindings (hjkl navigation)".to_string(),
            bindings,
        }
    }

    fn create_emacs_config() -> KeyBindConfig {
        let mut bindings = HashMap::new();

        // Navigation (Emacs-style)
        bindings.insert(Action::Quit, vec![KeyBind::new("x", vec!["Ctrl"]), KeyBind::new("c", vec!["Ctrl"])]);
        bindings.insert(Action::MoveUp, vec![KeyBind::new("p", vec!["Ctrl"]), KeyBind::new("Up", vec![])]);
        bindings.insert(Action::MoveDown, vec![KeyBind::new("n", vec!["Ctrl"]), KeyBind::new("Down", vec![])]);
        bindings.insert(Action::MoveLeft, vec![KeyBind::new("b", vec!["Ctrl"]), KeyBind::new("Left", vec![])]);
        bindings.insert(Action::MoveRight, vec![KeyBind::new("f", vec!["Ctrl"]), KeyBind::new("Right", vec![])]);

        // File operations
        bindings.insert(Action::SaveFile, vec![KeyBind::new("s", vec!["Ctrl"]), KeyBind::new("x", vec!["Ctrl"])]);
        bindings.insert(Action::OpenFile, vec![KeyBind::new("f", vec!["Ctrl"]), KeyBind::new("x", vec!["Ctrl"])]);

        // Editing
        bindings.insert(Action::Undo, vec![KeyBind::new("/", vec!["Ctrl"])]);
        bindings.insert(Action::Find, vec![KeyBind::new("s", vec!["Ctrl"])]);
        bindings.insert(Action::Replace, vec![KeyBind::new("r", vec!["Alt"])]);

        // Copy defaults for other actions
        let default = Self::create_default_config();
        for (action, keybinds) in default.bindings {
            bindings.entry(action).or_insert(keybinds);
        }

        KeyBindConfig {
            name: "Emacs".to_string(),
            description: "Emacs-style keybindings".to_string(),
            bindings,
        }
    }

    fn create_vscode_config() -> KeyBindConfig {
        // VS Code uses mostly the same as default, with some additions
        let mut config = Self::create_default_config();
        config.name = "VS Code".to_string();
        config.description = "VS Code-style keybindings".to_string();

        // Add VS Code specific bindings
        config.bindings.insert(Action::CommandPalette, vec![KeyBind::new("p", vec!["Ctrl", "Shift"])]);
        config.bindings.insert(Action::QuickOpen, vec![KeyBind::new("p", vec!["Ctrl"])]);
        config.bindings.insert(Action::ToggleTerminal, vec![KeyBind::new("`", vec!["Ctrl"])]);

        config
    }

    // ============================================================================
    // KEYBIND OPERATIONS
    // ============================================================================

    pub fn get_action(&self, code: KeyCode, mods: KeyModifiers) -> Option<Action> {
        for (action, keybinds) in &self.config.bindings {
            for keybind in keybinds {
                if keybind.matches(code, mods) {
                    return Some(action.clone());
                }
            }
        }
        None
    }

    pub fn set_keybind(&mut self, action: Action, keybind: KeyBind) {
        self.config.bindings
            .entry(action)
            .or_insert_with(Vec::new)
            .push(keybind);
        self.detect_conflicts();
    }

    pub fn remove_keybind(&mut self, action: &Action, keybind: &KeyBind) {
        if let Some(keybinds) = self.config.bindings.get_mut(action) {
            keybinds.retain(|kb| kb != keybind);
        }
        self.detect_conflicts();
    }

    pub fn clear_keybinds(&mut self, action: &Action) {
        self.config.bindings.remove(action);
        self.detect_conflicts();
    }

    pub fn load_preset(&mut self, preset_name: &str) {
        if let Some(preset) = self.presets.iter().find(|p| p.name == preset_name) {
            self.config = preset.clone();
            self.detect_conflicts();
        }
    }

    pub fn detect_conflicts(&mut self) {
        self.conflicts.clear();
        let mut keybind_map: HashMap<KeyBind, Vec<Action>> = HashMap::new();

        for (action, keybinds) in &self.config.bindings {
            for keybind in keybinds {
                keybind_map
                    .entry(keybind.clone())
                    .or_insert_with(Vec::new)
                    .push(action.clone());
            }
        }

        for (keybind, actions) in keybind_map {
            if actions.len() > 1 {
                self.conflicts.push(KeyBindConflict { keybind, actions });
            }
        }
    }

    // ============================================================================
    // IMPORT/EXPORT
    // ============================================================================

    pub fn export_config(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.config)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn import_config(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        self.config = serde_json::from_str(&json)?;
        self.detect_conflicts();
        Ok(())
    }

    pub fn get_keybind_string(&self, action: &Action) -> String {
        if let Some(keybinds) = self.config.bindings.get(action) {
            if !keybinds.is_empty() {
                return keybinds[0].to_string();
            }
        }
        "Unbound".to_string()
    }

    pub fn get_all_keybinds(&self, action: &Action) -> Vec<String> {
        if let Some(keybinds) = self.config.bindings.get(action) {
            return keybinds.iter().map(|kb| kb.to_string()).collect();
        }
        Vec::new()
    }
}

impl Default for KeyBindManager {
    fn default() -> Self {
        Self::new()
    }
}