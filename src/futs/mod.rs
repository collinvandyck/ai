//! my cursed collection of futures.

mod generate;
mod timed;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{stream, StreamExt};

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
