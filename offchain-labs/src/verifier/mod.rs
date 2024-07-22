use crate::error::HVMError;
use crate::zk_rollup::Proof;

use ark_bn254::Bn254;
use ark_groth16::{Groth16, PreparedVerifyingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_serialize::CanonicalDeserialize;

pub struct ZKVerifier {
    verifying_key: PreparedVerifyingKey<Bn254>,
}

impl ZKVerifier {
    pub fn new(verifying_key: VerifyingKey<Bn254>) -> Self {
        let prepared_verifying_key = Groth16::<Bn254>::process_vk(&verifying_key).unwrap();
        Self { verifying_key: prepared_verifying_key }
    }

    pub fn verify_proof(&self, proof: &Proof) -> Result<bool, HVMError> {
        let groth16_proof = ark_groth16::Proof::<Bn254>::deserialize_uncompressed(&proof.data[..])
            .map_err(|e| HVMError::Verifier(format!("Failed to deserialize proof: {}", e)))?;

        let public_inputs = vec![];
        
        Groth16::<Bn254>::verify_with_processed_vk(&self.verifying_key, &public_inputs, &groth16_proof)
            .map_err(|e| HVMError::Verifier(format!("Proof verification failed: {}", e)))
    }

    pub fn verify_dummy_proof(&self, proof: &Proof) -> Result<bool, HVMError> {
        self.verify_proof(proof)
    }
}

pub fn create_zk_verifier(verifying_key: VerifyingKey<Bn254>) -> ZKVerifier {
    ZKVerifier::new(verifying_key)
}