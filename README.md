# Terminal to PowerShell Bridge

Secure local IPC for sharing data between processes. Works on Windows (named pipes) and macOS/Linux (Unix domain sockets).

## Quick Start

### Build

```bash
cargo build --release
```

### Run

**Terminal 1 - Start the server:**
```bash
# macOS/Linux
./target/release/server

# Windows
.\target\release\server.exe
```

**Terminal 2 - Use the client:**
```bash
# macOS/Linux
./target/release/client ping
./target/release/client send "Hello!"

# Windows
.\target\release\client.exe ping
.\target\release\client.exe send "Hello!"
```

## Client Commands

| Command | Description |
|---------|-------------|
| `client ping` | Test server connection |
| `client send <text>` | Send text data |
| `client get <VAR>` | Get environment variable |
| `client getall` | Get all environment variables |
| `client set <VAR> <VALUE>` | Set environment variable |

## How It Works

- **Windows**: Uses named pipes (`\\.\pipe\terminal_to_ps`)
- **macOS/Linux**: Uses Unix domain sockets (`~/.terminal_to_ps.sock`)
- **Encryption**: ChaCha20-Poly1305 AEAD
- **Key file**: `~/.terminal_to_ps_key` (auto-generated on first run)

## Project Structure

```
src/
├── main.rs        # Server
├── client.rs      # Client
├── crypto.rs      # ChaCha20-Poly1305 encryption
├── protocol.rs    # JSON message types
└── pipe_server.rs # Platform-specific IPC
```

## License

MIT
