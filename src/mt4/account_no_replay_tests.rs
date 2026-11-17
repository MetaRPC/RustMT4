//! Trade calls are never re-sent after a transport failure; reads are retried; a refused API key
//! fails fast. Runs against an in-process fake gateway (plain HTTP/2 on 127.0.0.1, no network).

use std::convert::Infallible;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use tonic::body::BoxBody;
use tonic::codec::ProstCodec;
use tonic::codegen::{http, BoxFuture, Service};
use tonic::server::NamedService;
use tonic::transport::server::TcpIncoming;
use tonic::transport::{Endpoint, Server};
use tonic::{Code, Status};

use super::{ConnectParams, MT4Account};
use crate::error::MT4Error;
use crate::pb::mt4_term_api as pb;

/// What the fake gateway answers.
#[derive(Clone, Copy)]
enum Mode {
    /// Every trade call fails with this transport code.
    TradeStatus(Code),
    /// The first OrderSend answers TERMINAL_INSTANCE_NOT_FOUND in-band, the next succeeds.
    NotFoundOnce,
    /// Every call answers API_KEY_INVALID in-band.
    ApiKeyInvalid,
    /// Every read fails with UNAVAILABLE "API_KEY_REQUIRED".
    ApiKeyStatus,
    /// The first read fails with UNAVAILABLE, the next succeeds.
    ReadUnavailableOnce,
}

struct State {
    mode: Mutex<Mode>,
    trade_calls: AtomicU32,
    read_calls: AtomicU32,
    connect_calls: AtomicU32,
}

fn api_error(code: &str) -> pb::Error {
    pb::Error { error_code: code.into(), error_message: "fake".into(), ..Default::default() }
}

impl State {
    fn mode(&self) -> Mode {
        *self.mode.lock().unwrap()
    }

    fn trade_failure(&self) -> Option<Status> {
        self.trade_calls.fetch_add(1, Ordering::SeqCst);
        match self.mode() {
            Mode::TradeStatus(c) => Some(Status::new(c, "connection lost")),
            _ => None,
        }
    }

    fn order_send(&self, _: pb::OrderSendRequest) -> Result<pb::OrderSendReply, Status> {
        use pb::order_send_reply::Response;
        if let Some(s) = self.trade_failure() {
            return Err(s);
        }
        let n = self.trade_calls.load(Ordering::SeqCst);
        let response = match self.mode() {
            Mode::NotFoundOnce if n == 1 => Response::Error(api_error("TERMINAL_INSTANCE_NOT_FOUND")),
            Mode::ApiKeyInvalid => Response::Error(api_error("API_KEY_INVALID")),
            _ => Response::Data(pb::OrderSendData { ticket: 7, ..Default::default() }),
        };
        Ok(pb::OrderSendReply { response: Some(response) })
    }

    fn order_modify(&self, _: pb::OrderModifyRequest) -> Result<pb::OrderModifyReply, Status> {
        match self.trade_failure() {
            Some(s) => Err(s),
            None => Ok(pb::OrderModifyReply { response: None }),
        }
    }

    fn order_close_delete(&self, _: pb::OrderCloseDeleteRequest) -> Result<pb::OrderCloseDeleteReply, Status> {
        match self.trade_failure() {
            Some(s) => Err(s),
            None => Ok(pb::OrderCloseDeleteReply { response: None }),
        }
    }

    fn order_close_by(&self, _: pb::OrderCloseByRequest) -> Result<pb::OrderCloseByReply, Status> {
        match self.trade_failure() {
            Some(s) => Err(s),
            None => Ok(pb::OrderCloseByReply { response: None }),
        }
    }

    fn account_summary(&self, _: pb::AccountSummaryRequest) -> Result<pb::AccountSummaryReply, Status> {
        use pb::account_summary_reply::Response;
        let n = self.read_calls.fetch_add(1, Ordering::SeqCst) + 1;
        let response = match self.mode() {
            Mode::ReadUnavailableOnce if n == 1 => return Err(Status::unavailable("connection reset")),
            Mode::ApiKeyStatus => return Err(Status::unavailable("API_KEY_REQUIRED: no api key")),
            Mode::ApiKeyInvalid => Response::Error(api_error("API_KEY_INVALID")),
            _ => Response::Data(pb::AccountSummaryData { account_login: 42, ..Default::default() }),
        };
        Ok(pb::AccountSummaryReply { response: Some(response) })
    }

