# Messaging System Architecture

## Overview

The messaging and conversation system supports both school-based and cross-school/admin conversations through automatic database routing based on the presence of a school token in request headers.

## Key Concept: Single Endpoint with Optional School Context

The system uses **ONE set of endpoints** that automatically detect school context from the `School-Token` header:

- **With School-Token header** → Routes to school-specific database
- **Without School-Token header** → Routes to main database

## Middleware: OptionalSchoolTokenMiddleware

The `OptionalSchoolTokenMiddleware` is the key component:

```rust
// Checks for School-Token header but doesn't require it
// - If valid token exists → adds claims to request extensions
// - If no token → continues without school context
// - Never blocks the request
```

## Routing Structure

### Single Set of Endpoints

```
/conversations                      # Works with or without School-Token
/conversations/{id}                 # Works with or without School-Token  
/conversations/{id}/messages        # Works with or without School-Token
```

### How Clients Use It

**School Context (Student/Teacher)**:
```http
GET /conversations
Headers:
  School-Token: eyJ...
  Authorization: Bearer eyJ...
```
→ Returns conversations from school database

**No School Context (Admin/Cross-School)**:
```http
GET /conversations
Headers:
  Authorization: Bearer eyJ...
```
→ Returns conversations from main database

## Database Selection

```rust
pub fn get_database(req: &HttpRequest, state: &web::Data<AppState>) -> Database {
    if let Some(claims) = req.extensions().get::<SchoolToken>() {
        // OptionalSchoolTokenMiddleware added this
        return state.db.get_db(&claims.database_name);
    }
    state.db.main_db()
}
```

## Use Cases

### 1. Student in School
- Includes `School-Token` header
- Conversations stored in `school_123` database
- `school_id` field is set

### 2. Admin Without School
- No `School-Token` header
- Conversations stored in `main` database
- `school_id` field is `None`

### 3. Student Changes Schools
- Can include School A token → access School A conversations
- Can include School B token → access School B conversations
- Can omit token → access cross-school conversations

## Implementation Files

- `src/middleware/school_token_middleware.rs` - `OptionalSchoolTokenMiddleware`
- `src/utils/route_utils.rs` - `mount_messaging_routes()`
- `src/utils/db_utils.rs` - `get_database()`
- `src/api/conversations_api.rs` - Conversation endpoints
- `src/api/messages_api.rs` - Message endpoints
