# Changelog

## [Unreleased]

### Added
- **Room-based chat routing**: Server supports multiple rooms at `ws://host:port/room/<name>`
  - WebSocket connections are scoped to a specific room
  - Broadcasts only reach users in the same room
  - Room field added to client login screen (5th field)
  - Connection URL includes room path automatically

- **rustui-management crate**: CLI tool for managing rooms, users, and access
  - `room:create / room:delete / room:list / room:users` commands
  - `user:create / user:delete / user:list / user:rooms` commands
  - `user:add-room / user:remove-room` for access control
  - Username validation: alphanumeric + dash only
  - Blacklisted usernames: admin, root, system, server, mod, moderator, operator, superuser, sys, daemon
  - Room name validation: alphanumeric + dash only

- **SQLite persistent storage**: Users, rooms, and access mappings stored in database
  - Database at `~/.rustui/rustui.db` (cross-platform via `dirs` crate)
  - Tables: `users`, `rooms`, `user_rooms` (many-to-many)
  - Auto-created on first run
  - Password hashing with SHA-256

- **Room-based access control**: Users must be explicitly added to a room to join
  - Unauthorized users receive "Access denied" error
  - Per-room participant lists

- **Chat header redesign**: Multi-line header with block title
  - "SECURE CHAT" as centered block title
  - Info lines: Server, Room, User, State (online count)
  - Wraps cleanly on small terminals

- **Docker deployment**: Multi-stage Dockerfile with server and management CLI
  - Persistent data via volume mounts
  - Management commands runnable inside container

- **Server address shown in chat UI**: Current server address visible in header

### Changed
- User authentication now validates against SQLite database (not hardcoded)
- No default users seeded (use management CLI to create users)
- Database moved from in-memory to SQLite at `~/.rustui/rustui.db`
- Client login screen expanded to 5 fields (added Room)
- Server broadcasts scoped per-room instead of globally
- Password storage upgraded from plaintext to SHA-256 hash
- `.gitignore` cleaned up; `.dockerignore` added

### Removed
- Hardcoded default users (admin/secret123, rizky/pass123, john/john123)
- Global chat broadcast (replaced with room-scoped broadcast)

## Previous (E2E Encryption & TUI)

### Added
- **End-to-End Encryption (E2E)**: Messages encrypted before sending
  - AES-256-GCM encryption algorithm
  - Shared secret key entered during login
  - Server only sees encrypted messages (base64-encoded ciphertext)
  - Failed decryption shows `[encrypted: ...]` instead of plaintext

- **Configurable server address**: Input during login (default `ws://127.0.0.1:8080`)

- **Focus system**: Toggle between Message List and Input Field with SHIFT+TAB
  - Visual indicators for focused section
  - Colored borders (Cyan for messages, Green for input)

- **Enhanced input controls**: Full cursor movement (left, right, up, down, home, end, delete)

- **Enhanced message list controls**: Scroll with UP/DOWN, PAGE UP/DOWN, HOME, END

### Fixed
- Login flow: single Enter press to login (was double-enter)
- Participant count: accurate online user count via pending message processing
- Chat scrolling: intelligent height calculation with auto-scroll
- Long message wrapping: proper multi-line wrapping based on terminal width
- Input cursor positioning: correct placement on wrapped lines
- Client now processes ALL queued messages per loop iteration
