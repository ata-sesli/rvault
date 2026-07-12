use rvault_core::{DatabaseError, storage::{Database, Table}};

#[test]
fn table_constructor_signature_remains_compatible_after_migrations() {
    let _: fn(&Database, Option<String>) -> Result<Table, DatabaseError> = Table::new;
}
