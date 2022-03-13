use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use rusqlite::{Connection, Row};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn create<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let conn = Connection::open(&path).unwrap();

        // Create the repositories table.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS repositories (
                id   INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL
            );",
            [],
        )
        .unwrap();

        // Create the groups table.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS groups (
                id   INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            );",
            [],
        )
        .unwrap();

        // Create the repository-to-group N:M relationship table.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS repositories_groups (
                repository_id INTEGER REFERENCES repositories (id) ON DELETE CASCADE,
                group_id      INTEGER REFERENCES groups (id)       ON DELETE CASCADE,
                UNIQUE (repository_id, group_id)                   ON CONFLICT IGNORE
            );",
            [],
        )
        .unwrap();

        Database {
            conn,
        }
    }

    pub fn add_repository<P>(&self, path: P, group: Option<String>)
    where
        P: AsRef<Path>,
    {
        let path_str = path.as_ref().to_str().unwrap();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO repositories (path) VALUES (?);",
                [path_str],
            )
            .unwrap();

        if let Some(group) = group {
            self.conn
                .execute("INSERT OR IGNORE INTO groups (name) VALUES (?);", [&group])
                .unwrap();

            let repository_id: u64 = self
                .conn
                .query_row(
                    "SELECT id FROM repositories WHERE path = ?",
                    [path_str],
                    |row| row.get(0),
                )
                .unwrap();

            let group_id: u64 = self
                .conn
                .query_row("SELECT id FROM groups WHERE name = ?", [&group], |row| {
                    row.get(0)
                })
                .unwrap();

            self.conn
                .execute(
                    "INSERT INTO repositories_groups (
                    repository_id,
                    group_id
                ) VALUES (?, ?);",
                    [repository_id, group_id],
                )
                .unwrap();
        }
    }

    pub fn get_repositories(&self, group: Option<String>) -> rusqlite::Result<Vec<PathBuf>> {
        let fst = |r: &Row| r.get(0);
        if let Some(group) = group {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT path FROM repositories AS r JOIN repositories_groups AS rg ON r.id = rg.repository_id JOIN groups AS g ON rg.group_id = g.id WHERE g.name = ?;",
                )
                .unwrap();
            let rows = statement.query_map([&group], fst).unwrap();
            rows.map(|res| res.map(|s: String| PathBuf::from_str(&s).unwrap()))
                .collect()
        } else {
            let mut statement = self.conn.prepare("SELECT path FROM repositories;").unwrap();
            let rows = statement.query_map([], fst).unwrap();
            rows.map(|res| res.map(|s: String| PathBuf::from_str(&s).unwrap()))
                .collect()
        }
    }
}
