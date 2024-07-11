use offchain_labs::{Config, OffchainLabs};

#[test]
fn test_offchain_labs_initialization() {
    let config = Config {
        zk_params_path: "test_params.json".to_string(),
        state_db_path: "test_state.db".to_string(),
    };

    let hvm = OffchainLabs::new(config);
    assert!(hvm.is_ok());
}

#[test]
fn test_transaction_processing() {
    let config = Config {
        zk_params_path: "test_params.json".to_string(),
        state_db_path: "test_state.db".to_string(),
    };

    let mut hvm = OffchainLabs::new(config).unwrap();
    let transaction = vec![1, 2, 3, 4];

    let result = hvm.process_transaction(&transaction);
    assert!(result.is_ok());
}