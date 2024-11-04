use std::{
    sync::{Arc, Mutex},
    thread,
};

use tokio::{runtime::Builder, time::{interval_at, Duration, Instant}};


// 定义一个名为TimerTask的结构体，用于管理定时任务
pub struct TimerTask {
    // 用于表示定时任务是否正在运行的状态，通过Arc<Mutex<bool>>来实现线程安全的访问和修改
    is_running: Arc<Mutex<bool>>,
}

impl TimerTask {
    // 构造函数，用于创建一个新的TimerTask实例
    pub fn new() -> Self {
        TimerTask {
            // 初始化is_running字段，将其内部的布尔值设置为false，表示定时任务初始时未运行
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    // 启动定时任务的函数
    pub fn start<Fut>(&self, interval_millis: i64, task: impl Fn() -> Fut + std::marker::Send + 'static)
    where
        Fut: std::future::Future,
    {
        // 将传入的以毫秒为单位的间隔时间转换为无符号64位整数类型，以符合后续使用的要求
        let interval_millis = interval_millis as u64;

        // 克隆is_running字段，以便在新创建的线程中能够安全地访问和修改这个共享的运行状态变量
        let is_running = self.is_running.clone();

        // 获取is_running的锁，并将其值设置为true，标记定时任务开始运行
        *is_running.lock().unwrap() = true;

        // 创建一个新的线程来执行定时任务相关的异步操作
        thread::spawn(move || {
            // 构建一个Tokio的多线程运行时环境，启用所有可选的功能特性
            let rt = Builder::new_multi_thread()
              .enable_all()
              .build()
              .unwrap();

            // 在构建好的Tokio运行时环境中执行异步任务
            rt.block_on(async {
                // 创建一个时间间隔对象，用于按照指定的间隔时间触发任务执行
                // 它基于当前时间（Instant::now()）加上指定的间隔时间来初始化
                let mut interval = interval_at(
                    Instant::now() + Duration::from_millis(interval_millis),
                    Duration::from_millis(interval_millis),
                );

                // 只要is_running的值为true，就会不断循环执行以下操作
                while *is_running.lock().unwrap() {
                    // 等待下一个间隔时间点的到来
                    interval.tick().await;

                    // 调用传入的task闭包并等待其异步执行完成
                    task().await;
                }
            });
        });
    }

    // 停止正在运行的定时任务的函数
    pub fn stop(&self) {
        // 获取is_running的锁，并将其值设置为false，这样在start函数中的循环条件就不满足了，从而停止定时任务的循环执行
        *self.is_running.lock().unwrap() = false;
    }
}