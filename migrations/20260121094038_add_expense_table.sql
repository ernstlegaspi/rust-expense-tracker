-- Add migration script here

CREATE TABLE expense (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amount NUMERIC,
    description TEXT,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES category(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    payment_method VARCHAR(50),
    is_recurring BOOLEAN NOT NULL DEFAULT false,
    tags VARCHAR(100)[]
);

CREATE INDEX idx_expense_user_id ON expense(user_id);
CREATE INDEX idx_expense_category_id ON expense(category_id);
