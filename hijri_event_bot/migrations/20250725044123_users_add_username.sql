-- Add migration script here

ALTER TABLE users
ADD COLUMN username VARCHAR(500) NULL;
