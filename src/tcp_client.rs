use crate::common::*;
use crate::messages::*;
use anyhow::Result;
use async_std::{io::BufReader, net::TcpStream, prelude::*, task};
use futures::{channel::mpsc, select, FutureExt, SinkExt};
use std::{
  collections::{
    hash_map::{Entry, HashMap},
    HashSet,
  },
  thread,
  time::Duration,
};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
type SyncSender<T> = std::sync::mpsc::Sender<T>;
type SyncReceiver<T> = std::sync::mpsc::Receiver<T>;

#[derive(Debug)]
struct OneSubscribeConfig {
  pub addr: String,
  pub topic: String,
}

#[derive(Debug)]
pub struct RawMessage {
  pub source: String,
  pub topic: String,
  pub data: Vec<u8>,
}

#[derive(Debug)]
enum EventUser {
  AddNewClient(String),
  ClientSubscribeTopic(OneSubscribeConfig),
  ClientUnSubscribeTopic(OneSubscribeConfig),
}

pub struct Subscriber {
  tx_event: Sender<EventUser>,
  rx_msg: SyncReceiver<RawMessage>,
}

impl Subscriber {
  pub fn new() -> Subscriber {
    let (tx_event, rx_event) = mpsc::unbounded();
    let (tx_msg, rx_msg) = std::sync::mpsc::channel();
    thread::spawn(move || {
      if let Err(err) = task::block_on(run_multi_clients(rx_event, tx_msg)) {
        error!("err = {}", err);
      }
    });
    Subscriber { tx_event, rx_msg }
  }

  pub fn connect(&mut self, addr: &str) {
    task::block_on(async {
      self
        .tx_event
        .send(EventUser::AddNewClient(addr.to_string()))
        .await
        .unwrap();
    });
  }

  pub fn subscribe(&mut self, addr: &str, topic: &str) {
    task::block_on(async {
      self
        .tx_event
        .send(EventUser::ClientSubscribeTopic(OneSubscribeConfig {
          addr: addr.to_string(),
          topic: topic.to_string(),
        }))
        .await
        .unwrap();
    });
  }

  pub fn unsubscribe(&mut self, addr: &str, topic: &str) {
    task::block_on(async {
      self
        .tx_event
        .send(EventUser::ClientUnSubscribeTopic(OneSubscribeConfig {
          addr: addr.to_string(),
          topic: topic.to_string(),
        }))
        .await
        .unwrap();
    });
  }

  pub fn recv(&mut self) -> Result<RawMessage> {
    let data = self.rx_msg.recv()?;
    Ok(data)
  }
}

async fn run_multi_clients(
  mut rx_event: Receiver<EventUser>,
  tx_msg: SyncSender<RawMessage>,
) -> Result<()> {
  let mut peers: HashMap<String, Sender<EventUser>> = HashMap::new();

  loop {
    select! {
      event = rx_event.next().fuse() => match event {
        Some(event) => match event {
          EventUser::AddNewClient(addr) => match peers.entry(addr.clone()) {
            Entry::Occupied(..) => {}
            Entry::Vacant(entry) => {
              println!("new connection to {}", addr);
              let (tx_client_event, rx_client_event) = mpsc::unbounded();
              entry.insert(tx_client_event);
              spawn_and_log_error(client(addr, rx_client_event, tx_msg.clone()));
            }
          },
          EventUser::ClientSubscribeTopic(ref config) => {
            if let Some(tx_client_event) = peers.get_mut(&config.addr) {
              tx_client_event.send(event).await.unwrap();
            }
          },
          EventUser::ClientUnSubscribeTopic(ref config) => {
            if let Some(tx_client_event) = peers.get_mut(&config.addr) {
              tx_client_event.send(event).await.unwrap();
            }
          }
        },
        None => break,
      },
    }
  }
  Ok(())
}

async fn client(
  addr: String,
  mut rx_event: Receiver<EventUser>,
  tx_msg: SyncSender<RawMessage>,
) -> Result<()> {
  let mut topics = HashSet::new();

  loop {
    match TcpStream::connect(&addr).await {
      Ok(stream) => {
        debug!("Connected to {}", &stream.peer_addr()?);
        let (reader, mut writer) = (&stream, &stream);
        let reader = BufReader::new(reader);
        let mut lines_from_server = StreamExt::fuse(reader.lines());

        if !topics.is_empty() {
          let msg = MsgSubscribe::encode(topics.clone().into_iter().collect());
          writer.write_all(&msg).await?;
        }

        loop {
          select! {
            event = rx_event.next().fuse() => match event {
              Some(event) => match event {
                EventUser::ClientSubscribeTopic(config) => {
                  topics.insert(config.topic.clone());
                  if !topics.is_empty() {
                    let msg = MsgSubscribe::encode(vec![config.topic]);
                    writer.write_all(&msg).await?;
                  }
                },
                EventUser::ClientUnSubscribeTopic(config) => {
                  topics.remove(&config.topic);
                  let msg = MsgUnSubscribe::encode(vec![config.topic]);
                  writer.write_all(&msg).await?;
                }
                _ => break,
              },
              None => break,
            },
            line = lines_from_server.next().fuse() => match line {
              Some(line) => {
                let line = line?;
                let msg = MsgPublish::decode(&line);
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
        error!("connect to {} failed, err = {}", addr, err);
        task::sleep(Duration::from_secs(5)).await;
        continue;
      }
    }
  }
}
