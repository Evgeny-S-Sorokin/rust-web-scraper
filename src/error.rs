use std::fmt;

#[derive(Debug)]
pub enum ScraperError {
    Network(reqwest::Error),
    Parse(String),
    Io(std::io::Error),
}

impl fmt::Display for ScraperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScraperError::Network(e) => write!(f, "Network error: {}", e),
            ScraperError::Parse(e) => write!(f, "Parse error: {}", e),
            ScraperError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ScraperError {}

impl From<reqwest::Error> for ScraperError {
    fn from(err: reqwest::Error) -> Self {
        ScraperError::Network(err)
    }
}

impl From<std::io::Error> for ScraperError {
    fn from(err: std::io::Error) -> Self {
        ScraperError::Io(err)
    }
}

impl From<String> for ScraperError {
    fn from(err: String) -> Self {
        ScraperError::Parse(err)
    }
}
