use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use crate::{async_tcp_stream::AsyncTcpStream, executor::Executor, reactor::Reactor};

mod async_tcp_stream;
mod executor;
mod kqueue_wrapper;
mod reactor;
fn main() {
    let reactor = Reactor::new().unwrap();
    let mut executor = Executor::new(reactor.clone());

    executor.spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        listener.set_nonblocking(true).unwrap();

        loop {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let mut async_stream = AsyncTcpStream::new(stream, reactor.clone()).unwrap();
                    let mut buf = [0u8; 1024];
                    let read = async_stream.read(&mut buf).await.unwrap();
                    println!("{:?}", &buf[..read]);
                    
                }
            }
        }
    });

    executor.run();
}
