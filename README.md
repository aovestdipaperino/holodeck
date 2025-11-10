# Holodeck

<p align="center">
  <img src="logo.png" alt="Holo-Deck Logo" width="200"/>
</p>

<p align="center">
  <strong>A simple HTTP file server with built-in reverse SSH tunneling</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/holo-deck"><img src="https://img.shields.io/crates/v/holo-deck.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/holo-deck"><img src="https://docs.rs/holo-deck/badge.svg" alt="Documentation"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License: Apache 2.0"></a>
</p>

---

## Features

- 🚀 **Fast HTTP file server** built with Hyper and Tokio
- 📤 **File uploads** via POST requests
- 📥 **File downloads** via GET requests
- 🌐 **Reverse SSH tunneling** for external access
- 🔒 **Security** - Path traversal protection
- 🔑 **Smart SSH key management** - Automatic key detection with priority
- 🎨 **Clean output** - Beautiful tunnel URL display

## Quick Start

### Installation

```bash
cargo install holo-deck
```

### Local Server Only

```bash
# Start the server locally
holo-deck

# Or with cargo
cargo run
```

The server will automatically bind to a random available port (displayed on startup)

### With External Tunnel

```bash
# Start with automatic external tunnel via localhost.run
SSH_SERVER=ssh.localhost.run holo-deck
```

The tunnel URL will be automatically displayed:

```
╔════════════════════════════════════════════════════════════════╗
║                    TUNNEL ACTIVE                               ║
╠════════════════════════════════════════════════════════════════╣
║  External URL: https://abc123.lhr.life                         ║
╚════════════════════════════════════════════════════════════════╝
```

## Usage Examples

### List Files

```bash
# Local (use the port displayed on startup, e.g., 59830)
curl http://localhost:59830/

# External (via tunnel)
curl https://abc123.lhr.life/
```

### Upload a File

```bash
# Local (use your server's port)
curl -X POST --data-binary @myfile.txt http://localhost:59830/myfile.txt

# External
curl -X POST --data-binary @myfile.txt https://abc123.lhr.life/myfile.txt
```

### Download a File

```bash
# Local (use your server's port)
curl http://localhost:59830/myfile.txt

# External
curl https://abc123.lhr.life/myfile.txt
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SSH_SERVER` | SSH server address (e.g., ssh.localhost.run) | None (local only) |
| `SSH_USER` | SSH username | `localhost` |
| `SSH_PORT` | SSH server port | `22` |
| `SSH_KEY_PATH` | Path to SSH private key | None (required for key auth) |
| `SSH_PASSWORD` | SSH password | None (alternative to key auth) |
| `REMOTE_PORT` | Remote port to listen on | `80` |
| `RUST_LOG` | Enable debug logging | None |

### Custom Configuration

```bash
# Use a specific SSH key
SSH_SERVER=ssh.localhost.run SSH_KEY_PATH=~/.ssh/id_ed25519 holo-deck

# Use password authentication instead of key
SSH_SERVER=ssh.localhost.run SSH_PASSWORD=mypassword holo-deck

# Custom remote port
SSH_SERVER=ssh.localhost.run SSH_KEY_PATH=~/.ssh/id_ed25519 REMOTE_PORT=8080 holo-deck

# Enable debug logging
RUST_LOG=debug SSH_SERVER=ssh.localhost.run SSH_KEY_PATH=~/.ssh/id_ed25519 holo-deck
```

## Architecture

Holo-Deck is built with:

- **[Hyper](https://hyper.rs/)** - Fast HTTP implementation
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[russh](https://github.com/Eugeny/russh)** - Pure Rust SSH implementation
- **Rust 2024 edition** - Latest language features

### How It Works

1. **HTTP Server**: Binds to a random available port on localhost, handles GET/POST requests
2. **Reverse SSH Tunnel**: Connects to SSH server (e.g., localhost.run)
3. **Bidirectional Proxy**: Routes external traffic through SSH to local server
4. **File Storage**: Files stored in current working directory

## Security Considerations

- ✅ Path traversal protection (blocks `..` and `/` in filenames)
- ✅ Local-only HTTP server (binds to 127.0.0.1)
- ✅ SSH key authentication for tunneling
- ⚠️ No authentication on file access - suitable for temporary sharing
- ⚠️ Tunnel URLs are public - anyone with the URL can access files

**Recommendation**: Use Holo-Deck for temporary file sharing in trusted environments. For production use, add authentication and HTTPS.

## Development

### Building from Source

```bash
git clone https://github.com/enzolombardi/holo-deck.git
cd holo-deck
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
holo-deck/
├── src/
│   └── main.rs           # HTTP server and CLI
├── Cargo.toml
├── README.md
└── logo.png
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

You may choose either license for your use.

## Acknowledgments

- Built with ❤️ using Rust
- Inspired by Python's `http.server` and ngrok
- SSH tunneling powered by [russh](https://github.com/Eugeny/russh)
- Tunnel hosting via [localhost.run](https://localhost.run)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

---

**Made with Rust 🦀**
