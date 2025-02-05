use crate::transaction::Transaction;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

// This is the status of a block
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStatus {
    Pending,         
    AcceptedOnL2,    
    AcceptedOnL1,   
    Rejected,       
}

// this represents the format of a block header
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub block_number: u64,
    pub parent_block_hash: String,
    pub sequencer_address: String,
    pub block_timestamp: u64,
    pub transaction_count: usize,
    pub transaction_commitment: String,  // Hash of all txs in the block
    pub state_root: String,  // Placeholder for future state root commitment
    pub block_status: BlockStatus,
}

/// Represents a full block containing transactions
#[derive(Debug, Clone)]
pub struct L2Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl L2Block {
    /// Creates a new L2 block
    pub fn new(block_number: u64, parent_block_hash: String, sequencer_address: String, transactions: Vec<Transaction>) -> Self {
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let transaction_count = transactions.len();
        let transaction_commitment = Self::calculate_commitment(&transactions);
        let state_root = "placeholder_state_root".to_string(); // Placeholder for now

        let header = BlockHeader {
            block_number,
            parent_block_hash,
            sequencer_address,
            block_timestamp,
            transaction_count,
            transaction_commitment,
            state_root,
            block_status: BlockStatus::AcceptedOnL2,
        };

        L2Block { header, transactions }
    }

    /// Generates a commitment hash of all transactions in the block
    fn calculate_commitment(transactions: &[Transaction]) -> String {
        let mut hasher = Sha256::new();
        for tx in transactions {
            let tx_data = format!("{}-{}-{:?}", tx.id, tx.sender, tx.status);
            hasher.update(tx_data);
        }
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Generates a unique block hash
    pub fn get_block_hash(&self) -> String {
        let input = format!(
            "{}-{}-{}-{}",
            self.header.block_number,
            self.header.transaction_commitment,
            self.header.sequencer_address,
            self.header.parent_block_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}