# cudup install

Install a specific CUDA version.

## Usage

```bash
cudup install <VERSION>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `VERSION` | CUDA version to install (e.g., `12.4`, `12.4.1`) |

## Examples

```bash
# Install CUDA 12.4
cudup install 12.4

# Install a specific patch version
cudup install 12.4.1
```

## What Gets Installed

- CUDA Toolkit components
- Compatible cuDNN version (automatically selected)

## Installation Location

All versions are installed to:

```
~/.cudup/versions/<version>/
```

## See Also

- [`cudup uninstall`](uninstall.md) - Remove installed versions
- [`cudup list`](list.md) - See available versions
