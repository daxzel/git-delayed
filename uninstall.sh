#!/bin/bash
set -e

echo "Uninstalling git-delayed..."

# stop daemon if running
if command -v git-delayed &> /dev/null; then
    git-delayed daemon stop 2>/dev/null || true
fi

# remove service
if [[ "$OSTYPE" == "darwin"* ]]; then
    PLIST="$HOME/Library/LaunchAgents/com.git-delayed.daemon.plist"
    if [ -f "$PLIST" ]; then
        launchctl unload "$PLIST" 2>/dev/null || true
        rm "$PLIST"
        echo "✓ removed launchd service"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    systemctl --user stop git-delayed.service 2>/dev/null || true
    systemctl --user disable git-delayed.service 2>/dev/null || true
    
    SERVICE="$HOME/.config/systemd/user/git-delayed.service"
    if [ -f "$SERVICE" ]; then
        rm "$SERVICE"
        systemctl --user daemon-reload
        echo "✓ removed systemd service"
    fi
fi

# remove binary
if [ -f "/usr/local/bin/git-delayed" ]; then
    sudo rm /usr/local/bin/git-delayed
    echo "✓ removed binary"
fi

echo ""
echo "Uninstalled. Data is still at:"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  ~/Library/Application Support/git-delayed/"
else
    echo "  ~/.config/git-delayed/"
fi
echo ""
echo "To remove data: rm -rf <path above>"
