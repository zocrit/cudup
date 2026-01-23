# cudup

**The CUDA Version Manager**

Easily install and switch between CUDA versions on Linux.

## What is cudup?

`cudup` is a command-line tool that simplifies managing multiple CUDA toolkit and cuDNN installations on your system. It allows ML engineers, researchers, and GPU developers to:

- Install multiple CUDA versions side-by-side
- Switch between versions instantly
- Manage per-project CUDA requirements
- Verify installation health with built-in diagnostics

No more manual downloads, path conflicts, or environment variable headaches.

## Quick Example

```bash
# One-time setup (configures shell integration)
cudup manage setup

# Install CUDA 12.4.1 with compatible cuDNN
cudup install 12.4.1

# Switch to CUDA 12.4.1
cudup use 12.4.1

# Verify everything works
cudup check

# List installed versions
cudup list
```

## Why cudup?

Different ML frameworks require specific CUDA versions:

- PyTorch 2.0 needs CUDA 11.8
- TensorFlow 2.13 needs CUDA 11.8
- Older projects might need CUDA 10.2

Managing these manually is painful. `cudup` makes it trivial.

## System Requirements

- **OS:** Linux (Ubuntu 20.04+, Debian 11+)
- **Architecture:** x86_64, ARM64 (SBSA)
- **Disk Space:** ~10GB per CUDA version

## Status

!!! warning "Early Development"
    cudup is currently in early development and not yet ready for production use.
