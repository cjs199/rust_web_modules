use axum::{http::Method, routing::get, Json, Router};
use dotenv::dotenv;
use framework_utils::{pro_constant_pool_util, pro_str_util};
use log::info;
use std::{collections::HashMap, env};
use tower_http::cors::{Any, CorsLayer};
use crate::dto::log_dto::LogDto;
use crate::job::log_job::LogJob;
use crate::job::snowflake_job::SnowflakeJob;
use crate::utils::pro_snowflake_util;
use chrono::Utc;
use env_logger::{Builder, Env};
use std::fs::File;
use std::vec::Vec;
use std::io::{prelude::*, BufWriter};
use std::sync::{Arc, LazyLock, Mutex};

thread_local!(pub static TRACE_ID_THREAD_LOCAL: Mutex<Option<i64>> = Mutex::new(None));
pub static LOG_CACHE: Mutex<Vec<LogDto>> = Mutex::new(Vec::new());

pub static LOG_FILE_WRITER: LazyLock<Arc<Mutex<BufWriter<File>>>> = LazyLock::new(|| {
    let logging_app_file = env::var("logging_app_file").unwrap_or("app.log".to_string());
    let file = match File::options()
        .append(true)
        .create(true)
        .open(logging_app_file)
    {
        Ok(file) => file,
        Err(error) => {
            eprintln!("创建日志文件失败: {}", error);
            std::process::exit(1);
        }
    };
    Arc::new(Mutex::new(BufWriter::new(file)))
});

// 初始化服务器使用的各种工具
pub fn init_env() {
    // 初始化配置文件
    dotenv().ok();

    // 定义服务器名称
    let app_name = env::var("application_name").unwrap_or("".to_string());
    unsafe { pro_constant_pool_util::APP_NAME = Some(app_name.clone()) };

    let logging_pattern = env::var("logging_pattern")
        .unwrap_or("{time} {level}\t {logger} -{trace_id}- {msg}".to_string());

    // 初始化日志配置
    
    Builder::from_env(Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"))
        .format(move |buf, record| {
            let trace_id = TRACE_ID_THREAD_LOCAL.with(|local| {
                let mut local_data = local.lock().unwrap();
                if local_data.is_none() {
                    *local_data = Some(pro_snowflake_util::next_id()); // 设置一个默认值，你可以根据实际情况修改
                }
                local_data.unwrap()
            });

            let now = Utc::now();

            let time = now.format("%Y-%m-%d %H:%M:%S.%3f").to_string();
            let logger = format!("{}:{}", record.file().unwrap(), record.line().unwrap());
            let level = record.level();
            let msg = record.args();
            let mut map = HashMap::new();
            let level = level.to_string();
            let trace_id = trace_id.to_string();
            map.insert("time".to_string(), time);
            map.insert("level".to_string(), level.clone());
            map.insert("logger".to_string(), logger);
            map.insert("trace_id".to_string(), trace_id.clone());
            map.insert("msg".to_string(), msg.to_string());
            let message = pro_str_util::format(logging_pattern.clone(), &map);
            LOG_CACHE.lock().unwrap().push(LogDto {
                service_name: app_name.clone(),
                log_msg: message.clone(),
                time: Utc::now(),
                level: level,
                trace_id: trace_id,
            });
            writeln!(buf, "{}", message)
        })
        .is_test(false)
        .init();

    // 激活日志写入文件
    LogJob::init_job();
}

// 启动服务器
pub async fn init_server(app: Router, route_map: Vec<HashMap<String, String>>) {
    // 初始化雪花算法定时任务
    SnowflakeJob::init_job();

    // 增加一个接口,返回路由信息,包含是get还是post方法,请求地址等等
    let app = app.merge(Router::new().route("/get_api", get(|| async { Json(route_map) })));
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        // 允许一切来源
        .allow_origin(Any)
        .allow_headers(Any);

    let app = app.layer(cors); 

    // 服务器启动
    let server_url = env::var("server_address").expect("db url,初始化db失败");
    let listener = tokio::net::TcpListener::bind(server_url.clone())
        .await
        .unwrap();
    info!("服务器启动成功,地址{}", server_url);
    axum::serve(listener, app).await.unwrap();
}
