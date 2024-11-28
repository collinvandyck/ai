use std::{
    future::Future,
    pin::{Pin, pin},
};

use anyhow::Result;
use futures::{Stream, StreamExt};

fn gen_closure<T, F, Fut>(f: F) -> impl Stream<Item = T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
{
    async_stream::stream! {
        loop {
            let fut = (f)();
            yield fut.await;
        }
    }
}

// using these lifetimes makes the callsites better
fn gen_closure_pinned<'a, T, F, Fut>(f: F) -> Pin<Box<dyn Stream<Item = T> + 'a>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
    F: 'a,
    T: 'a,
{
    Box::pin(async_stream::stream! {
        loop {
            let fut = (f)();
            yield fut.await;
        }
    })
}

#[tokio::test]
async fn test_gen_closure() {
    let s = gen_closure(|| async { 1 }).take(3).collect::<Vec<_>>().await;
    assert_eq!(s, vec![1, 1, 1]);

    // ugh
    let a = 42;
    let fut = gen_closure(|| async { a + 1 });
    tokio::pin!(fut);
    assert_eq!(fut.next().await, Some(43));

    let a = 42;
    assert_eq!(gen_closure_pinned(|| async { a + 1 }).next().await, Some(43));
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
    async_stream::try_stream! {
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

    let f = etc_passwd().collect::<Vec<_>>().await.into_iter().next().unwrap().unwrap();
    assert!(!f.is_empty(), "{}", f.len());
}
