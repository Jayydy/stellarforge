# Code Quality Standards

## Debug Logs

The StellarForge codebase maintains a strict policy on debug logging:

### ✅ Current Status

- **No debug print statements** (`println!`, `dbg!`, `eprintln!`) in production code
- **No leftover debug logs** from development
- **Clean compilation** with no debug output

### 🔍 Allowed Debug Code

The following debug-related code is acceptable:

1. **Debug Assertions** (`debug_assert!`, `debug_assert_eq!`)
   - Compiled out in release builds
   - Used for invariant checking during development
   - Example: `contracts/forge-stream/src/lib.rs` uses `debug_assert_eq!` to verify token accounting

2. **Debug Trait Derivations** (`#[derive(Debug)]`)
   - Required for error types and data structures
   - Enables better error messages and testing
   - Does not produce runtime output

3. **Documentation Examples** (in doc comments)
   - May contain `println!` for illustration purposes
   - Not compiled into production code
   - Clearly marked as examples

### 🚫 Prohibited

- `println!()` in production code paths
- `dbg!()` macro calls
- `eprintln!()` for debugging
- Commented-out debug code
- Temporary logging statements

### 🔧 Verification

To verify the codebase is clean of debug logs:

```bash
# Search for debug print statements (should return no results)
rg "^\s*(println!|dbg!|eprintln!)" --type rust

# Search for TODO/FIXME related to debug code
rg "TODO.*debug|FIXME.*debug" --type rust -i
```

### 📝 Best Practices

When developing:

1. **Use tests instead of print debugging** - Write unit tests to verify behavior
2. **Use debug assertions** - For invariant checking that's compiled out in release
3. **Remove before committing** - Clean up any temporary debug code
4. **Use proper error handling** - Return errors instead of printing them

### 🧪 Testing

For debugging during test development, use:

```rust
#[cfg(test)]
mod tests {
    use std::println; // Only available in test module
    
    #[test]
    fn test_something() {
        println!("Debug output only in tests");
        // test code...
    }
}
```

This ensures debug output is isolated to test code and never reaches production.

## Code Review Checklist

Before submitting a PR, verify:

- [ ] No `println!`, `dbg!`, or `eprintln!` in production code
- [ ] No commented-out debug code
- [ ] No TODO/FIXME comments related to debug logging
- [ ] Tests pass without debug output
- [ ] `cargo clippy` passes with no warnings

---

Last verified: 2024
Status: ✅ Clean - No debug logs found
