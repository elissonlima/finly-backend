-- Definition of main database

CREATE TABLE user (
    id INTEGER PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password TEXT,
    created_at TEXT NOT NULL,
    auth_type TEXT NOT NULL,
    google_user_id TEXT,
    is_email_verified INTEGER NOT NULL, -- 0 for false 1 for true
    is_premium INTEGER NOT NULL -- 0 for false 1 for true
);
CREATE INDEX idx_user_email ON user (email);

CREATE TABLE reset_password (
    id TEXT PRIMARY KEY NOT NULL,
    user_email TEXT NOT NULL,
    sent_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    is_password_reset INTEGER NOT NULL
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_email TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    refresh_token_expires_at TEXT NOT NULL,
    current_access_token TEXT NOT NULL,
    current_access_token_expires_at TEXT NOT NULL,
    FOREIGN KEY(user_email) REFERENCES user(email)
);

CREATE TABLE category (
    id TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    icon_name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES user(id)
);

CREATE TABLE subcategory (
    id TEXT PRIMARY KEY NOT NULL,
    category_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    icon_name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(category_id) REFERENCES category(id)
);


