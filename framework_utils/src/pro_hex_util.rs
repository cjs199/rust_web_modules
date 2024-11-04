use hex::FromHexError;


pub fn byte_encode_str(data: &[u8]) -> String {
    hex::encode(data)
}

pub fn str_decode_byte(data: impl Into<String>) -> Result<Vec<u8>, FromHexError> {
    hex::decode(&data.into())
}