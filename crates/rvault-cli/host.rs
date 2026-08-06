use crate::cli::{Browser, BrowserCommands, HostCommands};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const HOST_NAME: &str = "io.github.ata_sesli.rvault";
pub const RVAULT_HELIUM_EXTENSION_ID: &str = "gnfmkmiklgghclejbbdmjgcldajahfhh";
pub const RVAULT_FIREFOX_EXTENSION_ID: &str = "rvault@ata-sesli.github.io";
const HOST_DESCRIPTION: &str = "RVault native messaging host";

#[cfg(any(target_os = "macos", target_os = "linux", test))]
#[allow(dead_code)] // Each runtime build constructs only its current platform variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Platform {
    Macos,
    Linux,
}

pub fn is_native_messaging_launch(first_arg: Option<&str>, second_arg: Option<&str>) -> bool {
    let chromium_launch = first_arg.is_some_and(|arg| arg.starts_with("chrome-extension://"));
    let firefox_launch = matches!(
        (first_arg, second_arg),
        (Some(manifest_path), Some(addon_id))
            if manifest_path.ends_with(&format!("{HOST_NAME}.json"))
                && addon_id == RVAULT_FIREFOX_EXTENSION_ID
    );
    chromium_launch || firefox_launch
}

pub fn handle_browser_command(command: &BrowserCommands) -> Result<(), String> {
    match command {
        BrowserCommands::Enable { browser } => enable(*browser),
        BrowserCommands::Disable { browser } => disable(*browser),
    }
}

pub fn handle_host_command(command: &HostCommands) -> Result<(), String> {
    match command {
        HostCommands::Serve => crate::native::serve_stdio(),
    }
}

fn enable(browser: Browser) -> Result<(), String> {
    let exe =
        env::current_exe().map_err(|e| format!("failed to find current rvault executable: {e}"))?;
    let exe = exe
        .to_str()
        .ok_or_else(|| "rvault executable path is not valid UTF-8".to_string())?;
    let manifest = build_manifest(browser, exe)?;
    let path = manifest_path(browser)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create native host directory: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("failed to serialize native host manifest: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("failed to write native host manifest: {e}"))?;

    #[cfg(target_os = "windows")]
    register_windows_host(browser, &path)?;

    println!(
        "RVault browser integration enabled for {} at {}",
        browser_name(browser),
        path.display()
    );
    Ok(())
}

fn disable(browser: Browser) -> Result<(), String> {
    let path = manifest_path(browser)?;

    #[cfg(target_os = "windows")]
    unregister_windows_host(browser)?;

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("failed to remove native host manifest: {e}"))?;
        println!(
            "RVault browser integration disabled for {}.",
            browser_name(browser)
        );
    } else {
        println!(
            "RVault browser integration is not enabled for {}.",
            browser_name(browser)
        );
    }
    Ok(())
}

fn browser_name(browser: Browser) -> &'static str {
    match browser {
        Browser::Helium => "Helium",
        Browser::Chrome => "Google Chrome",
        Browser::Chromium => "Chromium",
        Browser::Firefox => "Firefox",
    }
}

#[cfg(not(target_os = "windows"))]
fn manifest_path(browser: Browser) -> Result<PathBuf, String> {
    let home = env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        format!(
            "HOME is not set; cannot locate the {} native messaging directory",
            browser_name(browser)
        )
    })?;

    #[cfg(target_os = "macos")]
    let platform = Platform::Macos;
    #[cfg(target_os = "linux")]
    let platform = Platform::Linux;
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err(format!(
        "{} browser integration is not supported on this platform",
        browser_name(browser)
    ));

    manifest_path_from_home(browser, platform, &home)
}

#[cfg(target_os = "windows")]
fn manifest_path(browser: Browser) -> Result<PathBuf, String> {
    windows_registry_key(browser)?;
    let local_app_data = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| {
            "LOCALAPPDATA is not set; cannot install native messaging host".to_string()
        })?;
    Ok(local_app_data
        .join("RVault")
        .join("NativeMessagingHosts")
        .join(format!("{HOST_NAME}.json")))
}

