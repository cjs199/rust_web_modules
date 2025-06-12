use lazy_static::lazy_static;
use tokio::sync::RwLock;

pub const DICT_KEY: &str = "dict_key";

pub const SYSTEM_ID: i64 = 0;

pub const AUTHORIZATION: &str = "Auth";

pub const ONLINE_USERS: &str = "online_users";

pub const EMPTY: &str = "";

pub const EMPTY_STR: String = String::new();

pub const OK: &str = "OK";


lazy_static! {
    pub static ref APP_NAME: RwLock<String> = RwLock::new(String::new());
}
