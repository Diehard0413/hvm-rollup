use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, Mutex};
use web3::types::{H256, U256};
use web3::transports::Http;
use web3::Web3;
use log::{info, error, debug};
use reqwest::Client;
use sha3::{Digest, Keccak256};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Calldata {
    data: String,
    proof: String,
    nonce: U256,
}

#[derive(Debug)]
enum RelayerError {
    VerificationFailed,
    HashGenerationFailed,
    TransactionSendFailed(String),
    SequencerError(String),
}

impl std::fmt::Display for RelayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RelayerError::VerificationFailed => write!(f, "Calldata verification failed"),
            RelayerError::HashGenerationFailed => write!(f, "Failed to generate transaction hash"),
            RelayerError::TransactionSendFailed(e) => write!(f, "Failed to send transaction: {}", e),
            RelayerError::SequencerError(e) => write!(f, "Sequencer error: {}", e),
        }
    }
}

impl std::error::Error for RelayerError {}

struct Relayer {
    cache: Arc<Mutex<VecDeque<(H256, Calldata)>>>,
    web3: Web3<Http>,
    client: Client,
    sequencer_url: String,
}

impl Relayer {
    fn new(web3_url: &str, sequencer_url: &str) -> Result<Self, Box<dyn Error>> {
        let transport = Http::new(web3_url)?;
        let web3 = Web3::new(transport);
        let client = Client::new();
        Ok(Relayer {
            cache: Arc::new(Mutex::new(VecDeque::new())),
            web3,
            client,
            sequencer_url: sequencer_url.to_string(),
        })
    }

    async fn verify_calldata(&self, calldata: &Calldata) -> Result<bool, RelayerError> {
        if calldata.proof.len() < 32 {
            debug!("Proof length insufficient: {}", calldata.proof.len());
            return Ok(false);
        }

        let hash = Keccak256::digest(calldata.data.as_bytes());
        let hash_hex = format!("{:x}", hash);

        if !hash_hex.starts_with(&calldata.proof[..8]) {
            debug!("Proof mismatch: {} vs {}", hash_hex, calldata.proof);
            return Ok(false);
        }

        Ok(true)
    }

    async fn generate_tx_hash(&self, calldata: &Calldata) -> Result<H256, RelayerError> {
        if !self.verify_calldata(&calldata).await? {
            return Err(RelayerError::HashGenerationFailed);
        }

        let mut hasher = Keccak256::new();
        hasher.update(calldata.data.as_bytes());
        hasher.update(calldata.proof.as_bytes());
        hasher.update(calldata.nonce.to_string().as_bytes());

        let result = hasher.finalize();
        let hash = H256::from_slice(&result);

        Ok(hash)
    }

    async fn send_tx_hash_to_blockchain(&self, tx_hash: &H256) -> Result<(), RelayerError> {
        let accounts = self.web3.eth().accounts().await
            .map_err(|e| RelayerError::TransactionSendFailed(e.to_string()))?;

        if let Some(account) = accounts.get(0) {
            let tx = self.web3.eth().send_transaction(
                web3::types::TransactionRequest {
                    from: *account,
                    to: None,
                    gas: None,
                    gas_price: None,
                    value: None,
                    data: Some(web3::types::Bytes(tx_hash.as_bytes().to_vec())),
                    nonce: None,
                    condition: None,
                    transaction_type: None,
                    access_list: None,
                    max_fee_per_gas: None,
                    max_priority_fee_per_gas: None,
                }
            ).await;

            match tx {
                Ok(_) => {
                    info!("Transaction sent successfully with hash: {:?}", tx_hash);
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to send transaction: {:?}", e);
                    Err(RelayerError::TransactionSendFailed(e.to_string()))
                }
            }
        } else {
            error!("No accounts available to send transaction");
            Err(RelayerError::TransactionSendFailed("No accounts available".to_string()))
        }
    }

    async fn process_calldata(&self, calldata: Calldata) -> Result<(), RelayerError> {
        if !self.verify_calldata(&calldata).await? {
            return Err(RelayerError::VerificationFailed);
        }

        let tx_hash = self.generate_tx_hash(&calldata).await?;

        {
            let mut cache = self.cache.lock().unwrap();
            cache.push_back((tx_hash, calldata.clone()));
        }

        self.send_tx_hash_to_blockchain(&tx_hash).await?;

        Ok(())
    }

    async fn fetch_calldata_from_sequencer(&self) -> Result<Calldata, RelayerError> {
        let response = self.client.get(&self.sequencer_url).send().await
            .map_err(|e| RelayerError::SequencerError(e.to_string()))?;

        let calldata: Calldata = response.json().await
            .map_err(|e| RelayerError::SequencerError(e.to_string()))?;

        Ok(calldata)
    }

    fn handle_massive_overload(&self) {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() > 10000 {
            let removed = cache.pop_front();
            if let Some((hash, _)) = removed {
                debug!("Removed oldest entry from cache: {:?}", hash);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let relayer = Relayer::new("http://localhost:8545", "http://localhost:8080/sequencer")?;

    loop {
        match relayer.fetch_calldata_from_sequencer().await {
            Ok(calldata) => {
                info!("Received calldata from sequencer");
                match relayer.process_calldata(calldata).await {
                    Ok(_) => info!("Successfully processed calldata"),
                    Err(e) => error!("Error processing calldata: {:?}", e),
                }
            }
            Err(e) => {
                error!("Error fetching calldata: {:?}", e);
            }
        }

        relayer.handle_massive_overload();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}