//! A content mapper connection that several threads can call at once.
//!
//! PORT: not in Go, where `ipc.AsyncConn` is already concurrent: its `Run`
//! goroutine reads, and each `Call` waits for its own response. The ipc port
//! reads each response inside `call` on the dispatch thread
//! (ipc/conn_async.rs), so a mapper got one request at a time, while the
//! parse goroutines of Go keep many in flight. This connection reads on its
//! own thread, as `Run` does, and routes each response to the call that waits
//! for it, so the parse workers can transform content-mapped files while the
//! loader works (`FilesParser::prefetch_request`). The mapper protocol has no
//! requests from the mapper: as `AsyncConn` with the host's `RejectHandler`,
//! the reader answers one with an error and ignores notifications.

use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::gostd::{Context, GoError, errors};
use crate::ipc::{self, ERR_CONN_CLOSED, Message};
use crate::jsonrpc;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

/// Makes a protocol over the connection's transport. The reader thread reads
/// with its own; each write uses a new one under the write lock, so no
/// protocol value is shared between threads.
pub type ProtocolFactory = Arc<dyn Fn() -> Box<dyn ipc::Protocol> + Send + Sync>;

/// How often a waiting call looks at its context: a response wakes it at
/// once, a cancelled context within this time.
const CONTEXT_POLL: Duration = Duration::from_millis(20);

pub struct MuxConn {
    new_protocol: ProtocolFactory,
    /// Held for each whole message write.
    write: Mutex<()>,
    /// The calls that wait for a response, by request id.
    pending: Mutex<FxHashMap<jsonrpc::ID, SyncSender<Message>>>,
    /// What every call returns once the reader stopped (Go `terminal`).
    terminal: Mutex<Option<GoError>>,
    seq: AtomicI64,
}

fn lock<T: ?Sized>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl MuxConn {
    /// Starts the reader thread of a connection to a started process.
    pub fn start(new_protocol: ProtocolFactory) -> Arc<MuxConn> {
        let conn = Arc::new(MuxConn {
            new_protocol,
            write: Mutex::new(()),
            pending: Mutex::new(FxHashMap::default()),
            terminal: Mutex::new(None),
            seq: AtomicI64::new(0),
        });
        let reader = conn.clone();
        std::thread::Builder::new()
            .name("content mapper reader".to_string())
            .spawn(move || reader.read_loop())
            .expect("start the content mapper reader thread");
        conn
    }

    // Go: ipc/conn_async.go Run (the read loop), for a client connection.
    fn read_loop(&self) {
        let mut protocol = (self.new_protocol)();
        loop {
            let msg = match protocol.read_message() {
                Ok(msg) => msg,
                Err(err) => {
                    // Go `Run` ends without an error at EOF.
                    let cause = (!errors::is(&err, &errors::EOF)).then_some(err);
                    self.stop(cause);
                    return;
                }
            };
            if msg.is_response() {
                let Some(id) = &msg.id else { continue };
                let waiting = lock(&self.pending).remove(id);
                if let Some(waiting) = waiting {
                    // The call may have stopped waiting (its context ended).
                    let _ = waiting.send(msg);
                }
            } else if msg.is_request() {
                // Go: rejectHandler.HandleRequest, answered by AsyncConn.
                let error = jsonrpc::ResponseError {
                    code: jsonrpc::CODE_INTERNAL_ERROR,
                    message: format!("content mapper sent an unexpected request: {}", msg.method),
                    data: None,
                };
                let written = {
                    let _write = lock(&self.write);
                    (self.new_protocol)().write_error(msg.id.as_ref(), &error)
                };
                if let Err(err) = written {
                    self.stop(Some(errors::errorf(
                        format!("ipc: failed to write response: {}", err.error()),
                        vec![err],
                    )));
                    return;
                }
            }
            // A notification: Go rejectHandler.HandleNotification ignores it.
        }
    }

    // Go: ipc/conn_async.go closePendingCalls with recordTerminalErrorLocked.
    fn stop(&self, cause: Option<GoError>) {
        {
            let mut terminal = lock(&self.terminal);
            if terminal.is_none() {
                *terminal = Some(match cause {
                    Some(cause) => errors::join([ERR_CONN_CLOSED.clone(), cause])
                        .expect("both errors are non-nil"),
                    None => ERR_CONN_CLOSED.clone(),
                });
            }
        }
        // Dropping the senders wakes every waiting call.
        lock(&self.pending).clear();
    }

    fn terminal(&self) -> Option<GoError> {
        lock(&self.terminal).clone()
    }
}

impl ipc::Conn for MuxConn {
    /// The reader thread runs from `start`; this waits for it to end.
    fn run(&self, ctx: &Context) -> Result<(), GoError> {
        loop {
            if self.terminal().is_some() {
                return Ok(());
            }
            if let Some(err) = ctx.err() {
                return Err(err);
            }
            std::thread::sleep(CONTEXT_POLL);
        }
    }

    // Go: ipc/conn_async.go:256 Call
    fn call(
        &self,
        ctx: &Context,
        method: &str,
        params: Option<Box<dyn AnyValue>>,
    ) -> Result<JsonValue, GoError> {
        if let Some(err) = self.terminal() {
            return Err(err);
        }
        let id = jsonrpc::new_id_string(&format!(
            "api{}",
            self.seq.fetch_add(1, Ordering::Relaxed) + 1
        ));
        // Register the response channel before the request is sent.
        let (sender, receiver) = mpsc::sync_channel(1);
        lock(&self.pending).insert(id.clone(), sender);
        let forget = || {
            lock(&self.pending).remove(&id);
        };
        // The reader may have stopped before the registration.
        if let Some(err) = self.terminal() {
            forget();
            return Err(err);
        }
        let written = {
            let _write = lock(&self.write);
            (self.new_protocol)().write_request(Some(&id), method, params)
        };
        if let Err(err) = written {
            forget();
            return Err(err);
        }
        loop {
            if let Some(err) = ctx.err() {
                forget();
                return Err(err);
            }
            match receiver.recv_timeout(CONTEXT_POLL) {
                Ok(resp) => {
                    if let Some(error) = &resp.error {
                        return Err(errors::new(format!(
                            "ipc: remote error [{}]: {}",
                            error.code, error.message
                        )));
                    }
                    return Ok(resp.result);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(self.terminal().unwrap_or_else(|| ERR_CONN_CLOSED.clone()));
                }
            }
        }
    }

