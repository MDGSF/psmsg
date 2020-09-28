use serde::{Deserialize, Serialize};

pub const MSG_TYPE_SUBSCRIBE: &'static str = "subscribe";
pub const MSG_TYPE_SUBSCRIBE_ALL: &'static str = "subscribe_all";

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgHeader {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribe {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,

  pub topics: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribeAll {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,
}