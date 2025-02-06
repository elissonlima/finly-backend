-- Definition of main database
CREATE TYPE auth_type AS ENUM('USERNAMEPASSWORD', 'GOOGLE');


CREATE TABLE "user" (
    id SERIAL PRIMARY KEY,
    email VARCHAR(320) NOT NULL UNIQUE,
    name VARCHAR(50) NOT NULL,
    password VARCHAR(72),
    created_at TIMESTAMP NOT NULL DEFAULT(now() at time zone 'utc'),
    auth_type auth_type NOT NULL,
    google_user_id VARCHAR(255),
    is_email_verified BOOLEAN NOT NULL DEFAULT false,
    is_premium BOOLEAN NOT NULL DEFAULT false
);
CREATE INDEX idx_user_email ON "user" (email);

CREATE TABLE reset_password (
    id UUID PRIMARY KEY,
    user_email VARCHAR(320) NOT NULL,
    sent_at TIMESTAMP NOT NULL DEFAULT(now() at time zone 'utc'),
    expires_at TIMESTAMP NOT NULL,
    is_password_reset BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_email VARCHAR(320) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT(now() at time zone 'utc'),
    refresh_token VARCHAR(1000) NOT NULL,
    refresh_token_expires_at TIMESTAMP NOT NULL,
    current_access_token VARCHAR(1000) NOT NULL,
    current_access_token_expires_at TIMESTAMP NOT NULL,
    FOREIGN KEY(user_email) REFERENCES "user"(email)
);

CREATE TABLE category (
    id UUID PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name VARCHAR(50) NOT NULL,
    color VARCHAR(9) NOT NULL,
    icon_name VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT(now() at time zone 'utc'),
    updated_at TIMESTAMP NOT NULL DEFAULT(now() at time zone 'utc'),
    FOREIGN KEY(user_id) REFERENCES "user"(id)
);
CREATE UNIQUE INDEX category_id_user_id_idx ON category(id, user_id);


CREATE TABLE subcategory (
    id UUID PRIMARY KEY,
    category_id UUID NOT NULL,
    name VARCHAR(50) NOT NULL,
    color VARCHAR(9) NOT NULL,
    icon_name VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT(now() at time zone 'utc'),
    updated_at TIMESTAMP NOT NULL DEFAULT(now() at time zone 'utc'),
    FOREIGN KEY(category_id) REFERENCES category(id)
);
CREATE UNIQUE INDEX subcategory_id_category_id_idx ON subcategory(id, category_id);

