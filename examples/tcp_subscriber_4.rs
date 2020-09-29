use anyhow::Result;
use psmsg::{self, Subscriber};

fn main() -> Result<()> {
  env_logger::init();

  let mut client = Subscriber::new();
  client.connect("127.0.0.1:8080");
  client.subscribe("127.0.0.1:8080", "t1");
  client.subscribe("127.0.0.1:8080", "t2");
  client.subscribe("127.0.0.1:8080", "t3");
  client.subscribe("127.0.0.1:8080", "t4");
  client.subscribe("127.0.0.1:8080", "t5");
  client.subscribe("127.0.0.1:8080", "t6");
  for i in 0.. {
    let msg = client.recv()?;

    println!(
      "[{}]<{}>: {}",
      msg.source,
      msg.topic,
      String::from_utf8(msg.data).unwrap()
    );

    if i == 0 {
      client.unsubscribe("127.0.0.1:8080", "t1");
    } else if i == 6 {
      client.unsubscribe("127.0.0.1:8080", "t2");
    } else if i == 11 {
      client.unsubscribe("127.0.0.1:8080", "t3");
    } else if i == 15 {
      client.unsubscribe("127.0.0.1:8080", "t4");
    } else if i == 18 {
      client.unsubscribe("127.0.0.1:8080", "t5");
    }
  }
  Ok(())
}
