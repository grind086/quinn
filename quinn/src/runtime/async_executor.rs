use std::{future::Future, io, pin::Pin, time::Instant};

use async_executor::Executor;
use async_io::Timer;
use async_lock::OnceCell;

use super::{AsyncTimer, AsyncUdpSocket, Runtime, async_io::UdpSocket};

/// The global [`Executor`] used by quinn.
///
/// This must be set prior to creating any quinn [`Endpoint`]s when using `runtime-async-executor`.
/// Note that quinn will priorities the global tokio runtime over this one if `runtime-tokio` is
/// enabled. If `runtime-smol` is enabled, it will serve as a fallback when no executor is set.
///
/// [`Endpoint`]: crate::Endpoint
pub static ASYNC_EXECUTOR: OnceCell<Executor> = OnceCell::new();

/// A quinn runtime for a user-provided [`Executor`].
#[derive(Debug)]
pub struct AsyncExecutorRuntime;

impl Runtime for AsyncExecutorRuntime {
    fn new_timer(&self, t: Instant) -> Pin<Box<dyn AsyncTimer>> {
        Box::pin(Timer::at(t))
    }

    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        ASYNC_EXECUTOR
            .get()
            .expect("`quinn::ASYNC_EXECUTOR` is not set")
            .spawn(future)
            .detach();
    }

    fn wrap_udp_socket(&self, sock: std::net::UdpSocket) -> io::Result<Box<dyn AsyncUdpSocket>> {
        Ok(Box::new(UdpSocket::new(sock)?))
    }
}
