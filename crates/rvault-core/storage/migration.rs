use crate::error::DatabaseError;
use rusqlite::{Connection, Transaction};

pub(super) fn migrate(
    connection: &Connection,
    table_name: &str,
) -> Result<(), DatabaseError> {
    let transaction = connection.unchecked_transaction()?;
    let mut version: i64 =
        transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version < 1 {
        migrate_0_to_1(&transaction, table_name)?;
        version = 1;
    }
    if version < 2 {
        migrate_1_to_2(&transaction, table_name)?;
        version = 2;
    }
    if version < 3 {
        migrate_2_to_3(&transaction, table_name)?;
        version = 3;
    }
    if version > 3 {
        return Err(DatabaseError::Sqlite(rusqlite::Error::InvalidQuery));
    }
    transaction.commit()?;
    Ok(())
}

fn migrate_0_to_1(transaction: &Transaction<'_>, table_name: &str) -> rusqlite::Result<()> {
    add_column_if_missing(
        transaction,
        table_name,
        "pinned",
        "BOOLEAN DEFAULT FALSE",
    )?;
    transaction.pragma_update(None, "user_version", 1)
}

fn migrate_1_to_2(transaction: &Transaction<'_>, table_name: &str) -> rusqlite::Result<()> {
    add_column_if_missing(
        transaction,
        table_name,
        "created_at",
        "INTEGER DEFAULT 0",
    )?;
    transaction.pragma_update(None, "user_version", 2)
}

fn migrate_2_to_3(transaction: &Transaction<'_>, table_name: &str) -> rusqlite::Result<()> {
    add_column_if_missing(
        transaction,
        table_name,
        "updated_at",
        "INTEGER DEFAULT 0",
    )?;
    transaction.pragma_update(None, "user_version", 3)
}

fn add_column_if_missing(
    transaction: &Transaction<'_>,
    table_name: &str,
    column_name: &str,
    definition: &str,
) -> rusqlite::Result<()> {
    if table_has_column(transaction, table_name, column_name)? {
        return Ok(());
    }
    transaction.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {definition}"),
        [],
    )?;
    Ok(())
}

fn table_has_column(
    transaction: &Transaction<'_>,
    table_name: &str,
    column_name: &str,
) -> rusqlite::Result<bool> {
    let mut statement = transaction.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        if row.get::<_, String>(1)? == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns(connection: &Connection) -> Vec<String> {
        let mut statement = connection.prepare("PRAGMA table_info(main)").unwrap();
        statement
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn assert_migrates(schema: &str) {
        let connection = Connection::open_in_memory().unwrap();
        if !schema.is_empty() {
            connection.execute_batch(schema).unwrap();
        }
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS main (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    platform TEXT NOT NULL,
                    user_id TEXT NOT NULL,
                    password TEXT NOT NULL,
                    nonce TEXT,
                    salt TEXT,
                    pinned BOOLEAN DEFAULT FALSE,
                    created_at INTEGER DEFAULT 0,
                    updated_at INTEGER DEFAULT 0,
                    UNIQUE(platform, user_id)
                )",
            )
            .unwrap();
        migrate(&connection, "main").unwrap();
        let columns = columns(&connection);
        for expected in ["pinned", "created_at", "updated_at"] {
            assert!(columns.iter().any(|column| column == expected));
        }
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 3);
        if schema.contains("INSERT INTO") {
            let value: String = connection
                .query_row("SELECT password FROM main", [], |row| row.get(0))
                .unwrap();
            assert_eq!(value, "secret");
        }
    }

    #[test]
    fn migrates_supported_schema_fixtures_to_version_three() {
        assert_migrates("");
        assert_migrates(
            "CREATE TABLE main (
                id INTEGER PRIMARY KEY, platform TEXT NOT NULL, user_id TEXT NOT NULL,
                password TEXT NOT NULL, nonce TEXT, salt TEXT
            ); INSERT INTO main VALUES (1, 'example', 'user', 'secret', NULL, NULL);",
        );
        assert_migrates(
            "CREATE TABLE main (
                id INTEGER PRIMARY KEY, platform TEXT NOT NULL, user_id TEXT NOT NULL,
                password TEXT NOT NULL, nonce TEXT, salt TEXT, pinned BOOLEAN DEFAULT FALSE
            ); INSERT INTO main VALUES (1, 'example', 'user', 'secret', NULL, NULL, FALSE);",
        );
        assert_migrates(
            "CREATE TABLE main (
                id INTEGER PRIMARY KEY, platform TEXT NOT NULL, user_id TEXT NOT NULL,
                password TEXT NOT NULL, nonce TEXT, salt TEXT, pinned BOOLEAN DEFAULT FALSE,
                created_at INTEGER DEFAULT 0, updated_at INTEGER DEFAULT 0
            ); INSERT INTO main VALUES (1, 'example', 'user', 'secret', NULL, NULL, FALSE, 0, 0);",
        );
    }

    #[test]
    fn failed_migration_rolls_back_schema_and_version() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE VIEW main AS SELECT 1 AS id")
            .unwrap();
        assert!(migrate(&connection, "main").is_err());
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 0);
        assert_eq!(columns(&connection), vec!["id"]);
    }
}
