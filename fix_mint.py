#!/usr/bin/env python3
"""Fix mint calls back to StellarAssetClient."""

import re

path = "contracts/forge-multisig/src/lib.rs"
with open(path, "r") as f:
    content = f.read()

# Pattern: token::Client::new(...).mint(...)
# Should be: token::StellarAssetClient::new(...).mint(...)
# Replace Client::new(...)...mint( with StellarAssetClient::new(...)...mint(
content = re.sub(
    r"soroban_sdk::token::Client::new\(([^)]+)\)\.mint\(",
    r"soroban_sdk::token::StellarAssetClient::new(\1).mint(",
    content,
)

# Also handle token::Client::new(...).mint( in setup_token helper
content = re.sub(
    r"token::Client::new\(([^)]+)\)\.mint\(",
    r"token::StellarAssetClient::new(\1).mint(",
    content,
)

# Also handle StellarAssetClient::new(...).balance( that might have been introduced
# These should be Client::new(...).balance(
content = re.sub(
    r"StellarAssetClient::new\(([^)]+)\)\.balance\(",
    r"Client::new(\1).balance(",
    content,
)

with open(path, "w") as f:
    f.write(content)

print("Mint fixes applied.")

