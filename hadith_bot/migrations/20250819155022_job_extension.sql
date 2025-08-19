-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    chat_id BIGINT NOT NULL,
    CONSTRAINT pk_user_id PRIMARY KEY (id),
    CONSTRAINT uq_chat_id UNIQUE (chat_id)
);

CREATE TABLE IF NOT EXISTS job_extensions (
    job_id UUID NOT NULL DEFAULT gen_random_uuid(),
    type INTEGER NOT NULL,
    CONSTRAINT pk_job_extension_id PRIMARY KEY (job_id),
    CONSTRAINT fk_job_id FOREIGN KEY(job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS users_jobs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL,
    user_id UUID NOT NULL,
    CONSTRAINT pk_users_jobs PRIMARY KEY (id),
    CONSTRAINT fk_job_id FOREIGN KEY(job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    CONSTRAINT fk_user_id FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);