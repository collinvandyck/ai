use futures::{FutureExt, Stream, StreamExt};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Repeat<F, Fut> {
    f: Box<F>,
    state: RepeatState<Fut>,
}

impl<F, Fut, Item> Repeat<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Item> + Unpin,
{
    fn new(f: F) -> impl Stream<Item = Item> {
        Self {
            f: Box::new(f),
            state: RepeatState::Empty,
        }
    }
}

enum RepeatState<Fut> {
    Empty,
    Future { fut: Pin<Box<Fut>> },
}

impl<F, Fut, Item> Stream for Repeat<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Item> + Unpin,
{
    type Item = Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.state {
            RepeatState::Empty => {
                let mut fut = Box::pin((self.f)());
                match Pin::new(&mut fut).poll(cx) {
                    Poll::Ready(val) => Poll::Ready(Some(val)),
                    Poll::Pending => {
                        self.state = RepeatState::Future { fut };
                        Poll::Pending
                    }
                }
            }
            RepeatState::Future { fut } => {
                // poll again to see if it's ready
                match fut.as_mut().poll(cx) {
                    Poll::Ready(val) => {
                        self.state = RepeatState::Empty;
                        Poll::Ready(Some(val))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

#[cfg(test)]
mod repeat_test {
    use super::*;
    use futures::StreamExt;
    use tokio::pin;

    #[tokio::test]
    async fn repeat_test() {
        let mut s = Repeat::new(|| Box::pin(async { 42 }));
        assert_eq!(s.next().await.unwrap(), 42);
        assert_eq!(s.next().await.unwrap(), 42);
    }
}
