#[derive(Clone, Debug)]
pub struct Transaction {
    // Define transaction fields
    pub data: Vec<u8>,
}

impl Transaction {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}