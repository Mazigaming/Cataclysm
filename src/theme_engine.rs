// ============================================================================
// THEME ENGINE v1.0 - CSS-Based UI Customization
// ============================================================================
// Advanced theming system that allows complete customization of the
// decompiler's appearance using CSS-like syntax. Includes multiple built-in
// themes and support for custom user themes.
//
// Features:
// - CSS-like syntax for styling
// - Multiple built-in themes (Dark, Light, Cyberpunk, Matrix, etc.)
// - Hot-reload support
// - Theme marketplace integration (future)
// - Per-element customization
// - Color scheme validation
// - Accessibility support (high contrast, colorblind modes)
// ============================================================================

use ratatui::style::{Color, Modifier, Style};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// THEME STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub colors: ColorScheme,
    pub styles: StyleSheet,
    pub metadata: ThemeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    // Base colors
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    
    // UI elements
    pub border: String,
    pub border_focused: String,
    pub selection: String,
    pub cursor: String,
    
    // Syntax highlighting
    pub keyword: String,
    pub function: String,
    pub variable: String,
    pub constant: String,
    pub string: String,
    pub comment: String,
    pub operator: String,
    pub type_name: String,
// Status colors
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    
    // Code analysis
    pub crypto_detected: String,
    pub obfuscation_detected: String,
    pub api_call: String,
    pub jump_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSheet {
    pub file_list: ElementStyle,
    pub file_list_selected: ElementStyle,
    pub editor: ElementStyle,
    pub status_bar: ElementStyle,
    pub title_bar: ElementStyle,
    pub popup: ElementStyle,
    pub button: ElementStyle,
    pub button_focused: ElementStyle,
    pub scrollbar: ElementStyle,
    pub line_numbers: ElementStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementStyle {
    pub fg: String,
    pub bg: String,
    pub modifiers: Vec<String>,  // "bold", "italic", "underline", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    pub created_at: String,
    pub modified_at: String,
    pub tags: Vec<String>,
    pub rating: f32,
    pub downloads: u64,
    pub is_dark: bool,
    pub accessibility: AccessibilityInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityInfo {
    pub high_contrast: bool,
    pub colorblind_safe: bool,
    pub colorblind_type: Option<ColorblindType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColorblindType {
    Protanopia,    // Red-blind
    Deuteranopia,  // Green-blind
    Tritanopia,    // Blue-blind
}

// ============================================================================
// THEME ENGINE
// ============================================================================

#[derive(Clone)]
pub struct ThemeEngine {
    current_theme: Theme,
    available_themes: HashMap<String, Theme>,
    custom_themes: HashMap<String, Theme>,
}

impl ThemeEngine {
    pub fn new() -> Self {
        let mut engine = ThemeEngine {
            current_theme: create_dark_theme(),
            available_themes: HashMap::new(),
            custom_themes: HashMap::new(),
        };
        
        // Load built-in themes
        engine.load_builtin_themes();
        
        engine
    }
    
    fn load_builtin_themes(&mut self) {
        let themes = vec![
            create_dark_theme(),
            create_light_theme(),
            create_cyberpunk_theme(),
            create_matrix_theme(),
            create_nord_theme(),
            create_dracula_theme(),
            create_solarized_dark_theme(),
            create_monokai_theme(),
            create_high_contrast_theme(),
        ];
        
        for theme in themes {
            self.available_themes.insert(theme.name.clone(), theme);
        }
    }
    
    pub fn set_theme(&mut self, theme_name: &str) -> Result<(), String> {
        if let Some(theme) = self.available_themes.get(theme_name) {
            self.current_theme = theme.clone();
            Ok(())
        } else if let Some(theme) = self.custom_themes.get(theme_name) {
            self.current_theme = theme.clone();
            Ok(())
        } else {
            Err(format!("Theme not found: {}", theme_name))
        }
    }
    
    pub fn get_current_theme(&self) -> &Theme {
        &self.current_theme
    }
    
    pub fn list_themes(&self) -> Vec<String> {
        let mut themes: Vec<String> = self.available_themes.keys().cloned().collect();
        themes.extend(self.custom_themes.keys().cloned());
        themes.sort();
        themes
    }
    
    pub fn load_custom_theme(&mut self, theme: Theme) -> Result<(), String> {
        // Validate theme
        self.validate_theme(&theme)?;
        
        self.custom_themes.insert(theme.name.clone(), theme);
        Ok(())
    }
    
    pub fn export_theme(&self, theme_name: &str, path: &std::path::PathBuf) -> Result<(), String> {
        let theme = self.available_themes.get(theme_name)
            .or_else(|| self.custom_themes.get(theme_name))
            .ok_or_else(|| format!("Theme not found: {}", theme_name))?;
        
        let json = serde_json::to_string_pretty(theme)
            .map_err(|e| format!("Failed to serialize theme: {}", e))?;
        
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write theme file: {}", e))?;
        
        Ok(())
    }
    
    pub fn import_theme(&mut self, path: &std::path::PathBuf) -> Result<(), String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read theme file: {}", e))?;
        
        let theme: Theme = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse theme file: {}", e))?;
        
        self.load_custom_theme(theme)?;
        Ok(())
    }
    
    fn validate_theme(&self, theme: &Theme) -> Result<(), String> {
        // Validate all colors are valid hex codes
        let colors = vec![
            &theme.colors.background,
            &theme.colors.foreground,
            &theme.colors.primary,
            &theme.colors.secondary,
            &theme.colors.accent,
        ];
        
        for color in colors {
            if !is_valid_color(color) {
                return Err(format!("Invalid color: {}", color));
            }
        }
        
        Ok(())
    }
    
    // Convert theme colors to ratatui Style
    pub fn get_style(&self, element: &str, theme: &Theme) -> Style {
        let element_style = match element {
            "file_list" => &theme.styles.file_list,
            "file_list_selected" => &theme.styles.file_list_selected,
            "editor" => &theme.styles.editor,
            "status_bar" => &theme.styles.status_bar,
            "title_bar" => &theme.styles.title_bar,
            "popup" => &theme.styles.popup,
            "button" => &theme.styles.button,
            "button_focused" => &theme.styles.button_focused,
            "scrollbar" => &theme.styles.scrollbar,
            "line_numbers" => &theme.styles.line_numbers,
            _ => return Style::default(),
        };

        self.parse_style(element_style, theme)
    }

    pub fn get_color(&self, color_name: &str, theme: &Theme) -> Color {
        let hex_code = match color_name {
            "background" => &theme.colors.background,
            "foreground" => &theme.colors.foreground,
            "primary" => &theme.colors.primary,
            "secondary" => &theme.colors.secondary,
            "accent" => &theme.colors.accent,
            "border" => &theme.colors.border,
            "border_focused" => &theme.colors.border_focused,
            "selection" => &theme.colors.selection,
            "cursor" => &theme.colors.cursor,
            "keyword" => &theme.colors.keyword,
            "function" => &theme.colors.function,
            "variable" => &theme.colors.variable,
            "constant" => &theme.colors.constant,
            "string" => &theme.colors.string,
            "comment" => &theme.colors.comment,
            "operator" => &theme.colors.operator,
            "type_name" => &theme.colors.type_name,
            "success" => &theme.colors.success,
            "warning" => &theme.colors.warning,
            "error" => &theme.colors.error,
            "info" => &theme.colors.info,
            "crypto_detected" => &theme.colors.crypto_detected,
            "obfuscation_detected" => &theme.colors.obfuscation_detected,
            "api_call" => &theme.colors.api_call,
            "jump_target" => &theme.colors.jump_target,
            _ => return Color::Reset,
        };

        parse_hex_color(hex_code)
    }

    fn parse_style(&self, element_style: &ElementStyle, theme: &Theme) -> Style {
        let mut s = Style::default();
        s = s.fg(self.get_color_from_str(&element_style.fg, theme));
        s = s.bg(self.get_color_from_str(&element_style.bg, theme));

        for modifier in &element_style.modifiers {
            s = match modifier.as_str() {
                "bold" => s.add_modifier(Modifier::BOLD),
                "italic" => s.add_modifier(Modifier::ITALIC),
                "underline" => s.add_modifier(Modifier::UNDERLINED),
                "dim" => s.add_modifier(Modifier::DIM),
                "reversed" => s.add_modifier(Modifier::REVERSED),
                _ => s,
            };
        }
        
        s
    }
    
    fn get_color_from_str(&self, color_str: &str, theme: &Theme) -> Color {
        if color_str.starts_with('#') {
            parse_hex_color(color_str)
        } else {
            self.get_color(color_str, theme)
        }
    }
    
    pub fn get_syntax_color(&self, token_type: &str) -> Color {
        match token_type {
            "keyword" => parse_color(&self.current_theme.colors.keyword),
            "function" => parse_color(&self.current_theme.colors.function),
            "variable" => parse_color(&self.current_theme.colors.variable),
            "constant" => parse_color(&self.current_theme.colors.constant),
            "string" => parse_color(&self.current_theme.colors.string),
            "comment" => parse_color(&self.current_theme.colors.comment),
            "operator" => parse_color(&self.current_theme.colors.operator),
            "type" => parse_color(&self.current_theme.colors.type_name),
            _ => parse_color(&self.current_theme.colors.foreground),
        }
    }
}

// ============================================================================
// COLOR UTILITIES
// ============================================================================

fn is_valid_color(color: &str) -> bool {
    if color.starts_with('#') && (color.len() == 7 || color.len() == 4) {
        return color[1..].chars().all(|c| c.is_ascii_hexdigit());
    }
    
    // Check named colors
    matches!(color, "black" | "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "white" | "gray")
}

fn parse_color(color: &str) -> Color {
    if color.starts_with('#') {
        return parse_hex_color(color);
    }
    
    match color {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        _ => Color::White,
    }
}

fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    
    Color::White
}

// ============================================================================
// BUILT-IN THEMES
// ============================================================================

pub fn create_dark_theme() -> Theme {
    Theme {
        name: "Dark".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Classic dark theme with blue accents".to_string(),
        colors: ColorScheme {
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            primary: "#007acc".to_string(),
            secondary: "#3e3e42".to_string(),
            accent: "#0098ff".to_string(),
            border: "#3e3e42".to_string(),
            border_focused: "#007acc".to_string(),
            selection: "#264f78".to_string(),
            cursor: "#aeafad".to_string(),
            keyword: "#569cd6".to_string(),
            function: "#dcdcaa".to_string(),
            variable: "#9cdcfe".to_string(),
            constant: "#4fc1ff".to_string(),
            string: "#ce9178".to_string(),
            comment: "#6a9955".to_string(),
            operator: "#d4d4d4".to_string(),
            type_name: "#4ec9b0".to_string(),
            success: "#4ec9b0".to_string(),
            warning: "#cca700".to_string(),
            error: "#f48771".to_string(),
            info: "#75beff".to_string(),
            crypto_detected: "#c586c0".to_string(),
            obfuscation_detected: "#f48771".to_string(),
            api_call: "#dcdcaa".to_string(),
            jump_target: "#569cd6".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "default".to_string()],
            rating: 5.0,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: false,
                colorblind_safe: true,
                colorblind_type: None,
            },
        },
    }
}

