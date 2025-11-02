# Install

## Quick way

```bash
./install.sh
```

This builds the binary, installs it to `/usr/local/bin`, and sets up the daemon to run automatically.

## Homebrew (not yet)

```bash
brew install git-delayed
brew services start git-delayed
```

## Check it worked

```bash
git-delayed daemon status
```

Should say it's running.

## Uninstall

**macOS:**
```bash
launchctl unload ~/Library/LaunchAgents/com.git-delayed.daemon.plist
rm ~/Library/LaunchAgents/com.git-delayed.daemon.plist
sudo rm /usr/local/bin/git-delayed
```

**Linux:**
```bash
systemctl --user stop git-delayed.service
systemctl --user disable git-delayed.service
rm ~/.config/systemd/user/git-delayed.service
systemctl --user daemon-reload
sudo rm /usr/local/bin/git-delayed
```

**Remove data:**
```bash
# macOS
rm -rf ~/Library/Application\ Support/git-delayed

# Linux
rm -rf ~/.config/git-delayed
```

## Daemon logs

**macOS:**
```bash
cat ~/Library/Application\ Support/git-delayed/launchd.err
```

**Linux:**
```bash
journalctl --user -u git-delayed.service
```

## Restart daemon

**macOS:**
```bash
launchctl unload ~/Library/LaunchAgents/com.git-delayed.daemon.plist
launchctl load ~/Library/LaunchAgents/com.git-delayed.daemon.plist
```

**Linux:**
```bash
systemctl --user restart git-delayed.service
```
