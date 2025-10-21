// ============================================================================
// MENU SYSTEM v1.0 - Advanced UI Menu Framework
// ============================================================================
// Comprehensive menu system for managing themes, scripts, settings, and more.
// Features hierarchical menus, search, favorites, and context-aware actions.
//
// Features:
// - Hierarchical menu navigation
// - Search and filter functionality
// - Favorites and recent items
// - Context menus
// - Keyboard and mouse navigation
// - Custom menu items and actions
// - Import/Export wizards
// ============================================================================

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
// use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
// use std::path::PathBuf;

// ============================================================================
// MENU STRUCTURES
// ============================================================================

#[derive(Debug, Clone)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
    pub selected: usize,
    pub search_query: String,
    pub show_search: bool,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub description: String,
    pub action: MenuAction,
    pub icon: String,
    pub enabled: bool,
    pub submenu: Option<Box<Menu>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    // Theme actions
    ApplyTheme(String),
    EditTheme(String),
    CreateTheme,
    ImportTheme,
    ExportTheme(String),
    DeleteTheme(String),
    DuplicateTheme(String),
    
    // Script actions
    OpenScript(String),
    CreateScript(ScriptType),
    ImportScript,
    ExportScript(String),
    DeleteScript(String),
    RunScript(String),
    EditScript(String),
    
    // Settings actions
    OpenSettings(SettingsCategory),
    ChangeKeybinds,
    ResetSettings,
    ImportSettings,
    ExportSettings,
    
    // File actions
    OpenFile,
    SaveFile,
    SaveAs,
    CloseFile,
    
    // Navigation
    GoBack,
    OpenSubmenu,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
    Python,
    Lua,
    Template(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsCategory {
    General,
    Appearance,
    Keybinds,
    Decompiler,
    Scripts,
    Advanced,
}

// ============================================================================
// MENU IMPLEMENTATIONS
// ============================================================================

impl Menu {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
            selected: 0,
            search_query: String::new(),
            show_search: false,
        }
    }

    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn get_selected_action(&self) -> Option<MenuAction> {
        self.items.get(self.selected).map(|item| item.action.clone())
    }

    pub fn filter_items(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        
        let query = self.search_query.to_lowercase();
        self.items.retain(|item| {
            item.label.to_lowercase().contains(&query)
                || item.description.to_lowercase().contains(&query)
        });
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = if self.show_search {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Length(3),  // Search bar
                    Constraint::Min(1),     // Items
                    Constraint::Length(3),  // Help
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Items
                    Constraint::Length(3),  // Help
                ])
                .split(area)
        };

        // Title
        let title_block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(title_block, chunks[0]);

        // Search bar (if enabled)
        let items_chunk = if self.show_search {
            let search_text = format!("Search: {}", self.search_query);
            let search_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));
            let search_para = Paragraph::new(search_text).block(search_block);
            f.render_widget(search_para, chunks[1]);
            chunks[2]
        } else {
            chunks[1]
        };

        // Menu items
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if !item.enabled {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                let icon = if item.submenu.is_some() {
                    " ▶"
                } else {
                    ""
                };

                let content = format!("{} {}{}", item.icon, item.label, icon);
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(list, items_chunk);

        // Help text
        let help_chunk = if self.show_search { chunks[3] } else { chunks[2] };
        let help_text = if self.show_search {
            "Type to search | Esc: Cancel | Enter: Select"
        } else {
            "↑↓: Navigate | Enter: Select | /: Search | Esc: Back"
        };
        
        let help_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        let help_para = Paragraph::new(help_text).block(help_block);
        f.render_widget(help_para, help_chunk);

        // Description panel (if space available)
        if let Some(item) = self.items.get(self.selected) {
            if !item.description.is_empty() && area.height > 20 {
                // Render description in a separate area if there's space
                // This would require splitting the layout differently
            }
        }
    }
}

