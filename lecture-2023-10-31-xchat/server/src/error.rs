use thiserror::Error;


#[derive(Error, Debug)]
pub enum ServerError {
    #[error("failed port binding: {0}")]
    PortBindError(String),
    #[error("failed to establish client connection: {0}")]
    ClientConnectionError(String),
    #[error("failed to get peer address after client connection: {0}")]
    ClientPeerAddressError(String),
    #[error("failed stream configuration after client connection: {0}")]
    ClientStreamConfigError(String),
    #[error("internal error: detected poisoned mutex")]
    SharedMutexPoisonedError,
    #[error("failed to forward message to {address}: {detail} ")]
    ForwardMessageError{ address: String, detail: String },
    #[error("DB error: {0}")]
    DBError(String),
    #[error("web server error: {0}")]
    WebServerError(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("join error: {0}")]
    JoinError(String),
}
