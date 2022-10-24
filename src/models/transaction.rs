use crate::error;
use decimal::d128;
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
    pub fn validate_transaction(amount: Option<d128>, types: &TxTypes) -> Result<(), error::Error> {
        match types {
            TxTypes::Deposit | TxTypes::Withdrawal => {
                Self::validate_deposit_withdrawal_structure(amount)
            }
            _ => Self::validate_dispute_related_structure(amount),
        }
    }

    fn validate_deposit_withdrawal_structure(amount: Option<d128>) -> Result<(), error::Error> {
        if let Some(tx_amount) = amount {
            if tx_amount < d128!(0) {
                Err(error::Error::TransactionError(String::from(
                    "Amount must be a positive number",
                )))
            } else {
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
                    Ok(())
                } else {
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
