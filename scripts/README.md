# Scripts

This directory contains helper scripts for contributors.

## pre-commit

A git pre-commit hook that automatically checks code formatting and linting before each commit.

### Installation

```bash
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Enabling Tests in the Hook

By default the hook only runs `cargo fmt` and `cargo clippy`. To also run the full test suite before each commit, set the `FORGE_PRECOMMIT_TESTS` environment variable to `1`:

```bash
# Run tests on this commit only
FORGE_PRECOMMIT_TESTS=1 git commit -m "your message"

# Enable tests permanently for your local repo
export FORGE_PRECOMMIT_TESTS=1
```

> **Tip:** If you're adding new tests, use `FORGE_PRECOMMIT_TESTS=1 git commit` to verify they pass before pushing.

See [CONTRIBUTING.md](../CONTRIBUTING.md#pre-commit-hook-optional-but-recommended) for more details.

## seed.sh

Deploys and initializes all five StellarForge contracts with realistic sample data on a local or testnet Stellar network. Safe to run multiple times — already-deployed contracts are skipped.

### Usage

```bash
# Requires a funded identity named "seed-admin":
stellar keys generate seed-admin --network standalone --fund

bash scripts/seed.sh [--network standalone|testnet]
```

Override the identity or network via environment variables:

```bash
IDENTITY=my-key NETWORK=testnet bash scripts/seed.sh
```

Contract IDs are persisted to `.seed-state` in the project root.
