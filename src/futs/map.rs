use std::{future::Future, process::Output};

use futures::Stream;

pub struct Map<S, F> {
    stream: S,
    func: Box<F>,
}

impl<S, F, Fut, T> Map<S, F>
where
    S: Stream,
    F: Fn(S::Item) -> Fut,
    Fut: Future<Output = T>,
{
    fn new(stream: S, func: F) -> Self {
        let func = Box::new(func);
        Self { stream, func }
    }
}

impl<S, F, Fut, T> Stream for Map<S, F>
where
    S: Stream,
    F: Fn(S::Item) -> Fut,
    Fut: Future<Output = T>,
{
    type Item = ();
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use futures::stream;

    use super::*;

    #[tokio::test]
    async fn test_stream() {
        let s = stream::iter([String::from("hi")]);
        let s = Map::new(s, |x| async move { (x.len(), x) });
    }
}
