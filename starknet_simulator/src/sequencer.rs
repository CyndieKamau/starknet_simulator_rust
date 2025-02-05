use crate::mempool::Mempool;
use crate::transaction::{Transaction, TransactionStatus, TransactionType};
use crate::block::{L2Block, BlockStatus};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Sequencer {
    pub mempool: Arc<Mempool>,
    pub executed_txs: Arc<Mutex<Vec<Transaction>>>,
    pub blocks: Arc<Mutex<Vec<L2Block>>>,
    pub block_number: u64,
    pub parent_block_hash: String,
    pub sequencer_address: String,
}

impl Sequencer {
    pub fn new(mempool: Arc<Mempool>) -> Self {
        Sequencer {
            mempool,
            executed_txs: Arc::new(Mutex::new(vec![])),
            blocks: Arc::new(Mutex::new(vec![])),
            block_number: 1,
            parent_block_hash: "genesis_hash".to_string(),
            sequencer_address: "sequencer_0x123".to_string(),
        }
    }

    /// **Processes transactions one-by-one and creates an L2 block**
    pub fn process_transactions(&mut self) {
        let mut transactions = vec![];

        let tx_count = self.mempool.transactions.lock().unwrap().len();
        if tx_count == 0 {
            println!("[Sequencer] ⚠️ No transactions to process. Returning to menu.");
            return;
        }

    println!("[Sequencer] Processing {} transaction(s)...", tx_count);

        while let Some(mut tx) = self.mempool.validate_transaction() {
            println!("[Sequencer] Processing transaction ID: {}...", tx.id);

            // Ensure strict transaction ordering (FIFO)
            if !self.validate_transaction_again(&tx) {
                println!(
                    "[Sequencer] ❌ Transaction {} REJECTED! Failed validation.",
                    tx.id
                );
                tx.update_status(TransactionStatus::Rejected);
                continue;
            }

            self.execute_transaction(&mut tx);
            transactions.push(tx.clone());

            let mut executed = self.executed_txs.lock().unwrap();
            executed.push(tx.clone());

            // Ensure sequential processing by adding a slight delay
            thread::sleep(Duration::from_secs(1));

            let sender_balance = {
                let balances = self.mempool.balances.lock().unwrap();
                *balances.get(&tx.sender).unwrap_or(&0)
            };
            let receiver_balance = {
                let balances = self.mempool.balances.lock().unwrap();
                tx.receiver
                    .as_ref()
                    .map(|r| balances.get(r).unwrap_or(&0))
                    .unwrap_or(&0)
                    .to_owned()
            };
            println!("💰 {}'s remaining balance: {} tokens", tx.sender, sender_balance);
            if let Some(receiver) = &tx.receiver {
                println!("💰 {}'s new balance: {} tokens", receiver, receiver_balance);
            }
        }

        if !transactions.is_empty() {
            self.create_l2_block(transactions);
        }
    }

    /// **Repeats the validation process before execution**
    fn validate_transaction_again(&self, tx: &Transaction) -> bool {
        let balances = self.mempool.balances.lock().unwrap();
        let nonces = self.mempool.nonces.lock().unwrap();

        // ✅ Ensure transactions are processed in strict nonce order
        let expected_nonce = nonces.get(&tx.sender).unwrap_or(&0);
        if tx.nonce != *expected_nonce {
            println!(
                "[Sequencer] ❌ Transaction {} REJECTED! Nonce mismatch. Expected: {}",
                tx.id, expected_nonce
            );
            return false;
        }

        // ✅ Ensure sender has enough funds
        if let Some(amount) = tx.amount {
            let sender_balance = balances.get(&tx.sender).unwrap_or(&0);
            if *sender_balance < amount {
                println!(
                    "[Sequencer] ❌ Transaction {} REJECTED! Insufficient balance.",
                    tx.id
                );
                return false;
            }
        }

        true
    }

