mod error;
mod models;
pub mod processor;
mod transaction_feed;
mod writer;

use crate::models::accounts::Accounts;
use crate::models::transaction;
use crate::processor::Processor;
use crate::transaction_feed::TransactionFeed;
use crate::writer::write_accounts;
use env_logger;
use log::{debug, error, info, trace, warn};
use std::env;
use std::ffi::OsString;

fn get_first_arg() -> Result<OsString, error::Error> {
    match env::args_os().nth(1) {
        None => Err(error::Error::InvalidArgument(String::from(
            "No argument found",
        ))),
        Some(file_path) => Ok(file_path),
    }
}

fn get_transaction_feed() -> Result<TransactionFeed, error::Error> {
    let file_path = match get_first_arg() {
        Err(err) => {
            error!("[!] Fatal error parsing command line args: {}", err);
            return Err(err);
        }
        Ok(file_path) => file_path,
    };
    TransactionFeed::new(file_path)
}

fn main() -> Result<(), error::Error> {
    env_logger::init();
    let transaction_feed = match get_transaction_feed() {
        Err(err) => {
            error!("[!] Fatal error opening transaction feed: {}", err);
            return Err(err);
        }
        Ok(transaction_feed) => transaction_feed,
    };

    let mut accounts = Accounts::new();

    Processor::handle_transactions(transaction_feed, &mut accounts);

    if let Err(err) = write_accounts(&accounts) {
        error!("[!] Fatal error writing transactions: {}", err);
        return Err(err);
    }
    Ok(())
}
