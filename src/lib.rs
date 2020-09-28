#![recursion_limit = "512"]

#[macro_use]
extern crate log;

pub mod tcp_client;
pub use tcp_client::*;

pub mod tcp_server;
pub use tcp_server::*;

mod common;
mod messages;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
