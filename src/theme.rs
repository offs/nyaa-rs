
use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    pub fg: String,
    pub primary: String,
    pub secondary: String,
    pub selection_bg: String,
    pub border: String,
    pub border_focus: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub fg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub selection_bg: Color,
    pub border: Color,
    pub border_focus: Color,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            fg: Color::Reset,
            primary: Color::Blue,
            secondary: Color::Magenta,
            selection_bg: Color::DarkGray,
            border: Color::DarkGray,
            border_focus: Color::Blue,
        }
    }



    pub fn load() -> (Self, Option<SystemTime>) {
        if let Some(config_path) = Self::get_config_path() {
            return Self::load_from_path(&config_path);
        }
        (Self::new(), None)
    }

    pub fn load_from_path(path: &PathBuf) -> (Self, Option<SystemTime>) {
        if path.exists() {
            let modified = fs::metadata(path)
                .and_then(|m| m.modified())
                .ok();

            if let Some(config) = fs::read_to_string(path)
                .ok()
                .and_then(|c| serde_json::from_str::<ThemeConfig>(&c).ok())
            {
                return (Self::from_config(config), modified);
            }
        }
        (Self::new(), None)
    }

    pub fn path() -> Option<PathBuf> {
        Self::get_config_path()
    }

    fn get_config_path() -> Option<PathBuf> {
        if let Ok(cwd) = std::env::current_dir() {
            let path = cwd.join("theme.json");
            if path.exists() {
                return Some(path);
            }
        }

        if let Some(path) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("theme.json")))
            .filter(|p| p.exists())
        {
            return Some(path);
        }

        directories::ProjectDirs::from("com", "nyaa-rs", "nyaa").map(|proj_dirs| {
            proj_dirs.config_dir().join("theme.json")
        })
    }

    fn from_config(config: ThemeConfig) -> Self {
        Self {
            fg: parse_color(&config.fg).unwrap_or(Color::Reset),
            primary: parse_color(&config.primary).unwrap_or(Color::Blue),
            secondary: parse_color(&config.secondary).unwrap_or(Color::Magenta),
            selection_bg: parse_color(&config.selection_bg).unwrap_or(Color::DarkGray),
            border: parse_color(&config.border).unwrap_or(Color::DarkGray),
            border_focus: parse_color(&config.border_focus).unwrap_or(Color::Blue),
        }
    }
}

fn parse_color(s: &str) -> Option<Color> {
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).ok()?;
        let g = u8::from_str_radix(&s[3..5], 16).ok()?;
        let b = u8::from_str_radix(&s[5..7], 16).ok()?;
        return Some(Color::Rgb(r, g, b));
    }
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "reset" => Some(Color::Reset),
        _ => None,
    }
}
