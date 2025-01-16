CREATE TABLE session (
    id TEXT PRIMARY KEY,
    user_email TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    refresh_token_expires_at TEXT NOT NULL,
    current_access_token TEXT NOT NULL,
    current_access_token_expires_at TEXT NOT NULL
);
