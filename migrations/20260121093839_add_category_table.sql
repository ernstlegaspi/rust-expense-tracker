-- Add migration script here

CREATE TABLE category (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    description TEXT
);

CREATE UNIQUE INDEX idx_unique_category_per_user ON category (user_id, LOWER(name))
