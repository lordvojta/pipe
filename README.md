# Terminal to PowerShell Bridge

A secure Rust-based tool for bidirectional communication between your terminal and Windows PowerShell, with built-in encryption for sensitive data like environment variables.

## Features

- **Secure Communication**: All data encrypted using ChaCha20-Poly1305 AEAD cipher
- **Bidirectional**: Request/response pattern between terminal and PowerShell
- **Named Pipes**: Fast local IPC using Windows named pipes
- **Environment Variables**: Easily share sensitive env vars between terminal and PowerShell
- **Extensible**: Simple protocol for sending arbitrary data

## Architecture

```
┌─────────────────┐                    ┌──────────────────┐
│  Rust Server    │◄──Named Pipe IPC──►│  PowerShell      │
│  (Terminal)     │                    │  Client          │
│                 │                    │                  │
│  - Handles env  │  Encrypted with    │  - Requests data │
│  - Encrypts     │  ChaCha20-Poly1305 │  - Decrypts      │
│  - Listens      │                    │  - Uses data     │
└─────────────────┘                    └──────────────────┘
```

## Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Windows**: This tool uses Windows named pipes
- **PowerShell**: Version 5.1 or later (supports .NET cryptography)

## Installation

1. Clone or download this repository
2. Build the Rust server:

```bash
cargo build --release
```

The compiled binary will be at `target/release/terminal-to-ps.exe`

## Usage

### Step 1: Start the Rust Server

Run the server in your terminal (PowerShell, Windows Terminal, or any terminal on Windows):

```bash
cargo run --release
```

Or run the compiled binary:

```bash
./target/release/terminal-to-ps.exe
```

On first run, it will generate an encryption key at `~/.terminal_to_ps_key`. You'll see:

```
Generated new encryption key: C:\Users\YourName\.terminal_to_ps_key
IMPORTANT: Copy this key file to your PowerShell client location!

=== Terminal to PowerShell Bridge ===
Pipe: \\.\pipe\terminal_to_ps
Ready to accept connections...
```

The server will keep running and listening for connections.

### Step 2: Use the PowerShell Client

In a separate PowerShell window, navigate to the project directory and run:

```powershell
# Test the connection
. .\client.ps1
Test-Connection

# Get a specific environment variable
$path = Get-EnvVar -Name "PATH"
Write-Host $path

# Get all environment variables
$allVars = Get-AllEnvVars
$allVars

# Set an environment variable (in both server and PS session)
Set-EnvVar -Name "MY_SECRET" -Value "sensitive-data"

# Send custom data
Send-Data -Key "api_token" -Data "your-secret-token"
```

Or run the example script:

```powershell
.\example.ps1
```

## Security Considerations

### Encryption

- Uses **ChaCha20-Poly1305** authenticated encryption
- 256-bit keys generated using OS-level cryptographically secure RNG
- Unique nonce for every encryption operation
- Authentication tags prevent tampering

### Key Management

- Encryption key stored at `~/.terminal_to_ps_key`
- **IMPORTANT**: Protect this file! Anyone with the key can decrypt communications
- The key is shared between the Rust server and PowerShell client
- Consider using Windows DPAPI or other key management solutions for production

### Attack Surface

- **Local only**: Named pipes are local to the machine
- **No network exposure**: Cannot be accessed remotely
- **Process isolation**: Each connection is handled separately

### Recommendations

1. **Restrict key file permissions**: Make the key file readable only by your user
2. **Rotate keys**: Generate new keys periodically
3. **Audit usage**: Monitor what data is being transferred
4. **Use for development**: This tool is designed for local development workflows

## Protocol

The protocol uses JSON messages encrypted with ChaCha20-Poly1305.

### Request Types

```json
// Get a specific environment variable
{
  "type": "get_env",
  "name": "PATH"
}

// Get all environment variables
{
  "type": "get_all_env"
}

// Set an environment variable
{
  "type": "set_env",
  "name": "MY_VAR",
  "value": "my_value"
}

// Send arbitrary data
{
  "type": "send_data",
  "key": "identifier",
  "data": "your data here"
}

// Ping (test connection)
{
  "type": "ping"
}
```

### Response Types

```json
// Success response
{
  "type": "success",
  "data": "optional data"
}

// Environment variables response
{
  "type": "env_vars",
  "vars": {
    "PATH": "...",
    "HOME": "..."
  }
}

// Error response
{
  "type": "error",
  "message": "error description"
}

// Pong response
{
  "type": "pong"
}
```

## PowerShell Client Functions

The `client.ps1` script exports these functions:

- `Test-Connection`: Test if the server is running
- `Get-EnvVar -Name "VAR_NAME"`: Get a specific environment variable
- `Get-AllEnvVars`: Get all environment variables as a hashtable
- `Set-EnvVar -Name "VAR" -Value "value"`: Set an environment variable
- `Send-Data -Key "key" -Data "data"`: Send arbitrary data

## Troubleshooting

### "Encryption key file not found"

Make sure the Rust server has been run at least once to generate the key file. The key is stored at `~/.terminal_to_ps_key` (e.g., `C:\Users\YourName\.terminal_to_ps_key`).

### "Failed to connect to pipe"

Ensure the Rust server is running. You should see "Ready to accept connections..." in the server output.

### "Decryption failed"

This usually means:
1. The key files don't match between server and client
2. The data was corrupted in transit
3. You're using incompatible encryption implementations

Make sure both the Rust server and PowerShell client are using the same key file.

### Named pipe errors on non-Windows systems

This tool currently only works on Windows due to the use of Windows named pipes. For cross-platform support, consider using TCP sockets or Unix domain sockets.

## Development

### Running Tests

```bash
cargo test
```

### Project Structure

```
terminal-to-ps/
├── src/
│   ├── main.rs          # Server entry point and request handling
│   ├── crypto.rs        # Encryption/decryption using ChaCha20-Poly1305
│   ├── pipe_server.rs   # Windows named pipe server implementation
│   └── protocol.rs      # Request/response message types
├── client.ps1           # PowerShell client library
├── example.ps1          # Example usage script
├── Cargo.toml          # Rust dependencies
└── README.md           # This file
```

## License

MIT

## Contributing

Contributions welcome! Please ensure:
- Code follows Rust best practices
- Tests pass (`cargo test`)
- Security considerations are addressed
- Documentation is updated

## Future Enhancements

- [ ] Cross-platform support (Unix domain sockets)
- [ ] Key rotation mechanism
- [ ] Multiple concurrent connections
- [ ] Async I/O with tokio
- [ ] Client authentication
- [ ] Rate limiting
- [ ] Logging and audit trail
- [ ] GUI for easier management
