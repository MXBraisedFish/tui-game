//! Completion-event delivery without depending on the application loop.

use std::sync::Arc;

use crossbeam_channel::Sender;

/// A service-neutral sender for asynchronous completion events.
pub struct EventSink<T> {
  send: Arc<dyn Fn(T) + Send + Sync>,
}

impl<T: 'static> EventSink<T> {
  /// Create an event sink initialized from `sender`.
  pub fn new<E: From<T> + Send + 'static>(sender: Sender<E>) -> Self {
    Self {
      send: Arc::new(move |event| {
        // A disconnected receiver no longer consumes completion events; discard the send result.

        let _ = sender.send(E::from(event));
      }),
    }
  }

  /// Send the supplied event or command to the owning runtime channel.
  pub fn send(&self, event: T) {
    (self.send)(event);
  }
}

impl<T> Clone for EventSink<T> {
  fn clone(&self) -> Self {
    Self {
      send: Arc::clone(&self.send),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crossbeam_channel::unbounded;

  #[derive(Debug, PartialEq)]
  enum AppEvent {
    Number(u32),
  }

  impl From<u32> for AppEvent {
    fn from(value: u32) -> Self {
      Self::Number(value)
    }
  }

  #[test]
  fn sink_wraps_events_into_the_application_type() {
    let (sender, receiver) = unbounded::<AppEvent>();
    let sink = EventSink::<u32>::new(sender);
    sink.clone().send(7);
    assert_eq!(receiver.try_recv(), Ok(AppEvent::Number(7)));
  }

  #[test]
  fn sending_after_the_receiver_is_gone_is_ignored() {
    let (sender, receiver) = unbounded::<AppEvent>();
    let sink = EventSink::<u32>::new(sender);
    drop(receiver);
    sink.send(1);
  }
}
