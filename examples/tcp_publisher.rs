use psmsg;
use std::time::Duration;

fn main() {
  let mut p = psmsg::start_tcp_server("127.0.0.1:8080".to_string());
  for i in 0.. {
    let s = format!("hello world, {}", i);
    p.publish(s.as_bytes().to_vec());
    std::thread::sleep(Duration::from_secs(1));
  }
}
