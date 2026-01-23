# cudup install

Install a specific CUDA version.

## Usage

```bash
cudup install <VERSION>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `VERSION` | CUDA version to install (format: `X.Y.Z`, e.g., `12.4.1`) |

## Examples

```bash
# Install CUDA 12.4.1
cudup install 12.4.1

# Install an older version
cudup install 11.8.0
```

## What Gets Installed

- CUDA Toolkit components
- Compatible cuDNN version (automatically selected)

All downloads are verified with SHA256 checksums before extraction.

## Installation Location

All versions are installed to:

```
~/.cudup/versions/<version>/
```

## See Also

- [`cudup uninstall`](uninstall.md) - Remove installed versions
- [`cudup list`](list.md) - See available versions
