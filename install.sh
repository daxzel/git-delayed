#!/bin/bash
set -e

echo "Installing git-delayed..."

# Build the release binary
cargo build --release

# Install binary to /usr/local/bin
INSTALL_DIR="/usr/local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating $INSTALL_DIR..."
    sudo mkdir -p "$INSTALL_DIR"
fi

echo "Installing binary to $INSTALL_DIR..."
sudo cp target/release/git-delayed "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/git-delayed"

echo "✓ Binary installed to $INSTALL_DIR/git-delayed"

# Detect OS and set up daemon autostart
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - use launchd
    echo "Setting up macOS launchd service..."
    
    PLIST_DIR="$HOME/Library/LaunchAgents"
    PLIST_FILE="$PLIST_DIR/com.git-delayed.daemon.plist"
    
    mkdir -p "$PLIST_DIR"
    
    cat > "$PLIST_FILE" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.git-delayed.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/git-delayed</string>
        <string>daemon</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$HOME/Library/Application Support/git-delayed/launchd.out</string>
    <key>StandardErrorPath</key>
    <string>$HOME/Library/Application Support/git-delayed/launchd.err</string>
</dict>
</plist>
EOF
    
    # Load the service
    launchctl unload "$PLIST_FILE" 2>/dev/null || true
    launchctl load "$PLIST_FILE"
    
    echo "✓ Daemon configured to start automatically via launchd"
    echo "  Service file: $PLIST_FILE"
    
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux - use systemd
    echo "Setting up Linux systemd service..."
    
    SYSTEMD_DIR="$HOME/.config/systemd/user"
    SERVICE_FILE="$SYSTEMD_DIR/git-delayed.service"
    
    mkdir -p "$SYSTEMD_DIR"
    
    cat > "$SERVICE_FILE" << EOF
[Unit]
Description=Git Delayed Daemon
After=network.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/git-delayed daemon start
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
EOF
    
    # Reload systemd and enable service
    systemctl --user daemon-reload
    systemctl --user enable git-delayed.service
    systemctl --user start git-delayed.service
    
    echo "✓ Daemon configured to start automatically via systemd"
    echo "  Service file: $SERVICE_FILE"
    echo "  Check status: systemctl --user status git-delayed"
else
    echo "⚠ Automatic daemon startup not configured for this OS"
    echo "  You can manually start the daemon with: git-delayed daemon start"
fi

echo ""
echo "✓ Installation complete!"
echo ""
echo "Quick start:"
echo "  git-delayed schedule '+10 hours' commit -m 'your message'"
echo "  git-delayed schedule '+10 hours' push"
echo "  git-delayed list"
echo "  git-delayed daemon status"
echo ""
echo "The daemon is running in the background and will start automatically on boot."
