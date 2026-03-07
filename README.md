# git-delayed

Schedule git commits, pushes, and merges for later. Useful when you want to work now but push later.

## Why?

Sometimes you want to commit your work but not push it immediately. Maybe you're working offline and want to push when you get internet. Or you want to batch your pushes. Or you want to schedule a merge for a specific date. Whatever the reason, this tool lets you schedule when your git operations happen.

## Install

```bash
./install.sh
```

That's it. The daemon starts automatically and will run on boot.

## Uninstall

```bash
./uninstall.sh
```

This removes the daemon, service, and binary. Your scheduled operations data is preserved but the script shows you where it is if you want to remove it manually.

## Usage

```bash
# schedule a commit for 10 hours from now
git delayed schedule "+10 hours" commit -m "feat: add thing"

# schedule a push for Monday morning
git delayed schedule "Monday" push

# schedule a squash merge from feature into main on a specific date
git delayed schedule "2025-12-25 09:00" merge-commit --from feature --into main

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

Operations get stored locally with the current branch (for pushes). A daemon checks every minute and processes operations one at a time, in order.

For pushes:
1. Stashes any uncommitted changes
2. Switches to the target branch
3. Pushes
4. Switches back to original branch
5. Unstashes changes

For merge-commits:
1. Stashes any uncommitted changes
2. Switches to the target branch
3. Runs `git merge --squash` from the source branch
4. Commits with the source branch's last commit message
5. Switches back to original branch
6. Unstashes changes

If something fails, it retries every 10 minutes. If there's nothing to push, it's marked as skipped.

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