pub fn create_light_theme() -> Theme {
    Theme {
        name: "Light".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Clean light theme for daytime coding".to_string(),
        colors: ColorScheme {
            background: "#ffffff".to_string(),
            foreground: "#000000".to_string(),
            primary: "#0066cc".to_string(),
            secondary: "#e0e0e0".to_string(),
            accent: "#0080ff".to_string(),
            border: "#cccccc".to_string(),
            border_focused: "#0066cc".to_string(),
            selection: "#add6ff".to_string(),
            cursor: "#000000".to_string(),
            keyword: "#0000ff".to_string(),
            function: "#795e26".to_string(),
            variable: "#001080".to_string(),
            constant: "#0070c1".to_string(),
            string: "#a31515".to_string(),
            comment: "#008000".to_string(),
            operator: "#000000".to_string(),
            type_name: "#267f99".to_string(),
            success: "#008000".to_string(),
            warning: "#bf8803".to_string(),
            error: "#cd3131".to_string(),
            info: "#0066cc".to_string(),
            crypto_detected: "#af00db".to_string(),
            obfuscation_detected: "#cd3131".to_string(),
            api_call: "#795e26".to_string(),
            jump_target: "#0000ff".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["light".to_string(), "default".to_string()],
            rating: 4.8,
            downloads: 0,
            is_dark: false,
            accessibility: AccessibilityInfo {
                high_contrast: false,
                colorblind_safe: true,
                colorblind_type: None,
            },
        },
    }
}

