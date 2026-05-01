create TABLE utokens (
    access UUID UNIQUE,
    access_expire TIMESTAMPTZ,
    refresh UUID PRIMARY KEY,
    refresh_expire TIMESTAMPTZ,
    scope VARCHAR(200),
    claim TEXT,
    child_of UUID REFERENCES utokens(refresh) ON DELETE CASCADE ON UPDATE CASCADE
);
-- 
create UNIQUE INDEX refresh_index on utokens (refresh);
create INDEX scope_index on utokens (scope);