    /// **Executes a transaction and handles `REVERTED` cases**
    fn execute_transaction(&self, tx: &mut Transaction) {
        let mut balances = self.mempool.balances.lock().unwrap();
        let mut nonces = self.mempool.nonces.lock().unwrap();

        let sender_nonce = nonces.entry(tx.sender.clone()).or_insert(0);

        // ✅ Check if transaction nonce matches expected nonce
        if tx.nonce != *sender_nonce {
            println!(
                "[Sequencer] ❌ Transaction {} REJECTED! Nonce mismatch. Expected: {}, Got: {}",
                tx.id, *sender_nonce, tx.nonce
            );
            tx.update_status(TransactionStatus::Rejected);
            return;
        }

        match tx.tx_type {
            TransactionType::Invoke => {
                if let Some(amount) = tx.amount {
                    let sender_balance = balances.get(&tx.sender).unwrap();
                    if *sender_balance < amount {
                        println!(
                            "[Sequencer] ⚠️ Transaction {} REVERTED! Insufficient funds.",
                            tx.id
                        );

                        // 🛑 Deduct the fee even if transaction fails
                        if *sender_balance >= tx.fee {
                            *balances.get_mut(&tx.sender).unwrap() -= tx.fee;
                            println!(
                                "[Sequencer] 💰 Fee of {} deducted from {} for reverted transaction {}.",
                                tx.fee, tx.sender, tx.id
                            );
                        } else {
                            println!(
                                "[Sequencer] 🚨 Warning! {} does not have enough funds for the full fee of {}. Deducting available amount.",
                                tx.sender, tx.fee
                            );
                            *balances.get_mut(&tx.sender).unwrap() = 0;
                        }

                        tx.update_status(TransactionStatus::Reverted);
                    } else {
                        // ✅ Deduct funds sequentially
                        *balances.get_mut(&tx.sender).unwrap() -= amount;

                        // ✅ Update receiver balance
                        let receiver_balance = balances.get(&tx.receiver.clone().unwrap()).cloned().unwrap_or(0);
                        balances.insert(tx.receiver.clone().unwrap(), receiver_balance + amount);

                        println!("[Sequencer] ✅ Transaction {} EXECUTED!", tx.id);

                        tx.update_status(TransactionStatus::Succeeded);
                    }
                }
            }
            TransactionType::Declare | TransactionType::DeployAccount => {
                println!(
                    "[Sequencer] ✅ Transaction {} EXECUTED! (Contract Deployment)",
                    tx.id
                );

                tx.update_status(TransactionStatus::Succeeded);
            }
        }
        *sender_nonce += 1;
    }

    /// **Creates an L2 block containing all processed transactions**
    fn create_l2_block(&mut self, transactions: Vec<Transaction>) {
        let mut block_status = BlockStatus::Pending; // Initially, block is pending
    
        // Determine if any transactions were reverted
        if transactions.iter().all(|tx| tx.status == TransactionStatus::Reverted) {
            println!("[Sequencer] ❌ All transactions in this block failed. Marking block as REJECTED.");
            block_status = BlockStatus::Rejected;
        } else {
            println!("[Sequencer] ✅ Block successfully created on L2.");
            block_status = BlockStatus::AcceptedOnL2;
        }
    
        let new_block = L2Block::new(
            self.block_number,
            self.parent_block_hash.clone(),
            self.sequencer_address.clone(),
            transactions.clone(),
        );
    
        // Apply final block status
        let mut final_block = new_block.clone();
        final_block.header.block_status = block_status;
    
        self.parent_block_hash = new_block.get_block_hash();
        self.block_number += 1;
    
        let mut blocks = self.blocks.lock().unwrap();
        blocks.push(final_block.clone());
    
        println!(
            "[Sequencer] 🏗️ New L2 Block Created: #{} with {} transactions (Status: {:?})",
            final_block.header.block_number, final_block.header.transaction_count, final_block.header.block_status
        );
        println!("\n🌟 L2 Block Created 🌟");
        println!("🔢 Block Number: {}", final_block.header.block_number);
        println!("🔗 Previous Hash: {}", final_block.header.parent_block_hash);
        println!("🔗 Current Hash: {}", final_block.get_block_hash());
        println!("⏳ Timestamp: {}", final_block.header.block_timestamp);
        println!("💰 Transactions in Block: {}", final_block.header.transaction_count);
        println!("🚀 Block Status: {:?}", final_block.header.block_status);
        println!("\n📜 Transactions in Block #{}:", final_block.header.block_number);
    
        println!("[Sequencer] Transactions in Block #{}:", final_block.header.block_number);
        for tx in &transactions {
            println!(
                "    - ID: {} | Nonce: {} | Sender: {} | Receiver: {:?} | Status: {:?}",
                tx.id,tx.nonce, tx.sender,  tx.receiver, tx.status
            );
        }
    }
}