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
    func: Pin<Box<F>>,
}

impl<S, Acc, F, Fut> Stream for AccStream<S, Acc, F>
where
    S: Stream + Unpin,
    S::Item: Clone,
    F: Fn(Arc<Mutex<Acc>>, S::Item) -> Fut,
    Fut: Future<Output = i32>,
{
    type Item = (S::Item, Arc<Mutex<Acc>>);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        Pin::new(&mut this.stream).poll_next(cx).map(|item| {
            item.map(|item| {
                // call the closure
                // return the tuple
                println!("calling closure");
                let res = (this.func)(this.acc.clone(), item.clone());
                (item, this.acc.clone())
            })
        })
    }
}

trait AccStreamExt<T> {
    fn acc_stream<Acc, F, Fut>(self, acc: Acc, func: F) -> AccStream<T, Acc, F>
    where
        T: Stream + Unpin,
        F: Fn(Arc<Mutex<Acc>>, T::Item) -> Fut;
}

impl<T> AccStreamExt<T> for T
where
    T: Stream + Unpin,
    T::Item: Clone,
{
    fn acc_stream<Acc, F, Fut>(self, acc: Acc, func: F) -> AccStream<T, Acc, F>
    where
        F: Fn(Arc<Mutex<Acc>>, T::Item) -> Fut,
    {
        AccStream { acc: Arc::new(Mutex::new(acc)), stream: self, func: Box::pin(func) }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::{StreamExt, stream};
    use tokio::sync::Mutex;

    use crate::anthropic::stream::AccStreamExt;

    #[tokio::test]
    async fn test_acc_stream() {
        let s = stream::iter(&[1, 2, 3]);
        let mut s = s.acc_stream(0_i32, |acc, i| async move {
            println!("{i}");
            let acc = acc.lock().await;
            42
        });
        let (i, _) = s.next().await.unwrap();
        assert_eq!(i, &1);
    }
}
