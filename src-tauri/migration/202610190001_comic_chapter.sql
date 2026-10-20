-- Copy before replacing tables; SQLx executes this migration in one transaction.
CREATE TEMP TABLE previous_comic AS SELECT *, lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)), 2) || '-' || substr('89ab', (random() & 3) + 1, 1) || substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6))) AS parent_id FROM comic;
CREATE TEMP TABLE previous_page AS SELECT * FROM page;
CREATE TEMP TABLE previous_unit AS SELECT * FROM unit;
CREATE TEMP TABLE previous_work_position AS SELECT * FROM work_position;
DROP TABLE work_position;
DROP TABLE unit;
DROP TABLE page;
DROP TABLE comic;

CREATE TABLE comic (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL CHECK(length(title) > 0),
    author TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE chapter (
    id TEXT PRIMARY KEY NOT NULL,
    comic_id TEXT NOT NULL REFERENCES comic(id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK(length(title) > 0),
    position INTEGER NOT NULL CHECK(position >= 0),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(comic_id, position)
) STRICT;

CREATE TABLE page (
    id TEXT PRIMARY KEY NOT NULL,
    chapter_id TEXT NOT NULL REFERENCES chapter(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK(position >= 0),
    unit_revision INTEGER NOT NULL DEFAULT 0 CHECK(unit_revision BETWEEN 0 AND 9007199254740991),
    image_reference TEXT NOT NULL,
    image_original_name TEXT NOT NULL,
    image_format TEXT NOT NULL CHECK(image_format IN ('jpeg', 'png', 'webp', 'bmp')),
    image_width INTEGER NOT NULL CHECK(image_width > 0),
    image_height INTEGER NOT NULL CHECK(image_height > 0),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(chapter_id, position)
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
    chapter_id TEXT PRIMARY KEY NOT NULL REFERENCES chapter(id) ON DELETE CASCADE,
    last_opened_at INTEGER NOT NULL,
    page_id TEXT REFERENCES page(id) ON DELETE SET NULL,
    unit_id TEXT REFERENCES unit(id) ON DELETE SET NULL,
    mode TEXT NOT NULL CHECK(mode IN ('translation', 'proofreading', 'readonly'))
) STRICT;


INSERT INTO comic SELECT parent_id, title, author, created_at, updated_at FROM previous_comic;
INSERT INTO chapter SELECT id, parent_id, CASE WHEN subtitle = '' THEN '第1章' ELSE subtitle END, 0, created_at, updated_at FROM previous_comic;
INSERT INTO page SELECT * FROM previous_page;
INSERT INTO unit SELECT * FROM previous_unit;
INSERT INTO work_position SELECT * FROM previous_work_position;
DROP TABLE previous_work_position;
DROP TABLE previous_unit;
DROP TABLE previous_page;
DROP TABLE previous_comic;
