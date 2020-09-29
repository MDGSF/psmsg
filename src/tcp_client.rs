use crate::common::*;
use crate::messages::*;
use anyhow::Result;
use async_std::{io::BufReader, net::TcpStream, prelude::*, task};
use futures::{channel::mpsc, select, FutureExt, SinkExt};
use serde_json;
use std::{thread, time::Duration};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
type SyncSender<T> = std::sync::mpsc::Sender<T>;
type SyncReceiver<T> = std::sync::mpsc::Receiver<T>;

#[derive(Debug)]
pub struct OneSubscribeConfig {
  pub addr: String,
  pub topics: Vec<String>,
}

#[derive(Debug)]
pub struct SubscribeConfigs {
  pub subs: Vec<OneSubscribeConfig>,
}

#[derive(Debug)]
pub struct RawMessage {
  pub source: String,
  pub topic: String,
  pub data: Vec<u8>,
}

#[derive(Debug)]
enum EventUser {
  AddNewClient(OneSubscribeConfig),
}

pub struct Subscriber {
  tx_event: Sender<EventUser>,
  rx_msg: SyncReceiver<RawMessage>,
}

impl Subscriber {
  pub fn subscribe(&mut self, configs: SubscribeConfigs) {
    task::block_on(async {
      for config in configs.subs {
        self
          .tx_event
          .send(EventUser::AddNewClient(config))
          .await
          .unwrap();
      }
    });
  }

  pub fn recv(&mut self) -> Result<RawMessage> {
    let data = self.rx_msg.recv()?;
    Ok(data)
  }
}

pub fn start_tcp_client(addrs: Vec<String>) -> Subscriber {
  let mut subs = Vec::new();
  for addr in addrs {
    let sub = OneSubscribeConfig {
      addr,
      topics: Vec::new(),
    };
    subs.push(sub);
  }
  let configs = SubscribeConfigs { subs };

  let (tx_event, rx_event) = mpsc::unbounded();
  let (tx_msg, rx_msg) = std::sync::mpsc::channel();
  thread::spawn(move || {
    if let Err(err) = task::block_on(run_multi_clients(rx_event, tx_msg)) {
      error!("err = {}", err);
    }
  });

  let mut s = Subscriber { tx_event, rx_msg };
  s.subscribe(configs);
  s
}

pub fn start_tcp_client_with_topics(configs: SubscribeConfigs) -> Subscriber {
  let (tx_event, rx_event) = mpsc::unbounded();
  let (tx_msg, rx_msg) = std::sync::mpsc::channel();
  thread::spawn(move || {
    if let Err(err) = task::block_on(run_multi_clients(rx_event, tx_msg)) {
      error!("err = {}", err);
    }
  });

  let mut s = Subscriber { tx_event, rx_msg };
  s.subscribe(configs);
  s
}

async fn run_multi_clients(
  mut rx_event: Receiver<EventUser>,
  tx_msg: SyncSender<RawMessage>,
) -> Result<()> {
  loop {
    select! {
      event = rx_event.next().fuse() => match event {
        Some(event) => match event {
          EventUser::AddNewClient(config) => {
            spawn_and_log_error(client(config, tx_msg.clone()));
          }
        },
        None => break,
      },
    }
  }
  Ok(())
}

async fn client(sub: OneSubscribeConfig, tx_msg: SyncSender<RawMessage>) -> Result<()> {
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

                let msg: MsgPublish = match serde_json::from_str(&line) {
                  Ok(msg) => msg,
                  Err(err) => {
                    error!("parse json failed, err = {:?}", err);
                    continue;
                  }
                };
                debug!("msg = {:?}", msg);

                tx_msg.send(RawMessage {
                  source: msg.source,
                  topic: msg.topic,
                  data: msg.data,
                }).unwrap();
              }
              None => break,
            },
          }
        }
      }
      Err(err) => {
        error!("connect to {} failed, err = {}", sub.addr, err);
        task::sleep(Duration::from_secs(5)).await;
        continue;
      }
    }
  }
}
