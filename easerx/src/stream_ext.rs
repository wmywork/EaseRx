use std::pin::Pin;
use std::task::{Context, Poll};
use futures_core::stream::Stream;
use pin_project::pin_project;

pub trait EaseRxStreamExt: Stream {
    fn stop_if<F>(self, test: F) -> StopIf<Self, F>
    where
        F: FnMut(&Self::Item) -> bool,
        Self: Sized,
    {
        StopIf {
            stream: self,
            stopped: false,
            test,
        }
    }
}
impl<T: ?Sized> EaseRxStreamExt for T where T: Stream {}
#[pin_project(project = StopIfProj)]
#[derive(Debug)]
#[must_use = "Streams do nothing unless polled"]
pub struct StopIf<A, B> {
    #[pin]
    stream: A,
    stopped: bool,
    test: B,
}

impl<A, B> Stream for StopIf<A, B>
where A: Stream,
      B: FnMut(&A::Item) -> bool {
    type Item = A::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let StopIfProj { stream, stopped, test } = self.project();

        if *stopped {
            Poll::Ready(None)

        } else {
            match stream.poll_next(cx) {
                Poll::Ready(Some(value)) => {
                    if test(&value) {
                        *stopped = true;
                    }

                    Poll::Ready(Some(value))
                },
                Poll::Ready(None) => {
                    *stopped = true;
                    Poll::Ready(None)
                },
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
