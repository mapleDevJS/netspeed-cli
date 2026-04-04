use std::fmt;

#[derive(Debug)]
pub enum SpeedtestError {
    NetworkError(String),
    ParseError(String),
    ServerNotFound(String),
    IoError(String),
    Custom(String),
}

impl fmt::Display for SpeedtestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeedtestError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SpeedtestError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SpeedtestError::ServerNotFound(msg) => write!(f, "Server not found: {}", msg),
            SpeedtestError::IoError(msg) => write!(f, "I/O error: {}", msg),
            SpeedtestError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SpeedtestError {}

impl From<reqwest::Error> for SpeedtestError {
    fn from(err: reqwest::Error) -> Self {
        SpeedtestError::NetworkError(err.to_string())
    }
}

impl From<std::io::Error> for SpeedtestError {
    fn from(err: std::io::Error) -> Self {
        SpeedtestError::IoError(err.to_string())
    }
}

impl From<quick_xml::Error> for SpeedtestError {
    fn from(err: quick_xml::Error) -> Self {
        SpeedtestError::ParseError(err.to_string())
    }
}

impl From<serde_json::Error> for SpeedtestError {
    fn from(err: serde_json::Error) -> Self {
        SpeedtestError::ParseError(err.to_string())
    }
}

impl From<quick_xml::de::DeError> for SpeedtestError {
    fn from(err: quick_xml::de::DeError) -> Self {
        SpeedtestError::ParseError(err.to_string())
    }
}

impl From<csv::Error> for SpeedtestError {
    fn from(err: csv::Error) -> Self {
        SpeedtestError::Custom(err.to_string())
    }
}
