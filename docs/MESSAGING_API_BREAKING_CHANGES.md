# Messaging API Breaking Changes - Frontend Migration Guide

## Overview

The messaging system (conversations and messages) has been refactored to use a simpler, more consistent data structure. This document explains the changes and how to update your frontend code.

## What Changed?

### 1. Conversation Participants Structure

**BEFORE:**
```typescript
// Old structure - participants were full user objects
interface Conversation {
  _id: string;
  school_id?: string;
  participants: RelatedUser[];  // Full user objects with all fields
  is_group: boolean;
  name?: string;
  encryption_key_version: number;
  created_at: string;
  updated_at: string;
}

type RelatedUser = 
  | { user_type: "STUDENT"; /* all student fields */ }
  | { user_type: "TEACHER"; /* all teacher fields */ }
  | { user_type: "SCHOOLSTAFF"; /* all staff fields */ }
  | { user_type: "PARENT"; /* all parent fields */ };
```

**AFTER:**
```typescript
// New structure - participants are simple references
interface Conversation {
  _id: string;
  school_id?: string;
  participants: ActorRef[];  // Simple id + role references
  is_group: boolean;
  name?: string;
  encryption_key_version: number;
  created_at: string;
  updated_at: string;
}

interface ActorRef {
  id: string;
  role: "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT" | "ADMIN";
}
```

**WITH RELATIONS (when fetching):**
```typescript
// When you GET conversations, you receive populated data
interface ConversationWithRelations {
  _id: string;
  school_id?: string;
  participants: ActorRef[];  // Still simple references
  participants_users: RelatedUser[];  // Populated user data
  is_group: boolean;
  name?: string;
  encryption_key_version: number;
  created_at: string;
  updated_at: string;
}
```

### 2. Message Sender Structure

**BEFORE:**
```typescript
// Old structure - separate sender fields
interface Message {
  _id: string;
  school_id?: string;
  conversation_id: string;
  sender: {
    sender_role: "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT";
    sender_id: string;
  };
  encrypted_payload: string;
  nonce: string;
  // ... other fields
}
```

**AFTER:**
```typescript
// New structure - consistent ActorRef
interface Message {
  _id: string;
  school_id?: string;
  conversation_id: string;
  sender: ActorRef;  // Simple id + role reference
  encrypted_payload: string;
  nonce: string;
  // ... other fields
}

interface ActorRef {
  id: string;
  role: "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT" | "ADMIN";
}
```

**WITH RELATIONS (when fetching):**
```typescript
// When you GET messages, you receive populated data
interface MessageWithRelations {
  _id: string;
  school_id?: string;
  conversation_id: string;
  sender: ActorRef;  // Still simple reference
  sender_user?: RelatedUser;  // Populated user data
  encrypted_payload: string;
  nonce: string;
  // ... other fields
}
```

## API Endpoint Changes

### No endpoint URLs changed - only request/response formats

All endpoints remain the same:
- `POST /conversations`
- `GET /conversations`
- `GET /conversations/{id}`
- `GET /conversations/{id}/key`
- `POST /conversations/{conversation_id}/messages`
- `GET /conversations/{conversation_id}/messages`
- `GET /conversations/{conversation_id}/files`
- `DELETE /conversations/{conversation_id}/messages/{message_id}`

## Frontend Migration Steps

### Step 1: Update TypeScript Types

