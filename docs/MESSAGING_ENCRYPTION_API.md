# Messaging Encryption API Implementation

## Overview
This document describes the implemented end-to-end encrypted messaging system with public key infrastructure.

## Implemented Features

### 1. User Public Key Management

#### Upload/Update Public Key
```http
POST /m/users/public-key
Authorization: Bearer {token}
Content-Type: application/json

{
  "public_key": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----",
  "key_algorithm": "RSA-2048"
}
```

**Response (200 OK):**
```json
{
  "message": "Public key uploaded successfully",
  "user_id": "507f1f77bcf86cd799439011"
}
```

**Features:**
- Upserts (creates or updates) the public key for the authenticated user
- Validates PEM format
- Stores in main database (not school-specific)
- Unique per user_id

#### Get Public Keys for Multiple Users
```http
GET /m/users/public-keys?user_ids=507f1f77bcf86cd799439011,507f1f77bcf86cd799439012
Authorization: Bearer {token}
```

**Response (200 OK):**
```json
{
  "public_keys": [
    {
      "user_id": "507f1f77bcf86cd799439011",
      "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
      "key_algorithm": "RSA-2048",
      "created_at": "2026-02-28T10:00:00Z"
    },
    {
      "user_id": "507f1f77bcf86cd799439012",
      "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
      "key_algorithm": "RSA-2048",
      "created_at": "2026-02-28T09:30:00Z"
    }
  ]
}
```

**Error Response (404 Not Found):**
```json
{
  "message": "Public key not found for users: 507f1f77bcf86cd799439012",
  "missing_user_ids": ["507f1f77bcf86cd799439012"]
}
```

**Validation:**
- Maximum 50 user IDs per request
- All requested users must have public keys
- Returns 404 with missing user IDs if any are not found

### 2. Conversation Creation with Encryption

#### Create Conversation
```http
POST /conversations
Authorization: Bearer {token}
School-Token: {school_token} (optional)
Content-Type: application/json

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
      "encrypted_key": "base64_encrypted_symmetric_key_for_teacher"
    },
    {
      "user_id": "507f1f77bcf86cd799439012",
      "user_role": "STUDENT",
      "encrypted_key": "base64_encrypted_symmetric_key_for_student"
    }
  ]
}
```

**Response (201 Created):**
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

**Validation Rules:**
1. Minimum 2 participants required
2. Maximum 50 participants allowed
3. Group conversations require a name (1-100 characters)
4. Direct conversations must not have a name
5. No duplicate participants
6. Authenticated user must be in participants
7. Must provide exactly one encrypted key per participant
8. Each encrypted key must match a participant (user_id and role)
9. Encrypted keys must be valid base64
10. Duplicate 1-on-1 conversations return existing conversation (409 Conflict)

#### Get Conversation Encryption Key
```http
GET /conversations/{conversation_id}/key
Authorization: Bearer {token}
```

**Response (200 OK):**
```json
{
  "id": "507f1f77bcf86cd799439015",
  "conversation_id": "507f1f77bcf86cd799439014",
  "user_id": "507f1f77bcf86cd799439011",
  "user_role": "TEACHER",
  "encrypted_key_for_user": "base64_encrypted_symmetric_key",
  "created_at": "2026-02-28T10:00:00Z"
}
```

**Error Response (403 Forbidden):**
```json
{
  "message": "You are not a participant in this conversation"
}
```

## Database Schema

### user_public_keys Collection
```javascript
{
  _id: ObjectId,
  user_id: ObjectId,           // Unique index
  public_key: String,          // PEM format
  key_algorithm: String,       // "RSA-2048"
  created_at: ISODate,
  updated_at: ISODate
}
```

### conversations Collection
```javascript
{
  _id: ObjectId,
  school_id: ObjectId | null,
  participants: [
    {
      id: ObjectId,
      role: String  // "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT"
    }
  ],
  is_group: Boolean,
  name: String | null,
  encryption_key_version: Number,
  created_at: ISODate,
  updated_at: ISODate
}
```

