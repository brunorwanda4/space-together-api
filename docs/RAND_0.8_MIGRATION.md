# Rand 0.8 Migration Guide

## Overview
After downgrading from `rand = "0.9.2"` to `rand = "0.8.5"` for compatibility with `rsa = "0.9.6"`, several API changes needed to be addressed across the codebase.

## Changes Required

### API Differences: rand 0.9 → rand 0.8

| rand 0.9 | rand 0.8 | Notes |
|----------|----------|-------|
| `rand::rng()` | `rand::thread_rng()` | Function renamed |
| `IndexedRandom` | `SliceRandom` | Trait renamed |
| `IndexedMutRandom` | `SliceRandom` | Trait consolidated |

## Files Updated

### 1. src/utils/code.rs
```rust
// Before (rand 0.9)
use rand::{rng, seq::IndexedRandom};
let mut rng = rng();
chars.as_slice().choose(&mut rng)

// After (rand 0.8)
use rand::{seq::SliceRandom, thread_rng};
let mut rng = thread_rng();
chars.choose(&mut rng)
```

### 2. src/utils/names.rs
```rust
// Before (rand 0.9)
use rand::{rng, seq::SliceRandom};
let mut rng = rng();

// After (rand 0.8)
use rand::{seq::SliceRandom, thread_rng};
let mut rng = thread_rng();
```

### 3. src/handler/class_timetable_handler.rs
```rust
// Before (rand 0.9)
use rand::prelude::IndexedMutRandom;
use rand::seq::SliceRandom;
use rand::rng;
let mut rng = rng();

// After (rand 0.8)
use rand::{seq::SliceRandom, thread_rng};
let mut rng = thread_rng();
```

### 4. src/utils/crypto_utils.rs
```rust
// Using OsRng (works in both versions)
use rand::rngs::OsRng;
let mut rng = OsRng;
```

## Key Changes Summary

### 1. Import Changes
- Replace `rand::rng` with `rand::thread_rng`
- Replace `rand::seq::IndexedRandom` with `rand::seq::SliceRandom`
- Remove `rand::prelude::IndexedMutRandom` (use `SliceRandom` instead)

### 2. Function Calls
- Replace `rng()` with `thread_rng()`
- `.choose()` works the same way on slices
- No need for `.as_slice()` when calling `.choose()` on `Vec<T>`

### 3. Traits
- `SliceRandom` provides both `choose()` and `shuffle()` methods
- Import from `rand::seq::SliceRandom`

## Testing

After migration, verify:
```bash
cargo check
# Should compile without errors
```

## Why This Migration?

The `rsa = "0.9.6"` crate depends on `rand = "0.8.x"` and its trait system. Using `rand = "0.9.x"` causes trait bound errors because the trait definitions changed between versions.

## Alternative Approach

If you need rand 0.9 features, consider upgrading rsa instead:
```toml
rsa = "0.10.0"  # or latest
rand = "0.9.2"
```

However, this may require additional code changes if the rsa API has changed.

## Compatibility Notes

- `thread_rng()` is cryptographically secure (uses OS entropy)
- `OsRng` is also cryptographically secure and works in both versions
- Performance characteristics are similar between versions
- No functional changes to random number generation quality
