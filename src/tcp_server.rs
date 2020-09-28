use crate::common::*;
use crate::messages::*;
use anyhow::Result;
use async_std::{
  io::BufReader,
  net::{TcpListener, TcpStream, ToSocketAddrs},
  prelude::*,
  task,
};
use futures::{channel::mpsc, select, FutureExt, SinkExt};
use std::{
  collections::{
    hash_map::{Entry, HashMap},
    HashSet,
  },
  sync::Arc,
  thread,
};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[derive(Debug)]
enum Void {}

pub struct Publisher {
  tx: Sender<EventServer>,
}

impl Publisher {
  pub fn publish(&mut self, data: Vec<u8>) {
    task::block_on(async {
      self
        .tx
        .send(EventServer::PublishAll { data })
        .await
        .unwrap()
    });
  }

  pub fn publish_topic(&mut self, topic: &str, data: Vec<u8>) {
    task::block_on(async {
      self
        .tx
        .send(EventServer::Publish {
          topic: topic.to_string(),
          data,
        })
        .await
        .unwrap()
    });
  }
}

pub fn start_tcp_server(addr: String) -> Publisher {
  let (tx_new_data, rx_new_data) = mpsc::unbounded();

  thread::spawn(move || {
    if let Err(err) = task::block_on(accept_loop(addr, rx_new_data)) {
      error!("err = {}", err);
    }
  });

  Publisher { tx: tx_new_data }
}

async fn accept_loop(addr: impl ToSocketAddrs, rx_new_data: Receiver<EventServer>) -> Result<()> {
  let listener = TcpListener::bind(addr).await?;
  info!("Listening on {}", listener.local_addr()?);

  let mut uniq_client_id: usize = 0;
  let (broker_sender, broker_receiver) = mpsc::unbounded();
  let broker = task::spawn(broker_loop(broker_receiver, rx_new_data));
  let mut incoming = listener.incoming();
  while let Some(stream) = incoming.next().await {
    let stream = stream?;
    info!("Accepting from: {}", stream.peer_addr()?);
    uniq_client_id += 1;
    spawn_and_log_error(connection_loop(
      uniq_client_id,
      broker_sender.clone(),
      stream,
    ));
  }
  drop(broker_sender);
  broker.await;

  Ok(())
}

async fn connection_loop(
  client_id: usize,
  mut broker: Sender<EventClient>,
  stream: TcpStream,
) -> Result<()> {
  let stream = Arc::new(stream);
  let reader = BufReader::new(&*stream);
  let mut lines = reader.lines();

  let (_shutdown_sender, shutdown_receiver) = mpsc::unbounded::<Void>();
  broker
    .send(EventClient::NewPeer {
      id: client_id,
      stream: Arc::clone(&stream),
      shutdown: shutdown_receiver,
    })
    .await?;

  while let Some(line) = lines.next().await {
    let line = line?;
    debug!("[{}]: {}", client_id, line);

    let msg: MsgHeader = match serde_json::from_str(&line) {
      Ok(msg) => msg,
      Err(err) => {
        error!("parse json failed, err = {:?}", err);
        continue;
      }
    };

    match msg.msgtype.as_str() {
      MSG_TYPE_SUBSCRIBE => {
        let msg: MsgSubscribe = match serde_json::from_str(&line) {
          Ok(msg) => msg,
          Err(err) => {
            error!("parse json failed, err = {:?}", err);
            continue;
          }
        };
        broker
          .send(EventClient::Subscribe {
            id: client_id,
            topics: msg.topics,
          })
          .await?;
      }
      MSG_TYPE_SUBSCRIBE_ALL => {
        broker
          .send(EventClient::SubscribeAll { id: client_id })
          .await?;
      }
      &_ => {}
    }
  }

  Ok(())
}

async fn connection_writer_loop(
  messages: &mut Receiver<Vec<u8>>,
  stream: Arc<TcpStream>,
  mut shutdown: Receiver<Void>,
) -> Result<()> {
  let mut stream = &*stream;
  loop {
    select! {
      msg = messages.next().fuse() => match msg {
        Some(msg) => stream.write_all(&msg).await?,
        None => break,
      },
      void = shutdown.next().fuse() => match void {
        Some(void) => match void {},
        None => break,
      }
    }
  }
  Ok(())
}

