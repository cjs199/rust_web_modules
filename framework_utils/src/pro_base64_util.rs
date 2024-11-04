use base64::{engine::general_purpose, Engine as _};

pub fn byte_encode_str(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

pub fn str_decode_byte(data: impl Into<String>) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(data.into())
}