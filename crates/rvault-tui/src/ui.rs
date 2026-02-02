use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Tabs, Clear},
    Frame,
};
use crate::app::{App, AppState, AddEntryStage, SetupStage};

pub struct Theme;

impl Theme {
    pub const BG: Color = Color::Rgb(30, 30, 46); // #1e1e2e
    pub const SURFACE: Color = Color::Rgb(49, 50, 68); // #313244
    pub const ACCENT: Color = Color::Rgb(137, 180, 250); // #89b4fa
    pub const WARNING: Color = Color::Rgb(249, 226, 175); // #f9e2af
    pub const TEXT: Color = Color::Rgb(205, 214, 244); // #cdd6f4
    pub const MUTED: Color = Color::Rgb(166, 173, 200); // #a6adc8
    pub const ERROR: Color = Color::Rgb(243, 139, 168); // #f38ba8
}

pub fn draw(f: &mut Frame, app: &mut App) {
    match &app.state {
        AppState::Authentication(input) => draw_auth(f, input, &app.auth_error),
        AppState::MainTable => draw_main(f, app),
        AppState::Generator => draw_generator(f, app),
        AppState::Setup { password, confirm, stage, error } => draw_setup(f, password, confirm, stage, error),
        AppState::RemoveConfirmation { platform, user_id } => draw_remove_confirmation(f, platform, user_id),
        AppState::EditPassword { platform, user_id, input } => draw_edit_password(f, platform, user_id, input),
        AppState::AddEntry { platform, user_id, password, stage } => draw_add_entry(f, platform, user_id, password, stage),
    }
}

fn draw_auth(f: &mut Frame, input: &String, error: &Option<String>) {
    let area = centered_rect_fixed(40, 10, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" 🔒 Login ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT))
        .style(Style::default().bg(Theme::SURFACE).fg(Theme::TEXT));

    let mut text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Password  ", Style::default().fg(Theme::MUTED)),
            Span::styled("*".repeat(input.len()), Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
        ]),
    ];
    
    if let Some(err) = error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(err, Style::default().fg(Theme::ERROR))));
    }

    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_setup(f: &mut Frame, password: &String, confirm: &String, stage: &SetupStage, error: &Option<String>) {
    let area = centered_rect_fixed(50, 12, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" ✨ Setup ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT))
        .style(Style::default().bg(Theme::SURFACE).fg(Theme::TEXT));
    
    let stage_text = match stage {
        SetupStage::EnterPassword => "Create Master Password",
        SetupStage::ConfirmPassword => "Confirm Master Password",
    };

    let input_vis = match stage {
        SetupStage::EnterPassword => "*".repeat(password.len()),
        SetupStage::ConfirmPassword => "*".repeat(confirm.len()),
    };

    let mut text = vec![
        Line::from(Span::styled(stage_text, Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Password", Style::default().fg(Theme::MUTED))),
        Line::from(Span::styled(input_vis, Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD))),
    ];
    
    if let Some(err) = error {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(err, Style::default().fg(Theme::ERROR))));
    }

    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_main(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Main background
    f.render_widget(Block::default().style(Style::default().bg(Theme::BG)), f.area());

    draw_tabs(f, chunks[0], 0);

    let items: Vec<ListItem> = app.items
        .iter()
        .map(|(platform, id)| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{:<20}", platform), Style::default().fg(Theme::TEXT)),
                    Span::styled(" │ ", Style::default().fg(Theme::MUTED)),
                    Span::styled(id, Style::default().fg(Theme::TEXT)),
                ]),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Theme::MUTED))
            .title(Span::styled(" Entries ", Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD))))
        .style(Style::default().bg(Theme::BG))
        .highlight_style(Style::default().bg(Theme::ACCENT).fg(Theme::BG).add_modifier(Modifier::BOLD)) // Accent bg, dark text
        .highlight_symbol("  ");
        
    f.render_stateful_widget(list, chunks[1], &mut app.list_state);
    
    draw_help(f, chunks[2], "Navigate: ↑/↓ | Copy: Enter | Add: a | Edit: e | Delete: d | Switch Tab: Tab | Quit: q");
}

