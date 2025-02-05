use std::sync::atomic::{AtomicUsize, Ordering};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Declare,
    Invoke,
    DeployAccount
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Received, //tx is received by mempool
    Rejected, //tx failed in mempool validation and is not included in a block
    Validated, //tx passes mempool validation
    Executed, //tx is executed by sequencer. Note: can still be reverted
    Reverted, //tx is reverted by sequencer, will still be added to the block
    Succeeded, //tx executed by sequencer successfully, modified in state
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
    // Calculate hash of the transaction
    pub fn get_hash(&self) -> String {
        let input = format!(
            "{}-{}-{:?}-{}-{:?}",
            self.id, self.sender, self.tx_type, self.nonce, self.amount
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    // Calculate fee based on tx type
    pub fn calculate_fee(tx_type: &TransactionType) -> u64 {
        match tx_type {
            TransactionType::Invoke => 5,  // Fixed fee for invoking transactions
            TransactionType::Declare => 20, // Declaring contracts costs more
            TransactionType::DeployAccount => 10, // Deploying accounts costs more
        }
    }

    pub fn new(sender: String, tx_type: TransactionType, receiver: Option<String>, contract_address: Option<String>, amount: Option<u64>, nonce: u64) -> Self {
        let id = TX_COUNTER.fetch_add(1, Ordering::Relaxed);
        let fee = Self::calculate_fee(&tx_type); // Automatically calculate fee

        Transaction {
            id,
            sender,
            receiver,
            contract_address,
            amount,
            fee,
            tx_type,
            status: TransactionStatus::Received,
            nonce,
        }
    }
    //Update tx status
    pub fn update_status(&mut self, new_status: TransactionStatus) {
        self.status = new_status;
    }
}