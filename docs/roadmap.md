# Roadmap

Track development progress as features are implemented.

## MVP (v0.1)

- [x] Install CUDA + cuDNN together (`cudup install`)
- [x] List available/installed versions (`cudup list`)
- [x] Switch between versions (`cudup use`)
- [x] Shell integration (`cudup manage setup`) (bash, zsh, fish)
- [x] Health diagnostics (`cudup check`)
- [x] Basic error handling and user-friendly messages
- [x] Linux support (Ubuntu/Debian)

## v0.5
- [x] Per-project `.cuda-version` files (`cudup local`)
- [ ] Show current version (`cudup current`)
- [x] Automatic cuDNN version matching
- [x] Uninstall versions (`cudup uninstall`)
- [ ] Clean up old versions (`cudup clean`)
- [x] Diagnostics (runtime checks, GPU detection)
- [x] Show cudup version (`cudup --version`)
- [x] Checksum verification (SHA256 integrity checks)
- [ ] Resumable downloads for large installers
- [ ] Pre-flight compatibility checking (GPU driver, compute capability)

## v1.0

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
- [ ] Multi-architecture support (x86_64, ARM64/SBSA)
- [ ] Multi-distro support (RHEL, CentOS, Fedora)
- [ ] Performance optimizations (parallel downloads)
- [ ] Comprehensive documentation
- [ ] Automated testing suite

## Future Considerations

- [ ] WSL2 support
- [ ] Native Windows support (if demand exists?)
- [ ] Docker integration
- [ ] Plugin system for extensions
