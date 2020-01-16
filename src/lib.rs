use std::{
    future::Future,
    pin::Pin,
    task::{
        Context, Poll, RawWaker, RawWakerVTable, Waker,
    },
    thread::{self, Thread},
};

unsafe fn clone(data: *const ()) -> RawWaker {
    RawWaker::new(data, &RAW_WAKER_VTABLE)
}
unsafe fn wake(data: *const ()) {
    (*(data as *const Thread)).unpark()
}
unsafe fn wake_by_ref(data: *const ()) {
    (*(data as *const Thread)).unpark()
}
unsafe fn drop(_: *const ()) {}

static RAW_WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone, wake, wake_by_ref, drop);

/// Run a `Future`.
pub fn run<F, O>(mut f: F) -> O
where
    F: Future<Output = O>,
{
    let current_thread = thread::current();
    let raw_waker = RawWaker::new(
        &current_thread as *const _ as *const _,
        &RAW_WAKER_VTABLE,
    );
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    loop {
        let pin = unsafe { Pin::new_unchecked(&mut f) };
        match F::poll(pin, &mut cx) {
            Poll::Pending => thread::park(),
            Poll::Ready(val) => {
                return val;
            }
        }
    }
}
