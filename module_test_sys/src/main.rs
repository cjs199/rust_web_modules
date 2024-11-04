pub mod control;
pub mod entities;
pub mod job;
pub mod mq;

use std::collections::HashMap;

use axum::middleware;
use axum::routing::get;
use axum::Router;
use control::test_control::TestControl;
use framework_base_web::config::init_config;
use framework_base_web::config::layer_util;
use framework_macro::add_route;
use framework_utils::pro_json_util;
use job::test_job::TestJobQue;
use mq::test_mq::TestMqQue;
use std::env;

#[tokio::main]
async fn main() {
    // 激活系统基础
    framework_base_web::init_base_web().await;

    // redis初始化
    framework_redis::init_redis();

    // 初始化路由
    init_route();

    // 初始化定时任务模块
    init_job();

    init_mq();

    // 启动服务器
    let app = get_app();

    // 提取路由信息,包含是get还是post方法,请求地址等等
    let route_map = get_route();

    // 合并路由信息,包含是get还是post方法,请求地址等等
    init_config::init_server(app, route_map).await;
}

#[add_route]
fn add_route() -> Router {}

fn get_app() -> Router {
    // 普通路由接口
    let ordinary_route = ordinary_route();
    // 匿名接口
    let anonymous_route = anonymous_route();
    // 合并匿名路由
    let app = ordinary_route.merge(anonymous_route);
    app
}

// 初始化路由
fn init_route() {
    // redis初始化
    TestControl::init_control();
}

// 初始化定时任务模块
fn init_job() {
    TestJobQue::init_job();
}

// 初始化定时任务模块
fn init_mq() {
    TestMqQue::init_mq();
}
