CREATE TABLE settings (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    channel TEXT,
    user_refresh_token TEXT,
    tree TEXT,
    zoom_factor REAL
);
