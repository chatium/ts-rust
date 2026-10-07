//! Port of internal/ipc/conn.go (internal/api/conn.go before tsgo#4712).
//!
//! PORT: the API session and project state live on one thread, so the
//! handler and the connections are `Rc` values and their methods take
//! `&self`. Go `json.Value` is `JsonValue`; Go `any` params and results are
//! `Option<Box<dyn AnyValue>>` (nil is `None`).

use crate::ipc::prelude::*;

use crate::frontend::json::UnmarshalerFrom;
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::gostd::{Context, GoError, errors};
use std::sync::LazyLock;

// Go: ipc/conn.go:10
pub static ERR_CONN_CLOSED: LazyLock<GoError> =
    LazyLock::new(|| errors::new("ipc: connection closed"));
pub static ERR_REQUEST_TIMEOUT: LazyLock<GoError> =
    LazyLock::new(|| errors::new("ipc: request timeout"));

// Go: ipc/conn.go:16 Handler
// Handler processes incoming API requests and notifications.
pub trait Handler {
    // HandleRequest handles an incoming request and returns a result or error.
    fn handle_request(
        &self,
        ctx: &Context,
        method: &str,
        params: JsonValue,
    ) -> Result<Option<Box<dyn AnyValue>>, GoError>;
    // HandleNotification handles an incoming notification.
    fn handle_notification(
        &self,
        ctx: &Context,
        method: &str,
        params: JsonValue,
    ) -> Result<(), GoError>;
}

// Go: ipc/conn.go:24 Conn
// Conn represents a bidirectional connection for API communication.
pub trait Conn {
    // Run starts processing messages on the connection.
    // It blocks until the context is cancelled or an error occurs.
    fn run(&self, ctx: &Context) -> Result<(), GoError>;

    // Call sends a request to the client and waits for a response.
    fn call(
        &self,
        ctx: &Context,
        method: &str,
        params: Option<Box<dyn AnyValue>>,
    ) -> Result<JsonValue, GoError>;

    // Notify sends a notification to the client (no response expected).
    fn notify(
        &self,
        ctx: &Context,
        method: &str,
        params: Option<Box<dyn AnyValue>>,
    ) -> Result<(), GoError>;

    // PORT: not in Go. Lets an owner find its own connection type behind
    // `Rc<dyn Conn>` (the content mapper host's `ProcessConn`).
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

// Go: ipc/conn.go:37 UnmarshalParams
// UnmarshalParams is a helper to unmarshal params into a typed struct.
// PORT: Go returns `*T`; nil is `None`.
pub fn unmarshal_params<T: UnmarshalerFrom + Default>(
    params: impl AsRef<[u8]>,
) -> Result<Option<T>, GoError> {
    let params = params.as_ref();
    if params.is_empty() {
        return Ok(None);
    }
    let mut v = T::default();
    // Go `json.Unmarshal(params, &v)`, with the v2 error texts.
    if let Err(err) = crate::frontend::json_ext::unmarshal_root(params, &mut v) {
        return Err(errors::from_value(err));
    }
    Ok(Some(v))
}

/// Go `%v` of the value `recover()` returns in the connections' request
/// handlers.
/// PORT: a Rust panic payload is the panic message (`&str` or `String`,
/// or the message of a `core::go_panic`); any other payload has no text.
pub fn recovered_value(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(panic) = payload.downcast_ref::<crate::core::GoPanic>() {
        panic.message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::new()
    }
}