#[cfg(any(target_os = "macos", target_os = "linux", test))]
fn manifest_path_from_home(
    browser: Browser,
    platform: Platform,
    home: &Path,
) -> Result<PathBuf, String> {
    let directory = match (browser, platform) {
        (Browser::Helium, Platform::Macos) => {
            "Library/Application Support/net.imput.helium/NativeMessagingHosts"
        }
        (Browser::Chrome, Platform::Macos) => {
            "Library/Application Support/Google/Chrome/NativeMessagingHosts"
        }
        (Browser::Chromium, Platform::Macos) => {
            "Library/Application Support/Chromium/NativeMessagingHosts"
        }
        (Browser::Firefox, Platform::Macos) => {
            "Library/Application Support/Mozilla/NativeMessagingHosts"
        }
        (Browser::Chrome, Platform::Linux) => ".config/google-chrome/NativeMessagingHosts",
        (Browser::Chromium, Platform::Linux) => ".config/chromium/NativeMessagingHosts",
        (Browser::Firefox, Platform::Linux) => ".mozilla/native-messaging-hosts",
        (Browser::Helium, Platform::Linux) => {
            return Err("Helium browser integration is only supported on macOS".to_string());
        }
    };

    Ok(home.join(directory).join(format!("{HOST_NAME}.json")))
}

fn build_manifest(browser: Browser, rvault_path: &str) -> Result<Value, String> {
    let mut manifest = json!({
        "name": HOST_NAME,
        "description": HOST_DESCRIPTION,
        "path": rvault_path,
        "type": "stdio"
    });

    match browser {
        Browser::Helium | Browser::Chrome | Browser::Chromium => {
            validate_chromium_extension_id(RVAULT_HELIUM_EXTENSION_ID)?;
            manifest["allowed_origins"] =
                json!([format!("chrome-extension://{RVAULT_HELIUM_EXTENSION_ID}/")]);
        }
        Browser::Firefox => {
            manifest["allowed_extensions"] = json!([RVAULT_FIREFOX_EXTENSION_ID]);
        }
    }

    Ok(manifest)
}

#[cfg(any(target_os = "windows", test))]
fn windows_registry_key(browser: Browser) -> Result<String, String> {
    let vendor = match browser {
        Browser::Chrome => "Google\\Chrome",
        Browser::Chromium => "Chromium",
        Browser::Firefox => "Mozilla",
        Browser::Helium => {
            return Err("Helium browser integration is only supported on macOS".to_string());
        }
    };
    Ok(format!(
        r"Software\{vendor}\NativeMessagingHosts\{HOST_NAME}"
    ))
}

