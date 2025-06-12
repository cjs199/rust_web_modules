use config::init_config;
use utils::pro_sys_dict_util::{self, SysDictUpdatePubMQ};

pub mod base_service;
pub mod config;
pub mod dto;
pub mod job;
pub mod pro_local_cache_util;
pub mod utils;

// 系统基础相关初始化
pub async  fn init_base_web() {
    
    // 激活环境配置文件
    init_config::init_env();

    // 启动时就激活数据库
    base_service::DB_ONCE_LOCK.get();

    // 初始化字典和监听字典更新
    pro_sys_dict_util::update_dict().await;

    // 监听字典更新mq
    SysDictUpdatePubMQ::init_mq();

}
