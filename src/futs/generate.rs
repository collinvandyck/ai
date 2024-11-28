use anyhow::{Context, Result};
use async_stream::try_stream;
use futures::{future::BoxFuture, FutureExt, Stream, StreamExt, TryFutureExt};
use pin_project::pin_project;
use std::{future::Future, pin::Pin, sync::Arc, task::Poll};
use tokio::pin;

pub struct GenSimple<F, Fut> {
    f: Box<F>,
    fut: Pin<Box<Fut>>,
}

fn gen_fn(start: i32) -> impl Stream<Item = i32> {
    async_stream::stream! {
        for i in start..3 {
            yield i;
        }
    }
}

fn double<S: Stream<Item = i32>>(s: S) -> impl Stream<Item = i32> {
    async_stream::stream! {
        for await num in s {
            yield num * 2
        }
    }
}

fn etc_passwd() -> impl Stream<Item = Result<String>> {
    try_stream! {
        let bs = tokio::fs::read("/etc/passwd").await?;
        let s = String::from_utf8(bs)?;
        yield s
    }
}

#[tokio::test]
async fn test_gen_fn() {
    let res = gen_fn(0).collect::<Vec<_>>().await;
    assert_eq!(res, vec![0, 1, 2]);

    let res = double(gen_fn(0)).collect::<Vec<_>>().await;
    assert_eq!(res, vec![0, 2, 4]);

    let f = etc_passwd()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .nth(0)
        .unwrap()
        .unwrap();
    assert!(f.len() > 0, "{}", f.len());
}

#[pin_project]
pub struct Generate<F, Fut> {
    f: Box<F>,
    #[pin]
    state: GenerateState<Fut>,
}

#[pin_project]
enum GenerateState<Fut> {
    Empty,
    Future {
        #[pin]
        fut: Pin<Box<Fut>>,
    },
}

impl<F, Fut, Item> Generate<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Item>,
{
    fn new(f: F) -> Self {
        let stream = Self {
            f: Box::new(f),
            state: GenerateState::Empty,
        };
        must_stream(&stream);
        stream
    }
}

impl<F, Fut, Item> Stream for Generate<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Item>,
{
    type Item = Item;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();
        match this.state.as_mut().get_mut() {
            GenerateState::Empty => {
                let mut fut = Box::pin((this.f)());
                match fut.as_mut().poll(cx) {
                    Poll::Ready(val) => Poll::Ready(Some(val)),
                    Poll::Pending => {
                        self.state = GenerateState::Future { fut };
                        Poll::Pending
                    }
                }
            }
            GenerateState::Future { fut } => match fut.as_mut().poll(cx) {
                Poll::Ready(val) => {
                    self.state = GenerateState::Empty;
                    Poll::Ready(Some(val))
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

// ensures that the value is a stream
fn must_stream<T>(s: &dyn Stream<Item = T>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use futures::StreamExt;
    use std::{iter, time::Instant};
    use tokio::{pin, sync::Mutex};
    use tracing::instrument;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn generate() {
        let s = Generate::new(|| async { 1 })
            .take(3)
            .collect::<Vec<_>>()
            .await;
        assert_eq!(s, vec![1, 1, 1]);

        let a = 42;
        assert_eq!(Generate::new(|| async { a + 1 }).next().await, Some(43));

        fn foo() -> i32 {
            42
        };
        assert_eq!(Generate::new(|| async { foo() + 1 }).next().await, Some(43));

        async fn word_count() -> Result<usize> {
            let start = Instant::now();
            let res = tokio::fs::read("/usr/share/dict/words")
                .await
                .context("read words")
                .and_then(|l| String::from_utf8(l).context("utf8"))
                .map(|s| s.trim().lines().count());
            tracing::info!("elapsed: {:?}", start.elapsed());
            res
        }

        assert_eq!(
            Generate::new(word_count).next().await.transpose().unwrap(),
            Some(235976)
        );

        assert_eq!(
            Generate::new(word_count)
                .take(10)
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<usize>, _>>()
                .unwrap(),
            iter::repeat(235976).take(10).collect::<Vec<_>>()
        );
    }
}
