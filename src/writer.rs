use std::io;

use crate::{error, Accounts};

pub fn write_accounts(accounts: &Accounts) -> Result<(), error::Error> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // When writing records with Serde using structs, the header row is written
    // automatically.
    for (_client_id, client_record) in &accounts.clients {
        wtr.serialize(client_record)?;
    }
    wtr.flush()?;
    Ok(())
}
