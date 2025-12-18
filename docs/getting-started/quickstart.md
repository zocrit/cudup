# Quick Start

This guide will get you up and running with cudup in a few minutes.

## 1. Setup Shell Integration

```bash
cudup manage setup
```

Restart your terminal or source your shell config:

```bash
source ~/.bashrc  # or ~/.zshrc or ~/.config/fish/config.fish
```

## 2. List Available Versions

```bash
cudup list
```

This shows all CUDA versions available for installation.

## 3. Install a CUDA Version

```bash
cudup install 12.4.1
```

This downloads and installs CUDA 12.4.1 along with a compatible cuDNN version.


## 4. Activate the Version

```bash
cudup use 12.4.1
```

This sets `CUDA_HOME`, `PATH`, and `LD_LIBRARY_PATH` for your current shell.

## 5. Verify

```bash
cudup check
```

You should see all checks passing:

```
cudup check

[✓] cudup directory: /home/you/.cudup
[✓] shell integration: env file exists
[✓] installed versions: 1 (12.4.1)
[✓] active version: 12.4.1
[✓] nvcc: 12.4
[✓] nvidia driver: v550.54
[✓] gpu: NVIDIA GeForce RTX 4090

All checks passed!
```
