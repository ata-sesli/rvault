fn main() {
    slint_build::compile("main.slint").unwrap();
    slint_build::compile("unlock_view.slint").unwrap();
    slint_build::compile("setup_view.slint").unwrap();
    slint_build::compile("vault_list_view.slint").unwrap();
    slint_build::compile("session_timeout_view.slint").unwrap();
}