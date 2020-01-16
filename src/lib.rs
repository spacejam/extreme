use std::{
    future::Future,
    task::{
        Context, Poll, RawWaker, RawWakerVTable, Waker,
    },
    thread::{self, Thread},
};

unsafe fn clone(data: *const ()) -> RawWaker {
    RawWaker::new(data, &RAW_WAKER)
}
unsafe fn wake(data: *const ()) {
    (*(data as *const Thread)).unpark()
}
unsafe fn wake_by_ref(data: *const ()) {
    (*(data as *const Thread)).unpark()
}
unsafe fn drop(_: *const ()) {}

static RAW_WAKER: RawWakerVTable =
    RawWakerVTable::new(clone, wake, wake_by_ref, drop);

/// Run a `Future`.
pub fn run<F, O>(f: F) -> O
where
    F: Future<Output = O> + Unpin,
{
    let mut f_opt = Some(f);
    let current_thread = thread::current();
    let raw_waker = RawWaker::new(
        &current_thread as *const _ as *const _,
        &RAW_WAKER,
    );
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    loop {
        let mut f = f_opt.take().unwrap();
        let pin = std::pin::Pin::new(&mut f);
        match Future::poll(pin, &mut cx) {
            Poll::Pending => thread::park(),
            Poll::Ready(val) => return val,
        }
        f_opt = Some(f);
    }
}
