use sha2::{Digest, Sha256};
use std::env;

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

const BLACKLISTED_USERNAMES: &[&str] = &[
    "admin",
    "root",
    "system",
    "server",
    "mod",
    "moderator",
    "operator",
    "superuser",
    "sys",
    "daemon",
];

fn is_valid_room_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn is_valid_username(name: &str) -> bool {
    is_valid_room_name(name) && !BLACKLISTED_USERNAMES.contains(&name.to_lowercase().as_str())
}

fn open_db() -> rusqlite::Connection {
    let home = dirs::home_dir().expect("Could not determine home directory");
    let data_dir = home.join(".rustui");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    let db_path = data_dir.join("rustui.db");
    let conn = rusqlite::Connection::open(db_path).expect("Failed to open database");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS rooms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS user_rooms (
            user_id INTEGER NOT NULL,
            room_id INTEGER NOT NULL,
            PRIMARY KEY (user_id, room_id),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE
        );",
    )
    .expect("Failed to create tables");
    conn
}

fn cmd_room_create(conn: &rusqlite::Connection, name: &str) {
    if !is_valid_room_name(name) {
        eprintln!("Error: Room name must be alphanumeric or dash only, and non-empty");
        std::process::exit(1);
    }
    match conn.execute("INSERT INTO rooms (name) VALUES (?1)", [name]) {
        Ok(_) => println!("Room '{}' created.", name),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                eprintln!("Error: Room '{}' already exists.", name);
            } else {
                eprintln!("Error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn cmd_room_delete(conn: &rusqlite::Connection, name: &str) {
    let deleted = conn
        .execute("DELETE FROM rooms WHERE name = ?1", [name])
        .expect("Failed to delete room");
    if deleted == 0 {
        eprintln!("Error: Room '{}' not found.", name);
        std::process::exit(1);
    }
    println!("Room '{}' deleted.", name);
}

fn cmd_user_create(conn: &rusqlite::Connection, username: &str, password: &str) {
    if !is_valid_username(username) {
        if BLACKLISTED_USERNAMES.contains(&username.to_lowercase().as_str()) {
            eprintln!("Error: Username '{}' is reserved (blacklisted).", username);
        } else {
            eprintln!("Error: Username must be alphanumeric or dash only, and non-empty");
        }
        std::process::exit(1);
    }
    let hash = hash_password(password);
    match conn.execute(
        "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
        rusqlite::params![username, hash],
    ) {
        Ok(_) => println!("User '{}' created.", username),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                eprintln!("Error: User '{}' already exists.", username);
            } else {
                eprintln!("Error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn cmd_user_delete(conn: &rusqlite::Connection, username: &str) {
    let deleted = conn
        .execute("DELETE FROM users WHERE username = ?1", [username])
        .expect("Failed to delete user");
    if deleted == 0 {
        eprintln!("Error: User '{}' not found.", username);
        std::process::exit(1);
    }
    println!("User '{}' deleted.", username);
}

fn cmd_user_add_room(conn: &rusqlite::Connection, username: &str, room_name: &str) {
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE username = ?1",
            [username],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            eprintln!("Error: User '{}' not found.", username);
            std::process::exit(1);
        });

    let room_id: i64 = conn
        .query_row(
            "SELECT id FROM rooms WHERE name = ?1",
            [room_name],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            eprintln!("Error: Room '{}' not found.", room_name);
            std::process::exit(1);
        });

    match conn.execute(
        "INSERT INTO user_rooms (user_id, room_id) VALUES (?1, ?2)",
        rusqlite::params![user_id, room_id],
    ) {
        Ok(_) => println!("User '{}' added to room '{}'.", username, room_name),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") || msg.contains("PRIMARY KEY") {
                eprintln!(
                    "Error: User '{}' is already in room '{}'.",
                    username, room_name
                );
            } else {
                eprintln!("Error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn cmd_user_remove_room(conn: &rusqlite::Connection, username: &str, room_name: &str) {
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE username = ?1",
            [username],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            eprintln!("Error: User '{}' not found.", username);
            std::process::exit(1);
        });

    let room_id: i64 = conn
        .query_row(
            "SELECT id FROM rooms WHERE name = ?1",
            [room_name],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            eprintln!("Error: Room '{}' not found.", room_name);
            std::process::exit(1);
        });

    let deleted = conn
        .execute(
            "DELETE FROM user_rooms WHERE user_id = ?1 AND room_id = ?2",
            rusqlite::params![user_id, room_id],
        )
        .expect("Failed to remove user from room");

    if deleted == 0 {
        eprintln!(
            "Error: User '{}' is not in room '{}'.",
            username, room_name
        );
        std::process::exit(1);
    }
    println!("User '{}' removed from room '{}'.", username, room_name);
}

fn cmd_room_list(conn: &rusqlite::Connection) {
    let mut stmt = conn
        .prepare("SELECT name FROM rooms ORDER BY name")
        .expect("Failed to query rooms");
    let rooms: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .expect("Failed to read rooms")
        .filter_map(|r| r.ok())
        .collect();

    if rooms.is_empty() {
        println!("No rooms found.");
    } else {
        println!("Rooms:");
        for (i, room) in rooms.iter().enumerate() {
            println!("  {}. {}", i + 1, room);
        }
    }
}

