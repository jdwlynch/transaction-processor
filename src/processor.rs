use std::borrow::BorrowMut;
use decimal::d128;
use std::collections::HashMap;
use std::fmt::Display;

use crate::{error, TransactionFeed};
use crate::models::accounts::{Client, Accounts};
use crate::transaction::{Transaction, TxTypes};

#[derive(Default, Debug)]
pub struct Processor {
    ledger: HashMap<u32, Transaction>,
    accounts: Accounts
}

impl Processor {
    pub fn new() -> Result<Processor, error::Error> {
        Ok(Self{
            ledger: HashMap::new(),
            accounts: Accounts::new()
        })
    }

    pub fn process_transactions(mut consumer: TransactionFeed, mut clients: Accounts)
        -> Result<(), error::Error>{
        let mut processor = Self{
            ..Default::default()
        };
        for transaction in  consumer.borrow_mut() {
            let mut tx = transaction?;
            processor.validate_transaction(tx.tx_id, tx.amount, &tx.tx_type);
            let client = clients.get_client(tx.client).unwrap();
            processor.process_transaction(client, &mut tx)?;
            processor.update_ledger(tx);
        }
        Ok(())
    }

    fn update_ledger(&self, transaction: Transaction){

    }

    fn check_duplicate_transactions(&self, tx_id: u32)->Result<(), error::Error>{
        if self.ledger.contains_key(&tx_id){
            return Err(error::Error::TransactionError(
                String::from("Duplicate transaction"))
            )
        }else{
            Ok(())
        }
    }

    fn validate_deposit_withdrawal_structure(amount: Option<d128>) ->Result<(), error::Error>{
        if let Some (tx_amount) = amount{
            if tx_amount < d128!(0){
                Err(error::Error::TransactionError(
                    String::from("Amount must be a positive number"))
                )
            }else{
                Ok(())
            }
        }else{
            Err(error::Error::TransactionError(
                String::from("Deposits and withdrawals require an amount.")))
        }
    }

    fn validate_dispute_related_structure(amount: Option<d128>) -> Result<(), error::Error>{
        if let Some (_) = amount{
            Err(error::Error::TransactionError(
                String::from("Disputes, resolutions and \
                            chargebacks shouldn't have amounts"))
            )

        }else{
            Ok(())
        }
    }

    fn validate_transaction(& self, tx_id: u32, amount: Option<d128>, types:& TxTypes)
        ->Result<(), error::Error>{
        self.check_duplicate_transactions(tx_id)?;
        match types{
            TxTypes::Deposit | TxTypes::Withdrawal => Self::validate_deposit_withdrawal_structure(amount),
            _ => Self::validate_dispute_related_structure(amount)
        }
    }

    fn check_client_ids_match<T: Display + PartialEq>(id1: T, id2: T) ->Result<(), error::Error>{
        if id1 != id2 {
            Err(error::Error::TransactionError(format!(
                "Client {} is trying to dispute a transaction belonging to client {}",
                id1, id2)))
        } else{
            Ok(())
        }
    }

    fn check_transaction_dispute_compatible(tx_resolving: bool, ledger_disputed: bool) ->Result<(), error::Error>{
        if tx_resolving != ledger_disputed {
            Err(error::Error::TransactionError(String::from(
                "Cannot dispute a disputed transaction, \
                        or resolve an undisputed transaction.")))
        }else{
            Ok(())
        }
    }

    fn check_amount_is_valid(tx_type: & TxTypes, amount: Option<d128>) -> Result<(), error::Error>{
        match tx_type{
            TxTypes::Deposit=> {
                if let Some(_) = amount{
                    Ok(())
                }else{
                    panic!("System error, ledger shows a deposit \
                                with no amount");
                }
            },
            _ => Err(error::Error::TransactionError(format!(
                "Trying to dispute or resolve a {:?}. \
                        Only Deposits are valid targets.", tx_type)))
        }
    }

    fn validate_dispute(&mut self, client: &mut Client, tx_id: u32, resolving: bool, )
        ->Result<decimal::d128, error::Error> {
        match self.ledger.get_mut(&tx_id) {
            Some(tx) => {
                Processor::check_client_ids_match(tx.client, client.id)?;
                Processor::check_transaction_dispute_compatible(resolving, tx.disputed)?;
                Processor::check_amount_is_valid(&tx.tx_type, tx.amount)?;
                Ok(tx.amount.unwrap())
            },
            None => Err(error::Error::TransactionError(format!(
                "Trying to dispute transaction {},\
                but that transaction does not exist", tx_id)))
        }
    }

    fn process_transaction(& mut self, client: &mut Client, transaction:&mut Transaction) -> Result<(), error::Error>{
        match transaction.tx_type{
            TxTypes::Deposit => client.deposit(transaction.amount.expect(
                "System error, deposit called with no amount present.")),
            TxTypes::Withdrawal => client.withdraw(transaction.amount.expect(
                "System error, withdrawal called with no amount present."))?,
            TxTypes::Dispute => {
                let amount = self.validate_dispute(client, transaction.tx_id, false)?;
                client.dispute(amount);
                transaction.disputed = true;
            },
            TxTypes::Resolve => {
                let amount = self.validate_dispute(client, transaction.tx_id, true)?;
                client.resolve(amount);
                transaction.disputed = false;
            },
            TxTypes::Chargeback => {
                let amount = self.validate_dispute(client, transaction.tx_id, true)?;
                client.chargeback(amount);
                transaction.disputed = false;
            }
        };
        Ok(())
    }
}