use psmsg;
use anyhow::Result;

fn main() -> Result<()> {
  let rx = psmsg::start_tcp_client(vec!["127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string()]);
  loop {
    let data = rx.recv()?;
    println!("data = {}", String::from_utf8(data).unwrap());
  }
}
