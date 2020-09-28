use anyhow::Result;
use psmsg;

fn main() -> Result<()> {
  env_logger::init();

  let rx = psmsg::start_tcp_client(vec![
    "127.0.0.1:8080".to_string(),
    "127.0.0.1:8081".to_string(),
  ]);
  loop {
    let msg = rx.recv()?;
    println!(
      "[{}]<{}>: {}",
      msg.source,
      msg.topic,
      String::from_utf8(msg.data).unwrap()
    );
  }
}
