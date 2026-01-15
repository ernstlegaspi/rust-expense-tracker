-- Add migration script here

CREATE TABLE expense (
    id SERIAL PRIMARY KEY,
    amount NUMERIC,
    description TEXT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    payment_method VARCHAR(50),
    is_recurring BOOLEAN NOT NULL DEFAULT false,
    tags VARCHAR(100)[]
);

CREATE INDEX idx_expenses_user_id ON expense(user_id);
