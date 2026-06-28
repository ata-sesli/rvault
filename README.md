# RVault

RVault is a local-first password manager written in Rust, with a terminal UI, a CLI, and a Helium browser extension that talks to the local `rvault` binary through native messaging.

Current version: `1.2.0`.

RVault keeps storage local. Passwords are encrypted before they are written to SQLite, browser integration goes through a local native host, and the extension does not store plaintext credentials.

## Contents

- [Install RVault](#install-rvault)
- [Install the Helium Extension Locally](#install-the-helium-extension-locally)
- [Quick Start](#quick-start)
- [Backup and Restore](#backup-and-restore)
- [Encrypted Export and Import](#encrypted-export-and-import)
- [TUI Keybindings](#tui-keybindings)
- [How RVault Works](#how-rvault-works)
- [Build and Test From Source](#build-and-test-from-source)
- [Release Checklist](#release-checklist)
- [Current Boundaries](#current-boundaries)
- [License](#license)

## Install RVault

### From GitHub Releases

After the `v1.2.0` release is published, install the CLI from the release assets:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ata-sesli/rvault/releases/download/v1.2.0/rvault-cli-installer.sh | sh
```

On Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ata-sesli/rvault/releases/download/v1.2.0/rvault-cli-installer.ps1 | iex"
```

Confirm the install:

```bash
rvault --version
```

Expected version:

```text
rvault-cli 1.2.0
```

### From Source

Requirements:

- Rust toolchain with Cargo
- Bun, only needed for the browser extension
- Helium on macOS, only needed for browser integration

Install the CLI from this repository:

```bash
git clone https://github.com/ata-sesli/rvault.git
cd rvault
cargo install --path crates/rvault-cli --force
rvault --version
```

Run the terminal UI:

```bash
rvault
```

## Install the Helium Extension Locally

The RVault extension is not published in the Chrome Web Store for `1.2.0`. Install it locally in Helium.

This v1 path targets **Helium on macOS**. `rvault browser enable` writes Helium's native messaging manifest here:

```text
~/Library/Application Support/net.imput.helium/NativeMessagingHosts/io.github.ata_sesli.rvault.json
```

### Option A: Install From a Release ZIP

1. Install the `rvault` CLI first.

2. Download `rvault-extension-1.2.0.zip` from the `v1.2.0` GitHub release.

3. Unzip it somewhere stable, for example:

```bash
mkdir -p ~/Applications/rvault-extension
unzip rvault-extension-1.2.0.zip -d ~/Applications/rvault-extension
```

4. Open Helium and go to:

```text
chrome://extensions
```

5. Enable **Developer mode**.

6. Click **Load unpacked**.

7. Select:

```text
~/Applications/rvault-extension
```

8. Register RVault as Helium's native messaging host:

```bash
rvault browser enable
```

9. Click the RVault toolbar icon in Helium.

If you move or reinstall the `rvault` binary, run this again:

```bash
rvault browser enable
```

To remove the native messaging registration:

```bash
rvault browser disable
```

### Option B: Build the Extension Locally

From the repository root:

```bash
cd extension
bun install
bun run build
```

Then load this folder in Helium:

```text
extension/build/chrome-mv3-prod
```

After loading the extension, register the native host:

```bash
rvault browser enable
```

Do not load the top-level `extension/` folder. Helium must load the built `chrome-mv3-prod` folder.

### Troubleshooting Browser Integration

If the extension says the native host is unavailable:

```bash
rvault browser disable
rvault browser enable
```

Then reload the extension in `chrome://extensions`.

The extension ID is pinned by the manifest key. The expected Helium extension ID is:

```text
gnfmkmiklgghclejbbdmjgcldajahfhh
```

## Quick Start

Set up RVault once:

```bash
rvault setup
```

Unlock the vault:

```bash
rvault unlock
```

Add a credential:

```bash
rvault add github alice:correct-horse-battery-staple
```

Copy a credential password to the clipboard:

```bash
rvault get github alice
```

Generate a password and copy it to the clipboard:

```bash
rvault generate --length 20 --special-characters
```

Launch the terminal UI:

```bash
rvault
```

Lock the vault:

```bash
rvault lock
```

## Backup and Restore

Backups are full encrypted binary recovery bundles. A backup is for the owner of the vault, not for sharing selected entries.

Create a backup:

```bash
rvault backup create --out rvault.rvault-backup
```

Restore a backup:

```bash
rvault backup restore rvault.rvault-backup
```

Skip the interactive restore confirmation:

```bash
rvault backup restore rvault.rvault-backup --yes
```

Restore replaces local RVault data after confirmation. Keep backup files somewhere you control.

## Encrypted Export and Import

Exports are encrypted binary `.rvault-export` files for selected-entry sharing with another RVault user.

The recipient gets their public RVault identity code:

```bash
rvault unlock
rvault identity
```

The sender exports one entry for that recipient:

```bash
rvault export --to rvault1-recipient-code --entry github alice --out github.rvault-export
```

The sender can export multiple selected entries:

```bash
rvault export --to rvault1-recipient-code \
  --selected github:alice \
  --selected email:alice@example.com \
  --out shared.rvault-export
```

The recipient imports the file:

```bash
rvault unlock
rvault import shared.rvault-export
```

Conflict shortcuts:

```bash
rvault import shared.rvault-export --overwrite-all
rvault import shared.rvault-export --skip-all
```

Only the recipient identity can decrypt the export.

## TUI Keybindings

### Main Table

| Key | Action |
| --- | --- |
| `Up` / `Down` | Move through entries |
| `Enter` | Copy the selected password to the clipboard |
| `a` | Add a new entry |
| `e` | Edit the selected entry |
| `d` | Delete the selected entry |
| `p` | Pin or unpin the selected entry |
| `i` | Copy this device's public identity code |
| `b` | Create a backup |
| `r` | Restore a backup |
| `x` | Export the selected entry |
| `m` | Import an encrypted export file |
| `S` | Open sort selection |
| `t` | Open theme selection |
| `Tab` | Switch to the password generator |
| `q` / `Esc` | Quit |
| `Shift+Q` | Lock and quit |

### Generator View

| Key | Action |
| --- | --- |
| `Left` / `Right` | Decrease or increase password length |
| `s` | Toggle special characters |
| `Enter` | Generate a password and copy it |
| `Tab` | Return to the main table |
| `q` / `Esc` | Quit |

### Selection Dialogs

| Key | Action |
| --- | --- |
| `Up` / `Down` | Move through options |
| `j` / `k` | Move through options in sort and theme selection |
| `Enter` | Confirm |
| `q` / `Esc` | Close the dialog |

## How RVault Works

RVault is split into three Rust crates and one browser extension:

- `rvault-core` handles config, keystore management, encryption, sessions, binary envelopes, backup, identity, export/import, storage, and clipboard integration.
- `rvault-cli` builds the `rvault` binary, CLI commands, and native messaging host.
- `rvault-tui` provides the terminal UI.
- `extension` contains the Plasmo MV3 extension for Helium.

At setup time, RVault stores a master-password hash in the config directory and creates a local keystore file encrypted with a key derived from the master password.

When the vault is unlocked, protected operations use the active session key instead of asking for the master password for every command.

Browser integration uses Chrome-style native messaging. Helium launches `rvault` directly when the extension sends a native message. `rvault browser enable` only registers the native messaging manifest; it does not start a background daemon.

## Build and Test From Source

Build Rust crates:

```bash
cargo build
```

Run Rust tests:

```bash
cargo test -p rvault-core
cargo test -p rvault-cli
cargo test -p rvault-tui
cargo check
```

Build and test the extension:

```bash
cd extension
bun install
bun test
bun run build
```

Create a local extension ZIP:

```bash
cd extension/build/chrome-mv3-prod
zip -r ../../../rvault-extension-1.2.0.zip .
```

The ZIP must contain `manifest.json` at the ZIP root.

## Release Checklist

For `1.2.0`:

1. Confirm versions are `1.2.0` in `Cargo.toml`, `Cargo.lock`, and `extension/package.json`.
2. Run Rust checks:

```bash
cargo test -p rvault-core
cargo test -p rvault-cli
cargo test -p rvault-tui
cargo check
```

3. Run extension checks:

```bash
cd extension
bun test
bun run build
```

4. Tag and push the release:

```bash
./release-rvault 1.2.0
```

The cargo-dist release workflow builds the CLI installers. A separate extension release workflow builds and uploads `rvault-extension-1.2.0.zip` for local Helium installation after the GitHub Release exists.

## Current Boundaries

- The browser extension is distributed locally for `1.2.0`; it is not published in the Chrome Web Store.
- Browser native host registration targets Helium on macOS.
- Firefox support is not included in this release.
- RVault does not provide hosted sync.
- Export/import is encrypted recipient sharing, not plaintext export.
- Backups are full recovery files and replace local RVault data on restore.

## License

Dual-licensed under `MIT` or `Apache-2.0`.