#[derive(Debug)]
enum EventClient {
  NewPeer {
    id: usize,
    stream: Arc<TcpStream>,
    shutdown: Receiver<Void>,
  },
  Subscribe {
    id: usize,
    topics: Vec<String>,
  },
  SubscribeAll {
    id: usize,
  },
}

#[derive(Debug)]
enum EventServer {
  Publish { topic: String, data: Vec<u8> },
  PublishAll { data: Vec<u8> },
}

async fn broker_loop(mut events: Receiver<EventClient>, mut rx_new_data: Receiver<EventServer>) {
  // <client_id, send out>
  let mut peers: HashMap<usize, Sender<Vec<u8>>> = HashMap::new();

  // <topic, set<client_id>>
  let mut t2c: HashMap<String, HashSet<usize>> = HashMap::new();
  let mut c2t: HashMap<usize, HashSet<String>> = HashMap::new();

  // subscribe all topic's client's id
  let mut suball: HashSet<usize> = HashSet::new();

  // <client_id, send out>
  let (disconnect_sender, mut disconnect_receiver) =
    mpsc::unbounded::<(usize, Receiver<Vec<u8>>)>();

  loop {
    select! {
      event = events.next().fuse() => match event {
        Some(event) => {
          match event {
            EventClient::NewPeer {
              id,
              stream,
              shutdown,
            } => match peers.entry(id) {
              Entry::Occupied(..) => {}
              Entry::Vacant(entry) => {
                let (client_sender, mut client_receiver) = mpsc::unbounded();
                entry.insert(client_sender);
                let mut disconnect_sender = disconnect_sender.clone();
                spawn_and_log_error(async move {
                  let res = connection_writer_loop(&mut client_receiver, stream, shutdown).await;
                  disconnect_sender.send((id, client_receiver)).await.unwrap();
                  res
                });
              }
            },
            EventClient::Subscribe { id, topics } => {
              if !suball.contains(&id) {
                for topic in topics {
                  t2c.entry(topic.clone()).or_insert_with(|| HashSet::new()).insert(id);
                  c2t.entry(id).or_insert_with(|| HashSet::new()).insert(topic);
                }
              }
            },
            EventClient::SubscribeAll { id } => {
                debug!("[{}]: subscribe all topics", id);
                suball.insert(id);
            }
          }
        },
        None => break,
      },
      event = rx_new_data.next().fuse() => match event {
        Some(event) => match event {
          EventServer::Publish { topic, mut data } => {
            debug!("publish message, topic = {}", topic);
            data.push(b'\n');
            if let Some(client_ids) = t2c.get_mut(&topic) {
              for client_id in client_ids.iter() {
                if let Some(client) = peers.get_mut(&client_id) {
                  client.send(data.clone()).await.unwrap();
                }
              }
            }
            for id in suball.iter() {
              if let Some(client) = peers.get_mut(&id) {
                client.send(data.clone()).await.unwrap();
              }
            }
          }
          EventServer::PublishAll { mut data } => {
            debug!("publish all");
            data.push(b'\n');
            for id in suball.iter() {
              if let Some(client) = peers.get_mut(&id) {
                client.send(data.clone()).await.unwrap();
              }
            }
          }
        }
        None => break,
      },
      disconnect = disconnect_receiver.next().fuse() => {
        let (id, _pending_messages) = disconnect.unwrap();
        assert!(peers.remove(&id).is_some());
        suball.remove(&id);
        if let Some(topics) = c2t.get_mut(&id) {
          for topic in topics.iter() {
            if let Some(client_ids) = t2c.get_mut(topic) {
              client_ids.remove(&id);
            }
          }
          c2t.remove(&id);
        }
        continue;
      },
    }
  }
  drop(peers);
  drop(disconnect_sender);
  while let Some((_name, _pending_messages)) = disconnect_receiver.next().await {}
}
