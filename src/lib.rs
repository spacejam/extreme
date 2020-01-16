use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread::{self, Thread},
};

unsafe fn clone(data: *const ()) -> RawWaker {
    let arc = Arc::from_raw(data as *const Thread);
    std::mem::forget(arc.clone());
    RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
}
unsafe fn wake(data: *const ()) {
    Arc::from_raw(data as *const Thread).unpark()
}
unsafe fn wake_by_ref(data: *const ()) {
    (*(data as *const Thread)).unpark()
}
unsafe fn drop(data: *const ()) {
    Arc::from_raw(data as *const Thread);
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

/// Run a `Future`.
pub fn run<F: Future>(mut f: F) -> F::Output {
    let current_thread = Arc::into_raw(Arc::new(thread::current()));
    let raw_waker = RawWaker::new(current_thread as *const _, &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    loop {
        let pin = unsafe { Pin::new_unchecked(&mut f) };
        match F::poll(pin, &mut cx) {
            Poll::Pending => thread::park(),
            Poll::Ready(val) => return val,
        }
    }
}
