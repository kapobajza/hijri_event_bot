-- Add migration script here

ALTER TABLE job_extensions
    DROP CONSTRAINT fk_user_id;

ALTER TABLE job_extensions
    DROP COLUMN user_id;

CREATE TABLE IF NOT EXISTS users_jobs (
    id UUID NOT NULL,
    job_id UUID NOT NULL,
    user_id UUID NOT NULL,
    CONSTRAINT pk_users_jobs PRIMARY KEY (id),
    CONSTRAINT fk_job_id FOREIGN KEY(job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    CONSTRAINT fk_user_id FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

