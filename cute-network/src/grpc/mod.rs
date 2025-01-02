use tonic::{Code, Status};
use cute_core::{CuteError, CuteErrorCode};


pub use self::server::GRPCServer;
pub use self::client::GRPCClient;

#[allow(unused)]
fn convert_cute_error_to_status(e : CuteError) -> Status {
    match e.code {
        CuteErrorCode::SerializeInvalid => {
            Status::invalid_argument(e.message)
        }
        CuteErrorCode::DeSerializeInvalid => {
            Status::unimplemented(e.message)
        }
        CuteErrorCode::DeadlineExceeded => {
            Status::deadline_exceeded(e.message)
        }
        CuteErrorCode::PermissionDenied => {
            Status::permission_denied(e.message)
        }
        CuteErrorCode::NotFound => {
            Status::not_found(e.message)
        }
        CuteErrorCode::Internal => {
            Status::internal(e.message)
        }
        CuteErrorCode::Cancelled => {
            Status::cancelled(e.message)
        }
        CuteErrorCode::Unauthenticated => {
            Status::unauthenticated(e.message)
        }
        CuteErrorCode::Ok => {
            Status::ok(e.message)
        }
    }
}
#[allow(unused)]
fn convert_status_to_cute_error(e : Status) -> CuteError {
    match e.code() {
        Code::Ok => {
            CuteError::ok(e.message())
        }
        Code::Cancelled => {
            CuteError::cancelled(e.message())
        }
        Code::InvalidArgument => {
            CuteError::serialize_invalid(e.message())
        }
        Code::DeadlineExceeded => {
            CuteError::deadline_exceeded(e.message())
        }
        Code::NotFound => {
            CuteError::not_found(e.message())
        }
        Code::PermissionDenied => {
            CuteError::permission_denied(e.message())
        }
        Code::Unimplemented => {
            CuteError::deserialize_invalid(e.message())
        }
        Code::Unauthenticated => {
            CuteError::unauthenticated(e.message())
        }
        _ => CuteError::internal(e.message()),
    }
}


mod proto;
mod server;
mod client;