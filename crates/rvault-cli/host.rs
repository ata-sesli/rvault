use crate::cli::{BrowserCommands, HostCommands};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const HOST_NAME: &str = "io.github.ata_sesli.rvault";
pub const RVAULT_HELIUM_EXTENSION_ID: &str = "gnfmkmiklgghclejbbdmjgcldajahfhh";
const HOST_DESCRIPTION: &str = "RVault native messaging host";
const HELIUM_APP_SUPPORT: &str = "Library/Application Support/net.imput.helium";

pub fn is_native_messaging_launch(first_arg: Option<&str>) -> bool {
    first_arg
        .map(|arg| arg.starts_with("chrome-extension://") || arg.starts_with("moz-extension://"))
        .unwrap_or(false)
}

pub fn handle_browser_command(command: &BrowserCommands) -> Result<(), String> {
    match command {
        BrowserCommands::Enable => enable(),
        BrowserCommands::Disable => disable(),
    }
}

pub fn handle_host_command(command: &HostCommands) -> Result<(), String> {
    match command {
        HostCommands::Serve => crate::native::serve_stdio(),
    }
}

fn enable() -> Result<(), String> {
    let exe =
        env::current_exe().map_err(|e| format!("failed to find current rvault executable: {e}"))?;
    let exe = exe
        .to_str()
        .ok_or_else(|| "rvault executable path is not valid UTF-8".to_string())?;
    let manifest = build_helium_manifest(exe)?;
    let path = helium_manifest_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create native host directory: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("failed to serialize native host manifest: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("failed to write native host manifest: {e}"))?;
    println!(
        "RVault browser integration enabled for Helium at {}",
        path.display()
    );
    Ok(())
}

fn disable() -> Result<(), String> {
    let path = helium_manifest_path()?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("failed to remove native host manifest: {e}"))?;
        println!("RVault browser integration disabled for Helium.");
    } else {
        println!("RVault browser integration is not enabled for Helium.");
    }
    Ok(())
}

pub fn helium_manifest_path() -> Result<PathBuf, String> {
    let home = env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        "HOME is not set; cannot locate Helium native messaging directory".to_string()
    })?;
    Ok(helium_manifest_path_from_home(&home))
}

pub fn helium_manifest_path_from_home(home: &Path) -> PathBuf {
    home.join(HELIUM_APP_SUPPORT)
        .join("NativeMessagingHosts")
        .join(format!("{HOST_NAME}.json"))
}

pub fn build_helium_manifest(rvault_path: &str) -> Result<Value, String> {
    validate_chromium_extension_id(RVAULT_HELIUM_EXTENSION_ID)?;
    Ok(json!({
        "name": HOST_NAME,
        "description": HOST_DESCRIPTION,
        "path": rvault_path,
        "type": "stdio",
        "allowed_origins": [format!("chrome-extension://{RVAULT_HELIUM_EXTENSION_ID}/")]
    }))
}

fn validate_chromium_extension_id(extension_id: &str) -> Result<(), String> {
    let is_valid = extension_id.len() == 32 && extension_id.chars().all(|c| matches!(c, 'a'..='p'));
    if is_valid {
        Ok(())
    } else {
        Err(
            "extension id must be a 32-character Chromium extension id using letters a-p"
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn helium_manifest_path_uses_helium_native_messaging_directory() {
        let home = PathBuf::from("/Users/example");

        let path = helium_manifest_path_from_home(&home);

        assert_eq!(
            path,
            PathBuf::from(
                "/Users/example/Library/Application Support/net.imput.helium/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            )
        );
    }

    #[test]
    fn native_manifest_points_to_rvault_binary_and_known_extension() {
        let manifest = build_helium_manifest("/usr/local/bin/rvault").expect("build manifest");

        assert_eq!(manifest["name"], HOST_NAME);
        assert_eq!(manifest["path"], "/usr/local/bin/rvault");
        assert_eq!(manifest["type"], "stdio");
        assert!(manifest.get("args").is_none());
        assert_eq!(
            manifest["allowed_origins"],
            serde_json::json!([format!("chrome-extension://{RVAULT_HELIUM_EXTENSION_ID}/")])
        );
    }

    #[test]
    fn detects_chromium_native_launch_from_origin_argument() {
        assert!(is_native_messaging_launch(Some(
            "chrome-extension://abcdefghijklmnopabcdefghijklmnop/"
        )));
        assert!(!is_native_messaging_launch(Some("host")));
        assert!(!is_native_messaging_launch(None));
    }

    #[test]
    fn bundled_extension_id_is_a_valid_chromium_extension_id() {
        validate_chromium_extension_id(RVAULT_HELIUM_EXTENSION_ID)
            .expect("bundled extension id should be valid");
    }
}
