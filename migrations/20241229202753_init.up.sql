-- Definition of main database

CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password TEXT,
    created_at TEXT NOT NULL,
    auth_type TEXT NOT NULL,
    is_email_verified INTEGER NOT NULL, -- 0 for false 1 for true
    is_premium INTEGER NOT NULL -- 0 for false 1 for true
);
CREATE INDEX idx_user_email ON user (email);

CREATE TABLE reset_password (
    id TEXT PRIMARY KEY,
    user_email TEXT NOT NULL,
    sent_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    is_password_reset INTEGER NOT NULL
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_email TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    refresh_token_expires_at TEXT NOT NULL,
    current_access_token TEXT NOT NULL,
    current_access_token_expires_at TEXT NOT NULL
);
