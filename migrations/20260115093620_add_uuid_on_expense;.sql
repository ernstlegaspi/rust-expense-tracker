-- Add migration script here
ALTER TABLE expense ADD COLUMN uuid UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE;
