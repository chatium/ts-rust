//! Go package `internal/contentmapper` (tsgo#4712).
//!
//! PORT: the host is dispatch-thread state, like the ipc connection it
//! drives (`ipc::AsyncConn` is `Rc`-based) and the frontend values it takes
//! (`Rc<CompilerOptions>`, `Rc<Mapper>`). Go's mutexes and atomics on host
//! state are `Cell` and `RefCell` (PORTING.md "Go runtime"). Only the
//! spawned process connections (`ipc::ReadWriteCloser`) and the stderr
//! logger cross threads.

pub mod contentmapper;
pub mod host;
pub mod hostimpl;
pub mod muxconn;
pub mod transform;

pub use contentmapper::*;
pub use host::*;
pub use hostimpl::*;
pub use transform::*;

/// Glob import for contentmapper files: `use crate::contentmapper::prelude::*;`.
pub mod prelude {
    pub use super::{contentmapper::*, host::*, hostimpl::*, transform::*};
    // The package's own `Diagnostic` (hostimpl.go) wins over the crate
    // prelude's `Diagnostic` (Go `ast.Diagnostic`), as in Go. Write
    // `crate::core::Diagnostic` for the AST diagnostic.
    pub use super::hostimpl::Diagnostic;
    // Go `ast.MappedDiagnosticDirective` (tsgo#4712) has the
    // same name; the package's own wire type wins. Write
    // `ast::MappedDiagnosticDirective` for the AST one.
    pub use super::hostimpl::MappedDiagnosticDirective;
    pub use crate::frontend::json::{
        JsonDecoder, JsonError, JsonToken, MarshalerTo, UnmarshalerFrom, json_marshal,
        json_unmarshal, json_unmarshal_decode,
    };
    pub use crate::frontend::json_ext::{self, JsonValue, LspAny};
    pub use crate::frontend::{parser, tspath};
    pub use crate::gostd::{self, Context, GoError, errors};
    pub use crate::prelude::*;
    pub use crate::{ast, ipc, jsonrpc, locale, spanmap};
}
