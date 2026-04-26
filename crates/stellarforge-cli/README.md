# StellarForge CLI

A comprehensive command-line interface for StellarForge smart contracts.

## Installation

```bash
# Build the CLI
make cli

# Or build directly
cargo build -p stellarforge-cli

# Install globally (optional)
make install-cli
```

## Usage

The StellarForge CLI provides a comprehensive set of commands for interacting with StellarForge smart contracts:

### Basic Commands

```bash
# Show help
stellarforge help

# List all available contracts
stellarforge contracts

# Get detailed information about a specific contract
stellarforge contracts --name vesting

# Build all contracts
stellarforge build

# Build a specific contract in release mode
stellarforge build --contract vesting --release

# Run tests
stellarforge test

# Test a specific contract
stellarforge test --contract stream

# Deploy a contract
stellarforge deploy vesting --network testnet

# Get started with development
stellarforge quickstart
```

### Commands Overview

#### `help`
Display comprehensive help information about the CLI and its commands.

```bash
stellarforge help [command]
```

#### `contracts`
List and get detailed information about available contracts.

```bash
stellarforge contracts [--name <contract_name>]
```

Available contracts:
- `vesting` - Token vesting with cliff and linear release
- `stream` - Streaming payments contract
- `multisig` - Multi-signature wallet
- `governor` - Governance voting contract
- `oracle` - Price oracle contract

#### `build`
Build StellarForge contracts.

```bash
stellarforge build [--contract <name>] [--release]
```

#### `test`
Run tests for StellarForge contracts.

```bash
stellarforge test [--contract <name>]
```

#### `deploy`
Deploy contracts to the network.

```bash
stellarforge deploy <contract_type> [--network <network>]
```

Network options: `futurenet`, `testnet`, `mainnet` (default: `futurenet`)

#### `quickstart`
Get started with StellarForge development.

```bash
stellarforge quickstart [--path <path>]
```

## Examples

### Getting Help
```bash
# Show general help
stellarforge help

# Show help for a specific command
stellarforge help contracts
```

### Working with Contracts
```bash
# List all contracts
stellarforge contracts

# Get detailed info about vesting contract
stellarforge contracts --name vesting
```

### Building and Testing
```bash
# Build all contracts
stellarforge build

# Build only the vesting contract in release mode
stellarforge build --contract vesting --release

# Run all tests
stellarforge test

# Test only the stream contract
stellarforge test --contract stream
```

### Deployment
```bash
# Deploy vesting contract to testnet
stellarforge deploy vesting --network testnet

# Deploy governor contract to futurenet (default)
stellarforge deploy governor
```

## Development Setup

The quickstart command provides all the necessary information for setting up a development environment:

```bash
stellarforge quickstart
```

This will display:
- Prerequisites (Rust, WASM target, Stellar CLI)
- Installation instructions
- Build commands
- Testing commands

## Architecture

The CLI is built using Rust's standard library for maximum compatibility and minimal dependencies. It provides:

- **Comprehensive Help**: Detailed usage information for all commands
- **Contract Information**: In-depth descriptions of each contract type
- **Development Tools**: Build, test, and deployment commands
- **Quickstart Guide**: Complete setup instructions for new developers

## Contributing

When adding new features to the CLI:

1. Update the command parsing in `src/main.rs`
2. Add comprehensive help text
3. Update this README if needed
4. Test all new functionality

## License

This CLI is part of the StellarForge project and is licensed under the same terms as the main project.
