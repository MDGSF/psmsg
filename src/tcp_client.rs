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

pub struct OneSubscribe {
  pub addr: String,
  pub topics: Vec<String>,
}

pub struct SubscribeTopics {
  pub subs: Vec<OneSubscribe>,
}

pub fn start_tcp_client(addrs: Vec<String>) -> SyncReceiver<Vec<u8>> {
  let mut subs = Vec::new();
  for addr in addrs {
    let sub = OneSubscribe {
      addr,
      topics: Vec::new(),
    };
    subs.push(sub);
  }
  let config = SubscribeTopics { subs };
  let (tx, rx) = std::sync::mpsc::channel();
  thread::spawn(move || {
    if let Err(err) = task::block_on(run_multi_clients(config, tx)) {
      error!("err = {}", err);
    }
  });
  rx
}

pub fn start_tcp_client_with_topics(config: SubscribeTopics) -> SyncReceiver<Vec<u8>> {
  let (tx, rx) = std::sync::mpsc::channel();
  thread::spawn(move || {
    if let Err(err) = task::block_on(run_multi_clients(config, tx)) {
      error!("err = {}", err);
    }
  });
  rx
}

async fn run_multi_clients(config: SubscribeTopics, tx: SyncSender<Vec<u8>>) -> Result<()> {
  let mut tasks = Vec::new();
  for sub in config.subs.into_iter() {
    let t = spawn_and_log_error(client(sub, tx.clone()));
    tasks.push(t);
  }
  future::join_all(tasks).await;
  Ok(())
}

async fn client(sub: OneSubscribe, tx: SyncSender<Vec<u8>>) -> Result<()> {
  loop {
    match TcpStream::connect(&sub.addr).await {
      Ok(stream) => {
        debug!("Connected to {}", &stream.peer_addr()?);
        let (reader, mut writer) = (&stream, &stream);
        let reader = BufReader::new(reader);
        let mut lines_from_server = StreamExt::fuse(reader.lines());

        if sub.topics.is_empty() {
          let msg = MsgSubscribeAll {
            version: "1.0.0".to_string(),
            msgtype: MSG_TYPE_SUBSCRIBE_ALL.to_string(),
          };
          let subscribe_msg = serde_json::to_string(&msg)?;
          writer.write_all(subscribe_msg.as_bytes()).await?;
          writer.write_all(b"\n").await?;
          debug!("Subscribe to all topics");
        } else {
          let msg = MsgSubscribe {
            version: "1.0.0".to_string(),
            msgtype: MSG_TYPE_SUBSCRIBE.to_string(),
            topics: sub.topics.clone(),
          };
          let subscribe_msg = serde_json::to_string(&msg)?;
          writer.write_all(subscribe_msg.as_bytes()).await?;
          writer.write_all(b"\n").await?;
          debug!("Subscribe to topics: {:?}", sub.topics);
        }

        loop {
          select! {
            line = lines_from_server.next().fuse() => match line {
              Some(line) => {
                let line = line?;
                tx.send(line.as_bytes().to_vec()).unwrap();
              }
              None => break,
            },
          }
        }
      }
      Err(err) => {
        error!("err = {}", err);
        task::sleep(Duration::from_secs(5)).await;
        continue;
      }
    }
  }
}
