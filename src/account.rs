mod account {
    
}
pub struct Account {
    platform: String,
    user_id: String,
    passsword: String,
}
trait CanEncrypt {
    fn encrypt_password() -> String;
}
trait CanSave {
    fn save_field();
}
