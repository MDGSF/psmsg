#![recursion_limit = "256"]

pub mod tcp_client;
pub use tcp_client::*;

pub mod tcp_server;
pub use tcp_server::*;

mod common;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
