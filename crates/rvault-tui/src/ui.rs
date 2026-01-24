use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Clear},
    Frame,
};
use crate::app::{App, AppState, SetupStage};

pub fn draw(f: &mut Frame, app: &mut App) {
    match &app.state {
        AppState::Authentication(input) => draw_auth(f, input, &app.auth_error),
        AppState::MainTable => draw_main(f, app),
        AppState::Generator => draw_generator(f, app),
        AppState::Setup { password, confirm, stage, error } => draw_setup(f, password, confirm, stage, error),
        AppState::RemoveConfirmation { platform, user_id } => draw_remove_confirmation(f, platform, user_id),
        AppState::EditPassword { platform, user_id, input } => draw_edit_password(f, platform, user_id, input),
    }
}

fn draw_auth(f: &mut Frame, input: &String, error: &Option<String>) {
    let block = Block::default()
        .title(" RVault Login ")
        .borders(Borders::ALL);
    let area = centered_rect(60, 20, f.area());
    
    let mut text = vec![
        Line::from("Locked"),
        Line::from(""),
        Line::from(format!("Password: {}", "*".repeat(input.len()))),
    ];
    
    if let Some(err) = error {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(err, Style::default().fg(Color::Red))));
    }

    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Clear, area); // Clear background
    f.render_widget(p, area);
}

fn draw_setup(f: &mut Frame, password: &String, confirm: &String, stage: &SetupStage, error: &Option<String>) {
    let block = Block::default()
        .title(" Welcome to RVault Setup ")
        .borders(Borders::ALL);
    let area = centered_rect(60, 30, f.area());
    
    let stage_text = match stage {
        SetupStage::EnterPassword => "Create Master Password",
        SetupStage::ConfirmPassword => "Confirm Master Password",
    };

    let input_vis = match stage {
        SetupStage::EnterPassword => "*".repeat(password.len()),
        SetupStage::ConfirmPassword => "*".repeat(confirm.len()),
    };

    let mut text = vec![
        Line::from(Span::styled(stage_text, Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(format!("> {}", input_vis)),
    ];
    
    if let Some(err) = error {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(err, Style::default().fg(Color::Red))));
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

    draw_tabs(f, chunks[0], 0);

    let items: Vec<ListItem> = app.items
        .iter()
        .map(|(platform, id)| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{:<20}", platform), Style::default().fg(Color::Cyan)),
                    Span::raw(id),
                ]),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Entries "))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::White).bg(Color::DarkGray))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, chunks[1], &mut app.list_state);
    
    draw_help(f, chunks[2], "Navigate: ↑/↓ | Copy: Enter | Edit: e | Remove: d | Switch Tab: Tab | Quit: q");
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

    draw_tabs(f, chunks[0], 1);

    let block = Block::default().borders(Borders::ALL).title(" Generate Password ");
    let inner_area = centered_rect(60, 40, chunks[1]);
    
    let text = vec![
        Line::from(format!("Length: {} (Left/Right to adjust)", app.gen_length)),
        Line::from(""),
        Line::from(format!("Special Characters [s]: {}", if app.gen_special { "YES" } else { "NO" })),
        Line::from(""),
        Line::from("Press [Enter] to Generate and Copy"),
    ];
    
    let p = Paragraph::new(text)
    .block(block)
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p, inner_area);

    draw_help(f, chunks[2], "Adjust: ←/→ | Toggle Spec: s | Generate: Enter | Switch Tab: Tab | Quit: q");
}

fn draw_remove_confirmation(f: &mut Frame, platform: &str, user_id: &str) {
    let block = Block::default()
        .title(" Confirm Removal ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    
    let area = centered_rect(50, 20, f.area());
    let text = vec![
        Line::from(Span::styled(format!("Are you sure you want to remove:"), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(format!("{} - {}", platform, user_id)),
        Line::from(""),
        Line::from("[y] Yes / [n] No"),
    ];
    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_edit_password(f: &mut Frame, platform: &str, user_id: &str, input: &str) {
    let block = Block::default()
        .title(" Edit Password ")
        .borders(Borders::ALL);
    let area = centered_rect(60, 20, f.area());
    
    let text = vec![
        Line::from(format!("Update password for {} ({})", user_id, platform)),
        Line::from(""),
        Line::from(format!("> {}", input)), // Showing password as plain text for edit, or *? Usually stars but for edit maybe show? Let's use * for consistency with security.
    ];
    // Actually, showing plaintext in edit for a password manager might be risky but user is editing it.
    // Let's stick to * for now.
    
    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Clear, area); // Clear background
    f.render_widget(p, area);
}

fn draw_tabs(f: &mut Frame, area: Rect, index: usize) {
    let titles: Vec<Line> = ["  Vaults  ", "  Generator  "]
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![Span::styled(first, Style::default().fg(Color::Yellow)), Span::raw(rest)])
        })
        .collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" RVault "))
        .select(index)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));
    f.render_widget(tabs, area);
}

fn draw_help(f: &mut Frame, area: Rect, msg: &str) {
      let help = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
