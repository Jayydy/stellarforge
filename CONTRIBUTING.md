# Contributing to StellarForge

Thank you for your interest in contributing to StellarForge! This document provides guidelines for contributing to our collection of Soroban smart contracts.

## 🚀 Quick Start

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/your-feature-name`
3. **Make your changes**
4. **Run tests**: `make test` or `cargo test --workspace`
5. **Create Pull Request for Code Review**

## 📝 Commit Message Standards

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification for clear and consistent commit history.

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type

Must be one of the following:

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that don't affect code meaning (whitespace, formatting)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Performance improvement
- **test**: Adding or updating tests
- **chore**: Changes to build process or auxiliary tools
- **revert**: Reverts a previous commit

### Scope

The scope should specify the contract or component affected:

- `forge-stream`
- `forge-vesting`
- `forge-governor`
- `forge-multisig`
- `forge-oracle`
- `forge-vesting-factory`
- `forge-errors`
- `docs`
- `ci`

### Subject

- Use imperative, present tense: "add" not "added" or "adds"
- Don't capitalize first letter
- No period (.) at the end
- Limit to 50 characters

### Body (Optional)

- Explain what and why, not how
- Wrap at 72 characters
- Separate from subject with blank line

### Footer (Optional)

- Reference issues: `Closes #123`, `Fixes #456`, `Resolves #789`
- Note breaking changes: `BREAKING CHANGE: description`

### Examples

**Simple feature:**
```
feat(forge-stream): add pause functionality

Allows stream sender to temporarily pause token accrual.
```

**Bug fix with issue reference:**
```
fix(forge-oracle): return error instead of None when uninitialized

Changes get_admin() and get_staleness_threshold() to return
Result types for consistent error handling.

Fixes #224
```

**Multiple changes:**
```
feat(forge-multisig): add event emission to all state changes

- Add proposal_created event
- Add proposal_approved event
- Add proposal_rejected event
- Add proposal_executed event

Closes #226
```

**Breaking change:**
```
refactor(forge-vesting)!: change claim() return type

BREAKING CHANGE: claim() now returns Result<i128, VestingError>
instead of i128. Callers must handle the Result type.

Closes #123
```

### Commit Message Rules

1. **Keep commits atomic** - One logical change per commit
2. **Write meaningful messages** - Future you will thank you
3. **Reference issues** - Use `Closes #123` or `Fixes #456` in footer
4. **Use present tense** - "add feature" not "added feature"
5. **Be concise but descriptive** - Balance brevity with clarity
6. **Separate subject from body** - Use blank line between them
7. **Wrap body at 72 characters** - For better readability in git log

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

### Before Creating Pull Request

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
