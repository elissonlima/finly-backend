
-- Definition of main database

CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password TEXT NOT NULL,
    created_at TEXT NOT NULL,
    auth_type TEXT NOT NULL,
    is_email_verified INTEGER NOT NULL, -- 0 for false 1 for true
    is_premium INTEGER NOT NULL -- 0 for false 1 for true
);

