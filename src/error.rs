#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CSV Error")]
    CsvError(#[from] csv::Error),
    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),
    #[error("Transaction Error: {0}")]
    TransactionError(String),
    #[error("Account Error: {0}")]
    AccountError(String),
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
}

//pub type Result<T> = result::Result<T, Error>;
