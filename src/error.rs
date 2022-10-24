#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CSV Error")]
    Csv(#[from] csv::Error),
    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),
    #[error("Transaction Error: {0}")]
    Transaction(String),
    #[error("Account Error: {0}")]
    Account(String),
    #[error("IO Error")]
    Io(#[from] std::io::Error),
}

//pub type Result<T> = result::Result<T, Error>;
