use offchain_labs::{Config, OffchainLabs};

fn create_test_config() -> Config {
    Config {
        zk_params_path: PathBuf::from("test_params.json"),
        state_db_path: PathBuf::from("test_state.db"),
        prover_config: ProverConfig {
            proving_key_path: PathBuf::from("test_proving_key.bin"),
            max_batch_size: 10,
        },
        verifier_config: VerifierConfig {
            verification_key_path: PathBuf::from("test_verification_key.bin"),
        },
        sequencer_config: SequencerConfig {
            max_pending_transactions: 100,
            batch_interval_seconds: 10,
        },
    }
}

#[test]
fn test_offchain_labs_initialization() {
    let config = create_test_config();
    let hvm = OffchainLabs::new(config);
    assert!(hvm.is_ok());
}

#[test]
fn test_transaction_processing() {
    let config = create_test_config();
    let mut hvm = OffchainLabs::new(config).unwrap();
    let transaction = vec![1, 2, 3, 4];

    let result = hvm.process_transaction(&transaction);
    assert!(result.is_ok());
}