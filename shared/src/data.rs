#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct ScanInfo {
    pub progress: (usize, usize), // current, max
    pub value_size_b: u8, // Size of the searched value in bytes len
    pub found_addresses: Vec<u64>, // tested on a 64 bits machine, no guarantees for 32 bits systems 
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct ScanParams {
    pub value: Vec<u8>, // Vec of bytes
    pub start_addr: Option<u64>,
    pub end_addr: Option<u64>,
}
