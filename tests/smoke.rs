use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::SeqCst},
    },
    task::{Context, Poll},
    thread::self,
};

#[test]
fn smoke() {
    let s = S(Arc::new(AtomicBool::new(false)));
    extreme::run(async {});
    extreme::run(async {
        async {
            s.await
        }.await
    });
    extreme::run(async {
        async {
            async {
            }.await
        }.await
    });
}

struct S(Arc<AtomicBool>);

impl Future for S {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        if self.0.load(SeqCst) {
            Poll::Ready(())
        } else {
            let val = self.0.clone();
            let _ = cx.waker().clone();
            let _ = cx.waker().clone();
            let waker = cx.waker().clone();

            thread::spawn(move || {
                val.store(true, SeqCst);
                waker.wake_by_ref();
                waker.wake();
            });

            Poll::Pending
        }
    }
}

#[test]
fn nop() {
    extreme::run(async {
    });
}