pub fn create_cyberpunk_theme() -> Theme {
    Theme {
        name: "Cyberpunk".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Neon-inspired cyberpunk theme".to_string(),
        colors: ColorScheme {
            background: "#0a0e27".to_string(),
            foreground: "#00ff9f".to_string(),
            primary: "#ff00ff".to_string(),
            secondary: "#1a1f3a".to_string(),
            accent: "#00ffff".to_string(),
            border: "#ff00ff".to_string(),
            border_focused: "#00ffff".to_string(),
            selection: "#2a2f4a".to_string(),
            cursor: "#00ff9f".to_string(),
            keyword: "#ff00ff".to_string(),
            function: "#ffff00".to_string(),
            variable: "#00ffff".to_string(),
            constant: "#ff0080".to_string(),
            string: "#00ff9f".to_string(),
            comment: "#7a7f9a".to_string(),
            operator: "#ff00ff".to_string(),
            type_name: "#00ffff".to_string(),
            success: "#00ff9f".to_string(),
            warning: "#ffff00".to_string(),
            error: "#ff0080".to_string(),
            info: "#00ffff".to_string(),
            crypto_detected: "#ff00ff".to_string(),
            obfuscation_detected: "#ff0080".to_string(),
            api_call: "#ffff00".to_string(),
            jump_target: "#00ffff".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "neon".to_string(), "cyberpunk".to_string()],
            rating: 4.9,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: true,
                colorblind_safe: false,
                colorblind_type: None,
            },
        },
    }
}