fn cmd_user_list(conn: &rusqlite::Connection) {
    let mut stmt = conn
        .prepare("SELECT username FROM users ORDER BY username")
        .expect("Failed to query users");
    let users: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .expect("Failed to read users")
        .filter_map(|r| r.ok())
        .collect();

    if users.is_empty() {
        println!("No users found.");
    } else {
        println!("Users:");
        for (i, user) in users.iter().enumerate() {
            println!("  {}. {}", i + 1, user);
        }
    }
}

fn cmd_room_users(conn: &rusqlite::Connection, room_name: &str) {
    let room_id: i64 = conn
        .query_row(
            "SELECT id FROM rooms WHERE name = ?1",
            [room_name],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            eprintln!("Error: Room '{}' not found.", room_name);
            std::process::exit(1);
        });

    let mut stmt = conn
        .prepare(
            "SELECT u.username FROM users u
             JOIN user_rooms ur ON u.id = ur.user_id
             WHERE ur.room_id = ?1
             ORDER BY u.username",
        )
        .expect("Failed to query room users");
    let users: Vec<String> = stmt
        .query_map([room_id], |row| row.get(0))
        .expect("Failed to read users")
        .filter_map(|r| r.ok())
        .collect();

    if users.is_empty() {
        println!("Room '{}' has no users.", room_name);
    } else {
        println!("Users in room '{}':", room_name);
        for (i, user) in users.iter().enumerate() {
            println!("  {}. {}", i + 1, user);
        }
    }
}

fn cmd_user_rooms(conn: &rusqlite::Connection, username: &str) {
    let user_id: i64 = conn
        .query_row(
            "SELECT id FROM users WHERE username = ?1",
            [username],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            eprintln!("Error: User '{}' not found.", username);
            std::process::exit(1);
        });

    let mut stmt = conn
        .prepare(
            "SELECT r.name FROM rooms r
             JOIN user_rooms ur ON r.id = ur.room_id
             WHERE ur.user_id = ?1
             ORDER BY r.name",
        )
        .expect("Failed to query user rooms");
    let rooms: Vec<String> = stmt
        .query_map([user_id], |row| row.get(0))
        .expect("Failed to read rooms")
        .filter_map(|r| r.ok())
        .collect();

    if rooms.is_empty() {
        println!("User '{}' has no rooms.", username);
    } else {
        println!("Rooms for user '{}':", username);
        for (i, room) in rooms.iter().enumerate() {
            println!("  {}. {}", i + 1, room);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: rustui-management <command> [args...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  room:create <room_name>");
    eprintln!("  room:delete <room_name>");
    eprintln!("  room:list");
    eprintln!("  room:users <room_name>");
    eprintln!("  user:create <username> <password>");
    eprintln!("  user:delete <username>");
    eprintln!("  user:list");
    eprintln!("  user:rooms <username>");
    eprintln!("  user:add-room <username> <room_name>");
    eprintln!("  user:remove-room <username> <room_name>");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let conn = open_db();
    let cmd = &args[1];

    match cmd.as_str() {
        "room:create" => {
            if args.len() != 3 {
                eprintln!("Usage: room:create <room_name>");
                std::process::exit(1);
            }
            cmd_room_create(&conn, &args[2]);
        }
        "room:delete" => {
            if args.len() != 3 {
                eprintln!("Usage: room:delete <room_name>");
                std::process::exit(1);
            }
            cmd_room_delete(&conn, &args[2]);
        }
        "room:list" => {
            cmd_room_list(&conn);
        }
        "room:users" => {
            if args.len() != 3 {
                eprintln!("Usage: room:users <room_name>");
                std::process::exit(1);
            }
            cmd_room_users(&conn, &args[2]);
        }
        "user:create" => {
            if args.len() != 4 {
                eprintln!("Usage: user:create <username> <password>");
                std::process::exit(1);
            }
            cmd_user_create(&conn, &args[2], &args[3]);
        }
        "user:delete" => {
            if args.len() != 3 {
                eprintln!("Usage: user:delete <username>");
                std::process::exit(1);
            }
            cmd_user_delete(&conn, &args[2]);
        }
        "user:list" => {
            cmd_user_list(&conn);
        }
        "user:rooms" => {
            if args.len() != 3 {
                eprintln!("Usage: user:rooms <username>");
                std::process::exit(1);
            }
            cmd_user_rooms(&conn, &args[2]);
        }
        "user:add-room" => {
            if args.len() != 4 {
                eprintln!("Usage: user:add-room <username> <room_name>");
                std::process::exit(1);
            }
            cmd_user_add_room(&conn, &args[2], &args[3]);
        }
        "user:remove-room" => {
            if args.len() != 4 {
                eprintln!("Usage: user:remove-room <username> <room_name>");
                std::process::exit(1);
            }
            cmd_user_remove_room(&conn, &args[2], &args[3]);
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", cmd);
            print_usage();
            std::process::exit(1);
        }
    }
}
