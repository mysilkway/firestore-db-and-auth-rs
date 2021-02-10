use crate::errors::{FirebaseError, Result};
use backoff::{retry, ExponentialBackoff, future::retry as future_retry};
use std::future::Future;
use std::time::Duration;

pub const FIRESTORE_REQUEST_RETRY_MAX_ELAPSED_TIME: u64 = 30;

/// run async function with exponential backoff
pub async fn exp_backoff_async<T, F, Fut>(f: F, max_elapsed_time: u64) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = std::result::Result<T, backoff::Error<FirebaseError>>>,
{
    let mut backoff = ExponentialBackoff::default();
    backoff.max_elapsed_time = Some(Duration::from_secs(max_elapsed_time));
    future_retry(backoff, f).await
}

/// run function with exponential backoff
pub fn exp_backoff<T, F: FnMut() -> std::result::Result<T, backoff::Error<FirebaseError>>>(
    f: F,
    max_elapsed_time: u64,
) -> Result<T> {
    let mut backoff = ExponentialBackoff::default();
    backoff.max_elapsed_time = Some(Duration::from_secs(max_elapsed_time));
    retry(backoff, f).map_err(|err| match err {
        backoff::Error::Permanent(err) => err,
        backoff::Error::Transient(err) => err,
    })
}

/// HTTP status which should be needed to use exponential backoff
pub fn retryable_http_status(status: u16) -> bool {
    return status == 408 || status == 409 || status == 429 || (status >= 500 && status < 600);
}
