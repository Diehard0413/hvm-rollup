use crate::error::HVMError;
use crate::zk_rollup::Proof;
use crate::sequencer::{Batch, Transaction};

use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, ProvingKey};
use ark_snark::SNARK;
use ark_serialize::CanonicalSerialize;
use ark_std::One;
use ark_relations::lc;

pub struct ZKProver {
    proving_key: ProvingKey<Bn254>,
}

impl ZKProver {
    pub fn new(proving_key: ProvingKey<Bn254>) -> Self {
        Self { proving_key }
    }

    pub fn generate_proof(&self, batch: &Batch) -> Result<Proof, HVMError> {
        let circuit = BatchCircuit::new(batch);
        
        let rng = &mut ark_std::rand::thread_rng();
        let proof = Groth16::<Bn254>::prove(&self.proving_key, circuit, rng)
            .map_err(|e| HVMError::Prover(format!("Failed to generate proof: {}", e)))?;
        
        let mut proof_bytes = Vec::new();
        proof.serialize_uncompressed(&mut proof_bytes)
            .map_err(|e| HVMError::Prover(format!("Failed to serialize proof: {}", e)))?;
        
        Ok(Proof::new(proof_bytes))
    }

    pub fn create_dummy_proof(&self) -> Result<Proof, HVMError> {
        let dummy_batch = Batch::new(vec![
            Transaction::new("Alice".to_string(), "Bob".to_string(), 100, 1),
            Transaction::new("Bob".to_string(), "Charlie".to_string(), 50, 2),
        ]);
        self.generate_proof(&dummy_batch)
    }
}

pub fn create_zk_prover(proving_key: ProvingKey<Bn254>) -> ZKProver {
    ZKProver::new(proving_key)
}

pub struct BatchCircuit {
    transactions: Vec<(Fr, Fr)>,
}

impl BatchCircuit {
    pub fn new(batch: &Batch) -> Self {
        let transactions = batch
            .transactions()
            .iter()
            .map(|tx| {
                (
                    Fr::from(tx.amount as u64),
                    Fr::from(tx.nonce as u64),
                )
            })
            .collect();
        
        Self { transactions }
    }
}

impl ConstraintSynthesizer<Fr> for BatchCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        for (amount, nonce) in self.transactions {
            let amount_var = cs.new_witness_variable(|| Ok(amount))?;
            let nonce_var = cs.new_witness_variable(|| Ok(nonce))?;

            cs.enforce_constraint(
                lc!() + amount_var + nonce_var,
                lc!() + (Fr::one(), ark_relations::r1cs::Variable::One),
                lc!() + amount_var + nonce_var
            )?;
        }
        
        Ok(())
    }
}