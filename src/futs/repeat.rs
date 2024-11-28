use futures::{future::BoxFuture, FutureExt, Stream, StreamExt};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::pin;

pub struct Repeat<F, Fut> {
    f: Box<F>,
    state: RepeatState<Fut>,
}

fn must_stream<T>(s: &dyn Stream<Item = T>) {}

impl<F, Fut, Item> Repeat<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Item> + Unpin,
{
    fn new(f: F) -> Self {
        let stream = Self {
            f: Box::new(f),
            state: RepeatState::Empty,
        };
        must_stream(&stream);
        stream
    }
}

fn boxed_fut_fn<F, Fut, Item>(f: F) -> impl Fn() -> BoxFuture<'static, Item>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Item> + Send,
{
    let af = Arc::new(f);
    move || {
        let af = af.clone();
        Box::pin(async move {
            //
            af().await
        })
    }
}

fn new_closure<F, Fut, Item>(f: F) -> impl Stream<Item = Item>
where
    F: Fn() -> Fut + Sync + Send + 'static,
    Fut: Future<Output = Item> + Send,
{
    let f = boxed_fut_fn(f);
    Repeat {
        state: RepeatState::Empty,
        f: Box::new(f),
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
    use tokio::{pin, sync::Mutex};

    #[tokio::test]
    async fn repeat_mut() {
        let val = Arc::new(Mutex::new(42));
        let mut s = Repeat::new(|| {
            Box::pin(async {
                let mut val = val.lock().await;
                *val += 1;
                *val
            })
        });
        assert_eq!(s.next().await.unwrap(), 43);
        assert_eq!(s.next().await.unwrap(), 44);

        let mut s = Repeat::new(|| {
            Box::pin(async {
                let mut val = val.lock().await;
                *val += 1;
                *val
            })
        });
        assert_eq!(s.next().await.unwrap(), 45);
        assert_eq!(s.next().await.unwrap(), 46);
    }

    #[tokio::test]
    async fn repeat_test() {
        let mut s = Repeat::new(|| Box::pin(async { 42 }));
        assert_eq!(s.next().await.unwrap(), 42);
        assert_eq!(s.next().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn repeat_test_2() {
        let mut s = new_closure(|| async { 42 });
        assert_eq!(s.next().await.unwrap(), 42);
        assert_eq!(s.next().await.unwrap(), 42);
    }
}
