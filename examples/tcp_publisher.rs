use anyhow::{anyhow, Result};
use psmsg;
use std::time::Duration;

fn main() -> Result<()> {
  let mut args = std::env::args();
  let addr = match args.nth(1).as_ref().map(String::as_str) {
    Some(addr) => addr.to_string(),
    None => return Err(anyhow!("Usage: server <addr>")),
  };

  let mut p = psmsg::start_tcp_server(addr);
  for i in 0.. {
    let s = format!("hello world, {}", i);
    p.publish(s.as_bytes().to_vec());
    std::thread::sleep(Duration::from_secs(1));
  }

  Ok(())
}
