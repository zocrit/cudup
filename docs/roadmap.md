# Roadmap

Track development progress as features are implemented.

## MVP (v0.1) - Core Functionality

- [x] Install CUDA + cuDNN together (`cudup install`)
- [x] List available/installed versions (`cudup list`)
- [x] Switch between versions (`cudup use`)
- [x] Shell integration (`cudup manage setup`) - bash, zsh, fish
- [x] Health diagnostics (`cudup check`)
- [x] Basic error handling and user-friendly messages
- [x] Linux support (Ubuntu/Debian)

## v0.5 - Enhanced Experience

- [ ] Per-project `.cuda-version` files (`cudup local`)
- [x] Automatic cuDNN version matching
- [x] Uninstall versions (`cudup uninstall`)
- [ ] Clean up old versions (`cudup clean`)
- [x] Enhanced diagnostics (runtime checks, GPU detection)
- [x] Progress bars and colored output
- [x] Show cudup version (`cudup --version`)
- [x] Checksum verification (SHA256 integrity checks)
- [ ] Resumable downloads for large installers
- [ ] Pre-flight compatibility checking (GPU driver, compute capability)

## v1.0 - Production Ready

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
- [ ] Multi-distro support (RHEL, CentOS, Fedora)
- [ ] Performance optimizations (parallel downloads)
- [ ] Comprehensive documentation
- [ ] Automated testing suite

## Future Considerations

- [ ] WSL2 support and documentation
- [ ] Native Windows support (if demand exists)
- [ ] Docker integration
- [ ] Plugin system for extensions
