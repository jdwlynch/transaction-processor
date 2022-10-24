use std::collections::HashMap;
use std::fmt::Display;

use crate::models::accounts::{Accounts, Client};
use crate::transaction::{Transaction, TxTypes};
use crate::{error, TransactionFeed};
use log::{debug, trace};

#[derive(Default, Debug)]
pub struct Processor {
    ledger: HashMap<u32, Transaction>,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            ledger: HashMap::new(),
        }
    }

    pub fn handle_transactions(consumer: TransactionFeed, clients: &mut Accounts) {
        let mut processor = Self {
            ..Default::default()
        };
        for transaction in consumer {
            match transaction {
                Err(err) => {
                    error!("[!] Error parsing transaction: {}", err);
                    continue;
                }
                Ok(mut tx) => {
                    trace!("[!] transaction parsed = {:?}", tx);
                    if let Err(err) = processor.validate_transactions(&mut tx) {
                        error!("[!] Error validating transactions: {}", err);
                        continue;
                    }
                    match clients.get_client(tx.client) {
                        Ok(client) => {
                            processor.process_transaction(client, tx);
                        }
                        Err(err) => {
                            error!("[!] Error getting client: {}", err);
                        }
                    }
                }
            }
        }
    }

    fn validate_transactions(&self, tx: &mut Transaction) -> Result<(), error::Error> {
        if self.ledger.contains_key(&tx.tx_id) {
            return Err(error::Error::TransactionError(String::from(
                "Duplicate transaction",
            )));
        } else {
            Transaction::validate_transaction(tx.amount, &tx.tx_type)?;
            Ok(())
        }
    }

    fn check_client_ids_match<T: Display + PartialEq>(id1: T, id2: T) -> Result<(), error::Error> {
        if id1 != id2 {
            Err(error::Error::TransactionError(format!(
                "Client {} is trying to dispute a transaction belonging to client {}",
                id1, id2
            )))
        } else {
            debug!("Client ids {} and {} match.", id1, id2);
            Ok(())
        }
    }

    fn validate_dispute(
        &self,
        client: &Client,
        tx_id: u32,
        resolving: bool,
    ) -> Result<(), error::Error> {
        match self.ledger.get(&tx_id) {
            Some(tx) => {
                Processor::check_client_ids_match(tx.client, client.client)?;
                Transaction::check_transaction_dispute_compatible(resolving, tx.disputed)?;
                Transaction::check_amount_is_valid(&tx.tx_type, tx.amount)?;
                debug!("Dispute related transaction is valid");
                Ok(())
            }
            None => Err(error::Error::TransactionError(format!(
                "Trying to dispute transaction {},\
                but that transaction does not exist",
                tx_id
            ))),
        }
    }

    fn process_disputed_transaction(
        &self,
        client: &mut Client,
        transaction: &mut Transaction,
        resolving: bool,
    ) -> Result<(), error::Error> {
        if let Err(err) = self.validate_dispute(client, transaction.tx_id, resolving) {
            return Err(error::Error::TransactionError(format!(
                "Error validating dispute: {}",
                err
            )));
        } else {
            //Impossible as amount is checked in validators, so in the absence of a dto, use .expect.
            let amount = transaction
                .amount
                .expect("System error, amount check failed.");
            match transaction.tx_type {
                TxTypes::Dispute => {
                    client.dispute(amount);
                    transaction.disputed = true;
                }
                TxTypes::Resolve => {
                    client.resolve(amount);
                    transaction.disputed = false;
                }
                TxTypes::Chargeback => {
                    client.chargeback(amount);
                    transaction.disputed = false;
                }
                //This function is called as a fall-through of transaction parser that handles
                //all other cases. This should be impossible, and if reached is a critical bug.
                _ => panic!(
                    "System error, unreachable line. Non-dispute related transactions\
                must be handled before here."
                ),
            };
        }
        debug!("Dispute related transaction successfully handled");
        trace!("Dispute related transaction is: {:?}", transaction);
        Ok(())
    }

    fn process_transaction(&mut self, client: &mut Client, mut transaction: Transaction) {
        //Impossible as amount is checked in validators, so in the absence of a dto, use .expect.
        let amount = transaction
            .amount
            .expect("System error, amount check failed.");
        match transaction.tx_type {
            TxTypes::Deposit => client.deposit(amount),
            TxTypes::Withdrawal => {
                if let Err(err) = client.withdraw(amount) {
                    error!("[!] Error withdrawing funds: {}", err);
                    return;
                }
            }
            TxTypes::Dispute | TxTypes::Resolve | TxTypes::Chargeback => {
                debug!(
                    "Found dispute related transaction: {:?}",
                    transaction.tx_type
                );
                let resolving = transaction.tx_type != TxTypes::Dispute;
                trace!("Resolving found to be: {}", resolving);
                if let Err(err) =
                    self.process_disputed_transaction(client, &mut transaction, resolving)
                {
                    error!("[!] Error handling a dispute related transaction: {}", err);
                    return;
                }
            }
        };
        debug!("Inserting transaction into ledger: {:?}", transaction);
        self.ledger.insert(transaction.tx_id, transaction);
    }
}
