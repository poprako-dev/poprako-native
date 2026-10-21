CREATE TABLE termbase (
    id TEXT PRIMARY KEY NOT NULL,
    scope TEXT NOT NULL CHECK (scope IN ('global', 'comic')),
    comic_id TEXT REFERENCES comic(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(name) > 0),
    name_key TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK ((scope = 'global' AND comic_id IS NULL) OR (scope = 'comic' AND comic_id IS NOT NULL))
) STRICT;

CREATE UNIQUE INDEX termbase_global_name ON termbase(name_key) WHERE scope = 'global';
CREATE UNIQUE INDEX termbase_comic_name ON termbase(comic_id, name_key) WHERE scope = 'comic';
CREATE INDEX termbase_scope_order ON termbase(scope, comic_id, updated_at DESC, id);

CREATE TABLE term (
    id TEXT PRIMARY KEY NOT NULL,
    termbase_id TEXT NOT NULL REFERENCES termbase(id) ON DELETE CASCADE,
    source TEXT NOT NULL CHECK (length(source) > 0),
    source_key TEXT NOT NULL,
    targets TEXT NOT NULL CHECK (json_valid(targets) AND json_type(targets) = 'array' AND json_array_length(targets) > 0),
    comment TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (termbase_id, source_key)
) STRICT;

CREATE INDEX term_order ON term(termbase_id, updated_at DESC, id);
