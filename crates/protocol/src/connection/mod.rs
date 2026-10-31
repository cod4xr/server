// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy connection command fields, without authentication or slot admission.

mod connect_error;
mod connect_request;

pub use connect_error::{ConnectError, ConnectField};
pub use connect_request::ConnectRequest;
