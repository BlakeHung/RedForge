pub mod http_scanner;
pub mod ssl_scanner;
pub mod tech_detector;
pub mod vulnerability_scanner;
pub mod owasp_scanner;

use crate::models::*;
use std::error::Error;

#[derive(Debug)]
pub struct ScannerError {
    pub message: String,
}

impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scanner Error: {}", self.message)
    }
}

impl Error for ScannerError {}

pub type ScannerResult<T> = Result<T, Box<dyn Error>>;
