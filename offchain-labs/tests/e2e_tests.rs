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

    for (i, tx) in transactions.iter().enumerate() {
        let result = hvm.process_transaction(tx);
        assert!(result.is_ok(), "Failed to process transaction {}", i);
        let is_valid = result.unwrap();
        assert!(is_valid, "Transaction {} was invalid", i);
    }

    let final_state = hvm.get_current_state().unwrap();
    assert_eq!(final_state.balance(), 3, "Unexpected final balance");
    assert_eq!(final_state.nonce(), 3, "Unexpected final nonce");
}