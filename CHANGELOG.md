# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-10

### Added
- Initial release of Holo-Deck
- HTTP file server with GET and POST support
- File upload via curl POST requests
- File download via curl GET requests
- File listing at root path
- Random free port selection (avoids port conflicts)
- Current working directory as shared folder
- Reverse SSH tunneling integration
- Support for localhost.run and similar SSH tunnel services
- SSH key authentication via SSH_KEY_PATH environment variable
- SSH password authentication via SSH_PASSWORD environment variable
- Smart tunnel URL extraction and display
- Environment variable configuration
- Path traversal attack protection
- Bidirectional proxy for SSH tunnel connections
- Clean, professional console output
- Rust 2024 edition support

### Security
- Path traversal protection (blocks `..` and `/` in filenames)
- Local-only HTTP server binding (127.0.0.1)
- SSH key authentication for tunnel connections

[0.1.0]: https://github.com/enzolombardi/holo-deck/releases/tag/v0.1.0
