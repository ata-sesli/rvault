# 🔐 RVault - The Ultimate Secure Password Manager

**Stop compromising your digital security with weak, reused passwords.** RVault is a blazing-fast, military-grade password manager built in Rust that keeps your credentials encrypted locally while delivering lightning-fast access. No cloud dependencies, no subscription fees, no data breaches - just pure, unadulterated security that you control.

## ✨ Features

- 🔒 **Military-Grade Encryption** - ChaCha20-Poly1305 encryption with Argon2 key derivation
- ⚡ **Lightning Fast** - Built in Rust for maximum performance and memory safety
- 🏠 **Fully Local** - Your data never leaves your machine
- 📋 **Instant Clipboard** - Passwords copied to clipboard with a single command
- 🎲 **Secure Password Generation** - Cryptographically secure random passwords
- 🗂️ **Multiple Vaults** - Organize credentials by context (work, personal, etc.)
- 🔍 **Quick Retrieval** - Find and copy passwords in milliseconds
- 📦 **Vault Export** - Backup and migrate your encrypted vaults

## 🚀 Installation

```bash
git clone https://github.com/ata-sesli/rvault.git
cd rvault
# Option A: Install the binary to your Cargo bin (recommended)
cargo install --path .

# Option B: Build locally and run from target/
cargo build --release
```

## 📖 Usage Examples

### First-Time Setup
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

### Retrieve Passwords
```bash
# Get password (automatically copied to clipboard)
rvault get instagram johndoe

# Get from specific vault
rvault get --vault work github jane.doe
```

### Generate Secure Passwords
```bash
# Generate 12-character password
rvault generate

# Generate 20-character password with special characters
rvault generate --length 20 --special-characters
```

### Manage Credentials
```bash
# Remove an entry
rvault remove instagram johndoe
rvault remove --vault work github jane.doe
```

### Vault Export (Coming Soon)
```bash
# Export vault for backup (planned)
# rvault export work_vault ./backup/
```

### Clipboard Monitoring (Coming Soon)
```bash
# Watch clipboard and auto-save to vault
rvault watch

# Stop watching
rvault unwatch
```

## 🏗️ Project Structure

```
src/
├── main.rs          # Application entry point and command routing
├── cli.rs           # Command-line interface definitions using clap
├── crypto.rs        # Encryption, hashing, and password generation
├── storage.rs       # SQLite database operations and vault management
├── clipboard.rs     # Clipboard integration for password copying
├── account.rs       # Account data structures and traits
├── error.rs         # Custom error types and handling
├── vault.rs         # Vault management operations (planned)
└── watcher.rs       # Clipboard monitoring functionality (planned)
```

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

- **clap** - Command-line argument parsing
- **rusqlite** - SQLite database interface
- **chacha20poly1305** - Authenticated encryption
- **argon2** - Password hashing
- **arboard** - Cross-platform clipboard access
- **directories** - OS-appropriate data directories
- **rand** - Cryptographically secure random generation

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

*RVault: Because your passwords deserve better than