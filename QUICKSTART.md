# Quick Start Guide

Get up and running in 3 simple steps!

## Initial Setup (One-time)

```bash
./install.sh
```

This will:
- Build the server
- Set up Python environment
- Install dependencies

## Running Manually

### Terminal 1 - Start Server
```bash
./run_server.sh
```

### Terminal 2 - Send Data
```bash
./run_client.sh ping                    # Test connection
./run_client.sh send "Hello World"      # Send text
```

## Auto-Start on macOS (Optional)

Want the server to start automatically when you log in?

```bash
./setup_autostart.sh
```

After running this, the server will:
- Start automatically on login
- Restart automatically if it crashes
- Run in the background

## Simple Commands

From the project directory:

```bash
# Start server manually
./run_server.sh

# Send data from client
./run_client.sh send "your message here"

# Test connection
./run_client.sh ping

# View auto-start logs (if using auto-start)
tail -f server.log
```

## Sharing the Key Between Devices

After first run, the server creates an encryption key at:
```
~/.terminal_to_ps_key
```

To use the client on another device:
1. Copy this key file to the other device at the same location
2. Run `./install.sh` on the other device
3. Use `./run_client.sh` to send data

## Troubleshooting

**Can't connect?**
- Make sure server is running: `./run_server.sh`
- Check if auto-start is running: `launchctl list | grep terminal-to-ps`

**Key file missing?**
- Run the server at least once to generate it: `./run_server.sh`

That's it! See README.md for detailed documentation.
