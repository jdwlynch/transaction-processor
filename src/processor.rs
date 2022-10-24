use std::borrow::BorrowMut;
use serde::Deserialize;
use decimal::d128;
use std::collections::HashMap;
use std::error::Error;

use crate::TransactionFeed;
use crate::models::accounts::{Client, Accounts};

#[derive(Debug, Deserialize)]
enum TxTypes {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug, Deserialize)]
pub struct Transaction{
    #[serde(rename = "type")]
    tx_type: TxTypes,
    client: u16,
    #[serde(rename = "tx")]
    tx_id: u32,
    amount: Option<d128>,
    #[serde(skip)]
    disputed: bool
}

//#[derive(Default)]
pub struct Processor {
    ledger: HashMap<u32, Transaction>,
    clients: Accounts
}

impl Processor {
    pub fn new() -> Result<Processor, Box<dyn Error>> {
        Ok(Self{
            ledger: HashMap::new(),
            clients: Accounts::new()
        })
    }

    pub fn process_transactions(mut consumer: TransactionFeed, mut clients: Accounts){
        let mut engine = Processor::new().unwrap();
        for transaction in  consumer.borrow_mut() {
            let mut tx = transaction.unwrap();
            engine.validate_transaction(tx.tx_id);
            let client = clients.get_client(tx.client).unwrap();
            engine.process_transaction(client, &mut tx);
            engine.update_ledger(tx);
        }
    }

    fn update_ledger(&self, transaction: Transaction){

    }

    fn validate_transaction(& self, tx_id: u32)->Result<(), String>{
        if self.ledger.contains_key(&tx_id){
            return Err(String::from("Error: Duplicate transaction"))
        }
        println!("Transaction ok: {:?}", tx_id);
        Ok(())
    }

    fn process_transaction(& mut self, client: &mut Client, transaction:&mut Transaction) -> Result<(), String>{
        match transaction.tx_type{
            TxTypes::Deposit => process_deposit(client, transaction.amount.unwrap()),
            TxTypes::Withdrawal => process_withdrawal(client, transaction.amount.unwrap()),
            _ => self.disputed_transaction(client, transaction),
        };
        Ok(())
    }
    fn disputed_transaction(&mut self, client: &mut Client, transaction:& mut Transaction) -> Result<(), String>{
        match self.ledger.get_mut(&transaction.tx_id){
            Some(tx) => {
                if client.id == tx.client{
                    match transaction.tx_type{
                        TxTypes::Dispute => {
                            if !tx.disputed{
                                process_dispute(client, transaction.amount.unwrap())
                            }else{
                                Ok(())
                            }
                        },
                        TxTypes::Resolve => if tx.disputed{
                            process_resolve(client, transaction.amount.unwrap());
                            tx.disputed = false;
                            Ok(())
                        }else{
                            Ok(())
                        },
                        TxTypes::Chargeback=> if tx.disputed{
                            process_chargeback(client, transaction.amount.unwrap());
                            tx.disputed = false;
                            Ok(())
                        }else{
                            Ok(())
                        },
                        _ => Ok(()) //error
                    }
                }else {
                    Ok(())
                    //error
                }
            }
            None => Ok(())//error
            }
    }
}

fn process_deposit(client: &mut Client, amount: d128) -> Result<(), String> {
    println!("Found a deposit");
    client.deposit(amount);
    Ok(())
}

fn process_withdrawal(client: &mut Client, amount: d128) -> Result<(), String>{
    println!("Found a withdrawal");
    client.withdraw(amount);
    Ok(())
}

fn process_dispute(client: &mut Client, amount: d128) -> Result<(), String>{
    println!("Found a dispute");
    client.dispute(amount);
    Ok(())
}

fn process_resolve(client: &mut Client, amount: d128) -> Result<(), String>{
    println!("Found a resolve");
    client.resolve(amount);
    Ok(())
}

fn process_chargeback(client: &mut Client, amount: d128) -> Result<(), String>{
    println!("Found a chargeback");
    client.chargeback(amount);
    Ok(())
}