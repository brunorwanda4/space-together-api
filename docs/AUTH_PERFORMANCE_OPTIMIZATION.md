# Authentication Performance Optimization Guide

## Current Optimizations Applied

The `get_auth_user` function has been optimized with the following improvements:

### 1. Single Database Query
- **Before**: Sequential queries (try email, then username on failure)
- **After**: Single query using appropriate filter based on input validation
- **Impact**: Reduces database round trips from 2 to 1 in worst case

### 2. Early Password Verification
- Password verification happens immediately after user lookup
- Fails fast if password is incorrect
- Prevents unnecessary school data fetching

### 3. Efficient Error Handling
- Uses `ok_or_else` for lazy error creation
- Avoids unnecessary string allocations

## Additional Performance Recommendations

### Database Indexing (CRITICAL)

Ensure these indexes exist on the `users` collection:

```javascript
// MongoDB shell commands
db.users.createIndex({ "email": 1 }, { unique: true });
db.users.createIndex({ "username": 1 }, { unique: true, sparse: true });
db.users.createIndex({ "current_school_id": 1 });
```

**Expected Impact**: 50-80% reduction in query time for user lookups

### Password Hashing Configuration

The bcrypt algorithm is intentionally slow for security. Current performance:
- **Typical bcrypt time**: 100-300ms per verification
- **Recommendation**: Ensure bcrypt cost factor is set to 10-12 (not higher)

To check/adjust bcrypt cost in your hash utility:

```rust
// In src/utils/hash.rs
// Ensure cost is reasonable (10-12 is standard)
const BCRYPT_COST: u32 = 10;
```

### School Member Lookup Optimization

If school member lookups are slow, add indexes to school databases:

```javascript
// For each school database
db.teachers.createIndex({ "user_id": 1 });
db.students.createIndex({ "user_id": 1 });
db.school_staff.createIndex({ "user_id": 1 });
```

### Caching Strategy (Optional)

For high-traffic scenarios, consider caching:

1. **JWT Token Caching**
   - Cache generated tokens for 5-10 seconds
   - Reduces redundant token generation for rapid successive requests

2. **School Member Data Caching**
   - Cache school member lookups with short TTL (30-60 seconds)
   - Invalidate on member updates

3. **User Data Caching**
   - Cache user records with Redis/Memcached
   - TTL: 5-15 minutes
   - Invalidate on user updates

### Connection Pooling

Ensure MongoDB connection pool is properly configured:

```rust
// In database configuration
MongoClientOptions::builder()
    .max_pool_size(100)
    .min_pool_size(10)
    .max_idle_time(Duration::from_secs(600))
    .build()
```

## Performance Monitoring

### Add Timing Logs

```rust
use std::time::Instant;

pub async fn get_auth_user(...) -> Result<LoginResponse, AppError> {
    let start = Instant::now();
    
    // ... existing code ...
    
    let duration = start.elapsed();
    if duration.as_millis() > 500 {
        log::warn!("Slow auth: {}ms for user {}", duration.as_millis(), user_email);
    }
    
    Ok(response)
}
```

### Metrics to Track

1. **User lookup time**: Should be <10ms with proper indexes
2. **Password verification time**: 100-300ms (expected)
3. **School member lookup time**: Should be <20ms with indexes
4. **Token generation time**: Should be <5ms
5. **Total auth time**: Target <400ms, acceptable <1000ms

## Expected Performance After Optimization

| Operation | Before | After (with indexes) |
|-----------|--------|---------------------|
| User lookup | 50-200ms | 5-15ms |
| Password verify | 100-300ms | 100-300ms (unchanged) |
| School member lookup | 50-150ms | 10-30ms |
| Token generation | 5-10ms | 5-10ms |
| **Total** | **205-660ms** | **120-355ms** |

## Troubleshooting Slow Performance

If authentication is still slow (>1000ms):

1. **Check database indexes**: Run `db.users.getIndexes()` in MongoDB shell
2. **Monitor network latency**: Ensure database is geographically close to application
3. **Check bcrypt cost**: Verify it's not set too high (>12)
4. **Profile school member queries**: Add timing logs to `search_single_member`
5. **Check connection pool**: Ensure pool isn't exhausted
6. **Monitor CPU usage**: High CPU may indicate need for horizontal scaling

## Security vs Performance Trade-offs

- **DO NOT** reduce bcrypt cost below 10 (security risk)
- **DO NOT** cache passwords or password hashes
- **DO** cache non-sensitive user data (name, email, role)
- **DO** use short TTLs for cached authentication data
