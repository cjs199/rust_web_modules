use axum::body::Body;
use axum::http::Method;
use axum::middleware::Next;
use axum::{
    http::{Request, StatusCode},
    response::Response,
};
use chrono::Utc;
use framework_redis::utils::pro_redis_mq_msg_util;
use framework_utils::{pro_constant_pool_util, pro_time_util};
use crate::dto::request_record_dto::RequestRecordDto;
use crate::utils::pro_base_security_util;

// 请求记录
pub async fn middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let get_ip = get_ip(&req);
    let path = req.uri().path().to_string();
    let begin = pro_time_util::get_current_milliseconds();
    let response = next.run(req).await;
    if !Method::OPTIONS.eq(&method) {
        let end = pro_time_util::get_current_milliseconds();
        let get_login_user_id = pro_base_security_util::get_login_user_id();
        let status = response.status().as_u16();
        let app_name = pro_constant_pool_util::APP_NAME.try_read().unwrap().clone();
        let request_record_dto = RequestRecordDto{
            time: Utc::now(),
            user_id: get_login_user_id,
            request_uri: path,
            ip: get_ip,
            exec_time: (end - begin) as i32,
            app_name: app_name,
            state: status as i32,
        };
        pro_redis_mq_msg_util::put_msg_que("request_record", request_record_dto);
    }

    return Ok(response);
}

fn get_ip(req: &Request<Body>) -> String {
    let headers_to_check = [
        "X-Forwarded-For",
        "X-Real-IP",
        "Proxy-Client-IP",
        "WL-Proxy-Client-IP",
        "HTTP_CLIENT_IP",
        "HTTP_X_FORWARDED_FOR",
    ];
    let headers = req.headers();
    for header_name in headers_to_check {
        if let Some(header_value) = headers.get(header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                return ip_str.to_string();
            }
        }
    }
    return "0.0.0.0".to_string();
}
