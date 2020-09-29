use anyhow::Result;
use psmsg::{self, Subscriber};

fn main() -> Result<()> {
  env_logger::init();

  let mut client = Subscriber::new();
  client.connect("127.0.0.1:8080");
  client.subscribe("127.0.0.1:8080", "t1");
  client.subscribe("127.0.0.1:8080", "t2");
  for i in 0.. {
    let msg = client.recv()?;

    println!(
      "[{}]<{}>: {}",
      msg.source,
      msg.topic,
      String::from_utf8(msg.data).unwrap()
    );

    if i == 10 {
      client.connect("127.0.0.1:8081");
      client.subscribe("127.0.0.1:8081", "t1");
    }
  }
  Ok(())
}
