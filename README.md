# cudup

**The CUDA Version Manager**

Easily install and switch between CUDA versions on Linux.

## Status

**Early Development** - This is not ready to be used safely yet.

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

# Install CUDA 11.8 with compatible cuDNN
cudup install 11.8

# Switch to CUDA 11.8
cudup use 11.8

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

## Installation

> **Note:** cudup is currently in early development. Installation instructions will be available soon.
```bash
# Coming soon
```

## Roadmap

Track development progress as features are implemented:

### MVP (v0.1) - Core Functionality
- [x] Install CUDA + cuDNN together (`cudup install`)
- [x] List available/installed versions (`cudup list`)
- [x] Switch between versions (`cudup use`)
- [x] Shell integration (`cudup manage setup`)
- [x] Health diagnostics (`cudup check`)
- [x] Basic error handling and user-friendly messages
- [x] Linux support (Ubuntu/Debian)

### v0.5 - Enhanced Experience
- [ ] Per-project `.cuda-version` files (`cudup local`)
- [ ] Parallel downloads (async/multithread hybrid for download/install steps?)
- [ ] Show current version (`cudup current`)
- [x] Automatic cuDNN version matching
- [x] Uninstall versions (`cudup uninstall`)
- [ ] Clean up old versions (`cudup clean`)
- [x] Enhanced diagnostics (runtime checks, GPU detection)
- [x] Progress bars and colored output
- [x] Show cudup version (`cudup --version`)
- [x] Checksum verification (SHA256 integrity checks)
- [ ] Resumable downloads for large installers
- [ ] Pre-flight compatibility checking (GPU driver, compute capability)

### v1.0 - Production Ready
- [ ] NCCL support (`--with-nccl`)
- [ ] TensorRT support (`--with-tensorrt`)
- [ ] Framework compatibility checking (`cudup check pytorch/tensorflow`)
- [ ] Configuration file support (`~/.cudup/config.toml`)
- [ ] Self-update mechanism (`cudup manage self-update`)
- [ ] Remote version manifest (fetch latest available versions)
- [ ] Proxy configuration for corporate environments
- [ ] PATH rollback support (`cudup manage remove --rollback`)
- [ ] Advanced installation options (`--minimal`, `--from-cache`)
- [ ] Global vs local version modes (`cudup global`)
- [ ] Import existing installations (`cudup import`)
- [ ] Export/import environments
- [x] Multi-architecture support (x86_64, ARM64/SBSA)
- [ ] Multi-distro support (RHEL, CentOS, Fedora)
- [ ] Performance optimizations (parallel downloads)
- [ ] Comprehensive documentation
- [ ] Automated testing suite

### Future Considerations
- [ ] WSL2 support and documentation
- [ ] Native Windows support (if demand exists)
- [ ] Docker integration
- [ ] Plugin system for extensions

## System Requirements

- **OS:** Linux (Ubuntu 20.04+, Debian 11+)
- **Architecture:** x86_64, ARM64 (SBSA)
- **Disk Space:** ~10GB per CUDA version

## Documentation

Full documentation available at **[zocrit.github.io/cudup](https://zocrit.github.io/cudup/)**

- [Installation Guide](https://zocrit.github.io/cudup/getting-started/installation/)
- [Quick Start](https://zocrit.github.io/cudup/getting-started/quickstart/)
- [Command Reference](https://zocrit.github.io/cudup/commands/)

## Contributing

Contributions are welcome! This project is in early development, and we'd love your help.

Please check the [roadmap](#roadmap) above for planned features, and feel free to:
- Report bugs via [GitHub Issues](https://github.com/ZoCrit/cudup/issues)
- Suggest features via [GitHub Discussions](https://github.com/ZoCrit/cudup/discussions)
- Submit pull requests

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Note:** This project is not affiliated with NVIDIA. CUDA and cuDNN are trademarks of NVIDIA Corporation.