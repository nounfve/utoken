-- Add up migration script here
create TABLE utokens (
    access VARCHAR(60) PRIMARY KEY,
    access_expire TIMESTAMPTZ,
    refresh VARCHAR(60) UNIQUE,
    refresh_expire TIMESTAMPTZ,
    claim TEXT
);

create UNIQUE INDEX refresh_index on utokens (refresh);