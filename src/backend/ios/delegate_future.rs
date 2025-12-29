use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub struct DelegateFutureState<T> {
    waker: Option<Waker>,
    result: Option<T>,
}

impl<T> DelegateFutureState<T> {
    pub fn complete(&mut self, result: T) {
        self.result = Some(result);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

pub struct DelegateFuture<T> {
    state: Arc<Mutex<DelegateFutureState<T>>>,
}

impl<T> DelegateFuture<T> {
    pub fn new() -> (Self, Arc<Mutex<DelegateFutureState<T>>>) {
        let state = Arc::new(Mutex::new(DelegateFutureState {
            waker: None,
            result: None,
        }));
        (Self { state: state.clone() }, state)
    }
}

impl<T> Future for DelegateFuture<T> {
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

unsafe impl<T: Send> Send for DelegateFuture<T> {}
