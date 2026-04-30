# Security Features

## End-to-End Encryption (E2E)

RustUI Chat implements end-to-end encryption to protect message privacy.

### How It Works

1. **Shared Secret Key**: All users must agree on a shared encryption key before chatting
2. **Client-Side Encryption**: Messages are encrypted on the sender's client before transmission
3. **Server Blindness**: The server only sees encrypted ciphertext, not plaintext messages
4. **Client-Side Decryption**: Only recipients with the correct key can decrypt messages

### Technical Details

- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Derivation**: SHA-256 hash of the passphrase
- **Nonce**: Random 96-bit nonce generated for each message
- **Encoding**: Base64 encoding for transmission
- **Format**: `base64(nonce || ciphertext || auth_tag)`

### Security Properties

✅ **Confidentiality**: Server cannot read message content  
✅ **Authenticity**: GCM mode provides authentication  
✅ **Integrity**: Tampering is detected and rejected  
✅ **Forward Secrecy**: ❌ Not implemented (uses static shared key)  

### Limitations

⚠️ **Shared Key Model**: All users must know the same encryption key
- Not suitable for large groups
- Key distribution is manual (out-of-band)
- Compromised key affects all messages

⚠️ **Metadata Visible**: Server can see:
- Who is sending messages (username)
- When messages are sent (timestamp)
- Message size (ciphertext length)
- Who is online (participant list)

⚠️ **No Forward Secrecy**: 
- Same key used for all messages
- If key is compromised, all past messages can be decrypted
- Consider using different keys for different sessions

### Usage

1. **Agree on a Key**: All participants must use the same encryption key
2. **Enter Key at Login**: Type the shared key in the "E2E Encryption Key" field
3. **Chat Securely**: Messages are automatically encrypted/decrypted

### Example

```
User A enters key: "my_secret_chat_key_2024"
User B enters key: "my_secret_chat_key_2024"
User C enters key: "my_secret_chat_key_2024"

✅ All users can communicate
```

```
User A enters key: "key1"
User B enters key: "key2"

❌ User B sees: [encrypted: base64data...]
```

### Best Practices

1. **Use Strong Keys**: At least 20 characters, mix of letters/numbers/symbols
2. **Share Keys Securely**: Use a secure channel (not the chat itself!)
3. **Rotate Keys**: Change keys periodically
4. **Don't Reuse Keys**: Use different keys for different groups
5. **Keep Keys Secret**: Never share keys in public channels

### Server Logging

When the server logs messages, it only sees encrypted data:

```
Broadcast from alice: iKV3N2xQp8F7Hw== (encrypted)
Broadcast from bob: 9mK2L1pRt6G8Jx== (encrypted)
```

The server cannot decrypt these messages without the encryption key.

### Future Improvements

Potential enhancements for better security:

- [ ] Per-user key pairs (asymmetric encryption)
- [ ] Diffie-Hellman key exchange
- [ ] Forward secrecy with ephemeral keys
- [ ] Key rotation mechanism
- [ ] Multi-device support
- [ ] Encrypted participant list
- [ ] Encrypted metadata

## Authentication

- Username/password authentication
- Passwords transmitted in plaintext over WebSocket
- ⚠️ Use TLS/WSS in production for transport security

## Recommendations for Production

1. **Use WSS (WebSocket Secure)**: Encrypt the WebSocket connection
2. **Hash Passwords**: Server should hash passwords, not store plaintext
3. **Rate Limiting**: Prevent brute force attacks
4. **Session Tokens**: Use tokens instead of sending passwords repeatedly
5. **Certificate Pinning**: Verify server identity
6. **Audit Logging**: Log security events (not message content)
