use arboard::Clipboard;

pub fn copy_text(text: String){
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(text);
}