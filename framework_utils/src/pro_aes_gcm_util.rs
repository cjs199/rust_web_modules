
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};

use crate::{exception_enum::ProException, pro_base64_util};

pub fn aes_256_gcm_encrypt(
    key: impl Into<String>,
    nonce: impl Into<String>,
    content: impl Into<String>,
) -> Vec<u8> {
    let key = pro_base64_util::str_decode_byte(key.into()).unwrap();
    let nonce = pro_base64_util::str_decode_byte(nonce.into()).unwrap();
    let content = content.into();
    let key: &aes_gcm::aead::generic_array::GenericArray<u8, _> =
        Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce);
    cipher.encrypt(nonce, content.as_ref()).unwrap()
}

pub fn aes_256_gcm_decrypt(
    key: impl Into<String>,
    nonce: impl Into<String>,
    encrypt: Vec<u8>,
) -> Result<String, ProException> {
    let key = pro_base64_util::str_decode_byte(key.into()).unwrap();
    let nonce = pro_base64_util::str_decode_byte(nonce.into()).unwrap();
    let key: &aes_gcm::aead::generic_array::GenericArray<u8, _> =
        Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce);
    let decrypt_result = cipher.decrypt(nonce, encrypt.as_ref());
    match decrypt_result {
        Ok(decrypt) => {
            let from_utf8_result = String::from_utf8(decrypt);
            match from_utf8_result {
                Ok(ret)=>   Ok(ret),
                Err(_) =>  Err(ProException::解密失败),
            }
        },
        Err(_) =>  Err(ProException::解密失败),
    }
}
