use bcrypt::{hash, verify, DEFAULT_COST};

pub fn encode(password: impl Into<String>) -> String {
    hash(password.into(), DEFAULT_COST).unwrap()
}

pub fn verify_(password: impl Into<String>, encoded_password: impl Into<String>) -> bool {
    let var_name = encoded_password.into();
    let var_name = var_name.replace("{bcrypt}", "");
    verify(password.into(), &var_name).unwrap()
}
