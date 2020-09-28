use crate::common::*;
use anyhow::Result;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use futures::future;
use std::thread;
use std::time::Duration;

type SyncSender<T> = std::sync::mpsc::Sender<T>;
type SyncReceiver<T> = std::sync::mpsc::Receiver<T>;

pub fn start_tcp_client(addrs: Vec<String>) -> SyncReceiver<Vec<u8>> {
  let (tx, rx) = std::sync::mpsc::channel();
  thread::spawn(move || {
    if let Err(err) = task::block_on(runtime(addrs, tx)) {
      println!("err = {}", err);
    }
  });
  rx
}

async fn runtime(addrs: Vec<String>, tx: SyncSender<Vec<u8>>) -> Result<()> {
  let mut tasks = Vec::new();
  for addr in addrs.into_iter() {
    let t = spawn_and_log_error(client(addr, tx.clone()));
    tasks.push(t);
  }
  future::join_all(tasks).await;
  Ok(())
}

async fn client(addr: String, tx: SyncSender<Vec<u8>>) -> Result<()> {
  loop {
    match TcpStream::connect(&addr).await {
      Ok(mut stream) => {
        println!("Connected to {}", &stream.peer_addr()?);
        loop {
          let mut buf = vec![0u8; 1024];
          let n = stream.read(&mut buf).await?;
          if n == 0 {
            println!("EOF");
            break;
          }
          tx.send(buf[..n].to_vec()).unwrap();
        }
      }
      Err(err) => {
        println!("err = {}", err);
        task::sleep(Duration::from_secs(5)).await;
        continue;
      }
    }
  }
}
