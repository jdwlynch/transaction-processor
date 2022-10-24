use crate::error;
use decimal::d128;
use log::{debug, trace};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub enum TxTypes {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TxTypes,
    pub client: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Option<d128>,
    #[serde(skip)]
    pub disputed: bool,
}

impl Transaction {
    pub fn validate_transaction(
        amount: Option<d128>,
        tx_type: &TxTypes,
    ) -> Result<(), error::Error> {
        match tx_type {
            TxTypes::Deposit | TxTypes::Withdrawal => {
                trace!(
                    "Deposit or withdrawal detected, calling validate: {:?}",
                    tx_type
                );
                Self::validate_deposit_withdrawal_structure(amount)
            }
            _ => {
                trace!(
                    "Dispute related transaction detected, calling validate: {:?}",
                    tx_type
                );
                Self::validate_dispute_related_structure(amount)
            }
        }
    }

    fn validate_deposit_withdrawal_structure(amount: Option<d128>) -> Result<(), error::Error> {
        if let Some(tx_amount) = amount {
            if tx_amount < d128!(0) {
                Err(error::Error::TransactionError(String::from(
                    "Amount must be a positive number",
                )))
            } else {
                trace!(
                    "withdrawal or deposit of {:?} successfully validated.",
                    amount
                );
                Ok(())
            }
        } else {
            Err(error::Error::TransactionError(String::from(
                "Deposits and withdrawals require an amount.",
            )))
        }
    }

    fn validate_dispute_related_structure(amount: Option<d128>) -> Result<(), error::Error> {
        if let Some(_) = amount {
            Err(error::Error::TransactionError(String::from(
                "Disputes, resolutions and \
                            chargebacks shouldn't have amounts",
            )))
        } else {
            debug!("Dispute related transaction ok.");
            Ok(())
        }
    }

    pub fn check_transaction_dispute_compatible(
        tx_resolving: bool,
        ledger_disputed: bool,
    ) -> Result<(), error::Error> {
        if tx_resolving != ledger_disputed {
            Err(error::Error::TransactionError(String::from(
                "Cannot dispute a disputed transaction, \
                        or resolve an undisputed transaction.",
            )))
        } else {
            trace!(
                "Dispute related transaction ok. The targeted transaction disputed = {} \
            and the new transaction has resolving = {}",
                ledger_disputed,
                tx_resolving
            );
            Ok(())
        }
    }

    pub fn check_amount_is_valid(
        tx_type: &TxTypes,
        amount: Option<d128>,
    ) -> Result<(), error::Error> {
        match tx_type {
            TxTypes::Deposit => {
                if let Some(_) = amount {
                    debug!(
                        "A deposit is being disputed or resolved for a value of {:?}",
                        amount
                    );
                    Ok(())
                } else {
                    //This should be impossible. The ledger is malfunctioning, so the system can't be trusted
                    panic!(
                        "System error, ledger shows a deposit \
                                with no amount"
                    );
                }
            }
            _ => Err(error::Error::TransactionError(format!(
                "Trying to dispute or resolve a {:?}. \
                        Only Deposits are valid targets.",
                tx_type
            ))),
        }
    }
}
