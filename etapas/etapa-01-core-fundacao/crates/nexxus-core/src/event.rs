//! Typed in-process events owned by the Nexxus Core.

use crate::{ModuleId, ModuleState};
use std::sync::{Arc, Mutex, mpsc};

/// Core-owned events that modules may observe without depending on a concrete
/// graphics backend or on each other directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreEvent {
    ShutdownRequested,
    ModuleStateChanged {
        module: ModuleId,
        state: ModuleState,
    },
    ConfigurationChanged {
        namespace: String,
    },
}

#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<CoreEvent>>>>,
}

pub struct EventSubscription {
    receiver: mpsc::Receiver<CoreEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an independent receiver for future Core events.
    pub fn subscribe(&self) -> EventSubscription {
        let (sender, receiver) = mpsc::channel();
        let mut subscribers = self
            .subscribers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        subscribers.push(sender);
        EventSubscription { receiver }
    }

    /// Publishes an event and prunes subscribers that have disconnected.
    pub fn publish(&self, event: CoreEvent) {
        let mut subscribers = self
            .subscribers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        subscribers.retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }
}

impl EventSubscription {
    pub fn recv(&self) -> Result<CoreEvent, mpsc::RecvError> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Result<CoreEvent, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcasts_to_multiple_subscribers() {
        let bus = EventBus::new();
        let one = bus.subscribe();
        let two = bus.subscribe();
        bus.publish(CoreEvent::ShutdownRequested);

        assert_eq!(one.recv().unwrap(), CoreEvent::ShutdownRequested);
        assert_eq!(two.recv().unwrap(), CoreEvent::ShutdownRequested);
    }
}
