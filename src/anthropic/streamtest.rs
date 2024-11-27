use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

#[cfg(test)]
mod tests {
    use futures::{stream, StreamExt};

    use super::*;

    #[tokio::test]
    async fn test_streams() {
        let mut s = stream::iter([1, 2, 3]);
        assert_eq!(s.next().await, Some(1));
        assert_eq!(s.next().await, Some(2));
        assert_eq!(s.next().await, Some(3));
        assert_eq!(s.next().await, None);

        let mut s = stream::iter([1, 2, 3]);
        let s = s.collect::<Vec<_>>().await;
        assert_eq!(s, vec![1, 2, 3]);

        let mut s = stream::repeat(3).take(3);
        let s = s.collect::<Vec<_>>().await;
        assert_eq!(s, vec![3, 3, 3]);

        let s = stream::iter([1, 2, 3])
            .map(|f| f * 2)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(s, vec![2, 4, 6]);

        let mut acc = 0;
        let s = stream::repeat_with(|| {
            acc += 1;
            acc
        })
        .take(3)
        .collect::<Vec<_>>()
        .await;
        assert_eq!(s, vec![1, 2, 3]);

        let s = stream::unfold(0, |i| async move { (i < 3).then_some((i * 2, i + 1)) })
            .take(3)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(s, vec![0, 2, 4]);
    }
}

struct Repeat<F, Fut> {
    f: F,
    state: RepeatState<Fut>,
}

impl<F, Fut, Item> Repeat<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Item>,
{
    fn new(f: F) -> Self {
        Self {
            f,
            state: RepeatState::Empty,
        }
    }
}

enum RepeatState<Fut> {
    Empty,
    Future { fut: Fut },
}

impl<F, Fut, Item> Stream for Repeat<F, Fut>
where
    F: Fn() -> Fut + Unpin,
    Fut: Future<Output = Item> + Unpin,
{
    type Item = Item;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut();
        match &mut this.state {
            RepeatState::Empty => {
                let mut fut = (this.f)();
                match Pin::new(&mut fut).poll(cx) {
                    Poll::Ready(val) => Poll::Ready(Some(val)),
                    Poll::Pending => {
                        this.state = RepeatState::Future { fut };
                        Poll::Pending
                    }
                }
            }
            RepeatState::Future { fut } => match Pin::new(fut).poll(cx) {
                Poll::Ready(val) => {
                    this.state = RepeatState::Empty;
                    Poll::Ready(Some(val))
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

#[cfg(test)]
mod repeat_test {
    use futures::StreamExt;

    use super::*;

    #[tokio::test]
    async fn repeat_test() {
        let mut s = Repeat::new(|| async { 42 });
    }
}
