-- Add migration script here
ALTER TABLE users ADD COLUMN email TEXT UNIQUE;
