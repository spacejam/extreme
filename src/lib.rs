use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

#[derive(Default)]
struct Park(Mutex<u64>, Condvar);

impl Park {
    fn park(&self) {
        let mut counter = self.0.lock().unwrap();
        while *counter == 0 {
            counter = self.1.wait(counter).unwrap();
        }
        *counter -= 1;
    }
    fn unpark(&self) {
        let mut counter = self.0.lock().unwrap();
        *counter += 1;
        self.1.notify_one();
    }
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |clone_me| unsafe {
        let arc = Arc::from_raw(clone_me as *const Park);
        std::mem::forget(arc.clone());
        RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
    },
    |wake_me| unsafe { Arc::from_raw(wake_me as *const Park).unpark() },
    |wake_by_ref_me| unsafe { (*(wake_by_ref_me as *const Park)).unpark() },
    |drop_me| unsafe { drop(Arc::from_raw(drop_me as *const Park)) },
);

/// Run a `Future`.
pub fn run<F: std::future::Future>(mut f: F) -> F::Output {
    let s = Arc::new(Park::default());
    let sender = Arc::into_raw(s.clone());
    let raw_waker = RawWaker::new(sender as *const _, &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    loop {
        let pin = unsafe { std::pin::Pin::new_unchecked(&mut f) };
        match F::poll(pin, &mut cx) {
            Poll::Pending => s.park(),
            Poll::Ready(val) => return val,
        }
    }
}
