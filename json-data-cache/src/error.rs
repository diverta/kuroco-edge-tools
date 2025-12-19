use std::{error::Error, fmt};

#[derive(Debug)]
pub struct JsonDataCacheError {
    pub msg: String,
}

impl fmt::Display for JsonDataCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[EdgeError] {}", self.msg)
    }
}

impl Error for JsonDataCacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<&str> for JsonDataCacheError {
    fn from(value: &str) -> Self {
        JsonDataCacheError {
            msg: value.to_owned(),
        }
    }
}

impl From<String> for JsonDataCacheError {
    fn from(msg: String) -> Self {
        JsonDataCacheError {
            msg,
        }
    }
}

impl From<aho_corasick::BuildError> for JsonDataCacheError {
    fn from(value: aho_corasick::BuildError) -> Self {
        format!("[AC] {}", value.to_string()).into()
    }
}

impl From<aho_corasick::MatchError> for JsonDataCacheError {
    fn from(value: aho_corasick::MatchError) -> Self {
        format!("[AC] {}", value.to_string()).into()
    }
}

impl From<std::io::Error> for JsonDataCacheError {
    fn from(value: std::io::Error) -> Self { 
        JsonDataCacheError {
            msg: value.to_string()
        }
    }
}   

impl Into<std::io::Error> for JsonDataCacheError {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, self)
    }
}   
