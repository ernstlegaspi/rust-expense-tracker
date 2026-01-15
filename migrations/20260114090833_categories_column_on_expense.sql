-- Add migration script here
ALTER TABLE expense ADD COLUMN category_id INTEGER NOT NULL REFERENCES category(id) ON DELETE CASCADE;
