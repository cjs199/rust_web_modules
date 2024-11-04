use async_std::task;
use std::future::Future;

pub fn exec<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,

{
    // 返回的对象,使用时异常卡住
    // let handle = thread::spawn(move || {
    //     let rt = tokio::runtime::Runtime::new().unwrap();
    //     rt.block_on(async {
    //         future.await
    //     })
    // });
    // handle.join().unwrap()
    task::block_on(future)
}
