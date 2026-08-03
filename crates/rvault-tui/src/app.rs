use crate::input::InputState;
use crate::ui::Theme;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::ListState;
use rvault_core::{
    backup, clipboard, config, crypto, identity,
    keystore::{self, keystore_path},
    portable_export,
    session::{self, SessionKey},
    storage::{
        Database, EntryMetadata, EntryRepository, EntrySelector, EntryUpdate, NewEntry,
        StorageError, Table,
    },
    vault::Vault,
};
use std::io;
use std::time::{Duration, Instant};

pub enum SetupStage {
    EnterPassword,
    ConfirmPassword,
}

pub enum AddEntryStage {
    Platform,
    UserId,
    Password,
}

pub enum EditEntryStage {
    UserId,
    Password,
}

pub enum BackupCreateStage {
    Path,
    Password,
}

pub enum BackupRestoreStage {
    Path,
    Password,
    Confirm,
}

pub enum ExportEntryStage {
    Recipient,
    Path,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    TimeAsc,
    TimeDesc,
    PlatformAsc,
    PlatformDesc,
    UserIdAsc,
    UserIdDesc,
}

impl SortMode {
    pub fn all() -> Vec<SortMode> {
        vec![
            SortMode::TimeDesc,
            SortMode::TimeAsc,
            SortMode::PlatformAsc,
            SortMode::PlatformDesc,
            SortMode::UserIdAsc,
            SortMode::UserIdDesc,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SortMode::TimeAsc => "Time (Oldest First)",
            SortMode::TimeDesc => "Time (Newest First)",
            SortMode::PlatformAsc => "Platform (A-Z)",
            SortMode::PlatformDesc => "Platform (Z-A)",
            SortMode::UserIdAsc => "User ID (A-Z)",
            SortMode::UserIdDesc => "User ID (Z-A)",
        }
    }
}

pub struct Toast {
    pub message: String,
    pub expires_at: Instant,
}

pub enum AppState {
    Authentication(String), // Stores current password input
    MainTable,
    Generator,
    Setup {
        password: String,
        confirm: String,
        stage: SetupStage,
        error: Option<String>,
    },
    RemoveConfirmation {
        platform: String,
        user_id: String,
    },
    EditEntry {
        platform: String,         // Immutable
        original_user_id: String, // Target for update
        user_id: InputState,
        password: InputState,
        stage: EditEntryStage,
    },
    AddEntry {
        platform: InputState,
        user_id: InputState,
        password: InputState,
        stage: AddEntryStage,
    },
    BackupCreate {
        path: InputState,
        password: InputState,
        stage: BackupCreateStage,
    },
    BackupRestore {
        path: InputState,
        password: InputState,
        confirm: InputState,
        stage: BackupRestoreStage,
    },
    ExportEntry {
        platform: String,
        user_id: String,
        recipient: InputState,
        path: InputState,
        stage: ExportEntryStage,
    },
    ImportExport {
        path: InputState,
    },
    ImportExportConfirm {
        path: String,
        conflicts: usize,
    },
    ThemeSelection,
    SortSelection,
}

pub struct App {
    pub state: AppState,
    pub items: Vec<EntryMetadata>,
    pub list_state: ListState,

    // Generator state
    pub gen_length: u8,
    pub gen_special: bool,

    // Auth state
    pub auth_error: Option<String>,

    // Theme
    pub themes: Vec<Theme>,
    pub current_theme: Theme,

    // Sorting
    // Sorting
    pub sort_mode: SortMode,

    // Toast
    pub toast: Option<Toast>,
}

impl App {
    pub fn new() -> Self {
        let config = config::Config::new().unwrap_or_default();
        let initial_state = if config.master_password_hash.is_some() {
            AppState::Authentication(String::new())
        } else {
            AppState::Setup {
                password: String::new(),
                confirm: String::new(),
                stage: SetupStage::EnterPassword,
                error: None,
            }
        };

        let themes = vec![
            Theme::catppuccin(),
            Theme::dracula(),
            Theme::nord(),
            Theme::gruvbox(),
            Theme::solarized(),
            Theme::monokai(),
            Theme::tokyo_night(),
            Theme::one_dark(),
        ];

        let current_theme = if let Some(stored_hash) = &config.master_password_hash {
            // Config exists, try to load theme
            // Re-load config to be sure or just use the one we loaded?
            // Actually App::new loaded config at line 71.
            // We need to match config.theme string to our themes vec.
            themes
                .iter()
                .find(|t| t.name == config.theme)
                .cloned()
                .unwrap_or(Theme::default())
        } else {
            Theme::default()
        };

        Self {
            state: initial_state,
            items: Vec::new(),
            list_state: ListState::default(),
            gen_length: 12,
            gen_special: false,
            auth_error: None,
            themes,
            current_theme,
            sort_mode: SortMode::PlatformAsc,
            toast: None,
        }
    }

    pub fn show_toast(&mut self, message: &str) {
        self.toast = Some(Toast {
            message: message.to_string(),
            expires_at: Instant::now() + Duration::from_secs(3),
        });
    }

