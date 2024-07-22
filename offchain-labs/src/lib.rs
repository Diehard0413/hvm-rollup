pub mod config;
pub mod error;
pub mod prover;
pub mod sequencer;
pub mod verifier;
pub mod zk_rollup;

pub use config::Config;
use error::HVMError;
use sequencer::Transaction;
use prover::ZKProver;
use verifier::ZKVerifier;

use ark_bn254::Bn254;
use ark_groth16::Groth16;
use ark_snark::SNARK;

pub struct OffchainLabs {
    prover: ZKProver,
    sequencer: sequencer::Sequencer,
    verifier: ZKVerifier,
}

impl OffchainLabs {
    pub fn new(config: Config) -> Result<Self, HVMError> {
        let rng = &mut ark_std::rand::thread_rng();
        let circuit = prover::BatchCircuit::new(&sequencer::Batch::new(vec![]));
        
        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, rng)
            .map_err(|e| HVMError::Setup(format!("Failed to generate ZK-SNARK keys: {}", e)))?;
        
        let prover = prover::create_zk_prover(pk);
        let sequencer = sequencer::Sequencer::new(zk_rollup::State::default(), config.sequencer_config.clone());
        let verifier = verifier::create_zk_verifier(vk);

        Ok(Self {
            prover,
            sequencer,
            verifier,
        })
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<bool, HVMError> {
        println!("Processing transaction: {:?}", transaction);
        self.sequencer.process_transaction(transaction)?;

        if let Some(batch) = self.sequencer.create_batch(true)? {
            println!("Batch created: {:?}", batch);
            let proof = self.prover.generate_proof(&batch)?;
            println!("Proof generated: {:?}", proof);
            let is_valid = self.verifier.verify_proof(&proof)?;
            println!("Proof verification result: {}", is_valid);
            
            if is_valid {
                self.sequencer.apply_proof(proof, &batch)?;
                println!("Proof applied");
            }
    
            Ok(is_valid)
        } else {
            println!("No batch created");
            Ok(true)
        }
    }

    pub fn get_current_state(&self) -> Result<zk_rollup::State, HVMError> {
        Ok(self.sequencer.get_current_state())
    }

    pub fn pending_transactions_count(&self) -> usize {
        self.sequencer.pending_transactions_count()
    }

    pub fn processed_transactions_count(&self) -> usize {
        self.sequencer.processed_transactions_count()
    }

    pub fn get_pending_transactions(&self) -> &std::collections::VecDeque<Transaction> {
        self.sequencer.get_pending_transactions()
    }

    pub fn get_processed_transactions(&self) -> &Vec<Transaction> {
        self.sequencer.get_processed_transactions()
    }
}