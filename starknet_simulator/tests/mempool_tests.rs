#[cfg(test)]
mod tests {
    use starknet_simulator::mempool::*;
    use starknet_simulator::transaction::*;
    use std::sync::Arc;

    #[test]
    fn test_valid_transaction_submission() {
        let mempool = Arc::new(Mempool::new());
        let tx = Transaction::new_invoke("Alice".to_string(), "Bob".to_string(), 50, 2);
        
        mempool.submit_transaction(tx.clone());

        let validated_tx = mempool.validate_transaction();
        assert!(validated_tx.is_some());
        assert_eq!(validated_tx.unwrap().status, TransactionStatus::Validated);
    }

    #[test]
    fn test_insufficient_funds() {
        let mempool = Arc::new(Mempool::new());
        let tx = Transaction::new_invoke("Alice".to_string(), "Bob".to_string(), 9999, 2);  // Alice doesn't have 9999 tokens

        mempool.submit_transaction(tx.clone());

        let validated_tx = mempool.validate_transaction();
        assert!(validated_tx.is_none()); // Should be rejected
    }


}