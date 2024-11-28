use pin_project::pin_project;
use std::{
    future::Future,
    task::{Context as TaskCtx, Poll},
    time::{Duration, Instant},
};
use tracing::*;

#[pin_project]
pub struct TimedWrapper<Fut: Future> {
    start: Option<Instant>,
    #[pin]
    fut: Fut,
}

impl<Fut: Future> TimedWrapper<Fut> {
    pub fn new(fut: Fut) -> Self {
        Self { fut, start: None }
    }
}

impl<Fut: Future> Future for TimedWrapper<Fut> {
    type Output = (Fut::Output, Duration);
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut TaskCtx<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let start = this.start.get_or_insert_with(Instant::now);
        let inner_poll = this.fut.poll(cx);
        let elapsed = start.elapsed();
        match inner_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready((output, elapsed)),
        }
    }
}

#[tokio::test]
async fn test_timed_fut() {
    let (val, dur) = TimedWrapper::new(async { 42 }).await;
    assert_eq!(val, 42);
}
