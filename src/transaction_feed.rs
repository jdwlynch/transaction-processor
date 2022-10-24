use crate::error;
use crate::transaction::Transaction;
use csv::DeserializeRecordsIntoIter;
use std::env;
use std::ffi::OsString;
use std::fs::File;

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
fn get_first_arg() -> Result<OsString, error::Error> {
    match env::args_os().nth(1) {
        None => Err(error::Error::InvalidArgument(String::from(
            "No argument found",
        ))),
        Some(file_path) => Ok(file_path),
    }
}

pub struct TransactionFeed {
    iter: DeserializeRecordsIntoIter<File, Transaction>,
}

impl TransactionFeed {
    pub fn new() -> Result<TransactionFeed, error::Error> {
        let file_path = get_first_arg()?;
        let rdr = csv::Reader::from_path(file_path)?;
        Ok(Self {
            iter: rdr.into_deserialize(),
        })
    }
}

impl Iterator for TransactionFeed {
    type Item = Result<Transaction, csv::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
