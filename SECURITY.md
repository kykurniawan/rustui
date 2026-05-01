# Security

## End-to-End Encryption (E2E)

RustUI Chat implements end-to-end encryption to protect message privacy.

### How It Works

1. **Shared Secret Key**: All users in a room must use the same encryption key
2. **Client-Side Encryption**: Messages are encrypted on the sender's client before transmission
3. **Server Blindness**: The server only sees encrypted ciphertext, not plaintext
4. **Client-Side Decryption**: Only recipients with the correct key can decrypt messages

### Technical Details

- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Derivation**: SHA-256 hash of the passphrase
- **Nonce**: Random 96-bit nonce generated for each message
- **Encoding**: Base64 encoding for transmission
- **Format**: `base64(nonce || ciphertext || auth_tag)`

### Security Properties

- [x] **Confidentiality**: Server cannot read message content
- [x] **Authenticity**: GCM mode provides authentication
- [x] **Integrity**: Tampering is detected and rejected
- [ ] **Forward Secrecy**: Not implemented (uses static shared key)

### Limitations

- **Shared Key Model**: All users in a room must know the same encryption key. Not suitable for large groups. Key distribution is manual (out-of-band). Compromised key affects all messages.
- **Metadata Visible**: Server can see who is sending messages (username), when messages are sent (timestamp), message size (ciphertext length), who is online (participant list), and which room they're in.
- **No Forward Secrecy**: Same key used for all messages. If the key is compromised, all past messages can be decrypted.

### Usage

1. Agree on a key with all room participants
2. Enter the key in the "Encryption Key" field at login
3. Messages are automatically encrypted/decrypted

```
User A enters key: "shared-room-key-2024"
User B enters key: "shared-room-key-2024"
-> Both can read each other's messages

User A enters key: "key1"
User B enters key: "key2"
-> User B sees: [encrypted: base64data...]
```

### Best Practices

1. Use strong keys (20+ characters, mix of letters/numbers/symbols)
2. Share keys securely out-of-band (not through the chat itself)
3. Rotate keys periodically
4. Use different keys for different rooms
5. Never share keys in public channels

### Server Logging

The server only sees encrypted payloads:

```
Broadcast from alice in room general: iKV3N2xQp8F7Hw== (encrypted ciphertext)
```

## Authentication

- Username/password authentication validated against SQLite database
- Passwords stored as SHA-256 hashes (not plaintext)
- Passwords transmitted in plaintext over WebSocket — use TLS (wss://) in production
- Usernames must be alphanumeric + dash only
- Blacklisted usernames rejected at creation: `admin`, `root`, `system`, `server`, `mod`, `moderator`, `operator`, `superuser`, `sys`, `daemon`

## Authorization

- Room-based access control: users must be explicitly added to a room via the management CLI
- A user without room access cannot join, even with valid credentials
- Each room is isolated — broadcasts only reach users in the same room

## Database

- SQLite database stored at `~/.rustui/rustui.db`
- Database is a single file — protect it with filesystem permissions
- No built-in encryption at rest for the database
- Management CLI has full access to the database — restrict execution to administrators

## Recommendations for Production

1. **Use WSS (WebSocket Secure)**: Encrypt the WebSocket connection with TLS (e.g., nginx reverse proxy)
2. **Rate Limiting**: Prevent brute-force login attempts (not currently implemented)
3. **Session Tokens**: Use tokens instead of sending passwords for every connection (not currently implemented)
4. **Database Encryption**: Encrypt the SQLite database at rest if storing on shared infrastructure
5. **Audit Logging**: Log authentication events and management actions
6. **Filesystem Permissions**: Restrict access to `~/.rustui/` to the server process user
7. **Docker Volume Security**: When using Docker, ensure persistent volumes have appropriate permissions

## Future Improvements

- [ ] Per-user key pairs (asymmetric encryption)
- [ ] Diffie-Hellman key exchange
- [ ] Forward secrecy with ephemeral keys
- [ ] Key rotation mechanism
- [ ] Encrypted participant list and metadata
- [ ] Rate limiting for authentication
- [ ] Session-based authentication tokens
- [ ] Database encryption at rest
- [ ] Audit logging for management commands
