use psmsg;

fn main() {
  let rx = psmsg::start_tcp_client(vec!["127.0.0.1:8080".to_string()]);
  loop {
    let data = rx.recv().unwrap();
    println!("{}", String::from_utf8(data).unwrap());
  }
}
