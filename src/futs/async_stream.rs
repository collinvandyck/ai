use anyhow::Result;
use futures::{Stream, StreamExt};

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