```typescript
// Add new types
interface ActorRef {
  id: string;
  role: "STUDENT" | "TEACHER" | "SCHOOLSTAFF" | "PARENT" | "ADMIN";
}

// Update Conversation type
interface Conversation {
  _id: string;
  school_id?: string;
  participants: ActorRef[];  // Changed from RelatedUser[]
  is_group: boolean;
  name?: string;
  encryption_key_version: number;
  created_at: string;
  updated_at: string;
}

// Add new type for API responses
interface ConversationWithRelations extends Conversation {
  participants_users: RelatedUser[];  // Populated user data
}

// Update Message type
interface Message {
  _id: string;
  school_id?: string;
  conversation_id: string;
  sender: ActorRef;  // Changed from { sender_role, sender_id }
  encrypted_payload: string;
  nonce: string;
  key_version: number;
  message_type: "TEXT" | "FILE" | "SYSTEM";
  file_url?: string;
  file_public_id?: string;
  read_by: RelatedUser[];
  client_message_id: string;
  deleted_at?: string;
  created_at: string;
}

// Add new type for API responses
interface MessageWithRelations extends Message {
  sender_user?: RelatedUser;  // Populated user data
}
```

### Step 2: Update Create Conversation Request

**BEFORE:**
```typescript
// Old way - sending full user objects
const createConversation = async (participants: RelatedUser[]) => {
  const response = await fetch('/conversations', {
    method: 'POST',
    body: JSON.stringify({
      participants: participants,  // Full user objects
      is_group: participants.length > 2,
      encrypted_keys: [/* ... */]
    })
  });
};
```

**AFTER:**
```typescript
// New way - sending simple references
const createConversation = async (userIds: string[], userRoles: string[]) => {
  const participants: ActorRef[] = userIds.map((id, index) => ({
    id: id,
    role: userRoles[index]
  }));

  const response = await fetch('/conversations', {
    method: 'POST',
    body: JSON.stringify({
      participants: participants,  // Simple references
      is_group: participants.length > 2,
      encrypted_keys: [/* ... */]
    })
  });
};
```

### Step 3: Update Conversation Display Logic

**BEFORE:**
```typescript
// Old way - accessing user data directly from participants
const ConversationItem = ({ conversation }: { conversation: Conversation }) => {
  const otherParticipant = conversation.participants.find(
    p => p.user_type === "STUDENT" && p.id !== currentUserId
  );
  
  return (
    <div>
      <h3>{otherParticipant?.first_name} {otherParticipant?.last_name}</h3>
      <img src={otherParticipant?.profile_picture} />
    </div>
  );
};
```

**AFTER:**
```typescript
// New way - accessing user data from participants_users
const ConversationItem = ({ conversation }: { conversation: ConversationWithRelations }) => {
  // Find the other participant's ID from participants array
  const otherParticipantRef = conversation.participants.find(
    p => p.id !== currentUserId
  );
  
  // Get the full user data from participants_users
  const otherParticipant = conversation.participants_users.find(
    user => {
      const userId = user.user_type === "STUDENT" ? user.id :
                     user.user_type === "TEACHER" ? user.id :
                     user.user_type === "SCHOOLSTAFF" ? user.id :
                     user.user_type === "PARENT" ? user.id : null;
      return userId === otherParticipantRef?.id;
    }
  );
  
  return (
    <div>
      <h3>
        {otherParticipant?.user_type === "STUDENT" && otherParticipant.first_name}
        {otherParticipant?.user_type === "TEACHER" && otherParticipant.first_name}
        {/* ... handle other types */}
      </h3>
    </div>
  );
};
```

**BETTER APPROACH - Create a helper function:**
```typescript
// Helper to extract user data
const getUserFromRelatedUser = (relatedUser: RelatedUser) => {
  switch (relatedUser.user_type) {
    case "STUDENT":
      return {
        id: relatedUser.id,
        firstName: relatedUser.first_name,
        lastName: relatedUser.last_name,
        profilePicture: relatedUser.profile_picture,
        role: "STUDENT"
      };
    case "TEACHER":
      return {
        id: relatedUser.id,
        firstName: relatedUser.first_name,
        lastName: relatedUser.last_name,
        profilePicture: relatedUser.profile_picture,
        role: "TEACHER"
      };
    // ... handle other types
  }
};

// Helper to find participant user data
const findParticipantUser = (
  participantRef: ActorRef,
  participantsUsers: RelatedUser[]
) => {
  return participantsUsers.find(user => {
    const userId = getUserFromRelatedUser(user)?.id;
    return userId === participantRef.id;
  });
};

// Usage
const ConversationItem = ({ conversation }: { conversation: ConversationWithRelations }) => {
  const otherParticipantRef = conversation.participants.find(
    p => p.id !== currentUserId
  );
  
  const otherParticipant = otherParticipantRef 
    ? findParticipantUser(otherParticipantRef, conversation.participants_users)
    : null;
  
  const userData = otherParticipant ? getUserFromRelatedUser(otherParticipant) : null;
  
  return (
    <div>
      <h3>{userData?.firstName} {userData?.lastName}</h3>
      <img src={userData?.profilePicture} />
    </div>
  );
};
```

