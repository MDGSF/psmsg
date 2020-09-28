use crate::common::*;
use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use futures::join;
use futures::{channel::mpsc, select, FutureExt, SinkExt};
use std::collections::HashMap;
use std::collections::HashSet;
use std::thread;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub struct Publisher {
  tx: Sender<Vec<u8>>,
}

impl Publisher {
  pub fn publish(&mut self, data: Vec<u8>) {
    task::block_on(async {
        self.tx.send(data).await.unwrap()
    });
  }
}

pub fn start_tcp_server(addr: String) -> Publisher {
  let (tx_new_data, rx_new_data) = mpsc::unbounded();

  thread::spawn(move || {
    if let Err(err) = task::block_on(runtime(addr, rx_new_data)) {
      println!("err = {}", err);
    }
  });

  Publisher { tx: tx_new_data }
}

async fn runtime(addr: String, rx_new_data: Receiver<Vec<u8>>) -> Result<()> {
  let (tx_new_client, rx_new_client) = mpsc::unbounded();
  let t1 = spawn_and_log_error(server(addr, tx_new_client));
  let t2 = spawn_and_log_error(broadcast(rx_new_data, rx_new_client));
  join!(t1, t2);
  Ok(())
}

async fn broadcast(
  mut rx_new_data: Receiver<Vec<u8>>,
  mut rx_new_client: Receiver<TcpStream>,
) -> Result<()> {
  let mut uniq_client_id = 0;
  let mut clients: HashMap<i32, TcpStream> = HashMap::new();

  loop {
    select! {
      data = rx_new_data.next().fuse() => match data {
          Some(data) => {
            println!("new data = {:?}", String::from_utf8(data.clone()).unwrap());

            let mut invalid_clients = HashSet::new();
            for (&id, client) in clients.iter_mut() {
              if let Err(err) = client.write_all(&data).await {
                println!("write to client failed, err = {}", err);
                invalid_clients.insert(id);
              }
            }
            for client_id in invalid_clients {
              clients.remove(&client_id);
            }
          }
          None => break,
      },
      client = rx_new_client.next().fuse() => match client {
          Some(client) => {
            println!("new client = {:?}", client);
            uniq_client_id += 1;
            clients.insert(uniq_client_id, client);
          },
          None => break,
      }
    }
  }
  Ok(())
}

async fn server(addr: String, mut tx: Sender<TcpStream>) -> Result<()> {
  let listener = TcpListener::bind(addr).await?;
  println!("Listening on {}", listener.local_addr()?);

  let mut incoming = listener.incoming();
  while let Some(stream) = incoming.next().await {
    let stream = stream?;
    tx.send(stream).await.unwrap();
  }

  Ok(())
}
