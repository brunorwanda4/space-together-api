# Auth Login School Member Search Fix

## Problem
The login endpoint had multiple compilation errors when trying to search for a school member:
1. `format!` macro syntax error - using `&` before string literal
2. Type mismatch - `format!` returns `String` but `get_db` expects `&str`
3. Error handling issues with `ObjectId::from_str`
4. Type mismatch - `search_single_member` expects `Option<&IdType>` not `ObjectId`

## Solution

### 1. Fixed format! Macro
**Before:**
```rust
let school_db = state.db.get_db(format!(&"school_{}", &school_id));
```

**After:**
```rust
let school_db_name = format!("school_{}", school_id);
let school_db = state.db.get_db(&school_db_name);
```

### 2. Fixed ObjectId Parsing
**Before:**
```rust
let user_id = match response.id {
    None => return HttpResponse::BadRequest().json(AppError{message: "User not have Id"}),
    Some(id) => ObjectId::from_str(&id).map_err(|e| HttpResponse::BadRequest().json(AppError{message: "Some thing went worng"}))
};
```

**After:**
```rust
let user_id = match &response.id {
    None => {
        return HttpResponse::BadRequest().json(AppError {
            message: "User does not have Id".to_string()
        })
    },
    Some(id) => match ObjectId::from_str(id) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::BadRequest().json(AppError {
                message: "Invalid user ID format".to_string()
            })
        }
    }
};
```

### 3. Fixed IdType Conversion
**Before:**
```rust
let school_member = school_service.search_single_member(
    &school_db,
    user_id,  // Wrong: ObjectId instead of Option<&IdType>
    None,
    response.role
).await;
```

**After:**
```rust
let id_type = crate::models::id_model::IdType::ObjectId(user_id);

match school_service.search_single_member(
    &school_db,
    Some(&id_type),  // Correct: Option<&IdType>
    None,
    response.role.clone()
).await {
    Ok(_member) => {
        // Member found in school, continue with login
    },
    Err(_) => {
        // Member not found, but continue with login anyway
    }
}
```

### 4. Removed Unused Imports
```rust
// Removed:
use actix_web::App;
use mongodb::bson::doc;
use serde_json::from_str;
```

## Complete Fixed Login Function

```rust
#[post("/login")]
async fn login_user(data: web::Json<LoginUser>, state: web::Data<AppState>) -> impl Responder {
    let db = state.db.main_db();
    let user_repo = UserRepo::new(&db);
    let auth_service = AuthService::new(&user_repo);

    match auth_service.login(data.into_inner(), &state).await {
        Ok(response) => {
            if let Some(school_id) = response.current_school_id {
                let school_db_name = format!("school_{}", school_id);
                let school_db = state.db.get_db(&school_db_name);
                let school_service = SchoolService::new(&db);

                let user_id = match &response.id {
                    None => {
                        return HttpResponse::BadRequest().json(AppError {
                            message: "User does not have Id".to_string()
                        })
                    },
                    Some(id) => match ObjectId::from_str(id) {
                        Ok(oid) => oid,
                        Err(_) => {
                            return HttpResponse::BadRequest().json(AppError {
                                message: "Invalid user ID format".to_string()
                            })
                        }
                    }
                };

                let id_type = crate::models::id_model::IdType::ObjectId(user_id);
                
                match school_service.search_single_member(
                    &school_db,
                    Some(&id_type),
                    None,
                    response.role.clone()
                ).await {
                    Ok(_member) => {
                        // Member found in school, continue with login
                    },
                    Err(_) => {
                        // Member not found, but continue with login anyway
                    }
                }
            }
            HttpResponse::Ok().json(response)
        },
        Err(message) => HttpResponse::Unauthorized().json(ReqErrModel { message }),
    }
}
```

## Key Points

1. **String vs &str**: `format!` returns `String`, so store it in a variable and pass a reference
2. **Error Handling**: Use proper match statements instead of `map_err` with early returns
3. **Type Conversion**: Convert `ObjectId` to `IdType` before passing to `search_single_member`
4. **Clone**: Clone `response.role` since it's used after the response is returned
5. **Graceful Degradation**: Login succeeds even if school member search fails

## Related Files
- `src/api/auth_api.rs` - Fixed login endpoint
- `src/services/school_service.rs` - `search_single_member` function
- `src/models/id_model.rs` - `IdType` enum definition