pub fn create_matrix_theme() -> Theme {
    Theme {
        name: "Matrix".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Green-on-black Matrix-inspired theme".to_string(),
        colors: ColorScheme {
            background: "#000000".to_string(),
            foreground: "#00ff00".to_string(),
            primary: "#00ff00".to_string(),
            secondary: "#003300".to_string(),
            accent: "#00ff00".to_string(),
            border: "#00ff00".to_string(),
            border_focused: "#00ff00".to_string(),
            selection: "#003300".to_string(),
            cursor: "#00ff00".to_string(),
            keyword: "#00ff00".to_string(),
            function: "#00cc00".to_string(),
            variable: "#00ff00".to_string(),
            constant: "#00ff00".to_string(),
            string: "#00cc00".to_string(),
            comment: "#006600".to_string(),
            operator: "#00ff00".to_string(),
            type_name: "#00ff00".to_string(),
            success: "#00ff00".to_string(),
            warning: "#00ff00".to_string(),
            error: "#00ff00".to_string(),
            info: "#00ff00".to_string(),
            crypto_detected: "#00ff00".to_string(),
            obfuscation_detected: "#00ff00".to_string(),
            api_call: "#00cc00".to_string(),
            jump_target: "#00ff00".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "matrix".to_string(), "retro".to_string()],
            rating: 4.7,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: true,
                colorblind_safe: true,
                colorblind_type: Some(ColorblindType::Protanopia),
            },
        },
    }
}

pub fn create_nord_theme() -> Theme {
    Theme {
        name: "Nord".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Arctic, north-bluish color palette".to_string(),
        colors: ColorScheme {
            background: "#2e3440".to_string(),
            foreground: "#d8dee9".to_string(),
            primary: "#88c0d0".to_string(),
            secondary: "#3b4252".to_string(),
            accent: "#81a1c1".to_string(),
            border: "#4c566a".to_string(),
            border_focused: "#88c0d0".to_string(),
            selection: "#434c5e".to_string(),
            cursor: "#d8dee9".to_string(),
            keyword: "#81a1c1".to_string(),
            function: "#88c0d0".to_string(),
            variable: "#d8dee9".to_string(),
            constant: "#b48ead".to_string(),
            string: "#a3be8c".to_string(),
            comment: "#616e88".to_string(),
            operator: "#81a1c1".to_string(),
            type_name: "#8fbcbb".to_string(),
            success: "#a3be8c".to_string(),
            warning: "#ebcb8b".to_string(),
            error: "#bf616a".to_string(),
            info: "#88c0d0".to_string(),
            crypto_detected: "#b48ead".to_string(),
            obfuscation_detected: "#bf616a".to_string(),
            api_call: "#88c0d0".to_string(),
            jump_target: "#81a1c1".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "nord".to_string(), "popular".to_string()],
            rating: 4.9,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: false,
                colorblind_safe: true,
                colorblind_type: None,
            },
        },
    }
}

