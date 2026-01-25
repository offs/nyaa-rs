use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Table},
};

use crate::app::{App, InputMode};
use crate::theme::Theme;

pub fn ui(f: &mut Frame, app: &mut App) {
    let theme = app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    render_search(f, app, chunks[0], &theme);
    render_table(f, app, chunks[1], &theme);
    render_footer(f, app, chunks[3], &theme);
}

fn render_search(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let (border_color, text_style) = match app.search.mode {
        InputMode::Normal => (theme.border, Style::default().fg(theme.fg)),
        InputMode::Editing => (theme.border_focus, Style::default().fg(theme.primary)),
    };

    let input = Paragraph::new(app.search.input.as_str())
        .style(text_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(" search ")
                .title_style(
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
        );
    f.render_widget(input, area);
}

const DATE_WIDTH: u16 = 10;
const SIZE_WIDTH: u16 = 10;
const SEEDERS_WIDTH: u16 = 10;
const DOWNLOADS_WIDTH: u16 = 8;
const SPACERS: u16 = 5;

fn render_table(f: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    let header_cells = ["date", "title", "size", "s / l", "dls"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
    });

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let fixed_width = DATE_WIDTH + SIZE_WIDTH + SEEDERS_WIDTH + DOWNLOADS_WIDTH;
    let title_width = area
        .width
        .saturating_sub(2)
        .saturating_sub(fixed_width)
        .saturating_sub(SPACERS) as usize;
    let title_width = title_width.max(10);

    let selected_idx = app.table.state.selected();
    let rows = app.table.results.iter().enumerate().map(|(i, item)| {
        let is_selected = selected_idx == Some(i);
        let title_content = marquee(&item.title, title_width, app.animation_tick, is_selected);

        let cells = vec![
            Cell::from(item.date.as_str()),
            Cell::from(title_content).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(item.size.as_str()),
            Cell::from(format!("{} / {}", item.seeders, item.leechers)),
            Cell::from(item.downloads.to_string()),
        ];
        Row::new(cells)
            .height(1)
            .style(Style::default().fg(theme.fg))
    });

    let title = format!(
        " results (sort: {}) (page {}) ",
        app.table.current_sort, app.table.current_page
    );

    let t = Table::new(
        rows,
        [
            Constraint::Length(DATE_WIDTH),
            Constraint::Min(50),
            Constraint::Length(SIZE_WIDTH),
            Constraint::Length(SEEDERS_WIDTH),
            Constraint::Length(DOWNLOADS_WIDTH),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border))
            .title(title)
            .title_style(Style::default().fg(theme.secondary)),
    )
    .row_highlight_style(
        Style::default()
            .bg(theme.selection_bg)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(" ")
    .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(t, area, &mut app.table.state);
}

fn marquee(text: &str, width: usize, tick: usize, is_selected: bool) -> String {
    let char_count = text.chars().count();
    if char_count <= width || !is_selected {
        return text.to_string();
    }

    const DELAY_TICKS: usize = 5;
    if tick <= DELAY_TICKS {
        return text.to_string();
    }

    const SEPARATOR: &str = "   ";
    let cycle_len = char_count + SEPARATOR.len();
    let start = (tick - DELAY_TICKS) % cycle_len;

    text.chars()
        .chain(SEPARATOR.chars())
        .cycle()
        .skip(start)
        .take(width)
        .collect()
}

fn render_footer(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let base_style = Style::default().fg(theme.border);
    let key_style = Style::default()
        .fg(theme.secondary)
        .add_modifier(Modifier::BOLD);

    let k = |s: &'static str| Span::styled(s, key_style);
    let t = |s: &'static str| Span::styled(s, base_style);

    let mut spans = match app.search.mode {
        InputMode::Normal => vec![
            k("q"),
            t(" quit, "),
            k("tab"),
            t(" search, "),
            k("w/s/↑/↓"),
            t(" nav, "),
            k("enter"),
            t(" open, "),
            k("z"),
            t(" sort, "),
            k("a/d/←/→"),
            t(" page "),
        ],
        InputMode::Editing => vec![k("tab/esc"), t(" list, "), k("enter"), t(" submit ")],
    };

    if app.search.is_loading {
        spans.push(Span::styled(
            " [loading...]",
            Style::default().fg(theme.primary),
        ));
    } else if let Some(msg) = app.search.messages.last() {
        spans.push(Span::styled(
            format!(" [{}]", msg),
            Style::default().fg(theme.primary),
        ));
    }

    let p = Paragraph::new(Line::from(spans))
        .block(Block::default())
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(p, area);
}
