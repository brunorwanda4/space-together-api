# Frontend Integration Guide for Encrypted Messaging

## Overview
This guide shows how to integrate the end-to-end encrypted messaging system in your frontend application.

## Prerequisites

Install required libraries:
```bash
npm install crypto-js
# or
yarn add crypto-js
```

For RSA encryption, you can use the Web Crypto API (built into modern browsers) or a library like `node-forge`.

## Step 1: Generate and Upload RSA Key Pair

### Generate Key Pair (on first login or registration)

```typescript
// Using Web Crypto API
async function generateRSAKeyPair() {
  const keyPair = await window.crypto.subtle.generateKey(
    {
      name: "RSA-OAEP",
      modulusLength: 2048,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true, // extractable
    ["encrypt", "decrypt"]
  );

  // Export public key in PEM format
  const publicKeyBuffer = await window.crypto.subtle.exportKey(
    "spki",
    keyPair.publicKey
  );
  const publicKeyPEM = arrayBufferToPEM(publicKeyBuffer, "PUBLIC KEY");

  // Export private key (store locally, never send to server)
  const privateKeyBuffer = await window.crypto.subtle.exportKey(
    "pkcs8",
    keyPair.privateKey
  );
  const privateKeyPEM = arrayBufferToPEM(privateKeyBuffer, "PRIVATE KEY");

  // Store private key in IndexedDB or localStorage (encrypted)
  await storePrivateKey(privateKeyPEM);

  return { publicKeyPEM, privateKeyPEM };
}

function arrayBufferToPEM(buffer: ArrayBuffer, label: string): string {
  const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)));
  const formatted = base64.match(/.{1,64}/g)?.join('\n') || base64;
  return `-----BEGIN ${label}-----\n${formatted}\n-----END ${label}-----`;
}
```

### Upload Public Key to Server

```typescript
async function uploadPublicKey(publicKeyPEM: string, token: string) {
  const response = await fetch('/m/users/public-key', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      public_key: publicKeyPEM,
      key_algorithm: 'RSA-2048',
    }),
  });

  if (!response.ok) {
    throw new Error('Failed to upload public key');
  }

  return await response.json();
}
```

## Step 2: Create an Encrypted Conversation

### Fetch Public Keys for Participants

```typescript
async function getPublicKeys(userIds: string[], token: string) {
  const response = await fetch(
    `/m/users/public-keys?user_ids=${userIds.join(',')}`,
    {
      headers: {
        'Authorization': `Bearer ${token}`,
      },
    }
  );

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message);
  }

  const data = await response.json();
  return data.public_keys;
}
```

### Generate Symmetric Key and Encrypt for Each Participant

```typescript
import CryptoJS from 'crypto-js';

async function createEncryptedConversation(
  participants: Array<{ id: string; role: string }>,
  isGroup: boolean,
  groupName: string | null,
  token: string,
  schoolToken?: string
) {
  // 1. Generate random AES-256 symmetric key
  const symmetricKey = CryptoJS.lib.WordArray.random(32); // 256 bits
  const symmetricKeyBase64 = CryptoJS.enc.Base64.stringify(symmetricKey);

  // Store symmetric key locally for this conversation
  // (You'll need the conversation ID after creation)

  // 2. Fetch public keys for all participants
  const userIds = participants.map(p => p.id);
  const publicKeys = await getPublicKeys(userIds, token);

  // 3. Encrypt symmetric key for each participant
  const encryptedKeys = await Promise.all(
    participants.map(async (participant) => {
      const publicKeyInfo = publicKeys.find(
        pk => pk.user_id === participant.id
      );

      if (!publicKeyInfo) {
        throw new Error(`Public key not found for user ${participant.id}`);
      }

      // Import public key
      const publicKey = await importPublicKey(publicKeyInfo.public_key);

      // Encrypt symmetric key with participant's public key
      const encryptedKey = await window.crypto.subtle.encrypt(
        {
          name: "RSA-OAEP",
        },
        publicKey,
        base64ToArrayBuffer(symmetricKeyBase64)
      );

      return {
        user_id: participant.id,
        user_role: participant.role,
        encrypted_key: arrayBufferToBase64(encryptedKey),
      };
    })
  );

  // 4. Create conversation
  const headers: Record<string, string> = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  };

  if (schoolToken) {
    headers['School-Token'] = schoolToken;
  }

  const response = await fetch('/conversations', {
    method: 'POST',
    headers,
    body: JSON.stringify({
      participants,
      is_group: isGroup,
      name: groupName,
      encrypted_keys: encryptedKeys,
    }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message);
  }

  const data = await response.json();
  const conversation = data.conversation;

  // 5. Store symmetric key locally with conversation ID
  await storeConversationKey(conversation._id, symmetricKeyBase64);

  return conversation;
}

async function importPublicKey(pemKey: string): Promise<CryptoKey> {
  const pemContents = pemKey
    .replace('-----BEGIN PUBLIC KEY-----', '')
    .replace('-----END PUBLIC KEY-----', '')
    .replace(/\s/g, '');
  
  const binaryDer = base64ToArrayBuffer(pemContents);

  return await window.crypto.subtle.importKey(
    'spki',
    binaryDer,
    {
      name: 'RSA-OAEP',
      hash: 'SHA-256',
    },
    true,
    ['encrypt']
  );
}

function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}

function arrayBufferToBase64(buffer: ArrayBuffer): string {
  return btoa(String.fromCharCode(...new Uint8Array(buffer)));
}
```

