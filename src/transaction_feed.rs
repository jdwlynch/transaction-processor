use crate::error;
use crate::transaction::Transaction;
use csv::DeserializeRecordsIntoIter;
use std::ffi::OsString;
use std::fs::File;

pub struct TransactionFeed {
    iter: DeserializeRecordsIntoIter<File, Transaction>,
}

impl TransactionFeed {
    pub fn new(file_path: OsString) -> Result<TransactionFeed, error::Error> {
        let rdr = csv::Reader::from_path(file_path)?;
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
