use axum::body::Body;
use axum::{http::StatusCode, response::Response};
use framework_utils::exception_enum::ProException;
use framework_utils::pro_json_util;

pub fn return_err(exp_enum: ProException) -> Result<Response, StatusCode> {
    let response_body = Body::from(pro_json_util::object_to_str(&exp_enum));
    let response = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header( "Content-Type", "application/json;charset=UTF-8")
        .header("access-control-allow-headers" , "*")
        .header("access-control-allow-origin" , "*")
        .body(response_body)
        .unwrap();
    Ok(response)
}

