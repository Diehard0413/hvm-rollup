use offchain_labs::{Config, OffchainLabs};
use offchain_labs::bend::{BendProgram, ProgramMetadata};
use std::fs::File;
use std::io::Read;

fn create_test_bend_program() -> BendProgram {
    let mut file = File::open("test_program.wasm").expect("WASM file not found");
    let mut bytecode = Vec::new();
    file.read_to_end(&mut bytecode).expect("Failed to read WASM file");

    let metadata = ProgramMetadata {
        name: "Bend Program".to_string(),
        version: "1.0.0".to_string(),
        description: "Bend program for test".to_string(),
    };
    BendProgram::new(bytecode, metadata, "Author".to_string())
}

#[test]
fn test_bend_program_execution() {
    let config = Config::default();
    let mut hvm = OffchainLabs::new(config).unwrap();
    let program = create_test_bend_program();
    
    hvm.deploy_program(program.clone()).unwrap();

    let result = hvm.execute_program(program.id(), vec![1, 2, 3, 4], "user");
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert_eq!(output, vec![10, 20, 30, 40]);
}

#[test]
fn test_bend_program_optimization() {
    let config = Config::default();
    let hvm = OffchainLabs::new(config).unwrap();
    let program = create_test_bend_program();
    
    let optimized_program = hvm.optimize_program(&program).unwrap();
    assert_ne!(program.id(), optimized_program.id());
    
    let original_usage = hvm.estimate_program_resources(&program).unwrap();
    let optimized_usage = hvm.estimate_program_resources(&optimized_program).unwrap();
    
    assert!(optimized_usage.cpu_cycles <= original_usage.cpu_cycles);
    assert!(optimized_usage.memory_usage <= original_usage.memory_usage);
}