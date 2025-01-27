use std::fmt::{Display, Formatter};
use std::io::{ErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CuteErrorCode {
    /// Task Input 및 bin_code serde 실패시
    SerializeInvalid = 1,
    /// Task Input 및 bin_code serde 실패시
    DeSerializeInvalid = 2,
    /// TimeOut 발생시 사용.
    DeadlineExceeded = 3,
    /// 권한 실패로 인해 실패시 사용.
    PermissionDenied = 4,
    /// key 값 등을 찾지 못하였을 경우
    NotFound = 5,
    /// 내부적 에러. Task Execute 동작 과정 중에 나오면 사용.
    ///
    /// `Task<C>::execute()` 를 작성한 사람이 잘 만들어서 사용해주기 바람.
    Internal = 6,
    /// channel 이 cancelled 되는 경우 사용
    Cancelled = 7,
    /// `ProxyWorker` 와 같이 권한 대행을 실행 후 실패한 경우 사용.
    Unauthenticated = 8,
    /// Error 는 발생했지만 Error 로 처리하고 싶지 않는 경우 사용.
    Ok = 9
}

#[derive(Debug, Clone)]
pub struct CuteError {
    pub code : CuteErrorCode,
    pub message : String,
}

impl Default for CuteError {
    fn default() -> Self {
        Self {
            code : CuteErrorCode::Ok,
            message: "".to_string(),
        }
    }
}

impl CuteError {
    fn new(code : CuteErrorCode, msg : impl Into<String>) -> Self {
        Self {
            code,
            message: msg.into()
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        format!("{:?} : {}",self.code,self.message).as_bytes().to_vec()
    }

    pub fn serialize_invalid(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::SerializeInvalid, msg)
    }

    pub fn deserialize_invalid(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::DeSerializeInvalid, msg)
    }

    pub fn deadline_exceeded(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::DeadlineExceeded, msg)
    }

    pub fn permission_denied(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::PermissionDenied, msg)
    }

    pub fn not_found(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::NotFound, msg)
    }

    pub fn internal(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::Internal, msg)
    }

    pub fn cancelled(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::Cancelled, msg)
    }

    pub fn unauthenticated(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::Unauthenticated, msg)
    }

    pub fn ok(msg : impl Into<String>) -> CuteError {
        CuteError::new(CuteErrorCode::Ok, msg)
    }
}

impl Display for CuteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} : {}",self.code, self.message)
    }
}

impl From<std::io::Error> for CuteError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            ErrorKind::InvalidInput => {
                Self::serialize_invalid(value.to_string())
            }
            ErrorKind::InvalidData => {
                Self::deserialize_invalid(value.to_string())
            }
            ErrorKind::TimedOut => {
                Self::deadline_exceeded(value.to_string())
            }
            ErrorKind::PermissionDenied => {
                Self::permission_denied(value.to_string())
            }
            ErrorKind::NotFound => {
                Self::not_found(value.to_string())
            }
            ErrorKind::WouldBlock => {
                Self::ok(value.to_string())
            }
            ErrorKind::Interrupted => {
                Self::cancelled(value.to_string())
            }
            ErrorKind::Unsupported => {
                Self::unauthenticated(value.to_string())
            }
            _ => {
                Self::internal(format!("{}. {}",value.kind(),value.to_string()))
            }
        }
    }
}

impl From<CuteError> for std::io::Error {
    fn from(value: CuteError) -> Self {
        let kind = match value.code {
            CuteErrorCode::SerializeInvalid => ErrorKind::InvalidInput,
            CuteErrorCode::DeSerializeInvalid => ErrorKind::InvalidData,
            CuteErrorCode::DeadlineExceeded => ErrorKind::TimedOut,
            CuteErrorCode::PermissionDenied => ErrorKind::PermissionDenied,
            CuteErrorCode::NotFound => ErrorKind::NotFound,
            CuteErrorCode::Ok => ErrorKind::WouldBlock,
            CuteErrorCode::Cancelled => ErrorKind::Interrupted,
            CuteErrorCode::Unauthenticated => ErrorKind::Unsupported,
            CuteErrorCode::Internal => ErrorKind::Other,
        };

        std::io::Error::new(kind, value.message)
    }
}