    pub fn tick(&mut self) {
        if let Some(toast) = &self.toast {
            if Instant::now() >= toast.expires_at {
                self.toast = None;
            }
        }
    }

    pub fn next_tab(&mut self) {
        match self.state {
            AppState::MainTable => self.state = AppState::Generator,
            AppState::Generator => self.state = AppState::MainTable,
            _ => {}
        }
    }

    pub fn check_session(&mut self) -> bool {
        match SessionKey::load() {
            Ok(_) => {
                self.state = AppState::MainTable;
                self.refresh_vault_list();
                true
            }
            Err(_) => false,
        }
    }

    pub fn refresh_vault_list(&mut self) {
        if let Ok(db) = Database::new() {
            if let Ok(repository) = EntryRepository::new(&db, None) {
                if let Ok(entries) = repository.list_metadata() {
                    self.items = entries;
                    self.sort_items();
                }
            }
        }
    }

    pub fn sort_items(&mut self) {
        // Separate pinned and unpinned
        let (pinned, mut unpinned): (Vec<_>, Vec<_>) = self.items.drain(..).partition(|e| e.pinned);

        // Sort unpinned based on sort_mode
        match self.sort_mode {
            SortMode::TimeAsc => {
                unpinned.sort_by(|a, b| {
                    let a_time = if a.updated_at > 0 {
                        a.updated_at
                    } else if a.created_at > 0 {
                        a.created_at
                    } else {
                        a.id.unwrap_or(0)
                    };
                    let b_time = if b.updated_at > 0 {
                        b.updated_at
                    } else if b.created_at > 0 {
                        b.created_at
                    } else {
                        b.id.unwrap_or(0)
                    };
                    a_time.cmp(&b_time)
                });
            }
            SortMode::TimeDesc => {
                unpinned.sort_by(|a, b| {
                    let a_time = if a.updated_at > 0 {
                        a.updated_at
                    } else if a.created_at > 0 {
                        a.created_at
                    } else {
                        a.id.unwrap_or(0)
                    };
                    let b_time = if b.updated_at > 0 {
                        b.updated_at
                    } else if b.created_at > 0 {
                        b.created_at
                    } else {
                        b.id.unwrap_or(0)
                    };
                    b_time.cmp(&a_time)
                });
            }
            SortMode::PlatformAsc => {
                unpinned.sort_by(|a, b| a.platform.to_lowercase().cmp(&b.platform.to_lowercase()));
            }
            SortMode::PlatformDesc => {
                unpinned.sort_by(|a, b| b.platform.to_lowercase().cmp(&a.platform.to_lowercase()));
            }
            SortMode::UserIdAsc => {
                unpinned.sort_by(|a, b| a.user_id.to_lowercase().cmp(&b.user_id.to_lowercase()));
            }
            SortMode::UserIdDesc => {
                unpinned.sort_by(|a, b| b.user_id.to_lowercase().cmp(&a.user_id.to_lowercase()));
            }
        }

        // Merge: pinned first, then sorted unpinned
        self.items = pinned;
        self.items.extend(unpinned);
    }

    pub fn on_paste(&mut self, value: &str) {
        match &mut self.state {
            AppState::Authentication(input) => input.push_str(value),
            AppState::Setup {
                password,
                confirm,
                stage,
                ..
            } => match stage {
                SetupStage::EnterPassword => password.push_str(value),
                SetupStage::ConfirmPassword => confirm.push_str(value),
            },
            AppState::EditEntry {
                user_id,
                password,
                stage,
                ..
            } => match stage {
                EditEntryStage::UserId => user_id.insert_str(value),
                EditEntryStage::Password => password.insert_str(value),
            },
            AppState::AddEntry {
                platform,
                user_id,
                password,
                stage,
            } => match stage {
                AddEntryStage::Platform => platform.insert_str(value),
                AddEntryStage::UserId => user_id.insert_str(value),
                AddEntryStage::Password => password.insert_str(value),
            },
            AppState::BackupCreate {
                path,
                password,
                stage,
            } => active_backup_create_input(path, password, stage).insert_str(value),
            AppState::BackupRestore {
                path,
                password,
                confirm,
                stage,
            } => active_backup_restore_input(path, password, confirm, stage).insert_str(value),
            AppState::ExportEntry {
                recipient,
                path,
                stage,
                ..
            } => active_export_input(recipient, path, stage).insert_str(value),
            AppState::ImportExport { path } => path.insert_str(value),
            AppState::MainTable
            | AppState::Generator
            | AppState::RemoveConfirmation { .. }
            | AppState::ImportExportConfirm { .. }
            | AppState::ThemeSelection
            | AppState::SortSelection => {}
        }
    }

