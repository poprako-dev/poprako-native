-- Reject an unexpected legacy theme before changing the original payload.
CREATE TEMP TABLE preference_theme_guard (
    valid INTEGER NOT NULL CHECK(valid = 1)
);

INSERT INTO preference_theme_guard (valid)
SELECT CASE
    WHEN json_type(payload, '$.theme') = 'text'
        AND json_extract(payload, '$.theme') IN ('system', 'light', 'dark') THEN 1
    ELSE 0
END
FROM application_preference;

UPDATE application_preference SET payload = json_remove(payload, '$.theme');

DROP TABLE preference_theme_guard;
