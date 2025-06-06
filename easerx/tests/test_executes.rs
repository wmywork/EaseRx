#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::time::sleep;
    use easerx::*;

    #[derive(Default, Clone, Debug, PartialEq)]
    struct Counter {
        num: Async<i32>,
    }

    impl State for Counter {}

    #[tokio::test]
    async fn test_execute() {
        let counter = Counter::default();
        let store: StateStore<Counter> = StateStore::new(counter);
        store.execute(
            || 10,
            |state, result| Counter {
                num: result,
                ..state
            },
        );
        loop {
            if let Ok(state) = store.await_state().await {
                if let Counter { num: Async::Success { value } } = state {
                    assert_eq!(10, value);
                    break;
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
    }
}
