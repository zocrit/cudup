# cudup manage

Manage cudup installation and shell integration.

## Usage

```bash
cudup manage <COMMAND>
```

## Subcommands

| Command | Description |
|---------|-------------|
| `setup` | Configure shell integration |
| `remove` | Remove shell integration |

---

## cudup manage setup

Configure shell integration for cudup.

### Usage

```bash
cudup manage setup
```

### What It Does

1. Creates `~/.cudup/env` with the shell wrapper function
2. Adds `source ~/.cudup/env` to your `.bashrc` or `.zshrc`

### Example

```
$ cudup manage setup
Detected shell: zsh

This will:
  - Create: /home/you/.cudup/env
  - Append to: /home/you/.zshrc

Proceed with setup? [y/N] y

Created /home/you/.cudup/env
Updated /home/you/.zshrc

Setup complete!

To start using cudup, either:
  - Restart your terminal, or
  - Run: source /home/you/.zshrc
```

---

## cudup manage remove

Remove shell integration.

### Usage

```bash
cudup manage remove
```

### What It Does

1. Deletes `~/.cudup/env`
2. Removes the cudup source line from your `.bashrc` or `.zshrc`

### Example

```
$ cudup manage remove
Detected shell: zsh

This will:
  - Delete: /home/you/.cudup/env
  - Remove cudup lines from: /home/you/.zshrc

Proceed with removal? [y/N] y

Deleted /home/you/.cudup/env
Updated /home/you/.zshrc

Removal complete!
```