### conversation_keys Collection
```javascript
{
  _id: ObjectId,
  conversation_id: ObjectId,
  user_id: ObjectId,
  user_role: String,
  encrypted_key_for_user: String,  // Base64 encrypted symmetric key
  created_at: ISODate
}

// Compound unique index on (conversation_id, user_id)
```

## Encryption Flow

### 1. User Registration
```
Client generates RSA-2048 key pair
├─ Private key stored locally (never sent to server)
└─ Public key uploaded to server
    └─ POST /m/users/public-key
```

### 2. Creating a Conversation
```
1. Client generates random AES-256-GCM symmetric key
2. Client fetches public keys for all participants
   └─ GET /m/users/public-keys?user_ids=...
3. Client encrypts symmetric key with each participant's public key (RSA-OAEP)
4. Client sends conversation creation request
   └─ POST /conversations
       ├─ participants: [ActorRef]
       └─ encrypted_keys: [EncryptedKeyForUser]
5. Server stores conversation and encrypted keys
6. Server returns conversation metadata
```

### 3. Sending a Message
```
1. Client retrieves symmetric key from local storage
2. Client encrypts message with AES-256-GCM
3. Client sends encrypted message
   └─ POST /conversations/{id}/messages
       ├─ encrypted_payload: base64(encrypted_content)
       ├─ nonce: base64(random_nonce)
       └─ key_version: 1
4. Server stores encrypted message (cannot read content)
```

### 4. Receiving a Message
```
1. Client receives encrypted message
2. Client retrieves conversation symmetric key
   ├─ From local storage (if available)
   └─ Or GET /conversations/{id}/key (decrypt with private key)
3. Client decrypts message with symmetric key
4. Client displays decrypted message
```

## Security Features

1. **End-to-End Encryption**: Server never has access to message content
2. **Public Key Infrastructure**: RSA-2048 for key distribution
3. **Symmetric Encryption**: AES-256-GCM for message content
4. **Key Versioning**: Support for key rotation
5. **Access Control**: Only participants can access conversations and keys
6. **Validation**: Comprehensive input validation and error handling

## API Endpoints Summary

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/m/users/public-key` | Upload/update user's public key |
| GET | `/m/users/public-keys` | Get public keys for multiple users |
| POST | `/conversations` | Create encrypted conversation |
| GET | `/conversations` | List user's conversations |
| GET | `/conversations/{id}` | Get conversation details |
| GET | `/conversations/{id}/key` | Get encrypted symmetric key |
| POST | `/conversations/{id}/messages` | Send encrypted message |
| GET | `/conversations/{id}/messages` | Get conversation messages |

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Validation error |
| 401 | Unauthorized - Authentication required |
| 403 | Forbidden - Not a participant |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Duplicate conversation |
| 422 | Unprocessable Entity - Invalid format |
| 500 | Internal Server Error |

## Testing Checklist

- [x] Upload public key (create)
- [x] Upload public key (update existing)
- [x] Get single user's public key
- [x] Get multiple users' public keys
- [x] Return 404 for missing public keys
- [x] Validate PEM format
- [x] Create 1-on-1 conversation
- [x] Create group conversation
- [x] Validate group name requirement
- [x] Validate participant count
- [x] Validate encrypted keys match participants
- [x] Validate base64 format
- [x] Prevent duplicate participants
- [x] Return existing 1-on-1 conversation
- [x] Get conversation encryption key
- [x] Verify participant access control

## Implementation Files

- `src/domain/user_public_key.rs` - Public key domain model
- `src/services/user_public_key_service.rs` - Public key service
- `src/api/messaging_users_api.rs` - Public key API endpoints
- `src/api/conversations_api.rs` - Conversation API (updated with validation)
- `src/services/conversation_service.rs` - Conversation service
- `src/domain/conversation.rs` - Conversation domain models

## Next Steps

1. Implement key rotation functionality
2. Add rate limiting for public key requests
3. Add audit logging for key access
4. Implement conversation participant management (add/remove)
5. Add WebSocket notifications for new conversations
6. Implement message read receipts
7. Add file encryption support
