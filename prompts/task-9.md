# Secure Messaging System 

Before writing code:

1. Read:

   * `student.rs`
   * `student_service.rs`
   * `student_pipeline.rs`
   * `students_api.rs`
2. Follow same architecture pattern.
3. Respect `/m` prefix exactly (per frontend contract ).
4. Enforce strict school isolation (`school_id` always validated).
5. Integrate:

   * role_guard
   * permission system
   * audit logs
6. Use WebSocket with authentication.
7. Apply enterprise-grade security controls.
8. Prevent common attack vectors.

---

# 🎯 GOAL

Implement:

* Conversations
* Messages
* Attachments
* Realtime messaging
* Read receipts
* Mentions
* Strict security enforcement

---

# 1️⃣ FILE STRUCTURE

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

message_socket.rs
```

Follow same architecture as students module.

---

# 2️⃣ DATABASE SCHEMAS

## Conversation

```rust
pub struct Conversation {
    pub _id: ObjectId,
    pub school_id: ObjectId,
    pub participant_ids: Vec<ObjectId>,
    pub is_group: bool,
    pub title: Option<String>,

    pub last_message_at: Option<DateTime<Utc>>,
    pub created_by: ObjectId,
    pub created_at: DateTime<Utc>,
}
```

Indexes:

* school_id
* participant_ids
* last_message_at

---

## Message

```rust
pub struct Message {
    pub _id: ObjectId,
    pub school_id: ObjectId,
    pub conversation_id: ObjectId,
    pub sender_id: ObjectId,

    pub content: String,
    pub sanitized_content: String,

    pub attachment_url: Option<String>,
    pub attachment_public_id: Option<String>,

    pub mentions: Vec<ObjectId>,
    pub is_edited: bool,
    pub read_by: Vec<ObjectId>,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

Indexes:

* conversation_id
* school_id
* created_at

---

# 3️⃣ CRITICAL SECURITY REQUIREMENTS

## 🔒 1. Strict Authorization

Before ANY action:

* Validate user belongs to conversation.
* Validate user belongs to same school.
* Reject cross-tenant access.
* Reject if user suspended.

Never trust frontend conversationId.

---

## 🔒 2. XSS Protection

Messages arrive as HTML (frontend returns HTML ).

You MUST:

* Sanitize HTML server-side.
* Remove:

  * <script>

  * inline JS
  * event handlers
  * iframe
  * style injection
* Use whitelist policy (only allow: p, strong, em, ul, li, br).

Store:

* original raw content
* sanitized version

Always return sanitized_content.

---

## 🔒 3. Rate Limiting

Implement per-user rate limiter:

* Max 30 messages per minute.
* Use in-memory or Redis.
* Prevent spam and brute-force flood.

Reject with 429.

---

## 🔒 4. WebSocket Authentication

WebSocket must:

* Require JWT token on connection.
* Validate token.
* Extract user_id and school_id.
* Refuse connection if invalid.

On join:

User can subscribe only to conversations where:

* user_id ∈ participant_ids
* school_id matches

---

## 🔒 5. File Upload Security

Extend cloudinary_service:

* Only allow:

  * pdf
  * docx
  * xlsx
  * png
  * jpg
  * mp4
* Max 25MB for messaging files.
* Scan MIME type.
* Reject executable formats.
* Store files under:
  space-together/{school_id}/messages/{conversation_id}
* Use cloudinary to store those files but use exit services

---

## 🔒 6. Prevent ID Enumeration

All endpoints must:

* Validate ObjectId format
* Return generic error on invalid access
* Never expose whether conversation exists if user unauthorized

---

## 🔒 7. Encryption

* Enable TLS enforcement (assume reverse proxy)
* Hash sensitive logs
* Do NOT log message content in plaintext logs

Optional advanced:

* Encrypt message content at rest using AES-256
* Store encryption key per school (future-ready)

---

# 4️⃣ REST ENDPOINTS

Must match frontend contract 

```
GET    /message/conversations
POST   /message/conversations
GET    /message/conversations/{id}
GET    /message/conversations/{id}/messages
POST   /message/conversations/{id}/messages
GET    /message/conversations/{id}/files
```

No `/api/v1`.

---

# 5️⃣ WEBSOCKET

Namespace: `/m`

Events:

Server emits:

* message.created
* message.updated
* message.read
* conversation.created

Client sends:

* message.send
* message.read
* conversation.join

On new message:

1. Save to DB
2. Broadcast to conversation room
3. Update last_message_at

---

# 6️⃣ READ RECEIPTS

When user opens conversation:

* Mark messages as read
* Add user_id to read_by
* Emit message.read event

---

# 7️⃣ AUDIT LOG EVENTS

Log:

* conversation.created
* message.sent
* message.deleted
* participant.added
* participant.removed

Do NOT log content.

---

# 8️⃣ SOFT DELETE SUPPORT

Allow:

* Message delete (mark deleted flag)
* Replace content with:
  "Message removed"

---

# 🎓 EXPECTED BACKEND RESULT

Secure messaging system with:

* School isolation
* XSS protection
* Rate limiting
* WebSocket auth
* File validation
* Audit logging
* ID enumeration protection
* Sanitized HTML storage
* Read receipts
* Scalable architecture

This prevents:

* Cross-tenant data leak
* Script injection
* Spam flood
* Unauthorized subscription
* File-based malware upload

---

---

# ✅ OPTIONAL HARDENING (Tell IDE to Implement)

Add:

* CSP headers
* Helmet-like middleware
* IP throttling
* Account lockout on abuse
* Suspicious activity logging

