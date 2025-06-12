use axum::body::Body;
use axum::http::Method;
use axum::middleware::Next;
use axum::{
    http::{Request, StatusCode},
    response::Response,
};
use framework_redis::utils::pro_redis_util;
use framework_utils::exception_enum::ProException;
use framework_utils::{pro_json_util, pro_md5_util, pro_str_util, pro_time_util};
use log::warn;
use serde_json::Value;

use crate::utils::{exp_util, pro_base_security_util};

// 请求记录
pub async fn middleware(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // 获取请求方法
    let method = req.method().clone();
    let response;
    let path = req.uri().path().to_string();
    if ![Method::GET,Method::OPTIONS].contains(&method) {
        let req_id;
        let req_id_header_option = req.headers().get("req-id");
        if let Some(req_id_header) = req_id_header_option {
            req_id = req_id_header.to_str().ok().unwrap().to_string();
        } else {
            let (parts, body) = req.into_parts();
            let bytes = axum::body::to_bytes(body, 2048).await.unwrap();
            // 格式化一下,避免相同的请求,不同的格式导致过滤异常
            let mut body_str = String::from_utf8(bytes.to_vec()).unwrap();
            let get_login_user_id = pro_base_security_util::get_login_user_id();
            if pro_str_util::is_blank(&body_str) {
                warn!("警告,非get请求方法{},请求体是空!", path);
            } else {
                let body_obj: Value = pro_json_util::str_to_object(&body_str).unwrap();
                body_str = pro_json_util::object_to_str(&body_obj);
            }
            let req_id_md5 = pro_md5_util::to_md5_str(path + &body_str);
            req_id = req_id_kv(get_login_user_id, req_id_md5);
            let new_body = Body::from(bytes);
            req = Request::from_parts(parts, new_body);
        }
        let kv_set_if_absent =
            pro_redis_util::kv_set_if_absent(req_id, 0, pro_time_util::Millisecond::_500);
        if !kv_set_if_absent {
            // 没有添加成功,则返回异常
            return exp_util::return_err(ProException::幂等重复);
        }
        response = next.run(req).await;
    } else {
        response = next.run(req).await;
    }
    return Ok(response);
}

// 请求幂等id缓存
pub fn req_id_kv(login_user_id: i64, req_id: impl Into<String>) -> String {
    return format!("req_id_kv:{}:{}", login_user_id, req_id.into());
}
