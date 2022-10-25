use crate::{error, ClientRepo};
use log::trace;
use std::io;

/// Creates a writer to serialize all Client data to csv and output to stdout.
pub fn write_client_data(repo: &ClientRepo) -> Result<(), error::Error> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    for (client_id, client_record) in &repo.clients {
        trace!("Writing record for client #{}", client_id);
        wtr.serialize(client_record)?;
    }
    wtr.flush()?;
    Ok(())
}