impl MenuItem {
    pub fn new(label: &str, description: &str, action: MenuAction) -> Self {
        Self {
            label: label.to_string(),
            description: description.to_string(),
            action,
            icon: "•".to_string(),
            enabled: true,
            submenu: None,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    pub fn with_submenu(mut self, submenu: Menu) -> Self {
        self.submenu = Some(Box::new(submenu));
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

// ============================================================================
// PRESET MENUS
// ============================================================================

pub fn create_theme_menu(themes: Vec<String>, current_theme: &str) -> Menu {
    let mut menu = Menu::new("🎨 Theme Manager");

    // Create new theme
    menu.add_item(
        MenuItem::new(
            "Create New Theme",
            "Create a custom theme from scratch",
            MenuAction::CreateTheme,
        )
        .with_icon("✨"),
    );

    // Import theme
    menu.add_item(
        MenuItem::new(
            "Import Theme",
            "Import a theme from a file (.dctheme)",
            MenuAction::ImportTheme,
        )
        .with_icon("📥"),
    );

    menu.add_item(
        MenuItem::new("", "", MenuAction::None).with_icon("─────"),
    );

    // List available themes
    for theme in themes {
        let is_current = theme == current_theme;
        let icon = if is_current { "✓" } else { "○" };
        let label = if is_current {
            format!("{} (Active)", theme)
        } else {
            theme.clone()
        };

        let mut submenu = Menu::new(&format!("Theme: {}", theme));
        submenu.add_item(
            MenuItem::new("Apply Theme", "Switch to this theme", MenuAction::ApplyTheme(theme.clone()))
                .with_icon("✓"),
        );
        submenu.add_item(
            MenuItem::new("Edit Theme", "Customize this theme", MenuAction::EditTheme(theme.clone()))
                .with_icon("✏️"),
        );
        submenu.add_item(
            MenuItem::new("Duplicate Theme", "Create a copy of this theme", MenuAction::DuplicateTheme(theme.clone()))
                .with_icon("📋"),
        );
        submenu.add_item(
            MenuItem::new("Export Theme", "Save theme to file", MenuAction::ExportTheme(theme.clone()))
                .with_icon("📤"),
        );
        if !is_current {
            submenu.add_item(
                MenuItem::new("Delete Theme", "Remove this theme", MenuAction::DeleteTheme(theme.clone()))
                    .with_icon("🗑️"),
            );
        }

        menu.add_item(
            MenuItem::new(&label, &format!("Manage {} theme", theme), MenuAction::None)
                .with_icon(icon)
                .with_submenu(submenu),
        );
    }

    menu
}

pub fn create_script_menu(scripts: Vec<(String, String)>) -> Menu {
    let mut menu = Menu::new("📜 Script Manager");

    // Create new script submenu
    let mut new_script_menu = Menu::new("Create New Script");
    new_script_menu.add_item(
        MenuItem::new(
            "Python Script",
            "Create a new Python analysis script",
            MenuAction::CreateScript(ScriptType::Python),
        )
        .with_icon("🐍"),
    );
    new_script_menu.add_item(
        MenuItem::new(
            "Lua Script",
            "Create a new Lua analysis script",
            MenuAction::CreateScript(ScriptType::Lua),
        )
        .with_icon("🌙"),
    );
    new_script_menu.add_item(
        MenuItem::new("", "", MenuAction::None).with_icon("─────"),
    );
    new_script_menu.add_item(
        MenuItem::new(
            "From Template: Crypto Detector",
            "Create script from crypto detection template",
            MenuAction::CreateScript(ScriptType::Template("crypto_detector".to_string())),
        )
        .with_icon("🔐"),
    );
    new_script_menu.add_item(
        MenuItem::new(
            "From Template: String Extractor",
            "Create script from string extraction template",
            MenuAction::CreateScript(ScriptType::Template("string_extractor".to_string())),
        )
        .with_icon("📝"),
    );
    new_script_menu.add_item(
        MenuItem::new(
            "From Template: API Tracer",
            "Create script from API tracing template",
            MenuAction::CreateScript(ScriptType::Template("api_tracer".to_string())),
        )
        .with_icon("🔍"),
    );

    menu.add_item(
        MenuItem::new(
            "Create New Script",
            "Create a new analysis script",
            MenuAction::None,
        )
        .with_icon("✨")
        .with_submenu(new_script_menu),
    );

    // Import script
    menu.add_item(
        MenuItem::new(
            "Import Script",
            "Import a script from file (.dcscript)",
            MenuAction::ImportScript,
        )
        .with_icon("📥"),
    );

    menu.add_item(
        MenuItem::new("", "", MenuAction::None).with_icon("─────"),
    );

    // List available scripts
    for (name, lang) in scripts {
        let icon = match lang.as_str() {
            "Python" => "🐍",
            "Lua" => "🌙",
            _ => "📜",
        };

        let mut submenu = Menu::new(&format!("Script: {}", name));
        submenu.add_item(
            MenuItem::new("Run Script", "Execute this script", MenuAction::RunScript(name.clone()))
                .with_icon("▶️"),
        );
        submenu.add_item(
            MenuItem::new("Edit Script", "Open in script editor", MenuAction::EditScript(name.clone()))
                .with_icon("✏️"),
        );
        submenu.add_item(
            MenuItem::new("Export Script", "Save script to file", MenuAction::ExportScript(name.clone()))
                .with_icon("📤"),
        );
        submenu.add_item(
            MenuItem::new("Delete Script", "Remove this script", MenuAction::DeleteScript(name.clone()))
                .with_icon("🗑️"),
        );

        menu.add_item(
            MenuItem::new(&name, &format!("{} script", lang), MenuAction::None)
                .with_icon(icon)
                .with_submenu(submenu),
        );
    }

    menu
}

pub fn create_settings_menu() -> Menu {
    let mut menu = Menu::new("⚙️ Settings");

    // General settings
    let mut general_menu = Menu::new("General Settings");
    general_menu.add_item(
        MenuItem::new(
            "Auto-save",
            "Automatically save files on changes",
            MenuAction::OpenSettings(SettingsCategory::General),
        )
        .with_icon("💾"),
    );
    general_menu.add_item(
        MenuItem::new(
            "Project Folders",
            "Configure project folder behavior",
            MenuAction::OpenSettings(SettingsCategory::General),
        )
        .with_icon("📁"),
    );
    general_menu.add_item(
        MenuItem::new(
            "File Associations",
            "Configure file type associations",
            MenuAction::OpenSettings(SettingsCategory::General),
        )
        .with_icon("🔗"),
    );

    menu.add_item(
        MenuItem::new("General", "General application settings", MenuAction::None)
            .with_icon("⚙️")
            .with_submenu(general_menu),
    );

    // Appearance settings
    let mut appearance_menu = Menu::new("Appearance Settings");
    appearance_menu.add_item(
        MenuItem::new(
            "Theme",
            "Change color theme",
            MenuAction::OpenSettings(SettingsCategory::Appearance),
        )
        .with_icon("🎨"),
    );
    appearance_menu.add_item(
        MenuItem::new(
            "Font Size",
            "Adjust editor font size",
            MenuAction::OpenSettings(SettingsCategory::Appearance),
        )
        .with_icon("🔤"),
    );
    appearance_menu.add_item(
        MenuItem::new(
            "Line Numbers",
            "Show/hide line numbers",
            MenuAction::OpenSettings(SettingsCategory::Appearance),
        )
        .with_icon("🔢"),
    );

    menu.add_item(
        MenuItem::new("Appearance", "Visual settings", MenuAction::None)
            .with_icon("🎨")
            .with_submenu(appearance_menu),
    );

    // Keybinds
    menu.add_item(
        MenuItem::new(
            "Keybinds",
            "Customize keyboard shortcuts",
            MenuAction::ChangeKeybinds,
        )
        .with_icon("⌨️"),
    );

    // Decompiler settings
    let mut decompiler_menu = Menu::new("Decompiler Settings");
    decompiler_menu.add_item(
        MenuItem::new(
            "Anti-Obfuscation",
            "Configure obfuscation detection",
            MenuAction::OpenSettings(SettingsCategory::Decompiler),
        )
        .with_icon("🛡️"),
    );
    decompiler_menu.add_item(
        MenuItem::new(
            "Crypto Detection",
            "Configure cryptographic algorithm detection",
            MenuAction::OpenSettings(SettingsCategory::Decompiler),
        )
        .with_icon("🔐"),
    );
    decompiler_menu.add_item(
        MenuItem::new(
            "Output Format",
            "Configure decompilation output",
            MenuAction::OpenSettings(SettingsCategory::Decompiler),
        )
        .with_icon("📄"),
    );

    menu.add_item(
        MenuItem::new("Decompiler", "Decompilation settings", MenuAction::None)
            .with_icon("🔧")
            .with_submenu(decompiler_menu),
    );

    // Scripts settings
    let mut scripts_menu = Menu::new("Scripts Settings");
    scripts_menu.add_item(
        MenuItem::new(
            "Script Permissions",
            "Configure script sandbox permissions",
            MenuAction::OpenSettings(SettingsCategory::Scripts),
        )
        .with_icon("🔒"),
    );
    scripts_menu.add_item(
        MenuItem::new(
            "Auto-run Scripts",
            "Configure scripts to run automatically",
            MenuAction::OpenSettings(SettingsCategory::Scripts),
        )
        .with_icon("⚡"),
    );

    menu.add_item(
        MenuItem::new("Scripts", "Script execution settings", MenuAction::None)
            .with_icon("📜")
            .with_submenu(scripts_menu),
    );

    menu.add_item(
        MenuItem::new("", "", MenuAction::None).with_icon("─────"),
    );

    // Import/Export
    menu.add_item(
        MenuItem::new(
            "Import Settings",
            "Import settings from file",
            MenuAction::ImportSettings,
        )
        .with_icon("📥"),
    );
    menu.add_item(
        MenuItem::new(
            "Export Settings",
            "Export settings to file",
            MenuAction::ExportSettings,
        )
        .with_icon("📤"),
    );
    menu.add_item(
        MenuItem::new(
            "Reset to Defaults",
            "Reset all settings to default values",
            MenuAction::ResetSettings,
        )
        .with_icon("🔄"),
    );

    menu
}

pub fn create_keybinds_menu(keybind_manager: &crate::keybinds::KeyBindManager) -> Menu {
    let mut menu = Menu::new("⌨️ Keybinds Configuration");

    // Preset configurations
    menu.add_item(
        MenuItem::new(
            "Load Preset: Default",
            "Standard keybindings",
            MenuAction::OpenSettings(SettingsCategory::Keybinds),
        )
        .with_icon("📋"),
    );
    menu.add_item(
        MenuItem::new(
            "Load Preset: Vim",
            "Vim-style keybindings (hjkl navigation)",
            MenuAction::OpenSettings(SettingsCategory::Keybinds),
        )
        .with_icon("📋"),
    );
    menu.add_item(
        MenuItem::new(
            "Load Preset: Emacs",
            "Emacs-style keybindings",
            MenuAction::OpenSettings(SettingsCategory::Keybinds),
        )
        .with_icon("📋"),
    );
    menu.add_item(
        MenuItem::new(
            "Load Preset: VS Code",
            "VS Code-style keybindings",
            MenuAction::OpenSettings(SettingsCategory::Keybinds),
        )
        .with_icon("📋"),
    );

    menu.add_item(
        MenuItem::new("", "", MenuAction::None).with_icon("─────"),
    );

    // Import/Export
    menu.add_item(
        MenuItem::new(
            "Import Keybinds",
            "Import keybinds from file",
            MenuAction::ImportSettings,
        )
        .with_icon("📥"),
    );
    menu.add_item(
        MenuItem::new(
            "Export Keybinds",
            "Export keybinds to file",
            MenuAction::ExportSettings,
        )
        .with_icon("📤"),
    );

    menu.add_item(
        MenuItem::new("", "", MenuAction::None).with_icon("─────"),
    );

    // Show conflicts if any
    if !keybind_manager.conflicts.is_empty() {
        menu.add_item(
            MenuItem::new(
                &format!("⚠️ {} Conflicts Detected", keybind_manager.conflicts.len()),
                "View and resolve keybind conflicts",
                MenuAction::OpenSettings(SettingsCategory::Keybinds),
            )
            .with_icon("⚠️"),
        );
    }

    menu
}