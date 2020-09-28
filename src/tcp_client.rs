use crate::common::*;
use anyhow::Result;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use futures::stream::{FuturesUnordered, StreamExt};
use std::thread;

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
  let mut tasks = FuturesUnordered::new();
  for addr in addrs.into_iter() {
    let t = spawn_and_log_error(client(addr, tx.clone()));
    tasks.push(t);
  }
  loop {
    match tasks.next().await {
      Some(_result) => {}
      None => {
        println!("Done!");
        break;
      }
    }
  }
  Ok(())
}

async fn client(addr: String, tx: SyncSender<Vec<u8>>) -> Result<()> {
  let mut stream = TcpStream::connect(addr).await?;
  println!("Connected to {}", &stream.peer_addr()?);
  loop {
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    println!("-> {}", String::from_utf8_lossy(&buf[..n]));
    tx.send(buf[..n].to_vec()).unwrap();
  }
}
