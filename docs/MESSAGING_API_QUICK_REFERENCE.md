# Messaging API Quick Reference

## Base URL
All messaging endpoints are under `/m` prefix.

## Authentication
All endpoints require JWT authentication via `Authorization: Bearer {token}` header.

## School Context (Optional)
Add `School-Token: {school_token}` header for school-specific conversations.

---

## Public Key Management

### Upload Public Key
```http
POST /m/users/public-key
Content-Type: application/json

{
  "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
  "key_algorithm": "RSA-2048"
}
```

**Response 200:**
```json
{
  "message": "Public key uploaded successfully",
  "user_id": "507f1f77bcf86cd799439011"
}
```

### Get Public Keys
```http
GET /m/users/public-keys?user_ids=id1,id2,id3
```

**Response 200:**
```json
{
  "public_keys": [
    {
      "user_id": "507f1f77bcf86cd799439011",
      "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
      "key_algorithm": "RSA-2048",
      "created_at": "2026-02-28T10:00:00Z"
    }
  ]
}
```

**Response 404:**
```json
{
  "message": "Public key not found for users: id2",
  "missing_user_ids": ["id2"]
}
```

---

## Conversations

### Create Conversation
```http
POST /conversations
Content-Type: application/json

{
  "participants": [
    { "id": "user_id_1", "role": "TEACHER" },
    { "id": "user_id_2", "role": "STUDENT" }
  ],
  "is_group": false,
  "name": null,
  "encrypted_keys": [
    {
      "user_id": "user_id_1",
      "user_role": "TEACHER",
      "encrypted_key": "base64_encrypted_key"
    },
    {
      "user_id": "user_id_2",
      "user_role": "STUDENT",
      "encrypted_key": "base64_encrypted_key"
    }
  ]
}
```

**Response 201:**
```json
{
  "conversation": {
    "_id": "conversation_id",
    "school_id": "school_id",
    "participants": [...],
    "is_group": false,
    "name": null,
    "encryption_key_version": 1,
    "created_at": "2026-02-28T10:00:00Z",
    "updated_at": "2026-02-28T10:00:00Z"
  }
}
```

**Response 409 (Duplicate):**
```json
{
  "message": "Conversation already exists",
  "existing_conversation_id": "existing_id"
}
```

### List Conversations
```http
GET /conversations?page=1&limit=20
```

**Response 200:**
```json
{
  "data": [...],
  "total": 100,
  "total_pages": 5,
  "current_page": 1
}
```

### Get Conversation
```http
GET /conversations/{id}
```

**Response 200:**
```json
{
  "_id": "conversation_id",
  "school_id": "school_id",
  "participants": [...],
  "participants_users": [...],
  "is_group": false,
  "name": null,
  "encryption_key_version": 1,
  "created_at": "2026-02-28T10:00:00Z",
  "updated_at": "2026-02-28T10:00:00Z"
}
```

### Get Conversation Key
```http
GET /conversations/{id}/key
```

**Response 200:**
```json
{
  "id": "key_id",
  "conversation_id": "conversation_id",
  "user_id": "user_id",
  "user_role": "TEACHER",
  "encrypted_key_for_user": "base64_encrypted_key",
  "created_at": "2026-02-28T10:00:00Z"
}
```

---

## Messages

### Send Message
```http
POST /conversations/{id}/messages
Content-Type: application/json

{
  "encrypted_payload": "base64_encrypted_content",
  "nonce": "base64_nonce",
  "key_version": 1,
  "message_type": "TEXT",
  "client_message_id": "unique_client_id"
}
```

**Response 201:**
```json
{
  "_id": "message_id",
  "conversation_id": "conversation_id",
  "sender": {
    "id": "user_id",
    "role": "TEACHER"
  },
  "encrypted_payload": "base64_encrypted_content",
  "nonce": "base64_nonce",
  "key_version": 1,
  "message_type": "TEXT",
  "created_at": "2026-02-28T10:00:00Z"
}
```

### Get Messages
```http
GET /conversations/{id}/messages?page=1&limit=50
```

**Response 200:**
```json
{
  "data": [
    {
      "_id": "message_id",
      "encrypted_payload": "base64_encrypted_content",
      "nonce": "base64_nonce",
      "sender": {...},
      "created_at": "2026-02-28T10:00:00Z"
    }
  ],
  "total": 100,
  "total_pages": 2,
  "current_page": 1
}
```

### Get Files
```http
GET /conversations/{id}/files?page=1&limit=20
```

### Delete Message
```http
DELETE /conversations/{id}/messages/{message_id}
```

---

## Validation Rules

### Public Keys
- PEM format required
- Max 50 user IDs per request
- All users must have keys

### Conversations
- Min 2 participants
- Max 50 participants
- Group name required for groups (1-100 chars)
- No name for direct conversations
- No duplicate participants
- Auth user must be participant
- One encrypted key per participant
- Keys must match participants
- Base64 validation

### Messages
- Must be participant
- Encrypted payload required
- Nonce required (12 bytes for GCM)
- Client message ID required

---

## Error Codes

| Code | Meaning |
|------|---------|
| 400 | Bad Request - Validation error |
| 401 | Unauthorized - Auth required |
| 403 | Forbidden - Not a participant |
| 404 | Not Found - Resource missing |
| 409 | Conflict - Duplicate conversation |
| 422 | Unprocessable - Invalid format |
| 500 | Internal Server Error |

---

## Encryption Algorithms

**Asymmetric (Key Distribution)**
- Algorithm: RSA-OAEP
- Key Size: 2048 bits
- Hash: SHA-256

**Symmetric (Message Content)**
- Algorithm: AES-256-GCM
- Key Size: 256 bits (32 bytes)
- Nonce Size: 96 bits (12 bytes)
- Tag Size: 128 bits (16 bytes)

---

## Example Flow

1. **Setup**
   ```
   Generate RSA key pair → Upload public key
   ```

2. **Create Conversation**
   ```
   Generate AES key → Fetch public keys → Encrypt AES key for each user → Create conversation
   ```

3. **Send Message**
   ```
   Encrypt with AES-GCM → Send encrypted payload + nonce
   ```

4. **Receive Message**
   ```
   Get encrypted message → Decrypt with AES key → Display
   ```

---

## Rate Limits (Recommended)

- Public key upload: 10/hour per user
- Public key fetch: 100/minute per user
- Conversation creation: 10/hour per user
- Message send: 100/minute per user

---

## Security Notes

1. Private keys never sent to server
2. Server cannot decrypt messages
3. Only participants can access conversations
4. Base64 validation on all encrypted data
5. PEM format validation on public keys
6. Access control on all endpoints
