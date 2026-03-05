# Conversation Auto-Participant Fix

## Problem

The `create_conversation` endpoint had a logic issue where it would reject requests if the authenticated user wasn't explicitly included in the participants list. This created unnecessary friction for clients who might forget to include themselves.

## Solution

The endpoint now automatically adds the authenticated user to the participants list if they're not already included, using the appropriate ID and role based on context.

## Changes Made

### Before
```rust
// Validate authenticated user is in participants
let user_in_participants = body
    .participants
    .iter()
    .any(|p| p.id.to_hex() == auth_user.id);

if !user_in_participants {
    return HttpResponse::BadRequest().json(AppError {
        message: "You must be a participant in the conversation".to_string(),
    });
}
```

### After
```rust
// Determine which ID to use for the authenticated user
// If in school context and has current_school_user_id, use that; otherwise use regular id
let auth_user_id = if school_id.is_some() && auth_user.current_school_user_id.is_some() {
    auth_user.current_school_user_id.as_ref().unwrap()
} else {
    &auth_user.id
};

// Check if authenticated user is already in participants
let user_in_participants = participants
    .iter()
    .any(|p| p.id == auth_user_object_id && p.role == auth_user_role);

// If user is not in participants, add them automatically
if !user_in_participants {
    participants.push(ActorRef {
        id: auth_user_object_id,
        role: auth_user_role.clone(),
    });
}
```

## Key Improvements

### 1. Smart ID Selection

The endpoint now intelligently selects which ID to use for the authenticated user:

- **School Context**: If the user is in a school context (has `school_id`) and has a `current_school_user_id`, it uses the school-specific ID
- **Non-School Context**: Uses the regular user ID

This ensures conversations work correctly in both school-specific and cross-school scenarios.

### 2. Automatic Participant Addition

If the authenticated user is not in the participants list, they are automatically added with:
- The appropriate ID (school-specific or regular)
- Their current role from the auth token

### 3. Improved Validation Order

The validation now happens in the correct order:
1. Add auth user if missing
2. Validate minimum 2 participants (after potentially adding auth user)
3. Validate maximum participants
4. Check for duplicates
5. Validate encrypted keys

### 4. Better Duplicate Detection

The duplicate check now considers both ID and role:
```rust
let key = format!("{}:{:?}", participant.id.to_hex(), participant.role);
```

This prevents issues where the same user with different roles could be added multiple times.

## API Behavior

### Scenario 1: User Already in Participants

**Request**:
```json
{
  "participants": [
    { "id": "user123", "role": "STUDENT" },
    { "id": "user456", "role": "TEACHER" }
  ],
  "encrypted_keys": [
    { "user_id": "user123", "user_role": "STUDENT", "encrypted_key": "..." },
    { "user_id": "user456", "user_role": "TEACHER", "encrypted_key": "..." }
  ],
  "is_group": false
}
```

**Behavior**: Works as before, no changes needed.

### Scenario 2: User Not in Participants (School Context)

**Request** (from user with `current_school_user_id = "school_user_789"`):
```json
{
  "participants": [
    { "id": "user456", "role": "TEACHER" }
  ],
  "encrypted_keys": [
    { "user_id": "user456", "user_role": "TEACHER", "encrypted_key": "..." },
    { "user_id": "school_user_789", "user_role": "STUDENT", "encrypted_key": "..." }
  ],
  "is_group": false
}
```

**Behavior**: 
- Automatically adds `{ "id": "school_user_789", "role": "STUDENT" }` to participants
- Validates that encrypted keys match (2 keys for 2 participants)
- Creates conversation successfully

### Scenario 3: User Not in Participants (Non-School Context)

**Request** (from user with `id = "user123"`):
```json
{
  "participants": [
    { "id": "user456", "role": "TEACHER" }
  ],
  "encrypted_keys": [
    { "user_id": "user456", "user_role": "TEACHER", "encrypted_key": "..." },
    { "user_id": "user123", "user_role": "STUDENT", "encrypted_key": "..." }
  ],
  "is_group": false
}
```

**Behavior**:
- Automatically adds `{ "id": "user123", "role": "STUDENT" }` to participants
- Validates that encrypted keys match (2 keys for 2 participants)
- Creates conversation successfully

### Scenario 4: Only One Participant Provided

**Request**:
```json
{
  "participants": [
    { "id": "user456", "role": "TEACHER" }
  ],
  "encrypted_keys": [
    { "user_id": "user456", "user_role": "TEACHER", "encrypted_key": "..." }
  ],
  "is_group": false
}
```

