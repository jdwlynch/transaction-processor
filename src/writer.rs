use std::error::Error;
use std::io;

use crate::Accounts;

pub fn write_accounts(accounts: & Accounts) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // When writing records with Serde using structs, the header row is written
    // automatically.
    for (_client_id, client_record) in &accounts.clients{
        wtr.serialize(client_record)?;
    }
    wtr.flush()?;
    Ok(())
}