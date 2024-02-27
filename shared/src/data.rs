#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct ScanInfo {
    pub progress: (usize, usize), // current, max
    pub value_size_b: u8, // Size of the searched value
    pub found_addresses: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct ScanParams {
    pub value: Vec<u8>,
    pub start_addr: Option<u8>,
    pub end_addr: Option<u8>,
}
