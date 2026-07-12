use rvault_core::session::{get_key_from_session, start_session};

#[test]
fn legacy_session_signatures_remain_unchanged() {
    let _: fn(&[u8]) -> Result<String, std::io::Error> = start_session;
    let _: fn() -> Result<Vec<u8>, String> = get_key_from_session;
}
