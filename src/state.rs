use tide::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc};
use async_std::sync::RwLock;
use std::time::Duration;
use async_std::{task, task::JoinHandle};
use crossbeam_channel::unbounded;
use tide::sse::Sender; 
use futures::future::join_all;

use crate::models::{Monitor, MonitorKind, ComSender, ComReceiver, Message};


pub(crate) type State = Arc<RwLock<JanitorEngine>>;

pub(crate) type Subscriber = Arc<RwLock<Vec<Sender>>>;

pub(crate) fn new_state(secret: &str) -> State {
  Arc::new(RwLock::new(JanitorEngine::new(secret)))
}

#[derive(Debug)]
pub(crate) struct JanitorEngine {
  pub secret: String,
  pub minions: HashMap<String, Minion>,  
  pub subscribers: Subscriber,
}

#[derive(Debug)]
pub(crate) struct Minion {
  pub monitor: Monitor,
  pub subscribers: Subscriber,
  pub worker: Option<JoinHandle<()>>,
  pub channel: (ComSender, ComReceiver),
}

impl JanitorEngine {
  pub fn new(secret: &str) -> Self {
    JanitorEngine{
      secret: secret.to_string(),
      minions: HashMap::new(),
      subscribers: Arc::new(RwLock::new(Vec::new())),
    }
  }

  pub async fn start(&mut self) -> tide::Result<()> {
    // currently not persiting minions
    // engine starts with nothing to run 
    Ok(())
  }

  pub async fn stop(&self) -> tide::Result<()> {
    // send shutdown message to all minions
    self.minions
      .iter()
      .map(|x| {
        x.1.channel
          .0.send(Message::Shutdown)
          .is_ok()
      }).for_each(drop);
    Ok(())
  }
    
  pub async fn subscribe(&mut self, id: String, sender: Sender) -> bool {
    if let Some(m) = self.minions.get_mut(&id){
      m.subscribers.write().await.push(sender)
    }
    true
  }

  pub async fn subscribe_all(&mut self, sender: Sender) {
    self.subscribers
      .write()
      .await
      .push(sender)
  }

  pub fn heartbeat(&self, id: String, secret: Option<String>) -> bool {
    if !self.minions.contains_key(&id){
      return false;
    }
    let minion = self.minions.get(&id).unwrap();
    if !match (&minion.monitor.secret, &secret) {
      (Some(a), Some(b)) => *a == *b,
      (None, None) => true,
      _ => false
    } {
      return false;
    }
    
    // send heartbeat to worker
    match minion.channel.0.send(Message::Heartbeat) {
      Ok(_) => true,
      Err(_) => false,
    }
  }

  pub fn upsert(&mut self, id: String, monitor: Monitor) -> bool {
    let exist = self.minions.contains_key(&id);
    let mut minion = self.minions.entry(id).or_insert(Minion{
      monitor: monitor.clone(),
      subscribers: Arc::new(RwLock::new(Vec::new())),
      worker: None,
      channel: unbounded(),
    });

    let mut prev_worker = minion.worker.take();
    let monitor = monitor.clone();
    let rx = minion.channel.1.clone();

    let local_subscribers = minion.subscribers.clone();
    let global_subscribers = self.subscribers.clone();
    
    // update minion (monitor & worker)
    minion.monitor = monitor.clone();
    minion.worker = Some(task::spawn(async move {
      // cancel previous worker
      if let Some(handle) = prev_worker.take() {
        handle.cancel().await;
      }

      //TODO schedule new work loop
      println!("{:?}", monitor);
      if monitor.kind == MonitorKind::Ping {
        let url = monitor.url.unwrap();
        loop {
          task::sleep(Duration::from_secs(3)).await;
          println!("ping sent after 3 secs");

          //send request
          let msg = match surf::get(url.as_str()).await {
            Ok(resp) if resp.status() == 200 => json!({
              "status": false,
              "message": "ping succeded!"
            }).to_string(),
            _ => json!({
              "status": false,
              "message": "ping failed"
            }).to_string(), 
          };

          let msg = msg.as_str();
          join_all(local_subscribers.read().await.iter()
            .chain(global_subscribers.read().await.iter())
            .map(|subscriber| async move {
              subscriber.send("ping", msg, None).await
            })).await;
        }
      } else {
        loop {
          let next_time = (2, 5);
          let _ = rx.recv().unwrap();
          println!("heartbeat received ");
          //caclute current time

          //decide if heartbeat is [on-time, late]
          
          //boadcast message to subscribers
          let msg = json!({
              "status": false,
              "message": "heartbeat received!"
            }).to_string();

          let msg = msg.as_str();
          join_all(local_subscribers.read().await.iter()
            .chain(global_subscribers.read().await.iter())
            .map(|subscriber| async move {
              subscriber.send("heartbeat", msg, None).await
            })).await;   
        }
      }
  
    }));
    !exist
  }

  pub fn delete(&mut self, id: String) -> bool {
    match self.minions.get_mut(&id) {
      Some(minion) => {
        let mut worker = minion.worker.take();
        task::spawn(async move {
          // cancel worker
          if let Some(handle) = worker.take() {
            handle.cancel().await;
          }
        });
        self.minions.remove(&id);
        true
      },
      _ => false,
    }
  }
    
  pub fn exists(&self, id: String) -> bool {
    self.minions
      .contains_key(&id)    
  }
    
  pub fn list(&self, page: usize, size: usize) -> (usize, Vec<Monitor>) {
    // collect & sort by keys
    let mut data: Vec<_> = self.minions
      .iter()
      .map(|x| x.1.monitor.clone())
      .collect();
    data.sort_by(|x, y| x.id.cmp(&y.id));

    //skip offset, take size & clone 
    let data: Vec<Monitor> = data.into_iter()
      .skip((page - 1)* size)
      .take(size)
      .collect();
    (data.len(), data)
  }

}



