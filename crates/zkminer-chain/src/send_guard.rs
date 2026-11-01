//! Send guard — a tower layer over the alloy RPC transport that refuses every request that would
//! broadcast a transaction or ask the node to sign something.
//!
//! Installed on every [`ChainClient`](crate::client::ChainClient) and inert unless the client is
//! watch-only ([`ChainClient::new_watch_only`](crate::client::ChainClient::new_watch_only)). There
//! it sits beneath the miner's own watch-only checks as the last line: whatever path reaches a send,
//! the request never leaves the process.

use std::task::{Context, Poll};

use alloy::transports::{TransportError, TransportErrorKind, TransportFut};
use alloy_json_rpc::{RequestPacket, ResponsePacket};
use tower::{Layer, Service};

/// Whether `method` broadcasts a transaction or has the node sign one: `eth_sendRawTransaction`,
/// `eth_sendTransaction` and their variants, `eth_sign*`, `personal_*` and `wallet_*`.
pub fn is_send_method(method: &str) -> bool {
    method.starts_with("eth_send")
        || method.starts_with("eth_sign")
        || method.starts_with("personal_")
        || method.starts_with("wallet_")
}

/// tower [`Layer`] that installs the send guard; `refuse` turns it on.
#[derive(Clone, Copy)]
pub struct SendGuardLayer {
    refuse: bool,
}

impl SendGuardLayer {
    pub fn new(refuse: bool) -> Self {
        Self { refuse }
    }
}

impl<S> Layer<S> for SendGuardLayer {
    type Service = SendGuardService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        SendGuardService {
            inner,
            refuse: self.refuse,
        }
    }
}

/// Transport wrapper that fails a send request (single or in a batch) without forwarding it.
#[derive(Clone)]
pub struct SendGuardService<S> {
    inner: S,
    refuse: bool,
}

impl<S> Service<RequestPacket> for SendGuardService<S>
where
    S: Service<
        RequestPacket,
        Response = ResponsePacket,
        Error = TransportError,
        Future = TransportFut<'static>,
    >,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: RequestPacket) -> Self::Future {
        if self.refuse {
            if let Some(method) = req.method_names().find(|m| is_send_method(m)) {
                let err = TransportErrorKind::custom_str(&format!(
                    "watch-only: refusing to send {method}"
                ));
                return Box::pin(async move { Err(err) });
            }
        }
        self.inner.call(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_json_rpc::{Id, Request};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Counts the requests that reach the transport, and answers each with an error.
    #[derive(Clone, Default)]
    struct Counting(Arc<AtomicUsize>);

    impl Service<RequestPacket> for Counting {
        type Response = ResponsePacket;
        type Error = TransportError;
        type Future = TransportFut<'static>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _: RequestPacket) -> Self::Future {
            self.0.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(TransportErrorKind::custom_str("reached the transport")) })
        }
    }

    fn packet(methods: &[&'static str]) -> RequestPacket {
        let mut reqs: Vec<_> = methods
            .iter()
            .enumerate()
            .map(|(i, m)| {
                Request::new(*m, Id::Number(i as u64), ())
                    .serialize()
                    .unwrap()
            })
            .collect();
        if reqs.len() == 1 {
            RequestPacket::Single(reqs.remove(0))
        } else {
            RequestPacket::Batch(reqs)
        }
    }

    async fn send(refuse: bool, methods: &[&'static str]) -> (usize, String) {
        let inner = Counting::default();
        let mut svc = SendGuardLayer::new(refuse).layer(inner.clone());
        let err = svc.call(packet(methods)).await.unwrap_err().to_string();
        (inner.0.load(Ordering::SeqCst), err)
    }

    #[test]
    fn send_methods_are_recognised() {
        for m in [
            "eth_sendRawTransaction",
            "eth_sendTransaction",
            "eth_sendRawTransactionSync",
            "eth_sign",
            "eth_signTransaction",
            "eth_signTypedData_v4",
            "personal_sign",
            "wallet_sendCalls",
        ] {
            assert!(is_send_method(m), "{m}");
        }
        for m in [
            "eth_call",
            "eth_estimateGas",
            "eth_getTransactionCount",
            "eth_getLogs",
            "eth_blockNumber",
            "eth_chainId",
        ] {
            assert!(!is_send_method(m), "{m}");
        }
    }

    #[tokio::test]
    async fn a_refusing_guard_stops_sends_and_passes_reads() {
        let (reached, err) = send(true, &["eth_sendRawTransaction"]).await;
        assert_eq!(reached, 0);
        assert!(
            err.contains("watch-only: refusing to send eth_sendRawTransaction"),
            "{err}"
        );
        // A send hidden in a batch stops the whole batch.
        let (reached, _) = send(true, &["eth_call", "eth_sendTransaction"]).await;
        assert_eq!(reached, 0);
        let (reached, err) = send(true, &["eth_call"]).await;
        assert_eq!(reached, 1);
        assert!(err.contains("reached the transport"), "{err}");
    }

    #[tokio::test]
    async fn an_inert_guard_forwards_everything() {
        let (reached, _) = send(false, &["eth_sendRawTransaction"]).await;
        assert_eq!(reached, 1);
    }
}