    fn handle_entry_save_result(&mut self, result: Result<(), String>) -> bool {
        match result {
            Ok(()) => {
                self.show_toast("Entry saved!");
                true
            }
            Err(error) => {
                self.show_toast(&format!("Failed to save entry: {error}"));
                false
            }
        }
    }

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) -> io::Result<bool> {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        let mut transition_to_main = false;
        let mut transition_to_login = false;
        let mut entry_save_result = None;

        match &mut self.state {
            AppState::Authentication(input) => {
                match key.code {
                    KeyCode::Enter => {
                        let config = config::Config::new().unwrap_or_default();
                        if let Some(stored_hash) = &config.master_password_hash {
                            match Vault::get_encryption_key(input, stored_hash) {
                                Ok(key) => {
                                    if let Ok(token) = session::start_session(&key) {
                                        let _ = session::write_current(&token);
                                        transition_to_main = true;
                                    } else {
                                        self.auth_error = Some("Failed to start session".into());
                                    }
                                }
                                Err(_) => {
                                    self.auth_error = Some("Invalid Password".into());
                                }
                            }
                        } else {
                            self.auth_error = Some("RVault not set up.".into());
                        }
                        input.clear();
                    }
                    KeyCode::Esc => return Ok(true),
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    _ => {}
                }
            }
            AppState::MainTable => {
                match key.code {
                    KeyCode::Char('a') => {
                        self.state = AppState::AddEntry {
                            platform: InputState::new(),
                            user_id: InputState::new(),
                            password: InputState::new(),
                            stage: AddEntryStage::Platform,
                        };
                    }
                    KeyCode::Char('i') => match SessionKey::load()
                        .map_err(|e| e.to_string())
                        .and_then(|key| identity::load_or_create_identity(key.as_bytes()))
                    {
                        Ok(id) => {
                            clipboard::copy_text(identity::public_code_from_key(&id.public_key));
                            self.show_toast("Identity copied!");
                        }
                        Err(e) => self.auth_error = Some(e),
                    },
                    KeyCode::Char('b') => {
                        self.state = AppState::BackupCreate {
                            path: InputState::with_value("rvault.rvault-backup".to_string()),
                            password: InputState::new(),
                            stage: BackupCreateStage::Path,
                        };
                    }
                    KeyCode::Char('r') => {
                        self.state = AppState::BackupRestore {
                            path: InputState::with_value("rvault.rvault-backup".to_string()),
                            password: InputState::new(),
                            confirm: InputState::new(),
                            stage: BackupRestoreStage::Path,
                        };
                    }
                    KeyCode::Char('x') => {
                        if let Some(i) = self.list_state.selected() {
                            if let Some(entry) = self.items.get(i) {
                                self.state = AppState::ExportEntry {
                                    platform: entry.platform.clone(),
                                    user_id: entry.user_id.clone(),
                                    recipient: InputState::new(),
                                    path: InputState::with_value(format!(
                                        "{}.rvault-export",
                                        sanitize_file_name(&entry.platform)
                                    )),
                                    stage: ExportEntryStage::Recipient,
                                };
                            }
                        }
                    }
                    KeyCode::Char('m') => {
                        self.state = AppState::ImportExport {
                            path: InputState::with_value("rvault.rvault-export".to_string()),
                        };
                    }
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                    KeyCode::Char('Q') => {
                        let _ = rvault_core::lock();
                        return Ok(true);
                    }
                    KeyCode::Tab => self.next_tab(),
                    KeyCode::Down => {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i >= self.items.len().saturating_sub(1) {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match self.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.items.len().saturating_sub(1)
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                    KeyCode::Char('p') => {
                        if let Some(i) = self.list_state.selected() {
                            if let Some(entry) = self.items.get(i) {
                                if let Ok(db) = Database::new() {
                                    if let Ok(repository) = EntryRepository::new(&db, None) {
                                        match repository.set_pinned(
                                            EntrySelector::new(&entry.platform, &entry.user_id),
                                            !entry.pinned,
                                        ) {
                                            Ok(_) => {
                                                self.refresh_vault_list();
                                                self.auth_error = None;
                                            }
                                            Err(_) => {
                                                self.auth_error =
                                                    Some("Pin limit reached (max 10)".into());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(i) = self.list_state.selected() {
                            if let Some(entry) = self.items.get(i) {
                                self.state = AppState::RemoveConfirmation {
                                    platform: entry.platform.clone(),
                                    user_id: entry.user_id.clone(),
                                };
                            }
                        }
                    }
                    KeyCode::Char('e') => {
                        if let Some(i) = self.list_state.selected() {
                            if let Some(entry) = self.items.get(i) {
                                self.state = AppState::EditEntry {
                                    platform: entry.platform.clone(),
                                    original_user_id: entry.user_id.clone(),
                                    user_id: InputState::with_value(entry.user_id.clone()),
                                    password: InputState::new(), // Start empty for security, or fetch? Better empty to act as "change password"
                                    stage: EditEntryStage::UserId,
                                };
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(i) = self.list_state.selected() {
                            if let Some(entry) = self.items.get(i) {
                                if let Ok(db) = Database::new() {
                                    if let Ok(repository) = EntryRepository::new(&db, None) {
                                        if let Ok(ek) = SessionKey::load() {
                                            if let Ok(entry) = repository.get(
                                                &ek,
                                                EntrySelector::new(&entry.platform, &entry.user_id),
                                            ) {
                                                if let Ok(plaintext) =
                                                    std::str::from_utf8(entry.secret.expose())
                                                {
                                                    clipboard::copy_text(plaintext.to_string());
                                                }
                                                self.show_toast("Password has been copied!");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('t') => {
                        self.state = AppState::ThemeSelection;
                    }
                    KeyCode::Char('S') => {
                        self.state = AppState::SortSelection;
                    }
                    _ => {}
                }
            }
            AppState::SortSelection => match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('S') => {
                    self.state = AppState::MainTable;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let modes = SortMode::all();
                    if let Some(pos) = modes.iter().position(|&m| m == self.sort_mode) {
                        let next = (pos + 1) % modes.len();
                        self.sort_mode = modes[next];
                        self.sort_items();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let modes = SortMode::all();
                    if let Some(pos) = modes.iter().position(|&m| m == self.sort_mode) {
                        let prev = if pos == 0 { modes.len() - 1 } else { pos - 1 };
                        self.sort_mode = modes[prev];
                        self.sort_items();
                    }
                }
                KeyCode::Enter => {
                    self.sort_items();
                    self.state = AppState::MainTable;
                }
                _ => {}
            },
            AppState::ThemeSelection => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('t') => {
                        // Save theme code
                        let mut config = config::Config::new().unwrap_or_default();
                        config.theme = self.current_theme.name.clone();
                        let _ = config.save_config();
                        self.state = AppState::MainTable;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        // Cycle next theme
                        if let Some(pos) = self
                            .themes
                            .iter()
                            .position(|t| t.name == self.current_theme.name)
                        {
                            let next = (pos + 1) % self.themes.len();
                            self.current_theme = self.themes[next].clone();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        // Cycle prev theme
                        if let Some(pos) = self
                            .themes
                            .iter()
                            .position(|t| t.name == self.current_theme.name)
                        {
                            let prev = if pos == 0 {
                                self.themes.len() - 1
                            } else {
                                pos - 1
                            };
                            self.current_theme = self.themes[prev].clone();
                        }
                    }
                    _ => {}
                }
            }
            AppState::Generator => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                KeyCode::Tab => self.next_tab(),
                KeyCode::Char('t') => self.state = AppState::ThemeSelection,
                KeyCode::Char('s') => self.gen_special = !self.gen_special,
                KeyCode::Left => {
                    if self.gen_length > 4 {
                        self.gen_length -= 1
                    }
                }
                KeyCode::Right => {
                    if self.gen_length < 32 {
                        self.gen_length += 1
                    }
                }
                KeyCode::Enter => {
                    if let Ok(pass) =
                        crypto::try_generate_password(self.gen_length, self.gen_special)
                    {
                        clipboard::copy_text(pass);
                        self.show_toast("Password has been copied!");
                    }
                }
                _ => {}
            },
            AppState::Setup {
                password,
                confirm,
                stage,
                error,
            } => {
                match key.code {
                    KeyCode::Esc => return Ok(true),
                    KeyCode::Enter => {
                        match stage {
                            SetupStage::EnterPassword => {
                                if !password.is_empty() {
                                    *stage = SetupStage::ConfirmPassword;
                                    *error = None;
                                }
                            }
                            SetupStage::ConfirmPassword => {
                                if password == confirm {
                                    // Setup Logic
                                    let mut config = config::Config::new().unwrap_or_default();
                                    match crypto::hash_data(password.as_bytes()) {
                                        Ok(hashed) => {
                                            config.master_password_hash = Some(hashed.hash);
                                            if config.save_config().is_ok() {
                                                if let Ok(path) = keystore_path() {
                                                    let _ =
                                                        keystore::create_key_vault(password, &path);
                                                }
                                                transition_to_login = true;
                                            } else {
                                                *error = Some("Failed to save config".into());
                                            }
                                        }
                                        Err(e) => {
                                            *error = Some(format!("Hash error: {}", e));
                                        }
                                    }
                                } else {
                                    *error = Some("Passwords do not match".into());
                                    confirm.clear();
                                    *stage = SetupStage::EnterPassword; // Reset to first stage or stay? Let's reset purely confirm or just clear confirm.
                                    // Let's reset confirm but keep password for retry? Usually reset confirm is enough.
                                    // But to be safe lets modify flow: stay in confirm but it's cleared.
                                    // If user typed wrong first time, they can't see it.
                                    // Better UX: Go back to start
                                    password.clear();
                                    *stage = SetupStage::EnterPassword;
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => match stage {
                        SetupStage::EnterPassword => {
                            password.pop();
                        }
                        SetupStage::ConfirmPassword => {
                            confirm.pop();
                        }
                    },
                    KeyCode::Char(c) => match stage {
                        SetupStage::EnterPassword => password.push(c),
                        SetupStage::ConfirmPassword => confirm.push(c),
                    },
                    _ => {}
                }
            }
            AppState::RemoveConfirmation { platform, user_id } => match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    if let Ok(db) = Database::new() {
                        if let Ok(repository) = EntryRepository::new(&db, None) {
                            let _ = repository.remove(EntrySelector::new(platform, user_id));
                        }
                    }
                    transition_to_main = true;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    transition_to_main = true;
                }
                _ => {}
            },
            AppState::EditEntry {
                platform,
                original_user_id,
                user_id,
                password,
                stage,
            } => {
                match key.code {
                    KeyCode::Enter => {
                        match stage {
                            EditEntryStage::UserId => {
                                if !user_id.value.is_empty() {
                                    *stage = EditEntryStage::Password;
                                }
                            }
                            EditEntryStage::Password => {
                                if !password.value.is_empty() {
                                    entry_save_result = Some(update_entry(
                                        platform,
                                        original_user_id,
                                        &user_id.value,
                                        password.value.as_bytes(),
                                    ));
                                }
                            }
                        }
                    }
                    KeyCode::Left => match stage {
                        EditEntryStage::UserId => user_id.move_cursor_left(),
                        EditEntryStage::Password => password.move_cursor_left(),
                    },
                    KeyCode::Right => match stage {
                        EditEntryStage::UserId => user_id.move_cursor_right(),
                        EditEntryStage::Password => password.move_cursor_right(),
                    },
                    KeyCode::Char(c) => match stage {
                        EditEntryStage::UserId => user_id.insert_char(c),
                        EditEntryStage::Password => password.insert_char(c),
                    },
                    KeyCode::Backspace => match stage {
                        EditEntryStage::UserId => user_id.delete_char(),
                        EditEntryStage::Password => password.delete_char(),
                    },
                    KeyCode::Up => {
                        if let EditEntryStage::Password = stage {
                            *stage = EditEntryStage::UserId;
                        }
                    }
                    KeyCode::Down => {
                        if let EditEntryStage::UserId = stage {
                            *stage = EditEntryStage::Password;
                        }
                    }
                    KeyCode::Esc => {
                        transition_to_main = true;
                    }
                    _ => {}
                }
            }
            AppState::AddEntry {
                platform,
                user_id,
                password,
                stage,
            } => {
                match key.code {
                    KeyCode::Esc => transition_to_main = true,
                    KeyCode::Enter => {
                        match stage {
                            AddEntryStage::Platform => {
                                if !platform.value.is_empty() {
                                    *stage = AddEntryStage::UserId;
                                }
                            }
                            AddEntryStage::UserId => {
                                if !user_id.value.is_empty() {
                                    *stage = AddEntryStage::Password;
                                }
                            }
                            AddEntryStage::Password => {
                                if !password.value.is_empty() {
                                    entry_save_result = Some(save_entry(
                                        &platform.value,
                                        &user_id.value,
                                        password.value.as_bytes(),
                                    ));
                                }
                            }
                        }
                    }
                    KeyCode::Left => match stage {
                        AddEntryStage::Platform => platform.move_cursor_left(),
                        AddEntryStage::UserId => user_id.move_cursor_left(),
                        AddEntryStage::Password => password.move_cursor_left(),
                    },
                    KeyCode::Right => match stage {
                        AddEntryStage::Platform => platform.move_cursor_right(),
                        AddEntryStage::UserId => user_id.move_cursor_right(),
                        AddEntryStage::Password => password.move_cursor_right(),
                    },
                    KeyCode::Backspace => match stage {
                        AddEntryStage::Platform => platform.delete_char(),
                        AddEntryStage::UserId => user_id.delete_char(),
                        AddEntryStage::Password => password.delete_char(),
                    },
                    KeyCode::Up => match stage {
                        AddEntryStage::Platform => {}
                        AddEntryStage::UserId => *stage = AddEntryStage::Platform,
                        AddEntryStage::Password => *stage = AddEntryStage::UserId,
                    },
                    KeyCode::Down => match stage {
                        AddEntryStage::Platform => *stage = AddEntryStage::UserId,
                        AddEntryStage::UserId => *stage = AddEntryStage::Password,
                        AddEntryStage::Password => {}
                    },
                    KeyCode::Char(c) => match stage {
                        AddEntryStage::Platform => platform.insert_char(c),
                        AddEntryStage::UserId => user_id.insert_char(c),
                        AddEntryStage::Password => password.insert_char(c),
                    },
                    _ => {}
                }
            }
            AppState::BackupCreate {
                path,
                password,
                stage,
            } => match key.code {
                KeyCode::Esc => transition_to_main = true,
                KeyCode::Enter => match stage {
                    BackupCreateStage::Path => *stage = BackupCreateStage::Password,
                    BackupCreateStage::Password => {
                        let result = config::Config::new()
                            .map_err(|e| e.to_string())
                            .and_then(|config| {
                                verify_backup_master_password(&config, &password.value)
                            })
                            .and_then(|_| {
                                backup::create_backup_file(
                                    &password.value,
                                    std::path::Path::new(&path.value),
                                )
                            });
                        match result {
                            Ok(_) => self.show_toast("Backup written!"),
                            Err(e) => self.auth_error = Some(e),
                        }
                        transition_to_main = true;
                    }
                },
                KeyCode::Up => *stage = BackupCreateStage::Path,
                KeyCode::Down => *stage = BackupCreateStage::Password,
                KeyCode::Left => {
                    active_backup_create_input(path, password, stage).move_cursor_left()
                }
                KeyCode::Right => {
                    active_backup_create_input(path, password, stage).move_cursor_right()
                }
                KeyCode::Backspace => {
                    active_backup_create_input(path, password, stage).delete_char()
                }
                KeyCode::Char(c) => {
                    active_backup_create_input(path, password, stage).insert_char(c)
                }
                _ => {}
            },
            AppState::BackupRestore {
                path,
                password,
                confirm,
                stage,
            } => {
                match key.code {
                    KeyCode::Esc => transition_to_main = true,
                    KeyCode::Enter => match stage {
                        BackupRestoreStage::Path => *stage = BackupRestoreStage::Password,
                        BackupRestoreStage::Password => *stage = BackupRestoreStage::Confirm,
                        BackupRestoreStage::Confirm => {
                            if confirm.value == "RESTORE" {
                                match backup::restore_backup_file(
                                    &password.value,
                                    std::path::Path::new(&path.value),
                                ) {
                                    Ok(_) => self.show_toast("Backup restored. Restart RVault."),
                                    Err(e) => self.auth_error = Some(e),
                                }
                                transition_to_login = true;
                            } else {
                                self.auth_error = Some("Type RESTORE to confirm.".into());
                            }
                        }
                    },
                    KeyCode::Up => {
                        *stage = match stage {
                            BackupRestoreStage::Path => BackupRestoreStage::Path,
                            BackupRestoreStage::Password => BackupRestoreStage::Path,
                            BackupRestoreStage::Confirm => BackupRestoreStage::Password,
                        }
                    }
                    KeyCode::Down => {
                        *stage = match stage {
                            BackupRestoreStage::Path => BackupRestoreStage::Password,
                            BackupRestoreStage::Password => BackupRestoreStage::Confirm,
                            BackupRestoreStage::Confirm => BackupRestoreStage::Confirm,
                        }
                    }
                    KeyCode::Left => active_backup_restore_input(path, password, confirm, stage)
                        .move_cursor_left(),
                    KeyCode::Right => active_backup_restore_input(path, password, confirm, stage)
                        .move_cursor_right(),
                    KeyCode::Backspace => {
                        active_backup_restore_input(path, password, confirm, stage).delete_char()
                    }
                    KeyCode::Char(c) => {
                        active_backup_restore_input(path, password, confirm, stage).insert_char(c)
                    }
                    _ => {}
                }
            }
            AppState::ExportEntry {
                platform,
                user_id,
                recipient,
                path,
                stage,
            } => match key.code {
                KeyCode::Esc => transition_to_main = true,
                KeyCode::Enter => match stage {
                    ExportEntryStage::Recipient => *stage = ExportEntryStage::Path,
                    ExportEntryStage::Path => {
                        match export_one_entry(platform, user_id, &recipient.value, &path.value) {
                            Ok(_) => self.show_toast("Export written!"),
                            Err(e) => self.auth_error = Some(e),
                        }
                        transition_to_main = true;
                    }
                },
                KeyCode::Up => *stage = ExportEntryStage::Recipient,
                KeyCode::Down => *stage = ExportEntryStage::Path,
                KeyCode::Left => active_export_input(recipient, path, stage).move_cursor_left(),
                KeyCode::Right => active_export_input(recipient, path, stage).move_cursor_right(),
                KeyCode::Backspace => active_export_input(recipient, path, stage).delete_char(),
                KeyCode::Char(c) => active_export_input(recipient, path, stage).insert_char(c),
                _ => {}
            },
            AppState::ImportExport { path } => match key.code {
                KeyCode::Esc => transition_to_main = true,
                KeyCode::Enter => match preview_import_conflicts(&path.value) {
                    Ok(0) => {
                        match import_export_file(&path.value, false, false) {
                            Ok((imported, skipped)) => {
                                self.show_toast(&format!("Imported {imported}, skipped {skipped}"));
                            }
                            Err(e) => self.auth_error = Some(e),
                        }
                        transition_to_main = true;
                    }
                    Ok(conflicts) => {
                        self.state = AppState::ImportExportConfirm {
                            path: path.value.clone(),
                            conflicts,
                        };
                    }
                    Err(e) => {
                        self.auth_error = Some(e);
                        transition_to_main = true;
                    }
                },
                KeyCode::Left => path.move_cursor_left(),
                KeyCode::Right => path.move_cursor_right(),
                KeyCode::Backspace => path.delete_char(),
                KeyCode::Char(c) => path.insert_char(c),
                _ => {}
            },
            AppState::ImportExportConfirm { path, conflicts } => match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    match import_export_file(path, true, false) {
                        Ok((imported, skipped)) => {
                            self.show_toast(&format!("Imported {imported}, skipped {skipped}"));
                        }
                        Err(e) => self.auth_error = Some(e),
                    }
                    transition_to_main = true;
                }
                KeyCode::Char('n') => {
                    match import_export_file(path, false, true) {
                        Ok((imported, skipped)) => {
                            self.show_toast(&format!("Imported {imported}, skipped {skipped}"));
                        }
                        Err(e) => self.auth_error = Some(e),
                    }
                    transition_to_main = true;
                }
                KeyCode::Esc | KeyCode::Char('q') => transition_to_main = true,
                _ => {
                    let _ = conflicts;
                }
            },
        }

        if let Some(result) = entry_save_result {
            transition_to_main = self.handle_entry_save_result(result);
        }

        if transition_to_main {
            self.state = AppState::MainTable;
            self.refresh_vault_list();
            self.auth_error = None;
        }

        if transition_to_login {
            self.state = AppState::Authentication(String::new());
        }

        Ok(false)
    }
}

fn active_backup_create_input<'a>(
    path: &'a mut InputState,
    password: &'a mut InputState,
    stage: &BackupCreateStage,
) -> &'a mut InputState {
    match stage {
        BackupCreateStage::Path => path,
        BackupCreateStage::Password => password,
    }
}

fn verify_backup_master_password(
    config: &config::Config,
    master_password: &str,
) -> Result<(), String> {
    let stored_hash = config
        .master_password_hash
        .as_deref()
        .ok_or_else(|| "RVault not set up.".to_string())?;
    if crypto::verify_password(master_password.as_bytes(), stored_hash) {
        Ok(())
    } else {
        Err("Invalid Password".to_string())
    }
}

fn active_backup_restore_input<'a>(
    path: &'a mut InputState,
    password: &'a mut InputState,
    confirm: &'a mut InputState,
    stage: &BackupRestoreStage,
) -> &'a mut InputState {
    match stage {
        BackupRestoreStage::Path => path,
        BackupRestoreStage::Password => password,
        BackupRestoreStage::Confirm => confirm,
    }
}

fn active_export_input<'a>(
    recipient: &'a mut InputState,
    path: &'a mut InputState,
    stage: &ExportEntryStage,
) -> &'a mut InputState {
    match stage {
        ExportEntryStage::Recipient => recipient,
        ExportEntryStage::Path => path,
    }
}

fn export_one_entry(
    platform: &str,
    user_id: &str,
    recipient: &str,
    path: &str,
) -> Result<(), String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    let repository = EntryRepository::new(&db, None).map_err(|e| e.to_string())?;
    let key = SessionKey::load().map_err(|e| e.to_string())?;
    let decrypted = repository
        .get(&key, EntrySelector::new(platform, user_id))
        .map_err(|e| e.to_string())?;
    let password = std::str::from_utf8(decrypted.secret.expose())
        .map_err(|error| error.to_string())?
        .to_string();
    let entry = portable_export::ExportEntry {
        platform: platform.to_string(),
        user_id: user_id.to_string(),
        password,
        pinned: decrypted.metadata.pinned,
        created_at: decrypted.metadata.created_at,
        updated_at: decrypted.metadata.updated_at,
    };
    let bytes = portable_export::create_export_bytes(recipient, &[entry])?;
    std::fs::write(path, bytes).map_err(|e| format!("write export: {e}"))
}

#[allow(deprecated)] // 1.4 import boundary: preserves the existing conflict preview.
fn preview_import_conflicts(path: &str) -> Result<usize, String> {
    let entries = decrypt_export_file(path)?;
    let db = Database::new().map_err(|e| e.to_string())?;
    let table = Table::new(&db, None).map_err(|e| e.to_string())?;
    entries.iter().try_fold(0, |count, entry| {
        table
            .entry_exists(&db, &entry.platform, &entry.user_id)
            .map(|exists| count + usize::from(exists))
            .map_err(|e| e.to_string())
    })
}

#[allow(deprecated)] // 1.4 import boundary: preserves imported timestamps and pin state.
fn import_export_file(
    path: &str,
    overwrite_all: bool,
    skip_all: bool,
) -> Result<(usize, usize), String> {
    let entries = decrypt_export_file(path)?;
    let db = Database::new().map_err(|e| e.to_string())?;
    let table = Table::new(&db, None).map_err(|e| e.to_string())?;
    let key = SessionKey::load().map_err(|e| e.to_string())?;
    let mut imported = 0;
    let mut skipped = 0;
    for entry in entries {
        let exists = table
            .entry_exists(&db, &entry.platform, &entry.user_id)
            .map_err(|e| e.to_string())?;
        if exists && skip_all {
            skipped += 1;
            continue;
        }
        if exists && !overwrite_all {
            skipped += 1;
            continue;
        }
        table
            .import_entry_with_key_result(&db, key.as_bytes(), &entry)
            .map_err(|e| e.to_string())?;
        imported += 1;
    }
    Ok((imported, skipped))
}

fn decrypt_export_file(path: &str) -> Result<Vec<portable_export::ExportEntry>, String> {
    let key = SessionKey::load().map_err(|e| e.to_string())?;
    let id = identity::load_or_create_identity(key.as_bytes())?;
    let bytes = std::fs::read(path).map_err(|e| format!("read export: {e}"))?;
    portable_export::decrypt_export_bytes(&id, &bytes)
}

fn sanitize_file_name(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('_');
    if sanitized.is_empty() {
        "rvault-export".to_string()
    } else {
        sanitized.to_string()
    }
}

fn recover_duplicate_add(
    add_result: Result<(), StorageError>,
    update: impl FnOnce() -> Result<(), StorageError>,
) -> Result<(), StorageError> {
    match add_result {
        Err(StorageError::Conflict) => update(),
        result => result,
    }
}

fn add_or_update_entry(
    repository: &EntryRepository<'_>,
    key: &rvault_core::SecretKey,
    platform: &str,
    user_id: &str,
    password: &[u8],
) -> Result<(), StorageError> {
    recover_duplicate_add(
        repository.add(key, NewEntry::new(platform, user_id, password)),
        || {
            repository.update(
                key,
                EntrySelector::new(platform, user_id),
                EntryUpdate::new(user_id, password),
            )
        },
    )
}

fn save_entry(platform: &str, user_id: &str, password: &[u8]) -> Result<(), String> {
    let db = Database::new().map_err(|error| error.to_string())?;
    let repository = EntryRepository::new(&db, None).map_err(|error| error.to_string())?;
    let key = SessionKey::load().map_err(|error| error.to_string())?;
    add_or_update_entry(&repository, &key, platform, user_id, password)
        .map_err(|error| error.to_string())
}

fn update_entry(
    platform: &str,
    original_user_id: &str,
    user_id: &str,
    password: &[u8],
) -> Result<(), String> {
    let db = Database::new().map_err(|error| error.to_string())?;
    let repository = EntryRepository::new(&db, None).map_err(|error| error.to_string())?;
    let key = SessionKey::load().map_err(|error| error.to_string())?;
    repository
        .update(
            &key,
            EntrySelector::new(platform, original_user_id),
            EntryUpdate::new(user_id, password),
        )
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    #[test]
    fn uppercase_q_is_text_while_entering_a_secret() {
        let mut app = App::new();
        app.state = AppState::AddEntry {
            platform: InputState::with_value("service".to_string()),
            user_id: InputState::with_value("account".to_string()),
            password: InputState::new(),
            stage: AddEntryStage::Password,
        };

        let should_quit = app
            .on_key(KeyEvent::new(
                KeyCode::Char('Q'),
                KeyModifiers::SHIFT,
            ))
            .expect("handle key");

        assert!(!should_quit);
        let AppState::AddEntry { password, .. } = &app.state else {
            panic!("expected add-entry state");
        };
        assert_eq!(password.value, "Q");
    }

    #[test]
    fn pasted_secret_is_inserted_atomically_including_uppercase_q() {
        let mut app = App::new();
        app.state = AppState::AddEntry {
            platform: InputState::with_value("service".to_string()),
            user_id: InputState::with_value("account".to_string()),
            password: InputState::new(),
            stage: AddEntryStage::Password,
        };

        app.on_paste("api-key-Qxy");

        let AppState::AddEntry { password, .. } = &app.state else {
            panic!("expected add-entry state");
        };
        assert_eq!(password.value, "api-key-Qxy");
        assert_eq!(password.cursor_position, "api-key-Qxy".len());
    }

    #[test]
    fn entry_save_error_is_shown_in_the_tui() {
        let mut app = App::new();

        let saved = app.handle_entry_save_result(Err("database unavailable".to_string()));

        assert!(!saved);
        let toast = app.toast.expect("save error toast");
        assert_eq!(
            toast.message,
            "Failed to save entry: database unavailable"
        );
    }

    #[test]
    fn duplicate_add_runs_update_and_propagates_a_delete_race() {
        let mut updated = false;
        assert!(
            recover_duplicate_add(Err(StorageError::Conflict), || {
                updated = true;
                Ok(())
            })
            .is_ok()
        );
        assert!(updated);
        assert!(matches!(
            recover_duplicate_add(Err(StorageError::Conflict), || {
                Err(StorageError::NotFound)
            }),
            Err(StorageError::NotFound)
        ));
    }

    #[test]
    fn backup_master_password_validation_rejects_wrong_password() {
        let hash = crypto::hash_data(b"correct-password")
            .expect("hash password")
            .hash;
        let config = config::Config {
            master_password_hash: Some(hash),
            ..Default::default()
        };

        assert!(verify_backup_master_password(&config, "correct-password").is_ok());
        assert!(verify_backup_master_password(&config, "wrong-password").is_err());
    }
}
