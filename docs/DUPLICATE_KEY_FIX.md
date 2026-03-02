# Duplicate Key Error Fix - Public Keys

## Problem
When requesting public keys for users who don't have them, the system was encountering duplicate key errors:

```
E11000 duplicate key error collection: main_database.user_public_keys 
index: user_id_1 dup key: { user_id: "69a36b0dfc062ade140f54b6" }
```

## Root Cause

### Race Condition
The issue occurred due to a race condition in the `get_or_create_public_keys` method:

1. Request A checks if user has a key → No key found
2. Request B checks if user has a key → No key found (happens before A creates it)
3. Request A generates and inserts key → Success
4. Request B generates and tries to insert key → **Duplicate key error!**

### Why It Happened
The original implementation used `upsert_public_key` which:
1. First checks if key exists
2. Then creates or updates

But between step 1 and 2, another request could create the key, causing a duplicate insert attempt.

## Solution

### Optimistic Insert with Fallback
Instead of check-then-insert, we now use insert-then-handle-duplicate:

```rust
// Try to insert directly
match self.try_create_public_key(user_id).await {
    Ok(created_key) => {
        // Success - use the newly created key
    }
    Err(e) if e.message.contains("duplicate key error") => {
        // Key was created by another request - fetch it instead
        let existing_key = self.get_public_key(user_id).await?;
        // Use the existing key
    }
    Err(e) => {
        // Other error - propagate it
        return Err(e);
    }
}
```

### New Method: `try_create_public_key`
```rust
async fn try_create_public_key(&self, user_id: ObjectId) -> Result<UserPublicKey, AppError>
```

This method:
- Generates a new RSA key
- Attempts direct insertion (no pre-check)
- Relies on MongoDB's unique index to prevent duplicates
- Returns error if duplicate exists (which we handle gracefully)

## Benefits

1. **Race Condition Safe**: Multiple simultaneous requests won't cause errors
2. **Atomic**: Uses database constraints instead of application-level checks
3. **Efficient**: One database operation instead of two (check + insert)
4. **Graceful**: Handles duplicates by fetching the existing key

## Code Changes

### File: `src/services/user_public_key_service.rs`

**Before:**
```rust
pub async fn get_or_create_public_keys(...) {
    let (mut public_keys, missing_ids) = self.get_public_keys_partial(...).await?;
    
    for missing_id_str in missing_ids {
        let user_id = ObjectId::parse_str(&missing_id_str)?;
        let public_key = generate_rsa_public_key()?;
        
        // This could fail with duplicate key error!
        let created_key = self.upsert_public_key(user_id, public_key, ...).await?;
        
        public_keys.push(...);
    }
    
    Ok(public_keys)
}
```

**After:**
```rust
pub async fn get_or_create_public_keys(...) {
    let (mut public_keys, missing_ids) = self.get_public_keys_partial(...).await?;
    
    for missing_id_str in missing_ids {
        let user_id = ObjectId::parse_str(&missing_id_str)?;
        
        // Try to create, handle duplicate gracefully
        match self.try_create_public_key(user_id).await {
            Ok(created_key) => {
                // New key created successfully
                public_keys.push(...);
            }
            Err(e) if e.message.contains("duplicate key error") => {
                // Key exists, fetch it
                let existing_key = self.get_public_key(user_id).await?;
                public_keys.push(...);
            }
            Err(e) => return Err(e),
        }
    }
    
    Ok(public_keys)
}

async fn try_create_public_key(&self, user_id: ObjectId) -> Result<UserPublicKey, AppError> {
    let public_key = generate_rsa_public_key()?;
    let new_key = UserPublicKey { user_id, public_key, ... };
    
    // Direct insert - will fail if duplicate exists
    repo.create::<UserPublicKey>(doc, None).await
}
```

## Testing

### Test Scenario 1: Normal Creation
```bash
# User has no key
GET /api/m-users/public-keys?user_ids=user1
→ 200 OK (key auto-generated)
```

### Test Scenario 2: Existing Key
```bash
# User already has key
GET /api/m-users/public-keys?user_ids=user1
→ 200 OK (existing key returned)
```

### Test Scenario 3: Race Condition
```bash
# Two simultaneous requests for same user without key
Request A: GET /api/m-users/public-keys?user_ids=user1
Request B: GET /api/m-users/public-keys?user_ids=user1

→ Both return 200 OK
→ Only one key created in database
→ Both requests return the same key
```

## Database Constraints

The fix relies on the unique index on `user_id`:
```javascript
db.user_public_keys.createIndex({ user_id: 1 }, { unique: true })
```

This is already ensured by the `ensure_indexes()` method in the service.

## Performance Impact

- **Positive**: Reduced from 2 DB operations (check + insert) to 1 (insert)
- **Negligible**: Rare duplicate errors are caught and handled with a single fetch
- **Overall**: Faster in the common case, same speed in race condition case

## Related Files
- `src/services/user_public_key_service.rs` - Main fix
- `src/api/messaging_users_api.rs` - API endpoint (unchanged)
- `docs/PUBLIC_KEY_AUTO_GENERATION.md` - Feature documentation
