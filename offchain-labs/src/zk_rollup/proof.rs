#[derive(Debug, Clone)]
pub struct Proof {
    // Define proof fields
    pub data: Vec<u8>,
}

impl Proof {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}