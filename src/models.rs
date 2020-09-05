use serde::{Deserialize, Serialize};
use crossbeam_channel;


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub (crate) enum  MonitorKind {
  // The target should contact this server
  Heartbeat,
  // This server should contact the target
  Ping,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub (crate) struct Monitor {
  pub id: String,
  pub kind: MonitorKind,
  pub schedule: String,
  pub url: Option<String>,
  pub secret: Option<String>,
}
#[derive(Debug)]
pub(crate) enum Message {
  Shutdown,
  Heartbeat,
}

pub(crate) type ComSender = crossbeam_channel::Sender<Message>;
pub(crate) type ComReceiver = crossbeam_channel::Receiver<Message>;

