use serde::{Deserialize, Serialize};
use serde_json;

pub const MSG_VERSION: &'static str = "1.0.0";

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

impl MsgHeader {
  pub fn decode(data: &str) -> MsgHeader {
    serde_json::from_str(&data).expect("parse MsgHeader json failed")
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribe {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,

  pub topics: Vec<String>,
}

impl MsgSubscribe {
  pub fn encode(topics: Vec<String>) -> Vec<u8> {
    let msg = MsgSubscribe {
      version: MSG_VERSION.to_string(),
      msgtype: MSG_TYPE_SUBSCRIBE.to_string(),
      topics: topics,
    };
    let mut msg = serde_json::to_vec(&msg).unwrap();
    msg.push(b'\n');
    msg
  }

  pub fn decode(data: &str) -> MsgSubscribe {
    serde_json::from_str(&data).expect("parse MsgSubscribe json failed")
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribeAll {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,
}

impl MsgSubscribeAll {
  pub fn encode() -> Vec<u8> {
    let msg = MsgSubscribeAll {
      version: MSG_VERSION.to_string(),
      msgtype: MSG_TYPE_SUBSCRIBE_ALL.to_string(),
    };
    let mut msg = serde_json::to_vec(&msg).unwrap();
    msg.push(b'\n');
    msg
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribeSuccess {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,
}

impl MsgSubscribeSuccess {
  pub fn encode() -> Vec<u8> {
    let msg = MsgSubscribeSuccess {
      version: MSG_VERSION.to_string(),
      msgtype: MSG_TYPE_SUBSCRIBE_SUCCESS.to_string(),
    };
    let mut msg = serde_json::to_vec(&msg).unwrap();
    msg.push(b'\n');
    msg
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MsgSubscribeFailed {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,

  pub message: String,
}

impl MsgSubscribeFailed {
  pub fn encode(message: String) -> Vec<u8> {
    let msg = MsgSubscribeFailed {
      version: MSG_VERSION.to_string(),
      msgtype: MSG_TYPE_SUBSCRIBE_FAILED.to_string(),
      message: message,
    };
    let mut msg = serde_json::to_vec(&msg).unwrap();
    msg.push(b'\n');
    msg
  }
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

impl MsgPublish {
  pub fn encode(topic: &str, data: Vec<u8>) -> Vec<u8> {
    let msg = MsgPublish {
      version: MSG_VERSION.to_string(),
      msgtype: MSG_TYPE_PUBLISH.to_string(),
      source: "tcp_server".to_string(),
      topic: topic.to_string(),
      data: data,
    };
    let mut msg = serde_json::to_vec(&msg).unwrap();
    msg.push(b'\n');
    msg
  }

  pub fn decode(data: &str) -> MsgPublish {
    serde_json::from_str(&data).expect("parse MsgPublish json failed")
  }
}
