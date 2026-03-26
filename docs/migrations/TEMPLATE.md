# Migration Guide Template

This document provides a template for creating migration guides when breaking changes are introduced to StellarForge contracts.

---

## Template Structure

When creating a migration guide for a breaking change, use the following structure:

```markdown
# [Contract Name] Migration Guide: vX.Y.Z → vX.Y.Z

## Overview

Brief description of what changed and why.

## Breaking Changes

List all breaking changes with clear explanations:

### 1. [Change Title]

**What Changed:**
- Description of the change

**Why:**
- Reason for the change

**Impact:**
- How this affects existing deployments

**Migration Steps:**

1. Step-by-step instruction
2. Another step
3. Final step

**Code Example:**

```rust
// Before (old version)
fn old_function() { ... }

// After (new version)
fn new_function() { ... }
```

### 2. [Next Change]

[Repeat structure above]

## Compatibility Matrix

| Feature | Old Version | New Version | Notes |
| :--- | :--- | :--- | :--- |
| Function A | ✅ Supported | ✅ Supported | No changes |
| Function B | ✅ Supported | ❌ Removed | Use Function C instead |
| Function C | ❌ Not available | ✅ New | Replacement for Function B |

## Testing Checklist

Before deploying the new version, verify:

- [ ] All existing functionality works as expected
- [ ] New features are tested
- [ ] Error cases are handled correctly
- [ ] State transitions are correct
- [ ] Events are emitted properly

## Rollback Plan

If issues are discovered after deployment:

1. Describe how to rollback
2. Include any data migration considerations
3. Note any downtime required

## Support

For questions or issues:
- Open a GitHub Issue
- Check GitHub Discussions
- Review the CHANGELOG.md

---

## Example Migration Guide

Here's a complete example for a hypothetical breaking change:

# Forge Vesting Migration Guide: v1.0.0 → v2.0.0

## Overview

This guide covers the migration from v1.0.0 to v2.0.0 of forge-vesting. The main breaking change is the addition of support for multiple beneficiaries per vesting schedule.

## Breaking Changes

### 1. Single Beneficiary to Multiple Beneficiaries

**What Changed:**
- The `initialize` function now accepts an array of beneficiaries instead of a single beneficiary
- Each beneficiary can have different vesting parameters

**Why:**
- Allows more flexible token distribution
- Reduces the number of contract deployments needed for team allocations

**Impact:**
- Existing single-beneficiary vesting schedules must be migrated to the new format
- The `beneficiary` field in storage is now `beneficiaries` (array)

**Migration Steps:**

1. Export existing vesting data from the old contract
2. Transform the data to the new format
3. Deploy the new contract version
4. Initialize with the migrated data
5. Verify all vesting schedules are correct

**Code Example:**

```rust
// Before (v1.0.0)
client.initialize(
    &token,
    &beneficiary,  // Single address
    &admin,
    &total_amount,
    &cliff_seconds,
    &duration_seconds
);

// After (v2.0.0)
client.initialize(
    &token,
    &[
        Beneficiary {
            address: beneficiary1,
            amount: amount1,
            cliff_seconds: cliff_seconds1,
            duration_seconds: duration_seconds1,
        },
        Beneficiary {
            address: beneficiary2,
            amount: amount2,
            cliff_seconds: cliff_seconds2,
            duration_seconds: duration_seconds2,
        }
    ],
    &admin
);
```

## Compatibility Matrix

| Feature | v1.0.0 | v2.0.0 | Notes |
| :--- | :--- | :--- | :--- |
| Single beneficiary | ✅ Supported | ❌ Removed | Use array with one element |
| Multiple beneficiaries | ❌ Not available | ✅ New | Main new feature |
| Admin transfer | ✅ Supported | ✅ Supported | No changes |
| Claim function | ✅ Supported | ✅ Supported | Now takes beneficiary index |

## Testing Checklist

Before deploying v2.0.0, verify:

- [ ] Single beneficiary schedules work (array with one element)
- [ ] Multiple beneficiary schedules work
- [ ] Each beneficiary can claim independently
- [ ] Cliff periods work correctly for all beneficiaries
- [ ] Admin can cancel individual beneficiaries
- [ ] Events are emitted correctly

## Rollback Plan

If issues are discovered:

1. Pause the new contract using admin functions
2. Export any claimed amounts
3. Deploy v1.0.0 contract
4. Reinitialize with original data
5. Resume operations

**Note:** This requires downtime and careful data management.

## Support

For questions or issues:
- Open a GitHub Issue with the "migration" label
- Check GitHub Discussions for similar issues
- Review the CHANGELOG.md for additional details

---

## Best Practices

1. **Test Thoroughly:** Always test migrations on testnet before production
2. **Document Everything:** Include all breaking changes, no matter how small
3. **Provide Examples:** Code examples help developers understand changes quickly
4. **Version Clearly:** Use semantic versioning and clearly indicate which versions the guide applies to
5. **Include Rollback:** Always provide a rollback plan in case of issues
6. **Update Promptly:** Create migration guides as soon as breaking changes are merged

---

## Questions?

If you need help creating a migration guide, please open an issue or discussion on GitHub.
