use axum::body::Body;
use axum::http::HeaderValue;

use axum::{http::StatusCode, response::Response};
use framework_utils::exception_enum::ProException;
use framework_utils::pro_json_util;

pub fn return_err(exp_enum: ProException) -> Result<Response, StatusCode> {
    let response_body = Body::from(pro_json_util::object_to_str(&exp_enum));
    let mut response = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(response_body)
        .unwrap();
    response.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static("application/json;charset=UTF-8"),
    );
    Ok(response)
}
