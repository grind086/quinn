use std::{
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
    time::Instant,
};

use async_io::{Async, Timer};

use super::{AsyncTimer, AsyncUdpSocket, UdpSender, UdpSenderHelper, UdpSenderHelperSocket};

impl AsyncTimer for Timer {
    fn reset(mut self: Pin<&mut Self>, t: Instant) {
        self.set_at(t)
    }

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        Future::poll(self, cx).map(|_| ())
    }
}

#[derive(Debug, Clone)]
pub(super) struct UdpSocket {
    io: Arc<Async<std::net::UdpSocket>>,
    inner: Arc<udp::UdpSocketState>,
}

impl UdpSocket {
    pub(super) fn new(sock: std::net::UdpSocket) -> io::Result<Self> {
        Ok(Self {
            inner: Arc::new(udp::UdpSocketState::new((&sock).into())?),
            io: Arc::new(Async::new_nonblocking(sock)?),
        })
    }
}

impl UdpSenderHelperSocket for UdpSocket {
    fn max_transmit_segments(&self) -> usize {
        self.inner.max_gso_segments()
    }

    fn try_send(&self, transmit: &udp::Transmit) -> io::Result<()> {
        self.inner.send((&self.io).into(), transmit)
    }
}

impl AsyncUdpSocket for UdpSocket {
    fn create_sender(&self) -> Pin<Box<dyn UdpSender>> {
        Box::pin(UdpSenderHelper::new(self.clone(), |socket: &Self| {
            let socket = socket.clone();
            async move { socket.io.writable().await }
        }))
    }

    fn poll_recv(
        &mut self,
        cx: &mut Context,
        bufs: &mut [io::IoSliceMut<'_>],
        meta: &mut [udp::RecvMeta],
    ) -> Poll<io::Result<usize>> {
        loop {
            ready!(self.io.poll_readable(cx))?;
            if let Ok(res) = self.inner.recv((&self.io).into(), bufs, meta) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.io.as_ref().as_ref().local_addr()
    }

    fn may_fragment(&self) -> bool {
        self.inner.may_fragment()
    }

    fn max_receive_segments(&self) -> usize {
        self.inner.gro_segments()
    }
}
