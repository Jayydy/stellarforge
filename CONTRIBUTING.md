# Contributing to StellarForge

Thank you for your interest in contributing to StellarForge! This document provides guidelines for contributing to our collection of Soroban smart contracts.

## 🚀 Quick Start

1. **Fork the repository** on GitHub
2. **Clone your fork**: `git clone https://github.com/YOUR-USERNAME/stellarforge.git`
3. **Create a feature branch**: `git checkout -b feature/your-feature-name`
4. **Make your changes**
5. **Run tests**: `make test` or `cargo test --workspace`
6. **Commit your changes**: `git commit -m "Description of changes"`
7. **Push to your fork**: `git push origin feature/your-feature-name`
8. **Submit a pull request** from your fork to the main repository

## 🌿 Branch Naming Convention

Use descriptive branch names that follow this pattern:

- `feat/description` - New features (e.g., `feat/add-pause-functionality`)
- `fix/description` - Bug fixes (e.g., `fix/vesting-calculation-bug`)
- `docs/description` - Documentation updates (e.g., `docs/update-readme`)
- `refactor/description` - Code refactoring (e.g., `refactor/simplify-error-handling`)
- `test/description` - Test additions or updates (e.g., `test/add-multisig-tests`)
- `chore/description` - Maintenance tasks (e.g., `chore/update-dependencies`)

## 📦 Shared Error Crate (forge-errors)

When adding new common error variants to `forge-errors`:

1. **Consider if the error is truly common** across multiple contracts
2. **Add descriptive documentation** to the variant in `crates/forge-errors/src/lib.rs`
3. **Update error codes** to avoid conflicts with existing variants
4. **Test the change** across all affected contracts

### Adding New Common Errors

If you identify an error pattern that appears in 3+ contracts, consider adding it to `CommonError`:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CommonError {
    // Existing variants...
    
    /// New error description
    NewError = NEXT_AVAILABLE_CODE,
}
```

**Process:**
1. Add the variant to `CommonError` with next available error code
2. Update any contracts that should use this new shared variant
3. Add tests to verify the error behavior
4. Update documentation

## 🏗 Development Setup

### Prerequisites

- **Rust**: 2021 edition with `wasm32v1-none` target
- **Stellar CLI**: v25.2.0 or higher
- **Make**: (optional) for convenience commands

### Installation

```bash
# Install Rust
rustup target add wasm32v1-none

# Install Stellar CLI
cargo install --locked stellar-cli

# Verify installation
stellar --version
```

## 🧪 Testing

### Running Tests

```bash
# Test all contracts
make test

# Test specific contract
cargo test -p forge-governor
cargo test -p forge-multisig
cargo test -p forge-oracle
cargo test -p forge-stream
cargo test -p forge-vesting
cargo test -p forge-vesting-factory
```

### Test Coverage

We aim for high test coverage. When adding new features:
- Write unit tests for new functionality
- Test error paths exhaustively
- Include integration tests for contract interactions
- Verify all error variants are tested

## 📝 Code Style

Follow these conventions:

### Rust Style

- Use `rustfmt` for formatting: `make fmt`
- Use `clippy` for linting: `make lint`
- Follow Rust idioms and Soroban best practices
- Use `#![no_std]` for all contracts
- Prefer `require_auth()` over manual auth checks where possible

### Contract Patterns

- **Error Handling**: Use the shared `CommonError` variants when applicable
- **Storage**: Use appropriate storage types (instance vs persistent)
- **Events**: Emit events for all state changes
- **TTL Management**: Extend storage TTLs appropriately
- **Security**: Follow established security patterns from existing contracts

### Documentation

- Document all public functions with examples
- Include error conditions in docstrings
- Update README.md for new features
- Keep CHANGELOG.md updated

## 🐛 Bug Reports

When reporting bugs:

1. **Use the issue template** provided in GitHub Issues
2. **Include reproduction steps** with minimal example
3. **Specify contract name** and affected functions
4. **Include environment details** (OS, Rust version, Stellar CLI version)
5. **Add logs** and error messages when applicable

## 💡 Feature Requests

We welcome feature requests! Please:

1. **Check existing issues** for similar requests
2. **Describe the use case** clearly
3. **Consider impact** on existing contracts and integrators
4. **Propose implementation approach** if you have ideas

## 📄 Pull Request Process

### Before Submitting

- [ ] Tests pass: `make test`
- [ ] Code formatted: `make fmt` or `cargo fmt --all`
- [ ] Linting clean: `make lint` or `cargo clippy --workspace -- -D warnings`
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)
- [ ] Branch follows naming convention (feat/, fix/, docs/, etc.)

### PR Guidelines

- **Small, focused PRs** are preferred over large, multi-purpose changes
- **One feature per PR** when possible
- **Include tests** for new functionality
- **Update documentation** as needed
- **Link to related issues** using "Closes #123" or "Fixes #456" in the PR description
- **Provide context** in the PR description explaining what changed and why

### Review Process

Maintainers will review for:
- ✅ Code quality and style
- ✅ Test coverage
- ✅ Security considerations
- ✅ Documentation completeness
- ✅ Breaking changes (if any)

## 🔒 Security

Security is our top priority. If you discover a security vulnerability:

1. **Do NOT open a public issue**
2. **Email us privately**: security@stellarforge.org
3. **Include details**: Impact, reproduction steps, affected versions
4. **Allow time for response**: We'll acknowledge within 48 hours

## 📧 Development Tools

### Make Commands

```makefile
build:
	cargo build --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace -- -D warnings

check:
	cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace

clean:
	cargo clean --workspace
```

### Workspace Structure

```
stellarforge/
├── crates/
│   └── forge-errors/          # Shared error library
├── contracts/
│   ├── forge-governor/       # Governance contract
│   ├── forge-multisig/        # Multisig treasury
│   ├── forge-oracle/          # Price feed contract
│   ├── forge-stream/           # Token streaming
│   ├── forge-vesting/          # Token vesting
│   └── forge-vesting-factory/ # Multi-beneficiary vesting
├── benches/                   # Performance benchmarks
└── scripts/                   # Utility scripts
```

## 🤝 Community

- **GitHub Discussions**: Use for questions, ideas, and general discussion
- **Issues**: Bug reports and feature requests
- **Discord**: [Join our community](https://discord.gg/stellarforge) for real-time chat

## 📜 License

By contributing, you agree that your contributions will be licensed under the same [MIT License](LICENSE) as the project.

---

Thank you for contributing to StellarForge! 🚀