## Step 3: Send Encrypted Message

```typescript
async function sendEncryptedMessage(
  conversationId: string,
  messageContent: string,
  token: string,
  schoolToken?: string
) {
  // 1. Get symmetric key for this conversation
  const symmetricKeyBase64 = await getConversationKey(conversationId);
  const symmetricKey = CryptoJS.enc.Base64.parse(symmetricKeyBase64);

  // 2. Generate random nonce (12 bytes for GCM)
  const nonce = CryptoJS.lib.WordArray.random(12);

  // 3. Encrypt message with AES-256-GCM
  const encrypted = CryptoJS.AES.encrypt(messageContent, symmetricKey, {
    iv: nonce,
    mode: CryptoJS.mode.GCM,
    padding: CryptoJS.pad.NoPadding,
  });

  const encryptedPayload = encrypted.ciphertext.toString(CryptoJS.enc.Base64);
  const nonceBase64 = CryptoJS.enc.Base64.stringify(nonce);

  // 4. Generate unique client message ID
  const clientMessageId = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  // 5. Send encrypted message
  const headers: Record<string, string> = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  };

  if (schoolToken) {
    headers['School-Token'] = schoolToken;
  }

  const response = await fetch(`/conversations/${conversationId}/messages`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      encrypted_payload: encryptedPayload,
      nonce: nonceBase64,
      key_version: 1,
      message_type: 'TEXT',
      client_message_id: clientMessageId,
    }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message);
  }

  return await response.json();
}
```

## Step 4: Receive and Decrypt Messages

```typescript
async function getAndDecryptMessages(
  conversationId: string,
  page: number,
  limit: number,
  token: string,
  schoolToken?: string
) {
  // 1. Fetch encrypted messages
  const headers: Record<string, string> = {
    'Authorization': `Bearer ${token}`,
  };

  if (schoolToken) {
    headers['School-Token'] = schoolToken;
  }

  const response = await fetch(
    `/conversations/${conversationId}/messages?page=${page}&limit=${limit}`,
    { headers }
  );

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message);
  }

  const data = await response.json();
  const messages = data.data;

  // 2. Get symmetric key for this conversation
  const symmetricKeyBase64 = await getConversationKey(conversationId);
  const symmetricKey = CryptoJS.enc.Base64.parse(symmetricKeyBase64);

  // 3. Decrypt each message
  const decryptedMessages = messages.map((message: any) => {
    try {
      const nonce = CryptoJS.enc.Base64.parse(message.nonce);
      const ciphertext = CryptoJS.enc.Base64.parse(message.encrypted_payload);

      const decrypted = CryptoJS.AES.decrypt(
        { ciphertext } as any,
        symmetricKey,
        {
          iv: nonce,
          mode: CryptoJS.mode.GCM,
          padding: CryptoJS.pad.NoPadding,
        }
      );

      const decryptedContent = decrypted.toString(CryptoJS.enc.Utf8);

      return {
        ...message,
        content: decryptedContent,
      };
    } catch (error) {
      console.error('Failed to decrypt message:', error);
      return {
        ...message,
        content: '[Decryption failed]',
      };
    }
  });

  return {
    ...data,
    data: decryptedMessages,
  };
}
```

## Step 5: Key Storage (IndexedDB)

