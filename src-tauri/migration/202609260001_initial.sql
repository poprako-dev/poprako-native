CREATE TABLE comic (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL CHECK(length(title) > 0),
    subtitle TEXT NOT NULL,
    author TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE TABLE page (
    id TEXT PRIMARY KEY NOT NULL,
    comic_id TEXT NOT NULL REFERENCES comic(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK(position >= 0),
    unit_revision INTEGER NOT NULL DEFAULT 0 CHECK(unit_revision BETWEEN 0 AND 9007199254740991),
    image_reference TEXT NOT NULL,
    image_original_name TEXT NOT NULL,
    image_format TEXT NOT NULL CHECK(image_format IN ('jpeg', 'png', 'webp', 'bmp')),
    image_width INTEGER NOT NULL CHECK(image_width > 0),
    image_height INTEGER NOT NULL CHECK(image_height > 0),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(comic_id, position)
) STRICT;

CREATE TABLE unit (
    id TEXT PRIMARY KEY NOT NULL,
    page_id TEXT NOT NULL REFERENCES page(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK(position >= 0),
    x_coord REAL NOT NULL CHECK(x_coord BETWEEN 0.0 AND 1.0),
    y_coord REAL NOT NULL CHECK(y_coord BETWEEN 0.0 AND 1.0),
    is_bubble INTEGER NOT NULL CHECK(is_bubble IN (0, 1)),
    is_flagged INTEGER NOT NULL CHECK(is_flagged IN (0, 1)),
    translated_text TEXT NOT NULL,
    proofread_text TEXT NOT NULL,
    is_proofread INTEGER NOT NULL CHECK(is_proofread IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(page_id, position)
) STRICT;

CREATE TABLE work_position (
    comic_id TEXT PRIMARY KEY NOT NULL REFERENCES comic(id) ON DELETE CASCADE,
    last_opened_at INTEGER NOT NULL,
    page_id TEXT REFERENCES page(id) ON DELETE SET NULL,
    unit_id TEXT REFERENCES unit(id) ON DELETE SET NULL,
    mode TEXT NOT NULL CHECK(mode IN ('translation', 'proofreading', 'readonly'))
) STRICT;

CREATE TABLE application_preference (
    singleton INTEGER PRIMARY KEY NOT NULL CHECK(singleton = 1),
    payload TEXT NOT NULL
) STRICT;
