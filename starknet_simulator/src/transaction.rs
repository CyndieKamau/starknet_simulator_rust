use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Declare,
    Invoke,
    DeployAccount
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Received,
    Rejected,
    Validated,
    Executed,
    ProofGenerated,
    AcceptedOnL1,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: usize,
    pub sender: String,
    pub receiver: Option<String>,  //to be used in invoke txs
    pub contract_address: Option<String>, //to be used in declare txs
    pub amount: Option<u64>, //invoke txs -> token transfers
    pub fee: u64, //gas fees
    pub nonce: u64,
    pub tx_type: TransactionType,
    pub status: TransactionStatus,
}

static TX_COUNTER: AtomicUsize = AtomicUsize::new(1);

impl Transaction {

    //invoke tx
    pub fn new_invoke(sender: String, receiver: String, amount: u64, fee: u64, nonce: u64) -> Self {
        let id = TX_COUNTER.fetch_add(1, Ordering::Relaxed);
        Transaction {
            id,
            sender,
            receiver: Some(receiver),
            contract_address: None,
            amount: Some(amount),
            fee,
            tx_type: TransactionType::Invoke,
            status: TransactionStatus::Received,
            nonce,
        }
    }

    //declare tx
    pub fn new_declare(sender: String, contract_address: String, fee: u64, nonce: u64) -> Self {
        let id = TX_COUNTER.fetch_add(1, Ordering::Relaxed);
        Transaction {
            id,
            sender,
            receiver: None,
            contract_address: Some(contract_address),
            amount: None,
            fee,
            tx_type: TransactionType::Declare,
            status: TransactionStatus::Received,
            nonce,
        }
    }

    //deploy account tx
    pub fn new_deploy_account(sender: String, fee: u64, nonce: u64) -> Self {
        let id = TX_COUNTER.fetch_add(1, Ordering::Relaxed);
        Transaction {
            id,
            sender,
            receiver: None,
            contract_address: None,
            amount: None,
            fee,
            tx_type: TransactionType::DeployAccount,
            status: TransactionStatus::Received,
            nonce,
        }
    }
    //Update tx status
    pub fn update_status(&mut self, new_status: TransactionStatus) {
        self.status = new_status;
    }
}