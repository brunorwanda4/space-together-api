# Crypto Utils Fix - ThreadRng Trait Bound Issue

## Problem
```
error: the trait bound `ThreadRng: CryptoRngCore` is not satisfied
```

This error occurred because of a version mismatch between the `rand` and `rsa` crates.

## Root Cause
- `rsa = "0.9.6"` expects `rand = "0.8.x"` 
- Project was using `rand = "0.9.2"` which has incompatible trait definitions
- `ThreadRng` from rand 0.9 doesn't implement the `CryptoRngCore` trait that rsa 0.9 expects

## Solution

### 1. Downgrade rand to 0.8.5
**File**: `Cargo.toml`
```toml
# Changed from:
rand = "0.9.2"

# To:
rand = "0.8.5"
```

### 2. Use OsRng instead of ThreadRng
**File**: `src/utils/crypto_utils.rs`
```rust
// Changed from:
use rand::thread_rng;
let mut rng = thread_rng();

// To:
use rand::rngs::OsRng;
let mut rng = OsRng;
```

## Why OsRng?

`OsRng` (Operating System Random Number Generator) is:
- **Cryptographically secure**: Uses the OS's secure random number generator
- **Compatible**: Works with both rand 0.8 and rsa 0.9
- **Recommended**: Best practice for cryptographic key generation
- **No state**: Doesn't need to be mutable in newer versions, but we keep `mut` for compatibility

## Alternative Solutions

If you need to keep rand 0.9, you would need to upgrade rsa to a newer version:
```toml
rsa = "0.10.0"  # or latest
rand = "0.9.2"
```

However, this might require code changes if the rsa API has changed.

## Testing

After the fix, the code compiles successfully:
```bash
cargo check
# ✓ No errors
```

## Impact

- No functional changes to the key generation logic
- Same security level (both are cryptographically secure RNGs)
- Compatible with existing code
- No breaking changes to the API
