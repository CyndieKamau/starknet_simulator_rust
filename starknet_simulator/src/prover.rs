use std::sync::{Arc, Mutex};
use crate::block::{L2Block, BlockStatus};
use crate::sequencer::Sequencer;
use sha2::{Sha256, Digest};
use std::thread;
use std::time::Duration;

/// Represents the Prover responsible for generating and verifying STARK proofs
pub struct Prover {
    pub sequencer: Arc<Mutex<Sequencer>>,
}

impl Prover {
    /// Creates a new Prover instance
    pub fn new(sequencer: Arc<Mutex<Sequencer>>) -> Self {
        Prover { sequencer }
    }

    /// Generates a proof for a block (simulated)
    pub fn generate_proof(&self, block_hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(block_hash);
        format!("{:x}", hasher.finalize())
    }

    /// Verifies the proof and updates block status to `AcceptedOnL1`
    pub fn verify_proof(&self) {
        let sequencer = self.sequencer.lock().unwrap();
        let mut blocks = sequencer.blocks.lock().unwrap();

        let mut verified_any = false;

        for block in blocks.iter_mut().filter(|b| b.header.block_status == BlockStatus::AcceptedOnL2) {
            println!("[Prover] 🔍 Verifying proof for Block #{}...", block.header.block_number);
            thread::sleep(Duration::from_secs(2)); // Simulating verification delay

            // Simulate proof generation
            let proof = self.generate_proof(&block.get_block_hash());
            block.header.block_status = BlockStatus::AcceptedOnL1;

            println!("[Prover] ✅ Block #{} verified and proof generated!", block.header.block_number);
            println!("🌍 Block #{} Proof Generated ✅", block.header.block_number);
            println!("🔗 Proof: {}\n", proof);

            verified_any = true;
        }

        if !verified_any {
            println!("[Prover] ⚠️ No new L2 blocks available for proving.");
        }
    }
}