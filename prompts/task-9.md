# Secure End-to-End Encrypted Messaging 

Before writing any code:

1. Read:

   * `MESSAGING_IMPLEMENTATION.md` to understand frontend expectations 
   * `common_details.rs` (for `RelatedUser`)
2. Follow architecture:

   * `student.rs`
   * `student_service.rs`
   * `student_pipeline.rs`
   * `students_api.rs`
3. Use `/m` route prefix.
4. Do NOT use `/api/v1`.
5. Every school uses its own DB.
6. Sender must be stored as `RelatedUser`.
7. Implement End-to-End Encryption (E2EE).
8. Add strong anti-hacking protections.

---

# 🎯 GOAL

Implement:

* Conversations
* Messages
* Participants
* File attachments
* WebSocket real-time
* End-to-End encryption
* Role-aware messaging
* Security hardened

---

# 1️⃣ DOMAIN STRUCTURE

Create:

```
conversation.rs
conversation_service.rs
conversation_pipeline.rs
conversations_api.rs

message.rs
message_service.rs
message_pipeline.rs
messages_api.rs

messaging_socket.rs
```

---

# 2️⃣ DATA MODELS

## Conversation

```rust
pub struct Conversation {
    pub _id: ObjectId,
    pub school_id: ObjectId,

    pub participants: Vec<RelatedUser>,
    pub is_group: bool,
    pub name: Option<String>,

    pub encryption_key_version: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## Message

```rust

pub struct MessageSender {
 pub sender_role: UserRole,// This roles are when you getting data only user role you will get those data in common database
     #[serde(
            serialize_with = "object_id_helpers::serialize_oid",
            deserialize_with = "object_id_helpers::deserialize_oid",
            default
        )]
    pub sender_id: ObjectId, 
}

pub struct Message {
    pub _id: ObjectId,
    pub school_id: ObjectId,
    pub conversation_id: ObjectId,

    pub sender: MessageSender,

    pub encrypted_payload: String,  // ciphertext
    pub nonce: String,              // encryption nonce
    pub key_version: i32,

    pub message_type: String, // TEXT | FILE | SYSTEM

    pub file_url: Option<String>,
    pub file_public_id: Option<String>,

    pub read_by: Vec<RelatedUser>,

    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
```

⚠️ DO NOT store plaintext content in DB.

---

# 3️⃣ END-TO-END ENCRYPTION MODEL

### Design:

* Encryption is done client-side.
* Backend stores only ciphertext.
* Backend NEVER sees plaintext.
* Use AES-256-GCM.
* Each conversation has symmetric key.
* Each user receives encrypted conversation key using their public key.

Create new model:

## ConversationKey

```rust
pub struct ConversationKey {
    pub _id: ObjectId,
    pub conversation_id: ObjectId,
    pub user: MessageSender,
    pub encrypted_key_for_user: String,
}
```

Flow:

1. Client generates conversation symmetric key.
2. Client encrypts key using each participant’s public key.
3. Backend stores encrypted key per user.
4. Messages are encrypted using symmetric key before sending.

Server validates structure only — not content.

---

# 4️⃣ SECURITY HARDENING (CRITICAL)

Implement:

### 1. Strict Participant Validation

Before sending message:

* Verify sender is in conversation participants.
* Reject otherwise.

### 2. Rate Limiting

Add:

* 30 messages per minute per user
* 10 conversation creations per hour

### 3. Payload Size Limit

* Max encrypted payload 100KB
* Max file upload 50MB

### 4. HTML Sanitization

Even encrypted:

* If fallback REST plaintext used, sanitize.

### 5. Replay Attack Protection

Require:

```rust
client_message_id: String
```

Store and reject duplicate client_message_id.

### 6. WebSocket Authentication

* Require JWT during connection
* Validate school_id from token
* Reject cross-school access

### 7. Indexing

Indexes:

```
conversation_id
school_id
created_at
participants.user.id
```

---

# 5️⃣ REST ENDPOINTS

Based on frontend doc :

```
GET  /m/conversations
POST /m/conversations
GET  /m/conversations/{id}
GET  /m/conversations/{id}/messages
POST /m/conversations/{id}/messages
GET  /m/conversations/{id}/files
```

---

# 6️⃣ WEBSOCKET IMPLEMENTATION

Namespace:

```
/m
```

On connect:

* Validate JWT
* Join room = conversation_id
* Ensure user belongs to conversation

Emit events:

```
message.created
message.read
conversation.created
conversation.participant_added
```

Payload example must match frontend spec .

Ensure:

* Ordered delivery
* Idempotent message broadcast
* No cross-tenant leakage

---

# 7️⃣ FILE ATTACHMENTS

Reuse:

`cloudinary_service.rs`

For file messages:

* Upload file
* Store only URL + public_id
* File access must validate:

  * requester is participant
  * school matches

Optional:

* Signed URLs (short expiry)

---

# 8️⃣ ROLE RULES

Participants can be:

* Student
* Teacher
* Staff
* Parent
* Admin

Use `RelatedUser` for sender and participant.

Validate:

* Student ↔ Teacher allowed
* Parent ↔ Teacher allowed
* Cross-school blocked
* Super-admin only allowed if special flag

---

# 9️⃣ SOFT DELETE

Message delete:

* Set deleted_at
* Emit message.deleted event
* Do NOT physically remove immediately

---

# 🔐 FINAL BACKEND RESULT

Space-Together messaging will be:

* End-to-End encrypted
* Tenant isolated
* Replay protected
* Rate limited
* JWT secured
* WebSocket secured
* File protected
* Role validated
* Audit logged

Institution-grade secure messaging.

---

---

# ✅ FRONTEND INTEGRATION PROMPT (Security Alignment)

Since UI already exists, instruct frontend IDE:

1. Implement client-side encryption:

   * Generate symmetric key per conversation.
   * Use Web Crypto API AES-GCM.
2. Store encrypted conversation key in backend.
3. Before sending:

   * Encrypt message content.
   * Send ciphertext + nonce.
4. Decrypt messages after fetch.
5. Store private keys securely (IndexedDB).
6. Rotate keys on participant change.
7. Do NOT store plaintext in localStorage.

Also:

* Validate JWT before WebSocket connect.
* Auto reconnect securely.
* Handle `message.read` event.

---

# 🚀 What You Now Have

Space-Together messaging will be:

* Fully encrypted
* Multi-tenant safe
* School-isolated
* Real-time
* Replay-safe
* Ministry-grade secure
* Hard to exploit

