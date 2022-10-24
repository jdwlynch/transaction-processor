
mod models;
mod transaction_feed;
mod writer;
mod error;
pub mod processor;

use crate::transaction_feed::TransactionFeed;
use crate::models::transaction;
use crate::models::accounts::{Accounts};
use crate::processor::Processor;

fn main() -> Result<(), error::Error> {
/*    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }*/
    let cons = TransactionFeed::new()?;
    let clients = Accounts::new();
    let result = Processor::process_transactions(cons, clients);
    result
}
