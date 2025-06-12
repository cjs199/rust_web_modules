use std::future::Future;

use lazy_static::lazy_static;
use tokio::{runtime::Runtime, task::JoinHandle};

lazy_static! {
    // pub static ref THREAD_POOL: Runtime = Runtime::new().unwrap();
    pub static ref THREAD_POOL: Runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(10) // 设置线程池中的线程数量为 10
    .enable_all()
    .build()
    .unwrap();

}

/**
 * 使用线程执行任务
 */
pub fn thread<F, R>(func: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    THREAD_POOL.spawn_blocking(func)
}

/**
 * 使用协程执行任务
 */
pub fn fiber<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    THREAD_POOL.spawn(future)
}
