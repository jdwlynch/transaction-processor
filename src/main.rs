
mod models;
mod transaction_feed;
mod writer;
mod error;
pub mod processor;

use crate::transaction_feed::TransactionFeed;
use crate::models::transaction;
use crate::models::accounts::{Accounts};
use crate::processor::Processor;
use crate::writer::write_accounts;


fn main() -> Result<(), error::Error> {
/*    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }*/
    let cons = TransactionFeed::new()?;
    let mut accounts = Accounts::new();
    let result = Processor::process_transactions(cons, &mut accounts);
    write_accounts(&accounts);
    result
}
