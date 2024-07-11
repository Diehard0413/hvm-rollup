use offchain_labs::{Config, OffchainLabs};

#[tokio::test]
async fn test_end_to_end_workflow() {
    let config = Config {
        zk_params_path: "test_params.json".to_string(),
        state_db_path: "test_state.db".to_string(),
    };

    let mut hvm = OffchainLabs::new(config).unwrap();

    let transactions = vec![
        vec![1, 2, 3, 4],
        vec![5, 6, 7, 8],
        vec![9, 10, 11, 12],
    ];

    for tx in transactions {
        let result = hvm.process_transaction(&tx);
        assert!(result.is_ok());
    }

    // Add more assertions to verify the final state, etc.
}