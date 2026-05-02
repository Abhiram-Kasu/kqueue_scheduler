use libc::{EV_ADD, EV_ENABLE, EVFILT_READ, kevent, kqueue, timespec};
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
    fn os_err(code: i32) -> std::io::Error {
        io::Error::from_raw_os_error(-code.abs())
    }
    pub fn new() -> Result<Self, std::io::Error> {
        unsafe {
            match kqueue() {
                x if x > 0 => Ok(Self {
                    listeners: Default::default(),
                    fd: x,
                }),
                x => Err(kqueue_wrapper::os_err(-x)),
            }
        }
    }

    pub fn listen_to(&mut self, events: &mut [kevent]) -> Result<(), std::io::Error> {
        unsafe {
            let res = kevent(
                self.fd,
                events.as_mut_ptr(),
                events.len() as i32,
                null_mut(),
                0,
                null(),
            );
            if res > 0 {
                Ok(())
            } else {
                Err(kqueue_wrapper::os_err(-res))
            }
        }
    }

    pub fn listen_to_fd(&mut self, file_descriptors: &[usize]) -> Result<(), std::io::Error> {
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

        self.listen_to(events.as_mut_slice())
    }

    pub fn wait(
        &mut self,
        event_buffer: &mut [kevent],
        timeout: Option<std::time::Duration>,
    ) -> Result<usize, std::io::Error> {
        unsafe {
            let timeout = match timeout {
                Some(dur) => &timespec {
                    tv_sec: dur.as_secs() as libc::time_t,
                    tv_nsec: dur.subsec_nanos() as i64,
                },
                None => null(),
            };
            let result = kevent(
                self.fd,
                null_mut(),
                0,
                event_buffer.as_mut_ptr(),
                event_buffer.len() as i32,
                timeout,
            );
            if result > 0 {
                Ok(result as usize)
            } else {
                Err(kqueue_wrapper::os_err(result))
            }
        }
    }
}
