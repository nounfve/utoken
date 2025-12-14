-- Add up migration script here
create TABLE utokens (
    access VARCHAR(60) PRIMARY KEY,
    access_expire TIMESTAMPTZ,
    refresh VARCHAR(60) UNIQUE,
    refresh_expire TIMESTAMPTZ,
    scope VARCHAR(60),
    claim TEXT
);

create UNIQUE INDEX refresh_index on utokens (refresh);
create INDEX scope_index on utokens (scope);