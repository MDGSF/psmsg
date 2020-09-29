use serde::{Deserialize, Serialize};
use serde_json;

pub const MSG_VERSION: &'static str = "1.0.0";

pub const MSG_TYPE_SUBSCRIBE: &'static str = "subscribe";
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
pub struct MsgUnSubscribe {
  pub version: String,

  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub msgtype: String,

  pub topics: Vec<String>,
}

impl MsgUnSubscribe {
  pub fn encode(topics: Vec<String>) -> Vec<u8> {
    let msg = MsgUnSubscribe {
      version: MSG_VERSION.to_string(),
      msgtype: MSG_TYPE_UNSUBSCRIBE.to_string(),
      topics: topics,
    };
    let mut msg = serde_json::to_vec(&msg).unwrap();
    msg.push(b'\n');
    msg
  }

  pub fn decode(data: &str) -> MsgUnSubscribe {
    serde_json::from_str(&data).expect("parse MsgUnSubscribe json failed")
  }
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
  pub fn encode(source: &str, topic: &str, data: Vec<u8>) -> Vec<u8> {
    let msg = MsgPublish {
      version: MSG_VERSION.to_string(),
      msgtype: MSG_TYPE_PUBLISH.to_string(),
      source: source.to_string(),
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
