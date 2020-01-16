use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread::{self, Thread},
};

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |clone_me| unsafe {
        let arc = Arc::from_raw(clone_me as *const Thread);
        std::mem::forget(arc.clone());
        RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
    },
    |wake_me| unsafe { Arc::from_raw(wake_me as *const Thread).unpark() },
    |wake_by_ref_me| unsafe { (*(wake_by_ref_me as *const Thread)).unpark() },
    |drop_me| unsafe { drop(Arc::from_raw(drop_me as *const Thread)) },
);

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
