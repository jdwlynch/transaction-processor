#![warn(missing_docs)]
/*!
# Introduction
The 'transaction-processor' is a toy financial transactions engine designed to explore the
intricacies of handling five different transaction types:

- Deposits
- Withdrawals
- Disputes
- Resolves
- Chargebacks

# Input
transaction-processor takes a .csv file of transactions as input in the following form:

|   types    |  client  |    tx    |  amount  |
|------------|----------|----------|----------|
| deposit    |    1     |    1     |   1.0    |
| deposit    |    2     |    2     |   3.0    |
| deposit    |    1     |    3     |   1.5    |
|withdrawal  |    1     |    4     |   1.5    |
|withdrawal  |    1     |    5     |   1.5    |

See the test-inputs directory for sample input.

# Output
transaction-processor will output to stdout in csv form as well following this format:

|   client   |available |   held   |  total   |
|------------|----------|----------|----------|
| 1          |    3.4   |   1.0503 |   3.4503 |
| 2          |    2.1   |   0.0    |   2.1    |

# Usage
To run the program, run the following:

```ignore
$ cargo run --test input-file.csv < output-file.csv
```
Details of the rules engine are omitted from this documentation[^note].

[^note]: For detailed discussion on the theory, motivation, and rules around this engine,
see the README

*/
mod client_repo;
mod error;
mod models;
mod processor;
mod transaction_feed;
mod writer;

use crate::models::transaction;
use crate::processor::Processor;
use crate::transaction_feed::TransactionFeed;
use crate::writer::write_client_data;
use client_repo::ClientRepo;
use log::error;
use std::env;
use std::ffi::OsString;
use env_logger::Env;

///Gets the first command line argument and returns it, or an error::Error if no arg is found.
fn get_first_arg() -> Result<OsString, error::Error> {
    match env::args_os().nth(1) {
        None => Err(error::Error::InvalidArgument(String::from("No argument found"))),
        Some(file_path) => Ok(file_path),
    }
}

///Gets the input filename from command line args and uses it to instantiate a feed of
/// Transaction items from the csv input.
fn get_transaction_feed() -> Result<TransactionFeed, error::Error> {
    let file_path = match get_first_arg() {
        Err(err) => {
            error!("[!] Fatal error parsing command line args: {:?}", err);
            return Err(err);
        }
        Ok(file_path) => file_path,
    };
    TransactionFeed::new(file_path)
}

fn main() -> Result<(), error::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("off")).init();
    let transaction_feed = match get_transaction_feed() {
        Err(err) => {
            error!("[!] Fatal error opening transaction feed: {:?}", err);
            return Err(err);
        }
        Ok(transaction_feed) => transaction_feed,
    };

    let mut repo = ClientRepo::new();

    Processor::handle_transactions(transaction_feed, &mut repo);

    if let Err(err) = write_client_data(&repo) {
        error!("[!] Fatal error writing transactions: {:?}", err);
        return Err(err);
    }
    Ok(())
}