fn draw_generator(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Main background
    f.render_widget(Block::default().style(Style::default().bg(Theme::BG)), f.area());

    draw_tabs(f, chunks[0], 1);

    let area = centered_rect_fixed(60, 16, chunks[1]);
    draw_shadow(f, area);

    let block = Block::default()
        .title(" 🎲 Generate Password ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT))
        .style(Style::default().bg(Theme::SURFACE).fg(Theme::TEXT));
    
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
    let filled = app.gen_length.min(max_len) as usize;
    let bar_width: usize = 20;
    let filled_chars = (filled as f32 / max_len as f32 * bar_width as f32).round() as usize;
    let empty_chars = bar_width.saturating_sub(filled_chars);
    let bar = format!("[{}{}]", "█".repeat(filled_chars), "░".repeat(empty_chars));

    let len_text = Line::from(vec![
        Span::styled("Length: ", Style::default().fg(Theme::TEXT)),
        Span::styled(format!("{: <2}", app.gen_length), Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(bar, Style::default().fg(Theme::ACCENT)),
    ]);
    f.render_widget(Paragraph::new(len_text).block(Block::default().borders(Borders::NONE)).alignment(ratatui::layout::Alignment::Center), inner_layout[0]);

    // Toggles
    let spec_toggle = if app.gen_special {
        vec![
            Span::styled("( ) No  ", Style::default().fg(Theme::MUTED)),
            Span::styled("(●) Yes", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]
    } else {
        vec![
            Span::styled("(●) No  ", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("( ) Yes", Style::default().fg(Theme::MUTED)),
        ]
    };
    
    let spec_text = Line::from(vec![
        Span::styled("Special Characters:  ", Style::default().fg(Theme::TEXT)),
    ].into_iter().chain(spec_toggle).collect::<Vec<_>>());
    
    f.render_widget(Paragraph::new(spec_text).block(Block::default().borders(Borders::NONE)).alignment(ratatui::layout::Alignment::Center), inner_layout[2]);

    f.render_widget(Paragraph::new("Press [Enter] to Generate & Copy").style(Style::default().fg(Theme::MUTED)).alignment(ratatui::layout::Alignment::Center), inner_layout[4]);

    draw_help(f, chunks[2], "Adjust: ←/→ | Toggle Spec: s | Generate: Enter | Switch Tab: Tab | Quit: q");
}

fn draw_remove_confirmation(f: &mut Frame, platform: &str, user_id: &str) {
    let area = centered_rect_fixed(50, 10, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" 🗑️  Confirm Removal ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ERROR))
        .style(Style::default().bg(Theme::SURFACE).fg(Theme::TEXT));
    
    let text = vec![
        Line::from(Span::styled("Are you sure you want to remove:", Style::default().fg(Theme::TEXT))),
        Line::from(""),
        Line::from(vec![
            Span::styled(platform, Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled(user_id, Style::default().fg(Theme::ACCENT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y] Yes", Style::default().fg(Theme::ERROR).add_modifier(Modifier::BOLD)),
            Span::raw(" / "),
            Span::styled("[n] No", Style::default().fg(Theme::MUTED)),
        ]),
    ];
    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_edit_password(f: &mut Frame, platform: &str, user_id: &str, input: &str) {
    let area = centered_rect_fixed(50, 10, f.area());
    draw_shadow(f, area);

    // Modal block
    let block = Block::default()
        .title(" ✏️  Edit Password ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT))
        .style(Style::default().bg(Theme::SURFACE).fg(Theme::TEXT));
    
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    // Inner layout for content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Info text
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Input box
            Constraint::Min(0),
        ])
        .split(area);

    let info_text = Line::from(vec![
        Span::styled(format!("Update for {} ", platform), Style::default().fg(Theme::MUTED)),
        Span::styled(format!("({})", user_id), Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(info_text).alignment(ratatui::layout::Alignment::Center), chunks[0]);

    draw_input_box(f, chunks[2], "New Password", input, "Type new password...", true, false); // Active always true as it's the only field? Or handle focus if complex. It is the only field.
}

fn draw_add_entry(f: &mut Frame, platform: &str, user_id: &str, password: &str, stage: &AddEntryStage) {
    let area = centered_rect_fixed(50, 15, f.area());
    draw_shadow(f, area);

    let block = Block::default()
        .title(" ➕ Add New Entry ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT))
        .style(Style::default().bg(Theme::SURFACE).fg(Theme::TEXT));
    
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

    draw_input_box(f, chunks[0], "Platform", platform, "e.g. github.com", matches!(stage, AddEntryStage::Platform), false);
    draw_input_box(f, chunks[2], "User ID", user_id, "e.g. user@example.com", matches!(stage, AddEntryStage::UserId), false);
    draw_input_box(f, chunks[4], "Password", password, "••••••••", matches!(stage, AddEntryStage::Password), true); // Masked
}

fn draw_tabs(f: &mut Frame, area: Rect, index: usize) {
    let titles: Vec<Line> = ["  Vaults  ", "  Generator  "]
        .iter()
        .map(|t| {
            Line::from(Span::styled(*t, Style::default().fg(Theme::TEXT)))
        })
        .collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" RVault ").style(Style::default().bg(Theme::SURFACE)))
        .select(index)
        .highlight_style(Style::default().bg(Theme::ACCENT).fg(Theme::BG).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn draw_help(f: &mut Frame, area: Rect, msg: &str) {
      let help = Paragraph::new(Span::styled(msg, Style::default().fg(Theme::MUTED)))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Theme::MUTED))
            .style(Style::default().bg(Theme::BG)));
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

fn draw_input_box(f: &mut Frame, area: Rect, title: &str, value: &str, placeholder: &str, active: bool, mask: bool) {
    let border_style = if active {
        Style::default().fg(Theme::ACCENT)
    } else {
        Style::default().fg(Theme::MUTED)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .style(Style::default().bg(Theme::SURFACE));

    let display_value = if value.is_empty() && !active {
        Span::styled(placeholder, Style::default().fg(Theme::MUTED))
    } else {
        let content = if mask { "*".repeat(value.len()) } else { value.to_string() };
        let content = if active { format!("{}|", content) } else { content }; // Cursor
        Span::styled(content, Style::default().fg(Theme::TEXT))
    };

    let p = Paragraph::new(display_value).block(block);
    f.render_widget(p, area);
}
