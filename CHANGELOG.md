# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure
- Basic CLI framework with clap
- Project documentation (README, CONTRIBUTING, LICENSE)

### Changed
- Nothing yet

### Deprecated
- Nothing yet

### Removed
- Nothing yet

### Fixed
- Nothing yet

### Security
- Nothing yet

---

## How to Use This Changelog

### For Maintainers

When making changes, add them under the `[Unreleased]` section in the appropriate category:

- **Added** - New features
- **Changed** - Changes in existing functionality
- **Deprecated** - Soon-to-be removed features
- **Removed** - Removed features
- **Fixed** - Bug fixes
- **Security** - Security vulnerability fixes

When releasing a new version:

1. Change `[Unreleased]` to `[X.Y.Z] - YYYY-MM-DD`
2. Add a new `[Unreleased]` section at the top
3. Update the comparison links at the bottom
4. Commit with message: `chore: release vX.Y.Z`

### Version Format

- **X.0.0** (Major) - Breaking changes, incompatible API changes
- **0.X.0** (Minor) - New features, backward-compatible
- **0.0.X** (Patch) - Bug fixes, backward-compatible

### Example Entry Format
```markdown
## [0.1.0] - 2024-01-15

### Added
- `cudup install` command to download and install CUDA versions (#5)
- `cudup list` command to show installed versions (#8)
- Basic version parsing and validation (#3)

### Fixed
- Crash when `.cudup` directory doesn't exist (#12)
- Incorrect path resolution on some Linux distros (#15)
```

---

<!-- 
Template for new releases - copy this when releasing:

## [X.Y.Z] - YYYY-MM-DD

### Added
- 

### Changed
- 

### Deprecated
- 

### Removed
- 

### Fixed
- 

### Security
- 

-->

<!-- Comparison Links (update these when releasing) -->
[Unreleased]: https://github.com/ZoCrit/cudup/compare/v0.1.0...HEAD
<!-- [0.1.0]: https://github.com/ZoCrit/cudup/releases/tag/v0.1.0 -->