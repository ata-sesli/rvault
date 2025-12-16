slint::include_modules!();

use rvault_core::{config, session, vault};

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let ui_handle = ui.as_weak();

    ui.on_unlock(move |password: slint::SharedString| {
        let password = password.to_string();
        let ui = ui_handle.upgrade().unwrap();
        let config = config::Config::new().unwrap();
        if let Some(stored_hash) = &config.master_password_hash {
            match vault::Vault::get_encryption_key(&password, stored_hash) {
                Ok(encryption_key) => {
                    match session::start_session(&encryption_key) {
                        Ok(token) => {
                            session::write_current(&token).unwrap();
                            ui.set_status("Vault unlocked successfully!".into());
                        }
                        Err(e) => {
                            ui.set_status(format!("Failed to start session: {}", e).into());
                        }
                    }
                }
                Err(e) => {
                    ui.set_status(format!("Invalid password: {}", e).into());
                }
            }
        } else {
            ui.set_status("Vault not set up. Please run 'rvault setup' first.".into());
        }
    });

    ui.run()
}