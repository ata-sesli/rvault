# RVault

RVault is a local-first password manager written in Rust, with a terminal UI, a CLI, and a browser extension that talks to the local `rvault` binary through native messaging.

Current version: `1.4.2`.

RVault keeps storage local. Passwords are encrypted before they are written to SQLite, browser integration goes through a local native host, and the extension does not store plaintext credentials.

## Contents

- [Install RVault](#install-rvault)
- [Install the Browser Extension](#install-the-browser-extension)
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

Install the CLI from the `v1.4.2` release assets:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ata-sesli/rvault/releases/download/v1.4.2/rvault-cli-installer.sh | sh
```

On Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ata-sesli/rvault/releases/download/v1.4.2/rvault-cli-installer.ps1 | iex"
```

Confirm the install:

```bash
rvault --version
```

Expected version:

```text
rvault-cli 1.4.2
```

### From Source

Requirements:

- Rust toolchain with Cargo
- Bun, only needed for the browser extension
- Helium, Chrome, Chromium, or Firefox, only needed to run browser integration

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

## Install the Browser Extension

RVault supports Helium, Google Chrome, Chromium, and Firefox through local native messaging. The browser starts the installed `rvault` binary when the extension sends a request; RVault does not run a server or background daemon.

### Helium, Chrome, and Chromium

1. Install the `rvault` CLI.
2. Download `rvault-extension-<version>.zip` from the matching GitHub release.
3. Extract it somewhere that will not move.
4. Open `chrome://extensions` in the browser.
5. Enable **Developer mode**, choose **Load unpacked**, and select the extracted directory.
6. Register the native messaging host for that browser:

```bash
rvault browser enable --browser helium
rvault browser enable --browser chrome
rvault browser enable --browser chromium
```

The original command remains compatible and defaults to Helium:

```bash
rvault browser enable
```

To remove a registration, use the matching browser value:

```bash
rvault browser disable --browser chrome
```

Helium registration is available on macOS. Chrome and Chromium registration is available on macOS, Linux, and Windows.

### Firefox

Releases produced after this browser-support change include a Mozilla-signed `rvault-extension-firefox-<version>.xpi`. Open the XPI in Firefox and approve the installation, then register the native host:

```bash
rvault browser enable --browser firefox
```

Firefox registration is available on macOS, Linux, and Windows. The extension uses the fixed add-on ID `rvault@ata-sesli.github.io`, which must match RVault's native host manifest.

For local development, build the Firefox target and load its manifest temporarily from `about:debugging`:

```bash
cd extension
bun install
bun run build:firefox
```

Select `extension/build/firefox-mv3-prod/manifest.json` when Firefox asks for a temporary add-on file.

### Build the Chromium Extension Locally

```bash
cd extension
bun install
bun run build
```

Load `extension/build/chrome-mv3-prod` through the browser's extension page. Do not load the top-level `extension/` directory.

### Troubleshooting Browser Integration

If the extension says the native host is unavailable, disable and re-enable the same browser registration, then reload the extension:

```bash
rvault browser disable --browser firefox
rvault browser enable --browser firefox
```

If the `rvault` binary moves after an update or reinstall, run the enable command again. The pinned Chromium extension ID is `gnfmkmiklgghclejbbdmjgcldajahfhh`.

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

### Rust API guidance

Library users should prefer the typed `rvault-core` APIs: `SecretKey`, `Ciphertext`,
`encrypt`/`decrypt`, `SessionKey::load`, and `EntryRepository`. The retained `Table` clipboard
helpers and raw string crypto helpers are deprecated for the remainder of the 1.x line; existing
callers can migrate without changing the SQLite or session formats. See the
[RVault Core 2.0 migration guide](docs/migration/rvault-core-2.0.md) for replacements and rollout
order.

RVault is split into three Rust crates and one browser extension:

- `rvault-core` handles config, keystore management, encryption, sessions, binary envelopes, backup, identity, export/import, storage, and clipboard integration.
- `rvault-cli` builds the `rvault` binary, CLI commands, and native messaging host.
- `rvault-tui` provides the terminal UI.
- `extension` contains the Plasmo MV3 extension for Chromium-family browsers and Firefox.

At setup time, RVault stores a master-password hash in the config directory and creates a local keystore file encrypted with a key derived from the master password.

When the vault is unlocked, protected operations use the active session key instead of asking for the master password for every command.

Browser integration uses native messaging. Chromium-family browsers pass their extension origin to `rvault`; Firefox passes the fixed RVault add-on ID. `rvault browser enable` writes the browser-specific manifest or registry entry and does not start a background daemon.

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
bun run build:firefox
```

Create a local extension ZIP:

```bash
cd extension/build/chrome-mv3-prod
zip -r ../../../rvault-extension-1.4.2.zip .
```

The ZIP must contain `manifest.json` at the ZIP root.

## Release Checklist

For a release version `<version>`:

1. Confirm versions match in `Cargo.toml`, `Cargo.lock`, and `extension/package.json`.
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
bun run build:firefox
```

4. Configure the `AMO_JWT_ISSUER` and `AMO_JWT_SECRET` repository secrets used to request an unlisted Mozilla signature for the Firefox XPI.

5. Tag and push the release:

```bash
./release-rvault <version>
```

The cargo-dist release workflow builds the CLI installers. The extension release workflow uploads `rvault-extension-<version>.zip` for Chromium-family browsers and the Mozilla-signed `rvault-extension-firefox-<version>.xpi` after the GitHub Release exists.

## Current Boundaries

- The browser extension is distributed through GitHub Releases; it is not published in the Chrome Web Store or AMO.
- Helium native host registration is macOS-only.
- RVault does not provide hosted sync.
- Export/import is encrypted recipient sharing, not plaintext export.
- Backups are full recovery files and replace local RVault data on restore.

## License

Dual-licensed under `MIT` or `Apache-2.0`.
