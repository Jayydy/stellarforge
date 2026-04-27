#!/usr/bin/env python3
"""Fix forge-multisig test compilation errors."""

import re

path = "contracts/forge-multisig/src/lib.rs"
with open(path, "r") as f:
    content = f.read()

# 1) Remove .unwrap() after client.get_proposal(...) calls
# Pattern: client.get_proposal(&X).unwrap()
# These return Proposal directly in SDK 21.x
content = re.sub(
    r"client\.get_proposal\(([^)]+)\)\.unwrap\(\)",
    r"client.get_proposal(\1)",
    content,
)

# 2) Handle nested unwrap chains like client.get_proposal(&pid).unwrap().field
# After step 1, these should already be fixed. Let's double-check for remaining patterns
content = re.sub(
    r"client\.get_proposal\(([^)]+)\)\.unwrap\(\)([.])",
    r"client.get_proposal(\1)\2",
    content,
)

# 3) Replace StellarAssetClient(...).balance(...) with token::Client(...).balance(...)
# Pattern: soroban_sdk::token::StellarAssetClient::new(&env, &token_id).balance(&addr)
# Should be: soroban_sdk::token::Client::new(&env, &token_id).balance(&addr)
content = content.replace(
    "soroban_sdk::token::StellarAssetClient::new",
    "soroban_sdk::token::Client::new",
)
content = content.replace(
    "token::StellarAssetClient::new",
    "token::Client::new",
)

# 4) Also need to fix cases where there are two StellarAssetClient usages in one expression
# These were already covered by the string replacements above

# 5) Fix the mangled duplicate test function on lines 1358-1360
# Pattern: fn test_get_approval_count_tracks_lifecycle_through_execution() {
#     fn test_get_approval_count_full_lifecycle_including_execution() {
# Need to remove the first line and dedupe the test body
content = re.sub(
    r"    fn test_get_approval_count_tracks_lifecycle_through_execution\(\) \{\n    fn test_get_approval_count_full_lifecycle_including_execution\(\) \{",
    "    fn test_get_approval_count_full_lifecycle_including_execution() {",
    content,
)

# 6) Fix is_none() called on Proposal (which returns Proposal, not Option)
# Pattern: client.get_proposal(&pid).is_none() —> impossible, replace with appropriate pattern
# Actually we need to look at what this is trying to do.
# The pattern appears to be checking if a proposal exists (for get_proposal(&999),
# but get_proposal returns Proposal directly, not Option.
# For non-existent proposals, get_proposal will panic.
# So for checking non-existence, we should use try_get_proposal instead.
content = re.sub(
    r"client\.get_proposal\(([^)]+)\)\.is_none\(\)",
    r"client.try_get_proposal(\1).is_err()",
    content,
)

# 7) Fix assert_eq!(client.get_proposal(&pid).unwrap_err().unwrap(), ...)
# The first unwrap_err() extracts from Result<Result<T, E>, HostError>
# after try_* call. But if we changed get_proposal to try_get_proposal, we might have issues.
# Let's check what's actually in the file.

# Actually, for the client generated methods:
# - `client.get_proposal(&pid)` returns Result<Proposal, MultisigError> (returns on Ok, panics on Err)
# Wait no, looking at errors: "no method unwrap found for struct Proposal" means get_proposal returns PROPOSAL directly
# So client methods that return Result in the contract actually return T directly on the client
# For error handling, one uses `client.try_get_proposal()` which returns Result<Result<T, E>, HostError>

# The existing patterns like:
#   client.try_get_proposal(&999).unwrap_err().unwrap() -> MultisigError
# should still work.

# But patterns like:
#   client.get_proposal(&pid).unwrap().field  -> should be client.get_proposal(&pid).field
# are what we fixed in step 1.

# Let's also handle cases where get_proposal(&999).is_none() should be try_get_proposal

# 8) Fix Debug issue - when using assert_eq!(...) on Proposal
# If there are comparisons using Proposal with Debug not implemented, we need to
# check specific fields instead. But the error E0277 says Proposal doesn't implement Debug
# This likely happens when assert_eq! is used on a Proposal directly or on a Result<Proposal, _>
# Let's look for assert_eq! on Proposal-containing expressions

with open(path, "w") as f:
    f.write(content)

print("Replacements applied.")

