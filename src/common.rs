use anyhow::Result;

use async_std::prelude::*;
use async_std::task;

pub fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
  F: Future<Output = Result<()>> + Send + 'static,
{
  task::spawn(async move {
    if let Err(err) = fut.await {
      eprintln!("{}", err);
    }
  })
}
