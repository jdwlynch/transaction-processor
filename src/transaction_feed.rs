use crate::error;
use crate::transaction::Transaction;
use csv::{DeserializeRecordsIntoIter, Trim};
use std::ffi::OsString;
use std::fs::File;

///Provides a feed of transactions transparent to the user by implementing the Iterator trait.
/// Holds ownership of csv's iterator for subsequent method calls.
pub struct TransactionFeed {
    iter: DeserializeRecordsIntoIter<File, Transaction>,
}

impl TransactionFeed {
    ///Reads csv's from a file path trimming all whitespace and ignoring missing or extra fields.
    /// Takes ownership of the csv iterator.
    pub fn new(file_path: OsString) -> Result<TransactionFeed, error::Error> {
        let rdr = csv::ReaderBuilder::new().flexible(true).trim(Trim::All).from_path(file_path)?;
        Ok(Self {
            iter: rdr.into_deserialize(),
        })
    }
}

impl Iterator for TransactionFeed {
    type Item = Result<Transaction, error::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|e| e.map_err(error::Error::Csv))
    }
}
