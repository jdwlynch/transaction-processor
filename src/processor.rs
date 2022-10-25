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
                    error!("[!] Error parsing transaction: {:?}", err);
                    continue;
                }
                Ok(mut tx) => {
                    trace!("[!] transaction parsed = {:?}", tx);
                    if let Err(err) = Transaction::validate_transaction(&mut tx.amount, &tx.tx_type)
                    /*processor.check_transaction(&mut tx)*/
                    {
                        error!("[!] Error validating transactions: {:?}", err);
                        continue;
                    }
                    match clients.get_client(tx.client) {
                        Ok(client) => {
                            processor.process_transaction(client, tx);
                        }
                        Err(err) => {
                            error!("[!] Error getting client: {:?}", err);
                        }
                    }
                }
            }
        }
    }

    fn process_transaction(&mut self, client: &mut Client, mut transaction: Transaction) {
        match transaction.tx_type {
            TxTypes::Deposit | TxTypes::Withdrawal => {
                if let Err(err) = self.handle_deposits_withdrawals(&mut transaction, client) {
                    error!("[!] Error processing deposit or withdrawal: {}", err);
                } else {
                    debug!(
                        "Successful transaction. Inserting into ledger: {:?}",
                        transaction
                    );
                    self.ledger.insert(transaction.tx_id, transaction);
                }
            }
            TxTypes::Dispute | TxTypes::Resolve | TxTypes::Chargeback => {
                debug!(
                    "Found dispute related transaction: {:?}",
                    transaction.tx_type
                );
                //resolve and chargeback resolve disputed transactions
                trace!("Type of transaction checked is: {:?}", transaction.tx_type);
                let resolving = transaction.tx_type != TxTypes::Dispute;
                trace!("Resolving found to be: {}", resolving);
                if let Err(err) =
                    self.handle_disputed_transaction(client, &mut transaction, resolving)
                {
                    error!(
                        "[!] Error handling a dispute related transaction: {:?}",
                        err
                    );
                }
            }
        };
    }

    fn check_client_ids_match<T: Display + PartialEq>(id1: T, id2: T) -> Result<(), error::Error> {
        if id1 != id2 {
            Err(error::Error::Transaction(format!(
                "Client {} is trying to dispute a transaction belonging to client {}",
                id1, id2
            )))
        } else {
            debug!("Client ids {} and {} match.", id1, id2);
            Ok(())
        }
    }

    fn get_disputed_transaction(
        &mut self,
        client: &Client,
        tx_id: u32,
        resolving: bool,
    ) -> Result<&mut Transaction, error::Error> {
        match self.ledger.get_mut(&tx_id) {
            Some(tx) => {
                Processor::check_client_ids_match(tx.client, client.client)?;
                Transaction::check_transaction_dispute_valid(resolving, tx.disputed)?;
                Transaction::check_amount_is_valid(&tx.tx_type, tx.amount)?;
                debug!("Dispute related transaction is valid");
                Ok(tx)
            }
            None => Err(error::Error::Transaction(format!(
                "Trying to dispute transaction {},\
                but that transaction does not exist",
                tx_id
            ))),
        }
    }

    fn handle_deposits_withdrawals(
        &self,
        transaction: &mut Transaction,
        client: &mut Client,
    ) -> Result<(), error::Error> {
        if self.ledger.contains_key(&transaction.tx_id) {
            Err(error::Error::Transaction(String::from(
                "Duplicate transaction",
            )))
        } else {
            //Impossible as amount is checked in validators, so in the absence of a dto, use .expect.
            let amount = transaction
                .amount
                .expect("System error, amount check failed.")
                .round_dp(4);
            trace!(
                "Amount is : {}, and Tx amount is: {}",
                amount,
                transaction.amount.unwrap()
            );
            match transaction.tx_type {
                TxTypes::Deposit => {
                    client.deposit(amount);
                    Ok(())
                }
                TxTypes::Withdrawal => {
                    if let Err(err) = client.withdraw(amount) {
                        error!("[!] Error withdrawing funds: {:?}", err);
                        Err(err)
                    } else {
                        Ok(())
                    }
                }
                _ => panic!(
                    "System error, unreachable line. Non-dispute related transactions\
                must be handled before here."
                ),
            }
        }
    }

    fn handle_disputed_transaction(
        &mut self,
        client: &mut Client,
        transaction: &mut Transaction,
        resolving: bool,
    ) -> Result<(), error::Error> {
        match self.get_disputed_transaction(client, transaction.tx_id, resolving) {
            Err(err) => {
                return Err(error::Error::Transaction(format!(
                    "Error validating dispute: {}",
                    err
                )))
            }
            Ok(tx) => {
                trace!("Found disputed transaction: {:?}", tx);
                //Impossible as amount is checked in validators, so in the absence of a dto, use .expect.
                let amount = tx.amount.expect("System error, amount check failed.");
                match transaction.tx_type {
                    TxTypes::Dispute => {
                        client.dispute(amount);
                        tx.disputed = true;
                    }
                    TxTypes::Resolve => {
                        client.resolve(amount);
                        tx.disputed = false;
                    }
                    TxTypes::Chargeback => {
                        client.chargeback(amount);
                        tx.disputed = false;
                    }
                    //This function is called as a fall-through of transaction parser that handles
                    //all other cases. This should be impossible, and if reached is a critical bug.
                    _ => panic!(
                        "System error, unreachable line. Non-dispute related transactions\
                must be handled before here."
                    ),
                };
            }
        }
        debug!("Dispute related transaction successfully handled");
        trace!("Dispute related transaction is: {:?}", transaction);
        Ok(())
    }
}
