use starknet_simulator::mempool::Mempool;
use starknet_simulator::sequencer::Sequencer;
use starknet_simulator::transaction::{Transaction, TransactionType};
use starknet_simulator::prover::Prover;

use std::sync::{Arc, Mutex};
use std::io;
use std::io::Write; // For flushing stdout



fn main() {
    let mempool = Arc::new(Mempool::new());
    let sequencer = Arc::new(Mutex::new(Sequencer::new(mempool.clone())));
    let prover = Prover::new(Arc::clone(&sequencer)); 

    loop {
        println!("\n🌟 Welcome to the Starknet Simulator!! 🌟");
        println!("🚀 Choose an option between the following:");
        println!("1. View Wallet Balances");
        println!("2. Submit Transaction");
        println!("3. Process Transactions");
        println!("4. Prove Pending Blocks (Finalize on Ethereum)");
        println!("5. Exit");

        let choice = get_input("Select an option: ");

        match choice.trim() {
            "1" => display_balances(&mempool),
            "2" => submit_transaction(&mempool),
            "3" => sequencer.lock().unwrap().process_transactions(),
            "4" => prover.verify_proof(),
            "5" => {
                println!("👋 Exiting StarkNet Simulator. Goodbye!");
                break;
            }
            _ => println!("❌ Invalid choice, please try again."),
        }
    }
}

/// Handles transaction submission
fn submit_transaction(mempool: &Arc<Mempool>) {
    println!("💸 Submit a Transaction");

    let sender = get_input("Enter sender name (Alice, Bob, Mark, Cyndie, Mike): ");
     // Retrieve sender balance
     let sender_balance = {
        let balances = mempool.balances.lock().unwrap();
        *balances.get(&sender).unwrap_or(&0)
    };
    println!("💰 {}'s current balance: {} tokens", sender, sender_balance);

    let tx_type = get_input("Enter transaction type (invoke, declare, deploy): ").to_lowercase();

    // Retrieve the correct nonce automatically from mempool
    let nonce = {
        let nonces = mempool.nonces.lock().unwrap();
        *nonces.get(&sender).unwrap_or(&0)
    };

    let transaction = match tx_type.as_str() {
        "invoke" => {
            let receiver = get_input("Enter receiver name: ");
            let amount: u64 = get_input("Enter amount: ").parse().unwrap_or(0);
            let fee = Transaction::calculate_fee(&TransactionType::Invoke);
            println!("💸 Transaction Fee: {} tokens", fee);
            println!("🔢 Assigned Nonce: {}", nonce);
            Transaction::new(sender, TransactionType::Invoke, Some(receiver), None, Some(amount), nonce)
        }
        "declare" => {
            let contract_address = get_input("Enter contract address: ");
            let fee = Transaction::calculate_fee(&TransactionType::Declare);
            println!("💸 Transaction Fee: {} tokens", fee);
            println!("🔢 Assigned Nonce: {}", nonce);
            Transaction::new(sender, TransactionType::Declare, None, Some(contract_address), None, nonce)
        }
        "deploy" => {
            let fee = Transaction::calculate_fee(&TransactionType::DeployAccount);
            println!("💸 Transaction Fee: {} tokens", fee);
            println!("🔢 Assigned Nonce: {}", nonce);
            Transaction::new(sender, TransactionType::DeployAccount, None, None, None, nonce)
        }
        _ => {
            println!("❌ Invalid transaction type.");
            return;
        }
    };

    mempool.submit_transaction(transaction);
    println!("✅ Transaction submitted successfully!");
}

/// Displays wallet balances
fn display_balances(mempool: &Arc<Mempool>) {
    let balances = mempool.balances.lock().unwrap();
    println!("\n💰 Wallet Balances:");
    for (account, balance) in balances.iter() {
        println!("   - {}: {} tokens", account, balance);
    }
}

/// Gets user input
fn get_input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}