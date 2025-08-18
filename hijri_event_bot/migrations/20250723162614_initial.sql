-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    id UUID NOT NULL,
    chat_id BIGINT NOT NULL,
    CONSTRAINT pk_user_id PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS jobs (
    id UUID,
    last_updated BIGINT,
    next_tick BIGINT,
    last_tick BIGINT,
    job_type INTEGER NOT NULL,
    count INTEGER,
    ran BOOL,
    stopped BOOL,
    time_offset_seconds INTEGER,
    schedule TEXT,
    repeating BOOL,
    repeated_every INTEGER,
    extra BYTEA,
    CONSTRAINT pk_job_id PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS job_extensions (
    job_id UUID,
    user_id UUID NOT NULL,
    CONSTRAINT fk_user_id FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT pk_job_extension_id PRIMARY KEY (job_id),
    CONSTRAINT fk_job_id FOREIGN KEY(job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS notifications (
    id UUID,
    job_id UUID,
    extra BYTEA,
    CONSTRAINT pk_notification_id PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS notification_states (
    id UUID NOT NULL,
    state INTEGER NOT NULL,
    CONSTRAINT pk_notification_states PRIMARY KEY (id, state),
    CONSTRAINT fk_notification_id FOREIGN KEY(id) REFERENCES notifications(id) ON DELETE CASCADE
);