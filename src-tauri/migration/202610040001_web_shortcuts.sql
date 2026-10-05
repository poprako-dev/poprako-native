-- Track data upgrades separately so customized shortcuts are never remigrated.
CREATE TABLE application_preference_upgrade (
    name TEXT PRIMARY KEY NOT NULL
);