### Step 4: Update Message Display Logic

**BEFORE:**
```typescript
// Old way - accessing sender fields directly
const MessageItem = ({ message }: { message: Message }) => {
  const senderId = message.sender.sender_id;
  const senderRole = message.sender.sender_role;
  
  return (
    <div>
      <span>From: {senderId}</span>
      <span>Role: {senderRole}</span>
    </div>
  );
};
```

**AFTER:**
```typescript
// New way - using sender ActorRef and sender_user
const MessageItem = ({ message }: { message: MessageWithRelations }) => {
  const senderId = message.sender.id;  // Changed from sender.sender_id
  const senderRole = message.sender.role;  // Changed from sender.sender_role
  
  // Access populated user data
  const senderUser = message.sender_user;
  const senderData = senderUser ? getUserFromRelatedUser(senderUser) : null;
  
  return (
    <div>
      <span>From: {senderData?.firstName} {senderData?.lastName}</span>
      <span>Role: {senderRole}</span>
      <img src={senderData?.profilePicture} />
    </div>
  );
};
```

### Step 5: Update Message Creation

**BEFORE:**
```typescript
// Old way - backend automatically set sender from auth
const sendMessage = async (conversationId: string, payload: string) => {
  const response = await fetch(`/conversations/${conversationId}/messages`, {
    method: 'POST',
    body: JSON.stringify({
      encrypted_payload: payload,
      nonce: generateNonce(),
      client_message_id: generateClientId()
    })
  });
};
```

**AFTER:**
```typescript
// New way - same! Backend still automatically sets sender from auth
const sendMessage = async (conversationId: string, payload: string) => {
  const response = await fetch(`/conversations/${conversationId}/messages`, {
    method: 'POST',
    body: JSON.stringify({
      encrypted_payload: payload,
      nonce: generateNonce(),
      client_message_id: generateClientId()
    })
  });
  
  // Response now includes sender_user populated data
  const message: MessageWithRelations = await response.json();
};
```

## Key Differences Summary

| Aspect | Before | After |
|--------|--------|-------|
| **Conversation participants** | `RelatedUser[]` (full objects) | `ActorRef[]` (id + role) |
| **Conversation response** | Same as stored | Includes `participants_users` array |
| **Message sender** | `{ sender_id, sender_role }` | `ActorRef { id, role }` |
| **Message response** | Same as stored | Includes `sender_user` object |
| **Creating conversation** | Send full user objects | Send simple `{ id, role }` refs |
| **Creating message** | No change | No change (backend sets sender) |
| **Display logic** | Access data from participants | Access data from `participants_users` |

## Benefits of New Structure

1. **Consistency**: Same `ActorRef` pattern used across comments, likes, conversations, and messages
2. **Smaller payloads**: When creating conversations, you only send `{ id, role }` instead of full user objects
3. **Cleaner separation**: Storage format (ActorRef) vs display format (with relations)
4. **Better performance**: Database queries are more efficient with simple references
5. **Type safety**: Clearer distinction between reference and populated data

## Migration Checklist

