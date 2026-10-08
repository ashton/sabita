use std::future::Future;

/// Runs `future` to completion on a dedicated, single-threaded Tokio runtime,
/// on its own OS thread, and bridges the result back through a channel.
///
/// iced's own executor (the `thread-pool` feature) doesn't provide a Tokio
/// reactor, so futures that need one (anything built on `reqwest`/`hyper`)
/// panic with "there is no reactor running" if driven directly by
/// `Task::perform`. Wrap such futures in `on_tokio` before handing them to
/// `Task::perform`.
pub fn on_tokio<F>(future: F) -> impl Future<Output = F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        let _ = tx.send(runtime.block_on(future));
    });

    async move { rx.await.expect("on_tokio task panicked") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn noop_waker() -> Waker {
        fn clone(_: *const ()) -> RawWaker {
            raw_waker()
        }
        fn noop(_: *const ()) {}
        fn raw_waker() -> RawWaker {
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
            RawWaker::new(std::ptr::null(), &VTABLE)
        }

        unsafe { Waker::from_raw(raw_waker()) }
    }

    /// Reproduces the bug this module fixes: a future that needs a Tokio
    /// reactor (like one built on `reqwest`) panics if polled directly by a
    /// non-Tokio executor. `on_tokio` must let it resolve cleanly even when
    /// polled from a bare, hand-rolled loop with no ambient Tokio runtime,
    /// the same way iced's own executor would drive it.
    #[test]
    fn on_tokio_resolves_without_an_ambient_tokio_runtime() {
        let mut future: Pin<Box<dyn Future<Output = i32>>> = Box::pin(on_tokio(async { 21 + 21 }));
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let result = loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(value) => break value,
                Poll::Pending => std::thread::yield_now(),
            }
        };

        assert_eq!(result, 42);
    }
}
