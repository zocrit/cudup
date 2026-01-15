# cudup uninstall

Remove an installed CUDA version.

## Usage

```bash
cudup uninstall [OPTIONS] <VERSION>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `VERSION` | CUDA version to uninstall (e.g., `12.4.1`) |

## Options

| Option | Description |
|--------|-------------|
| `-f, --force` | Skip confirmation prompt |

## Examples

```bash
# Uninstall with confirmation
cudup uninstall 12.4.1

# Uninstall without confirmation (for scripts)
cudup uninstall -f 12.4.1
```

## Behavior

- Shows the size of the installation before removal
- Asks for confirmation (unless `--force` is used)
- Warns if uninstalling the currently active version

!!! warning "Active Version"
    If you uninstall the currently active version (the one `CUDA_HOME` points to), your shell environment will have invalid paths. Run `cudup use <other-version>` to switch to a different version.

## See Also

- [`cudup install`](install.md) - Install versions
- [`cudup list`](list.md) - See installed versions
