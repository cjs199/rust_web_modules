use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{Error, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

use crate::pro_base64_util;


/**
 * 生成一个rsa的公钥私钥
 */
pub fn generate_keypair(length: usize) -> Result<(RsaPublicKey, RsaPrivateKey), Error> {
    let mut rng = rand::thread_rng();
    let pri_key = RsaPrivateKey::new(&mut rng, length)?;
    let pub_key = pri_key.to_public_key();
    return Ok((pub_key, pri_key));
}

pub fn pub_key_encrypt_str_ret_str(pub_key: String, data: String) -> String {
    let mut rng = rand::thread_rng();
    // 从字节中解析公钥
    let pub_key = RsaPublicKey::from_public_key_pem(&pub_key).unwrap();
    // 公钥加密
    let data = data.bytes().collect::<Vec<u8>>();
    let encrypt = pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, &data).unwrap();
    return pro_base64_util::byte_encode_str(&encrypt);
}

pub fn pri_key_decrypt_str_ret_str(pri_key: String, data: String) -> String {
    // 从字节中解析私钥
    let pri_key = RsaPrivateKey::from_pkcs8_pem(&pri_key).unwrap();
    // 私钥解密
    let data = pro_base64_util::str_decode_byte(&data).unwrap();
    let decrypt = pri_key.decrypt(Pkcs1v15Encrypt, &data).unwrap();
    return String::from_utf8(decrypt).unwrap();
}
