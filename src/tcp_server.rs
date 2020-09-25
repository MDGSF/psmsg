use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Registry, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::str::from_utf8;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::thread;

// Setup some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(0);

pub struct PSTcpServer {
  pub connections: HashMap<Token, TcpStream>, // Map of `Token` -> `TcpStream`.
}

impl PSTcpServer {
  pub fn new() -> PSTcpServer {
    PSTcpServer {
      connections: HashMap::new(),
    }
  }

  pub fn publish(&mut self, data: &[u8]) -> io::Result<()> {
    Ok(())
  }

  pub fn start_server(&self, addr: String) -> io::Result<()> {
    let (tx, rx) = sync_channel::<&[u8]>(10);

    let map = HashMap::<u32, TcpStream>::new();

    let mut connections_1 = Arc::new(Mutex::new(map));
    let connections_2 = Arc::clone(&connections_1);

    thread::spawn(move || -> io::Result<()> {
      loop {
        match rx.recv() {
          Ok(data) => {
            let a = connections_1.lock().unwrap();
            for (_token, stream) in (connections_1.lock().unwrap()).iter_mut() {
              match stream.write(data) {
                // Ok(n) if n < data.len() => return Err(io::ErrorKind::WriteZero.into()),
                Ok(_) => {}
                // Would block "errors" are the OS's way of saying that the
                // connection is not actually ready to perform this I/O operation.
                Err(ref err) if would_block(err) => {}
                // Other errors we'll consider fatal.
                Err(err) => return Err(err),
              }
            }
          }
          Err(err) => println!("try recv {}", err),
        }
      }
    });

    thread::spawn(move || -> io::Result<()> {
      // Create a poll instance.
      let mut poll = Poll::new()?;
      // Create storage for events.
      let mut events = Events::with_capacity(128);

      // Setup the TCP server socket.
      let addr = addr.parse().unwrap();
      let mut server = TcpListener::bind(addr)?;

      // Register the server with poll we can receive events for it.
      poll
        .registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

      // Unique token for each incoming connection.
      let mut unique_token = Token(SERVER.0 + 1);

      loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
          match event.token() {
            SERVER => loop {
              let (mut connection, address) = match server.accept() {
                Ok((connection, address)) => (connection, address),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                  break;
                }
                Err(e) => {
                  return Err(e);
                }
              };

              println!("Accepted connection from: {}", address);

              let token = next(&mut unique_token);
              poll
                .registry()
                .register(&mut connection, token, Interest::READABLE)?;

              // connections.insert(token, connection);
            },
            token => {
              // let done = handle_connection_event(ps_tcp_server, poll.registry(), token, event)?;
              // if done {
              //   connections.remove(&token);
              // }
            }
          }
        }
      }
    });
    Ok(())
  }
}

/// Returns `true` if the connection is done.
fn handle_connection_event(
  server: &mut PSTcpServer,
  _registry: &Registry,
  token: Token,
  event: &Event,
) -> io::Result<bool> {
  // Maybe received an event for a TCP connection.
  let connection = if let Some(connection) = server.connections.get_mut(&token) {
    connection
  } else {
    // Sporadic events happen, we can safely ignore them.
    return Ok(false);
  };

  if event.is_readable() {
    let mut connection_closed = false;
    let mut received_data = Vec::with_capacity(4096);
    // We can (maybe) read from the connection.
    loop {
      let mut buf = [0; 256];
      match connection.read(&mut buf) {
        Ok(0) => {
          // Reading 0 bytes means the other side has closed the
          // connection or is done writing, then so are we.
          connection_closed = true;
          break;
        }
        Ok(n) => received_data.extend_from_slice(&buf[..n]),
        // Would block "errors" are the OS's way of saying that the
        // connection is not actually ready to perform this I/O operation.
        Err(ref err) if would_block(err) => break,
        Err(ref err) if interrupted(err) => continue,
        // Other errors we'll consider fatal.
        Err(err) => return Err(err),
      }
    }
    if let Ok(str_buf) = from_utf8(&received_data) {
      println!("Received data: {}", str_buf.trim_end());
    } else {
      println!("Received (none UTF-8) data: {:?}", &received_data);
    }
    if connection_closed {
      println!("Connection closed");
      return Ok(true);
    }
  }
  Ok(false)
}

fn next(current: &mut Token) -> Token {
  let next = current.0;
  current.0 += 1;
  Token(next)
}

fn would_block(err: &io::Error) -> bool {
  err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
  err.kind() == io::ErrorKind::Interrupted
}
