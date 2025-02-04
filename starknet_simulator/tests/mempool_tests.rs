#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use starknet_simulator::mempool::Mempool;
    use starknet_simulator::transaction::{Transaction, TransactionType, TransactionStatus};

    #[test]
    fn test_valid_transaction_passes() {
        let mempool = Arc::new(Mempool::new());
        let tx = Transaction::new("Alice".to_string(), TransactionType::Invoke, Some("Bob".to_string()), None, Some(10), 0);
        
        mempool.submit_transaction(tx.clone());
        let validated_tx = mempool.validate_transaction();

        assert!(validated_tx.is_some());
        assert_eq!(validated_tx.as_ref().unwrap().status, TransactionStatus::Validated);
    }

    #[test]
    fn test_transaction_rejected_due_to_insufficient_funds() {
        let mempool = Arc::new(Mempool::new());
        let tx = Transaction::new("Mark".to_string(), TransactionType::Invoke, Some("Bob".to_string()), None, Some(50), 0);  
        // Mark has 0 balance

        mempool.submit_transaction(tx.clone());
        let validated_tx = mempool.validate_transaction();

        assert!(validated_tx.is_none());
    }

    #[test]
    fn test_transaction_rejected_due_to_insufficient_funds_for_fee() {
        let mempool = Arc::new(Mempool::new());
        let tx = Transaction::new("Mike".to_string(), TransactionType::Invoke, Some("Bob".to_string()), None, Some(90), 0);  
        // Mike has 90, but he needs 90 + 5 (fee) = 95

        mempool.submit_transaction(tx.clone());
        let validated_tx = mempool.validate_transaction();

        assert!(validated_tx.is_none());
    }

    #[test]
    fn test_transaction_rejected_due_to_incorrect_nonce() {
        let mempool = Arc::new(Mempool::new());

        // First transaction should succeed (nonce = 0)
        let tx1 = Transaction::new("Alice".to_string(), TransactionType::Invoke, Some("Bob".to_string()), None, Some(10), 0);
        mempool.submit_transaction(tx1.clone());
        let validated_tx1 = mempool.validate_transaction();
        assert!(validated_tx1.is_some());

        // Second transaction with incorrect nonce (should be 1)
        let tx2 = Transaction::new("Alice".to_string(), TransactionType::Invoke, Some("Bob".to_string()), None, Some(10), 0);
        mempool.submit_transaction(tx2.clone());
        let validated_tx2 = mempool.validate_transaction();
        assert!(validated_tx2.is_none());
    }

    #[test]
    fn test_fee_is_deducted_only_if_validated() {
        let mempool = Arc::new(Mempool::new());

        // Alice starts with 200 tokens
        let tx = Transaction::new("Alice".to_string(), TransactionType::Invoke, Some("Bob".to_string()), None, Some(10), 0);
        
        // Capture balance before transaction validation
        let balances_before = mempool.balances.lock().unwrap().clone();
        let initial_balance = balances_before.get("Alice").unwrap();

        mempool.submit_transaction(tx.clone());
        let validated_tx = mempool.validate_transaction();
        assert!(validated_tx.is_some());

        // Capture balance after transaction validation
        let balances_after = mempool.balances.lock().unwrap().clone();
        let updated_balance = balances_after.get("Alice").unwrap();

        // Check that the fee was deducted
        assert_eq!(*updated_balance, initial_balance - tx.fee);
    }

    #[test]
    fn test_nonce_is_incremented_correctly() {
        let mempool = Arc::new(Mempool::new());

        let tx1 = Transaction::new("Alice".to_string(), TransactionType::Invoke, Some("Bob".to_string()), None, Some(10), 0);
        mempool.submit_transaction(tx1.clone());
        let validated_tx1 = mempool.validate_transaction();
        assert!(validated_tx1.is_some());

        // Fetch the updated nonce
        let nonces_after = mempool.nonces.lock().unwrap().clone();
        let updated_nonce = nonces_after.get("Alice").unwrap();

        // Check that nonce increased to 1
        assert_eq!(*updated_nonce, 1);
    }
}