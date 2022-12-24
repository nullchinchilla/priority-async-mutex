use std::collections::BinaryHeap;

use simple_mutex::Mutex;

use crate::pv::PV;

#[derive(Debug)]
pub struct PriorityEventListener {
    events: Mutex<BinaryHeap<PV<u32, oneshot::Sender<()>>>>,
}

impl PriorityEventListener {
    /// Creates a new PEL.
    pub fn new() -> Self {
        Self {
            events: Mutex::new(BinaryHeap::new()),
        }
    }

    /// Listens with a given priority.
    pub fn listen(&self, priority: u32) -> PriorityEvent {
        let (send, recv) = oneshot::channel();
        self.events.lock().push(PV {
            p: priority,
            v: send,
        });
        PriorityEvent(recv)
    }

    /// Notifies one listener.
    pub fn notify_one(&self) {
        while let Some(val) = self.events.lock().pop() {
            // we might need to skip a few dead ones
            if val.v.send(()).is_ok() {
                break;
            }
        }
    }
}

pub struct PriorityEvent(oneshot::Receiver<()>);

impl PriorityEvent {
    pub async fn wait(self) {
        let _ = self.0.await;
    }
}
