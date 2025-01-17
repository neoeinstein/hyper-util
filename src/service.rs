//! Service utilities.

use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{util::Oneshot, ServiceExt};

/// A tower service converted into a hyper service.
#[cfg(all(
    any(feature = "http1", feature = "http2"),
    any(feature = "server", feature = "client")
))]
#[derive(Debug, Copy, Clone)]
pub struct TowerToHyperService<S> {
    service: S,
}

impl<S> TowerToHyperService<S> {
    /// Create a new `TowerToHyperService` from a tower service.
    pub fn new(tower_service: S) -> Self {
        Self {
            service: tower_service,
        }
    }
}

impl<S, R> hyper::service::Service<R> for TowerToHyperService<S>
where
    S: tower_service::Service<R> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TowerToHyperServiceFuture<S, R>;

    fn call(&self, req: R) -> Self::Future {
        TowerToHyperServiceFuture {
            future: self.service.clone().oneshot(req),
        }
    }
}

pin_project! {
    /// Response future for [`TowerToHyperService`].
    #[cfg(all(
        any(feature = "http1", feature = "http2"),
        any(feature = "server", feature = "client")
    ))]
    pub struct TowerToHyperServiceFuture<S, R>
    where
        S: tower_service::Service<R>,
    {
        #[pin]
        future: Oneshot<S, R>,
    }
}

impl<S, R> Future for TowerToHyperServiceFuture<S, R>
where
    S: tower_service::Service<R>,
{
    type Output = Result<S::Response, S::Error>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}
