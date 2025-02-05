use std::sync::{Arc, Mutex};
use crate::sequencer::Sequencer;
use crate::block::{L2Block, BlockStatus};
use std::thread;
use std::time::Duration;
use rand::random;

/// The Verifier is responsible for verifying cryptographic proofs and finalizing blocks on L1.
pub struct Verifier {
    pub sequencer: Arc<Mutex<Sequencer>>, // Reference to the sequencer
}

impl Verifier {
    /// Creates a new Verifier instance
    pub fn new(sequencer: Arc<Mutex<Sequencer>>) -> Self {
        Verifier { sequencer }
    }

    /// Simulates proof verification for all blocks that have `AcceptedOnL1` status
    pub fn verify_proofs(&self) {
        let sequencer = self.sequencer.lock().unwrap();
        let mut blocks = sequencer.blocks.lock().unwrap();

        let mut verified_any = false;

        for block in blocks.iter_mut().filter(|b| b.header.block_status == BlockStatus::AcceptedOnL1) {
            println!("[Verifier] 🔍 Verifying proof on Ethereum for Block #{}...", block.header.block_number);
            thread::sleep(Duration::from_secs(2)); // Simulating verification delay

            // Simulate a verification process with a 95% success rate
            let verification_success = random::<f32>() > 0.05;

            if verification_success {
                block.header.block_status = BlockStatus::AcceptedOnL1;
                println!("[Verifier] ✅ Proof for Block #{} is valid!", block.header.block_number);
                println!("🌍 Block #{} is now **Finalized on Ethereum L1** ✅\n", block.header.block_number);
                verified_any = true;
            } else {
                println!("[Verifier] ❌ Proof for Block #{} **failed verification**! Retrying required.", block.header.block_number);
            }
        }

        if !verified_any {
            println!("[Verifier] ⚠️ No new proofs available for verification.");
        }
    }
}