- [ ] Update TypeScript types for `Conversation`, `Message`, and `ActorRef`
- [ ] Add new types `ConversationWithRelations` and `MessageWithRelations`
- [ ] Update conversation creation to send `ActorRef[]` instead of `RelatedUser[]`
- [ ] Update conversation display to use `participants_users` array
- [ ] Update message display to use `sender.id` instead of `sender.sender_id`
- [ ] Update message display to use `sender_user` for populated data
- [ ] Create helper functions for extracting user data from `RelatedUser`
- [ ] Test all conversation and messaging features
- [ ] Update any WebSocket message handlers if needed

## Example: Complete React Component

```typescript
import React from 'react';

// Helper functions
const getUserFromRelatedUser = (relatedUser: RelatedUser) => {
  const baseData = {
    id: relatedUser.id || '',
    role: relatedUser.user_type
  };

  switch (relatedUser.user_type) {
    case "STUDENT":
      return {
        ...baseData,
        firstName: relatedUser.first_name,
        lastName: relatedUser.last_name,
        profilePicture: relatedUser.profile_picture
      };
    case "TEACHER":
      return {
        ...baseData,
        firstName: relatedUser.first_name,
        lastName: relatedUser.last_name,
        profilePicture: relatedUser.profile_picture
      };
    case "SCHOOLSTAFF":
      return {
        ...baseData,
        firstName: relatedUser.first_name,
        lastName: relatedUser.last_name,
        profilePicture: relatedUser.profile_picture
      };
    case "PARENT":
      return {
        ...baseData,
        firstName: relatedUser.first_name,
        lastName: relatedUser.last_name,
        profilePicture: relatedUser.profile_picture
      };
    default:
      return baseData;
  }
};

// Conversation List Component
const ConversationList: React.FC = () => {
  const [conversations, setConversations] = React.useState<ConversationWithRelations[]>([]);
  const currentUserId = "current-user-id"; // Get from auth context

  React.useEffect(() => {
    fetch('/conversations')
      .then(res => res.json())
      .then(data => setConversations(data.data));
  }, []);

  return (
    <div>
      {conversations.map(conv => {
        // Find other participant
        const otherParticipantRef = conv.participants.find(
          p => p.id !== currentUserId
        );
        
        // Get full user data
        const otherParticipantUser = conv.participants_users.find(user => {
          const userData = getUserFromRelatedUser(user);
          return userData.id === otherParticipantRef?.id;
        });
        
        const userData = otherParticipantUser 
          ? getUserFromRelatedUser(otherParticipantUser)
          : null;

        return (
          <div key={conv._id}>
            <img src={userData?.profilePicture} alt="" />
            <h3>{userData?.firstName} {userData?.lastName}</h3>
            <span>{userData?.role}</span>
          </div>
        );
      })}
    </div>
  );
};

// Message List Component
const MessageList: React.FC<{ conversationId: string }> = ({ conversationId }) => {
  const [messages, setMessages] = React.useState<MessageWithRelations[]>([]);

  React.useEffect(() => {
    fetch(`/conversations/${conversationId}/messages`)
      .then(res => res.json())
      .then(data => setMessages(data.data));
  }, [conversationId]);

  return (
    <div>
      {messages.map(msg => {
        const senderData = msg.sender_user 
          ? getUserFromRelatedUser(msg.sender_user)
          : null;

        return (
          <div key={msg._id}>
            <img src={senderData?.profilePicture} alt="" />
            <span>{senderData?.firstName} {senderData?.lastName}</span>
            <span>{msg.sender.role}</span>
            <p>{msg.encrypted_payload}</p>
          </div>
        );
      })}
    </div>
  );
};

export { ConversationList, MessageList };
```

## Questions or Issues?

If you encounter any issues during migration, check:

1. Are you using `participants_users` instead of `participants` for display?
2. Are you using `sender.id` instead of `sender.sender_id`?
3. Are you using `sender_user` for populated sender data?
4. Are you sending `ActorRef[]` when creating conversations?

The API will return proper error messages if the request format is incorrect.
