# Secure Messenger

Send encrypted messages between devices on your network.

## Setup

### 1. Build (on both devices)
```bash
cargo build --release
```

### 2. Start server (on one device)
```bash
./target/release/server
```
First run creates a key file and prints it. Copy this key to the other device.

### 3. Copy the key
Put the same key in `~/.terminal_to_ps_key` on both devices.

### 4. Send messages
```bash
# From the other device
./target/release/client <server-ip>:9876 send "Hello!"
```

## Commands

```bash
client <host:port> ping              # Test connection
client <host:port> send <message>    # Send message
```

## Example

**Mac (server):**
```bash
./target/release/server
# Shows: Listening on port 9876
```

**Windows (client):**
```powershell
.\target\release\client.exe 192.168.1.50:9876 send "Hello from Windows!"
```

The Mac will display: `>>> MESSAGE: Hello from Windows!`
