# extreme!

extremely boring async function runner, written in 44 lines of 0-dependency Rust.

## why?

I teach custom Rust workshops that cover a wide variety of low-level subjects. This lays bear the essential runtime complexity of Rust's async functionality for educational purposes. 

## documentation

```rust
/// Run a `Future`.
pub fn run<F, O>(f: F) -> O
where
    F: Future<Output = O>
```

## implementation

```rust
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

#[derive(Default)]
struct Park(Mutex<bool>, Condvar);

fn unpark(park: &Park) {
    *park.0.lock().unwrap() = true;
    park.1.notify_one();
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |clone_me| unsafe {
        let arc = Arc::from_raw(clone_me as *const Park);
        std::mem::forget(arc.clone());
        RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
    },
    |wake_me| unsafe { unpark(&Arc::from_raw(wake_me as *const Park)) },
    |wake_by_ref_me| unsafe { unpark(&*(wake_by_ref_me as *const Park)) },
    |drop_me| unsafe { drop(Arc::from_raw(drop_me as *const Park)) },
);

/// Run a `Future`.
pub fn run<F: std::future::Future>(mut f: F) -> F::Output {
    let mut f = unsafe { std::pin::Pin::new_unchecked(&mut f) };
    let park = Arc::new(Park::default());
    let sender = Arc::into_raw(park.clone());
    let raw_waker = RawWaker::new(sender as *const _, &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Pending => {
                let mut runnable = park.0.lock().unwrap();
                while !*runnable {
                    runnable = park.1.wait(runnable).unwrap();
                }
                *runnable = false;
            }
            Poll::Ready(val) => return val,
        }
    }
}
```

## how does it work?

Rust async blocks and functions evaluate to an implementation of [the `Future` trait](https://doc.rust-lang.org/std/future/trait.Future.html), which has one method: `poll`. If you want to run a Rust `Future` by calling its `poll` method, you need to have a `Context` that you can pass to it. This `Context` allows the `Future` to have some information about the system it is running inside of. In particular, a `Context` provides access to a `Waker`, which is essentially just a raw pointer and a `RawWakerVTable` which can be thought of almost like a trait implementation. The `Waker` allows the `Future` to notify the runtime at a later time, communicating that it should be polled again. You must 

## bugs encountered over time

But it didn't always look this way... Here is a mostly-complete account of the reliability-related engineering effort that went into this.

1. use after free of thread, where the backing future could clone the waker and send a reference to the thread object that lives in the stack frame of the `run` function. If the backing Future wakes the waker after the `run` function returns, it's a use after free. caught by @stjepang
1. use after free due to accidentally dropping an Arc in the vtable clone, caught with ASAN via the sanitizer script
1. race condition triggered by usage of `thread::park`, which can race when other code may rely on thread parking, like `std::sync::Once` being used from the called `Future`'s `poll` method. caught by @tomaka
1. potential correctness issue in pin usage, alleviated by shadowing the input future to guarantee that it is never reused, and using `Pin::as_mut` in the `poll` loop instead of creating a new `Pin` in each loop. caught by @withoutboats

miri, LSAN, and TSAN have also been run on this code, although they have not found issues yet.

## conclusion

Runtimes can be pretty simple. Simple is not easy. `unsafe` must be paired with tools like the sanitizer script in this repo, and ideally as much peer feedback as possible. None of the people mentioned above were requested to look at this code, but because it was only a few dozen lines long, they took a look and provided helpful feedback. Time spent making things look cute and short turned into peer feedback that caught real bugs.
