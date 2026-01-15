# Installation

## Prerequisites

- Linux (Ubuntu 20.04+, Debian 11+)
- x86_64 architecture
- ~10GB disk space per CUDA version

## Install cudup

!!! note "Coming Soon"
    Installation instructions will be available once cudup reaches a stable release.

### From Source

```bash
# Clone the repository
git clone https://github.com/ZoCrit/cudup.git
cd cudup

# Build with Cargo
cargo build --release

# Add to PATH
cp target/release/cudup ~/.local/bin/
```

## Shell Setup

After installing the binary, configure shell integration:

```bash
cudup manage setup
```

This will:

1. Create `~/.cudup/env` with the shell function
2. Add a source line to your `.bashrc` or `.zshrc`

Then restart your terminal or run:

```bash
source ~/.bashrc  # or ~/.zshrc
```

## Verify Installation

```bash
cudup check
```

This shows the status of your cudup configuration and CUDA installation.