```typescript
// Store private key (encrypted with user password or device key)
async function storePrivateKey(privateKeyPEM: string) {
  // In production, encrypt the private key before storing
  localStorage.setItem('user_private_key', privateKeyPEM);
}

// Store conversation symmetric keys
async function storeConversationKey(
  conversationId: string,
  symmetricKeyBase64: string
) {
  const keys = JSON.parse(localStorage.getItem('conversation_keys') || '{}');
  keys[conversationId] = symmetricKeyBase64;
  localStorage.setItem('conversation_keys', JSON.stringify(keys));
}

// Retrieve conversation symmetric key
async function getConversationKey(conversationId: string): Promise<string> {
  const keys = JSON.parse(localStorage.getItem('conversation_keys') || '{}');
  
  if (keys[conversationId]) {
    return keys[conversationId];
  }

  // If not found locally, fetch from server and decrypt
  return await fetchAndDecryptConversationKey(conversationId);
}

async function fetchAndDecryptConversationKey(
  conversationId: string
): Promise<string> {
  const token = localStorage.getItem('auth_token');
  
  // 1. Fetch encrypted key from server
  const response = await fetch(`/conversations/${conversationId}/key`, {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    throw new Error('Failed to fetch conversation key');
  }

  const data = await response.json();
  const encryptedKeyBase64 = data.encrypted_key_for_user;

  // 2. Get user's private key
  const privateKeyPEM = localStorage.getItem('user_private_key');
  if (!privateKeyPEM) {
    throw new Error('Private key not found');
  }

  // 3. Import private key
  const privateKey = await importPrivateKey(privateKeyPEM);

  // 4. Decrypt symmetric key
  const encryptedKeyBuffer = base64ToArrayBuffer(encryptedKeyBase64);
  const decryptedKeyBuffer = await window.crypto.subtle.decrypt(
    {
      name: 'RSA-OAEP',
    },
    privateKey,
    encryptedKeyBuffer
  );

  const symmetricKeyBase64 = arrayBufferToBase64(decryptedKeyBuffer);

  // 5. Store for future use
  await storeConversationKey(conversationId, symmetricKeyBase64);

  return symmetricKeyBase64;
}

async function importPrivateKey(pemKey: string): Promise<CryptoKey> {
  const pemContents = pemKey
    .replace('-----BEGIN PRIVATE KEY-----', '')
    .replace('-----END PRIVATE KEY-----', '')
    .replace(/\s/g, '');
  
  const binaryDer = base64ToArrayBuffer(pemContents);

  return await window.crypto.subtle.importKey(
    'pkcs8',
    binaryDer,
    {
      name: 'RSA-OAEP',
      hash: 'SHA-256',
    },
    true,
    ['decrypt']
  );
}
```

## Complete Example: Creating a 1-on-1 Conversation

```typescript
async function createDirectConversation(
  otherUserId: string,
  otherUserRole: string,
  currentUserId: string,
  currentUserRole: string,
  token: string,
  schoolToken?: string
) {
  const participants = [
    { id: currentUserId, role: currentUserRole },
    { id: otherUserId, role: otherUserRole },
  ];

  const conversation = await createEncryptedConversation(
    participants,
    false, // not a group
    null, // no name for direct conversation
    token,
    schoolToken
  );

  console.log('Conversation created:', conversation);
  return conversation;
}

// Usage
const conversation = await createDirectConversation(
  '507f1f77bcf86cd799439012', // other user ID
  'STUDENT',
  '507f1f77bcf86cd799439011', // current user ID
  'TEACHER',
  'your-auth-token',
  'optional-school-token'
);
```

## Complete Example: Creating a Group Conversation

```typescript
async function createGroupConversation(
  groupName: string,
  participantIds: Array<{ id: string; role: string }>,
  token: string,
  schoolToken?: string
) {
  const conversation = await createEncryptedConversation(
    participantIds,
    true, // is a group
    groupName,
    token,
    schoolToken
  );

  console.log('Group conversation created:', conversation);
  return conversation;
}

// Usage
const groupConversation = await createGroupConversation(
  'Math Study Group',
  [
    { id: '507f1f77bcf86cd799439011', role: 'TEACHER' },
    { id: '507f1f77bcf86cd799439012', role: 'STUDENT' },
    { id: '507f1f77bcf86cd799439013', role: 'STUDENT' },
  ],
  'your-auth-token',
  'optional-school-token'
);
```

## Security Best Practices

1. **Never send private keys to the server**
2. **Encrypt private keys before storing locally** (use user password or device key)
3. **Use IndexedDB instead of localStorage** for better security
4. **Implement key rotation** when participants change
5. **Clear keys on logout**
6. **Validate all inputs** before encryption
7. **Use secure random number generation** for keys and nonces
8. **Implement proper error handling** for decryption failures

## Error Handling

```typescript
try {
  const conversation = await createEncryptedConversation(/* ... */);
} catch (error) {
  if (error.message.includes('Public key not found')) {
    // Handle missing public key - prompt user to generate keys
    console.error('User needs to generate encryption keys');
  } else if (error.message.includes('already exists')) {
    // Handle duplicate conversation
    console.log('Conversation already exists');
  } else {
    // Handle other errors
    console.error('Failed to create conversation:', error);
  }
}
```

## Testing

Use the browser console to test the encryption flow:

```javascript
// 1. Generate keys
const { publicKeyPEM, privateKeyPEM } = await generateRSAKeyPair();
console.log('Public Key:', publicKeyPEM);

// 2. Upload public key
await uploadPublicKey(publicKeyPEM, 'your-token');

// 3. Create conversation
const conv = await createDirectConversation(/* ... */);

// 4. Send message
await sendEncryptedMessage(conv._id, 'Hello!', 'your-token');

// 5. Receive messages
const messages = await getAndDecryptMessages(conv._id, 1, 20, 'your-token');
console.log('Decrypted messages:', messages);
```
