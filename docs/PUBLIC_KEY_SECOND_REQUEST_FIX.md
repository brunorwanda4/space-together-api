# Public Key Second Request Fix

## Problem
After the duplicate key fix, a new issue appeared:
- **First request**: Key created successfully ✓
- **Second request**: Error - "Public key not found for user: 69787b725448cbee08a17122" ✗

## Root Cause Analysis

### What Was Happening
1. Second request calls `get_public_keys_partial()` 
2. Query doesn't find the key (even though it exists!)
3. Marks user as "missing"
4. Tries to create key → Duplicate key error
5. Catches duplicate error, tries to fetch key
6. `get_public_key()` still doesn't find it
7. Returns error: "Public key not found"

### Why It Happened
The issue was that **indexes weren't being ensured before queries**:

- `try_create_public_key()` didn't call `ensure_indexes()` before inserting
- `get_public_keys_partial()` didn't call `ensure_indexes()` before querying  
- `get_public_key()` didn't call `ensure_indexes()` before querying

Without the unique index on `user_id`:
1. Multiple keys could be inserted for the same user (no constraint)
2. Queries might not use the index efficiently
3. Race conditions could cause inconsistent results

## Solution

### Ensure Indexes Before All Operations

Added `self.ensure_indexes().await?` to:

1. **`get_public_keys_partial()`** - Before querying
2. **`get_public_key()`** - Before querying
3. **`try_create_public_key()`** - Before inserting

### Code Changes

#### Before
```rust
pub async fn get_public_keys_partial(...) {
    // Validation...
    
    let filter = doc! { "user_id": { "$in": user_ids.clone() } };
    let cursor = self.collection.find(filter).await?;
    // ...
}

pub async fn get_public_key(...) {
    let repo = BaseRepository::new(...);
    repo.find_one::<UserPublicKey>(doc! { "user_id": user_id }, None).await?
    // ...
}

async fn try_create_public_key(...) {
    let public_key = generate_rsa_public_key()?;
    let new_key = UserPublicKey { ... };
    repo.create::<UserPublicKey>(doc, None).await
}
```

#### After
```rust
pub async fn get_public_keys_partial(...) {
    // Validation...
    
    // Ensure indexes exist
    self.ensure_indexes().await?;
    
    let filter = doc! { "user_id": { "$in": user_ids.clone() } };
    let cursor = self.collection.find(filter).await?;
    // ...
}

pub async fn get_public_key(...) {
    // Ensure indexes exist
    self.ensure_indexes().await?;
    
    let repo = BaseRepository::new(...);
    repo.find_one::<UserPublicKey>(doc! { "user_id": user_id }, None).await?
    // ...
}

async fn try_create_public_key(...) {
    // Ensure indexes exist (including unique index on user_id)
    self.ensure_indexes().await?;
    
    let public_key = generate_rsa_public_key()?;
    let new_key = UserPublicKey { ... };
    repo.create::<UserPublicKey>(doc, None).await
}
```

## What `ensure_indexes()` Does

```rust
pub async fn ensure_indexes(&self) -> Result<(), AppError> {
    let indexes = vec![IndexDef::single("user_id", true)]; // true = unique
    let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
    repo.ensure_indexes(&indexes).await?;
    Ok(())
}
```

Creates a unique index on `user_id` field:
- **Unique constraint**: Prevents duplicate user_id values
- **Query optimization**: Makes lookups by user_id fast
- **Idempotent**: Safe to call multiple times (MongoDB handles it)

## Benefits

1. **Consistency**: Indexes are always present before operations
2. **Performance**: Queries use indexes efficiently
3. **Data Integrity**: Unique constraint prevents duplicates at database level
4. **Reliability**: No race conditions from missing indexes

## Testing

### Test Scenario 1: First Request
```bash
GET /api/m-users/public-keys?user_ids=user1
→ 200 OK
→ Key created with unique index enforced
```

### Test Scenario 2: Second Request (Same User)
```bash
GET /api/m-users/public-keys?user_ids=user1
→ 200 OK
→ Existing key found and returned
```

### Test Scenario 3: Concurrent Requests
```bash
# Two simultaneous requests
Request A: GET /api/m-users/public-keys?user_ids=user1
Request B: GET /api/m-users/public-keys?user_ids=user1

→ Both return 200 OK
→ Only one key in database
→ Both return the same key
```

## Performance Impact

- **Minimal**: `ensure_indexes()` is idempotent and cached by MongoDB
- **First call**: Creates index (one-time cost)
- **Subsequent calls**: No-op (MongoDB recognizes index exists)
- **Overall**: Negligible overhead, significant reliability gain

## Related Files
- `src/services/user_public_key_service.rs` - All methods updated
- `docs/DUPLICATE_KEY_FIX.md` - Previous fix documentation
- `docs/PUBLIC_KEY_AUTO_GENERATION.md` - Feature documentation