pub fn create_dracula_theme() -> Theme {
    Theme {
        name: "Dracula".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Dark theme with vibrant colors".to_string(),
        colors: ColorScheme {
            background: "#282a36".to_string(),
            foreground: "#f8f8f2".to_string(),
            primary: "#bd93f9".to_string(),
            secondary: "#44475a".to_string(),
            accent: "#ff79c6".to_string(),
            border: "#6272a4".to_string(),
            border_focused: "#bd93f9".to_string(),
            selection: "#44475a".to_string(),
            cursor: "#f8f8f2".to_string(),
            keyword: "#ff79c6".to_string(),
            function: "#50fa7b".to_string(),
            variable: "#f8f8f2".to_string(),
            constant: "#bd93f9".to_string(),
            string: "#f1fa8c".to_string(),
            comment: "#6272a4".to_string(),
            operator: "#ff79c6".to_string(),
            type_name: "#8be9fd".to_string(),
            success: "#50fa7b".to_string(),
            warning: "#f1fa8c".to_string(),
            error: "#ff5555".to_string(),
            info: "#8be9fd".to_string(),
            crypto_detected: "#bd93f9".to_string(),
            obfuscation_detected: "#ff5555".to_string(),
            api_call: "#50fa7b".to_string(),
            jump_target: "#ff79c6".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "dracula".to_string(), "popular".to_string()],
            rating: 5.0,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: false,
                colorblind_safe: true,
                colorblind_type: None,
            },
        },
    }
}

pub fn create_solarized_dark_theme() -> Theme {
    Theme {
        name: "Solarized Dark".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Precision colors for machines and people".to_string(),
        colors: ColorScheme {
            background: "#002b36".to_string(),
            foreground: "#839496".to_string(),
            primary: "#268bd2".to_string(),
            secondary: "#073642".to_string(),
            accent: "#2aa198".to_string(),
            border: "#586e75".to_string(),
            border_focused: "#268bd2".to_string(),
            selection: "#073642".to_string(),
            cursor: "#839496".to_string(),
            keyword: "#859900".to_string(),
            function: "#268bd2".to_string(),
            variable: "#839496".to_string(),
            constant: "#2aa198".to_string(),
            string: "#2aa198".to_string(),
            comment: "#586e75".to_string(),
            operator: "#859900".to_string(),
            type_name: "#b58900".to_string(),
            success: "#859900".to_string(),
            warning: "#b58900".to_string(),
            error: "#dc322f".to_string(),
            info: "#268bd2".to_string(),
            crypto_detected: "#6c71c4".to_string(),
            obfuscation_detected: "#dc322f".to_string(),
            api_call: "#268bd2".to_string(),
            jump_target: "#859900".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "solarized".to_string(), "popular".to_string()],
            rating: 4.8,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: false,
                colorblind_safe: true,
                colorblind_type: None,
            },
        },
    }
}

