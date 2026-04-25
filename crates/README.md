# Crates

This folder contains shared Rust crates used by multiple contracts in the workspace.

## Purpose

- Centralize reusable logic used across contract crates.
- Reduce duplication and keep cross-contract behavior consistent.

## Current Crates

- `forge-errors`: Shared common error variants used across StellarForge contracts.

As the project grows, new shared libraries can be added here when functionality is used by multiple contracts.
