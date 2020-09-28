use serde::{Deserialize, Serialize};

pub const MSG_TYPE_SUBSCRIBE: &'static str = "subscribe";
pub const MSG_TYPE_SUBSCRIBE_ALL: &'static str = "subscribe_all";
pub const MSG_TYPE_SUBSCRIBE_SUCCESS: &'static str = "subscribe_success";
pub const MSG_TYPE_SUBSCRIBE_FAILED: &'static str = "subscribe_failed";
pub const MSG_TYPE_UNSUBSCRIBE: &'static str = "unsubscribe";
pub const MSG_TYPE_UNSUBSCRIBE_ALL: &'static str = "unsubscribe_all";
pub const MSG_TYPE_PUBLISH: &'static str = "publish";

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

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribeSuccess {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribeFailed {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,

  pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgUnSubscribe {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,

  pub topics: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgPublish {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,

  pub source: String,

  pub topic: String,

  pub data: Vec<u8>,
}