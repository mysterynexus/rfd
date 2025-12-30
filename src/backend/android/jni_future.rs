use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub struct JniFutureState<T> {
    waker: Option<Waker>,
    result: Option<T>,
}

impl<T> JniFutureState<T> {
    pub fn complete(&mut self, result: T) {
        self.result = Some(result);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

pub struct JniFuture<T> {
    state: Arc<Mutex<JniFutureState<T>>>,
}

impl<T> JniFuture<T> {
    pub fn new() -> (Self, Arc<Mutex<JniFutureState<T>>>) {
        let state = Arc::new(Mutex::new(JniFutureState {
            waker: None,
            result: None,
        }));
        (Self { state: state.clone() }, state)
    }
}

impl<T> Future for JniFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

unsafe impl<T: Send> Send for JniFuture<T> {}
