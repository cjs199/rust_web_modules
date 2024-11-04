use std::sync::Mutex;

use axum::body::Body;
use axum::middleware::Next;

use axum::{
    http::{Request, StatusCode},
    response::Response,
};
use framework_redis::utils::pro_redis_util;
use framework_utils::exception_enum::ProException;
use framework_utils::{pro_constant_pool_util, pro_time_util};

use crate::pro_local_cache_util;
use crate::utils::{exp_util, pro_base_security_util};

// thread_local!(static MY_THREAD_LOCAL: RefCell<Option<String>> = RefCell::new(None));
// 在 Rust 中，当线程退出时，thread_local! 声明的静态变量会自动被清理。
// 如果你想在特定的地方手动清理资源，可以这样做：
// fn clean_thread_local() {
//     MY_THREAD_LOCAL.with(|local| {
//         *local.borrow_mut() = None;
//     });
// }
// 在上面的代码中，clean_thread_local 函数可以用来手动将线程局部存储的值设置为 None，从而清理资源。
// 需要注意的是，通常情况下不需要手动清理 thread_local! 的资源，因为它们会在合适的时候自动清理。只有在特殊情况下，比如需要明确控制资源的释放时机，才考虑手动清理。
thread_local!(pub static LOGIN_INFO_THREAD_LOCAL: Mutex<Option<String>> = Mutex::new(None));

// LOGIN_INFO_THREAD_LOCAL.with(|local| {
//     let mut local_data = local.borrow_mut();
//     if local_data.is_none() {
//         *local_data =
//             Some(authorization.to_string());
//     }
// });

// 登录授权中间件函数，用于验证请求中的 Authorization 头部信息
pub async fn login_authorization(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // 获取请求中的 Authorization 头部信息
    let auth_header = req.headers().get(pro_constant_pool_util::AUTHORIZATION);
    if let Some(header_value) = auth_header {
        // 将头部信息转换为字符串
        let authorization_result = header_value.to_str();
        match authorization_result {
            // 如果转换成功
            Ok(authorization) => {
                let authorization_key = format!("login_authorization:{}", authorization);
                let cache_authorization_key: Option<String> = pro_local_cache_util::cache_read(&authorization_key);
                match cache_authorization_key {
                    Some(_) => {
                        LOGIN_INFO_THREAD_LOCAL.with(|local| {
                            let mut local_data = local.lock().unwrap();
                            if local_data.is_none() {
                                *local_data = Some(authorization.to_string());
                            }
                        });
                        return Ok(next.run(req).await);
                    }
                    None => {
                        let get_login_info_dto =
                            pro_base_security_util::token_to_login_info_dto(authorization);
                        match get_login_info_dto {
                            Some(login_info_dto) => {
                                let get_current_seconds =
                                    pro_time_util::get_current_seconds();
                                let exp = login_info_dto.exp;
                                if get_current_seconds < exp {
                                    let aid = login_info_dto.aid;
                                    let uid = login_info_dto.uid;
                                    let redis_aid_option = pro_local_cache_util::cache_read_by_fn(
                                        &authorization_key,
                                        async {
                                            let return_redis_aid_option: Option<String> =
                                                pro_redis_util::map_get(
                                                    pro_constant_pool_util::ONLINE_USERS,
                                                    uid,
                                                ).await;
                                            return return_redis_aid_option;
                                        },
                                        pro_time_util::Millisecond::_1_MINUTE,
                                    )
                                    .await;
                                    match redis_aid_option {
                                        Some(redis_aid) => {
                                            // 使用包含判断,和java服务哪里兼容,java服务用json包含了一层
                                            // "123".contains(&123)
                                            if redis_aid.contains(&aid) {
                                                LOGIN_INFO_THREAD_LOCAL.with(|local| {
                                                    let mut local_data = local.lock().unwrap();
                                                    if local_data.is_none() {
                                                        *local_data = Some(authorization.to_string());
                                                    }
                                                });
                                                return Ok(next.run(req).await);
                                            } else {
                                                return exp_util::return_err(
                                                    ProException::签名已过期,
                                                );
                                            }
                                        }
                                        None => {
                                            return exp_util::return_err(
                                                ProException::签名已过期,
                                            )
                                        }
                                    }
                                } else {
                                    // 密钥已过期
                                    return exp_util::return_err(ProException::签名已过期);
                                }
                            }
                            None => return exp_util::return_err(ProException::请登录),
                        };
                    }
                }
            }
            // 如果转换失败
            Err(_) => return exp_util::return_err(ProException::请登录),
        }
    }
    // 如果没有 Authorization 头部信息，打印校验失败信息并返回错误状态码
    exp_util::return_err(ProException::请登录)
}
