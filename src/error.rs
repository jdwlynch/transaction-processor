///Errors specific to transaction-processor. Uses thiserror to hide implementation details.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CSV Error")]
    Csv(#[from] csv::Error),
    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),
    ///Any issue with a transaction from formatting to violating transaction rules.
    #[error("Transaction Error: {0}")]
    Transaction(String),
    ///Any issue actioning a clients account, from locked to insufficient funds and more.
    #[error("Client Error: {0}")]
    Client(String),
    #[error("IO Error")]
    Io(#[from] std::io::Error),
}
