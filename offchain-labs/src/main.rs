use offchain_labs::{Config, OffchainLabs};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::load()?;
    let mut hvm = OffchainLabs::new(config)?;

    info!("OffchainLabs initialized");

    // Add your main application logic here

    Ok(())
}