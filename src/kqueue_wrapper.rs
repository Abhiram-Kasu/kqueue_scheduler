use libc::{EV_ADD, EV_ENABLE, EVFILT_READ, kevent, kqueue};
use std::{
    collections::HashSet,
    io::{self},
    ptr::{null, null_mut},
};

pub struct kqueue_wrapper {
    listeners: HashSet<u32>,
    fd: i32,
}

impl kqueue_wrapper {
    pub fn new() -> Result<Self, std::io::Error> {
        unsafe {
            match kqueue() {
                x if x > 0 => Ok(Self {
                    listeners: Default::default(),
                    fd: x,
                }),
                x => Err(io::Error::from_raw_os_error(-x)),
            }
        }
    }

    pub fn listen_to(&mut self, file_descriptors: &[usize]) -> Result<(), std::io::Error> {
        let mut events = Vec::new();
        for fd in file_descriptors {
            events.push(kevent {
                ident: *fd,
                filter: EVFILT_READ,
                flags: EV_ADD | EV_ENABLE,
                data: 0,
                udata: null_mut(),
                fflags: 0,
            });
        }

        unsafe {
            match kevent(
                self.fd,
                events.as_mut_ptr(),
                events.len() as i32,
                null_mut(),
                0,
                null(),
            ) {
                x if x > 0 => Ok(()),
                x => Err(io::Error::from_raw_os_error(-x)),
            }
        }
    }
}
