use framework_utils::{futures_util, once_get::OnceGet};
use lazy_static::lazy_static;
use log::info;
use sqlx::{mysql::MySqlPoolOptions, Pool};
use std::{env, time::Duration};

lazy_static! {
    pub static ref DB_ONCE_LOCK: OnceGet<Pool<sqlx::MySql>> = OnceGet::new(init);
}

pub fn init() -> Pool<sqlx::MySql> {
    // 异步转同步
    futures_util::exec(async {
        // 初始化连接
        info!("开始初始化数据库连接");
        let db_url = env::var("db_url").expect("db url,初始化db失败");
        let pool = MySqlPoolOptions::new()
            .min_connections(2)
            .max_connections(30)
            .max_lifetime(Duration::from_millis(1800000))
            .connect(&db_url)
            .await
            .unwrap();
        info!("初始化数据库连接成功");
        return pool;
    })
}
