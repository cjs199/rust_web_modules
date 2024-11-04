use std::future::Future;
use tokio::task::JoinHandle;

pub fn minthread<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    return tokio::spawn(future);
}
