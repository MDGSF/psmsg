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
  tx_event: Sender<EventUser>,
  _tx_shut_down: Sender<()>,
}

impl Publisher {
  pub fn new(name: &str, addr: &str) -> Publisher {
    let (tx_event_user, rx_event_user) = mpsc::unbounded();
    let (tx_shut_down, rx_shut_down) = mpsc::unbounded();
    let name = name.to_string();
    let addr = addr.to_string();
    thread::spawn(move || {
      if let Err(err) = task::block_on(accept_loop(name, addr, rx_event_user, rx_shut_down)) {
        error!("err = {}", err);
      }
    });

    Publisher {
      tx_event: tx_event_user,
      _tx_shut_down: tx_shut_down,
    }
  }

  pub fn publish(&mut self, data: Vec<u8>) {
    task::block_on(async {
      self
        .tx_event
        .send(EventUser::PublishAll { data })
        .await
        .unwrap()
    });
  }

  pub fn publish_topic(&mut self, topic: &str, data: Vec<u8>) {
    task::block_on(async {
      self
        .tx_event
        .send(EventUser::Publish {
          topic: topic.to_string(),
          data,
        })
        .await
        .unwrap()
    });
  }
}

async fn accept_loop(
  name: String,
  addr: impl ToSocketAddrs,
  rx_event_user: Receiver<EventUser>,
  rx_shut_down: Receiver<()>,
) -> Result<()> {
  let listener = TcpListener::bind(addr).await?;
  info!("Listening on {}", listener.local_addr()?);

  let mut uniq_client_id: usize = 0;
  let mut peers: HashMap<usize, Sender<()>> = HashMap::new();

  let (broker_sender, broker_receiver) = mpsc::unbounded();
  let broker = task::spawn(broker_loop(name, broker_receiver, rx_event_user));
  let incoming = listener.incoming();
  let mut incoming = StreamExt::fuse(incoming);
  let mut rx_shut_down = StreamExt::fuse(rx_shut_down);
  loop {
    select! {
      stream = incoming.next().fuse() => match stream {
        Some(stream) => {
          let stream = stream?;
          info!("Accepting from: {}", stream.peer_addr()?);
          uniq_client_id += 1;
          let (tx_conn_shutdown, rx_conn_shutdown) = mpsc::unbounded();
          peers.insert(uniq_client_id, tx_conn_shutdown);
          spawn_and_log_error(connection_loop(
            uniq_client_id,
            broker_sender.clone(),
            stream,
            rx_conn_shutdown,
          ));
        },
        None => break,
      },
      _ = rx_shut_down.next().fuse() => break,
    }
  }
  drop(broker_sender);
  broker.await;

  Ok(())
}

async fn connection_loop(
  client_id: usize,
  mut broker: Sender<EventClient>,
  stream: TcpStream,
  rx_conn_shutdown: Receiver<()>,
) -> Result<()> {
  let stream = Arc::new(stream);
  let reader = BufReader::new(&*stream);
  let mut lines = StreamExt::fuse(reader.lines());
  let mut rx_conn_shutdown = StreamExt::fuse(rx_conn_shutdown);

  let (_shutdown_sender, shutdown_receiver) = mpsc::unbounded::<Void>();
  broker
    .send(EventClient::NewPeer {
      id: client_id,
      stream: Arc::clone(&stream),
      shutdown: shutdown_receiver,
    })
    .await?;

  loop {
    select! {
      line = lines.next().fuse() => match line {
        Some(line) => {
          let line = line?;
          debug!("[{}]: {}", client_id, line);

          let msg = MsgHeader::decode(&line);
          match msg.msgtype.as_str() {
            MSG_TYPE_SUBSCRIBE => {
              let msg = MsgSubscribe::decode(&line);
              broker
                .send(EventClient::Subscribe {
                  id: client_id,
                  topics: msg.topics,
                })
                .await?;
            }
            MSG_TYPE_UNSUBSCRIBE => {
              let msg = MsgUnSubscribe::decode(&line);
              broker
                .send(EventClient::UnSubscribe {
                  id: client_id,
                  topics: msg.topics,
                })
                .await?;
            }
            MSG_TYPE_UNSUBSCRIBE_ALL => {}
            &_ => {}
          }
        },
        None => break,
      },
      _ = rx_conn_shutdown.next().fuse() => break,
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
  UnSubscribe {
    id: usize,
    topics: Vec<String>,
  },
}

#[derive(Debug)]
enum EventUser {
  Publish { topic: String, data: Vec<u8> },
  PublishAll { data: Vec<u8> },
}

async fn broker_loop(
  name: String,
  mut rx_event_client: Receiver<EventClient>,
  mut rx_event_user: Receiver<EventUser>,
) {
  // <client_id, send out>
  let mut peers: HashMap<usize, Sender<Vec<u8>>> = HashMap::new();

  // <topic, set<client_id>>
  let mut t2c: HashMap<String, HashSet<usize>> = HashMap::new();
  let mut c2t: HashMap<usize, HashSet<String>> = HashMap::new();

  // <client_id, send out>
  let (disconnect_sender, mut disconnect_receiver) =
    mpsc::unbounded::<(usize, Receiver<Vec<u8>>)>();

  loop {
    select! {
      event = rx_event_client.next().fuse() => match event {
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
              for topic in topics {
                t2c.entry(topic.clone()).or_insert_with(|| HashSet::new()).insert(id);
                c2t.entry(id).or_insert_with(|| HashSet::new()).insert(topic);
              }
            },
            EventClient::UnSubscribe { id, topics } => {
              for topic in topics {
                if let Some(topic_client_ids) = t2c.get_mut(&topic) {
                  topic_client_ids.remove(&id);
                }
                if let Some(client_topics) = c2t.get_mut(&id) {
                  client_topics.remove(&topic);
                }
              }
            },
          }
        },
        None => break,
      },
      event = rx_event_user.next().fuse() => match event {
        Some(event) => match event {
          EventUser::Publish { topic, data } => {
            debug!("publish message, topic = {}", topic);
            let msg = MsgPublish::encode(&name, &topic, data);
            if let Some(client_ids) = t2c.get_mut(&topic) {
              for client_id in client_ids.iter() {
                if let Some(client) = peers.get_mut(&client_id) {
                  client.send(msg.clone()).await.unwrap();
                }
              }
            }
          }
          EventUser::PublishAll { data } => {
            debug!("publish all");
            let msg = MsgPublish::encode(&name, "*", data);
            for (client_id, mut client) in peers.iter() {
              client.send(msg.clone()).await.unwrap();
            }
          }
        }
        None => break,
      },
      disconnect = disconnect_receiver.next().fuse() => {
        let (id, _pending_messages) = disconnect.unwrap();
        assert!(peers.remove(&id).is_some());
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
