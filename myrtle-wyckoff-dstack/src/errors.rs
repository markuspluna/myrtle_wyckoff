use rocket::{
    http::Status,
    response::{status::Custom, Responder},
    serde::json::Json,
    Request,
};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum MwError {
    InvalidSignature,
    InsufficientBalance { token: String },
    InvalidTimestamp,
    OrderNotFound { order_id: u32 },
    UnauthorizedAccess,
    InvalidOrderParams,
    NotTaker,
    InvalidRequestType,
    SignatureRecoveryError,
    SignerCreationError,
    SigningError,
    SignatureConversionError,
    TransactionError,
    EncryptionError,
    InvalidBook,
    NoOrdersFound,
    SnapshotError(String),
    GulpError(String),
}

impl fmt::Display for MwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InsufficientBalance { token } => {
                write!(f, "Insufficient balance for token {}", token)
            }
            Self::InvalidTimestamp => write!(f, "Invalid timestamp"),
            Self::OrderNotFound { order_id } => write!(f, "Order {} not found", order_id),
            Self::UnauthorizedAccess => write!(f, "Unauthorized access"),
            Self::InvalidOrderParams => write!(f, "Invalid order parameters"),
            Self::NotTaker => write!(f, "Not taker"),
            Self::InvalidRequestType => write!(f, "Invalid request type"),
            Self::SignatureRecoveryError => write!(f, "Signature recovery error"),
            Self::SignerCreationError => write!(f, "Signer creation error"),
            Self::SigningError => write!(f, "Signing error"),
            Self::SignatureConversionError => write!(f, "Signature conversion error"),
            Self::TransactionError => write!(f, "Transaction error"),
            Self::EncryptionError => write!(f, "Encryption error"),
            Self::InvalidBook => write!(f, "Invalid book"),
            Self::NoOrdersFound => write!(f, "No orders found"),
            Self::SnapshotError(message) => write!(f, "Snapshot error: {}", message),
            Self::GulpError(message) => write!(f, "Gulp error: {}", message),
        }
    }
}

impl std::error::Error for MwError {}

impl MwError {
    fn status_code(&self) -> Status {
        match self {
            Self::InvalidSignature => Status::Unauthorized,
            Self::InsufficientBalance { .. } => Status::BadRequest,
            Self::InvalidTimestamp => Status::BadRequest,
            Self::OrderNotFound { .. } => Status::NotFound,
            Self::UnauthorizedAccess => Status::Unauthorized,
            Self::InvalidOrderParams => Status::BadRequest,
            Self::NotTaker => Status::Unauthorized,
            Self::InvalidRequestType => Status::BadRequest,
            Self::SignatureRecoveryError => Status::BadRequest,
            Self::SignerCreationError => Status::BadRequest,
            Self::SigningError => Status::BadRequest,
            Self::SignatureConversionError => Status::BadRequest,
            Self::TransactionError => Status::BadRequest,
            Self::EncryptionError => Status::BadRequest,
            Self::InvalidBook => Status::BadRequest,
            Self::NoOrdersFound => Status::NotFound,
            Self::SnapshotError(_) => Status::BadRequest,
            Self::GulpError(_) => Status::BadRequest,
        }
    }
}

impl<'r> Responder<'r, 'static> for MwError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        Custom(self.status_code(), Json(self)).respond_to(req)
    }
}
