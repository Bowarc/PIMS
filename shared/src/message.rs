#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub enum PayloadMessage {
    Ping, Pong,
    Boot,
    Info(String),
    ScanUpdate(crate::data::ScanInfo),
    Eject,
    Exit,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub enum ServerMessage {
    Ping, Pong,
    ScanRequest(crate::data::ScanParams)
}

impl networking::Message for PayloadMessage {
    fn is_ping(&self) -> bool {
        matches!(self, Self::Ping)
    }
    fn is_pong(&self) -> bool {
        matches!(self, Self::Pong)
    }

    fn default_ping() -> Self {
        Self::Ping
    }
    fn default_pong() -> Self {
        Self::Pong
    }
}

impl networking::Message for ServerMessage {
    fn is_ping(&self) -> bool {
        matches!(self, Self::Ping)
    }
    fn is_pong(&self) -> bool {
        matches!(self, Self::Pong)
    }

    fn default_ping() -> Self {
        Self::Ping
    }
    fn default_pong() -> Self {
        Self::Pong
    }
}
