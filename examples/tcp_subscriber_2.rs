use anyhow::Result;
use psmsg;

fn main() -> Result<()> {
  env_logger::init();

  let mut subs = Vec::new();
  let sub1 = psmsg::OneSubscribe {
    addr: "127.0.0.1:8080".to_string(),
    topics: vec!["t2".to_string(), "t1".to_string()],
  };
  subs.push(sub1);
  let sub2 = psmsg::OneSubscribe {
    addr: "127.0.0.1:8081".to_string(),
    topics: vec!["t1".to_string()],
  };
  subs.push(sub2);
  let config = psmsg::SubscribeTopics { subs };

  let rx = psmsg::start_tcp_client_with_topics(config);
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
