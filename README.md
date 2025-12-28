# cudup

**The CUDA Version Manager**

Easily install and switch between CUDA versions on Linux.

## Status

**Early Development** - Not yet ready for production use.

## What is cudup?

`cudup` is a command-line tool that simplifies managing multiple CUDA toolkit and cuDNN installations on your system. It allows ML engineers, researchers, and GPU developers to:

- Install multiple CUDA versions side-by-side
- Switch between versions instantly
- Manage per-project CUDA requirements
- Verify installation health with built-in diagnostics

No more manual downloads, path conflicts, or environment variable headaches.

## Quick Example
```bash
# Install CUDA 11.8 with compatible cuDNN
cudup install 11.8

# Switch to CUDA 11.8 in current shell
eval "$(cudup use 11.8)"

# Verify everything works
cudup doctor

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
- [ ] Install CUDA + cuDNN together (`cudup install`)
- [ ] List available versions (`cudup list-remote`)
- [ ] List installed versions (`cudup list`)
- [ ] Switch between versions (`cudup use`)
- [ ] Show current active version (`cudup current`)
- [ ] Health diagnostics (`cudup doctor`)
- [ ] Basic error handling and user-friendly messages
- [ ] Linux support (Ubuntu/Debian)

### v0.5 - Enhanced Experience
- [ ] Per-project `.cuda-version` files (`cudup local`)
- [ ] Shell integration (bash/zsh) (`cudup init`)
- [ ] Automatic cuDNN version matching
- [ ] Uninstall versions (`cudup uninstall`)
- [ ] Clean up old versions (`cudup clean`)
- [ ] Enhanced diagnostics (runtime checks, GPU detection)
- [ ] Disk space management and warnings
- [ ] Progress bars and colored output
- [ ] Show cudup version (`cudup --version`)

### v1.0 - Production Ready
- [ ] NCCL support (`--with-nccl`)
- [ ] TensorRT support (`--with-tensorrt`)
- [ ] Framework compatibility checking (`cudup check pytorch/tensorflow`)
- [ ] Configuration file support (`~/.cudup/config.toml`)
- [ ] Self-update mechanism (`cudup self-update`)
- [ ] Advanced installation options (`--minimal`, `--from-cache`)
- [ ] Global vs local version modes (`cudup global`)
- [ ] Import existing installations (`cudup import`)
- [ ] Export/import environments
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
- **Architecture:** x86_64
- **Disk Space:** ~10GB per CUDA version

## Documentation

> Coming soon

- [Installation Guide](docs/installation.md)
- [Quick Start](docs/quickstart.md)
- [Command Reference](docs/commands.md)
- [Troubleshooting](docs/troubleshooting.md)

## Contributing

Contributions are welcome! This project is in early development, and we'd love your help.

Please check the [roadmap](#roadmap) above for planned features, and feel free to:
- Report bugs via [GitHub Issues](https://github.com/yourusername/cudup/issues)
- Suggest features via [GitHub Discussions](https://github.com/yourusername/cudup/discussions)
- Submit pull requests

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Note:** This project is not affiliated with NVIDIA. CUDA and cuDNN are trademarks of NVIDIA Corporation.