    fn connect_ex(&self, _: pb::ConnectExRequest) -> Result<pb::ConnectExReply, Status> {
        use pb::connect_ex_reply::Response;
        self.connect_calls.fetch_add(1, Ordering::SeqCst);
        let data = pb::ConnectData { terminal_instance_guid: "11111111-2222-3333-4444-555555555555".into(), ..Default::default() };
        Ok(pb::ConnectExReply { response: Some(Response::Data(data)) })
    }
}

/// A unary handler around a closure.
struct Handler<F>(F);

impl<Req, Resp, F> tonic::server::UnaryService<Req> for Handler<F>
where
    F: FnMut(Req) -> Result<Resp, Status>,
    Resp: Send + 'static,
{
    type Response = Resp;
    type Future = BoxFuture<tonic::Response<Resp>, Status>;

    fn call(&mut self, req: tonic::Request<Req>) -> Self::Future {
        let r = (self.0)(req.into_inner());
        Box::pin(async move { r.map(tonic::Response::new) })
    }
}

/// A fake gRPC service: routes `/<NAME>/<Method>` paths to `State` methods.
macro_rules! fake_service {
    ($ty:ident, $name:literal, { $($path:literal => $req:ty => $handler:ident),* $(,)? }) => {
        #[derive(Clone)]
        struct $ty(Arc<State>);

        impl NamedService for $ty {
            const NAME: &'static str = $name;
        }

        impl Service<http::Request<BoxBody>> for $ty {
            type Response = http::Response<BoxBody>;
            type Error = Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;

            fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
                let st = self.0.clone();
                match req.uri().path() {
                    $(
                        $path => Box::pin(async move {
                            let mut grpc = tonic::server::Grpc::new(ProstCodec::default());
                            Ok(grpc.unary(Handler(move |r: $req| st.$handler(r)), req).await)
                        }),
                    )*
                    _ => Box::pin(async { Ok(Status::unimplemented("fake").into_http()) }),
                }
            }
        }
    };
}

fake_service!(FakeTrading, "mt4_term_api.TradingHelper", {
    "/mt4_term_api.TradingHelper/OrderSend" => pb::OrderSendRequest => order_send,
    "/mt4_term_api.TradingHelper/OrderModify" => pb::OrderModifyRequest => order_modify,
    "/mt4_term_api.TradingHelper/OrderCloseDelete" => pb::OrderCloseDeleteRequest => order_close_delete,
    "/mt4_term_api.TradingHelper/OrderCloseBy" => pb::OrderCloseByRequest => order_close_by,
});

fake_service!(FakeAccountHelper, "mt4_term_api.AccountHelper", {
    "/mt4_term_api.AccountHelper/AccountSummary" => pb::AccountSummaryRequest => account_summary,
});

fake_service!(FakeConnection, "mt4_term_api.Connection", {
    "/mt4_term_api.Connection/ConnectEx" => pb::ConnectExRequest => connect_ex,
});

