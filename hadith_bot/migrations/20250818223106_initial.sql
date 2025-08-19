-- Add migration script here

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

CREATE TABLE IF NOT EXISTS notifications (
    id UUID,
    job_id UUID NOT NULL,
    extra BYTEA,
    CONSTRAINT pk_notification_id PRIMARY KEY (id),
    CONSTRAINT fk_job_id FOREIGN KEY(job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS notification_states (
    id UUID NOT NULL,
    state INTEGER NOT NULL,
    CONSTRAINT pk_notification_states PRIMARY KEY (id, state),
    CONSTRAINT fk_notification_id FOREIGN KEY(id) REFERENCES notifications(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS books (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT pk_book_id PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS hadiths (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    text_bos TEXT NOT NULL,
    text_arabic TEXT NOT NULL,
    transmitters_text TEXT NOT NULL,
    book_id UUID NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT pk_hadith_id PRIMARY KEY (id),
    CONSTRAINT fk_book_id FOREIGN KEY(book_id) REFERENCES books(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS hadith_numbers (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    value INTEGER NOT NULL,
    hadith_id UUID NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT pk_hadith_number_id PRIMARY KEY (id),
    CONSTRAINT fk_hadith_id FOREIGN KEY(hadith_id) REFERENCES hadiths(id) ON DELETE CASCADE
);