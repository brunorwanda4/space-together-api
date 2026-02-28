# Conversation Creation & User Public Keys API

## Overview
This document describes the API endpoints required for creating conversations and managing encryption keys in the messaging system. These endpoints support end-to-end encrypted messaging with multiple participants.

---

## Table of Contents
1. [Create Conversation](#1-create-conversation)
2. [Get User Public Keys](#2-get-user-public-keys)
3. [Encryption Flow](#3-encryption-flow)
4. [Error Handling](#4-error-handling)

---

## 1. Create Conversation

### Endpoint
```
POST /conversations
```

Creates a new conversation with specified participants and encrypted keys for each participant.

### Authentication
- **Required**: Yes
- **Headers**: 
  - `Authorization: Bearer {token}`
  - `School-Token: {school_token}` (optional, for school context)

### Request Body

```typescript
interface CreateConversationRequest {
  participants: ActorRef[];           // Array of participant references
  is_group: boolean;                  // Whether this is a group conversation
  name?: string;                      // Group name (required if is_group = true)
  encrypted_keys: EncryptedKeyForUser[];  // Encrypted conversation keys
}

interface ActorRef {
  id: string;                         // User ID
  role: "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT" | "ADMIN";
}

interface EncryptedKeyForUser {
  user_id: string;                    // User ID
  user_role: string;                  // User role (STUDENT, TEACHER, etc.)
  encrypted_key: string;              // Base64 encoded encrypted symmetric key
}
```

### Request Example

```json
{
  "participants": [
    {
      "id": "507f1f77bcf86cd799439011",
      "role": "TEACHER"
    },
    {
      "id": "507f1f77bcf86cd799439012",
      "role": "STUDENT"
    }
  ],
  "is_group": false,
  "encrypted_keys": [
    {
      "user_id": "507f1f77bcf86cd799439011",
      "user_role": "TEACHER",
      "encrypted_key": "base64_encrypted_key_for_teacher"
    },
    {
      "user_id": "507f1f77bcf86cd799439012",
      "user_role": "STUDENT",
      "encrypted_key": "base64_encrypted_key_for_student"
    }
  ]
}
```

### Group Conversation Example

```json
{
  "participants": [
    {
      "id": "507f1f77bcf86cd799439011",
      "role": "TEACHER"
    },
    {
      "id": "507f1f77bcf86cd799439012",
      "role": "STUDENT"
    },
    {
      "id": "507f1f77bcf86cd799439013",
      "role": "PARENT"
    }
  ],
  "is_group": true,
  "name": "Math Study Group",
  "encrypted_keys": [
    {
      "user_id": "507f1f77bcf86cd799439011",
      "user_role": "TEACHER",
      "encrypted_key": "base64_encrypted_key_for_teacher"
    },
    {
      "user_id": "507f1f77bcf86cd799439012",
      "user_role": "STUDENT",
      "encrypted_key": "base64_encrypted_key_for_student"
    },
    {
      "user_id": "507f1f77bcf86cd799439013",
      "user_role": "PARENT",
      "encrypted_key": "base64_encrypted_key_for_parent"
    }
  ]
}
```

### Response

**Success (201 Created)**

```json
{
  "conversation": {
    "_id": "507f1f77bcf86cd799439014",
    "school_id": "507f1f77bcf86cd799439010",
    "participants": [
      {
        "id": "507f1f77bcf86cd799439011",
        "role": "TEACHER"
      },
      {
        "id": "507f1f77bcf86cd799439012",
        "role": "STUDENT"
      }
    ],
    "is_group": false,
    "name": null,
    "encryption_key_version": 1,
    "created_at": "2026-02-28T10:00:00Z",
    "updated_at": "2026-02-28T10:00:00Z"
  }
}
```

### Validation Rules

1. **Participants**:
   - Minimum 2 participants required
   - Maximum 50 participants for group conversations
   - All participant IDs must exist in the system
   - Authenticated user must be included in participants
   - No duplicate participants

2. **Group Conversations**:
   - If `is_group = true`, `name` is required
   - Group name must be 1-100 characters
   - If `is_group = false`, `name` must be null or omitted

3. **Encrypted Keys**:
   - Must provide exactly one encrypted key per participant
   - Each `user_id` in encrypted_keys must match a participant ID
   - Each `user_role` must match the corresponding participant role
   - Encrypted keys must be valid base64 strings

4. **School Context**:
   - If School-Token is provided, all participants must belong to that school
   - Conversation will be associated with the school

### Backend Implementation Notes

1. **Automatic Fields**:
   - Set `encryption_key_version` to 1 for new conversations
   - Set `created_at` and `updated_at` to current timestamp
   - Extract `school_id` from School-Token if provided

2. **Duplicate Prevention**:
   - Check if a conversation with the exact same participants already exists
   - For 1-on-1 conversations: return existing conversation if found
   - For group conversations: allow duplicates (different groups can have same members)

3. **Database Storage**:
   ```rust
   // Store in conversations collection
   struct Conversation {
       _id: ObjectId,
       school_id: Option<ObjectId>,
       participants: Vec<ActorRef>,
       is_group: bool,
       name: Option<String>,
       encryption_key_version: i32,
       created_at: DateTime,
       updated_at: DateTime,
   }
   
   // Store encrypted keys in separate collection
   struct ConversationKey {
       _id: ObjectId,
       conversation_id: ObjectId,
       user_id: String,
       user_role: String,
       encrypted_key: String,
       key_version: i32,
       created_at: DateTime,
   }
   ```

4. **Indexes**:
   - Create compound index on `participants.id` for efficient lookup
   - Create index on `conversation_id` in conversation_keys collection
   - Create compound index on `(conversation_id, user_id)` for key retrieval

---

## 2. Get User Public Keys

### Endpoint
```
GET /m/users/public-keys
```

Retrieves the public encryption keys for specified users. Used to encrypt the conversation symmetric key for each participant.

### Authentication
- **Required**: Yes
- **Headers**: 
  - `Authorization: Bearer {token}`
  - `School-Token: {school_token}` (optional)

### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_ids` | string | Yes | Comma-separated list of user IDs |

### Request Example

```bash
GET /m/users/public-keys?user_ids=507f1f77bcf86cd799439011,507f1f77bcf86cd799439012,507f1f77bcf86cd799439013
```

### Response

**Success (200 OK)**

```json
{
  "public_keys": [
    {
      "user_id": "507f1f77bcf86cd799439011",
      "public_key": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----",
      "key_algorithm": "RSA-2048",
      "created_at": "2026-01-15T08:30:00Z"
    },
    {
      "user_id": "507f1f77bcf86cd799439012",
      "public_key": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----",
      "key_algorithm": "RSA-2048",
      "created_at": "2026-01-20T14:22:00Z"
    }
  ]
}
```

### Response Fields

```typescript
interface GetPublicKeysResponse {
  public_keys: PublicKeyInfo[];
}

interface PublicKeyInfo {
  user_id: string;              // User ID
  public_key: string;           // PEM-formatted RSA public key
  key_algorithm: string;        // Algorithm (e.g., "RSA-2048")
  created_at: string;           // ISO 8601 timestamp
}
```

### Error Responses

**User Not Found (404)**
```json
{
  "message": "Public key not found for user: 507f1f77bcf86cd799439012",
  "missing_user_ids": ["507f1f77bcf86cd799439012"]
}
```

**Invalid Request (400)**
```json
{
  "message": "user_ids parameter is required"
}
```

**Too Many Users (400)**
```json
{
  "message": "Maximum 50 user IDs allowed per request"
}
```

### Validation Rules

1. **Query Parameters**:
   - `user_ids` is required
   - Must contain at least 1 user ID
   - Maximum 50 user IDs per request
   - User IDs must be valid ObjectId format

2. **Public Key Requirements**:
   - All requested users must have public keys registered
   - If any user is missing a public key, return 404 with list of missing users
   - Public keys must be in PEM format

### Backend Implementation Notes

1. **Public Key Storage**:
   ```rust
   struct UserPublicKey {
       _id: ObjectId,
       user_id: String,
       public_key: String,        // PEM format
       key_algorithm: String,     // "RSA-2048"
       created_at: DateTime,
       updated_at: DateTime,
   }
   ```

2. **Key Generation**:
   - Users should generate their RSA key pair on first login
   - Private key stays on client device (never sent to server)
   - Public key is uploaded to server via `POST /m/users/public-key`

3. **Security Considerations**:
   - Public keys are not sensitive, but access should still be authenticated
   - Only allow fetching public keys for users in the same school context
   - Rate limit this endpoint to prevent abuse

4. **Performance**:
   - Cache public keys in Redis with 1-hour TTL
   - Use batch query to fetch multiple keys efficiently
   - Return keys in the same order as requested user_ids

---

## 3. Encryption Flow

### Overview

The messaging system uses hybrid encryption:
1. **Symmetric encryption** (AES-256-GCM) for message content
2. **Asymmetric encryption** (RSA-2048) for distributing the symmetric key

### Step-by-Step Flow

#### Step 1: User Registration
```
Client generates RSA key pair
├─ Private key stored locally (IndexedDB/localStorage)
└─ Public key sent to server
    └─ POST /m/users/public-key
```

#### Step 2: Creating a Conversation
```
1. Client generates random AES-256 symmetric key
2. Client requests public keys for all participants
   └─ GET /m/users/public-keys?user_ids=...
3. Client encrypts symmetric key with each participant's public key
4. Client sends conversation creation request
   └─ POST /conversations
       ├─ participants: [ActorRef]
       └─ encrypted_keys: [EncryptedKeyForUser]
5. Server stores conversation and encrypted keys
6. Server returns conversation metadata
```

#### Step 3: Sending a Message
```
1. Client retrieves symmetric key from local storage
2. Client encrypts message content with AES-256-GCM
3. Client sends encrypted message
   └─ POST /conversations/{id}/messages
       ├─ encrypted_payload: base64(encrypted_content)
       ├─ nonce: base64(random_nonce)
       └─ key_version: 1
4. Server stores encrypted message (cannot read content)
5. Server broadcasts to all participants
```

#### Step 4: Receiving a Message
```
1. Client receives encrypted message via WebSocket
2. Client retrieves conversation symmetric key
   ├─ From local storage (if available)
   └─ Or GET /conversations/{id}/key (decrypt with private key)
3. Client decrypts message content with symmetric key
4. Client displays decrypted message
```

### Encryption Algorithms

**Symmetric Encryption (Message Content)**
- Algorithm: AES-256-GCM
- Key size: 256 bits (32 bytes)
- Nonce size: 96 bits (12 bytes)
- Tag size: 128 bits (16 bytes)

**Asymmetric Encryption (Key Distribution)**
- Algorithm: RSA-OAEP
- Key size: 2048 bits
- Hash: SHA-256
- Padding: OAEP with SHA-256

### Key Rotation

When a participant leaves or is removed:
```
1. Generate new symmetric key (version 2)
2. Encrypt new key for remaining participants
3. Update conversation.encryption_key_version
4. Store new encrypted keys
5. Future messages use new key version
```

---

## 4. Error Handling

### Common Error Responses

#### 400 Bad Request
```json
{
  "message": "Validation error",
  "errors": [
    {
      "field": "participants",
      "message": "Minimum 2 participants required"
    },
    {
      "field": "name",
      "message": "Group name is required for group conversations"
    }
  ]
}
```

#### 401 Unauthorized
```json
{
  "message": "Authentication required"
}
```

#### 403 Forbidden
```json
{
  "message": "You must be a participant in the conversation"
}
```

#### 404 Not Found
```json
{
  "message": "User not found: 507f1f77bcf86cd799439012"
}
```

#### 409 Conflict
```json
{
  "message": "Conversation already exists",
  "existing_conversation_id": "507f1f77bcf86cd799439014"
}
```

#### 422 Unprocessable Entity
```json
{
  "message": "Invalid encrypted key format",
  "details": "Encrypted key must be valid base64"
}
```

#### 500 Internal Server Error
```json
{
  "message": "Failed to create conversation",
  "error_id": "err_abc123"
}
```

### Error Handling Best Practices

1. **Validation Errors**: Return 400 with detailed field-level errors
2. **Authentication**: Return 401 for missing/invalid tokens
3. **Authorization**: Return 403 for permission issues
4. **Not Found**: Return 404 for missing resources
5. **Conflicts**: Return 409 for duplicate conversations
6. **Server Errors**: Return 500 and log error details

---

## 5. Additional Endpoints Needed

### Get Conversation Encryption Key

```
GET /conversations/{conversation_id}/key
```

Returns the encrypted symmetric key for the authenticated user.

**Response:**
```json
{
  "conversation_id": "507f1f77bcf86cd799439014",
  "encrypted_key": "base64_encrypted_key",
  "key_version": 1
}
```

### Upload User Public Key

```
POST /m/users/public-key
```

Uploads or updates the user's public encryption key.

**Request:**
```json
{
  "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
  "key_algorithm": "RSA-2048"
}
```

**Response:**
```json
{
  "message": "Public key uploaded successfully",
  "user_id": "507f1f77bcf86cd799439011"
}
```

---

## 6. Testing Checklist

### Create Conversation Tests

- [ ] Create 1-on-1 conversation successfully
- [ ] Create group conversation with name
- [ ] Reject group conversation without name
- [ ] Reject conversation with < 2 participants
- [ ] Reject conversation with > 50 participants
- [ ] Reject duplicate participants
- [ ] Reject if authenticated user not in participants
- [ ] Reject if participant doesn't exist
- [ ] Reject if encrypted_keys count doesn't match participants
- [ ] Reject if encrypted_keys user_id doesn't match participant
- [ ] Return existing conversation for duplicate 1-on-1
- [ ] Allow duplicate group conversations
- [ ] Associate conversation with school when School-Token provided
- [ ] Reject if participants not in same school

### Get Public Keys Tests

- [ ] Fetch single user's public key
- [ ] Fetch multiple users' public keys
- [ ] Return 404 if any user missing public key
- [ ] Reject if user_ids parameter missing
- [ ] Reject if > 50 user IDs requested
- [ ] Reject if user_ids contains invalid ObjectId
- [ ] Return keys in same order as requested
- [ ] Only allow fetching keys for users in same school

### Integration Tests

- [ ] Complete flow: register keys → create conversation → send message
- [ ] Multiple participants can decrypt same message
- [ ] Key rotation works correctly
- [ ] WebSocket broadcasts conversation creation
- [ ] Conversation appears in all participants' lists

---

## 7. Database Schema

### conversations Collection

```javascript
{
  _id: ObjectId,
  school_id: ObjectId | null,
  participants: [
    {
      id: String,
      role: String  // "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT"
    }
  ],
  is_group: Boolean,
  name: String | null,
  encryption_key_version: Number,
  created_at: ISODate,
  updated_at: ISODate
}

// Indexes
db.conversations.createIndex({ "participants.id": 1 })
db.conversations.createIndex({ school_id: 1 })
db.conversations.createIndex({ created_at: -1 })
```

### conversation_keys Collection

```javascript
{
  _id: ObjectId,
  conversation_id: ObjectId,
  user_id: String,
  user_role: String,
  encrypted_key: String,  // Base64
  key_version: Number,
  created_at: ISODate
}

// Indexes
db.conversation_keys.createIndex({ conversation_id: 1, user_id: 1 }, { unique: true })
db.conversation_keys.createIndex({ user_id: 1 })
```

### user_public_keys Collection

```javascript
{
  _id: ObjectId,
  user_id: String,
  public_key: String,  // PEM format
  key_algorithm: String,  // "RSA-2048"
  created_at: ISODate,
  updated_at: ISODate
}

// Indexes
db.user_public_keys.createIndex({ user_id: 1 }, { unique: true })
```

---

## 8. Security Considerations

1. **End-to-End Encryption**:
   - Server never has access to message content
   - Server only stores encrypted payloads
   - Symmetric keys are encrypted per-user

2. **Key Management**:
   - Private keys never leave client device
   - Public keys are validated before storage
   - Key rotation on participant changes

3. **Access Control**:
   - Only participants can access conversation
   - Only participants can retrieve encrypted keys
   - School context enforced when applicable

4. **Rate Limiting**:
   - Limit conversation creation (e.g., 10 per hour per user)
   - Limit public key requests (e.g., 100 per minute)
   - Prevent spam and abuse

5. **Audit Logging**:
   - Log conversation creation
   - Log key access attempts
   - Log failed authentication attempts

---

## 9. Performance Optimization

1. **Caching**:
   - Cache public keys in Redis (1-hour TTL)
   - Cache conversation metadata (5-minute TTL)
   - Invalidate cache on updates

2. **Database Queries**:
   - Use compound indexes for participant lookups
   - Batch fetch encrypted keys
   - Paginate conversation lists

3. **WebSocket**:
   - Use rooms for conversation-specific broadcasts
   - Compress large payloads
   - Implement reconnection logic

---

## Questions?

For any questions or clarifications about this API specification, please contact the frontend team or refer to the existing messaging documentation:
- `backend/MESSAGING_API_BREAKING_CHANGES.md`
- `docs/MESSAGING_IMPLEMENTATION.md`
- `backend/SCHOOL_MEMBER_SEARCH_API.md`
