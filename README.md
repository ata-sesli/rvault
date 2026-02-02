# 🔐 RVault - The Ultimate Secure Password Manager

**Stop compromising your digital security with weak, reused passwords.** RVault is a blazing-fast, military-grade password manager built in Rust that keeps your credentials encrypted locally while delivering lightning-fast access. No cloud dependencies, no subscription fees, no data breaches - just pure, unadulterated security that you control.

## ✨ Features

- 🖥️ **Modern TUI** - A beautiful, responsive Terminal User Interface for easy management
- 🔒 **Military-Grade Encryption** - ChaCha20-Poly1305 encryption with Argon2 key derivation
- 🎨 **Theme System** - Choose from 8+ premium themes (Gruvbox, Catppuccin, Nord, etc.)
- 📌 **Pinned Entries** - Keep your most important credentials at the top for instant access
- 🔃 **Advanced Sorting** - Sort by time, platform, or user ID (ascending/descending)
- 🎲 **Secure Generator** - Integrated password generator with visual progress bar
- 📋 **Instant Clipboard** - Copy credentials with a single keypress or command
- ⚡ **Rust-Powered** - Blazing-fast performance with zero-cost abstractions
- 🏠 **Fully Local** - Your data never leaves your machine - no cloud, no leaks
- 📦 **Multi-Vault** - Organize your life into different encrypted containers

## 🚀 Installation

```bash
git clone https://github.com/ata-sesli/rvault.git
cd rvault
# Option A: Install the binary to your Cargo bin (recommended)
cargo install --path .

# Option B: Build locally and run from target/
cargo build --release
```

### 🖥️ Launching the TUI

Simply run `rvault` without any arguments to enter the interactive mode:

```bash
rvault
```

### ⌨️ TUI Keyboard Shortcuts

| Key         | Action                             |
| ----------- | ---------------------------------- |
| `↑/↓`       | Navigate through list              |
| `Enter`     | Copy password to clipboard         |
| `a`         | Add new entry                      |
| `e`         | Edit password                      |
| `d`         | Delete entry                       |
| `p`         | Pin/Unpin entry                    |
| `S`         | Open Sort Selection                |
| `t`         | Change Theme                       |
| `Tab`       | Switch between Vault and Generator |
| `q` / `Esc` | Quit Application                   |

### 🛠️ CLI Setup

```bash
# Create your master password and keystore (run once)
rvault setup
```

### Unlock / Lock the Vault

```bash
# Start a session (required for protected commands like create/add/get/remove)
rvault unlock

# When you’re done
rvault lock
```

### Create a New Vault

```bash
# Create a vault for work credentials
rvault create work_vault

# Create personal vault (optional name - defaults to 'main')
rvault create
```

### Add Passwords

```bash
# Add to default vault
rvault add instagram johndoe:super_secret_password

# Add to specific vault
rvault add --vault work github jane.doe:my_github_token
```

### Instant Search & Retrieval

```bash
# Get password via CLI (copies to clipboard)
rvault get instagram johndoe
```

### Advanced Password Generation

```bash
# Generate 24-character password with special characters
rvault generate --length 24 --special-characters
```

## 🎨 Themes

RVault comes with beautifully crafted themes to match your terminal setup:

- **Gruvbox** (Default)
- **Catppuccin**
- **Dracula**
- **Tokyo Night**
- **Nord**
- **One Dark**
- **Solarized**
- **Monokai**

## 🏗️ Project Structure

The project is structured as a Rust workspace for maximum modularity:

- `crates/rvault-core` - The engine: Cryptography, Database, and internal logic
- `crates/rvault-tui` - The experience: Ratatui-based Terminal UI
- `crates/rvault-cli` - The interface: Clap-based Command Line Interface

## 🔧 Core Components

### **CLI Interface** (`cli.rs`)

- Built with `clap` for robust argument parsing
- Supports subcommands for all vault operations
- Comprehensive help system with usage examples

### **Cryptography** (`crypto.rs`)

- Secure password generation with customizable constraints
- ChaCha20-Poly1305 encryption for vault contents
- Argon2 key derivation for master password hashing

### **Storage Engine** (`storage.rs`)

- SQLite-based local storage for maximum reliability
- Each vault is a separate table with encrypted entries
- Automatic database creation and management

### **Security Features**

- All passwords encrypted at rest
- Memory-safe Rust implementation
- No network dependencies
- Local-only operation

## 🛠️ Dependencies

- **Ratatui** - Modern TUI library for rich terminal interfaces
- **Crossterm** - Terminal manipulation and event handling
- **Clap** - Command-line argument parsing
- **Rusqlite** - SQLite database interface
- **Argon2** - Password hashing (Key Derivation)
- **ChaCha20-Poly1305** - State-of-the-art encryption
- **Chrono** - Time and date management
- **Arboard** - Cross-platform clipboard access
- **Directories** - OS-appropriate data locations

## 🔒 Security Model

RVault follows a **zero-trust, local-first** security model:

1. **Master Password Protection** - All vaults are protected by a master password
2. **Local Encryption** - Data is encrypted before being written to disk
3. **Memory Safety** - Rust prevents buffer overflows and memory corruption
4. **No Cloud Dependencies** - Your data never leaves your machine
5. **Audit Trail** - All operations are logged for security monitoring

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines and:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License and Apache 2.0 License - see the [MIT-LICENSE](LICENSE-MIT.MD) or [APACHE-LICENSE](LICENSE-APACHE.MD) file for details.

**Built with ⚡️ in Rust by [Ata Sesli](https://github.com/ata-sesli)**

\*RVault: Because your passwords deserve better
