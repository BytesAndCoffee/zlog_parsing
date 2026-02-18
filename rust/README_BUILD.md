# Cross-Compilation and Deployment Guide

## Building for x86_64 Linux from Apple Silicon macOS

### Prerequisites

1. Install the Linux target:
```bash
rustup target add x86_64-unknown-linux-gnu
```

2. Install the cross-compilation toolchain:
```bash
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
```

### Building

Build for x86_64 Linux:
```bash
cargo build --target x86_64-unknown-linux-gnu --release
```

The binary will be located at:
```
target/x86_64-unknown-linux-gnu/release/irc_log_parser
```

### Alternative: Using Cross

For easier cross-compilation, you can use the `cross` tool:

```bash
cargo install cross
cross build --target x86_64-unknown-linux-gnu --release
```

This uses Docker containers and doesn't require manual toolchain installation.

## Deployment to plex (Tailscale Host)

### Deploy and Run

Copy the binary to the plex host via Tailscale:
```bash
scp target/x86_64-unknown-linux-gnu/release/irc_log_parser plex:~/irc_log_parser/
```

Copy the .env configuration:
```bash
scp .env plex:~/irc_log_parser/
```

SSH into plex and run:
```bash
ssh plex
cd ~/irc_log_parser
./irc_log_parser
```

### Systemd Service (Optional)

To run as a service on plex, create `/etc/systemd/system/irc-log-parser.service`:

```ini
[Unit]
Description=IRC Log Parser
After=network.target mysql.service

[Service]
Type=simple
User=your_user
WorkingDirectory=/home/your_user/irc_log_parser
ExecStart=/home/your_user/irc_log_parser/irc_log_parser
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable irc-log-parser
sudo systemctl start irc-log-parser
sudo systemctl status irc-log-parser
```