/// Start the fake gateway and return a client wired to it (plain HTTP/2, stored connect params so
/// `reconnect()` works).
async fn fake_gateway(mode: Mode) -> (Arc<State>, MT4Account) {
    let st = Arc::new(State {
        mode: Mutex::new(mode),
        trade_calls: AtomicU32::new(0),
        read_calls: AtomicU32::new(0),
        connect_calls: AtomicU32::new(0),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
    let router = Server::builder()
        .add_service(FakeTrading(st.clone()))
        .add_service(FakeAccountHelper(st.clone()))
        .add_service(FakeConnection(st.clone()));
    tokio::spawn(router.serve_with_incoming(incoming));

    let acc = MT4Account::with_api_key("127.0.0.1", addr.port(), "test-key")
        .with_id("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
    *acc.channel.lock().unwrap() = Some(Endpoint::from_shared(format!("http://{addr}")).unwrap().connect_lazy());
    *acc.params.lock().unwrap() = Some(ConnectParams {
        login: 1,
        password: "x".into(),
        server: "Fake-Demo".into(),
        name: None,
        timeout_seconds: 5,
    });
    (st, acc)
}

fn send_req() -> pb::OrderSendRequest {
    pb::OrderSendRequest { symbol: "EURUSD".into(), volume: 0.01, ..Default::default() }
}

fn assert_outcome_unknown(e: &MT4Error, op: &str, code: Code) {
    match e {
        MT4Error::TradeOutcomeUnknown { operation, status } => {
            assert_eq!(operation, op);
            assert_eq!(status.code(), code);
        }
        other => panic!("expected TradeOutcomeUnknown, got {other:?}"),
    }
    assert!(e.is_trade_outcome_unknown());
    assert_eq!(e.api_code(), "TRADE_OUTCOME_UNKNOWN");
    assert_eq!(e.grpc_status().map(|s| s.code()), Some(code));
    assert!(!e.is_retryable(), "an unknown trade outcome must not look retryable");
    assert!(std::error::Error::source(e).is_some(), "the original status is the source");
    let msg = e.to_string();
    assert!(msg.starts_with(&format!("TRADE_OUTCOME_UNKNOWN: {op} failed with gRPC ")), "{msg}");
    assert!(msg.contains("It was NOT re-sent automatically"), "{msg}");
}

#[tokio::test]
async fn order_send_unavailable_is_not_resent() {
    let (st, acc) = fake_gateway(Mode::TradeStatus(Code::Unavailable)).await;
    let e = acc.order_send(send_req()).await.unwrap_err();
    assert_outcome_unknown(&e, "OrderSend", Code::Unavailable);
    assert!(e.to_string().contains("gRPC UNAVAILABLE"));
    assert_eq!(st.trade_calls.load(Ordering::SeqCst), 1, "OrderSend must be sent exactly once");
    assert_eq!(st.connect_calls.load(Ordering::SeqCst), 0, "no reconnect");

    // The service layer passes it through unchanged.
    let svc = crate::mt4::service::MT4Service::new(acc);
    let e = svc.order_send(send_req()).await.unwrap_err();
    assert_outcome_unknown(&e, "OrderSend", Code::Unavailable);
    assert_eq!(st.trade_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn every_trade_and_unknown_outcome_code_is_not_resent() {
    for code in [Code::Unavailable, Code::DeadlineExceeded, Code::Unknown, Code::Internal] {
        let (st, acc) = fake_gateway(Mode::TradeStatus(code)).await;
        let e = acc.order_send(send_req()).await.unwrap_err();
        assert_outcome_unknown(&e, "OrderSend", code);
        let e = acc
            .order_modify(pb::OrderModifyRequest { order_ticket: 1, ..Default::default() })
            .await
            .unwrap_err();
        assert_outcome_unknown(&e, "OrderModify", code);
        let e = acc
            .order_close_delete(pb::OrderCloseDeleteRequest { order_ticket: 1, lots: Some(0.01), ..Default::default() })
            .await
            .unwrap_err();
        assert_outcome_unknown(&e, "OrderCloseDelete", code);
        let e = acc
            .order_close_by(pb::OrderCloseByRequest { ticket_to_close: 1, opposite_ticket_closing_by: 2 })
            .await
            .unwrap_err();
        assert_outcome_unknown(&e, "OrderCloseBy", code);
        assert_eq!(st.trade_calls.load(Ordering::SeqCst), 4, "{code:?}: one call per trade");
        assert_eq!(st.connect_calls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn order_send_terminal_not_found_reconnects_and_resends() {
    let (st, acc) = fake_gateway(Mode::NotFoundOnce).await;
    let d = acc.order_send(send_req()).await.unwrap();
    assert_eq!(d.ticket, 7);
    assert_eq!(st.trade_calls.load(Ordering::SeqCst), 2);
    assert_eq!(st.connect_calls.load(Ordering::SeqCst), 1, "one reconnect");
}

#[tokio::test]
async fn read_unavailable_is_retried() {
    let (st, acc) = fake_gateway(Mode::ReadUnavailableOnce).await;
    let d = acc.account_summary().await.unwrap();
    assert_eq!(d.account_login, 42);
    assert_eq!(st.read_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn api_key_invalid_fails_fast_and_keeps_code() {
    let (st, acc) = fake_gateway(Mode::ApiKeyInvalid).await;
    let e = acc.order_send(send_req()).await.unwrap_err();
    assert_eq!(e.api_code(), "API_KEY_INVALID");
    assert!(e.is_api_key_refused());
    assert_eq!(st.trade_calls.load(Ordering::SeqCst), 1);

    let e = acc.account_summary().await.unwrap_err();
    assert_eq!(e.api_code(), "API_KEY_INVALID");
    assert_eq!(st.read_calls.load(Ordering::SeqCst), 1);
    assert_eq!(st.connect_calls.load(Ordering::SeqCst), 0, "no reconnect");

    *st.mode.lock().unwrap() = Mode::ApiKeyStatus;
    let e = acc.account_summary().await.unwrap_err();
    assert!(e.is_api_key_refused(), "{e:?}");
    assert!(e.to_string().contains("API_KEY_REQUIRED"));
    assert_eq!(st.read_calls.load(Ordering::SeqCst), 2, "the status-carried refusal is not retried");
    assert_eq!(st.connect_calls.load(Ordering::SeqCst), 0);
}
