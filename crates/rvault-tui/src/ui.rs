use crate::app::{
    AddEntryStage, App, AppState, BackupCreateStage, BackupRestoreStage, EditEntryStage,
    ExportEntryStage, SetupStage, SortMode,
}; // Added EditEntryStage
use crate::input::InputState; // Import InputState
use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, HighlightSpacing, List, ListItem, ListState,
        Paragraph, Row, Table, TableState, Tabs,
    },
};
use rvault_core::vault::VaultEntry;

const PASSWORD_MASK_SYMBOL: &str = "•";

#[derive(Clone)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub surface: Color,
    pub accent: Color,
    pub warning: Color,
    pub text: Color,
    pub muted: Color,
    pub error: Color,
}

impl Theme {
    pub fn catppuccin() -> Self {
        Self {
            name: "Catppuccin".into(),
            bg: Color::Rgb(30, 30, 46),         // #1e1e2e
            surface: Color::Rgb(49, 50, 68),    // #313244
            accent: Color::Rgb(137, 180, 250),  // #89b4fa
            warning: Color::Rgb(249, 226, 175), // #f9e2af
            text: Color::Rgb(205, 214, 244),    // #cdd6f4
            muted: Color::Rgb(166, 173, 200),   // #a6adc8
            error: Color::Rgb(243, 139, 168),   // #f38ba8
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: "Dracula".into(),
            bg: Color::Rgb(40, 42, 54),
            surface: Color::Rgb(68, 71, 90),
            accent: Color::Rgb(189, 147, 249),  // Purple
            warning: Color::Rgb(241, 150, 240), // Pinkish for better contrast? Or keep Yellow
            text: Color::Rgb(248, 248, 242),
            muted: Color::Rgb(98, 114, 164),
            error: Color::Rgb(255, 85, 85),
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord".into(),
            bg: Color::Rgb(46, 52, 64),
            surface: Color::Rgb(59, 66, 82),
            accent: Color::Rgb(136, 192, 208), // Cyan
            warning: Color::Rgb(235, 203, 139),
            text: Color::Rgb(236, 239, 244),
            muted: Color::Rgb(76, 86, 106),
            error: Color::Rgb(191, 97, 106),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: "Gruvbox".into(),
            bg: Color::Rgb(40, 40, 40),
            surface: Color::Rgb(60, 56, 54),
            accent: Color::Rgb(251, 73, 52), // Red/Orange
            warning: Color::Rgb(250, 189, 47),
            text: Color::Rgb(235, 219, 178),
            muted: Color::Rgb(168, 153, 132),
            error: Color::Rgb(204, 36, 29),
        }
    }

    pub fn solarized() -> Self {
        Self {
            name: "Solarized".into(),
            bg: Color::Rgb(0, 43, 54),        // Base03
            surface: Color::Rgb(7, 54, 66),   // Base02
            accent: Color::Rgb(38, 139, 210), // Cyan/Blue
            warning: Color::Rgb(181, 137, 0), // Yellow
            text: Color::Rgb(238, 232, 213),  // Base2
            muted: Color::Rgb(147, 161, 161), // Base1
            error: Color::Rgb(220, 50, 47),   // Red
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "Monokai".into(),
            bg: Color::Rgb(39, 40, 34),
            surface: Color::Rgb(62, 61, 50),
            accent: Color::Rgb(249, 38, 114),   // Pink/Magenta
            warning: Color::Rgb(230, 219, 116), // Yellow
            text: Color::Rgb(248, 248, 242),
            muted: Color::Rgb(117, 113, 94),
            error: Color::Rgb(253, 151, 31), // Orange
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".into(),
            bg: Color::Rgb(26, 27, 38),
            surface: Color::Rgb(65, 72, 104),
            accent: Color::Rgb(122, 162, 247),  // Blue
            warning: Color::Rgb(224, 175, 104), // Yellow
            text: Color::Rgb(192, 202, 245),
            muted: Color::Rgb(86, 95, 137),
            error: Color::Rgb(247, 118, 142),
        }
    }

