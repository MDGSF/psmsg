use crate::common::*;
use crate::messages::*;
use anyhow::Result;
use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use futures::future;
use futures::{select, FutureExt};
use serde_json;
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
      Ok(stream) => {
        println!("Connected to {}", &stream.peer_addr()?);
        let (reader, mut writer) = (&stream, &stream);
        let reader = BufReader::new(reader);
        let mut lines_from_server = StreamExt::fuse(reader.lines());

        let msg = MsgSubscribeAll {
          version: "1.0.0".to_string(),
          msgtype: MSG_TYPE_SUBSCRIBE_ALL.to_string(),
        };
        let subscribe_msg = serde_json::to_string(&msg)?;
        writer.write_all(subscribe_msg.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        println!("Subscribe to all topics");

        loop {
          select! {
            line = lines_from_server.next().fuse() => match line {
              Some(line) => {
                let line = line?;
                println!("{}", line);
                tx.send(line.as_bytes().to_vec()).unwrap();
              }
              None => break,
            },
          }
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
