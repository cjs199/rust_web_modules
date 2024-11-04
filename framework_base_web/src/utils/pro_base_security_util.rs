use framework_utils::{pro_aes_gcm_util, pro_base64_util, pro_constant_pool_util, pro_json_util};
use log::info;

use crate::{config::layer_util::LOGIN_INFO_THREAD_LOCAL, dto::login_info_dto::LoginInfoDto};

// 加密密钥和初始化向量（Nonce），用于AES-GCM加密
const KEY: &str = "QRo5nSm+Pg/8zaNdoXHOpCFV3V+X90ACC/yCMQfJias=";
const NONCE: &str = "If6QbzHVTTnpSfHy";

/// 将登录用户信息（LoginInfoDto）转换为token
///
/// 1. 将LoginInfoDto对象序列化为JSON字符串。
/// 2. 使用AES-256-GCM算法，以KEY和NONCE为参数，对JSON字符串进行加密。
/// 3. 将加密后的数据进行Base64编码，得到最终的token字符串。
pub fn login_info_dto_to_token(login_info_dto: LoginInfoDto) -> String {
    let to_string = pro_json_util::object_to_str(&login_info_dto);
    let encrypt = pro_aes_gcm_util::aes_256_gcm_encrypt(KEY, NONCE, to_string);
    pro_base64_util::byte_encode_str(&encrypt)
}

/// 将token转换为LoginInfoDto对象
///
/// 1. 对token进行Base64解码。
/// 2. 使用AES-256-GCM算法，以KEY和NONCE为参数，对解码后的数据进行解密。
/// 3. 将解密后的JSON字符串反序列化为LoginInfoDto对象。
pub fn token_to_login_info_dto(authorization: impl Into<String>) -> Option<LoginInfoDto> {
    let str_decode_byte_result = pro_base64_util::str_decode_byte(authorization);
    match str_decode_byte_result {
        Ok(str_decode_byte) => {
            let aes_256_gcm_decrypt =
                pro_aes_gcm_util::aes_256_gcm_decrypt(KEY, NONCE, str_decode_byte);
            match aes_256_gcm_decrypt {
                Ok(json) => {
                    let login_info_dto: LoginInfoDto = pro_json_util::str_to_object(&json).unwrap();
                    Some(login_info_dto)
                }
                Err(_) => {
                    info!("json转登录对象出错");
                    None
                }
            }
        }
        Err(_) => {
            info!("提取json对象出错");
            None
        }
    }
}

/// 从线程局部存储中获取登录用户信息
pub fn get_login_info_dto() -> Option<LoginInfoDto> {
    LOGIN_INFO_THREAD_LOCAL.with(|local| {
        let value = local.lock().unwrap();
        if let Some(ref authorization) = *value {
            token_to_login_info_dto(authorization)
        } else {
            info!("没有从线程局部存储中获取到用户信息");
            None
        }
    })
}

/// 获取登录用户的ID
///
/// 如果用户已登录，则返回用户的ID；否则返回系统默认的ID。
pub fn get_login_user_id() -> i64 {
    let get_login_info_dto_option = get_login_info_dto();
    match get_login_info_dto_option {
        Some(get_login_info_dto) => get_login_info_dto.uid,
        // 如果没有提取到id,则直接返回系统id
        None => pro_constant_pool_util::SYSTEM_ID,
    }
}