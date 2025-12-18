# cudup manage

Manage cudup installation and shell integration.

## Subcommands

| Command | Description |
|---------|-------------|
| `setup` | Configure shell integration |
| `remove` | Remove shell integration |
| `self-update` | Update cudup to the latest version |

---

## cudup manage setup

Configure shell integration for cudup.

1. Detects your shell (bash, zsh, or fish)
2. Creates `~/.cudup/env` (or `env.fish` for fish) with the shell wrapper function
3. Adds a source line to your shell config (`.bashrc`, `.zshrc`, or `config.fish`)

## cudup manage remove

1. Deletes `~/.cudup/env` (or `env.fish`)
2. Removes the cudup source line from your shell config

## cudup manage self-update (not implemented yet)

Update cudup to the latest version.

- Download and install the latest cudup binary from GitHub releases
- Binary integrity checks
- Check for updates without installing (`--check` flag)
