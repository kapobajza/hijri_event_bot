-- Add migration script here

ALTER TABLE job_extensions
ADD COLUMN type INTEGER NOT NULL;
