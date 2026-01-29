# Installation

## Prerequisites

- Linux (Ubuntu 20.04+, Debian 11+)
- x86_64 or ARM64 (SBSA) architecture
- ~10GB disk space per CUDA version

!!! note "ARM64 Support"
    cudup supports ARM64 servers using NVIDIA's SBSA (Server Base System Architecture) builds.
    This includes NVIDIA Grace Hopper, AWS Graviton with NVIDIA GPUs, and similar server platforms.
    Jetson devices are not supported - use NVIDIA JetPack SDK instead.

## Install cudup

### Using Cargo (Recommended)

```bash
cargo install cudup
```

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
2. Add a source line to your shell config (`.bashrc`, `.zshrc`, or `config.fish`)

Supported shells: **bash**, **zsh**, **fish**

Then restart your terminal or source your config:

=== "Bash"

    ```bash
    source ~/.bashrc
    ```

=== "Zsh"

    ```bash
    source ~/.zshrc
    ```

=== "Fish"

    ```bash
    source ~/.config/fish/config.fish
    ```

## Verify Installation

```bash
cudup check
```

This shows the status of your cudup configuration and CUDA installation.
