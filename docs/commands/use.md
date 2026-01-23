# cudup use

Activate a specific CUDA version in your current shell.

## Usage

```bash
cudup use <VERSION>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `VERSION` | CUDA version to activate (format: `X.Y.Z`, e.g., `12.4.1`) |

## Examples

```bash
# Activate CUDA 12.4.1
cudup use 12.4.1

# Activate a different version
cudup use 11.8.0
```

## What It Does

Sets the following environment variables in your current shell:

- `CUDA_HOME` - Points to the CUDA installation
- `PATH` - Adds CUDA binaries
- `LD_LIBRARY_PATH` - Adds CUDA libraries

## Prerequisites

!!! important "Shell Setup Required"
    The `use` command requires shell integration. Run `cudup manage setup` first.

Without shell integration, `cudup use` will print export commands but won't modify your environment.

## See Also

- [`cudup manage setup`](manage.md) - Configure shell integration
- [`cudup check`](check.md) - Verify active version
