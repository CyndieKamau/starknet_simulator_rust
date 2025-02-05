//mempool transaction validation
//the mempool is responsible for validating transactions before they are sent to the sequencer

use std::collections::{VecDeque, HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::transaction::{Transaction, TransactionStatus};

pub struct Mempool {
    pub transactions: Arc<Mutex<VecDeque<Transaction>>>, //track all txs in mempool
    pub rejected_transactions: Arc<Mutex<HashSet<String>>>, // Track rejected tx hashes
    pub balances: Arc<Mutex<HashMap<String, u64>>>, //track balances of all accounts
    pub nonces: Arc<Mutex<HashMap<String, u64>>>, //track nonces of all accounts
}

impl Mempool {
    pub fn new() -> Self {
        let mut initial_balances = HashMap::new();
        initial_balances.insert("Alice".to_string(), 200);
        initial_balances.insert("Bob".to_string(), 500);
        initial_balances.insert("Mark".to_string(), 0);
        initial_balances.insert("Cyndie".to_string(), 700);
        initial_balances.insert("Mike".to_string(), 90);

        let mut initial_nonces = HashMap::new();
        initial_nonces.insert("Alice".to_string(), 0);
        initial_nonces.insert("Bob".to_string(), 0);
        initial_nonces.insert("Mark".to_string(), 0);
        initial_nonces.insert("Cyndie".to_string(), 0);
        initial_nonces.insert("Mike".to_string(), 0);

        Mempool {
            transactions: Arc::new(Mutex::new(VecDeque::new())),
            rejected_transactions: Arc::new(Mutex::new(HashSet::new())), // Store rejected tx hashes
            balances: Arc::new(Mutex::new(initial_balances)),
            nonces: Arc::new(Mutex::new(initial_nonces)),
        }
    }

    //submit a transaction to the mempool, to be marked as RECEIVED
    pub fn submit_transaction(&self, mut tx: Transaction) {
        let rejected_txs = self.rejected_transactions.lock().unwrap();
        let tx_hash = tx.get_hash();
         // Check if transaction is already rejected
         if rejected_txs.contains(&tx_hash) {
            println!(
                "[Mempool] ❌ Transaction {} is already rejected. Cannot resend!",
                tx.id
            );
            return;
        }
        tx.update_status(TransactionStatus::Received);
        let mut txs = self.transactions.lock().unwrap();
        txs.push_back(tx);
    }

    //initial check to validate tx requirements
    pub fn validate_transaction(&self) -> Option<Transaction> {
        let mut txs = self.transactions.lock().unwrap();
        let mut balances = self.balances.lock().unwrap();
        let nonces = self.nonces.lock().unwrap();
        let mut rejected_txs = self.rejected_transactions.lock().unwrap();
    
        if let Some(mut tx) = txs.pop_front() {
            println!("[Mempool] is now validating transaction ID: {}", tx.id);
    
            // ✅ 1. Check Nonce (Prevents Replay Attacks)
            let sender_nonce = nonces.get(&tx.sender).unwrap_or(&0);
            if tx.nonce != *sender_nonce {
            println!(
                "[Mempool] ❌ Transaction {} is rejected! Incorrect nonce. Expected: {}",
                tx.id, *sender_nonce
            );
            tx.update_status(TransactionStatus::Rejected);
            return None;
        }
    
            // ✅ 2. Check If Sender Has Enough Funds for Fee + Amount
            let sender_balance = balances.get(&tx.sender).unwrap_or(&0);
            
            if *sender_balance <= tx.fee {
                println!(
                    "[Mempool] ❌ Transaction {} is rejected! Account balance must be greater than the fee.",
                    tx.id
                );
                tx.update_status(TransactionStatus::Rejected);
                rejected_txs.insert(tx.get_hash()); // Store rejected tx hash
                return None;
            }
    
            if let Some(amount) = tx.amount {
                let total_cost = amount + tx.fee;
                if *sender_balance < total_cost {
                    println!(
                        "[Mempool] ❌ Transaction {} is rejected! Insufficient balance for transfer + fee.",
                        tx.id
                    );
                    tx.update_status(TransactionStatus::Rejected);
                    rejected_txs.insert(tx.get_hash()); // Store rejected tx hash
                    return None;
                }
            }
    
            // ✅ 3. Deduct Fee After Validation (Only If Passed)
            *balances.get_mut(&tx.sender).unwrap() -= tx.fee;
    
            tx.update_status(TransactionStatus::Validated);
            println!("[Mempool] ✅ Transaction {} is validated!", tx.id);
            return Some(tx);
        }
    
        None    
    }
}
