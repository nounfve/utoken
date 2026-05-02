CREATE TABLE link_bind (
    link UUID PRIMARY KEY,
    path TEXT NOT NULL,
    query TEXT,
    bind_to UUID REFERENCES utokens(refresh)
    /* force break */
    ON DELETE
    SET NULL ON UPDATE CASCADE
);
CREATE INDEX path_index on link_bind(path);