use anyhow::{anyhow, Result};
use psmsg::{self, Publisher};
use std::time::Duration;

fn main() -> Result<()> {
  env_logger::init();

  let mut args = std::env::args();
  let addr = match args.nth(1).as_ref().map(String::as_str) {
    Some(addr) => addr.to_string(),
    None => return Err(anyhow!("Usage: server <addr>")),
  };

  let mut p = Publisher::new("s5", &addr);
  for i in 0.. {
    let s = format!("{}", i);
    p.publish_topic("t1", s.as_bytes().to_vec());
    std::thread::sleep(Duration::from_millis(50));
  }

  Ok(())
}
