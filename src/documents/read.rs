use super::*;
use crate::backoff::{exp_backoff, exp_backoff_async, retryable_http_status, FIRESTORE_REQUEST_RETRY_MAX_ELAPSED_TIME};

///
/// Read a document of a specific type from a collection by its Firestore document name
///
/// ## Arguments
/// * 'auth' The authentication token
/// * 'document_name' The document path / collection and document id; For example "projects/my_project/databases/(default)/documents/tests/test"
pub fn read_by_name<T>(auth: &impl FirebaseAuthBearer, document_name: impl AsRef<str>) -> Result<T>
where
    for<'b> T: Deserialize<'b>,
{
    let url = firebase_url_base(document_name.as_ref());

    let resp = exp_backoff(
        || {
            let resp = auth
                .client()
                .get(&url)
                .bearer_auth(auth.access_token().to_owned())
                .send()
                .map_err(|err| backoff::Error::Permanent(FirebaseError::from(err)))?;

            let status = resp.status().as_u16();

            match extract_google_api_error(resp, || document_name.as_ref().to_owned()) {
                Ok(new_resp) => Ok(new_resp),
                Err(err) => {
                    if retryable_http_status(status) {
                        Err(backoff::Error::Transient(err))
                    } else {
                        Err(backoff::Error::Permanent(err))
                    }
                }
            }
        },
        FIRESTORE_REQUEST_RETRY_MAX_ELAPSED_TIME,
    )?;

    let json: dto::Document = resp.json()?;
    Ok(document_to_pod(&json)?)
}

///
/// [Async] Read a document of a specific type from a collection by its Firestore document name
///
/// ## Arguments
/// * 'auth' The authentication token
/// * 'document_name' The document path / collection and document id; For example "projects/my_project/databases/(default)/documents/tests/test"
pub async fn read_by_name_async<T>(auth: &impl FirebaseAuthBearer, document_name: impl AsRef<str>) -> Result<T>
where
    for<'b> T: Deserialize<'b>,
{
    let url = firebase_url_base(document_name.as_ref());

    let resp = exp_backoff_async(
        || async {
            let resp = auth
                .client_async()
                .get(&url)
                .bearer_auth(auth.access_token().to_owned())
                .send()
                .await
                .map_err(|err| backoff::Error::Permanent(FirebaseError::from(err)))?;

            let status = resp.status().as_u16();

            match extract_google_api_error_async(resp, || document_name.as_ref().to_owned()).await {
                Ok(new_resp) => Ok(new_resp),
                Err(err) => {
                    if retryable_http_status(status) {
                        Err(backoff::Error::Transient(err))
                    } else {
                        Err(backoff::Error::Permanent(err))
                    }
                }
            }
        },
        FIRESTORE_REQUEST_RETRY_MAX_ELAPSED_TIME,
    )
    .await?;

    let json: dto::Document = resp.json().await?;
    Ok(document_to_pod(&json)?)
}

///
/// Read a document of a specific type from a collection
///
/// ## Arguments
/// * 'auth' The authentication token
/// * 'path' The document path / collection; For example "my_collection" or "a/nested/collection"
/// * 'document_id' The document id. Make sure that you do not include the document id to the path argument.
pub fn read<T>(auth: &impl FirebaseAuthBearer, path: &str, document_id: impl AsRef<str>) -> Result<T>
where
    for<'b> T: Deserialize<'b>,
{
    let document_name = format!(
        "projects/{}/databases/(default)/documents/{}/{}",
        auth.project_id(),
        path,
        document_id.as_ref()
    );
    read_by_name(auth, &document_name)
}

///
/// [Async] Read a document of a specific type from a collection
///
/// ## Arguments
/// * 'auth' The authentication token
/// * 'path' The document path / collection; For example "my_collection" or "a/nested/collection"
/// * 'document_id' The document id. Make sure that you do not include the document id to the path argument.
pub async fn read_async<T>(auth: &impl FirebaseAuthBearer, path: &str, document_id: impl AsRef<str>) -> Result<T>
where
    for<'b> T: Deserialize<'b>,
{
    let document_name = format!(
        "projects/{}/databases/(default)/documents/{}/{}",
        auth.project_id(),
        path,
        document_id.as_ref()
    );
    read_by_name_async(auth, &document_name).await
}
