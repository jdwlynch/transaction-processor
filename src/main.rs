mod error;
mod models;
pub mod processor;
mod transaction_feed;
mod writer;

use std::process;
use crate::models::accounts::Accounts;
use crate::models::transaction;
use crate::processor::Processor;
use crate::transaction_feed::TransactionFeed;
use crate::writer::write_accounts;
use env_logger;
use log::{error, warn, info, debug, trace};

fn main() -> Result<(), error::Error> {
    env_logger::init();
    let transaction_feed = match TransactionFeed::new(){
        Err(err) => {
            error!("[!] Fatal error opening transaction feed: {}", err);
            return Err(err);
        },
        Ok(transaction_feed) => transaction_feed
    };

    let mut accounts = Accounts::new();

    Processor::handle_transactions(transaction_feed, &mut accounts);

    if let Err(err) = write_accounts(&accounts){
        error!("[!] Fatal error writing transactions: {}", err);
        return Err(err);
    }
    Ok(())
}
