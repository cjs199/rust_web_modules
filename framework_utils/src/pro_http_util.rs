use reqwest::header::HeaderMap;
use reqwest::multipart;
use reqwest::{Error, Response};
use serde_json::Value;
use std::collections::HashMap;

// 异步发送 HTTP 请求并解析响应为一个字符串键值对的哈希表。
//
// # 参数
// - `url`：可以转换为字符串的类型，表示要发送请求的 URL。
//
// # 返回值
// - 如果请求成功并成功解析响应为哈希表，则返回一个包含字符串键值对的`HashMap`。
// - 如果在请求过程中发生错误，则返回`reqwest::Error`类型的错误。
pub async fn get_to_json<T: Into<String>>(url: T) -> Result<HashMap<String, Value>, Error> {
    let resp: HashMap<String, Value> = reqwest::get(url.into())
        .await?
        .json::<HashMap<String, Value>>()
        .await?;
    Ok(resp)
}

pub async fn get_to_str<T: Into<String>>(url: T) -> Result<String, Error> {
    let resp: String = reqwest::get(url.into()).await?.text().await?;
    Ok(resp)
}

pub async fn post_by_json_to_str<T: Into<String>>(
    url: T,
    body: HashMap<String, Value>,
) -> Result<String, Error> {
    let resp: String = reqwest::Client::new()
        .post(url.into())
        .json(&body)
        .send()
        .await?
        .text()
        .await?;
    Ok(resp)
}

pub async fn print_response_type(url: &str) -> Result<Response, Error> {
    let resp = reqwest::get(url).await?;
    Ok(resp)
}

pub async fn post_by_form_add_headers_to_str<T: Into<String>>(
    url: T,
    form: multipart::Form,
    headers: HeaderMap,
) -> Result<String, Error> {
    let resp = reqwest::Client::new()
        .post(url.into())
        .multipart(form)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;
    Ok(resp)
}

