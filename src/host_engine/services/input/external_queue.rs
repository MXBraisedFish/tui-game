use crossbeam_channel::{Receiver, Sender, unbounded};

use super::RawInputEvent;

#[derive(Clone)]
pub struct ExternalRawInputSender {
  sender: Sender<RawInputEvent>,
}

impl ExternalRawInputSender {
  pub fn push(&self, event: RawInputEvent) {
    let _ = self.sender.send(event);
  }
}

pub struct ExternalRawInputQueue {
  sender: ExternalRawInputSender,
  receiver: Receiver<RawInputEvent>,
}

impl ExternalRawInputQueue {
  pub fn new() -> Self {
    let (sender, receiver) = unbounded();

    Self {
      sender: ExternalRawInputSender { sender },
      receiver,
    }
  }

  pub fn sender(&self) -> ExternalRawInputSender {
    self.sender.clone()
  }

  pub fn push(&self, event: RawInputEvent) {
    self.sender.push(event);
  }

  pub fn pop(&self) -> Option<RawInputEvent> {
    self.receiver.try_recv().ok()
  }

  pub fn len(&self) -> usize {
    self.receiver.len()
  }

  pub fn is_empty(&self) -> bool {
    self.receiver.is_empty()
  }

  pub fn clear(&self) {
    while self.pop().is_some() {}
  }
}
