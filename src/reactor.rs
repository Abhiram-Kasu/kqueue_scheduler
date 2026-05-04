use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    task::Waker,
};

use libc::{EVFILT_READ, EVFILT_WRITE, kevent};

use crate::kqueue_wrapper::{FilterType, kqueue_wrapper};

#[derive(Debug, Default)]
struct WakerPair {
    reader: Option<Waker>,
    writer: Option<Waker>,
}

#[derive(Eq, Hash, PartialEq)]
struct Fd(i32);

pub struct Reactor {
    kqueue_wrapper: Mutex<kqueue_wrapper>,
    wakers: Mutex<HashMap<Fd, WakerPair>>,
}
#[derive(Debug)]
pub enum ReactorErrors {
    LockError,
    KqueueError(std::io::Error),
}

impl Reactor {
    pub fn new() -> Result<Arc<Self>, std::io::Error> {
        Ok(Arc::new(Self {
            kqueue_wrapper: Mutex::new(kqueue_wrapper::new()?),
            wakers: Default::default(),
        }))
    }

    pub fn register_read(&self, fd: i32, waker: Waker) -> Result<(), ReactorErrors> {
        self.register(fd, waker, true)
    }

    pub fn register_write(&self, fd: i32, waker: Waker) -> Result<(), ReactorErrors> {
        self.register(fd, waker, false)
    }

    fn register(&self, fd: i32, waker: Waker, is_read: bool) -> Result<(), ReactorErrors> {
        {
            let mut wakers = self.wakers.lock().map_err(|_| ReactorErrors::LockError)?;
            let entry = wakers.entry(Fd(fd)).or_default();
            if is_read {
                entry.reader = Some(waker);
            } else {
                entry.writer = Some(waker);
            }
        }

        let filter = if is_read {
            FilterType::EvfiltRead
        } else {
            FilterType::EvfiltWrite
        };

        let mut kq = self
            .kqueue_wrapper
            .lock()
            .map_err(|_| ReactorErrors::LockError)?;
        kq.listen_to_fd_one_shot(&[(fd as usize, filter)])
            .map_err(ReactorErrors::KqueueError)?;

        Ok(())
    }

    const KEVENT_BUFFER_SIZE: usize = 128;

    pub fn wait(&self) -> Result<(), ReactorErrors> {
        let mut events = vec![unsafe { std::mem::zeroed::<kevent>() }; Self::KEVENT_BUFFER_SIZE];

        let items_read = {
            let kq = self
                .kqueue_wrapper
                .lock()
                .map_err(|_| ReactorErrors::LockError)?;
            kq.wait(events.as_mut_slice(), None)
                .map_err(ReactorErrors::KqueueError)?
        };

        // Take the wakers to wake while holding the lock, then wake outside.
        let mut to_wake = Vec::new();
        {
            let mut wakers = self.wakers.lock().map_err(|_| ReactorErrors::LockError)?;
            for event in &events[..items_read] {
                let fd = event.ident as i32;
                if let Some(entry) = wakers.get_mut(&Fd(fd)) {
                    if event.filter == EVFILT_READ {
                        if let Some(w) = entry.reader.take() {
                            to_wake.push(w);
                        }
                    } else if event.filter == EVFILT_WRITE {
                        if let Some(w) = entry.writer.take() {
                            to_wake.push(w);
                        }
                    }
                }
            }
        }

        for w in to_wake {
            w.wake();
        }

        Ok(())
    }
}
