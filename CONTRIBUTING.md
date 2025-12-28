# Contributing to cudup

Thank you for your interest in contributing to cudup! We welcome contributions from everyone.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Development Workflow](#development-workflow)
- [Code Guidelines](#code-guidelines)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Issue Guidelines](#issue-guidelines)
- [Community](#community)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow:

- **Be respectful and inclusive** - We welcome contributors of all backgrounds and experience levels
- **Be constructive** - Provide helpful feedback and be open to receiving it
- **Be collaborative** - Work together to improve the project
- **Be patient** - Remember that everyone is volunteering their time

Unacceptable behavior includes harassment, discrimination, or any form of abuse. Violations may result in being banned from the project.

## Getting Started

Before you begin:
- Read the [README.md](README.md) to understand what cudup does
- Check the [roadmap](README.md#roadmap) to see what's planned
- Look at [open issues](https://github.com/yourusername/cudup/issues) to find something to work on
- Join [discussions](https://github.com/yourusername/cudup/discussions) to ask questions


## Development Setup

### Prerequisites

- **Rust:** Install via [rustup](https://rustup.rs/)
```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- **Git:** For version control
- **Linux:** Development primarily targets Linux (Ubuntu/Debian recommended)
- **NVIDIA GPU:** For testing actual CUDA functionality

### Clone and Build
```bash
# Clone the repository
git clone https://github.com/yourusername/cudup.git
cd cudup

# Build the project
cargo build

# Run tests
cargo test

# Run cudup locally
cargo run -- --help
```

### Recommended Tools
```bash
# Install development tools
cargo install cargo-watch    # Auto-rebuild on file changes
cargo install cargo-nextest  # Faster test runner
cargo install cargo-edit     # Manage dependencies easily

# Run with auto-reload
cargo watch -x check -x test -x run
```

## How to Contribute

### Types of Contributions

We welcome many types of contributions:

- **Bug reports** - Found something broken? Let us know!
- **Feature requests** - Have an idea? Suggest it!
- **Documentation** - Improve README, add examples, write guides
- **Tests** - Add test coverage, improve existing tests
- **Code** - Implement features, fix bugs, refactor
- **UX improvements** - Better error messages, clearer output
- **Platform support** - Help with different Linux distros

### Before Starting Work

**For significant changes:**
1. Open an issue first to discuss your approach
2. Wait for maintainer feedback before investing significant time
3. Comment on the issue to claim it and avoid duplicate work

**For small changes:**
- Typo fixes, documentation improvements, minor bug fixes can be submitted directly as PRs

## Development Workflow

### 1. Fork and Clone
```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/cudup.git
cd cudup
git remote add upstream https://github.com/yourusername/cudup.git
```

### 2. Create a Branch
```bash
# Create a descriptive branch name
git checkout -b fix-issue-123
# or
git checkout -b add-feature-xyz
```

Branch naming conventions:
- `fix-*` for bug fixes
- `feat-*` for new features
- `docs-*` for documentation
- `refactor-*` for code refactoring
- `test-*` for adding tests

### 3. Make Changes

Write your code, following the [code guidelines](#code-guidelines) below.

### 4. Test Your Changes
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Build in release mode
cargo build --release
```

### 5. Commit Your Changes

Write clear, descriptive commit messages:
```bash
git add .
git commit -m "Fix issue with version parsing in CUDA 12.x

- Updated regex pattern to handle new version format
- Added test cases for CUDA 12.0, 12.1, 12.2
- Fixes #123"
```

**Commit message guidelines:**
- Use present tense ("Add feature" not "Added feature")
- First line: brief summary (50 chars or less)
- Blank line, then detailed explanation if needed
- Reference issues/PRs with `#123`
- Use `Fixes #123` to auto-close issues

### 6. Push and Create Pull Request
```bash
git push origin fix-issue-123
```

Then create a pull request on GitHub.

## Code Guidelines

### Rust Style

Follow standard Rust conventions:
```bash
# Format code (required before submitting)
cargo fmt

# Run linter (must pass)
cargo clippy -- -D warnings
```

### Code Quality

- **Use descriptive variable names** - Except for common idioms (`i`, `e`, `acc`)
- **Add comments for complex logic** - Explain "why", not "what"

### Documentation

Add doc comments for public APIs:
```rust
/// Downloads and installs a specific CUDA version.
///
/// # Arguments
///
/// * `version` - The CUDA version to install (e.g., "11.8")
///
/// # Returns
///
/// Returns `Ok(PathBuf)` with the installation path on success,
/// or an `Error` if download or extraction fails.
///
/// # Examples
///
/// ```
/// let path = install_cuda("11.8").await?;
/// println!("Installed to: {:?}", path);
/// ```
pub async fn install_cuda(version: &str) -> Result<PathBuf, Error> {
    // implementation
}
```

### Project Structure

Organize code logically:
```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library root
├── commands/         # Command implementations
│   ├── mod.rs
│   ├── install.rs
│   ├── list.rs
│   └── doctor.rs
├── cuda/             # CUDA-specific logic
│   ├── mod.rs
│   ├── version.rs
│   └── download.rs
├── env/              # Environment management
│   └── mod.rs
└── config/           # Configuration
    └── mod.rs
```

## Testing

### Test Types

We use two main types of tests in cudup:

#### Unit Tests
Place unit tests in the same file as the code being tested:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_version_parsing() {
        let version = parse_version("11.8");
        assert_eq!(version, Some(Version { major: 11, minor: 8 }));
    }

    #[test]
    fn test_invalid_version_returns_none() {
        assert_eq!(parse_version("invalid"), None);
    }
}
```

#### Integration Tests
Integration tests go in the `tests/` directory and test the CLI as a whole:
```rust
// tests/cli_tests.rs
use std::process::Command;

#[test]
fn test_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run cudup --help");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cudup"));
}
```

### Running Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test cuda::version

# Run integration tests only
cargo test --test '*'

# Run a specific test by name
cargo test test_valid_version_parsing
```

### Writing Tests

**Best Practices:**

1. **Test one thing per test** - Each test should verify one behavior
   ```rust
   // Good
   #[test]
   fn test_parses_valid_version() { /* ... */ }
   
   // Less good - tests multiple things
   #[test]
   fn test_version_parsing() { /* ... */ }
   ```

2. **Use descriptive names** - Start with `test_`, then describe the scenario
   ```rust
   #[test]
   fn test_install_with_invalid_version_returns_error() { /* ... */ }
   ```

3. **Use fixtures for test data** - Place sample data in `tests/fixtures/`
   ```
   tests/fixtures/
   ├── sample_config.toml
   ├── cuda_versions.json
   └── README.md
   ```

4. **Test both success and failure paths** - Don't just test happy paths
   ```rust
   #[test]
   fn test_install_success() { /* ... */ }
   
   #[test]
   fn test_install_with_missing_file_fails() { /* ... */ }
   ```

5. **Keep tests isolated** - Use temporary files/directories
   ```rust
   use tempfile::TempDir;
   
   #[test]
   fn test_creates_config_file() {
       let temp_dir = TempDir::new().unwrap();
       // Test code using temp_dir
       // Automatically cleaned up when dropped
   }
   ```

### Test Coverage

Aim for:
- **Unit tests** for all public functions
- **Error cases** - Test failure paths, not just happy paths
- **Edge cases** - Empty inputs, boundary conditions, invalid data
- **Integration tests** for CLI commands and end-to-end workflows

### Continuous Integration

All tests run automatically on:
- **Push** to any branch
- **Pull requests** before merge

Tests must pass before a PR can be merged. The CI workflow checks:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Unit and integration tests (`cargo test`)
- Release build (`cargo build --release`)

## Submitting Changes

### Pull Request Process

1. **Update documentation** if you changed behavior or added features
2. **Add tests** for new functionality
3. **Update CHANGELOG.md** with your changes (under "Unreleased")
4. **Ensure all tests pass** locally before pushing
5. **Create the pull request** with a clear description

### Pull Request Template

When creating a PR, include:
```markdown
## Description
Brief description of what this PR does.

## Related Issues
Fixes #123
Related to #456

## Changes Made
- Added feature X
- Fixed bug Y
- Updated documentation

## Testing
- [ ] Added unit tests
- [ ] Added integration tests
- [ ] Manually tested on Ubuntu 22.04
- [ ] All tests pass locally

## Checklist
- [ ] Code follows project style guidelines
- [ ] `cargo fmt` has been run
- [ ] `cargo clippy` passes with no warnings
- [ ] Documentation has been updated
- [ ] CHANGELOG.md has been updated
```

### Review Process

After submitting:
1. Maintainers will review your PR (usually within a few days)
2. Address any feedback or requested changes
3. Once approved, a maintainer will merge your PR

**Be patient and responsive:**
- Respond to review comments promptly
- Don't take criticism personally - it's about the code, not you
- Ask questions if feedback is unclear

## Issue Guidelines

### Reporting Bugs

Include:
- **Description** of the bug
- **Steps to reproduce** - Be specific!
- **Expected behavior** vs actual behavior
- **Environment details:**
  - OS and version (e.g., Ubuntu 22.04)
  - cudup version (`cudup --version`)
  - CUDA versions involved
  - Rust version (`rustc --version`)
- **Error messages** or logs (use code blocks)
- **Screenshots** if relevant

**Example:**
```markdown
## Bug: cudup doctor fails with permission error

**Steps to reproduce:**
1. Install cudup
2. Run `cudup doctor`
3. See error

**Expected:** Should run diagnostics and show status
**Actual:** Crashes with "Permission denied"

**Environment:**
- Ubuntu 22.04
- cudup v0.1.0
- No CUDA currently installed

**Error output:**
```
Error: Permission denied (os error 13)
```
```

### Requesting Features

Include:
- **Clear use case** - Why is this needed?
- **Proposed solution** - How should it work?
- **Alternatives considered** - Other approaches?
- **Additional context** - Examples, mockups, etc.

## Community

### Communication Channels

- **GitHub Issues** - Bug reports, feature requests
- **GitHub Discussions** - Questions, ideas, general chat
- **Pull Requests** - Code review and collaboration

### Getting Help

- Check existing issues and discussions first
- Ask questions in GitHub Discussions
- Be specific about your problem
- Include relevant details (OS, versions, error messages)

### Recognition

Contributors will be:
- Listed in the project's contributors page
- Mentioned in release notes for significant contributions
- Credited in the README if they make major contributions

## Questions?

If you have questions about contributing, feel free to:
- Open a discussion on GitHub
- Comment on relevant issues
- Reach out to maintainers

**Thank you for contributing to cudup!**