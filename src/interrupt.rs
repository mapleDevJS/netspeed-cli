//! Async operation cancellation on Ctrl+C.

use std::future::Future;

/// The active operation was cancelled by the interrupt future.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("speed test interrupted")]
pub struct Interrupted;

/// Race an operation against a future that resolves when interruption is requested.
///
/// The operation future is dropped when the interrupt future resolves, cancelling
/// in-flight async work and releasing resources owned by that operation.
pub async fn run_or_interrupt<R, I>(run: R, interrupt: I) -> Result<R::Output, Interrupted>
where
    R: Future,
    I: Future<Output = ()>,
{
    tokio::pin!(run);
    tokio::pin!(interrupt);
    tokio::select! {
        biased;
        _ = &mut interrupt => Err(Interrupted),
        result = &mut run => Ok(result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn interruption_cancels_the_active_operation() {
        let (send, receive) = tokio::sync::oneshot::channel::<()>();
        let run = std::future::pending::<()>();
        let interrupt = async move {
            let _ = receive.await;
        };
        let task = tokio::spawn(run_or_interrupt(run, interrupt));

        send.send(()).expect("interrupt receiver should be active");
        assert_eq!(
            task.await.expect("race task should complete"),
            Err(Interrupted)
        );
    }

    #[tokio::test]
    async fn completed_operation_wins_the_race() {
        let result = run_or_interrupt(async { 42 }, std::future::pending()).await;
        assert_eq!(result, Ok(42));
    }
}