    pub fn one_dark() -> Self {
        Self {
            name: "One Dark".into(),
            bg: Color::Rgb(40, 44, 52),         // #282c34
            surface: Color::Rgb(33, 37, 43),    // #21252b
            accent: Color::Rgb(97, 175, 239),   // Blue #61afef
            warning: Color::Rgb(229, 192, 123), // Yellow #e5c07b
            text: Color::Rgb(171, 178, 191),    // #abb2bf
            muted: Color::Rgb(92, 99, 112),     // #5c6370
            error: Color::Rgb(224, 108, 117),   // Red #e06c75
        }
    }

    pub fn default() -> Self {
        Self::gruvbox()
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let theme_data = app.current_theme.clone();
    let theme = &theme_data;
    match &app.state {
        AppState::Authentication(input) => draw_auth(f, input, &app.auth_error, theme),
        AppState::MainTable => {
            draw_main(f, &app.items, &mut app.list_state, &app.auth_error, theme)
        }
        AppState::Generator => draw_generator(f, app.gen_length, app.gen_special, theme),
        AppState::Setup {
            password,
            confirm,
            stage,
            error,
        } => draw_setup(f, password, confirm, stage, error, theme),
        AppState::RemoveConfirmation { platform, user_id } => {
            draw_remove_confirmation(f, platform, user_id, theme)
        }
        AppState::EditEntry {
            platform,
            original_user_id,
            user_id,
            password,
            stage,
        } => draw_edit_entry(
            f,
            platform,
            original_user_id,
            user_id,
            password,
            stage,
            theme,
        ),
        AppState::AddEntry {
            platform,
            user_id,
            password,
            stage,
        } => draw_add_entry(f, platform, user_id, password, stage, theme),
        AppState::BackupCreate {
            path,
            password,
            stage,
        } => draw_backup_create(f, path, password, stage, theme),
        AppState::BackupRestore {
            path,
            password,
            confirm,
            stage,
        } => draw_backup_restore(f, path, password, confirm, stage, theme),
        AppState::ExportEntry {
            platform,
            user_id,
            recipient,
            path,
            stage,
        } => draw_export_entry(f, platform, user_id, recipient, path, stage, theme),
        AppState::ImportExport { path } => draw_import_export(f, path, theme),
        AppState::ImportExportConfirm { path: _, conflicts } => {
            draw_import_confirm(f, *conflicts, theme)
        }
        AppState::ThemeSelection => draw_theme_selection(f, &app.themes, theme),
        AppState::SortSelection => draw_sort_selection(f, &app.sort_mode, theme),
    }

    if let Some(toast) = &app.toast {
        draw_toast(f, &toast.message, theme);
    }
}

fn draw_sort_selection(f: &mut Frame, current_mode: &SortMode, theme: &Theme) {
    let area = centered_rect_fixed(40, 20, f.area());
    draw_shadow(f, area);

    let modes = SortMode::all();
    let items: Vec<ListItem> = modes
        .iter()
        .map(|mode| {
            let style = if mode == current_mode {
                Style::default()
                    .bg(theme.accent)
                    .fg(theme.bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(Line::from(mode.name())).style(style)
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let list = List::new(items).block(
        Block::default()
            .title(" 🔃 Sort By ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.surface).fg(theme.text)),
    );

    f.render_widget(list, chunks[0]);

    let help_text = Paragraph::new("Press <Enter> to sort")
        .style(Style::default().fg(theme.muted))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(help_text, chunks[1]);
}

fn draw_theme_selection(f: &mut Frame, themes: &[Theme], current: &Theme) {
    let area = centered_rect_fixed(40, 20, f.area());
    draw_shadow(f, area);

    let items: Vec<ListItem> = themes
        .iter()
        .map(|t| {
            let style = if t.name == current.name {
                Style::default()
                    .bg(current.accent)
                    .fg(current.bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(current.text)
            };
            ListItem::new(Line::from(t.name.as_str())).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" 🎨 Select Theme ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(current.accent))
            .style(Style::default().bg(current.surface).fg(current.text)),
    );

    f.render_widget(list, area);
}

fn draw_toast(f: &mut Frame, message: &str, theme: &Theme) {
    let area = f.area();
    let toast_width = (message.len() + 4) as u16;
    let toast_height = 3;
    let toast_area = Rect {
        x: area.width.saturating_sub(toast_width + 2),
        y: 1, // Top right, slightly below border
        width: toast_width,
        height: toast_height,
    };

    draw_shadow(f, toast_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface));

    let text = Paragraph::new(Span::styled(
        message,
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    ))
    .block(block)
    .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(Clear, toast_area);
    f.render_widget(text, toast_area);
}

fn draw_auth(f: &mut Frame, input: &String, error: &Option<String>, theme: &Theme) {
    let area = centered_rect_fixed(40, 10, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" 🔒 Login ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new("Enter master password")
        .style(Style::default().fg(theme.muted))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(title, chunks[0]);

    let input_state = InputState::with_value(input.clone());
    draw_input_box(
        f,
        chunks[1],
        "Password",
        &input_state,
        "",
        true,
        true,
        theme,
    );

    if let Some(err) = error {
        let error_text = Paragraph::new(Span::styled(err, Style::default().fg(theme.error)))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(error_text, chunks[2]);
    }
}

fn draw_setup(
    f: &mut Frame,
    password: &String,
    confirm: &String,
    stage: &SetupStage,
    error: &Option<String>,
    theme: &Theme,
) {
    let area = centered_rect_fixed(50, 12, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" ✨ Setup ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));

    let stage_text = match stage {
        SetupStage::EnterPassword => "Create Master Password",
        SetupStage::ConfirmPassword => "Confirm Master Password",
    };

    let active_password = match stage {
        SetupStage::EnterPassword => password,
        SetupStage::ConfirmPassword => confirm,
    };

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let stage_title = Paragraph::new(Span::styled(
        stage_text,
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(stage_title, chunks[0]);

    let input_state = InputState::with_value(active_password.clone());
    draw_input_box(
        f,
        chunks[2],
        "Password",
        &input_state,
        "",
        true,
        true,
        theme,
    );

    if let Some(err) = error {
        let error_text = Paragraph::new(Span::styled(err, Style::default().fg(theme.error)))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(error_text, chunks[3]);
    }
}

fn draw_main(
    f: &mut Frame,
    items: &[VaultEntry],
    list_state: &mut ListState,
    error: &Option<String>,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Main background
    f.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        f.area(),
    );

    draw_tabs(f, chunks[0], 0, theme);

    // Error Overlay (if any)
    if let Some(err) = error {
        // We can display it at the top or bottom. Let's overlay slightly below tabs?
        // Or better: Use the bottom help area or a toaster.
        // Let's render a block in the middle if error.
        let area = centered_rect_fixed(40, 5, f.area());
        draw_shadow(f, area);
        let p = Paragraph::new(Line::from(Span::styled(
            err,
            Style::default().fg(theme.error),
        )))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Error ")
                .style(Style::default().bg(theme.surface)),
        )
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(p, area);
    }

    let rows: Vec<Row> = items
        .iter()
        .map(|entry| {
            let platform = if entry.pinned {
                format!("📌 {}", entry.platform)
            } else {
                entry.platform.clone()
            };

            Row::new(vec![
                Cell::new(platform).style(Style::default().fg(theme.text)),
                Cell::new(Text::from(entry.user_id.clone()).centered())
                    .style(Style::default().fg(theme.text)),
                Cell::new(Text::from(format_updated_at(entry.updated_at)).right_aligned())
                    .style(Style::default().fg(theme.muted)),
            ])
        })
        .collect();

    let header = Row::new(vec![
        Cell::new("Platform"),
        Cell::new(Text::from("Username").centered()),
        Cell::new(Text::from("Last Update Date").right_aligned()),
    ])
    .style(
        Style::default()
            .fg(theme.muted)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let entries_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.muted))
        .title(Span::styled(
            " Entries ",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ));
    let entries_inner = entries_block.inner(chunks[1]);
    f.render_widget(entries_block, chunks[1]);

    let table_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(entries_inner)[1];

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .style(Style::default().bg(theme.bg))
    .row_highlight_style(
        Style::default()
            .bg(theme.accent)
            .fg(theme.bg)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_spacing(HighlightSpacing::Never);

    let mut table_state = TableState::new()
        .with_offset(list_state.offset())
        .with_selected(list_state.selected());
    f.render_stateful_widget(table, table_area, &mut table_state);
    *list_state.offset_mut() = table_state.offset();
    *list_state.selected_mut() = table_state.selected();

    draw_help(
        f,
        chunks[2],
        "Navigate: ↑/↓ | Copy: Enter | Pin: p | Add: a | Export: x | Import: m | Backup: b | Restore: r | Identity: i | Sort: S | Lock: Q | Quit: q",
        theme,
    );
}

fn format_updated_at(updated_at: i64) -> String {
    if updated_at > 0 {
        let dt = DateTime::from_timestamp(updated_at, 0).unwrap_or_default();
        let local_dt: DateTime<Local> = DateTime::from(dt);
        format!("{}", local_dt.format("%B %d %Y %H:%M"))
    } else {
        "Unknown".to_string()
    }
}

fn draw_generator(f: &mut Frame, gen_length: u8, gen_special: bool, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Main background
    f.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        f.area(),
    );

    draw_tabs(f, chunks[0], 1, theme);

    let area = centered_rect_fixed(60, 16, chunks[1]);
    draw_shadow(f, area);

    let block = Block::default()
        .title(" 🎲 Generate Password ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Length slider
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Special chars
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Generate btn interaction hint
        ])
        .split(area);

    // Visual Slider
    let max_len = 32;
    let filled = gen_length.min(max_len) as usize;
    let bar_width: usize = 32;
    let filled_chars = (filled as f32 / max_len as f32 * bar_width as f32).round() as usize;
    let empty_chars = bar_width.saturating_sub(filled_chars);
    let bar = format!("[{}{}]", "█".repeat(filled_chars), "░".repeat(empty_chars));

    let len_text = Line::from(vec![
        Span::styled("Length: ", Style::default().fg(theme.text)),
        Span::styled(
            format!("{: <2}", gen_length),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(bar, Style::default().fg(theme.accent)),
    ]);
    f.render_widget(
        Paragraph::new(len_text)
            .block(Block::default().borders(Borders::NONE))
            .alignment(ratatui::layout::Alignment::Center),
        inner_layout[0],
    );

    // Toggles
    let spec_toggle = if gen_special {
        vec![
            Span::styled("( ) No  ", Style::default().fg(theme.muted)),
            Span::styled(
                "(●) Yes",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![
            Span::styled(
                "(●) No  ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("( ) Yes", Style::default().fg(theme.muted)),
        ]
    };

    let spec_text = Line::from(
        vec![Span::styled(
            "Special Characters:  ",
            Style::default().fg(theme.text),
        )]
        .into_iter()
        .chain(spec_toggle)
        .collect::<Vec<_>>(),
    );

    f.render_widget(
        Paragraph::new(spec_text)
            .block(Block::default().borders(Borders::NONE))
            .alignment(ratatui::layout::Alignment::Center),
        inner_layout[2],
    );

    f.render_widget(
        Paragraph::new("Press [Enter] to Generate & Copy")
            .style(Style::default().fg(theme.muted))
            .alignment(ratatui::layout::Alignment::Center),
        inner_layout[4],
    );

    draw_help(
        f,
        chunks[2],
        "Adjust: ←/→ | Toggle Spec: s | Generate: Enter | Switch Tab: Tab | Themes: t | Lock: Q | Quit: q",
        theme,
    );
}

fn draw_remove_confirmation(f: &mut Frame, platform: &str, user_id: &str, theme: &Theme) {
    let area = centered_rect_fixed(50, 10, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" 🗑️  Confirm Removal ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.error))
        .style(Style::default().bg(theme.surface).fg(theme.text));

    let text = vec![
        Line::from(Span::styled(
            "Are you sure you want to remove:",
            Style::default().fg(theme.text),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                platform,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::styled(user_id, Style::default().fg(theme.accent)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[y] Yes",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" / "),
            Span::styled("[n] No", Style::default().fg(theme.muted)),
        ]),
    ];
    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_edit_entry(
    f: &mut Frame,
    platform: &str,
    _original_user_id: &str,
    user_id: &InputState,
    password: &InputState,
    stage: &EditEntryStage,
    theme: &Theme,
) {
    let area = centered_rect_fixed(50, 15, f.area());
    draw_shadow(f, area);

    // Modal block
    let block = Block::default()
        .title(" ✏️  Edit Entry ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    // Inner layout for content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Platform (Read-only)
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // UserID (Editable)
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Password (Editable)
        ])
        .split(area);

    // Platform is read-only
    let platform_block = Block::default()
        .title("Platform (Immutable)")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.muted))
        .style(Style::default().bg(theme.surface));
    let p_platform = Paragraph::new(platform)
        .block(platform_block)
        .style(Style::default().fg(theme.muted));
    f.render_widget(p_platform, chunks[0]);

    draw_input_box(
        f,
        chunks[2],
        "User ID",
        user_id,
        "e.g. user@example.com",
        matches!(stage, EditEntryStage::UserId),
        false,
        theme,
    );
    draw_input_box(
        f,
        chunks[4],
        "New Password",
        password,
        "Leave empty to keep current",
        matches!(stage, EditEntryStage::Password),
        true,
        theme,
    );
}

fn draw_add_entry(
    f: &mut Frame,
    platform: &InputState,
    user_id: &InputState,
    password: &InputState,
    stage: &AddEntryStage,
    theme: &Theme,
) {
    let area = centered_rect_fixed(50, 15, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" ➕ Add New Entry ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));

    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1) // Inner margin
        .constraints([
            Constraint::Length(3), // Platform
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // UserID
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Password
        ])
        .split(area);

    draw_input_box(
        f,
        chunks[0],
        "Platform",
        platform,
        "e.g. github.com",
        matches!(stage, AddEntryStage::Platform),
        false,
        theme,
    );
    draw_input_box(
        f,
        chunks[2],
        "User ID",
        user_id,
        "e.g. user@example.com",
        matches!(stage, AddEntryStage::UserId),
        false,
        theme,
    );
    draw_input_box(
        f,
        chunks[4],
        "Password",
        password,
        "••••••••",
        matches!(stage, AddEntryStage::Password),
        true,
        theme,
    ); // Masked
}

fn draw_backup_create(
    f: &mut Frame,
    path: &InputState,
    password: &InputState,
    stage: &BackupCreateStage,
    theme: &Theme,
) {
    let area = centered_rect_fixed(58, 12, f.area());
    draw_two_field_modal(
        f,
        area,
        " 💾 Create Backup ",
        (
            "Path",
            path,
            matches!(stage, BackupCreateStage::Path),
            false,
        ),
        (
            "Master Password",
            password,
            matches!(stage, BackupCreateStage::Password),
            true,
        ),
        theme,
    );
}

fn draw_backup_restore(
    f: &mut Frame,
    path: &InputState,
    password: &InputState,
    confirm: &InputState,
    stage: &BackupRestoreStage,
    theme: &Theme,
) {
    let area = centered_rect_fixed(58, 16, f.area());
    draw_shadow(f, area);
    let block = Block::default()
        .title(" ⬆ Restore Backup ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.error))
        .style(Style::default().bg(theme.surface).fg(theme.text));
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(area);
    draw_input_box(
        f,
        chunks[0],
        "Path",
        path,
        "backup.rvault-backup",
        matches!(stage, BackupRestoreStage::Path),
        false,
        theme,
    );
    draw_input_box(
        f,
        chunks[2],
        "Backup Password",
        password,
        "••••••••",
        matches!(stage, BackupRestoreStage::Password),
        true,
        theme,
    );
    draw_input_box(
        f,
        chunks[4],
        "Type RESTORE",
        confirm,
        "RESTORE",
        matches!(stage, BackupRestoreStage::Confirm),
        false,
        theme,
    );
}

fn draw_export_entry(
    f: &mut Frame,
    platform: &str,
    user_id: &str,
    recipient: &InputState,
    path: &InputState,
    stage: &ExportEntryStage,
    theme: &Theme,
) {
    let area = centered_rect_fixed(62, 15, f.area());
    draw_shadow(f, area);
    let block = Block::default()
        .title(" 📤 Export Entry ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(area);
    let selected = Paragraph::new(Line::from(vec![
        Span::styled(platform, Style::default().fg(theme.accent)),
        Span::raw(" / "),
        Span::styled(user_id, Style::default().fg(theme.text)),
    ]))
    .style(Style::default().bg(theme.surface));
    f.render_widget(selected, chunks[0]);
    draw_input_box(
        f,
        chunks[1],
        "Recipient Identity",
        recipient,
        "rvault1-...",
        matches!(stage, ExportEntryStage::Recipient),
        false,
        theme,
    );
    draw_input_box(
        f,
        chunks[3],
        "Output Path",
        path,
        "entry.rvault-export",
        matches!(stage, ExportEntryStage::Path),
        false,
        theme,
    );
}

fn draw_import_export(f: &mut Frame, path: &InputState, theme: &Theme) {
    let area = centered_rect_fixed(58, 9, f.area());
    draw_shadow(f, area);
    let block = Block::default()
        .title(" 📥 Import Export ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3)])
        .split(area);
    draw_input_box(
        f,
        chunks[0],
        "Path",
        path,
        "file.rvault-export",
        true,
        false,
        theme,
    );
}

fn draw_import_confirm(f: &mut Frame, conflicts: usize, theme: &Theme) {
    let area = centered_rect_fixed(50, 8, f.area());
    draw_shadow(f, area);
    let block = Block::default()
        .title(" Import Conflicts ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(theme.surface).fg(theme.text));
    let text = vec![
        Line::from(format!("{conflicts} entries already exist.")),
        Line::from(""),
        Line::from("[Enter/y] overwrite conflicts   [n] skip conflicts"),
    ];
    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_two_field_modal(
    f: &mut Frame,
    area: Rect,
    title: &str,
    first: (&str, &InputState, bool, bool),
    second: (&str, &InputState, bool, bool),
    theme: &Theme,
) {
    draw_shadow(f, area);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.surface).fg(theme.text));
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(area);
    draw_input_box(f, chunks[0], first.0, first.1, "", first.2, first.3, theme);
    draw_input_box(
        f, chunks[2], second.0, second.1, "", second.2, second.3, theme,
    );
}

fn draw_tabs(f: &mut Frame, area: Rect, index: usize, theme: &Theme) {
    let titles: Vec<Line> = ["  Vaults  ", "  Generator  "]
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(theme.text))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" RVault ")
                .style(Style::default().bg(theme.surface)),
        )
        .select(index)
        .highlight_style(
            Style::default()
                .bg(theme.accent)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, area);
}

fn draw_help(f: &mut Frame, area: Rect, msg: &str, theme: &Theme) {
    let help = Paragraph::new(Span::styled(msg, Style::default().fg(theme.muted)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.muted))
                .style(Style::default().bg(theme.bg)),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(help, area);
}

fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

fn draw_shadow(f: &mut Frame, area: Rect) {
    let shadow_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width,
        height: area.height,
    };
    let shadow_block = Block::default().style(Style::default().bg(Color::Black)); // Simple shadow
    // Or use characters for transparency simulation if background wasn't solid.
    // Since we can't do true transparency, a dark block works best on dark bg if not overlapping text.
    // Actually, TUI shadows are hard without layers. Best is to clear the shadow area with a dark color.
    f.render_widget(Clear, shadow_area);
    f.render_widget(shadow_block, shadow_area);
}

fn draw_input_box(
    f: &mut Frame,
    area: Rect,
    title: &str,
    state: &InputState,
    placeholder: &str,
    active: bool,
    mask: bool,
    theme: &Theme,
) {
    let border_style = if active {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.muted)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .style(Style::default().bg(theme.surface));

    let value = &state.value;
    let cursor_pos = state.cursor_position;

    let mut cursor_offset = 0;
    let content = if value.is_empty() && !active {
        Line::from(Span::styled(placeholder, Style::default().fg(theme.muted)))
    } else if mask && active {
        let before_cursor = value[..cursor_pos].chars().count();
        let after_cursor = value[cursor_pos..].chars().count();
        Line::from(vec![
            Span::styled(
                PASSWORD_MASK_SYMBOL.repeat(before_cursor),
                Style::default().fg(theme.text),
            ),
            Span::styled(
                "▏",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                PASSWORD_MASK_SYMBOL.repeat(after_cursor),
                Style::default().fg(theme.text),
            ),
        ])
    } else if mask {
        Line::from(Span::styled(
            PASSWORD_MASK_SYMBOL.repeat(value.chars().count()),
            Style::default().fg(theme.text),
        ))
    } else {
        cursor_offset = Span::raw(&value[..cursor_pos]).width() as u16;
        Line::from(Span::styled(value.clone(), Style::default().fg(theme.text)))
    };

    let p = Paragraph::new(content).block(block);
    f.render_widget(p, area);

    if active && !mask {
        let cursor_x = area.x + 1 + cursor_offset;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn auth_screen_renders_password_in_a_nested_input_box() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        let theme = Theme::catppuccin();
        let input = "abc".to_string();

        terminal
            .draw(|f| draw_auth(f, &input, &None, &theme))
            .expect("draw auth");

        let symbols = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(symbols.contains("•••"));
        assert!(!symbols.contains("***"));
        assert!(!symbols.contains("abc"));
        assert!(symbols.matches('╭').count() >= 2);
    }

    #[test]
    fn entries_render_date_column_at_same_x_for_each_row() {
        let mut terminal = Terminal::new(TestBackend::new(120, 20)).expect("test terminal");
        let theme = Theme::catppuccin();
        let mut list_state = ListState::default();
        let items = vec![
            test_entry("practicetestautomation.com", "ata"),
            test_entry("Gmail", "atasesli05@gmail.com"),
        ];

        terminal
            .draw(|f| draw_main(f, &items, &mut list_state, &None, &theme))
            .expect("draw main");

        let lines = buffer_lines(&terminal);
        let rendered = lines.join("\n");
        let platform_x = find_cell_positions(&terminal, "Platform")[0];
        let username_x = find_cell_positions(&terminal, "Username")[0];
        let updated_at_x = find_cell_positions(&terminal, "Last Update Date")[0];
        let unknown_positions = find_cell_positions(&terminal, "Unknown");

        assert!(platform_x >= 3);
        assert!((50..=65).contains(&username_x));
        assert!(updated_at_x + "Last Update Date".chars().count() <= 117);
        assert_eq!(unknown_positions.len(), 2);
        assert_eq!(unknown_positions[0], unknown_positions[1]);
        assert!(unknown_positions.iter().all(|x| *x > 100));
        assert!(rendered.contains("Platform"));
        assert!(rendered.contains("Username"));
        assert!(rendered.contains("Last Update Date"));
    }

    #[test]
    fn masked_password_input_uses_bullet_mask_and_inline_cursor() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        let theme = Theme::catppuccin();
        let input = "abc".to_string();

        terminal
            .draw(|f| draw_auth(f, &input, &None, &theme))
            .expect("draw auth");

        let symbols = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(symbols.contains("•••▏"));
        assert!(!symbols.contains("***"));
    }

    fn test_entry(platform: &str, user_id: &str) -> VaultEntry {
        VaultEntry {
            platform: platform.to_string(),
            user_id: user_id.to_string(),
            password: "password".to_string(),
            salt: None,
            nonce: None,
            pinned: false,
            id: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn buffer_lines(terminal: &Terminal<TestBackend>) -> Vec<String> {
        let buffer = terminal.backend().buffer();
        buffer
            .content()
            .chunks(buffer.area.width as usize)
            .map(|cells| cells.iter().map(|cell| cell.symbol()).collect())
            .collect()
    }

    fn find_cell_positions(terminal: &Terminal<TestBackend>, needle: &str) -> Vec<usize> {
        let buffer = terminal.backend().buffer();
        let width = buffer.area.width as usize;
        let needle: Vec<String> = needle.chars().map(|c| c.to_string()).collect();

        buffer
            .content()
            .chunks(width)
            .flat_map(|cells| {
                if needle.len() > cells.len() {
                    return Vec::new();
                }

                (0..=cells.len() - needle.len())
                    .filter(|x| {
                        needle
                            .iter()
                            .enumerate()
                            .all(|(offset, symbol)| cells[*x + offset].symbol() == symbol)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
