use std::pin::Pin;
use std::task::{Context, Poll};
use futures_core::stream::Stream;
use pin_project::pin_project;

/// Extension trait that provides additional utility methods for Stream types.
///
/// This trait is implemented for all types that implement the `Stream` trait,
/// providing additional functionality for stream processing in the EaseRx framework.
pub trait EaseRxStreamExt: Stream {
    /// Creates a stream that stops producing items once the provided predicate returns true.
    ///
    /// This method takes a stream and a predicate function. It returns a new stream that
    /// yields items from the original stream until the predicate returns true for an item.
    /// After that point, the stream will terminate and no more items will be produced.
    ///
    /// ## Examples
    ///
    /// ```
    /// use futures_signals::signal::{Signal, SignalExt};
    /// use futures_signals::signal_vec::SignalVecExt;
    /// use easerx::EaseRxStreamExt;
    ///
    /// async fn example() {
    ///     let stream = futures_signals::signal::always(0)
    ///         .to_stream()
    ///         .stop_if(|&value| value > 5);
    ///
    ///     // The stream will stop once it encounters a value greater than 5
    /// }
    /// ```
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

/// A stream that stops producing items once a predicate returns true.
///
/// This stream is created by the `stop_if` method on `EaseRxStreamExt`.
/// It wraps an inner stream and a predicate function, yielding items from the
/// inner stream until the predicate returns true for an item.
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
