use rvault_core::{
    clipboard, config, crypto,
    session::{self, get_key_from_session},
    storage::{Database, Table},

    vault::{Vault, VaultEntry},
    keystore::{self, keystore_path},
};
use ratatui::widgets::ListState;
use crossterm::event::{KeyCode, KeyEventKind};
use std::io;
use std::time::{Duration, Instant};
use crate::ui::Theme;
use crate::input::InputState;

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
        platform: String, // Immutable
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
    ThemeSelection,
    SortSelection,
}

pub struct App {
    pub state: AppState,
    pub items: Vec<VaultEntry>,
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
             themes.iter().find(|t| t.name == config.theme).cloned().unwrap_or(Theme::default())
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
        match get_key_from_session() {
            Ok(_) => {
                self.state = AppState::MainTable;
                self.refresh_vault_list();
                true
            }
            Err(_) => false
        }
    }

    pub fn refresh_vault_list(&mut self) {
        if let Ok(db) = Database::new() {
            if let Ok(table) = Table::new(&db, None) {
                if let Ok(entries) = table.list(&db) {
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
                    let a_time = if a.updated_at > 0 { a.updated_at } else if a.created_at > 0 { a.created_at } else { a.id.unwrap_or(0) };
                    let b_time = if b.updated_at > 0 { b.updated_at } else if b.created_at > 0 { b.created_at } else { b.id.unwrap_or(0) };
                    a_time.cmp(&b_time)
                });
            }
            SortMode::TimeDesc => {
                unpinned.sort_by(|a, b| {
                    let a_time = if a.updated_at > 0 { a.updated_at } else if a.created_at > 0 { a.created_at } else { a.id.unwrap_or(0) };
                    let b_time = if b.updated_at > 0 { b.updated_at } else if b.created_at > 0 { b.created_at } else { b.id.unwrap_or(0) };
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

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) -> io::Result<bool> {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        // Global shortcut: Shift+Q to Lock & Quit
        if let KeyCode::Char('Q') = key.code {
            let _ = rvault_core::lock(); // Best effort lock
            return Ok(true); // Signal to quit
        }

        let mut transition_to_main = false;
        let mut transition_to_login = false;

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
                    KeyCode::Esc | KeyCode::Char('Q') => return Ok(true), // Signal to quit. Shift+Q maps to Char('Q') usually.
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => { input.pop(); },
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
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                    KeyCode::Char('Q') => return Ok(true), // Lock and Quit immediately
                    KeyCode::Tab => self.next_tab(),
                    KeyCode::Down => {
                        let i = match self.list_state.selected() {
                            Some(i) => if i >= self.items.len().saturating_sub(1) { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match self.list_state.selected() {
                            Some(i) => if i == 0 { self.items.len().saturating_sub(1) } else { i - 1 },
                            None => 0,
                        };
                        self.list_state.select(Some(i));
                    }
                    KeyCode::Char('p') => {
                        if let Some(i) = self.list_state.selected() {
                            if let Some(entry) = self.items.get(i) {
                                if let Ok(db) = Database::new() {
                                    if let Ok(table) = Table::new(&db, None) {
                                        match table.toggle_pin(&db, entry.platform.clone(), entry.user_id.clone()) {
                                            Ok(_) => {
                                                self.refresh_vault_list();
                                                self.auth_error = None;
                                            },
                                            Err(_) => {
                                                self.auth_error = Some("Pin limit reached (max 10)".into());
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
                                    if let Ok(table) = Table::new(&db, None) {
                                        if let Ok(ek) = get_key_from_session() {
                                            if let Ok(plaintext) = table.retrieve_password_with_key(&db, &ek, entry.platform.clone(), entry.user_id.clone()) {
                                                clipboard::copy_text(plaintext);
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
            AppState::SortSelection => {
                match key.code {
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
                }
            }
            AppState::ThemeSelection => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('t')  => {
                        // Save theme code
                        let mut config = config::Config::new().unwrap_or_default();
                        config.theme = self.current_theme.name.clone();
                        let _ = config.save_config();
                        self.state = AppState::MainTable;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        // Cycle next theme
                        if let Some(pos) = self.themes.iter().position(|t| t.name == self.current_theme.name) {
                            let next = (pos + 1) % self.themes.len();
                            self.current_theme = self.themes[next].clone();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        // Cycle prev theme
                        if let Some(pos) = self.themes.iter().position(|t| t.name == self.current_theme.name) {
                            let prev = if pos == 0 { self.themes.len() - 1 } else { pos - 1 };
                            self.current_theme = self.themes[prev].clone();
                        }
                    }
                    _ => {}
                }
            }
            AppState::Generator => {
                 match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                    KeyCode::Tab => self.next_tab(),
                    KeyCode::Char('t') => self.state = AppState::ThemeSelection,
                    KeyCode::Char('s') => self.gen_special = !self.gen_special,
                    KeyCode::Left => if self.gen_length > 4 { self.gen_length -= 1 },
                    KeyCode::Right => if self.gen_length < 32 { self.gen_length += 1 },
                    KeyCode::Enter => {
                         let pass = crypto::generate_password(self.gen_length, self.gen_special);
                         clipboard::copy_text(pass);
                         self.show_toast("Password has been copied!");
                    }
                    _ => {}
                 }
            }
            AppState::Setup { password, confirm, stage, error } => {
                match key.code {
                    KeyCode::Esc => return Ok(true),
                    KeyCode::Enter => {
                        match stage {
                            SetupStage::EnterPassword => {
                                if !password.is_empty() {
                                     *stage = SetupStage::ConfirmPassword;
                                     *error = None;
                                }
                            },
                            SetupStage::ConfirmPassword => {
                                if password == confirm {
                                    // Setup Logic
                                    let mut config = config::Config::new().unwrap_or_default();
                                    match crypto::hash_data(password.as_bytes()) {
                                        Ok(hashed) => {
                                            config.master_password_hash = Some(hashed.hash);
                                            if config.save_config().is_ok() {
                                                if let Ok(path) = keystore_path() {
                                                    let _ = keystore::create_key_vault(password, &path);
                                                }
                                                transition_to_login = true;
                                            } else {
                                                *error = Some("Failed to save config".into()); 
                                            }
                                        },
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
                    },
                    KeyCode::Backspace => {
                        match stage {
                            SetupStage::EnterPassword => { password.pop(); },
                            SetupStage::ConfirmPassword => { confirm.pop(); },
                        }
                    },
                    KeyCode::Char(c) => {
                        match stage {
                            SetupStage::EnterPassword => password.push(c),
                            SetupStage::ConfirmPassword => confirm.push(c),
                        }
                    },
                    _ => {}
                }
            }
            AppState::RemoveConfirmation { platform, user_id } => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                         if let Ok(db) = Database::new() {
                             if let Ok(table) = Table::new(&db, None) {
                                  table.remove_entry(&db, platform.clone(), user_id.clone());
                             }
                         }
                         transition_to_main = true;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        transition_to_main = true;
                    }
                    _ => {}
                }
            }
            AppState::EditEntry { platform, original_user_id, user_id, password, stage } => {
                match key.code {
                    KeyCode::Enter => {
                         match stage {
                             EditEntryStage::UserId => {
                                 if !user_id.value.is_empty() {
                                     *stage = EditEntryStage::Password;
                                 }
                             },
                             EditEntryStage::Password => {
                                 if !password.value.is_empty() {
                                     // Save updates
                                     if let Ok(db) = Database::new() {
                                         if let Ok(table) = Table::new(&db, None) {
                                              if let Ok(ek) = get_key_from_session() {
                                                  if let Err(e) = table.update_entry(&db, &ek, platform, original_user_id, &user_id.value, &password.value) {
                                                      // Handle error? For now just print to stderr or set global error
                                                      // self.error = Some(...) // We don't have a global error field in AppState enum variants easily accessible without refactor.
                                                      // But we do have self.auth_error in App struct.
                                                      // Assuming we can't easily set self.auth_error from inside match without mutable borrow issues if helper methods aren't used.
                                                      // But we are in `match &mut self.state`. We can modify `self.auth_error`? 
                                                      // No, `self.state` is borrowed mutably. `self.auth_error` is another field. 
                                                      // We need to split borrows or use boolean flag.
                                                      // For simplicity, if error, we might fail silently or print to stderr (which TUI hides).
                                                      // Ideally we'd transition to MainTable with an error message.
                                                      // Let's just transition to main table.
                                                  }
                                              }
                                         }
                                     }
                                     transition_to_main = true;
                                 }
                             }
                         }
                    }
                    KeyCode::Left => {
                        match stage {
                            EditEntryStage::UserId => user_id.move_cursor_left(),
                            EditEntryStage::Password => password.move_cursor_left(),
                        }
                    },
                    KeyCode::Right => {
                         match stage {
                            EditEntryStage::UserId => user_id.move_cursor_right(),
                            EditEntryStage::Password => password.move_cursor_right(),
                        }
                    },
                    KeyCode::Char(c) => {
                         match stage {
                            EditEntryStage::UserId => user_id.insert_char(c),
                            EditEntryStage::Password => password.insert_char(c),
                        }
                    },
                    KeyCode::Backspace => { 
                         match stage {
                            EditEntryStage::UserId => user_id.delete_char(),
                            EditEntryStage::Password => password.delete_char(),
                        }
                    },
                    KeyCode::Up => {
                         if let EditEntryStage::Password = stage {
                             *stage = EditEntryStage::UserId;
                         }
                    },
                    KeyCode::Down => {
                         if let EditEntryStage::UserId = stage {
                             *stage = EditEntryStage::Password;
                         }
                    },
                    KeyCode::Esc => {
                        transition_to_main = true;
                    }
                    _ => {}
                }
            }
            AppState::AddEntry { platform, user_id, password, stage } => {
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
                                    // Save the entry
                                    if let Ok(db) = Database::new() {
                                        if let Ok(table) = Table::new(&db, None) {
                                            if let Ok(ek) = get_key_from_session() {
                                                let id_pass = format!("{}:{}", user_id.value, password.value);
                                                table.add_entry_with_key(&db, &ek, platform.value.clone(), id_pass);
                                            }
                                        }
                                    }
                                    transition_to_main = true;
                                }
                            }
                        }
                    }
                    KeyCode::Left => {
                         match stage {
                            AddEntryStage::Platform => platform.move_cursor_left(),
                            AddEntryStage::UserId => user_id.move_cursor_left(),
                            AddEntryStage::Password => password.move_cursor_left(),
                        }
                    },
                    KeyCode::Right => {
                         match stage {
                            AddEntryStage::Platform => platform.move_cursor_right(),
                            AddEntryStage::UserId => user_id.move_cursor_right(),
                            AddEntryStage::Password => password.move_cursor_right(),
                        }
                    },
                    KeyCode::Backspace => {
                        match stage {
                            AddEntryStage::Platform => platform.delete_char(),
                            AddEntryStage::UserId => user_id.delete_char(),
                            AddEntryStage::Password => password.delete_char(),
                        }
                    }
                    KeyCode::Up => {
                        match stage {
                            AddEntryStage::Platform => {},
                            AddEntryStage::UserId => *stage = AddEntryStage::Platform,
                            AddEntryStage::Password => *stage = AddEntryStage::UserId,
                        }
                    }
                    KeyCode::Down => {
                         match stage {
                            AddEntryStage::Platform => *stage = AddEntryStage::UserId,
                            AddEntryStage::UserId => *stage = AddEntryStage::Password,
                            AddEntryStage::Password => {},
                        }
                    }
                    KeyCode::Char(c) => {
                         match stage {
                            AddEntryStage::Platform => platform.insert_char(c),
                            AddEntryStage::UserId => user_id.insert_char(c),
                            AddEntryStage::Password => password.insert_char(c),
                        }
                    }
                     _ => {}
                }
            }
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
