use crate::client::Client;
use crate::model::{Category, Sort, Torrent};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::TableState;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct SearchState {
    pub input: String,
    pub mode: InputMode,
    pub is_loading: bool,
    pub messages: Vec<String>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            input: String::new(),
            mode: InputMode::Editing,
            is_loading: false,
            messages: Vec::new(),
        }
    }
}

pub struct TableData {
    pub results: Vec<Torrent>,
    pub state: TableState,
    pub current_page: u32,
    pub current_sort: Sort,
    pub last_selected_index: Option<usize>,
}

impl Default for TableData {
    fn default() -> Self {
        Self {
            results: Vec::new(),
            state: TableState::default(),
            current_page: 1,
            current_sort: Sort::Seeders,
            last_selected_index: None,
        }
    }
}

impl TableData {
    pub fn next(&mut self) -> bool {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.results.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        if self.last_selected_index != Some(i) {
            self.last_selected_index = Some(i);
            return true;
        }
        false
    }

    pub fn previous(&mut self) -> bool {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.results.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        if self.last_selected_index != Some(i) {
            self.last_selected_index = Some(i);
            return true;
        }
        false
    }
}

pub struct App {
    pub search: SearchState,
    pub table: TableData,
    pub client: Client,
    pub should_quit: bool,
    pub animation_tick: usize,
    pub theme: Theme,
    pub theme_last_modified: Option<SystemTime>,
    pub theme_path: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        let (theme, theme_last_modified) = Theme::load();
        let theme_path = Theme::path();
        Self {
            search: SearchState::default(),
            table: TableData::default(),
            client: Client::new(),
            should_quit: false,
            animation_tick: 0,
            theme,
            theme_last_modified,
            theme_path,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match self.search.mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Tab | KeyCode::Char('i') => {
                    self.search.mode = InputMode::Editing;
                }
                KeyCode::Char('s') | KeyCode::Down | KeyCode::Char('j') => {
                    if self.table.next() {
                        self.reset_animation();
                    }
                }
                KeyCode::Char('w') | KeyCode::Up | KeyCode::Char('k') => {
                    if self.table.previous() {
                        self.reset_animation();
                    }
                }
                KeyCode::Char('z') => {
                    self.cycle_sort().await;
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    self.next_page().await;
                }
                KeyCode::Left | KeyCode::Char('a') => {
                    self.prev_page().await;
                }
                KeyCode::Enter => self.open_magnet(),
                _ => {}
            },
            InputMode::Editing => match key.code {
                KeyCode::Tab | KeyCode::Esc => {
                    self.search.mode = InputMode::Normal;
                }
                KeyCode::Enter => {
                    self.search.mode = InputMode::Normal;
                    self.perform_search().await;
                }
                KeyCode::Char(c) => {
                    self.search.input.push(c);
                }
                KeyCode::Backspace => {
                    self.search.input.pop();
                }
                _ => {}
            },
        }
    }

    pub fn on_tick(&mut self) {
        self.animation_tick = self.animation_tick.wrapping_add(1);

        if self.animation_tick.is_multiple_of(5) {
            if let Some(path) = &self.theme_path {
                if std::fs::metadata(path)
                    .and_then(|m| m.modified())
                    .ok()
                    .filter(|&m| Some(m) != self.theme_last_modified)
                    .is_some()
                {
                    let (new_theme, new_modified) = Theme::load_from_path(path);
                    self.theme = new_theme;
                    self.theme_last_modified = new_modified;
                }
            } else if self.animation_tick.is_multiple_of(30) {
                self.theme_path = Theme::path();
            }
        }
    }

    fn reset_animation(&mut self) {
        self.animation_tick = 0;
    }

    pub async fn perform_search(&mut self) {
        if self.search.input.trim().is_empty() {
            return;
        }

        self.search.is_loading = true;
        self.search.messages.clear();

        let query = self.search.input.clone();
        let sort = self.table.current_sort;
        let page = self.table.current_page;

        match self.client.search(&query, Category::All, sort, page).await {
            Ok(torrents) => {
                self.table.results = torrents;
                self.table.state.select(Some(0));
                self.table.last_selected_index = Some(0);
                self.reset_animation();
            }
            Err(e) => {
                self.search.messages.push(format!("error: {}", e));
            }
        }
        self.search.is_loading = false;
    }

    pub async fn next_page(&mut self) {
        if self.table.results.is_empty() {
            return;
        }
        self.table.current_page += 1;
        self.perform_search().await;
    }

    pub async fn prev_page(&mut self) {
        if self.table.current_page > 1 {
            self.table.current_page -= 1;
            self.perform_search().await;
        }
    }

    pub async fn cycle_sort(&mut self) {
        self.table.current_sort = match self.table.current_sort {
            Sort::Date => Sort::Downloads,
            Sort::Downloads => Sort::Seeders,
            Sort::Seeders => Sort::Size,
            Sort::Size => Sort::Date,
        };
        self.table.current_page = 1;
        if !self.search.input.trim().is_empty() {
            self.perform_search().await;
        }
    }

    pub fn open_magnet(&self) {
        if let Some(torrent) = self
            .table
            .state
            .selected()
            .and_then(|i| self.table.results.get(i))
            .filter(|t| !t.magnet_url.is_empty())
        {
            let _ = open::that(&torrent.magnet_url);
        }
    }
}
