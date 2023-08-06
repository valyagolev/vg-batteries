use futures::stream::Stream;

// use futures_lite::StreamExt;
use pin_project::pin_project;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::pin::{pin, Pin};
use std::task::{Context, Poll};

#[pin_project]
pub struct HighestStream<S>
where
    S: Stream,
    S::Item: Ord + Clone,
{
    how_many: usize,
    highest: BinaryHeap<Reverse<S::Item>>,

    #[pin]
    stream: S,
}

impl<S> HighestStream<S>
where
    S: Stream,
    S::Item: Ord + Clone,
{
    pub fn new(stream: S, how_many: usize) -> Self {
        Self {
            how_many,
            highest: BinaryHeap::new(),
            stream,
        }
    }
}

impl<S> Stream for HighestStream<S>
where
    S: Stream,
    S::Item: Ord + Clone,
{
    type Item = Vec<S::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        let next_item_poll = futures::ready!(this.stream.poll_next(cx));

        if let Some(next_item) = next_item_poll {
            if this.highest.len() < *this.how_many {
                this.highest.push(Reverse(next_item));
            } else if &this.highest.peek().unwrap().0 < &next_item {
                this.highest.pop();
                this.highest.push(Reverse(next_item));
            }

            let mut highest_so_far = this
                .highest
                .iter()
                .map(|item| (item.0).clone())
                .collect::<Vec<_>>();
            highest_so_far.sort();
            highest_so_far.reverse();
            Poll::Ready(Some(highest_so_far))
        } else {
            Poll::Ready(None)
        }
    }
}
