use std::{future::Future, io, pin::Pin, time::Instant};

use async_io::Timer;

use super::{AsyncTimer, AsyncUdpSocket, Runtime, async_io::UdpSocket};

/// A Quinn runtime for smol
#[derive(Debug)]
pub struct SmolRuntime;

impl Runtime for SmolRuntime {
    fn new_timer(&self, t: Instant) -> Pin<Box<dyn AsyncTimer>> {
        Box::pin(Timer::at(t))
    }

    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        ::smol::spawn(future).detach();
    }

    fn wrap_udp_socket(&self, sock: std::net::UdpSocket) -> io::Result<Box<dyn AsyncUdpSocket>> {
        Ok(Box::new(UdpSocket::new(sock)?))
    }
}