    // Go: ipc/conn_async.go:307 Notify
    fn notify(
        &self,
        ctx: &Context,
        method: &str,
        params: Option<Box<dyn AnyValue>>,
    ) -> Result<(), GoError> {
        if let Some(err) = ctx.err() {
            return Err(err);
        }
        if let Some(err) = self.terminal() {
            return Err(err);
        }
        let _write = lock(&self.write);
        (self.new_protocol)().write_notification(method, params)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::gostd::context;
    use crate::ipc::{Conn, ReadWriteCloser};
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    /// The client end of a socket pair. `close` shuts it down, which ends a
    /// blocked read.
    struct End(UnixStream);

    impl ReadWriteCloser for End {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            (&self.0).read(buf)
        }

        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            (&self.0).write(buf)
        }

        fn flush(&self) -> std::io::Result<()> {
            (&self.0).flush()
        }

        fn close(&self) -> Result<(), GoError> {
            let _ = self.0.shutdown(std::net::Shutdown::Both);
            Ok(())
        }
    }

    fn connect() -> (Arc<MuxConn>, UnixStream) {
        let (client, server) = UnixStream::pair().expect("socket pair");
        let client: Arc<dyn ReadWriteCloser> = Arc::new(End(client));
        let conn = MuxConn::start(Arc::new(move || {
            Box::new(ipc::new_jsonrpc_protocol(client.clone())) as Box<dyn ipc::Protocol>
        }));
        (conn, server)
    }

    /// Reads one message with its `Content-Length` header.
    fn read_framed(stream: &UnixStream) -> String {
        let mut header = Vec::new();
        let mut byte = [0u8; 1];
        while !header.ends_with(b"\r\n\r\n") {
            (&*stream).read_exact(&mut byte).expect("header byte");
            header.push(byte[0]);
        }
        let length: usize = String::from_utf8(header)
            .expect("utf-8 header")
            .trim()
            .strip_prefix("Content-Length: ")
            .and_then(|length| length.parse().ok())
            .expect("Content-Length header");
        let mut body = vec![0u8; length];
        (&*stream).read_exact(&mut body).expect("body");
        String::from_utf8(body).expect("utf-8 body")
    }

    fn write_framed(stream: &UnixStream, body: &str) {
        write!(&*stream, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("write");
    }

    /// The value of `"key":"…"` in a message.
    fn string_field(message: &str, key: &str) -> String {
        let start = message.find(&format!("\"{key}\":\"")).expect(key) + key.len() + 4;
        message[start..].split('"').next().unwrap().to_string()
    }

    // Several threads call at once; the peer answers in the reverse order.
    #[test]
    fn concurrent_calls_get_their_own_responses() {
        const CALLS: i32 = 8;
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            let requests: Vec<String> = (0..CALLS).map(|_| read_framed(&server)).collect();
            for request in requests.iter().rev() {
                let id = string_field(request, "id");
                let params = &request[request.find("\"params\":").unwrap() + 9..];
                let n = params.trim_end_matches('}');
                write_framed(
                    &server,
                    &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":{n}}}"#),
                );
            }
            server
        });
        let callers: Vec<_> = (0..CALLS)
            .map(|n| {
                let conn = conn.clone();
                std::thread::spawn(move || {
                    let result = conn
                        .call(&context::background(), "echo", Some(Box::new(n)))
                        .expect("the call returns");
                    assert_eq!(result.0, n.to_string().as_bytes());
                })
            })
            .collect();
        for caller in callers {
            caller.join().expect("caller thread");
        }
        drop(peer.join().expect("peer thread"));
    }

    #[test]
    fn call_returns_when_peer_closes() {
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            read_framed(&server);
            drop(server);
        });
        let err = conn
            .call(&context::background(), "transform", None)
            .expect_err("the call fails when the peer closes");
        peer.join().expect("peer thread");
        assert!(errors::is(&err, &ERR_CONN_CLOSED), "{}", err.error());
        // Later calls return the same error at once.
        let again = conn
            .call(&context::background(), "transform", None)
            .expect_err("a later call fails");
        assert!(errors::is(&again, &ERR_CONN_CLOSED), "{}", again.error());
    }

    // The mapper protocol has no requests from the mapper: the reader
    // answers one with an error, as `AsyncConn` with `RejectHandler` does.
    #[test]
    fn request_from_the_peer_is_rejected() {
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            let call = read_framed(&server);
            write_framed(
                &server,
                r#"{"jsonrpc":"2.0","id":"m1","method":"readFile"}"#,
            );
            let rejection = read_framed(&server);
            let id = string_field(&call, "id");
            write_framed(
                &server,
                &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":true}}"#),
            );
            rejection
        });
        let result = conn
            .call(&context::background(), "transform", None)
            .expect("the call returns");
        let rejection = peer.join().expect("peer thread");
        assert_eq!(result.0, b"true");
        assert!(
            rejection.contains(r#""id":"m1""#)
                && rejection.contains("content mapper sent an unexpected request: readFile"),
            "{rejection}"
        );
    }
}
