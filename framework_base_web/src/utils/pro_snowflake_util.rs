use idgenerator::*;

pub fn next_id() -> i64 {
    let id = IdInstance::next_id();
    return id;
}

pub fn next_id_str() -> String {
    let id = IdInstance::next_id();
    return id.to_string();
}
