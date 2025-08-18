-- Add migration script here

ALTER TABLE notifications
ALTER COLUMN job_id SET NOT NULL;