**Behavior**:
- Automatically adds authenticated user to participants (now 2 participants)
- **ERROR**: "Must provide exactly one encrypted key per participant. Expected 2, got 1"
- Client must provide encrypted key for themselves

## Client Implementation Guide

### Option 1: Always Include Yourself (Recommended)

```typescript
// Always include the authenticated user in participants
const createConversation = async (otherUserId: string) => {
  const response = await fetch('/api/conversations', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      participants: [
        { id: currentUser.schoolUserId || currentUser.id, role: currentUser.role },
        { id: otherUserId, role: 'STUDENT' }
      ],
      encrypted_keys: [
        { user_id: currentUser.schoolUserId || currentUser.id, user_role: currentUser.role, encrypted_key: myKey },
        { user_id: otherUserId, user_role: 'STUDENT', encrypted_key: theirKey }
      ],
      is_group: false
    })
  });
};
```

### Option 2: Let Server Add You (Simpler)

```typescript
// Only include other participants, server adds you automatically
const createConversation = async (otherUserId: string) => {
  const response = await fetch('/api/conversations', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      participants: [
        { id: otherUserId, role: 'STUDENT' }
      ],
      encrypted_keys: [
        { user_id: currentUser.schoolUserId || currentUser.id, user_role: currentUser.role, encrypted_key: myKey },
        { user_id: otherUserId, user_role: 'STUDENT', encrypted_key: theirKey }
      ],
      is_group: false
    })
  });
};
```

**Important**: Even with Option 2, you must still provide your encrypted key in the `encrypted_keys` array.

## Error Messages

### Improved Error Message

**Before**:
```json
{
  "message": "Must provide exactly one encrypted key per participant"
}
```

**After**:
```json
{
  "message": "Must provide exactly one encrypted key per participant. Expected 2, got 1"
}
```

This helps clients understand exactly how many keys are needed.

## Testing

### Test Case 1: User Not in Participants, Correct Keys

```bash
curl -X POST http://localhost:4646/api/conversations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "participants": [
      {"id": "other_user_id", "role": "TEACHER"}
    ],
    "encrypted_keys": [
      {"user_id": "my_school_user_id", "user_role": "STUDENT", "encrypted_key": "base64key1"},
      {"user_id": "other_user_id", "user_role": "TEACHER", "encrypted_key": "base64key2"}
    ],
    "is_group": false
  }'
```

**Expected**: 201 Created with conversation object

### Test Case 2: User Not in Participants, Missing Key

```bash
curl -X POST http://localhost:4646/api/conversations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "participants": [
      {"id": "other_user_id", "role": "TEACHER"}
    ],
    "encrypted_keys": [
      {"user_id": "other_user_id", "user_role": "TEACHER", "encrypted_key": "base64key2"}
    ],
    "is_group": false
  }'
```

**Expected**: 400 Bad Request - "Must provide exactly one encrypted key per participant. Expected 2, got 1"

### Test Case 3: Duplicate Participants

```bash
curl -X POST http://localhost:4646/api/conversations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "participants": [
      {"id": "user_id", "role": "STUDENT"},
      {"id": "user_id", "role": "STUDENT"}
    ],
    "encrypted_keys": [
      {"user_id": "user_id", "user_role": "STUDENT", "encrypted_key": "base64key1"},
      {"user_id": "user_id", "user_role": "STUDENT", "encrypted_key": "base64key2"}
    ],
    "is_group": false
  }'
```

**Expected**: 400 Bad Request - "Duplicate participants are not allowed"

## Migration Notes

### For Existing Clients

No changes required! Existing code that includes the authenticated user in participants will continue to work exactly as before.

### For New Clients

You can now simplify your code by:
1. Only including other participants in the `participants` array
2. Still providing all encrypted keys (including your own) in `encrypted_keys`

## Security Considerations

1. **Encrypted Keys Required**: Even though the server adds the user to participants, the client must still provide the encrypted key. This ensures the user can decrypt messages.

2. **Role Validation**: The server uses the role from the auth token, preventing privilege escalation.

3. **ID Selection**: The server intelligently selects between `current_school_user_id` and regular `id`, ensuring proper context isolation.

## Related Files

- `src/api/conversations_api.rs` - Main implementation
- `src/domain/conversation.rs` - Conversation domain model
- `src/schema/common_schema.rs` - ActorRef structure
- `src/domain/auth_user.rs` - AuthUserDto with current_school_user_id
