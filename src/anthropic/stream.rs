//! stream related helpers

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{FutureExt, Stream};
use futures_util::StreamExt;
use tokio::sync::Mutex;

struct AccStream<S, Acc, F> {
    acc: Arc<Mutex<Acc>>,
    stream: S,
    func: F,
}

impl<S, Acc, F, Fut> Stream for AccStream<S, Acc, F>
where
    S: Stream + Unpin,
    F: Fn() -> Fut + Unpin,
    Fut: Future<Output = i32>,
{
    type Item = (S::Item, Arc<Mutex<Acc>>);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        Pin::new(&mut this.stream)
            .poll_next(cx)
            .map(|item| item.map(|item| (item, this.acc.clone())))
    }
}

trait AccStreamExt<S> {
    fn acc_stream<Acc, F>(self, acc: Acc, func: F) -> AccStream<S, Acc, F>;
}

impl<T> AccStreamExt<T> for T
where
    T: Stream,
{
    fn acc_stream<Acc, F>(self, acc: Acc, func: F) -> AccStream<T, Acc, F> {
        AccStream {
            acc: Arc::new(Mutex::new(acc)),
            stream: self,
            func,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::anthropic::stream::AccStreamExt;
    use futures::{stream, StreamExt};

    #[tokio::test]
    async fn test_acc_stream() {
        let s = stream::iter(&[1, 2, 3]);
        let mut s = s.acc_stream(0_i32, || async { 42 });
        let (i, _) = s.next().await.unwrap();
        assert_eq!(i, &1);
    }
}
