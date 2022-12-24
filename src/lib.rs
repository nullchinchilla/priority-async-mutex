use std::ops::{Deref, DerefMut};

use event_listener::PriorityEventListener;
use parking_lot::{Mutex, MutexGuard};

mod event_listener;
mod pv;

/// A mutex where the lock operation takes a priority.
pub struct PriorityMutex<T> {
    inner: Mutex<T>,
    listen: PriorityEventListener,
}

impl<T> PriorityMutex<T> {
    /// Creates a new priority mutex.
    pub fn new(t: T) -> Self {
        Self {
            inner: Mutex::new(t),
            listen: PriorityEventListener::new(),
        }
    }

    /// Locks the mutex. When the mutex becomes available, lower priorities are woken up first.
    pub async fn lock(&self, priority: u32) -> PriorityMutexGuard<'_, T> {
        let guard = loop {
            if let Some(val) = self.inner.try_lock() {
                break val;
            } else {
                let listener = self.listen.listen(priority);
                if let Some(val) = self.inner.try_lock() {
                    break val;
                }
                listener.wait().await;
            }
        };
        PriorityMutexGuard {
            inner: guard,
            parent: self,
        }
    }
}

pub struct PriorityMutexGuard<'a, T> {
    inner: MutexGuard<'a, T>,
    parent: &'a PriorityMutex<T>,
}

impl<'a, T> Drop for PriorityMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.parent.listen.notify_one();
    }
}

impl<'a, T> Deref for PriorityMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for PriorityMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}
