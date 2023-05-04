use async_trait::async_trait;

use super::{Error, Permission};

/// An authorizer is used to validate a request
/// (+ associated permissions needed to fulfill the request)
/// with an authorization token that has been extracted from the request.
#[async_trait]
pub trait Authorizer: std::fmt::Debug + Send + Sync {
    /// Determine the permissions associated with a request token.
    ///
    /// The returned list of permissions is the intersection of the permissions
    /// requested and the permissions associated with the token.
    ///
    /// Implementations of this trait should only error if:
    ///     * there is a failure processing the token.
    ///     * the token is invalid.
    ///     * there is not any intersection of permissions.
    async fn permissions(
        &self,
        token: Option<Vec<u8>>,
        perms: &[Permission],
    ) -> Result<Vec<Permission>, Error>;

    /// Make a test request that determines if end-to-end communication
    /// with the service is working.
    async fn probe(&self) -> Result<(), Error> {
        match self.permissions(Some(b"".to_vec()), &[]).await {
            // got response from authorizer server
            Ok(_) | Err(Error::Forbidden) | Err(Error::InvalidToken) => Ok(()),
            // other errors, including Error::Verification
            Err(e) => Err(e),
        }
    }
}

/// Wrapped `Option<dyn Authorizer>`
/// Provides response to inner `IoxAuthorizer::permissions()`
#[async_trait]
impl<T: Authorizer> Authorizer for Option<T> {
    async fn permissions(
        &self,
        token: Option<Vec<u8>>,
        perms: &[Permission],
    ) -> Result<Vec<Permission>, Error> {
        match self {
            Some(authz) => authz.permissions(token, perms).await,
            // no authz rpc service => return same perms requested. Used for testing.
            None => Ok(perms.to_vec()),
        }
    }
}

#[async_trait]
impl<T: AsRef<dyn Authorizer> + std::fmt::Debug + Send + Sync> Authorizer for T {
    async fn permissions(
        &self,
        token: Option<Vec<u8>>,
        perms: &[Permission],
    ) -> Result<Vec<Permission>, Error> {
        self.as_ref().permissions(token, perms).await
    }
}