#[cfg(target_os = "windows")]
fn register_windows_host(browser: Browser, manifest_path: &Path) -> Result<(), String> {
    use std::process::Command;

    let key_path = format!(r"HKCU\{}", windows_registry_key(browser)?);
    let output = Command::new("reg")
        .args(["ADD", &key_path, "/ve", "/t", "REG_SZ", "/d"])
        .arg(manifest_path)
        .args(["/f"])
        .output()
        .map_err(|e| format!("failed to run Windows registry command: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to register native host manifest: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(target_os = "windows")]
fn unregister_windows_host(browser: Browser) -> Result<(), String> {
    use std::process::Command;

    let key_path = format!(r"HKCU\{}", windows_registry_key(browser)?);
    let query = Command::new("reg")
        .args(["QUERY", &key_path])
        .output()
        .map_err(|e| format!("failed to run Windows registry command: {e}"))?;
    if !query.status.success() {
        return Ok(());
    }

    let output = Command::new("reg")
        .args(["DELETE", &key_path, "/f"])
        .output()
        .map_err(|e| format!("failed to run Windows registry command: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to remove native host registry key: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
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
    fn macos_manifest_paths_are_browser_specific() {
        let home = PathBuf::from("/Users/example");

        assert_eq!(
            manifest_path_from_home(Browser::Helium, Platform::Macos, &home).expect("Helium path"),
            PathBuf::from(
                "/Users/example/Library/Application Support/net.imput.helium/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            )
        );
        assert_eq!(
            manifest_path_from_home(Browser::Chrome, Platform::Macos, &home).expect("Chrome path"),
            PathBuf::from(
                "/Users/example/Library/Application Support/Google/Chrome/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            )
        );
        assert_eq!(
            manifest_path_from_home(Browser::Chromium, Platform::Macos, &home)
                .expect("Chromium path"),
            PathBuf::from(
                "/Users/example/Library/Application Support/Chromium/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            )
        );
        assert_eq!(
            manifest_path_from_home(Browser::Firefox, Platform::Macos, &home)
                .expect("Firefox path"),
            PathBuf::from(
                "/Users/example/Library/Application Support/Mozilla/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            )
        );
    }

    #[test]
    fn linux_manifest_paths_are_browser_specific() {
        let home = PathBuf::from("/home/example");

        assert_eq!(
            manifest_path_from_home(Browser::Chrome, Platform::Linux, &home).expect("Chrome path"),
            PathBuf::from(
                "/home/example/.config/google-chrome/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            )
        );
        assert_eq!(
            manifest_path_from_home(Browser::Chromium, Platform::Linux, &home)
                .expect("Chromium path"),
            PathBuf::from(
                "/home/example/.config/chromium/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            )
        );
        assert_eq!(
            manifest_path_from_home(Browser::Firefox, Platform::Linux, &home)
                .expect("Firefox path"),
            PathBuf::from(
                "/home/example/.mozilla/native-messaging-hosts/io.github.ata_sesli.rvault.json"
            )
        );
        assert!(manifest_path_from_home(Browser::Helium, Platform::Linux, &home).is_err());
    }

    #[test]
    fn chromium_manifest_uses_the_pinned_origin() {
        let manifest =
            build_manifest(Browser::Chrome, "/usr/local/bin/rvault").expect("build manifest");

        assert_eq!(manifest["name"], HOST_NAME);
        assert_eq!(manifest["path"], "/usr/local/bin/rvault");
        assert_eq!(manifest["type"], "stdio");
        assert!(manifest.get("args").is_none());
        assert_eq!(
            manifest["allowed_origins"],
            serde_json::json!([format!("chrome-extension://{RVAULT_HELIUM_EXTENSION_ID}/")])
        );
        assert!(manifest.get("allowed_extensions").is_none());
    }

    #[test]
    fn firefox_manifest_uses_the_pinned_addon_id() {
        let manifest =
            build_manifest(Browser::Firefox, "/usr/local/bin/rvault").expect("build manifest");

        assert_eq!(
            manifest["allowed_extensions"],
            serde_json::json!([RVAULT_FIREFOX_EXTENSION_ID])
        );
        assert!(manifest.get("allowed_origins").is_none());
    }

    #[test]
    fn detects_browser_native_launch_arguments() {
        assert!(is_native_messaging_launch(
            Some("chrome-extension://abcdefghijklmnopabcdefghijklmnop/"),
            None
        ));
        assert!(is_native_messaging_launch(
            Some(
                "/Users/example/Library/Application Support/Mozilla/NativeMessagingHosts/io.github.ata_sesli.rvault.json"
            ),
            Some(RVAULT_FIREFOX_EXTENSION_ID)
        ));
        assert!(!is_native_messaging_launch(
            Some("/tmp/unknown.json"),
            Some(RVAULT_FIREFOX_EXTENSION_ID)
        ));
        assert!(!is_native_messaging_launch(
            Some("/tmp/io.github.ata_sesli.rvault.json"),
            Some("unknown@example.com")
        ));
        assert!(!is_native_messaging_launch(Some("host"), None));
        assert!(!is_native_messaging_launch(None, None));
    }

    #[test]
    fn windows_registry_keys_are_browser_specific() {
        assert_eq!(
            windows_registry_key(Browser::Chrome).expect("Chrome registry key"),
            format!(r"Software\Google\Chrome\NativeMessagingHosts\{HOST_NAME}")
        );
        assert_eq!(
            windows_registry_key(Browser::Chromium).expect("Chromium registry key"),
            format!(r"Software\Chromium\NativeMessagingHosts\{HOST_NAME}")
        );
        assert_eq!(
            windows_registry_key(Browser::Firefox).expect("Firefox registry key"),
            format!(r"Software\Mozilla\NativeMessagingHosts\{HOST_NAME}")
        );
        assert!(windows_registry_key(Browser::Helium).is_err());
    }

    #[test]
    fn bundled_extension_id_is_a_valid_chromium_extension_id() {
        validate_chromium_extension_id(RVAULT_HELIUM_EXTENSION_ID)
            .expect("bundled extension id should be valid");
    }
}
