CREATE TABLE actions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    script BLOB NOT NULL,
    config BLOB NOT NULL
);

CREATE TABLE kv_store (
    bucket TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (bucket, key)
);
CREATE INDEX kv_store_bucket_idx ON kv_store(bucket);

CREATE TABLE settings (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    zoom_factor REAL,
    tree BLOB,
    channel TEXT,
    user_access_token TEXT,
    user_refresh_token TEXT
);
