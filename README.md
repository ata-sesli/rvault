# RVault

RVault is a local-first password manager written in Rust.

It stores encrypted credentials in a local SQLite database and can be used either through a command-line interface or an interactive terminal UI built with ratatui.

## What it does

RVault stores credentials in a local SQLite database and encrypts each password before writing it to disk.

The project currently has two ways to use it:

- `rvault` launches a `ratatui`-based terminal UI for browsing, adding, editing, deleting, pinning, and copying entries.
- `rvault <subcommand>` exposes a few core operations from the shell, including setup, unlock, lock, create, add, get, remove, and password generation.

Passwords are copied to the system clipboard rather than printed to stdout.

## Why this project exists

RVault is built around a simple goal: keep password management local and scriptable without depending on a hosted service or browser extension.

The codebase is structured as a Rust workspace, so the storage and crypto logic live in a reusable core crate while the TUI and CLI sit on top of it.

## Demo

https://github.com/user-attachments/assets/8d165491-0f6d-4da9-9a37-0c68e1f3dea2

## Key features

- Local-only storage under the OS-specific application data directory
- Master-password setup with Argon2-based verification
- Per-entry encryption using ChaCha20-Poly1305
- Session-based unlock flow with automatic expiration
- Terminal UI for day-to-day use
- Built-in password generator
- Clipboard copy for generated or retrieved passwords
- Pinning and sorting in the TUI
- Multiple vault tables in the underlying database via the CLI
- Theme support in the TUI

## Installation

The easiest way to install RVault is from the GitHub Releases page:

- Open: <https://github.com/ata-sesli/rvault/releases>
- Download the archive or installer for your platform
- Add the installed `rvault` binary to your `PATH` if your platform-specific package does not do that automatically

If you prefer the release install scripts used by the project site, the current documented commands are:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ata-sesli/rvault/releases/download/v0.1.1/rvault-cli-installer.sh | sh
```

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ata-sesli/rvault/releases/download/v0.1.1/rvault-cli-installer.ps1 | iex"
```

To build from source instead:

```bash
git clone https://github.com/ata-sesli/rvault.git
cd rvault
cargo install --path crates/rvault-cli
```

For local development:

```bash
cargo build --release
cargo run -p rvault-cli
```

## Quick example

First-time setup:

```bash
rvault setup
```

Unlock the vault for protected commands:

```bash
rvault unlock
```

Add a credential:

```bash
rvault add github alice:correct-horse-battery-staple
```

Retrieve a credential and copy it to the clipboard:

```bash
rvault get github alice
```

Launch the terminal UI:

```bash
rvault
```

## TUI keybindings

These are the main bindings exposed by the current TUI implementation.

### Main table

| Key | Action |
| --- | --- |
| `Up` / `Down` | Move through entries |
| `Enter` | Copy the selected password to the clipboard |
| `a` | Add a new entry |
| `e` | Edit the selected entry |
| `d` | Delete the selected entry |
| `p` | Pin or unpin the selected entry |
| `S` | Open sort selection |
| `t` | Open theme selection |
| `Tab` | Switch to the password generator |
| `q` / `Esc` | Quit |
| `Shift+Q` | Lock and quit |

### Generator view

| Key | Action |
| --- | --- |
| `Left` / `Right` | Decrease or increase password length |
| `s` | Toggle special characters |
| `Enter` | Generate a password and copy it |
| `Tab` | Return to the main table |
| `q` / `Esc` | Quit |

### Selection dialogs

| Key | Action |
| --- | --- |
| `Up` / `Down` | Move through options |
| `j` / `k` | Move through options in sort and theme selection |
| `Enter` | Confirm |
| `q` / `Esc` | Close the dialog |

## How it works

RVault splits responsibilities across three crates:

- `rvault-core` handles config, keystore management, encryption, sessions, storage, and clipboard integration.
- `rvault-cli` is the main binary and command parser.
- `rvault-tui` provides the interactive terminal interface.

At setup time, RVault hashes the master password and stores the hash in the config directory. It also generates a master encryption key and writes it to a local keystore file encrypted with a key derived from the master password.

When you unlock the vault, RVault decrypts that master encryption key and stores it in a session file with a timeout. Protected operations read the encryption key from the active session instead of prompting every time.

Password entries are stored in SQLite. Each entry gets its own salt-derived key based on the current session key, and the encrypted password, nonce, and salt are stored with the row.

## Limitations and trade-offs

- The project is local-only. There is no sync, sharing, or remote backup layer.
- The CLI surface is larger than the currently implemented command set. Some declared commands such as clipboard watching and export are not wired up yet.
- The TUI works against the default vault table; multi-vault workflows are more visible in the CLI than in the UI.
- Retrieved passwords are copied to the clipboard. If you want a different retrieval model, that would need code changes.
- I could not fully verify a clean build in this environment because `cargo check` was blocked from downloading crates outside the sandbox.

## Project structure

- `Cargo.toml`: workspace definition
- `crates/rvault-core`: shared storage, crypto, config, keystore, session, and clipboard logic
- `crates/rvault-cli`: CLI binary named `rvault`
- `crates/rvault-tui`: terminal UI library and TUI-specific app state

## Roadmap

Possible near-term improvements based on the current code structure:

- Finish or remove partially declared CLI commands
- Improve multi-vault support in the TUI
- Add import/export with a documented format
- Add tests around storage migrations and session handling
- Tighten README and CLI help so supported workflows are easier to discover

## License

Dual-licensed under `MIT` or `Apache-2.0`.
