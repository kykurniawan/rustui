# Rust TUI Chat Application

A terminal-based chat application built with Rust, featuring a WebSocket server, TUI client, and CLI management tool.

## Workspace Structure

| Crate               | Description                                             |
| ------------------- | ------------------------------------------------------- |
| `rustui-server`     | WebSocket chat server with room-based routing           |
| `rustui-client`     | Terminal UI chat client                                 |
| `rustui-management` | CLI tool for managing rooms, users, and access controls |

## Prerequisites

- Rust 1.85+ (edition 2024)
- SQLite (bundled, no system install required)

## Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p rustui-server
cargo build -p rustui-client
cargo build -p rustui-management

# Release build
cargo build --release
```

## Data Directory

All persistent data is stored at `~/.rustui/` (cross-platform via `dirs` crate):

- `~/.rustui/rustui.db` — SQLite database

The directory is automatically created on first run.

## Running

### 1. Start the Server

```bash
cargo run -p rustui-server
# Or release binary:
./target/release/rustui-server
```

The server listens on `ws://127.0.0.1:8080`. Clients connect to specific rooms at `ws://host:port/room/<room-name>`.

### 2. Create Users and Rooms (Management CLI)

Before anyone can chat, you need rooms and users. The management CLI operates on the same database as the server.

```bash
# Create a room
cargo run -p rustui-management -- room:create general

# Create a user
cargo run -p rustui-management -- user:create alice mypassword

# Add user to room (required for access)
cargo run -p rustui-management -- user:add-room alice general
```

### 3. Start the Client

```bash
cargo run -p rustui-client
```

Login screen fields:

| #   | Field          | Description                                        |
| --- | -------------- | -------------------------------------------------- |
| 1   | Server Address | WebSocket URL, default `ws://127.0.0.1:8080`       |
| 2   | Room           | Room name (must match an existing room)            |
| 3   | Username       | Your username (must be created via management CLI) |
| 4   | Password       | Your password                                      |
| 5   | Encryption Key | Shared secret for E2E message encryption           |

## Management CLI Commands

```bash
# Room management
room:create <room_name>       # Create a room (alphanumeric + dash only)
room:delete <room_name>       # Delete a room
room:list                     # List all rooms
room:users <room_name>        # List users allowed in a room

# User management
user:create <username> <password>   # Create a user (blacklisted names rejected)
user:delete <username>              # Delete a user
user:list                           # List all users
user:rooms <username>               # List rooms a user can access
user:add-room <username> <room_name>    # Grant user access to a room
user:remove-room <username> <room_name> # Revoke user's access to a room
```

### Username Rules

- Alphanumeric characters and dash (`-`) only
- Case-insensitive blacklist: `admin`, `root`, `system`, `server`, `mod`, `moderator`, `operator`, `superuser`, `sys`, `daemon`

### Room Name Rules

- Alphanumeric characters and dash (`-`) only, non-empty

## Client Controls

### Login Screen

| Key       | Action                |
| --------- | --------------------- |
| TAB       | Switch between fields |
| ENTER     | Connect and login     |
| BACKSPACE | Delete character      |
| ESC       | Exit                  |

### Chat Screen

| Key       | Action                              |
| --------- | ----------------------------------- |
| SHIFT+TAB | Toggle focus (Message List / Input) |
| ENTER     | Send message (Input focused)        |
| ESC       | Exit                                |

**Message List (focused — cyan border):**
| Key | Action |
|---|---|
| UP/DOWN | Scroll one message |
| PAGE UP/DOWN | Scroll 10 messages |
| HOME | Jump to first message |
| END | Jump to latest (bottom, re-enables auto-scroll) |

**Input (focused — green border):**
| Key | Action |
|---|---|
| LEFT/RIGHT | Move cursor horizontally |
| UP/DOWN | Move cursor between lines |
| HOME/END | Jump to start/end of input |
| BACKSPACE | Delete before cursor |
| DELETE | Delete after cursor |

## Features

- WebSocket-based real-time chat with room isolation
- Terminal UI with crossterm and tui-rs
- End-to-End Encryption (AES-256-GCM)
- User authentication with SHA-256 password hashing
- Per-room broadcasting — messages only reach users in the same room
- Smart message scrolling with auto-scroll
- Multi-line message wrapping
- Participant list tracking per room
- SQLite persistent storage

## Docker Deployment

### Build the Image

```bash
docker build -t rustui-server .
```

### Run the Container

```bash
# Basic run (data lost on container removal)
docker run -p 8080:8080 rustui-server

# With persistent data volume
docker run -p 8080:8080 -v rustui-data:/root/.rustui rustui-server

# With a local data directory
docker run -p 8080:8080 -v $(pwd)/.rustui:/root/.rustui rustui-server
```

### Manage Users and Rooms in Docker

The management CLI connects to the same database. When using Docker, run management commands against the persisted data volume:

```bash
# Using a shared volume
docker run --rm -v rustui-data:/root/.rustui rustui-server \
  rustui-management room:create general

docker run --rm -v rustui-data:/root/.rustui rustui-server \
  rustui-management user:create alice secretpass

docker run --rm -v rustui-data:/root/.rustui rustui-server \
  rustui-management user:add-room alice general
```

Or run the management CLI locally pointing to the same directory:

```bash
# If using a local bind mount
cargo run -p rustui-management -- room:create general
```

## Security Considerations

- Passwords are SHA-256 hashed before storage (not plaintext)
- Messages are encrypted with AES-256-GCM on the client; the server only sees ciphertext
- The encryption key is a shared secret — all participants in a room must use the same key to decrypt each other's messages
- This is a shared-key model and does not provide forward secrecy
- For production, consider TLS (wss://) and a reverse proxy like nginx
