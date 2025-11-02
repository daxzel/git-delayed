# git-delayed

Schedule git commits and pushes for later. Useful when you want to work now but push later.

## Why?

Sometimes you want to commit your work but not push it immediately. Maybe you're working offline and want to push when you get internet. Or you want to batch your pushes. Whatever the reason, this tool lets you schedule when your commits and pushes happen.

## Install

```bash
./install.sh
```

That's it. The daemon starts automatically and will run on boot.

## Usage

```bash
# schedule a commit for 10 hours from now
git delayed schedule "+10 hours" commit -m "feat: add thing"

# schedule a push for Monday morning
git delayed schedule "Monday" push

# see what's scheduled
git delayed list

# check the logs
git delayed logs
```

## Time formats

- `+10 hours`, `+2 days`, `+30 minutes` - relative time
- `Monday`, `Tuesday`, etc - next occurrence at 9am
- `2025-12-25 09:00` - exact time

## How it works

Operations get stored locally. A daemon checks every minute if anything needs to run. If something fails (network issues, auth problems, whatever), it retries every 10 minutes until it works.

Storage is at:
- macOS: `~/Library/Application Support/git-delayed/`
- Linux: `~/.config/git-delayed/`

## Daemon

The daemon runs automatically. You can check on it:

```bash
git delayed daemon status
git delayed daemon stop
git delayed daemon start
```

## Troubleshooting

**Not in a git repo?**
Make sure you're in a git directory when you schedule stuff.

**Daemon not running?**
```bash
git delayed daemon status
```

If it's stuck, delete the PID file:
```bash
# macOS
rm ~/Library/Application\ Support/git-delayed/daemon.pid

# Linux
rm ~/.config/git-delayed/daemon.pid
```

**Check logs:**
```bash
# macOS
tail -f ~/Library/Application\ Support/git-delayed/daemon.err

# Linux
tail -f ~/.config/git-delayed/daemon.err
```
