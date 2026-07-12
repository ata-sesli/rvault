use rusqlite::{Connection, backup::Backup};
use std::{fs, path::Path, time::Duration};

pub(super) fn snapshot_database(source_path: &Path) -> Result<Vec<u8>, String> {
    let snapshot_path = std::env::temp_dir().join(format!(
        "rvault-snapshot-{}-{}.sqlite",
        std::process::id(),
        rand::random::<u64>()
    ));
    let snapshot_result = (|| {
        let source = Connection::open(source_path)
            .map_err(|error| format!("open database snapshot source: {error}"))?;
        let mut destination = Connection::open(&snapshot_path)
            .map_err(|error| format!("open database snapshot destination: {error}"))?;
        {
            let backup = Backup::new(&source, &mut destination)
                .map_err(|error| format!("start database snapshot: {error}"))?;
            backup
                .run_to_completion(16, Duration::from_millis(10), None)
                .map_err(|error| format!("copy database snapshot: {error}"))?;
        }
        drop(destination);
        drop(source);
        fs::read(&snapshot_path).map_err(|error| format!("read database snapshot: {error}"))
    })();
    let cleanup_result = if snapshot_path.exists() {
        fs::remove_file(&snapshot_path)
            .map_err(|error| format!("remove database snapshot: {error}"))
    } else {
        Ok(())
    };
    match (snapshot_result, cleanup_result) {
        (Ok(bytes), Ok(())) => Ok(bytes),
        (Ok(_), Err(cleanup_error)) => Err(cleanup_error),
        (Err(primary_error), Ok(())) => Err(primary_error),
        (Err(primary_error), Err(cleanup_error)) => {
            Err(format!("{primary_error}; cleanup failed: {cleanup_error}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn snapshot_includes_committed_wal_rows() {
        let root =
            std::env::temp_dir().join(format!("rvault-backup-test-{}", rand::random::<u64>()));
        fs::create_dir_all(&root).unwrap();
        let source_path = root.join("source.sqlite");
        let source = Connection::open(&source_path).unwrap();
        source.pragma_update(None, "journal_mode", "WAL").unwrap();
        source
            .execute("CREATE TABLE entries (value TEXT NOT NULL)", [])
            .unwrap();
        source
            .execute("INSERT INTO entries (value) VALUES ('committed')", [])
            .unwrap();

        let bytes = snapshot_database(&source_path).unwrap();
        let snapshot_path = root.join("snapshot.sqlite");
        fs::write(&snapshot_path, bytes).unwrap();
        let snapshot = Connection::open(snapshot_path).unwrap();
        let value: String = snapshot
            .query_row("SELECT value FROM entries", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "committed");

        drop(snapshot);
        drop(source);
        fs::remove_dir_all(root).unwrap();
    }
}
