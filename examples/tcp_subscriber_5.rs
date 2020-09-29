use anyhow::{anyhow, Result};
use psmsg::{self, Subscriber};

fn main() -> Result<()> {
  env_logger::init();

  let mut args = std::env::args();
  let addr = match args.nth(1).as_ref().map(String::as_str) {
    Some(addr) => addr.to_string(),
    None => return Err(anyhow!("Usage: server <addr>")),
  };

  let mut client = Subscriber::new();
  client.connect(&addr);
  client.subscribe(&addr, "t1");

  let mut pre_num = 0;
  for i in 0.. {
    let msg = client.recv()?;

    println!(
      "{} [{}]<{}>: {}",
      i,
      msg.source,
      msg.topic,
      String::from_utf8(msg.data.clone()).unwrap()
    );

    let cur_num = String::from_utf8(msg.data).unwrap().parse::<i32>().unwrap();
    if pre_num == 0 {
      pre_num = cur_num;
    } else {
      if pre_num + 1 != cur_num {
        panic!("pre_num = {}, cur_num = {}", pre_num, cur_num);
      }
      pre_num = cur_num;
    }
  }
  Ok(())
}
