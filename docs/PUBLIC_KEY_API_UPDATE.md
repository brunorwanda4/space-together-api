# Public Key API Update - Partial Results Support

## Problem Fixed
Previously, the `/public-keys` endpoint would return a 404 error if ANY user was missing a public key, causing conversation creation to fail completely.

## New Behavior
The endpoint now returns a 200 OK response with partial results, including:
- All found public keys
- List of user IDs that don't have public keys yet
- A descriptive message

## API Response

### All Keys Found (200 OK)
```json
{
  "public_keys": [
    {
      "user_id": "507f1f77bcf86cd799439011",
      "public_key": "-----BEGIN PUBLIC KEY-----...",
      "key_algorithm": "RSA-2048",
      "created_at": "2026-03-01T10:00:00Z"
    }
  ]
}
```

### Some Keys Missing (200 OK)
```json
{
  "public_keys": [
    {
      "user_id": "507f1f77bcf86cd799439011",
      "public_key": "-----BEGIN PUBLIC KEY-----...",
      "key_algorithm": "RSA-2048",
      "created_at": "2026-03-01T10:00:00Z"
    }
  ],
  "missing_user_ids": ["69a3a774c5da1d5a13ae154b"],
  "message": "Public keys not found for 1 user(s). These users need to upload their public keys."
}
```

## Frontend Integration

### Updated TypeScript Function

```typescript
export async function getUserPublicKeysAction(userIds: string[]) {
  const auth = await authContext();
  if (!auth) {
    throw new Error("Unauthorized - Please log in");
  }

  try {
    const response = await fetch(
      `${API_BASE}/m-users/public-keys?user_ids=${userIds.join(",")}`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${auth.token}`,
          ...(auth.schoolToken ? { "School-Token": auth.schoolToken } : {}),
        },
        cache: "no-store",
      }
    );

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        errorData.message || `Failed to get public keys: ${response.status}`
      );
    }

    const data = await response.json();
    
    // Check if some keys are missing
    if (data.missing_user_ids && data.missing_user_ids.length > 0) {
      console.warn(
        `Missing public keys for users: ${data.missing_user_ids.join(", ")}`
      );
      
      // Option 1: Throw error to prevent conversation creation
      throw new Error(
        `Cannot create encrypted conversation: ${data.missing_user_ids.length} user(s) haven't set up encryption yet. ` +
        `Please ask them to enable encryption in their settings.`
      );
      
      // Option 2: Return partial data and let caller decide
      // return {
      //   publicKeys: data.public_keys,
      //   missingUserIds: data.missing_user_ids,
      //   hasAllKeys: false
      // };
    }

    return {
      publicKeys: data.public_keys,
      missingUserIds: [],
      hasAllKeys: true
    };
  } catch (error) {
    console.error("Error getting public keys:", error);
    throw error;
  }
}
```

### Handling Missing Keys in Conversation Creation

```typescript
async function createEncryptedConversation(participantIds: string[]) {
  try {
    const keyData = await getUserPublicKeysAction(participantIds);
    
    if (!keyData.hasAllKeys) {
      // Show user-friendly error
      return {
        success: false,
        error: `Cannot create encrypted conversation. ${keyData.missingUserIds.length} participant(s) need to enable encryption first.`,
        missingUsers: keyData.missingUserIds
      };
    }
    
    // Proceed with encryption and conversation creation
    const encryptedKeys = encryptConversationKey(keyData.publicKeys);
    // ... rest of conversation creation
    
  } catch (error) {
    // Handle other errors
    return {
      success: false,
      error: error.message
    };
  }
}
```

## User Experience Recommendations

1. **Proactive Check**: Before showing "Create Conversation" UI, check if all selected users have public keys
2. **Clear Messaging**: Show which specific users need to enable encryption
3. **Fallback Option**: Offer to create a non-encrypted conversation if encryption isn't available
4. **Settings Link**: Provide a direct link to encryption settings for users who need to set it up

## Backend Changes

- Added `get_public_keys_partial()` method to `UserPublicKeyService`
- Updated `/public-keys` endpoint to return partial results instead of failing
- Maintains backward compatibility - response structure is the same when all keys are found
