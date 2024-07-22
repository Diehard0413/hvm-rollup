use crate::error::HVMError;
use crate::zk_rollup::Proof;
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, PreparedVerifyingKey, VerifyingKey};
use ark_snark::SNARK;
use ark_serialize::CanonicalDeserialize;

pub struct ZKVerifier {
    verifying_key: PreparedVerifyingKey<Bn254>,
}

impl ZKVerifier {
    pub fn new(verifying_key: VerifyingKey<Bn254>) -> Self {
        println!("Creating new ZKVerifier with verifying key: {:?}", verifying_key);
        let prepared_verifying_key = Groth16::<Bn254>::process_vk(&verifying_key).unwrap();
        println!("Prepared verifying key: {:?}", prepared_verifying_key);
        Self { verifying_key: prepared_verifying_key }
    }

    pub fn verify_proof(&self, proof: &Proof) -> Result<bool, HVMError> {
        println!("Verifying proof: {:?}", proof);
        let groth16_proof = ark_groth16::Proof::<Bn254>::deserialize_uncompressed(&proof.data[..])
            .map_err(|e| HVMError::Verifier(format!("Failed to deserialize proof: {}", e)))?;
        
        println!("Deserialized Groth16 proof: {:?}", groth16_proof);
        let public_inputs = vec![Fr::from(1u64)];
        println!("Public inputs: {:?}", public_inputs);
        
        println!("Verifying with processed key: {:?}", self.verifying_key);
        let result = Groth16::<Bn254>::verify_with_processed_vk(&self.verifying_key, &public_inputs, &groth16_proof)
            .map_err(|e| HVMError::Verifier(format!("Proof verification failed: {}", e)));
        println!("Verification result: {:?}", result);

        result.map_err(|e| HVMError::Verifier(format!("Proof verification failed: {}", e)))
    }
}

pub fn create_zk_verifier(verifying_key: VerifyingKey<Bn254>) -> ZKVerifier {
    ZKVerifier::new(verifying_key)
}