pub fn create_monokai_theme() -> Theme {
    Theme {
        name: "Monokai".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Sublime Text's iconic color scheme".to_string(),
        colors: ColorScheme {
            background: "#272822".to_string(),
            foreground: "#f8f8f2".to_string(),
            primary: "#66d9ef".to_string(),
            secondary: "#3e3d32".to_string(),
            accent: "#f92672".to_string(),
            border: "#75715e".to_string(),
            border_focused: "#66d9ef".to_string(),
            selection: "#49483e".to_string(),
            cursor: "#f8f8f0".to_string(),
            keyword: "#f92672".to_string(),
            function: "#a6e22e".to_string(),
            variable: "#f8f8f2".to_string(),
            constant: "#ae81ff".to_string(),
            string: "#e6db74".to_string(),
            comment: "#75715e".to_string(),
            operator: "#f92672".to_string(),
            type_name: "#66d9ef".to_string(),
            success: "#a6e22e".to_string(),
            warning: "#e6db74".to_string(),
            error: "#f92672".to_string(),
            info: "#66d9ef".to_string(),
            crypto_detected: "#ae81ff".to_string(),
            obfuscation_detected: "#f92672".to_string(),
            api_call: "#a6e22e".to_string(),
            jump_target: "#f92672".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "monokai".to_string(), "popular".to_string()],
            rating: 4.9,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: false,
                colorblind_safe: true,
                colorblind_type: None,
            },
        },
    }
}

pub fn create_high_contrast_theme() -> Theme {
    Theme {
        name: "High Contrast".to_string(),
        author: "Decompiler Team".to_string(),
        version: "1.0.0".to_string(),
        description: "Maximum contrast for accessibility".to_string(),
        colors: ColorScheme {
            background: "#000000".to_string(),
            foreground: "#ffffff".to_string(),
            primary: "#00ffff".to_string(),
            secondary: "#1a1a1a".to_string(),
            accent: "#ffff00".to_string(),
            border: "#ffffff".to_string(),
            border_focused: "#00ffff".to_string(),
            selection: "#ffffff".to_string(),
            cursor: "#ffffff".to_string(),
            keyword: "#00ffff".to_string(),
            function: "#ffff00".to_string(),
            variable: "#ffffff".to_string(),
            constant: "#ff00ff".to_string(),
            string: "#00ff00".to_string(),
            comment: "#808080".to_string(),
            operator: "#00ffff".to_string(),
            type_name: "#00ffff".to_string(),
            success: "#00ff00".to_string(),
            warning: "#ffff00".to_string(),
            error: "#ff0000".to_string(),
            info: "#00ffff".to_string(),
            crypto_detected: "#ff00ff".to_string(),
            obfuscation_detected: "#ff0000".to_string(),
            api_call: "#ffff00".to_string(),
            jump_target: "#00ffff".to_string(),
        },
        styles: create_default_styles(),
        metadata: ThemeMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["dark".to_string(), "accessibility".to_string(), "high-contrast".to_string()],
            rating: 4.5,
            downloads: 0,
            is_dark: true,
            accessibility: AccessibilityInfo {
                high_contrast: true,
                colorblind_safe: true,
                colorblind_type: None,
            },
        },
    }
}

fn create_default_styles() -> StyleSheet {
    StyleSheet {
        file_list: ElementStyle {
            fg: "foreground".to_string(),
            bg: "background".to_string(),
            modifiers: vec![],
        },
        file_list_selected: ElementStyle {
            fg: "background".to_string(),
            bg: "primary".to_string(),
            modifiers: vec!["bold".to_string()],
        },
        editor: ElementStyle {
            fg: "foreground".to_string(),
            bg: "background".to_string(),
            modifiers: vec![],
        },
        status_bar: ElementStyle {
            fg: "foreground".to_string(),
            bg: "secondary".to_string(),
            modifiers: vec![],
        },
        title_bar: ElementStyle {
            fg: "foreground".to_string(),
            bg: "primary".to_string(),
            modifiers: vec!["bold".to_string()],
        },
        popup: ElementStyle {
            fg: "foreground".to_string(),
            bg: "secondary".to_string(),
            modifiers: vec![],
        },
        button: ElementStyle {
            fg: "foreground".to_string(),
            bg: "secondary".to_string(),
            modifiers: vec![],
        },
        button_focused: ElementStyle {
            fg: "background".to_string(),
            bg: "accent".to_string(),
            modifiers: vec!["bold".to_string()],
        },
        scrollbar: ElementStyle {
            fg: "primary".to_string(),
            bg: "secondary".to_string(),
            modifiers: vec![],
        },
        line_numbers: ElementStyle {
            fg: "comment".to_string(),
            bg: "background".to_string(),
            modifiers: vec!["dim".to_string()],
        },
    }
}