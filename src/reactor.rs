use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
    task::Waker,
};

use crate::kqueue_wrapper::kqueue_wrapper;
#[derive(Debug, Default)]
struct WakerPair {
    reader: Option<Waker>,
    writer: Option<Waker>,
}
#[derive(Eq, Hash, PartialEq)]
struct Fd(i32);
struct Reactor {
    kqueue_wrapper: kqueue_wrapper,
    wakers: Mutex<HashMap<Fd, WakerPair>>,
}

enum ReactorErrors {
    FdNotFound,
    LockError,
}

impl Reactor {
    pub fn new() -> Result<Arc<Self>, std::io::Error> {
        Ok(Arc::new(Self {
            kqueue_wrapper: kqueue_wrapper::new()?,
            wakers: Default::default(),
        }))
    }

    pub fn add_listener_for_reading(&mut self, fd: Fd, waker: Waker) -> Result<(), ReactorErrors> {
        let wakers = self
            .wakers
            .get_mut()
            .map_err(|_| ReactorErrors::LockError)?;
        if let Some(WakerPair { reader, writer: _ }) = wakers.get_mut(&fd) {
            *reader = Some(waker);
            Ok(())
        } else {
            Err(ReactorErrors::FdNotFound)
        }
    }

    pub fn add_listener_for_writing(&mut self, fd: Fd, waker: Waker) -> Result<(), ReactorErrors> {
        let wakers = self
            .wakers
            .get_mut()
            .map_err(|_| ReactorErrors::LockError)?;
        if let Some(WakerPair { reader: _, writer }) = wakers.get_mut(&fd) {
            *writer = Some(waker);
            Ok(())
        } else {
            Err(ReactorErrors::FdNotFound)
        }
    }
}
